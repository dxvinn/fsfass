extends Node2D
## Draws the world: terrain texture, plants/objects, creatures, effects.
## Reads render buffers from the Rust simulation; never changes it.

const TILE := 16.0
const C_STRIDE := 16
const O_STRIDE := 6

var world  # GenesisWorld
var terrain_sprite: Sprite2D
var creatures := PackedFloat32Array()
var prev_pos := {}      # id -> Vector2 (tile coords, last frame)
var draw_pos := {}      # id -> Vector2 (smoothed, pixels)
var facing := {}        # id -> float
var objects := PackedFloat32Array()
var hover_id := -1
var time := 0.0
var rain_drops: Array = []
var flashes: Array = []  # {pos, t}
var effects: Array = []  # {pos, t, color, text}
var weather := 0
var font: Font


func setup(w) -> void:
	world = w
	font = ThemeDB.fallback_font
	terrain_sprite = Sprite2D.new()
	terrain_sprite.centered = false
	terrain_sprite.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	terrain_sprite.z_index = -10
	add_child(terrain_sprite)
	refresh_terrain()
	for i in 260:
		rain_drops.append(Vector2(randf(), randf()))


func refresh_terrain() -> void:
	var scale_px := 8
	var bytes: PackedByteArray = world.terrain_image(scale_px)
	var w: int = world.width() * scale_px
	var h: int = world.height() * scale_px
	if bytes.size() != w * h * 4:
		push_error("terrain size mismatch")
		return
	var img := Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, bytes)
	terrain_sprite.texture = ImageTexture.create_from_image(img)
	terrain_sprite.scale = Vector2.ONE * (TILE / scale_px)


func world_size_px() -> Vector2:
	return Vector2(world.width(), world.height()) * TILE


func sync(delta: float) -> void:
	time += delta
	creatures = world.creatures()
	objects = world.objects()
	weather = world.weather()
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
			draw_pos[id] = nxt
		else:
			draw_pos[id] = target
	for id in draw_pos.keys():
		if not seen.has(id):
			draw_pos.erase(id)
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
	_draw_objects(vis, zoom)
	_draw_creatures(vis, zoom)
	for f in flashes:
		var a: float = f.t / 0.6
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


func _draw_objects(vis: Rect2, zoom: float) -> void:
	var n := objects.size() / O_STRIDE
	for i in n:
		var b := i * O_STRIDE
		var p := Vector2(objects[b + 2] + 0.5, objects[b + 3] + 0.5) * TILE
		if not vis.has_point(p):
			continue
		var kind := int(objects[b + 1])
		var amount: float = objects[b + 4]
		var seed := int(objects[b])
		var jitter := Vector2(float((seed * 7919) % 7) - 3.0, float((seed * 104729) % 7) - 3.0)
		p += jitter
		match kind:
			0:  # berry bush
				draw_circle(p + Vector2(0, 2), 7.5, Color(0, 0, 0, 0.18))
				draw_circle(p, 7.0, Color("#3f7a3a"))
				draw_circle(p + Vector2(-3, -2), 4.5, Color("#4f9147"))
				var berries := mini(int(amount), 6)
				for j in berries:
					var a := TAU * j / 6.0 + seed
					draw_circle(p + Vector2(cos(a), sin(a)) * 4.0, 1.7, Color("#d2434f"))
			1:  # tree
				draw_circle(p + Vector2(3, 5), 11.0, Color(0, 0, 0, 0.22))
				draw_rect(Rect2(p + Vector2(-1.5, 0), Vector2(3, 7)), Color("#5a3d26"))
				draw_circle(p + Vector2(0, -4), 10.0, Color("#2e5e2c"))
				draw_circle(p + Vector2(-3, -7), 6.5, Color("#3c7536"))
				draw_circle(p + Vector2(3, -2), 5.0, Color("#356b31"))
			2:  # stone
				draw_circle(p + Vector2(1, 2), 5.0, Color(0, 0, 0, 0.2))
				draw_colored_polygon(PackedVector2Array([p + Vector2(-5, 2), p + Vector2(-3, -3), p + Vector2(3, -4), p + Vector2(6, 1), p + Vector2(2, 4)]), Color("#8d8a84"))
				draw_line(p + Vector2(-3, -3), p + Vector2(3, -4), Color("#b3afa6"), 1.2)
			3:  # flint
				draw_colored_polygon(PackedVector2Array([p + Vector2(-4, 2), p + Vector2(0, -5), p + Vector2(5, 0), p + Vector2(1, 4)]), Color("#3d4450"))
				draw_line(p + Vector2(0, -5), p + Vector2(5, 0), Color("#8fa0b8"), 1.2)
			4:  # fire
				var f := 1.0 + 0.15 * sin(time * 13.0 + seed)
				var size := clampf(amount / 20.0, 0.6, 1.6)
				draw_circle(p, 22.0 * size, Color(1.0, 0.55, 0.15, 0.10))
				draw_circle(p, 12.0 * size, Color(1.0, 0.6, 0.2, 0.18))
				draw_colored_polygon(PackedVector2Array([p + Vector2(-6, 4) * size, p + Vector2(0, -12 * f) * size, p + Vector2(6, 4) * size]), Color("#f0782a"))
				draw_colored_polygon(PackedVector2Array([p + Vector2(-3, 4) * size, p + Vector2(0, -7 * f) * size, p + Vector2(3, 4) * size]), Color("#ffd35a"))
			5:  # carcass
				var col := Color("#8a4a3a") if objects[b + 5] < 0.5 else Color("#6b4a2a")
				draw_circle(p, 6.0, Color(0, 0, 0, 0.2))
				draw_rect(Rect2(p + Vector2(-6, -3), Vector2(12, 6)), col)
				draw_line(p + Vector2(-4, -1), p + Vector2(4, -1), Color("#e7d9c0"), 1.0)
			6:  # flake (made by a human)
				draw_colored_polygon(PackedVector2Array([p + Vector2(-3, 1), p + Vector2(0, -3), p + Vector2(3, 1)]), Color("#cfd8e6"))
				draw_arc(p, 6.0, 0, TAU, 16, Color(UI.ACCENT, 0.6), 1.0)
			7:  # remains
				draw_line(p + Vector2(-5, -2), p + Vector2(5, 2), Color("#d8d0bf"), 1.6)
				draw_line(p + Vector2(-4, 3), p + Vector2(4, -3), Color("#d8d0bf"), 1.6)


func _draw_creatures(vis: Rect2, zoom: float) -> void:
	var n := creatures.size() / C_STRIDE
	var show_names := zoom > 0.9
	for i in n:
		var b := i * C_STRIDE
		var id := int(creatures[b])
		var p: Vector2 = draw_pos.get(id, Vector2.ZERO)
		if not vis.has_point(p):
			continue
		var kind := int(creatures[b + 1])
		var action := int(creatures[b + 6])
		var selected := creatures[b + 12] > 0.5
		var bob := 0.0
		if action == 1 or action == 5 or action == 8:
			bob = absf(sin(time * 12.0 + id)) * 1.6
		var face: float = facing.get(id, 1.0)
		if zoom < 0.75:
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
			draw_arc(p + Vector2(0, 3), 13.0 + pulse * 2.0, 0, TAU, 40, Color(UI.ACCENT, 0.9), 2.0)
		elif id == hover_id:
			draw_arc(p + Vector2(0, 3), 12.0, 0, TAU, 40, Color(1, 1, 1, 0.5), 1.5)
		match kind:
			0:
				_draw_human(p, b, action, bob, face)
			1:
				_draw_grazer(p, b, action, bob, face)
			2:
				_draw_wolf(p, b, action, bob, face)
		if kind == 0 and (show_names or selected):
			_draw_action_icon(p + Vector2(9, -20), action)
		if creatures[b + 9] < 0.5:
			var hw := 14.0
			draw_rect(Rect2(p + Vector2(-hw / 2, 10), Vector2(hw, 2.5)), Color(0, 0, 0, 0.6))
			draw_rect(Rect2(p + Vector2(-hw / 2, 10), Vector2(hw * creatures[b + 9], 2.5)), UI.BAD)


func _draw_human(p: Vector2, b: int, action: int, bob: float, face: float) -> void:
	var age: float = creatures[b + 7]
	var female := creatures[b + 8] > 0.5
	var asleep := creatures[b + 10] > 0.5
	var pregnant := creatures[b + 11] > 0.5
	var skin_t: float = creatures[b + 13]
	var hair_t: float = creatures[b + 14]
	var height_t: float = creatures[b + 15]
	var s := clampf(0.45 + age / 18.0 * 0.55, 0.45, 1.0) * (0.92 + height_t * 0.16) * 1.3
	var skin := Color("#f1c9a5").lerp(Color("#5b3a26"), skin_t)
	var hair := Color("#e8d18a").lerp(Color("#1d1410"), hair_t)
	if age > 55.0:
		hair = hair.lerp(Color("#d9d9d9"), clampf((age - 55.0) / 15.0, 0.0, 0.9))
	var cloth := Color("#c98f5a") if female else Color("#7a8f5a")
	draw_set_transform(p, 0.0, Vector2(face, 1.0))
	draw_circle(Vector2(0, 7), 6.0 * s, Color(0, 0, 0, 0.25))
	if asleep:
		draw_colored_polygon(_ellipse(Vector2(0, 3), Vector2(9, 4) * s), cloth)
		draw_circle(Vector2(-7 * s, 2), 3.5 * s, skin)
		draw_set_transform_matrix(Transform2D.IDENTITY)
		var z := fmod(time, 2.0) / 2.0
		draw_string(font, p + Vector2(4, -8 - z * 8), "z", HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(1, 1, 1, 1.0 - z))
		return
	var y := -bob
	# legs
	draw_line(Vector2(-2 * s, 2 + y), Vector2(-2.5 * s, 7), Color("#3b2f28"), 2.0 * s)
	draw_line(Vector2(2 * s, 2 + y), Vector2(2.5 * s, 7), Color("#3b2f28"), 2.0 * s)
	# body
	var body := Rect2(Vector2(-4 * s, -6 * s + y), Vector2(8 * s, 9 * s))
	draw_rect(body, cloth)
	if pregnant:
		draw_circle(Vector2(3 * s, -1 * s + y), 3.0 * s, cloth.lightened(0.1))
	# arms
	var arm_end := Vector2(5 * s, 1 * s + y)
	if action == 6 or action == 2:
		arm_end = Vector2(8 * s, -2 * s + y)
	elif action == 5:
		arm_end = Vector2(6 * s, -8 * s + y)
	draw_line(Vector2(3 * s, -5 * s + y), arm_end, skin, 2.0 * s)
	draw_line(Vector2(-3 * s, -5 * s + y), Vector2(-5 * s, 1 * s + y), skin, 2.0 * s)
	# head
	var head := Vector2(0, -10 * s + y)
	draw_circle(head, 4.2 * s, skin)
	draw_arc(head + Vector2(0, -0.5), 4.3 * s, PI, TAU, 12, hair, 2.6 * s)
	if female:
		draw_line(head + Vector2(-3.5 * s, 0), head + Vector2(-4 * s, 5 * s), hair, 2.0 * s)
	draw_circle(head + Vector2(2 * s, -0.5 * s), 0.8, Color("#1b1b1b"))
	draw_set_transform_matrix(Transform2D.IDENTITY)


func _draw_grazer(p: Vector2, b: int, action: int, bob: float, face: float) -> void:
	var age: float = creatures[b + 7]
	var s := clampf(0.55 + age / 3.0 * 0.45, 0.55, 1.0)
	draw_set_transform(p, 0.0, Vector2(face, 1.0))
	draw_colored_polygon(_ellipse(Vector2(0, 6), Vector2(9, 3) * s), Color(0, 0, 0, 0.22))
	var y := -bob
	var fur := Color("#b98a5a")
	for lx in [-5.0, -2.0, 3.0, 6.0]:
		draw_line(Vector2(lx * s, 1 + y), Vector2(lx * s, 6), Color("#6b4e33"), 1.6)
	draw_colored_polygon(_ellipse(Vector2(0, -1 + y), Vector2(8, 4.5) * s), fur)
	draw_colored_polygon(_ellipse(Vector2(0, 0.5 + y), Vector2(6, 2.2) * s), Color("#e3d0b4"))
	var hy := -4.0 if action != 2 else 2.0
	draw_circle(Vector2(8 * s, hy * s + y), 3.2 * s, fur)
	draw_line(Vector2(8 * s, (hy - 3) * s + y), Vector2(9.5 * s, (hy - 6) * s + y), Color("#4a3a2a"), 1.2)
	draw_circle(Vector2(9 * s, (hy - 0.5) * s + y), 0.7, Color("#1b1b1b"))
	draw_set_transform_matrix(Transform2D.IDENTITY)


func _draw_wolf(p: Vector2, b: int, action: int, bob: float, face: float) -> void:
	draw_set_transform(p, 0.0, Vector2(face, 1.0))
	draw_colored_polygon(_ellipse(Vector2(0, 6), Vector2(10, 3)), Color(0, 0, 0, 0.25))
	var y := -bob
	var fur := Color("#6d7178")
	for lx in [-6.0, -3.0, 3.0, 6.0]:
		draw_line(Vector2(lx, 1 + y), Vector2(lx, 6), Color("#3c3f44"), 1.8)
	draw_colored_polygon(_ellipse(Vector2(0, -1 + y), Vector2(9, 4)), fur)
	draw_line(Vector2(-8, -2 + y), Vector2(-13, -5 + y), fur, 2.5)
	draw_colored_polygon(PackedVector2Array([Vector2(7, -6 + y), Vector2(14, -2 + y), Vector2(7, 1 + y)]), fur.darkened(0.1))
	draw_colored_polygon(PackedVector2Array([Vector2(7, -6 + y), Vector2(8, -10 + y), Vector2(10, -5 + y)]), fur.darkened(0.25))
	var eye := Color("#f2d27a") if action != 8 else Color("#ff5a4a")
	draw_circle(Vector2(10, -4 + y), 0.9, eye)
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


func _ellipse(c: Vector2, r: Vector2, n := 14) -> PackedVector2Array:
	var pts := PackedVector2Array()
	for i in n:
		var a := TAU * i / n
		pts.append(c + Vector2(cos(a) * r.x, sin(a) * r.y))
	return pts
