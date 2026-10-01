extends Control
## Brain visualisation: the active pathway behind the current decision, laid out in
## columns perception -> cue -> outcome/memory -> emotion -> goal -> action.
## Edge thickness is the connection strength; pulses travel along active edges.

const COLUMNS := ["perception", "cue", "outcome", "memory", "emotion", "goal", "action"]
const COL_OF := {"perception": 0, "cue": 1, "outcome": 2, "memory": 2, "emotion": 3, "goal": 4, "action": 5}
const NCOL := 6

var nodes: Array = []
var edges: Array = []
var pos := {}
var t := 0.0
var font: Font
var hovered := ""


func _ready() -> void:
	font = ThemeDB.fallback_font
	custom_minimum_size = Vector2(300, 360)
	mouse_filter = Control.MOUSE_FILTER_PASS


func set_graph(g) -> void:
	if typeof(g) != TYPE_DICTIONARY:
		nodes = []
		edges = []
	else:
		nodes = g.get("nodes", [])
		edges = g.get("edges", [])
	_layout()
	queue_redraw()


func _layout() -> void:
	pos.clear()
	var cols := {}
	for n in nodes:
		var c: int = COL_OF.get(n.layer, 2)
		if not cols.has(c):
			cols[c] = []
		cols[c].append(n.id)
	var w := size.x
	var h := size.y
	for c in cols:
		var ids: Array = cols[c]
		for i in ids.size():
			var x := 24.0 + (w - 48.0) * float(c) / float(NCOL - 1)
			var y := 26.0 + (h - 40.0) * (float(i) + 0.5) / float(ids.size())
			pos[ids[i]] = Vector2(x, y)


func _notification(what: int) -> void:
	if what == NOTIFICATION_RESIZED:
		_layout()


func _process(delta: float) -> void:
	if is_visible_in_tree() and nodes.size() > 0:
		t += delta
		queue_redraw()


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var best := ""
		for id in pos:
			if (pos[id] as Vector2).distance_to(event.position) < 14.0:
				best = id
		if best != hovered:
			hovered = best
			queue_redraw()


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Color(0, 0, 0, 0.18))
	if nodes.is_empty():
		draw_string(font, Vector2(16, size.y / 2), "No active thought right now.", HORIZONTAL_ALIGNMENT_LEFT, size.x - 32, 13, UI.MUTED)
		return
	var headers := ["SENSE", "LEARNED CUE", "EXPECT / RECALL", "FEELING", "GOAL", "ACT"]
	for c in NCOL:
		var x := 24.0 + (size.x - 48.0) * float(c) / float(NCOL - 1)
		draw_string(font, Vector2(x - 40, 12), headers[c], HORIZONTAL_ALIGNMENT_CENTER, 80, 9, Color(UI.MUTED, 0.8))
	for e in edges:
		if not pos.has(e.from) or not pos.has(e.to):
			continue
		var a: Vector2 = pos[e.from]
		var b: Vector2 = pos[e.to]
		var w: float = clampf(absf(float(e.w)), 0.05, 1.0)
		var hl: bool = hovered != "" and (e.from == hovered or e.to == hovered)
		var col := Color(0.85, 0.82, 0.72, 0.15 + w * 0.5)
		if hl:
			col = Color(UI.ACCENT, 0.95)
		var mid := (a + b) / 2.0 + Vector2(0, -12)
		var pts := PackedVector2Array()
		for i in 13:
			var u := i / 12.0
			pts.append(a.lerp(mid, u).lerp(mid.lerp(b, u), u))
		draw_polyline(pts, col, 1.0 + w * 3.5, true)
		var ph := fmod(t * (0.6 + w) + float(hash(e.from + e.to) % 100) / 100.0, 1.0)
		var idx := int(ph * 12.0)
		draw_circle(pts[idx], 1.5 + w * 2.0, Color(1, 0.95, 0.75, 0.8))
	for n in nodes:
		var p: Vector2 = pos.get(n.id, Vector2.ZERO)
		var col: Color = UI.LAYER_COLORS.get(n.layer, UI.MUTED)
		var r := 8.0 if n.layer != "action" else 11.0
		draw_circle(p, r + 3.0, Color(col, 0.18 + 0.1 * sin(t * 3.0 + p.y)))
		draw_circle(p, r, col)
		draw_circle(p, r - 3.0, col.lightened(0.35))
	# Labels fit their column; hovering a node shows its full text on top.
	var colw := (size.x - 48.0) / float(NCOL - 1)
	var hover_node = null
	var per_col := {}
	for n in nodes:
		if n.id == hovered:
			hover_node = n
			continue
		var p: Vector2 = pos.get(n.id, Vector2.ZERO)
		var c: int = COL_OF.get(n.layer, 2)
		var k: int = per_col.get(c, 0)
		per_col[c] = k + 1
		var txt := _fit(str(n.label), colw - 2.0, 10)
		var tw := font.get_string_size(txt, HORIZONTAL_ALIGNMENT_LEFT, -1, 10).x
		var lp := p + Vector2(-tw / 2.0, 21.0 if k % 2 == 0 else -13.0)
		lp.x = clampf(lp.x, 2.0, size.x - tw - 2.0)
		draw_string(font, lp, txt, HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(UI.TEXT, 0.85))
	if hover_node != null:
		var p: Vector2 = pos.get(hover_node.id, Vector2.ZERO)
		var txt: String = hover_node.label
		var tw := minf(font.get_string_size(txt, HORIZONTAL_ALIGNMENT_LEFT, -1, 12).x, size.x - 12.0)
		var lp := Vector2(clampf(p.x - tw / 2.0, 6.0, size.x - tw - 6.0), p.y + 26.0)
		if lp.y > size.y - 6.0:
			lp.y = p.y - 16.0
		draw_rect(Rect2(lp + Vector2(-5, -13), Vector2(tw + 10, 19)), Color(UI.BG, 0.96))
		draw_rect(Rect2(lp + Vector2(-5, -13), Vector2(tw + 10, 19)), UI.ACCENT, false, 1.0)
		draw_string(font, lp, txt, HORIZONTAL_ALIGNMENT_LEFT, tw, 12, UI.TEXT)


func _fit(t: String, w: float, fs: int) -> String:
	if font.get_string_size(t, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x <= w:
		return t
	var s := t
	while s.length() > 2 and font.get_string_size(s + "…", HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > w:
		s = s.substr(0, s.length() - 1)
	return s + "…"
