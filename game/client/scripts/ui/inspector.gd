extends PanelContainer
## Creature inspector: tabs for overview, body, brain, memories, knowledge, beliefs,
## relationships, family, skills and genetics. Built from the simulation's JSON view.

signal select_requested(id: int)
signal follow_toggled(on: bool)
signal closed

const TABS := ["Overview", "Body", "Brain", "Memories", "Knowledge", "Beliefs", "Relations", "Family", "Skills", "Genetics"]

var data := {}
var title_lbl: Label
var sub_lbl: Label
var tabs: HFlowContainer
var tab_buttons: Array = []
var content: VBoxContainer
var scroll: ScrollContainer
var brain: Control
var follow_btn: Button
var current := 0
var portrait: Control


func _ready() -> void:
	custom_minimum_size = Vector2(430, 0)
	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 8)
	add_child(v)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 10)
	v.add_child(head)
	portrait = preload("res://scripts/ui/portrait.gd").new()
	head.add_child(portrait)
	var names := VBoxContainer.new()
	names.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	names.add_theme_constant_override("separation", 0)
	head.add_child(names)
	title_lbl = UI.label("", 22, UI.TEXT)
	names.add_child(title_lbl)
	sub_lbl = UI.label("", 12, UI.MUTED, true)
	names.add_child(sub_lbl)
	follow_btn = UI.button("Follow", "Camera follows this creature (F)", true)
	follow_btn.toggled.connect(func(on): follow_toggled.emit(on))
	head.add_child(follow_btn)
	var close := UI.button("✕", "Close (Esc)")
	close.pressed.connect(func(): closed.emit())
	head.add_child(close)
	tabs = HFlowContainer.new()
	tabs.add_theme_constant_override("h_separation", 4)
	tabs.add_theme_constant_override("v_separation", 4)
	var group := ButtonGroup.new()
	for i in TABS.size():
		var b := UI.button(TABS[i], "", true)
		b.button_group = group
		b.add_theme_font_size_override("font_size", 12)
		b.button_pressed = i == 0
		b.pressed.connect(_on_tab.bind(i))
		tab_buttons.append(b)
		tabs.add_child(b)
	v.add_child(tabs)
	scroll = UI.scroll_list()
	v.add_child(scroll)
	content = scroll.get_child(0)
	brain = preload("res://scripts/ui/brain_graph.gd").new()
	brain.custom_minimum_size = Vector2(380, 380)


func select_tab(i: int) -> void:
	tab_buttons[i].button_pressed = true
	_on_tab(i)


func _on_tab(i: int) -> void:
	current = i
	scroll.scroll_vertical = 0
	_rebuild()


func show_json(json: String) -> void:
	var parsed = JSON.parse_string(json)
	if typeof(parsed) != TYPE_DICTIONARY or parsed.is_empty():
		return
	data = parsed
	title_lbl.text = data.get("name", "?")
	var alive: bool = data.get("alive", false)
	var bits := "%s · %s · %d years · %s" % [data.get("kind", ""), data.get("sex", ""), int(data.get("age", 0)), data.get("stage", "")]
	if not alive:
		bits = "† " + bits + " · died: " + str(data.get("cause_of_death", ""))
	sub_lbl.text = bits
	portrait.set_data(data)
	var human: bool = data.get("kind", "") == "Human"
	for i in TABS.size():
		tab_buttons[i].visible = human or not (i in [2, 3, 4, 5, 8])
	if not human and current in [2, 3, 4, 5, 8]:
		current = 0
		tab_buttons[0].button_pressed = true
	var keep := scroll.scroll_vertical
	_rebuild()
	scroll.set_deferred("scroll_vertical", keep)


func _rebuild() -> void:
	if brain.get_parent():
		brain.get_parent().remove_child(brain)
	UI.clear(content)
	if data.is_empty():
		return
	match current:
		0: _overview()
		1: _body()
		2: _brain()
		3: _memories()
		4: _knowledge()
		5: _beliefs()
		6: _relations()
		7: _family()
		8: _skills()
		9: _genetics()


func _add(c: Control) -> void:
	content.add_child(c)


func _section(t: String) -> VBoxContainer:
	var p := UI.card(t)
	p.add_theme_stylebox_override("panel", UI.box(Color(UI.PANEL_HI, 0.55), 10, Color(0, 0, 0, 0), 0, 10))
	_add(p)
	return UI.card_body(p)


func _overview() -> void:
	var s := _section("Now")
	s.add_child(UI.label(str(data.get("action", "")).capitalize(), 17, UI.ACCENT, true))
	var b: Dictionary = data.get("body", {})
	var needs := HFlowContainer.new()
	needs.add_theme_constant_override("h_separation", 6)
	needs.add_theme_constant_override("v_separation", 6)
	s.add_child(needs)
	for k in ["hunger", "thirst", "pain"]:
		var v := float(b.get(k, 0))
		if v > 0.5:
			needs.add_child(UI.chip("%s %s" % [k, UI.pct(v)], Color(UI.BAD, 0.8), Color.WHITE))
	if float(b.get("energy", 1)) < 0.3:
		needs.add_child(UI.chip("exhausted", Color(UI.COOL, 0.8), Color.WHITE))
	if b.get("asleep", false):
		needs.add_child(UI.chip("asleep", Color(UI.MAGIC, 0.8), Color.WHITE))
	if b.get("pregnant", false):
		needs.add_child(UI.chip("pregnant", Color(UI.GOOD, 0.8), Color.WHITE))
	if data.has("trace") and data.trace is Dictionary:
		_why_section(data.trace)
	var vit := _section("Vitals")
	vit.add_child(UI.meter("Health", float(data.get("health", 0)), UI.GOOD))
	vit.add_child(UI.meter("Hunger", float(b.get("hunger", 0)), UI.BAD, true))
	vit.add_child(UI.meter("Thirst", float(b.get("thirst", 0)), UI.BAD, true))
	vit.add_child(UI.meter("Energy", float(b.get("energy", 0)), UI.COOL))
	var fam: Dictionary = data.get("family", {})
	var who := _section("Life")
	if fam.get("partner") is Dictionary:
		who.add_child(UI.row("Partner", fam.partner.name))
	who.add_child(UI.row("Children", str(fam.get("children", []).size())))
	if data.has("knowledge"):
		who.add_child(UI.row("Knows", "%d learned links · %d beliefs · %d memories" % [data.knowledge.size(), data.get("beliefs", []).size(), data.get("episodes_stored", 0)]))
	var near: Array = data.get("nearby", [])
	if near.size() > 0:
		who.add_child(UI.row("Nearby", ", ".join(near)))
	who.add_child(UI.row("Thinks every", "%d s" % int(data.get("think_interval", 1))))


func _why_section(tr: Dictionary) -> void:
	var w := _section("Why")
	w.add_child(UI.row("Goal", str(tr.get("goal", ""))))
	w.add_child(UI.row("Reason", str(tr.get("reason", "")), UI.ACCENT))
	var emo: Dictionary = tr.get("emotions", {})
	if not emo.is_empty():
		var parts := []
		for k in emo:
			parts.append("%s %d%%" % [k, roundi(float(emo[k]) * 100.0)])
		w.add_child(UI.row("Feeling", ", ".join(parts)))
	var mem: Array = tr.get("memories", [])
	if mem.size() > 0:
		w.add_child(UI.row("Remembers", str(mem[0].summary)))
	var assoc: Array = tr.get("associations", [])
	if assoc.size() > 0:
		var a = assoc[0]
		var txt := "%s%s → %s" % ["" if a.action == null else str(a.action) + " + ", a.cue, a.outcome]
		w.add_child(UI.row("Expects", txt))
	var opts: Array = tr.get("options", [])
	if opts.size() > 1:
		var alt := []
		for o in opts.slice(0, 3):
			alt.append("%s (%+.2f)" % [o.option, float(o.total)])
		w.add_child(UI.row("Weighed", ", ".join(alt), UI.MUTED))
	var learn: Array = tr.get("learning", [])
	if learn.size() > 0:
		w.add_child(UI.row("Learning", str(learn[0]), UI.GOOD))


func _body() -> void:
	var b: Dictionary = data.get("body", {})
	var s := _section("Body")
	s.add_child(UI.meter("Health", float(data.get("health", 0)), UI.GOOD))
	for k in ["hunger", "thirst", "pain", "loneliness"]:
		if b.has(k):
			s.add_child(UI.meter(k.capitalize(), float(b[k]), UI.BAD, true))
	if b.has("energy"):
		s.add_child(UI.meter("Energy", float(b.energy), UI.COOL))
	if b.has("warmth"):
		s.add_child(UI.meter("Body heat", float(b.warmth), Color("#f0a04a")))
	if b.has("fear_of_humans"):
		s.add_child(UI.meter("Fear of humans", float(b.fear_of_humans), UI.MAGIC))
	var d := _section("Condition")
	if b.has("wounds"):
		d.add_child(UI.row("Wounds", UI.pct(b.wounds)))
		d.add_child(UI.row("Burns", UI.pct(b.burns)))
	d.add_child(UI.row("Asleep", "yes" if b.get("asleep", false) else "no"))
	d.add_child(UI.row("Pregnant", "yes" if b.get("pregnant", false) else "no"))
	d.add_child(UI.row("Position", "%d, %d" % [int(data.get("x", 0)), int(data.get("y", 0))]))


func _brain() -> void:
	var s := _section("Active pathway")
	s.add_child(UI.label("What this mind sensed, what it has learned those things predict, what it recalled, how it felt, and what it chose. Hover a node for detail.", 11, UI.MUTED, true))
	s.add_child(brain)
	brain.set_graph(data.get("graph"))
	var tsec := _section("Decision trace")
	var lbl := UI.label(str(data.get("trace_text", "")), 11, UI.TEXT, true)
	lbl.add_theme_font_override("font", _mono())
	tsec.add_child(lbl)
	var st := _section("Mind")
	st.add_child(UI.row("Concepts formed", str(data.get("concepts", 0))))
	st.add_child(UI.row("Episodes stored", str(data.get("episodes_stored", 0))))


func _mono() -> Font:
	var f := SystemFont.new()
	f.font_names = PackedStringArray(["DejaVu Sans Mono", "Monospace", "Consolas", "Menlo"])
	return f


func _memories() -> void:
	var mem: Array = data.get("memories", [])
	if mem.is_empty():
		_add(UI.label("No memories yet.", 13, UI.MUTED))
	for m in mem:
		var v := float(m.valence)
		var edge := UI.GOOD.lerp(UI.BAD, clampf(0.5 - v, 0.0, 1.0)) if v != 0.0 else UI.LINE
		var detail := "%s · felt %+.2f · intensity %.2f · recalled %d×%s" % [m.ago, v, float(m.arousal), int(m.recalls), " · ★ unforgettable" if m.pinned else ""]
		_add(UI.entry(m.text, detail, edge))


func _knowledge() -> void:
	_add(UI.label("Things this person has learned through experience. Nothing here was given to them; every link was learned.", 11, UI.MUTED, true))
	var k: Array = data.get("knowledge", [])
	if k.is_empty():
		_add(UI.label("Has not learned anything yet.", 13, UI.MUTED))
	for e in k:
		var col := UI.COOL
		var w: String = e.what
		if w.contains("pain"):
			col = UI.BAD
		elif w.contains("nourish") or w.contains("hydrat") or w.contains("warm") or w.contains("comfort"):
			col = UI.GOOD
		var p := UI.entry("%s  →  %s" % [e.cue, w], "strength %.2f · from %d experiences" % [float(e.strength), int(e.evidence)], col)
		_add(p)


func _beliefs() -> void:
	_add(UI.label("Explicit beliefs: knowledge confident enough to state and pass on. Beliefs can be wrong.", 11, UI.MUTED, true))
	var bs: Array = data.get("beliefs", [])
	if bs.is_empty():
		_add(UI.label("No firm beliefs yet.", 13, UI.MUTED))
	for b in bs:
		var src: String = b.source
		var col := UI.ACCENT if src.begins_with("own") else (UI.MAGIC if src.begins_with("told") else UI.COOL)
		_add(UI.entry(b.belief, "confidence %s · %s · evidence %d" % [UI.pct(b.confidence), src, int(b.evidence)], col))


func _relations() -> void:
	var rs: Array = data.get("relationships", [])
	if rs.is_empty():
		_add(UI.label("Knows no one yet.", 13, UI.MUTED))
	for r in rs:
		var p := UI.card()
		p.add_theme_stylebox_override("panel", UI.box(Color(UI.PANEL_HI, 0.55), 10, Color(0, 0, 0, 0), 0, 10))
		var body := UI.card_body(p)
		var h := HBoxContainer.new()
		var nb := UI.button(r.name, "Select")
		nb.pressed.connect(func(): select_requested.emit(int(r.id)))
		h.add_child(nb)
		h.add_child(UI.label("  " + _rel_word(r), 12, UI.MUTED))
		body.add_child(h)
		body.add_child(UI.meter("Familiarity", float(r.familiarity), UI.COOL))
		body.add_child(UI.meter("Affection", float(r.affection), Color("#e88aa0")))
		body.add_child(UI.meter("Trust", float(r.trust), UI.GOOD))
		if float(r.attraction) > 0.05:
			body.add_child(UI.meter("Attraction", float(r.attraction), Color("#e86fa0")))
		if float(r.fear) > 0.05:
			body.add_child(UI.meter("Fear", float(r.fear), UI.MAGIC))
		if float(r.resentment) > 0.05:
			body.add_child(UI.meter("Resentment", float(r.resentment), UI.BAD))
		_add(p)


func _rel_word(r: Dictionary) -> String:
	var a := float(r.affection)
	var f := float(r.familiarity)
	if float(r.fear) > 0.4:
		return "feared"
	if a > 0.6:
		return "loved"
	if a > 0.3:
		return "friend"
	if f > 0.3:
		return "familiar"
	return "acquaintance"


func _family() -> void:
	var fam: Dictionary = data.get("family", {})
	var tree := preload("res://scripts/ui/family_tree.gd").new()
	tree.custom_minimum_size = Vector2(380, 80)
	tree.person_clicked.connect(func(id): select_requested.emit(id))
	_add(tree)
	tree.set_family(data, fam)
	var s := _section("Relatives")
	for key in ["mother", "father", "partner"]:
		if fam.get(key) is Dictionary:
			s.add_child(_person_row(key.capitalize(), fam[key]))
	for key in ["children", "siblings", "grandparents"]:
		for p in fam.get(key, []):
			s.add_child(_person_row(key.capitalize().trim_suffix("ren").trim_suffix("s"), p))


func _person_row(rel: String, p: Dictionary) -> Control:
	var h := HBoxContainer.new()
	var k := UI.label(rel, 12, UI.MUTED)
	k.custom_minimum_size.x = 90
	h.add_child(k)
	var b := UI.button("%s%s, %d" % ["" if p.alive else "† ", p.name, int(p.age)])
	b.disabled = not p.alive
	b.pressed.connect(func(): select_requested.emit(int(p.id)))
	h.add_child(b)
	return h


func _skills() -> void:
	var sk: Array = data.get("skills", [])
	_add(UI.label("Skills grow only by doing.", 11, UI.MUTED, true))
	if sk.is_empty():
		_add(UI.label("No practiced skills yet.", 13, UI.MUTED))
	var s := _section("Skills")
	for e in sk:
		s.add_child(UI.meter(str(e.name).capitalize(), float(e.level), UI.ACCENT))


func _genetics() -> void:
	var g: Dictionary = data.get("genetics", {})
	var s := _section("Physical")
	for k in ["height", "build", "skin", "hair", "eye", "metabolism", "vitality", "fertility", "dexterity"]:
		s.add_child(UI.meter(k.capitalize(), float(g.get(k, 0)), UI.COOL))
	var p := _section("Temperament")
	for k in ["openness", "conscientiousness", "extraversion", "agreeableness", "neuroticism"]:
		p.add_child(UI.meter(k.capitalize(), float(g.get(k, 0)), UI.MAGIC))
	var gs := _section("Genome")
	var gl := UI.label(str(g.get("genome", "")), 10, UI.MUTED, true)
	gl.add_theme_font_override("font", _mono())
	gs.add_child(gl)
