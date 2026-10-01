extends Control
## Small painted portrait for the inspector header, derived from genetics.

var d := {}


func _ready() -> void:
	custom_minimum_size = Vector2(56, 56)


func set_data(data: Dictionary) -> void:
	d = data
	queue_redraw()


func _draw() -> void:
	var c := size / 2.0
	draw_circle(c, 27.0, UI.PANEL_HI)
	draw_arc(c, 27.0, 0, TAU, 48, UI.ACCENT if d.get("alive", true) else UI.MUTED, 2.0)
	var kind: String = d.get("kind", "Human")
	var g: Dictionary = d.get("genetics", {})
	if kind == "Human":
		var skin := Color("#f1c9a5").lerp(Color("#5b3a26"), float(g.get("skin", 0.5)))
		var hair := Color("#e8d18a").lerp(Color("#1d1410"), float(g.get("hair", 0.5)))
		if float(d.get("age", 20)) > 55.0:
			hair = hair.lerp(Color("#d9d9d9"), 0.7)
		var eye := Color("#5b8fd0").lerp(Color("#3a2618"), float(g.get("eye", 0.5)))
		draw_rect(Rect2(c + Vector2(-13, 10), Vector2(26, 14)), Color("#c98f5a") if d.get("sex", "") == "female" else Color("#7a8f5a"))
		draw_circle(c + Vector2(0, -2), 12.0, skin)
		draw_arc(c + Vector2(0, -3), 12.5, PI * 1.05, TAU - 0.15, 20, hair, 6.0)
		if d.get("sex", "") == "female":
			draw_line(c + Vector2(-11, -4), c + Vector2(-12, 10), hair, 4.0)
			draw_line(c + Vector2(11, -4), c + Vector2(12, 10), hair, 4.0)
		draw_circle(c + Vector2(-4, -2), 1.6, eye)
		draw_circle(c + Vector2(4, -2), 1.6, eye)
		draw_arc(c + Vector2(0, 4), 3.0, 0.3, PI - 0.3, 8, skin.darkened(0.35), 1.2)
	elif kind == "Grazer":
		draw_circle(c, 13.0, Color("#b98a5a"))
		draw_line(c + Vector2(-6, -10), c + Vector2(-9, -20), Color("#4a3a2a"), 2.0)
		draw_line(c + Vector2(6, -10), c + Vector2(9, -20), Color("#4a3a2a"), 2.0)
		draw_circle(c + Vector2(-4, -2), 1.6, Color.BLACK)
		draw_circle(c + Vector2(4, -2), 1.6, Color.BLACK)
	else:
		draw_circle(c, 13.0, Color("#6d7178"))
		draw_colored_polygon(PackedVector2Array([c + Vector2(-12, -6), c + Vector2(-8, -20), c + Vector2(-3, -9)]), Color("#55585e"))
		draw_colored_polygon(PackedVector2Array([c + Vector2(12, -6), c + Vector2(8, -20), c + Vector2(3, -9)]), Color("#55585e"))
		draw_circle(c + Vector2(-4, -2), 1.8, Color("#f2d27a"))
		draw_circle(c + Vector2(4, -2), 1.8, Color("#f2d27a"))
