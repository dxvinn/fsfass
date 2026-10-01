extends Node2D
## Draws the world: terrain texture, plants/objects, creatures, effects.
## Reads render buffers from the Rust simulation; never changes it.
## All sprites are drawn procedurally with _draw calls.

const Art = preload("res://scripts/art.gd")

const TILE := 16.0
const C_STRIDE := 16
const O_STRIDE := 6
## Terrain texture pixels per tile (3200 x 2240 for the default 200 x 140 world).
const TERRAIN_SCALE := 16
## Static objects are cached per CHUNK x CHUNK tiles.
const CHUNK := 20

const WATER_SHADER := """
shader_type canvas_item;
// Terrain alpha: 1.0 = land; below 1.0 = water, lower = deeper (see render.rs).
uniform vec2 tiles = vec2(200.0, 140.0);

void fragment() {
	vec4 c = texture(TEXTURE, UV);
	float wv = (1.0 - c.a) * 255.0;
	float wm = clamp(wv, 0.0, 1.0);
	vec3 col = c.rgb;
	if (wm > 0.0) {
		float depth = clamp((wv - 1.0) / 50.0, 0.0, 1.0);
		vec2 p = UV * tiles;
		float t = TIME;
		// Two crossing wave trains, bent by each other: soft caustic ripples.
		vec2 q = p * 2.4 + vec2(sin(p.y * 0.55 + t * 0.35) + sin(p.y * 1.7 + p.x * 0.4), sin(p.x * 0.5 - t * 0.3) + sin(p.x * 1.5 - p.y * 0.5)) * 1.1;
		float s = (sin(q.x + t * 0.8) + sin(dot(q, vec2(-0.5, 0.866)) - t * 0.7) + sin(dot(q, vec2(-0.5, -0.866)) + t * 0.6)) / 3.0;
		// Fade the ripples out when zoomed far out, where they would alias into dots.
		float detail = 1.0 - smoothstep(0.04, 0.11, fwidth(p.x));
		float glint = smoothstep(0.45, 0.9, s) * (0.022 + 0.03 * depth) * detail;
		float dark = smoothstep(-0.35, -0.85, s) * 0.03 * detail;
		// Swell lines running toward the shore in the shallows.
		float shore = 1.0 - smoothstep(0.02, 0.3, depth);
		float wave = smoothstep(0.82, 1.0, sin(depth * 38.0 + t * 1.5 + sin(p.x * 0.7 + p.y * 0.5) * 2.0)) * shore * 0.10;
		col += (vec3(glint + wave) - vec3(dark * 0.6, dark * 0.4, dark * 0.2)) * wm;
	}
	// COLOR already holds the texture sample; replace it so alpha stays opaque.
	COLOR = vec4(col, 1.0);
}
"""

const OUTLINE := Color(0.12, 0.09, 0.07, 0.85)

var world  # GenesisWorld
var terrain_sprite: Sprite2D
var creatures := PackedFloat32Array()
var draw_pos := {}      # id -> Vector2 (smoothed, pixels)
var facing := {}        # id -> float
var moving := {}        # id -> float, 0..1 walking amount
var objects := PackedFloat32Array()
var hover_id := -1
var time := 0.0
var rain_drops: Array = []
var flashes: Array = []  # {pos, t}
var effects: Array = []  # {pos, t, color, text}
var weather := 0
var font: Font
var soft: Texture2D      # radial white -> transparent, for shadows and glows
var last_terrain_ms := 0.0
var _terrain_tick := -1
var _terrain_real := -1.0e9
var _terrain_pending := false
## Static objects (plants, stones, remains...) are drawn into canvas items, one
## per world chunk, and a chunk is only redrawn when its objects change. A frame
## then only pays for creatures and fires.
var chunks: Array[Node2D] = []
var chunk_sig := PackedFloat64Array()
var chunk_data: Array[PackedFloat32Array] = []
var chunk_cols := 1
var ci: CanvasItem       # where the object helpers draw: self or a chunk
var _lod := false
var _static_lod := false
var _static_timer := 0.0


func setup(w) -> void:
	world = w
	font = ThemeDB.fallback_font
	soft = _make_soft()
	terrain_sprite = Sprite2D.new()
	terrain_sprite.centered = false
	terrain_sprite.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS
	var sh := Shader.new()
	sh.code = WATER_SHADER
	var mat := ShaderMaterial.new()
	mat.shader = sh
	terrain_sprite.material = mat
	terrain_sprite.z_index = -10
	add_child(terrain_sprite)
	ci = self
	chunk_cols = ceili(world.width() / float(CHUNK))
	var rows := ceili(world.height() / float(CHUNK))
	for k in chunk_cols * rows:
		var layer := Node2D.new()
		layer.show_behind_parent = true
		add_child(layer)
		layer.draw.connect(_draw_chunk.bind(k))
		chunks.append(layer)
		chunk_data.append(PackedFloat32Array())
	chunk_sig.resize(chunks.size())
	chunk_sig.fill(-1.0)
	refresh_terrain()
	for i in 260:
		rain_drops.append(Vector2(randf(), randf()))


func _make_soft() -> Texture2D:
	var g := Gradient.new()
	g.set_color(0, Color(1, 1, 1, 1))
	g.set_color(1, Color(1, 1, 1, 0))
	g.add_point(0.45, Color(1, 1, 1, 0.55))
	var t := GradientTexture2D.new()
	t.gradient = g
	t.fill = GradientTexture2D.FILL_RADIAL
	t.fill_from = Vector2(0.5, 0.5)
	t.fill_to = Vector2(1.0, 0.5)
	t.width = 64
	t.height = 64
	return t


## Rebuild the terrain texture. Called for god powers and once per game hour.
## At high game speed the hourly calls are throttled so painting the terrain
## never takes more than about a tenth of real time.
func refresh_terrain() -> void:
	var now := Time.get_ticks_msec() / 1000.0
	var periodic: bool = _terrain_tick >= 0 and int(world.tick()) - _terrain_tick >= 3600
	if periodic and now - _terrain_real < _terrain_min_interval():
		_terrain_pending = true
		return
	_rebuild_terrain()


func _terrain_min_interval() -> float:
	return maxf(2.0, last_terrain_ms / 1000.0 * 10.0)


func _rebuild_terrain() -> void:
	_terrain_pending = false
	var scale_px := TERRAIN_SCALE
	var t0 := Time.get_ticks_usec()
	var bytes: PackedByteArray = world.terrain_image(scale_px)
	var w: int = world.width() * scale_px
	var h: int = world.height() * scale_px
	if bytes.size() != w * h * 4:
		push_error("terrain size mismatch")
		return
	var img := Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, bytes)
	img.generate_mipmaps()
	var tex := terrain_sprite.texture as ImageTexture
	if tex != null and tex.get_width() == w and tex.get_height() == h:
		tex.update(img)
	else:
		terrain_sprite.texture = ImageTexture.create_from_image(img)
	terrain_sprite.scale = Vector2.ONE * (TILE / scale_px)
	(terrain_sprite.material as ShaderMaterial).set_shader_parameter("tiles", Vector2(world.width(), world.height()))
	last_terrain_ms = (Time.get_ticks_usec() - t0) / 1000.0
	_terrain_tick = int(world.tick())
	_terrain_real = Time.get_ticks_msec() / 1000.0


func world_size_px() -> Vector2:
	return Vector2(world.width(), world.height()) * TILE


func sync(delta: float) -> void:
	time += delta
	creatures = world.creatures()
	objects = world.objects()
	weather = world.weather()
	if _terrain_pending and Time.get_ticks_msec() / 1000.0 - _terrain_real >= _terrain_min_interval():
		_rebuild_terrain()
	_static_timer -= delta
	if _static_timer <= 0.0:
		_static_timer = 0.4
		_update_chunks()
	var seen := {}
	var n := creatures.size() / C_STRIDE
	var k: float = clampf(delta * 10.0, 0.0, 1.0)
	for i in n:
		var b := i * C_STRIDE
		var id := int(creatures[b])
		seen[id] = true
		var target := Vector2(creatures[b + 2] + 0.5, creatures[b + 3] + 0.5) * TILE
		if draw_pos.has(id):
			var cur: Vector2 = draw_pos[id]
			if cur.distance_to(target) > TILE * 6.0:
				cur = target
			var nxt := cur.lerp(target, k)
			if nxt.distance_to(cur) > 0.3:
				facing[id] = signf(nxt.x - cur.x) if absf(nxt.x - cur.x) > 0.1 else facing.get(id, 1.0)
			var mv: float = moving.get(id, 0.0)
			var want := 1.0 if cur.distance_to(target) > 1.0 else 0.0
			moving[id] = move_toward(mv, want, delta * 6.0)
			draw_pos[id] = nxt
		else:
			draw_pos[id] = target
			facing[id] = 1.0 if Art.hash01(id, 9) < 0.5 else -1.0
	for id in draw_pos.keys():
		if not seen.has(id):
			draw_pos.erase(id)
			moving.erase(id)
			facing.erase(id)
	var fi := 0
	while fi < flashes.size():
		flashes[fi].t -= delta
		if flashes[fi].t <= 0:
			flashes.remove_at(fi)
		else:
			fi += 1
	var ei := 0
	while ei < effects.size():
		effects[ei].t -= delta
		if effects[ei].t <= 0:
			effects.remove_at(ei)
		else:
			ei += 1
	queue_redraw()


## Sort static objects into chunks and redraw the chunks whose contents changed
## (fires are excluded: their fuel changes every tick and they are drawn every
## frame anyway).
func _update_chunks() -> void:
	var nk := chunks.size()
	var sig := PackedFloat64Array()
	sig.resize(nk)
	sig.fill(0.0)
	var data: Array[PackedFloat32Array] = []
	for k in nk:
		data.append(PackedFloat32Array())
	var n := objects.size() / O_STRIDE
	for i in n:
		var b := i * O_STRIDE
		if int(objects[b + 1]) == 4:
			continue
		var k := clampi(int(objects[b + 3]) / CHUNK, 0, nk / chunk_cols - 1) * chunk_cols + clampi(int(objects[b + 2]) / CHUNK, 0, chunk_cols - 1)
		var id: float = objects[b]
		sig[k] += 1.0 + id * 1.618 + objects[b + 1] * 3.1 + objects[b + 2] * 0.37 + objects[b + 3] * 0.71 + objects[b + 4] * (7.3 + id * 0.001) + objects[b + 5] * 0.5
		data[k].append_array(objects.slice(b, b + O_STRIDE))
	for k in nk:
		if sig[k] != chunk_sig[k]:
			chunk_sig[k] = sig[k]
			chunk_data[k] = data[k]
			chunks[k].queue_redraw()


func add_flash(tile: Vector2i) -> void:
	flashes.append({"pos": (Vector2(tile) + Vector2(0.5, 0.5)) * TILE, "t": 0.6})


func add_effect(tile: Vector2i, color: Color, text: String) -> void:
	effects.append({"pos": (Vector2(tile) + Vector2(0.5, 0.5)) * TILE, "t": 1.6, "color": color, "text": text})


func pos_of(id: int) -> Vector2:
	return draw_pos.get(id, Vector2(-1, -1))


func _draw() -> void:
	if world == null:
		return
	var cam := get_viewport().get_camera_2d()
	var zoom := cam.zoom.x if cam else 1.0
	var view := get_viewport_rect()
	var center := cam.get_screen_center_position() if cam else view.size / 2.0
	var half := view.size / zoom / 2.0 + Vector2(TILE * 3, TILE * 3)
	var vis := Rect2(center - half, half * 2.0)
	ci = self
	_lod = zoom < 0.75
	if _lod != _static_lod:
		_static_lod = _lod
		for layer in chunks:
			layer.queue_redraw()
	_draw_fires(vis)
	_draw_creatures(vis, zoom)
	for f in flashes:
		var a: float = f.t / 0.6
		draw_texture_rect(soft, Rect2(f.pos - Vector2(70, 70), Vector2(140, 140)), false, Color(1, 0.95, 0.7, a * 0.6))
		draw_line(f.pos + Vector2(-10, -300), f.pos + Vector2(6, -140), Color(1, 1, 0.85, a), 4.0)
		draw_line(f.pos + Vector2(6, -140), f.pos + Vector2(-6, -60), Color(1, 1, 0.85, a), 4.0)
		draw_line(f.pos + Vector2(-6, -60), f.pos, Color(1, 1, 0.85, a), 4.0)
		draw_circle(f.pos, 40.0 * (1.0 - a) + 6.0, Color(1, 0.95, 0.6, a * 0.5))
	for e in effects:
		var a2: float = clampf(e.t / 1.6, 0.0, 1.0)
		var r := 10.0 + (1.0 - a2) * 30.0
		draw_arc(e.pos, r, 0, TAU, 40, Color(e.color, a2), 2.5)
		draw_string(font, e.pos + Vector2(-40, -r - 8 - (1.0 - a2) * 20.0), e.text, HORIZONTAL_ALIGNMENT_CENTER, 80, 13, Color(e.color, a2))
	if weather >= 2:
		_draw_rain(vis.intersection(Rect2(Vector2.ZERO, world_size_px())))


# ------------------------------------------------------------------ helpers

func _shadow(c: Vector2, r: Vector2, a := 0.32) -> void:
	ci.draw_texture_rect(soft, Rect2(c - r, r * 2.0), false, Color(0.05, 0.04, 0.03, a))


func _glow(c: Vector2, r: float, col: Color) -> void:
	ci.draw_texture_rect(soft, Rect2(c - Vector2(r, r), Vector2(r, r) * 2.0), false, col)


var _unit_circles := {}  # point count -> unit circle polygon


func _unit(n: int) -> PackedVector2Array:
	var u: PackedVector2Array = _unit_circles.get(n, PackedVector2Array())
	if u.is_empty():
		for i in n:
			var a := TAU * i / n
			u.append(Vector2(cos(a), sin(a)))
		_unit_circles[n] = u
	return u


## Native transform of a cached unit circle: no per-point script work.
func _ellipse(c: Vector2, r: Vector2, n := 14) -> PackedVector2Array:
	return Transform2D(0.0, r, 0.0, c) * _unit(n)


func _rot_ellipse(c: Vector2, r: Vector2, angle: float, n := 12) -> PackedVector2Array:
	return Transform2D(angle, r, 0.0, c) * _unit(n)


func _blob(pts: PackedVector2Array, col: Color, outline := OUTLINE, width := 1.0) -> void:
	ci.draw_colored_polygon(pts, col)
	if outline.a > 0.0:
		var closed := pts.duplicate()
		closed.append(pts[0])
		ci.draw_polyline(closed, outline, width, true)


func _ring(c: Vector2, r: float, col: Color, outline := OUTLINE, width := 1.0) -> void:
	ci.draw_circle(c, r, col)
	if outline.a > 0.0:
		ci.draw_arc(c, r, 0, TAU, 16, outline, width, true)


## Teardrop flame standing on `base`, tip leaning by `lean`.
func _flame(base: Vector2, w: float, h: float, lean: float) -> PackedVector2Array:
	return PackedVector2Array([
		base + Vector2(-w, 0), base + Vector2(-w * 0.85, -h * 0.35), base + Vector2(-w * 0.35 + lean * 0.5, -h * 0.7),
		base + Vector2(lean, -h), base + Vector2(w * 0.35 + lean * 0.5, -h * 0.7), base + Vector2(w * 0.85, -h * 0.35),
		base + Vector2(w, 0), base + Vector2(w * 0.5, w * 0.55), base + Vector2(-w * 0.5, w * 0.55)])


# ------------------------------------------------------------------ objects

func _draw_chunk(k: int) -> void:
	ci = chunks[k]
	var d: PackedFloat32Array = chunk_data[k]
	var lod := _static_lod
	var n := d.size() / O_STRIDE
	# Shadows first so no shadow ever falls on top of a neighbour's sprite.
	for i in n:
		var b := i * O_STRIDE
		var p := _obj_pos(d, b)
		match int(d[b + 1]):
			0: _shadow(p + Vector2(1.5, 4.5), Vector2(9, 4), 0.3)
			1: _shadow(p + Vector2(4, 6), Vector2(13, 5.5) * _tree_size(int(d[b])), 0.36)
			2: _shadow(p + Vector2(1.5, 3), Vector2(7, 3.5), 0.3)
			3: _shadow(p + Vector2(1, 2.5), Vector2(5.5, 2.5), 0.3)
			5: _shadow(p + Vector2(0, 3), Vector2(9, 3.5), 0.28)
			7: _shadow(p + Vector2(0, 2), Vector2(8, 3.5), 0.3)
	# Back to front, so lower trees overlap the ones behind them.
	var order := range(n)
	order.sort_custom(func(i: int, j: int) -> bool: return d[i * O_STRIDE + 3] < d[j * O_STRIDE + 3])
	for i in order:
		var b: int = i * O_STRIDE
		var p := _obj_pos(d, b)
		var amount: float = d[b + 4]
		var id := int(d[b])
		match int(d[b + 1]):
			0: _draw_bush(p, id, amount, lod)
			1: _draw_tree(p, id, lod)
			2: _draw_stone(p, id)
			3: _draw_flint(p, id)
			5: _draw_carcass(p, id, amount, d[b + 5] > 0.5)
			6: _draw_flake(p, id)
			7: _draw_remains(p, id)
	ci = self


func _draw_fires(vis: Rect2) -> void:
	var n := objects.size() / O_STRIDE
	for i in n:
		var b := i * O_STRIDE
		if int(objects[b + 1]) != 4:
			continue
		var p := _obj_pos(objects, b)
		if not vis.has_point(p):
			continue
		var fuel: float = objects[b + 4]
		_glow(p, 40.0 * _fire_size(fuel), Color(1.0, 0.55, 0.18, 0.20 + 0.04 * sin(time * 7.0 + objects[b])))
		_draw_fire(p, int(objects[b]), fuel)


func _obj_pos(d: PackedFloat32Array, b: int) -> Vector2:
	var id := int(d[b])
	var p := Vector2(d[b + 2] + 0.5, d[b + 3] + 0.5) * TILE
	return p + Vector2(float((id * 7919) % 7) - 3.0, float((id * 104729) % 7) - 3.0)


func _tree_size(id: int) -> float:
	return 0.85 + Art.hash01(id, 21) * 0.35


func _draw_bush(p: Vector2, id: int, amount: float, lod: bool) -> void:
	var dark := Color("#2f5e2c")
	var mid := Color("#447a37")
	var lite := Color("#5f9a48")
	if lod:
		ci.draw_circle(p, 7.0, mid)
		if amount > 0.0:
			ci.draw_circle(p + Vector2(1, -1), 3.0, Color("#d23c4a"))
		return
	var sx := 0.9 + Art.hash01(id, 5) * 0.25
	ci.draw_circle(p + Vector2(0, 1), 7.6 * sx, OUTLINE)
	ci.draw_circle(p + Vector2(0, 1), 6.8 * sx, dark)
	ci.draw_circle(p + Vector2(-3.4, -0.5) * sx, 4.3 * sx, mid)
	ci.draw_circle(p + Vector2(3.2, 0.2) * sx, 4.0 * sx, mid)
	ci.draw_circle(p + Vector2(0.2, -3.2) * sx, 4.4 * sx, mid)
	ci.draw_circle(p + Vector2(-1.6, -4.0) * sx, 2.4 * sx, lite)
	ci.draw_circle(p + Vector2(2.2, -1.6) * sx, 1.8 * sx, lite)
	if amount <= 0.0:
		return
	var berries := clampi(int(ceil(amount / 2.5)), 1, 8)
	for j in berries:
		var a := TAU * (j / 8.0) + Art.hash01(id, 6) * TAU
		var rr := 2.4 + Art.hash01(id, 30 + j) * 2.6
		var bp := p + Vector2(cos(a) * rr * 1.1, sin(a) * rr * 0.9 - 1.0) * sx
		ci.draw_circle(bp, 1.55, Color("#7a1626"))
		ci.draw_circle(bp, 1.2, Color("#d8344a"))
		ci.draw_circle(bp + Vector2(-0.45, -0.45), 0.5, Color(1, 0.85, 0.85, 0.9))


func _draw_tree(p: Vector2, id: int, lod: bool) -> void:
	var s := _tree_size(id)
	var kind := Art.hash01(id, 22)
	var tint := (Art.hash01(id, 23) - 0.5) * 0.16
	var sway := 0.0
	if lod:
		ci.draw_circle(p + Vector2(0, -5) * s, 9.0 * s, Color("#2c5a2c").lightened(tint))
		return
	var trunk := Color("#5c3e26")
	if kind < 0.3:
		# Conifer: stacked tiers, darker on the shadow side.
		ci.draw_rect(Rect2(p + Vector2(-1.5, 1) * s, Vector2(3, 6) * s), trunk)
		var base := Color("#24513a").lightened(tint)
		for t in 3:
			var y0 := (2.0 - t * 6.0) * s
			var wd := (9.5 - t * 2.4) * s
			var ht := 10.0 * s
			var tip := p + Vector2(sway * (t + 1) * 0.5, y0 - ht)
			var pts := PackedVector2Array([p + Vector2(-wd, y0), tip, p + Vector2(wd, y0)])
			_blob(pts, base.lightened(t * 0.07))
			ci.draw_colored_polygon(PackedVector2Array([tip, p + Vector2(wd, y0), p + Vector2(wd * 0.15, y0)]), Color(0, 0.05, 0.02, 0.22))
		return
	# Broadleaf: trunk, then dark, mid and lit canopy layers.
	var tk := PackedVector2Array([p + Vector2(-2.2, 7) * s, p + Vector2(-1.3, -3) * s, p + Vector2(1.3, -3) * s, p + Vector2(2.2, 7) * s])
	_blob(tk, trunk, OUTLINE, 0.8)
	ci.draw_line(p + Vector2(0.6, 6) * s, p + Vector2(0.5, -2) * s, trunk.darkened(0.3), 1.0)
	var c := p + Vector2(sway, -8.0 * s)
	var dark := Color("#29502a").lightened(tint)
	var mid := Color("#3b6e34").lightened(tint)
	var lite := Color("#5a9446").lightened(tint)
	if kind > 0.85:
		# Autumn-ish variety for a few trees.
		mid = Color("#6f7a33")
		lite = Color("#9aa048")
	ci.draw_circle(c + Vector2(-5, 2) * s, 6.8 * s, OUTLINE)
	ci.draw_circle(c + Vector2(5, 2) * s, 6.8 * s, OUTLINE)
	ci.draw_circle(c + Vector2(0, -4) * s, 7.6 * s, OUTLINE)
	ci.draw_circle(c + Vector2(-5, 2) * s, 6.0 * s, dark)
	ci.draw_circle(c + Vector2(5, 2) * s, 6.0 * s, dark)
	ci.draw_circle(c + Vector2(0, -4) * s, 6.8 * s, dark)
	ci.draw_circle(c + Vector2(0, 1) * s, 6.5 * s, dark)
	ci.draw_circle(c + Vector2(-3, -2) * s, 5.2 * s, mid)
	ci.draw_circle(c + Vector2(3, -1) * s, 4.4 * s, mid)
	ci.draw_circle(c + Vector2(-1, -5) * s, 4.6 * s, mid)
	ci.draw_circle(c + Vector2(-3.5, -5) * s, 2.6 * s, lite)
	ci.draw_circle(c + Vector2(0.5, -7) * s, 2.0 * s, lite)
	ci.draw_circle(c + Vector2(-5, -1.5) * s, 1.6 * s, lite)


func _draw_stone(p: Vector2, id: int) -> void:
	var pts := PackedVector2Array()
	var n := 7
	for i in n:
		var a := TAU * i / n + Art.hash01(id, 40) * 0.6
		var r := 4.4 + Art.hash01(id, 41 + i) * 1.8
		pts.append(p + Vector2(cos(a) * r * 1.2, sin(a) * r * 0.8))
	_blob(pts, Color("#8c8780"), OUTLINE, 1.0)
	ci.draw_colored_polygon(PackedVector2Array([pts[4], pts[5], pts[6], p + Vector2(0.5, -0.5)]), Color("#b2ada4"))
	ci.draw_line(p + Vector2(-2.5, 1.5), p + Vector2(1.5, 2.2), Color(0, 0, 0, 0.25), 0.8)


func _draw_flint(p: Vector2, id: int) -> void:
	var f := 1.0 if Art.hash01(id, 50) < 0.5 else -1.0
	var pts := PackedVector2Array([p + Vector2(-4.5 * f, 2), p + Vector2(-1 * f, -4.5), p + Vector2(4.5 * f, -1), p + Vector2(1.5 * f, 3.5)])
	_blob(pts, Color("#3a4252"), OUTLINE, 0.9)
	ci.draw_colored_polygon(PackedVector2Array([pts[1], pts[2], p + Vector2(0.3 * f, 0.2)]), Color("#62718a"))
	ci.draw_line(p + Vector2(-0.6 * f, -3.2), p + Vector2(2.6 * f, -1.2), Color(0.85, 0.92, 1.0, 0.75), 0.9)


func _fire_size(fuel: float) -> float:
	return clampf(0.85 + fuel / 40.0, 0.85, 1.6)


func _draw_fire(p: Vector2, id: int, amount: float) -> void:
	var size := _fire_size(amount)
	var fl := sin(time * 11.0 + id) * 0.5 + sin(time * 17.0 + id * 1.7) * 0.5
	_glow(p + Vector2(0, -3) * size, 16.0 * size, Color(1.0, 0.75, 0.3, 0.35 + fl * 0.05))
	# Hearth stones and logs.
	for i in 6:
		var a := TAU * i / 6.0 + 0.3
		ci.draw_circle(p + Vector2(cos(a) * 7.0, sin(a) * 3.6 + 2.0) * size, 1.7 * size, Color("#77736c"))
	ci.draw_line(p + Vector2(-5.5, 3.5) * size, p + Vector2(5, 0.5) * size, Color("#3d2616"), 2.6 * size)
	ci.draw_line(p + Vector2(-5, 0.5) * size, p + Vector2(5.5, 3.5) * size, Color("#4b2f1b"), 2.6 * size)
	ci.draw_circle(p + Vector2(0, 2) * size, 3.2 * size, Color(1.0, 0.45, 0.12, 0.8))
	# Flames: three tongues, each with an orange, yellow and white core.
	for t in 3:
		var ph := time * (8.0 + t * 1.7) + id + t * 2.1
		var off := Vector2((t - 1) * 3.0, 1.0) * size
		var h := (10.0 + 3.0 * sin(ph) - absf(t - 1) * 2.5) * size
		var lean := sin(ph * 0.7) * 1.6 * size
		var w := 3.2 * size
		ci.draw_colored_polygon(_flame(p + off, w, h, lean), Color(0.94, 0.38, 0.1, 0.92))
		ci.draw_colored_polygon(_flame(p + off + Vector2(0, 0.5), w * 0.65, h * 0.7, lean * 0.8), Color(1.0, 0.68, 0.18))
		ci.draw_colored_polygon(_flame(p + off + Vector2(0, 0.8), w * 0.32, h * 0.38, lean * 0.6), Color(1.0, 0.93, 0.62))
	# Sparks.
	for i in 3:
		var u := fmod(time * 0.9 + i * 0.37 + id * 0.13, 1.0)
		var sp := p + Vector2(sin(u * 9.0 + i * 2.0) * 3.0 + (i - 1) * 2.0, -8.0 - u * 18.0) * size
		ci.draw_circle(sp, 0.8, Color(1.0, 0.8, 0.35, 1.0 - u))


func _draw_carcass(p: Vector2, id: int, amount: float, cooked: bool) -> void:
	var hide := Color("#8a6040") if not cooked else Color("#6a4326")
	var meat := Color("#b0423c") if not cooked else Color("#8c4a24")
	var f := 1.0 if Art.hash01(id, 60) < 0.5 else -1.0
	# Lying on its side, legs out.
	ci.draw_line(p + Vector2(-3 * f, 2), p + Vector2(-5 * f, 6), hide.darkened(0.35), 1.4)
	ci.draw_line(p + Vector2(2 * f, 2), p + Vector2(4 * f, 6), hide.darkened(0.35), 1.4)
	_blob(_ellipse(p + Vector2(0, 0), Vector2(7, 3.4), 12), hide, OUTLINE, 0.9)
	_ring(p + Vector2(7.5 * f, -1.5), 2.4, hide, OUTLINE, 0.8)
	var eaten := clampf(1.0 - amount / 10.0, 0.0, 1.0)
	if eaten > 0.05:
		ci.draw_colored_polygon(_ellipse(p + Vector2(-1 * f, -0.5), Vector2(4.5, 2.2) * (0.5 + eaten * 0.5), 10), meat)
		for r in 3:
			var x := (-3.0 + r * 2.0) * f
			ci.draw_line(p + Vector2(x, -2), p + Vector2(x + 0.6 * f, 1.5), Color("#eee2cc"), 0.9)


func _draw_flake(p: Vector2, id: int) -> void:
	var gl := 0.6
	ci.draw_arc(p, 6.5, 0, TAU, 20, Color(UI.ACCENT, 0.35 + gl * 0.25), 1.0)
	var pts := PackedVector2Array([p + Vector2(-3.5, 1.5), p + Vector2(0.5, -3.5), p + Vector2(3.5, 0.5), p + Vector2(0, 2.5)])
	_blob(pts, Color("#d6dfeb"), Color(0.2, 0.24, 0.3, 0.9), 0.8)
	ci.draw_line(p + Vector2(-1.5, 0.5), p + Vector2(1.5, -2.0), Color(1, 1, 1, 0.6 + gl * 0.4), 0.8)


func _draw_remains(p: Vector2, id: int) -> void:
	var bone := Color("#e2dac8")
	var a := Art.hash01(id, 70) * 0.8 - 0.4
	for s in [-1.0, 1.0]:
		var d := Vector2(cos(a + s * 0.6), sin(a + s * 0.6)) * 5.0
		ci.draw_line(p - d, p + d, OUTLINE, 2.6)
		ci.draw_line(p - d, p + d, bone, 1.6)
		ci.draw_circle(p + d, 1.3, bone)
		ci.draw_circle(p - d, 1.3, bone)
	_ring(p + Vector2(0, -3.5), 2.8, bone, OUTLINE, 0.8)
	ci.draw_circle(p + Vector2(-1.0, -3.6), 0.7, Color("#3a3028"))
	ci.draw_circle(p + Vector2(1.0, -3.6), 0.7, Color("#3a3028"))


# ------------------------------------------------------------------ creatures

func _draw_creatures(vis: Rect2, zoom: float) -> void:
	var n := creatures.size() / C_STRIDE
	var show_names := zoom > 0.9
	var map_mode := zoom < 0.75
	if not map_mode:
		for i in n:
			var b := i * C_STRIDE
			var p: Vector2 = draw_pos.get(int(creatures[b]), Vector2.ZERO)
			if not vis.has_point(p):
				continue
			var kind := int(creatures[b + 1])
			if kind == 0:
				var asleep := creatures[b + 10] > 0.5
				var k := _human_scale(b)
				_shadow(p + Vector2(0, 7.0), Vector2(9.0 if asleep else 5.5, 2.6) * k, 0.38)
			else:
				var ks := _animal_scale(b)
				_shadow(p + Vector2(0, 6.5), Vector2(10.0, 3.0) * ks, 0.36)
	for i in n:
		var b := i * C_STRIDE
		var id := int(creatures[b])
		var p: Vector2 = draw_pos.get(id, Vector2.ZERO)
		if not vis.has_point(p):
			continue
		var kind := int(creatures[b + 1])
		var action := int(creatures[b + 6])
		var selected := creatures[b + 12] > 0.5
		var mv: float = moving.get(id, 0.0)
		var face: float = facing.get(id, 1.0)
		if map_mode:
			# Map mode: readable markers instead of tiny sprites.
			var mr := 4.5 / zoom
			var mc: Color = [Color("#f2c46a"), Color("#c9a27a"), Color("#d06a5a")][kind]
			draw_circle(p, mr + 1.5 / zoom, Color(0, 0, 0, 0.55))
			draw_circle(p, mr, mc)
			if selected:
				draw_arc(p, mr + 5.0 / zoom, 0, TAU, 32, UI.ACCENT, 2.0 / zoom)
			continue
		if selected:
			var pulse := 0.5 + 0.5 * sin(time * 4.0)
			draw_arc(p + Vector2(0, 5), 13.0 + pulse * 2.0, 0, TAU, 40, Color(UI.ACCENT, 0.9), 2.0)
		elif id == hover_id:
			draw_arc(p + Vector2(0, 5), 12.0, 0, TAU, 40, Color(1, 1, 1, 0.5), 1.5)
		match kind:
			0:
				_draw_human(p, b, id, action, mv, face)
			1:
				_draw_grazer(p, b, id, action, mv, face)
			2:
				_draw_wolf(p, b, id, action, mv, face)
		if kind == 0 and (show_names or selected):
			_draw_action_icon(p + Vector2(10, -24), action)
		if creatures[b + 9] < 0.5:
			var hw := 14.0
			draw_rect(Rect2(p + Vector2(-hw / 2, 11), Vector2(hw, 2.5)), Color(0, 0, 0, 0.6))
			draw_rect(Rect2(p + Vector2(-hw / 2, 11), Vector2(hw * creatures[b + 9], 2.5)), UI.BAD)


func _human_scale(b: int) -> float:
	var age: float = creatures[b + 7]
	var height_t: float = creatures[b + 15]
	return clampf(0.5 + age / 16.0 * 0.5, 0.5, 1.0) * (0.92 + height_t * 0.16) * 1.15


func _animal_scale(b: int) -> float:
	var age: float = creatures[b + 7]
	return clampf(0.6 + age / 2.0 * 0.4, 0.6, 1.0)


func _draw_human(p: Vector2, b: int, id: int, action: int, mv: float, face: float) -> void:
	var age: float = creatures[b + 7]
	var female := creatures[b + 8] > 0.5
	var asleep := creatures[b + 10] > 0.5
	var pregnant := creatures[b + 11] > 0.5
	var k := _human_scale(b)
	var child := age < 13.0
	var elder := age >= 55.0
	var skin := Art.skin_color(creatures[b + 13])
	var hair := Art.hair_color(creatures[b + 14], age)
	if absf(hair.get_luminance() - skin.get_luminance()) < 0.15:
		# Keep the hair readable against similar skin.
		hair = hair.darkened(0.35) if skin.get_luminance() > 0.25 else hair.lightened(0.25)
	var cloth := Art.cloth_color(id)
	var style := Art.hair_style(id, female, age)
	# Children have relatively big heads.
	var hr := 3.5 * k * (1.0 + (1.0 - clampf(age / 16.0, 0.0, 1.0)) * 0.45)
	var line := skin.darkened(0.5)
	line.a = 0.9
	draw_set_transform(p, 0.0, Vector2(face, 1.0))
	if asleep:
		_draw_sleeper(k, hr, skin, hair, cloth, style, line)
		draw_set_transform_matrix(Transform2D.IDENTITY)
		var z := fmod(time + id * 0.37, 2.4) / 2.4
		draw_string(font, p + Vector2(5 + z * 4, -6 - z * 12), "z", HORIZONTAL_ALIGNMENT_LEFT, -1, 9 + int(z * 4), Color(1, 1, 1, 0.9 * (1.0 - z)))
		return
	var ph := time * 11.0 + id
	var swing := sin(ph) * 2.2 * k * mv
	var bob := absf(sin(ph)) * 1.0 * mv
	if action == 5:
		swing *= 1.5
		bob *= 1.6
	var crouch := 2.5 * k if action == 3 else 0.0
	var y := -bob + crouch
	var lean := 1.2 * k if elder else 0.0
	var hip := Vector2(0, -1.0 * k + y)
	var foot_y := 6.5 * k
	var leg := skin.darkened(0.12)
	# Legs.
	for sgn in [-1.0, 1.0]:
		var top := hip + Vector2(sgn * 1.4 * k, 0)
		var ft := Vector2(sgn * 1.6 * k + swing * sgn, foot_y)
		if crouch > 0.0:
			ft = Vector2(sgn * 2.4 * k + 1.5 * k, foot_y)
		draw_line(top, ft, line, 2.6 * k)
		draw_line(top, ft, leg, 1.8 * k)
		draw_circle(ft + Vector2(0.5 * k, 0), 1.1 * k, Color("#3b2a1e"))
	var sh := Vector2(lean * 0.5, -8.0 * k + y)
	var hem := 1.5 * k if not female else 3.0 * k
	# Back arm.
	var back_hand := sh + Vector2(-2.6 * k - swing * 0.8, 6.5 * k)
	draw_line(sh + Vector2(-2.4 * k, 0.8 * k), back_hand, skin.darkened(0.2), 1.7 * k)
	# Tunic.
	var tunic := PackedVector2Array([
		sh + Vector2(-3.0 * k, 0), sh + Vector2(3.0 * k, 0),
		Vector2(3.8 * k, hem + y), Vector2(1.2 * k, hem + y + 0.8 * k), Vector2(-1.4 * k, hem + y + 0.4 * k), Vector2(-3.8 * k, hem + y)])
	_blob(tunic, cloth, cloth.darkened(0.55), 0.9)
	draw_line(Vector2(-3.4 * k, -2.0 * k + y), Vector2(3.4 * k, -2.0 * k + y), cloth.darkened(0.35), 1.1 * k)
	draw_colored_polygon(PackedVector2Array([sh + Vector2(-3.0 * k, 0), sh + Vector2(-0.5 * k, 0), Vector2(-1.5 * k, hem + y), Vector2(-3.8 * k, hem + y)]), Color(1, 1, 1, 0.08))
	if pregnant:
		draw_colored_polygon(_ellipse(Vector2(2.6 * k, -2.6 * k + y), Vector2(2.4, 2.8) * k, 12), cloth.lightened(0.08))
		draw_arc(Vector2(2.6 * k, -2.6 * k + y), 2.6 * k, -1.3, 1.3, 8, cloth.darkened(0.55), 0.9)
	# Front arm, posed by action.
	var hand := sh + Vector2(2.6 * k + swing * 0.8, 6.5 * k)
	match action:
		6: hand = sh + Vector2(8.0 * k, 1.5 * k)
		2: hand = sh + Vector2(3.2 * k, -2.0 * k)
		3: hand = Vector2(6.0 * k, 4.0 * k)
		5: hand = sh + Vector2(-1.0 * k, -4.0 * k)
		7: hand = sh + Vector2(1.5 * k, -4.5 * k)
		8: hand = sh + Vector2(7.0 * k, -2.0 * k)
	draw_line(sh + Vector2(2.4 * k, 0.8 * k), hand, line, 2.5 * k)
	draw_line(sh + Vector2(2.4 * k, 0.8 * k), hand, skin, 1.7 * k)
	draw_circle(hand, 1.1 * k, skin)
	if elder and age >= 62.0 and action != 6:
		draw_line(hand + Vector2(0.5, -1.5), Vector2(hand.x + 1.5, foot_y + 0.5), Color("#6b4a2c"), 1.2)
	# Head.
	var head := sh + Vector2(lean + 0.3 * k, -hr - 0.6 * k)
	if action == 3:
		head += Vector2(1.5 * k, 1.5 * k)
	if style == 1:
		# Long hair falls behind the shoulders.
		draw_colored_polygon(PackedVector2Array([head + Vector2(-hr * 1.05, -hr * 0.2), head + Vector2(hr * 0.2, -hr * 0.6),
			head + Vector2(hr * 0.1, hr * 1.9), head + Vector2(-hr * 1.25, hr * 1.7)]), hair.darkened(0.12))
	draw_circle(head, hr + 0.7, line)
	draw_circle(head, hr, skin)
	# Hair cap (covers back and top of the head; the face looks toward +x).
	var cap := PackedVector2Array()
	for i in 10:
		var a := PI * 0.8 + i / 9.0 * PI * 1.05
		cap.append(head + Vector2(cos(a), sin(a)) * (hr + 0.45))
	cap.append(head + Vector2(hr * 0.45, -hr * 0.5))
	cap.append(head + Vector2(-hr * 0.1, -hr * 0.42))
	cap.append(head + Vector2(-hr * 0.45, hr * 0.1))
	cap.append(head + Vector2(-hr * 0.7, hr * 0.55))
	draw_colored_polygon(cap, hair)
	draw_polyline(cap.slice(9), hair.darkened(0.4), 0.7, true)
	draw_arc(head + Vector2(-hr * 0.1, -hr * 0.1), hr * 0.85, PI * 1.15, PI * 1.55, 5, hair.lightened(0.25), maxf(0.6, hr * 0.18))
	if style == 2:
		draw_circle(head + Vector2(-hr * 0.75, -hr * 0.85), hr * 0.45, hair)
	if Art.has_beard(id, female, age):
		draw_colored_polygon(PackedVector2Array([head + Vector2(-hr * 0.1, hr * 0.35), head + Vector2(hr * 0.95, hr * 0.3),
			head + Vector2(hr * 0.55, hr * 1.15), head + Vector2(hr * 0.0, hr * 0.95)]), hair)
	# Face.
	var eye := head + Vector2(hr * 0.5, -hr * 0.05)
	if action == 7:
		draw_line(eye + Vector2(-0.6, -0.4), eye + Vector2(0.6, 0.4), Color("#1b1410"), 0.8)
	else:
		draw_circle(eye, maxf(0.65 * k, 0.55), Color("#1b1410"))
	draw_circle(head + Vector2(hr * 0.98, hr * 0.15), 0.7 * k, skin.darkened(0.12))
	if child:
		draw_circle(head + Vector2(hr * 0.35, hr * 0.45), 0.9 * k, Color(0.95, 0.45, 0.4, 0.35))
	draw_set_transform_matrix(Transform2D.IDENTITY)


func _draw_sleeper(k: float, hr: float, skin: Color, hair: Color, cloth: Color, style: int, line: Color) -> void:
	# A bed of grass, the body lying along x, head to the left.
	draw_colored_polygon(_ellipse(Vector2(0, 5.0), Vector2(10.0, 3.6) * k, 14), Color("#7b8a45"))
	draw_colored_polygon(_ellipse(Vector2(0.5, 4.2), Vector2(8.5, 2.6) * k, 14), Color("#97a04e"))
	_blob(_ellipse(Vector2(1.5 * k, 2.8), Vector2(6.5, 2.8) * k, 14), cloth, cloth.darkened(0.55), 0.9)
	draw_line(Vector2(-1.0 * k, 1.0), Vector2(5.5 * k, 1.5), cloth.lightened(0.12), 1.0)
	var head := Vector2(-6.0 * k, 2.2)
	if style == 1:
		draw_colored_polygon(_ellipse(head + Vector2(-hr * 0.6, hr * 0.2), Vector2(hr * 1.3, hr * 0.8), 10), hair.darkened(0.1))
	draw_circle(head, hr + 0.6, line)
	draw_circle(head, hr * 0.98, skin)
	draw_arc(head, hr + 0.3, PI * 0.55, PI * 1.45, 10, hair, hr * 0.7)
	draw_line(head + Vector2(hr * 0.15, -hr * 0.15), head + Vector2(hr * 0.55, -hr * 0.1), Color("#1b1410"), 0.7)


func _draw_grazer(p: Vector2, b: int, id: int, action: int, mv: float, face: float) -> void:
	var age: float = creatures[b + 7]
	var male := creatures[b + 8] < 0.5
	var k := _animal_scale(b)
	var fur := Color("#b27a48").lightened((Art.hash01(id, 80) - 0.5) * 0.18)
	var belly := Color("#efe0c4")
	var line := Color(0.18, 0.11, 0.06, 0.9)
	var ph := time * (13.0 if action == 5 else 9.0) + id
	var bob := absf(sin(ph)) * (2.4 if action == 5 else 0.8) * mv
	var y := -bob
	draw_set_transform(p, 0.0, Vector2(face, 1.0))
	# Legs: far pair darker, near pair lighter, gait in diagonal pairs.
	var legc := fur.darkened(0.45)
	for j in 4:
		var lx: float = [-4.6, -2.6, 3.0, 5.0][j]
		var sw := sin(ph + (PI if j % 2 == 0 else 0.0)) * 1.6 * mv
		var top := Vector2(lx * k, -0.5 * k + y)
		var ft := Vector2(lx * k + sw, 6.5)
		var c := legc if j % 2 == 0 else fur.darkened(0.25)
		draw_line(top, ft, c, 1.3 * k)
		draw_circle(ft, 0.7, Color("#2a1d14"))
	# Tail flash.
	draw_colored_polygon(_ellipse(Vector2(-7.3 * k, -3.6 * k + y), Vector2(1.4, 1.9) * k, 8), Color("#f4ece0"))
	# Body.
	_blob(_ellipse(Vector2(0, -2.3 * k + y), Vector2(7.2, 3.5) * k, 16), fur, line, 0.9)
	draw_colored_polygon(_ellipse(Vector2(0.3 * k, -0.6 * k + y), Vector2(5.2, 1.5) * k, 12), belly)
	draw_colored_polygon(_ellipse(Vector2(-0.5 * k, -4.4 * k + y), Vector2(5.0, 1.0) * k, 12), fur.darkened(0.18))
	if age < 1.0:
		for j in 5:
			draw_circle(Vector2((-4.0 + j * 1.8) * k, (-3.2 + (j % 2) * 1.0) * k + y), 0.6 * k, Color(1, 1, 1, 0.8))
	# Neck and head; head down while eating or drinking.
	var down := action == 2 or action == 3
	var head := Vector2(8.6 * k, (2.6 if down else -8.0) * k + y)
	var neck := PackedVector2Array([Vector2(4.0 * k, -4.6 * k + y), Vector2(6.4 * k, -2.0 * k + y), head + Vector2(0.6 * k, 0.8 * k), head + Vector2(-1.6 * k, -0.6 * k)])
	_blob(neck, fur, line, 0.8)
	var hd := PackedVector2Array([head + Vector2(-2.0, -1.6) * k, head + Vector2(1.2, -1.8) * k, head + Vector2(3.6, -0.2) * k,
		head + Vector2(3.4, 0.9) * k, head + Vector2(0.4, 1.6) * k, head + Vector2(-1.8, 1.0) * k])
	_blob(hd, fur, line, 0.8)
	draw_circle(head + Vector2(3.5, 0.3) * k, 0.7 * k, Color("#1d1410"))
	draw_circle(head + Vector2(0.6, -0.6) * k, 0.55 * k, Color("#1d1410"))
	# Ear.
	draw_colored_polygon(PackedVector2Array([head + Vector2(-1.2, -1.4) * k, head + Vector2(-3.6, -3.4) * k, head + Vector2(-0.4, -2.2) * k]), fur.darkened(0.2))
	if male and age >= 1.0:
		var ac := Color("#6a5034")
		var base := head + Vector2(-0.6, -1.8) * k
		var tip := base + Vector2(-1.5, -5.5) * k
		draw_line(base, tip, ac, 1.0)
		draw_line(base + Vector2(-0.6, -2.5) * k, base + Vector2(1.2, -4.0) * k, ac, 0.9)
		draw_line(tip, tip + Vector2(-1.6, -0.6) * k, ac, 0.9)
	draw_set_transform_matrix(Transform2D.IDENTITY)


func _draw_wolf(p: Vector2, b: int, id: int, action: int, mv: float, face: float) -> void:
	var k := _animal_scale(b) * 1.05
	var fur := Color("#767d86").lightened((Art.hash01(id, 90) - 0.5) * 0.15)
	var saddle := fur.darkened(0.32)
	var belly := Color("#d3cbbb")
	var line := Color(0.08, 0.08, 0.1, 0.9)
	var hunting := action == 8
	var ph := time * (12.0 if hunting or action == 5 else 8.0) + id
	var bob := absf(sin(ph)) * (1.6 if hunting else 0.8) * mv
	var y := -bob + (1.0 if hunting else 0.0)
	draw_set_transform(p, 0.0, Vector2(face, 1.0))
	for j in 4:
		var lx: float = [-5.4, -3.2, 3.2, 5.4][j]
		var sw := sin(ph + (PI if j % 2 == 0 else 0.0)) * 1.8 * mv
		var c := fur.darkened(0.45) if j % 2 == 0 else fur.darkened(0.2)
		draw_line(Vector2(lx * k, -0.5 * k + y), Vector2(lx * k + sw, 6.5), c, 1.7 * k)
		draw_circle(Vector2(lx * k + sw, 6.5), 0.9, Color("#26272b"))
	# Bushy tail: straight back when hunting, low otherwise.
	var tail_ang := PI - 0.15 if hunting else PI - 0.75 + sin(time * 3.0 + id) * 0.08
	var tail_dir := Vector2(cos(tail_ang), sin(tail_ang))
	var tail_c := Vector2(-7.0 * k, -3.4 * k + y) + tail_dir * 4.2 * k
	_blob(_rot_ellipse(tail_c, Vector2(4.8, 1.9) * k, tail_ang), fur, line, 0.8)
	draw_circle(tail_c + tail_dir * 3.6 * k, 1.4 * k, fur.darkened(0.45))
	_blob(_ellipse(Vector2(0, -2.2 * k + y), Vector2(8.0, 3.6) * k, 16), fur, line, 0.9)
	draw_colored_polygon(_ellipse(Vector2(0.8 * k, -0.4 * k + y), Vector2(5.4, 1.4) * k, 12), belly)
	draw_colored_polygon(_ellipse(Vector2(-1.5 * k, -4.2 * k + y), Vector2(5.5, 1.5) * k, 12), saddle)
	var head := Vector2(8.6 * k, (-3.2 if hunting else -6.2) * k + y)
	# Ruff and head.
	draw_colored_polygon(_ellipse(Vector2(5.6 * k, -3.4 * k + y), Vector2(2.8, 3.4) * k, 10), fur.lightened(0.06))
	var hd := PackedVector2Array([head + Vector2(-2.6, -2.2) * k, head + Vector2(1.4, -2.0) * k, head + Vector2(5.2, -0.4) * k,
		head + Vector2(5.0, 0.8) * k, head + Vector2(1.0, 2.0) * k, head + Vector2(-2.4, 1.6) * k])
	_blob(hd, fur, line, 0.9)
	draw_colored_polygon(PackedVector2Array([head + Vector2(1.6, 0.6) * k, head + Vector2(5.0, 0.6) * k, head + Vector2(1.2, 1.8) * k]), belly)
	for ex in [-1.8, 0.2]:
		draw_colored_polygon(PackedVector2Array([head + Vector2(ex - 0.9, -1.8) * k, head + Vector2(ex - 0.2, -5.2) * k, head + Vector2(ex + 1.1, -1.9) * k]), saddle)
	draw_circle(head + Vector2(5.1, 0.1) * k, 0.8 * k, Color("#111214"))
	var eye := Color("#f2c84a") if not hunting else Color("#ff5a3a")
	draw_circle(head + Vector2(1.3, -0.7) * k, 0.75 * k, eye)
	if hunting:
		draw_line(head + Vector2(3.0, 1.2) * k, head + Vector2(4.6, 1.0) * k, Color(1, 1, 1, 0.8), 0.6)
	draw_set_transform_matrix(Transform2D.IDENTITY)


func _draw_action_icon(p: Vector2, action: int) -> void:
	var col := Color(0, 0, 0, 0)
	match action:
		2: col = Color("#d2434f")
		3: col = UI.COOL
		5: col = Color("#f0a04a")
		6: col = Color("#e8b04b")
		7: col = UI.BAD
		8: col = UI.BAD
		9: col = Color("#b48be8")
		_: return
	draw_circle(p, 4.5, Color(0, 0, 0, 0.55))
	draw_circle(p, 3.2, col)
	if action == 7:
		draw_string(font, p + Vector2(-2, 4), "!", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color.WHITE)


func _draw_rain(vis: Rect2) -> void:
	var a := 0.35 if weather == 2 else 0.55
	var c := Color(0.75, 0.85, 1.0, a)
	var dir := Vector2(-4, 14) if weather == 2 else Vector2(-9, 16)
	for d in rain_drops:
		var x: float = vis.position.x + fmod(d.x * vis.size.x + time * 40.0, vis.size.x)
		var y: float = vis.position.y + fmod(d.y * vis.size.y + time * 520.0, vis.size.y)
		draw_line(Vector2(x, y), Vector2(x, y) + dir, c, 1.0)
