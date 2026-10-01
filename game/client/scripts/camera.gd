extends Camera2D
## God camera: WASD/arrow or edge pan, right/middle-drag pan, wheel zoom toward cursor,
## and follow mode for the selected creature.

signal user_moved

var bounds := Rect2(0, 0, 3200, 2240)
var follow_target := Vector2(-1, -1)
var following := false
var target_zoom := 1.0
var dragging := false
var edge_pan := true

const MIN_ZOOM := 0.3
const MAX_ZOOM := 4.0


func _ready() -> void:
	target_zoom = zoom.x


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.pressed and (mb.button_index == MOUSE_BUTTON_WHEEL_UP or mb.button_index == MOUSE_BUTTON_WHEEL_DOWN):
			var f := 1.15 if mb.button_index == MOUSE_BUTTON_WHEEL_UP else 1.0 / 1.15
			_zoom_at(clampf(target_zoom * f, MIN_ZOOM, MAX_ZOOM), mb.position)
			get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_RIGHT or mb.button_index == MOUSE_BUTTON_MIDDLE:
			dragging = mb.pressed
	elif event is InputEventMouseMotion and dragging:
		position -= (event as InputEventMouseMotion).relative / zoom.x
		_stop_follow()
	elif event is InputEventMagnifyGesture:
		_zoom_at(clampf(target_zoom * (event as InputEventMagnifyGesture).factor, MIN_ZOOM, MAX_ZOOM), get_viewport().get_mouse_position())
	elif event is InputEventPanGesture:
		position += (event as InputEventPanGesture).delta * 20.0 / zoom.x
		_stop_follow()


func _zoom_at(z: float, screen_pos: Vector2) -> void:
	var before := get_canvas_transform().affine_inverse() * screen_pos
	target_zoom = z
	zoom = Vector2.ONE * z
	var after := get_canvas_transform().affine_inverse() * screen_pos
	if not following:
		position += before - after


func _stop_follow() -> void:
	if following:
		following = false
		user_moved.emit()


func _process(delta: float) -> void:
	var move := Vector2.ZERO
	if not _typing():
		if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT): move.x -= 1
		if Input.is_key_pressed(KEY_D) or Input.is_key_pressed(KEY_RIGHT): move.x += 1
		if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP): move.y -= 1
		if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN): move.y += 1
	if edge_pan and DisplayServer.window_is_focused():
		var m := get_viewport().get_mouse_position()
		var vs := get_viewport_rect().size
		if m.x >= 0 and m.y >= 0 and m.x <= vs.x and m.y <= vs.y:
			if m.x < 6: move.x -= 1
			if m.x > vs.x - 6: move.x += 1
			if m.y < 6: move.y -= 1
			if m.y > vs.y - 6: move.y += 1
	if move != Vector2.ZERO:
		position += move.normalized() * 700.0 * delta / zoom.x
		_stop_follow()
	if following and follow_target.x >= 0:
		position = position.lerp(follow_target, clampf(delta * 6.0, 0.0, 1.0))
	# Keep the view over the world (centre it when the world is smaller than the view).
	var half := get_viewport_rect().size / zoom.x / 2.0
	for ax in 2:
		var lo: float = bounds.position[ax] + half[ax]
		var hi: float = bounds.end[ax] - half[ax]
		position[ax] = (bounds.position[ax] + bounds.end[ax]) / 2.0 if lo > hi else clampf(position[ax], lo, hi)


func _typing() -> bool:
	var f := get_viewport().gui_get_focus_owner()
	return f is LineEdit or f is TextEdit
