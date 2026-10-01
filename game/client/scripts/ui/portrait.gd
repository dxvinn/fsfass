extends Control
## Small painted portrait for the inspector header, derived from genetics.
## Uses the same colours as the world sprites (see scripts/art.gd).

const Art = preload("res://scripts/art.gd")
const R := 27.0

var d := {}


func _ready() -> void:
	custom_minimum_size = Vector2(56, 56)


func set_data(data: Dictionary) -> void:
	d = data
	queue_redraw()


func _draw() -> void:
	var c := size / 2.0
	var alive: bool = d.get("alive", true)
	# Backdrop: sky over a soft hill, inside the frame.
	draw_circle(c, R, Color("#2c3a48"))
	draw_colored_polygon(_disc_below(c, R - 0.5, 6.0), Color("#34503a"))
	draw_circle(c + Vector2(-9, -11), 9.0, Color(1, 1, 1, 0.05))
	var kind: String = d.get("kind", "Human")
	var g: Dictionary = d.get("genetics", {})
	match kind:
		"Human":
			_human(c, g)
		"Grazer":
			_grazer(c)
		_:
			_wolf(c)
	if not alive:
		draw_circle(c, R, Color(0.1, 0.1, 0.12, 0.45))
	draw_arc(c, R, 0, TAU, 48, UI.ACCENT if alive else UI.MUTED, 2.0, true)


## Points of the part of a circle below y = c.y + top (for clipping shoulders).
func _disc_below(c: Vector2, r: float, top: float, bump := 0.0) -> PackedVector2Array:
	var pts := PackedVector2Array()
	var a0 := asin(clampf(top / r, -1.0, 1.0))
	for i in 17:
		var a := lerpf(a0, PI - a0, i / 16.0)
		pts.append(c + Vector2(cos(a), sin(a)) * r)
	if bump > 0.0:
		# Rounded shoulders rising toward the neck.
		var x0 := cos(a0) * r
		for i in 9:
			var t := i / 8.0
			var x := lerpf(-x0, x0, t)
			pts.append(c + Vector2(x, top - sin(t * PI) * bump))
	return pts


func _human(c: Vector2, g: Dictionary) -> void:
	var id := int(d.get("id", 0))
	var age := float(d.get("age", 20))
	var female: bool = d.get("sex", "") == "female"
	var skin := Art.skin_color(float(g.get("skin", 0.5)))
	var hair := Art.hair_color(float(g.get("hair", 0.5)), age)
	var eye := Color("#5b8fd0").lerp(Color("#6b8a4a"), clampf(float(g.get("eye", 0.5)) * 2.0, 0.0, 1.0)).lerp(Color("#3a2618"), clampf(float(g.get("eye", 0.5)) * 2.0 - 1.0, 0.0, 1.0))
	var cloth := Art.cloth_color(id)
	var style := Art.hair_style(id, female, age)
	var line := skin.darkened(0.5)
	var young := clampf(1.0 - age / 16.0, 0.0, 1.0)
	var hr := 10.5 + young * 1.5
	var head := c + Vector2(0, -3.0 + young * 2.0)
	var width := 1.0 + (float(g.get("build", 0.5)) - 0.5) * 0.2
	if style == 1:
		# Long hair behind the head and shoulders.
		draw_colored_polygon(PackedVector2Array([head + Vector2(-hr - 1.5, -2), head + Vector2(hr + 1.5, -2),
			head + Vector2(hr + 2.5, hr + 9), head + Vector2(-hr - 2.5, hr + 9)]), hair.darkened(0.15))
	# Shoulders in the tunic, neck.
	draw_colored_polygon(_disc_below(c, R - 0.5, 13.0, 5.0 * width), cloth)
	draw_line(c + Vector2(-6, 9), c + Vector2(0, 14), cloth.darkened(0.35), 1.5)
	draw_line(c + Vector2(6, 9), c + Vector2(0, 14), cloth.darkened(0.35), 1.5)
	draw_rect(Rect2(head + Vector2(-3.5, hr - 3.5), Vector2(7, 6)), skin.darkened(0.08))
	# Head.
	draw_circle(head, hr + 1.0, line)
	draw_circle(head, hr, skin)
	draw_circle(head + Vector2(-hr - 0.2, 1.5), 2.2, skin.darkened(0.06))
	draw_circle(head + Vector2(hr + 0.2, 1.5), 2.2, skin.darkened(0.06))
	# Hair cap with a parting.
	var cap := PackedVector2Array()
	for i in 13:
		var a := PI * 1.02 + i / 12.0 * PI * 0.96
		cap.append(head + Vector2(cos(a), sin(a)) * (hr + 1.2))
	cap.append(head + Vector2(hr * 0.75, -hr * 0.35))
	cap.append(head + Vector2(hr * 0.15, -hr * 0.5))
	cap.append(head + Vector2(-hr * 0.4, -hr * 0.42))
	cap.append(head + Vector2(-hr * 0.85, -hr * 0.1))
	draw_colored_polygon(cap, hair)
	draw_arc(head + Vector2(-2, -2), hr * 0.8, PI * 1.2, PI * 1.5, 6, hair.lightened(0.25), 1.6)
	if style == 2:
		draw_circle(head + Vector2(0, -hr - 2.5), 4.0, hair)
	if style == 1:
		draw_line(head + Vector2(-hr - 0.5, -2), head + Vector2(-hr - 1.0, hr * 0.6), hair, 3.0)
		draw_line(head + Vector2(hr + 0.5, -2), head + Vector2(hr + 1.0, hr * 0.6), hair, 3.0)
	# Face.
	var ey := head.y + 1.0
	for sx in [-1.0, 1.0]:
		var e := Vector2(head.x + sx * 4.0, ey)
		draw_circle(e, 2.0, Color("#f6f1e8"))
		draw_circle(e + Vector2(0.3, 0.2), 1.3, eye)
		draw_circle(e + Vector2(0.3, 0.2), 0.6, Color("#120c08"))
		draw_line(e + Vector2(-2.2, -3.0), e + Vector2(2.0, -3.4 + sx * 0.2), hair.darkened(0.2), 1.2)
	draw_line(head + Vector2(0, 2.5), head + Vector2(-0.8, 5.0), skin.darkened(0.25), 1.0)
	var smile := 0.6 if d.get("alive", true) else -0.1
	draw_arc(head + Vector2(0, 5.0 - smile), 2.8, 0.35 * PI - smile * 0.2, 0.65 * PI + smile * 0.2, 8, Color("#8a4a3a"), 1.2)
	if young > 0.3:
		draw_circle(head + Vector2(-6.0, 4.5), 1.8, Color(0.95, 0.45, 0.4, 0.3))
		draw_circle(head + Vector2(6.0, 4.5), 1.8, Color(0.95, 0.45, 0.4, 0.3))
	if age >= 50.0:
		var wr := skin.darkened(0.3)
		draw_line(head + Vector2(-7.5, 0.5), head + Vector2(-6.2, 1.8), wr, 0.8)
		draw_line(head + Vector2(7.5, 0.5), head + Vector2(6.2, 1.8), wr, 0.8)
		draw_line(head + Vector2(-3, -5.8), head + Vector2(3, -5.8), wr, 0.7)
	if Art.has_beard(id, female, age):
		var bd := PackedVector2Array()
		for i in 9:
			var a := lerpf(0.05 * PI, 0.95 * PI, i / 8.0)
			bd.append(head + Vector2(cos(a) * (hr + 0.5), sin(a) * (hr + 2.0) + 1.0))
		bd.append(head + Vector2(-hr * 0.5, 4.0))
		bd.append(head + Vector2(-2.5, 7.5))
		bd.append(head + Vector2(2.5, 7.5))
		bd.append(head + Vector2(hr * 0.5, 4.0))
		draw_colored_polygon(bd, hair)


func _grazer(c: Vector2) -> void:
	var fur := Color("#b27a48")
	var line := Color(0.18, 0.11, 0.06, 0.9)
	var male: bool = d.get("sex", "") == "male"
	var adult := float(d.get("age", 2)) >= 1.0
	draw_colored_polygon(_disc_below(c, R - 0.5, 12.0, 4.0), fur.darkened(0.1))
	if male and adult:
		var ac := Color("#6a5034")
		for sx in [-1.0, 1.0]:
			var base := c + Vector2(sx * 5, -12)
			var tip := base + Vector2(sx * 7, -11)
			draw_line(base, tip, ac, 2.2)
			draw_line(base + Vector2(sx * 3, -5), base + Vector2(sx * 0, -10), ac, 1.6)
			draw_line(tip, tip + Vector2(sx * 3, 2), ac, 1.6)
	for sx in [-1.0, 1.0]:
		var ear := PackedVector2Array([c + Vector2(sx * 6, -9), c + Vector2(sx * 17, -14), c + Vector2(sx * 9, -4)])
		draw_colored_polygon(ear, fur)
		draw_colored_polygon(PackedVector2Array([c + Vector2(sx * 8, -8.5), c + Vector2(sx * 14.5, -12.5), c + Vector2(sx * 9.5, -6)]), Color("#e9c9a8"))
	var head := PackedVector2Array([c + Vector2(-9, -10), c + Vector2(9, -10), c + Vector2(10, 0), c + Vector2(5, 12), c + Vector2(-5, 12), c + Vector2(-10, 0)])
	draw_colored_polygon(head, fur)
	var closed := head.duplicate()
	closed.append(head[0])
	draw_polyline(closed, line, 1.2, true)
	draw_colored_polygon(PackedVector2Array([c + Vector2(-4.5, 5), c + Vector2(4.5, 5), c + Vector2(4, 12), c + Vector2(-4, 12)]), Color("#efe0c4"))
	draw_circle(c + Vector2(0, 10), 2.6, Color("#2a1d14"))
	for sx in [-1.0, 1.0]:
		draw_circle(c + Vector2(sx * 5.5, -2), 2.0, Color("#1d1410"))
		draw_circle(c + Vector2(sx * 5.5 - 0.6, -2.6), 0.6, Color(1, 1, 1, 0.8))


func _wolf(c: Vector2) -> void:
	var fur := Color("#767d86")
	var saddle := fur.darkened(0.32)
	var line := Color(0.08, 0.08, 0.1, 0.9)
	draw_colored_polygon(_disc_below(c, R - 0.5, 11.0, 5.0), fur.lightened(0.05))
	for sx in [-1.0, 1.0]:
		draw_colored_polygon(PackedVector2Array([c + Vector2(sx * 4, -9), c + Vector2(sx * 11, -21), c + Vector2(sx * 13, -5)]), saddle)
		draw_colored_polygon(PackedVector2Array([c + Vector2(sx * 6, -9), c + Vector2(sx * 10.5, -17), c + Vector2(sx * 11, -7)]), Color("#c9b8a8"))
	var head := PackedVector2Array([c + Vector2(-13, -6), c + Vector2(-8, -12), c + Vector2(8, -12), c + Vector2(13, -6),
		c + Vector2(9, 4), c + Vector2(4, 13), c + Vector2(-4, 13), c + Vector2(-9, 4)])
	draw_colored_polygon(head, fur)
	var closed := head.duplicate()
	closed.append(head[0])
	draw_polyline(closed, line, 1.2, true)
	draw_colored_polygon(PackedVector2Array([c + Vector2(-8, -12), c + Vector2(8, -12), c + Vector2(0, -2)]), saddle)
	draw_colored_polygon(PackedVector2Array([c + Vector2(-5, 2), c + Vector2(5, 2), c + Vector2(4, 13), c + Vector2(-4, 13)]), Color("#d3cbbb"))
	draw_circle(c + Vector2(0, 9), 2.6, Color("#111214"))
	for sx in [-1.0, 1.0]:
		draw_colored_polygon(PackedVector2Array([c + Vector2(sx * 2.5, -3), c + Vector2(sx * 8.5, -4.5), c + Vector2(sx * 6, -1)]), Color("#f2c84a"))
		draw_circle(c + Vector2(sx * 5.6, -2.9), 0.9, Color("#111214"))
