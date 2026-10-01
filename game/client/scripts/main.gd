extends Node2D
## Project Genesis client. Owns the GenesisWorld (Rust simulation), the world view,
## the god camera and the HUD (time bar, god powers, inspector, history).

const SPEEDS := [0, 1, 5, 25, 100, 1000]
const SPEED_LABELS := ["❚❚", "1×", "5×", "25×", "100×", "1000×"]
const WORLD_W := 200
const WORLD_H := 140

const POWERS := [
	# [id, label, tooltip, colour, needs_target]
	["select", "Select", "Click a creature to inspect it", "#e9e4d8", false],
	["create_human", "Human", "Create a human here (a new mind that knows nothing)", "#e8b04b", true],
	["create_grazer", "Grazer", "Create a grazer (herbivore) here", "#b98a5a", true],
	["create_predator", "Wolf", "Create a wolf (predator) here", "#8a8f98", true],
	["create_plant", "Plants", "Grow plants and trees here", "#7fc77a", true],
	["add_food", "Food", "Make berry bushes grow here", "#d2434f", true],
	["add_water", "Water", "Make a pond here", "#6fb3d9", true],
	["heal", "Heal", "Heal the creature (or creatures) here", "#7fc77a", true],
	["bless", "Bless", "Bless: full health, fed, rested, calm", "#f2d27a", true],
	["curse", "Curse", "Curse: sickness and fear", "#b48be8", true],
	["kill", "Smite", "Kill the creature here", "#e0675a", true],
	["lightning", "Lightning", "Strike lightning here", "#fff2a8", true],
	["fire", "Fire", "Start a fire here", "#f0782a", true],
	["rain", "Rain", "Bring rain (puts out fires)", "#6fb3d9", false],
	["warmer", "Warmer", "Raise the world's temperature", "#f0a04a", false],
	["colder", "Colder", "Lower the world's temperature", "#9fd0f0", false],
]

var world
var view: Node2D
var cam: Camera2D
var hud: CanvasLayer
var night: CanvasModulate
var speed_idx := 1
var tool := "select"
var selected := -1
var inspector: PanelContainer
var history_box: VBoxContainer
var history_scroll: ScrollContainer
var history_panel: PanelContainer
var history_count := 0
var toasts: VBoxContainer
var clock_lbl: Label
var info_lbl: Label
var pop_lbl: Label
var speed_lbl: Label
var fid_lbl: Label
var tool_hint: Label
var tooltip_panel: PanelContainer
var tooltip_lbl: Label
var speed_buttons: Array = []
var tool_buttons := {}
var inspect_timer := 0.0
var status_timer := 0.0
var history_timer := 0.0
var last_terrain_tick := 0
var shot_path := ""
var shot_frames := -1
var auto_select := false
var shot_tab := -1
var shot_saveload := false
var warp_days := 0.0
var shot_gods: PackedStringArray = []
var shot_click := Vector2(-1, -1)
var zoom_from_args := 1.4
var speed_from_args := 1
var status := {}


func _ready() -> void:
	get_window().title = "Project Genesis"
	_parse_args()
	world = ClassDB.instantiate("GenesisWorld")
	if world == null:
		push_error("GenesisWorld not found: run game/client/build.sh to build the Rust extension")
		return
	var seed := 7
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--seed="):
			seed = int(a.substr(7))
	world.new_world(seed, WORLD_W, WORLD_H)
	if warp_days > 0.0:
		# Testing aid: simulate ahead (at 1000x cognition LOD) before showing the world.
		var t0 := Time.get_ticks_msec()
		world.run_ticks(int(warp_days * 86400.0), 1000)
		print("warped %.1f days in %d ms" % [warp_days, Time.get_ticks_msec() - t0])
	night = CanvasModulate.new()
	add_child(night)
	view = preload("res://scripts/world_view.gd").new()
	add_child(view)
	view.setup(world)
	cam = preload("res://scripts/camera.gd").new()
	add_child(cam)
	cam.bounds = Rect2(Vector2.ZERO, view.world_size_px())
	cam.user_moved.connect(func(): _set_follow(false))
	cam.make_current()
	_focus_on_humans()
	_build_hud()
	_set_speed(speed_from_args)


func _parse_args() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--screenshot="):
			shot_path = a.substr(13)
		elif a.begins_with("--frames="):
			shot_frames = int(a.substr(9))
		elif a.begins_with("--tab="):
			shot_tab = int(a.substr(6))
		elif a.begins_with("--speed="):
			speed_from_args = int(a.substr(8))
		elif a.begins_with("--zoom="):
			zoom_from_args = float(a.substr(7))
		elif a.begins_with("--god="):
			shot_gods = a.substr(6).split(",")
		elif a.begins_with("--click="):
			var xy := a.substr(8).split(",")
			shot_click = Vector2(float(xy[0]), float(xy[1]))
		elif a.begins_with("--warp-days="):
			warp_days = float(a.substr(12))
		elif a == "--saveload":
			shot_saveload = true
		elif a == "--select":
			auto_select = true


func _focus_on_humans() -> void:
	var c: PackedFloat32Array = world.creatures()
	for i in c.size() / 16:
		if int(c[i * 16 + 1]) == 0:
			cam.position = (Vector2(c[i * 16 + 2], c[i * 16 + 3]) + Vector2(0.5, 0.5)) * view.TILE
			break
	cam.zoom = Vector2.ONE * zoom_from_args
	cam.target_zoom = zoom_from_args


# ---------------------------------------------------------------- HUD

func _build_hud() -> void:
	hud = CanvasLayer.new()
	add_child(hud)
	var root := Control.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.theme = UI.theme()
	hud.add_child(root)

	# Top bar: title, clock, speed controls, world info.
	var top := PanelContainer.new()
	top.set_anchors_preset(Control.PRESET_CENTER_TOP)
	top.position = Vector2(-10, 10)
	top.grow_horizontal = Control.GROW_DIRECTION_BOTH
	top.anchor_left = 0.5
	top.anchor_right = 0.5
	root.add_child(top)
	var tb := HBoxContainer.new()
	tb.add_theme_constant_override("separation", 12)
	top.add_child(tb)
	var title := UI.label("GENESIS", 15, UI.ACCENT)
	tb.add_child(title)
	tb.add_child(VSeparator.new())
	var clock_box := VBoxContainer.new()
	clock_box.add_theme_constant_override("separation", -2)
	clock_lbl = UI.label("Day 1 06:00", 17, UI.TEXT)
	clock_box.add_child(clock_lbl)
	info_lbl = UI.label("", 11, UI.MUTED)
	clock_box.add_child(info_lbl)
	tb.add_child(clock_box)
	tb.add_child(VSeparator.new())
	var sp := HBoxContainer.new()
	sp.add_theme_constant_override("separation", 3)
	var group := ButtonGroup.new()
	for i in SPEEDS.size():
		var b := UI.button(SPEED_LABELS[i], "Pause (Space)" if i == 0 else "%d game minutes per second (key %d)" % [SPEEDS[i], i], true)
		b.button_group = group
		b.custom_minimum_size.x = 44
		b.pressed.connect(_set_speed.bind(i))
		speed_buttons.append(b)
		sp.add_child(b)
	tb.add_child(sp)
	var spinfo := VBoxContainer.new()
	spinfo.add_theme_constant_override("separation", -2)
	speed_lbl = UI.label("", 12, UI.TEXT)
	spinfo.add_child(speed_lbl)
	fid_lbl = UI.label("", 10, UI.MUTED)
	spinfo.add_child(fid_lbl)
	tb.add_child(spinfo)
	tb.add_child(VSeparator.new())
	pop_lbl = UI.label("", 13, UI.TEXT)
	tb.add_child(pop_lbl)
	var hist_btn := UI.button("History", "Show or hide the history of this world (H)", true)
	hist_btn.button_pressed = true
	hist_btn.toggled.connect(func(on): history_panel.visible = on)
	tb.add_child(hist_btn)
	var save_btn := UI.button("Save", "Save this world (F5)")
	save_btn.pressed.connect(_save_world)
	tb.add_child(save_btn)
	var load_btn := UI.button("Load", "Load a saved world (F9)")
	load_btn.pressed.connect(_show_load_menu)
	tb.add_child(load_btn)
	_build_load_ui(root)

	# God toolbar (left).
	var tools := PanelContainer.new()
	tools.position = Vector2(10, 90)
	root.add_child(tools)
	var tv := VBoxContainer.new()
	tv.add_theme_constant_override("separation", 4)
	tools.add_child(tv)
	tv.add_child(UI.heading("God powers"))
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 4)
	grid.add_theme_constant_override("v_separation", 4)
	tv.add_child(grid)
	var tgroup := ButtonGroup.new()
	for p in POWERS:
		var b := UI.button(p[1], p[2], p[4] or p[0] == "select")
		b.custom_minimum_size = Vector2(84, 30)
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.icon = _dot_icon(Color(p[3]))
		if p[4] or p[0] == "select":
			b.button_group = tgroup
			b.pressed.connect(_set_tool.bind(p[0]))
		else:
			b.pressed.connect(_instant_power.bind(p[0]))
		tool_buttons[p[0]] = b
		grid.add_child(b)
	tool_buttons["select"].button_pressed = true
	tool_hint = UI.label("", 11, UI.MUTED, true)
	tool_hint.custom_minimum_size.x = 172
	tv.add_child(tool_hint)

	# History panel (bottom left).
	history_panel = UI.card("History")
	history_panel.anchor_top = 1.0
	history_panel.anchor_bottom = 1.0
	history_panel.offset_left = 10
	history_panel.offset_top = -210
	history_panel.offset_right = 470
	history_panel.offset_bottom = -10
	root.add_child(history_panel)
	history_scroll = UI.scroll_list()
	UI.card_body(history_panel).add_child(history_scroll)
	history_box = history_scroll.get_child(0)
	history_box.add_theme_constant_override("separation", 3)

	# Inspector (right).
	inspector = preload("res://scripts/ui/inspector.gd").new()
	inspector.anchor_left = 1.0
	inspector.anchor_right = 1.0
	inspector.anchor_bottom = 1.0
	inspector.offset_left = -450
	inspector.offset_right = -10
	inspector.offset_top = 80
	inspector.offset_bottom = -10
	inspector.visible = false
	inspector.select_requested.connect(_select)
	inspector.follow_toggled.connect(_set_follow)
	inspector.closed.connect(func(): _select(-1))
	root.add_child(inspector)

	# Toasts (top centre, under bar).
	toasts = VBoxContainer.new()
	toasts.anchor_left = 0.5
	toasts.anchor_right = 0.5
	toasts.offset_left = -220
	toasts.offset_right = 220
	toasts.offset_top = 78
	toasts.mouse_filter = Control.MOUSE_FILTER_IGNORE
	toasts.add_theme_constant_override("separation", 4)
	root.add_child(toasts)

	# Hover tooltip.
	tooltip_panel = PanelContainer.new()
	tooltip_panel.add_theme_stylebox_override("panel", UI.box(Color(UI.BG, 0.92), 7, UI.LINE, 1, 7))
	tooltip_panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	tooltip_lbl = UI.label("", 12)
	tooltip_panel.add_child(tooltip_lbl)
	tooltip_panel.visible = false
	root.add_child(tooltip_panel)

	# Controls hint (bottom right when nothing is selected).
	var help := UI.label("WASD / drag right mouse: move   ·   wheel: zoom   ·   click: inspect   ·   Space: pause   ·   1–5: speed   ·   F: follow", 11, Color(UI.TEXT, 0.6))
	help.anchor_left = 1.0
	help.anchor_right = 1.0
	help.anchor_top = 1.0
	help.anchor_bottom = 1.0
	help.offset_left = -760
	help.offset_right = -16
	help.offset_top = -28
	help.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	help.name = "Help"
	root.add_child(help)
	_set_tool("select")


func _dot_icon(c: Color) -> Texture2D:
	var img := Image.create(14, 14, false, Image.FORMAT_RGBA8)
	for y in 14:
		for x in 14:
			var d := Vector2(x - 6.5, y - 6.5).length()
			if d < 5.5:
				img.set_pixel(x, y, c if d < 4.5 else c.darkened(0.4))
	return ImageTexture.create_from_image(img)


# ---------------------------------------------------------------- input

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_SPACE:
				_set_speed(0 if speed_idx != 0 else 1)
			KEY_1, KEY_2, KEY_3, KEY_4, KEY_5:
				_set_speed(event.keycode - KEY_0)
			KEY_F:
				if selected >= 0:
					_set_follow(not cam.following)
			KEY_F5:
				_save_world()
			KEY_F9:
				_show_load_menu()
			KEY_H:
				history_panel.visible = not history_panel.visible
			KEY_ESCAPE:
				if tool != "select":
					tool_buttons["select"].button_pressed = true
					_set_tool("select")
				else:
					_select(-1)
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var wp := get_global_mouse_position()
		var tile := Vector2i(floori(wp.x / view.TILE), floori(wp.y / view.TILE))
		if tool == "select":
			var id: int = world.creature_at(wp.x / view.TILE, wp.y / view.TILE, 1.4 / maxf(cam.zoom.x, 0.5) + 0.6)
			_select(id)
		else:
			_use_power(tool, tile)
	elif event is InputEventMouseMotion:
		_update_hover()


func _update_hover() -> void:
	if world == null:
		return
	var wp := get_global_mouse_position()
	var id: int = world.creature_at(wp.x / view.TILE, wp.y / view.TILE, 1.2)
	view.hover_id = id
	if id >= 0 and tool == "select":
		tooltip_lbl.text = world.tooltip(id)
		tooltip_panel.visible = true
		tooltip_panel.position = get_viewport().get_mouse_position() + Vector2(16, 16)
		tooltip_panel.reset_size()
	else:
		tooltip_panel.visible = false


func _set_speed(i: int) -> void:
	speed_idx = clampi(i, 0, SPEEDS.size() - 1)
	speed_buttons[speed_idx].button_pressed = true


func _set_tool(t: String) -> void:
	tool = t
	for p in POWERS:
		if p[0] == t:
			tool_hint.text = p[2] + ("" if t == "select" else ". Click the world.")
	Input.set_default_cursor_shape(Input.CURSOR_ARROW if t == "select" else Input.CURSOR_CROSS)


func _instant_power(p: String) -> void:
	var c := cam.get_screen_center_position()
	_use_power(p, Vector2i(floori(c.x / view.TILE), floori(c.y / view.TILE)))


func _use_power(p: String, tile: Vector2i) -> void:
	var msg: String = world.god(p, tile.x, tile.y)
	var col := Color("#e9e4d8")
	for e in POWERS:
		if e[0] == p:
			col = Color(e[3])
	if p == "lightning":
		view.add_flash(tile)
	elif p in ["create_plant", "add_food", "add_water", "fire", "lightning"]:
		view.refresh_terrain()
	if msg != "":
		view.add_effect(tile, col, "")
		_toast(msg, col)
	if p in ["add_water", "create_plant", "lightning", "fire"]:
		view.refresh_terrain()


func _select(id: int) -> void:
	selected = id
	world.select(id)
	if id < 0:
		inspector.visible = false
		hud.get_child(0).get_node("Help").visible = true
		_set_follow(false)
		return
	inspector.visible = true
	hud.get_child(0).get_node("Help").visible = false
	inspector.show_json(world.inspect(id))
	inspect_timer = 0.0


func _set_follow(on: bool) -> void:
	cam.following = on and selected >= 0
	inspector.follow_btn.set_pressed_no_signal(cam.following)


func _toast(text: String, col := UI.TEXT) -> void:
	var p := PanelContainer.new()
	var s := UI.box(Color(UI.BG, 0.9), 9, col, 1, 10)
	p.add_theme_stylebox_override("panel", s)
	var l := UI.label(text, 13, col)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	p.add_child(l)
	p.mouse_filter = Control.MOUSE_FILTER_IGNORE
	toasts.add_child(p)
	while toasts.get_child_count() > 4:
		var old := toasts.get_child(0)
		toasts.remove_child(old)
		old.queue_free()
	var tw := create_tween()
	tw.tween_interval(3.5)
	tw.tween_property(p, "modulate:a", 0.0, 0.8)
	tw.tween_callback(p.queue_free)


# ---------------------------------------------------------------- frame

func _process(delta: float) -> void:
	if world == null:
		return
	if world.is_loading():
		var p: float = world.load_step(30.0)
		load_bar.value = p
		load_label.text = "Rebuilding the world from its journal… %d%%" % roundi(p * 100.0)
		if p >= 1.0:
			_finish_load()
		return
	var budget := 12.0 if SPEEDS[speed_idx] < 1000 else 22.0
	world.advance(delta, SPEEDS[speed_idx], budget)
	view.sync(delta)
	# Day / night.
	var l: float = world.light()
	night.color = Color(0.32, 0.36, 0.6).lerp(Color.WHITE, clampf(l, 0.0, 1.0))
	if world.weather() >= 2:
		night.color = night.color.darkened(0.15)
	# Follow.
	if selected >= 0:
		var p: Vector2 = view.pos_of(selected)
		if p.x >= 0:
			cam.follow_target = p
	status_timer -= delta
	if status_timer <= 0.0:
		status_timer = 0.25
		_update_status()
	history_timer -= delta
	if history_timer <= 0.0:
		history_timer = 0.5
		_update_history()
	if selected >= 0:
		inspect_timer -= delta
		if inspect_timer <= 0.0:
			inspect_timer = 0.5
			inspector.show_json(world.inspect(selected))
	if world.tick() - last_terrain_tick > 3600:
		last_terrain_tick = world.tick()
		view.refresh_terrain()
	_screenshot_hook()


func _update_status() -> void:
	var s = JSON.parse_string(world.status())
	if typeof(s) != TYPE_DICTIONARY:
		return
	status = s
	clock_lbl.text = "%s  ·  Year %d" % [s.clock, int(s.year)]
	info_lbl.text = "%s · %s · %.0f°C" % [s.season, s.weather, float(s.temp)]
	pop_lbl.text = "👤 %d   🦌 %d   🐺 %d   ✦ %d born  ✝ %d died" % [int(s.humans), int(s.grazers), int(s.wolves), int(s.births), int(s.deaths)]
	var want: int = SPEEDS[speed_idx]
	if want == 0:
		speed_lbl.text = "Paused"
		speed_lbl.add_theme_color_override("font_color", UI.MUTED)
	else:
		var actual := float(s.actual_speed)
		speed_lbl.text = "running %.0f× (asked %d×)" % [actual, want] if actual < want * 0.9 else "running %d×" % want
		speed_lbl.add_theme_color_override("font_color", UI.TEXT if actual >= want * 0.9 else UI.ACCENT)
	fid_lbl.text = str(s.fidelity)


func _update_history() -> void:
	var arr = JSON.parse_string(world.history(history_count))
	if typeof(arr) != TYPE_ARRAY or arr.is_empty():
		return
	var initial: bool = history_count == 0 or arr.size() > 8
	var at_bottom := history_scroll.scroll_vertical >= history_scroll.get_v_scroll_bar().max_value - history_scroll.size.y - 30
	for e in arr:
		history_count = int(e.i) + 1
		var col := _hist_color(e.kind)
		var h := HBoxContainer.new()
		h.add_theme_constant_override("separation", 8)
		var w := UI.label(e.when, 11, UI.MUTED)
		w.custom_minimum_size.x = 82
		h.add_child(w)
		var dot := ColorRect.new()
		dot.color = col
		dot.custom_minimum_size = Vector2(4, 14)
		dot.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		h.add_child(dot)
		var t := UI.label(e.text, 12, UI.TEXT if e.important else Color(UI.TEXT, 0.75), true)
		t.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		h.add_child(t)
		var b := Button.new()
		b.flat = true
		b.text = "◎"
		b.tooltip_text = "Look here"
		b.focus_mode = Control.FOCUS_NONE
		var target: Vector2 = (Vector2(float(e.x), float(e.y)) + Vector2(0.5, 0.5)) * view.TILE
		b.pressed.connect(func(): _set_follow(false); cam.position = target)
		h.add_child(b)
		history_box.add_child(h)
		if not initial and e.important and e.kind in ["birth", "death", "first", "knowledge", "bond", "lightning"]:
			_toast(e.text, col)
	while history_box.get_child_count() > 300:
		var old := history_box.get_child(0)
		history_box.remove_child(old)
		old.queue_free()
	if at_bottom:
		await get_tree().process_frame
		history_scroll.scroll_vertical = int(history_scroll.get_v_scroll_bar().max_value)


func _hist_color(kind: String) -> Color:
	match kind:
		"birth": return UI.GOOD
		"death": return UI.BAD
		"first": return UI.ACCENT
		"knowledge": return UI.MAGIC
		"bond", "pregnancy": return Color("#e88aa0")
		"god": return Color("#f2d27a")
		"lightning", "weather": return UI.COOL
		"attack": return Color("#f0782a")
	return UI.MUTED


# ---------------------------------------------------------------- automated screenshots

func _screenshot_hook() -> void:
	if shot_path == "":
		return
	var f := Engine.get_process_frames()
	if auto_select and f == 20:
		# Prefer a human with a family (parent or children) for the screenshot.
		var c: PackedFloat32Array = world.creatures()
		var pick := -1
		for i in c.size() / 16:
			if int(c[i * 16 + 1]) == 0:
				var id := int(c[i * 16])
				if pick < 0:
					pick = id
				var d = JSON.parse_string(world.inspect(id))
				if d is Dictionary and (d.family.mother is Dictionary or d.family.children.size() > 0):
					pick = id
					break
		if pick >= 0:
			_select(pick)
			_set_follow(true)
	for k in shot_gods.size():
		if f == 30 + k * 6:
			var c := cam.get_screen_center_position()
			var off := Vector2((k % 3 - 1) * 60, (k / 3 - 1) * 50)
			_use_power(shot_gods[k], Vector2i(floori((c.x + off.x) / view.TILE), floori((c.y + off.y) / view.TILE)))
	if shot_saveload and f == 40:
		_save_world()
		var d := DirAccess.open(SAVE_DIR)
		var files := Array(d.get_files())
		files.sort()
		_load_file("%s/%s" % [SAVE_DIR, files[-1]])
	if shot_click.x >= 0 and f == 25:
		# Drive the real input path: a left click at a screen position.
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = true
		ev.position = shot_click
		ev.global_position = shot_click
		Input.warp_mouse(shot_click)
		Input.parse_input_event(ev)
	if shot_tab >= 0 and f == 22:
		inspector.select_tab(shot_tab)
	if f >= shot_frames:
		shot_frames = 1 << 30
		var img := get_viewport().get_texture().get_image()
		img.save_png(shot_path)
		print("saved screenshot ", shot_path)
		get_tree().quit()


# ---------------------------------------------------------------- save / load

const SAVE_DIR := "user://saves"
var load_menu: PanelContainer
var load_list: VBoxContainer
var load_overlay: PanelContainer
var load_bar: ProgressBar
var load_label: Label


func _build_load_ui(root: Control) -> void:
	load_menu = UI.card("Load a world")
	load_menu.set_anchors_preset(Control.PRESET_CENTER)
	load_menu.custom_minimum_size = Vector2(420, 0)
	load_menu.visible = false
	root.add_child(load_menu)
	var body := UI.card_body(load_menu)
	load_list = VBoxContainer.new()
	load_list.add_theme_constant_override("separation", 4)
	body.add_child(load_list)
	var cancel := UI.button("Cancel")
	cancel.pressed.connect(func(): load_menu.visible = false)
	body.add_child(cancel)
	load_overlay = UI.card("Loading")
	load_overlay.set_anchors_preset(Control.PRESET_CENTER)
	load_overlay.custom_minimum_size = Vector2(460, 0)
	load_overlay.visible = false
	root.add_child(load_overlay)
	var ob := UI.card_body(load_overlay)
	load_label = UI.label("", 13, UI.TEXT, true)
	ob.add_child(load_label)
	load_bar = ProgressBar.new()
	load_bar.max_value = 1.0
	load_bar.show_percentage = false
	load_bar.custom_minimum_size = Vector2(400, 12)
	ob.add_child(load_bar)
	ob.add_child(UI.label("A save stores the world's seed and every act of God, so loading replays history exactly.", 11, UI.MUTED, true))


func _save_world() -> void:
	DirAccess.make_dir_recursive_absolute(SAVE_DIR)
	var s = JSON.parse_string(world.status())
	var name := "%s/%s.gsave" % [SAVE_DIR, Time.get_datetime_string_from_system().replace(":", "-")]
	var f := FileAccess.open(name, FileAccess.WRITE)
	if f == null:
		_toast("Could not save: %s" % error_string(FileAccess.get_open_error()), UI.BAD)
		return
	f.store_string(world.save_text())
	f.close()
	_toast("Saved (%s, year %d)" % [s.clock if s is Dictionary else "", int(s.year) if s is Dictionary else 0], UI.GOOD)


func _show_load_menu() -> void:
	UI.clear(load_list)
	var files := []
	var d := DirAccess.open(SAVE_DIR)
	if d:
		for f in d.get_files():
			if f.ends_with(".gsave"):
				files.append(f)
	files.sort()
	files.reverse()
	if files.is_empty():
		load_list.add_child(UI.label("No saved worlds yet. Press Save (F5) first.", 13, UI.MUTED, true))
	for f in files.slice(0, 12):
		var b := UI.button(f.trim_suffix(".gsave").replace("T", "  "))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.pressed.connect(_load_file.bind("%s/%s" % [SAVE_DIR, f]))
		load_list.add_child(b)
	load_menu.visible = true


func _load_file(path: String) -> void:
	load_menu.visible = false
	var text := FileAccess.get_file_as_string(path)
	if not world.begin_load(text):
		_toast("Could not load: %s" % world.load_status(), UI.BAD)
		return
	_select(-1)
	load_overlay.visible = true


func _finish_load() -> void:
	load_overlay.visible = false
	view.refresh_terrain()
	view.draw_pos.clear()
	UI.clear(history_box)
	history_count = 0
	_set_speed(0)
	_focus_on_humans()
	var ok: bool = world.load_status().begins_with("Loaded. ")
	_toast(world.load_status(), UI.GOOD if ok else UI.BAD)
