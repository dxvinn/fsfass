extends Control
## Three-generation family tree: grandparents, parents, the person and partner, siblings,
## children. Click a living relative to select them.

signal person_clicked(id: int)

var boxes: Array = []  # {rect, id, alive}
var links: Array = []  # [Vector2, Vector2]
var font: Font


func _ready() -> void:
	font = ThemeDB.fallback_font
	mouse_filter = Control.MOUSE_FILTER_STOP


func set_family(me: Dictionary, fam: Dictionary) -> void:
	boxes.clear()
	links.clear()
	var w := maxf(size.x, custom_minimum_size.x)
	var rows := [70.0, 140.0, 220.0]
	var self_p := {"id": me.get("id", -1), "name": me.get("name", "?"), "alive": me.get("alive", true), "age": me.get("age", 0), "me": true}
	var grand: Array = fam.get("grandparents", [])
	var parents := []
	for k in ["mother", "father"]:
		if fam.get(k) is Dictionary:
			parents.append(fam[k])
	var mid := [self_p]
	if fam.get("partner") is Dictionary:
		mid.append(fam.partner)
	for s in fam.get("siblings", []):
		if mid.size() < 5:
			mid.append(s)
	var kids: Array = fam.get("children", []).slice(0, 6)
	var g_rects := _place(grand, 8.0, w)
	var p_rects := _place(parents, rows[0], w)
	var m_rects := _place(mid, rows[1], w)
	var k_rects := _place(kids, rows[2], w)
	for g in g_rects:
		for p in p_rects:
			links.append([g.get_center() + Vector2(0, 14), p.get_center() - Vector2(0, 14)])
	if p_rects.size() > 0:
		for i in mid.size():
			if i == 1 and fam.get("partner") is Dictionary:
				continue
			links.append([p_rects[0].get_center() + Vector2(0, 14), m_rects[i].get_center() - Vector2(0, 14)])
	for kr in k_rects:
		links.append([m_rects[0].get_center() + Vector2(0, 14), kr.get_center() - Vector2(0, 14)])
	if mid.size() > 1 and fam.get("partner") is Dictionary:
		links.append([m_rects[0].get_center() + Vector2(m_rects[0].size.x / 2, 0), m_rects[1].get_center() - Vector2(m_rects[1].size.x / 2, 0)])
	queue_redraw()


func _place(people: Array, y: float, w: float) -> Array:
	var rects := []
	var n := people.size()
	if n == 0:
		return rects
	var bw := minf(110.0, (w - 10.0) / n - 6.0)
	var total := n * (bw + 6.0) - 6.0
	var x0 := (w - total) / 2.0
	for i in n:
		var r := Rect2(Vector2(x0 + i * (bw + 6.0), y), Vector2(bw, 46))
		rects.append(r)
		var p: Dictionary = people[i]
		boxes.append({"rect": r, "id": int(p.get("id", -1)), "alive": p.get("alive", true), "name": p.get("name", "?"), "age": int(p.get("age", 0)), "me": p.get("me", false)})
	return rects


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		for b in boxes:
			if b.rect.has_point(event.position) and b.alive and not b.me:
				person_clicked.emit(b.id)


func _draw() -> void:
	if boxes.size() <= 1:
		draw_string(font, Vector2(10, 30), "No known family yet.", HORIZONTAL_ALIGNMENT_LEFT, -1, 13, UI.MUTED)
	for l in links:
		draw_line(l[0], l[1], UI.LINE, 2.0)
	for b in boxes:
		var col := UI.PANEL_HI if not b.me else UI.ACCENT.darkened(0.45)
		draw_style_box(UI.box(col, 8, UI.ACCENT if b.me else UI.LINE, 1, 4), b.rect)
		var tc := UI.TEXT if b.alive else UI.MUTED
		var name: String = ("† " if not b.alive else "") + b.name
		draw_string(font, b.rect.position + Vector2(6, 19), name, HORIZONTAL_ALIGNMENT_LEFT, b.rect.size.x - 10, 13, tc)
		draw_string(font, b.rect.position + Vector2(6, 37), "%d yrs" % b.age, HORIZONTAL_ALIGNMENT_LEFT, b.rect.size.x - 10, 11, UI.MUTED)
