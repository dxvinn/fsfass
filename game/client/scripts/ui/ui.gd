class_name UI
## Shared look and reusable UI building blocks for Project Genesis.
## Every panel in the game is assembled from these helpers so the style stays consistent.

const BG := Color("#14181d")
const PANEL := Color("#1d232b")
const PANEL_HI := Color("#262e38")
const LINE := Color("#3a4552")
const TEXT := Color("#e9e4d8")
const MUTED := Color("#9aa3ad")
const ACCENT := Color("#e8b04b")
const GOOD := Color("#7fc77a")
const BAD := Color("#e0675a")
const COOL := Color("#6fb3d9")
const MAGIC := Color("#b48be8")

const LAYER_COLORS := {
	"perception": Color("#6fb3d9"),
	"cue": Color("#8fd0c4"),
	"outcome": Color("#e8b04b"),
	"memory": Color("#b48be8"),
	"emotion": Color("#e0675a"),
	"goal": Color("#f2d27a"),
	"action": Color("#7fc77a"),
}

static var _theme: Theme


static func box(color: Color, radius := 10, border := Color(0, 0, 0, 0), border_w := 0, pad := 10) -> StyleBoxFlat:
	var s := StyleBoxFlat.new()
	s.bg_color = color
	s.set_corner_radius_all(radius)
	s.border_color = border
	s.set_border_width_all(border_w)
	s.content_margin_left = pad
	s.content_margin_right = pad
	s.content_margin_top = pad * 0.7
	s.content_margin_bottom = pad * 0.7
	s.anti_aliasing = true
	return s


static func theme() -> Theme:
	if _theme:
		return _theme
	var t := Theme.new()
	t.default_font_size = 14
	t.set_color("font_color", "Label", TEXT)
	t.set_color("font_color", "Button", TEXT)
	t.set_color("font_hover_color", "Button", Color.WHITE)
	t.set_color("font_pressed_color", "Button", Color("#1b1b1b"))
	t.set_color("font_hover_pressed_color", "Button", Color("#1b1b1b"))
	t.set_stylebox("normal", "Button", box(PANEL_HI, 8, LINE, 1, 8))
	t.set_stylebox("hover", "Button", box(Color("#323c48"), 8, ACCENT.darkened(0.3), 1, 8))
	t.set_stylebox("pressed", "Button", box(ACCENT, 8, ACCENT, 1, 8))
	t.set_stylebox("hover_pressed", "Button", box(ACCENT.lightened(0.1), 8, ACCENT, 1, 8))
	t.set_stylebox("focus", "Button", StyleBoxEmpty.new())
	t.set_stylebox("disabled", "Button", box(PANEL, 8, LINE, 1, 8))
	t.set_stylebox("panel", "PanelContainer", box(Color(PANEL, 0.94), 14, LINE, 1, 12))
	t.set_stylebox("panel", "TabContainer", box(Color(0, 0, 0, 0), 0, Color(0, 0, 0, 0), 0, 4))
	t.set_stylebox("tab_selected", "TabBar", box(ACCENT, 7, ACCENT, 0, 7))
	t.set_stylebox("tab_unselected", "TabBar", box(PANEL_HI, 7, LINE, 0, 7))
	t.set_stylebox("tab_hovered", "TabBar", box(Color("#323c48"), 7, LINE, 0, 7))
	t.set_color("font_selected_color", "TabBar", Color("#1b1b1b"))
	t.set_color("font_unselected_color", "TabBar", MUTED)
	t.set_color("font_hovered_color", "TabBar", TEXT)
	t.set_constant("h_separation", "TabBar", 3)
	t.set_font_size("font_size", "TabBar", 12)
	t.set_stylebox("panel", "TooltipPanel", box(Color(BG, 0.96), 8, LINE, 1, 8))
	t.set_color("font_color", "TooltipLabel", TEXT)
	t.set_stylebox("scroll", "VScrollBar", box(Color(0, 0, 0, 0.15), 4, Color(0, 0, 0, 0), 0, 2))
	t.set_stylebox("grabber", "VScrollBar", box(LINE, 4, Color(0, 0, 0, 0), 0, 2))
	t.set_stylebox("grabber_highlight", "VScrollBar", box(MUTED, 4, Color(0, 0, 0, 0), 0, 2))
	t.set_stylebox("grabber_pressed", "VScrollBar", box(ACCENT, 4, Color(0, 0, 0, 0), 0, 2))
	t.set_color("default_color", "RichTextLabel", TEXT)
	t.set_stylebox("normal", "RichTextLabel", StyleBoxEmpty.new())
	t.set_stylebox("background", "ProgressBar", box(Color(0, 0, 0, 0.35), 5, Color(0, 0, 0, 0), 0, 0))
	t.set_stylebox("fill", "ProgressBar", box(ACCENT, 5, Color(0, 0, 0, 0), 0, 0))
	_theme = t
	return t


## A rounded card panel with an optional title.
static func card(title := "") -> PanelContainer:
	var p := PanelContainer.new()
	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 6)
	p.add_child(v)
	if title != "":
		v.add_child(heading(title))
	return p


static func card_body(p: PanelContainer) -> VBoxContainer:
	return p.get_child(0)


static func label(text: String, size := 14, color := TEXT, wrap := false) -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", size)
	l.add_theme_color_override("font_color", color)
	if wrap:
		l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		l.custom_minimum_size.x = 40
	return l


static func heading(text: String) -> Label:
	var l := label(text.to_upper(), 11, ACCENT)
	l.add_theme_constant_override("outline_size", 0)
	return l


static func button(text: String, tip := "", toggle := false) -> Button:
	var b := Button.new()
	b.text = text
	b.tooltip_text = tip
	b.toggle_mode = toggle
	b.focus_mode = Control.FOCUS_NONE
	b.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	return b


## Labelled meter: name, a coloured bar, and a value.
static func meter(name: String, value: float, color := ACCENT, invert_bad := false) -> HBoxContainer:
	var h := HBoxContainer.new()
	h.add_theme_constant_override("separation", 8)
	var n := label(name, 13, MUTED)
	n.custom_minimum_size.x = 92
	h.add_child(n)
	var bar := ProgressBar.new()
	bar.min_value = 0.0
	bar.max_value = 1.0
	bar.value = clampf(value, 0.0, 1.0)
	bar.show_percentage = false
	bar.custom_minimum_size = Vector2(120, 10)
	bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	var c := color
	if invert_bad:
		c = GOOD.lerp(BAD, clampf(value, 0.0, 1.0))
	bar.add_theme_stylebox_override("fill", box(c, 5, Color(0, 0, 0, 0), 0, 0))
	h.add_child(bar)
	var v := label("%d%%" % roundi(value * 100.0), 12, TEXT)
	v.custom_minimum_size.x = 38
	v.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	h.add_child(v)
	return h


## Small rounded tag.
static func chip(text: String, color := LINE, text_color := TEXT) -> PanelContainer:
	var p := PanelContainer.new()
	p.add_theme_stylebox_override("panel", box(color, 9, Color(0, 0, 0, 0), 0, 6))
	var l := label(text, 11, text_color)
	p.add_child(l)
	return p


## Two-column key/value row.
static func row(key: String, value: String, value_color := TEXT) -> HBoxContainer:
	var h := HBoxContainer.new()
	var k := label(key, 13, MUTED)
	k.custom_minimum_size.x = 110
	h.add_child(k)
	var v := label(value, 13, value_color, true)
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	h.add_child(v)
	return h


## A list entry card: title line, optional detail line, coloured left edge.
static func entry(title: String, detail := "", edge := LINE) -> PanelContainer:
	var p := PanelContainer.new()
	var s := box(Color(PANEL_HI, 0.85), 8, Color(0, 0, 0, 0), 0, 8)
	s.border_color = edge
	s.border_width_left = 3
	p.add_theme_stylebox_override("panel", s)
	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 2)
	p.add_child(v)
	v.add_child(label(title, 13, TEXT, true))
	if detail != "":
		v.add_child(label(detail, 11, MUTED, true))
	return p


static func scroll_list() -> ScrollContainer:
	var s := ScrollContainer.new()
	s.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	s.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var v := VBoxContainer.new()
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	v.add_theme_constant_override("separation", 5)
	s.add_child(v)
	return s


static func clear(node: Node) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()


static func pct(v) -> String:
	return "%d%%" % roundi(float(v) * 100.0)
