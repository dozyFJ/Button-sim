extends Node3D

const SAVE_FILE := "user://button_sim_save.json"
var money: float = 0.0
var multiplier: int = 1
var base_income: float = 1.0
var upgrade_level: int = 0
var active_tab: int = 0
var button_mesh: MeshInstance3D
var money_label: Label
var mult_label: Label
var rate_label: Label
var panel: PanelContainer
var panel_title: Label
var panel_body: Label
var dock_buttons: Array[Button] = []
var tap_area: Button
var click_anim: Tween

func _ready() -> void:
	_load_save()
	_build_world()
	_build_gui()
	_refresh()
	var timer := Timer.new()
	timer.wait_time = 0.1
	timer.autostart = true
	add_child(timer)
	timer.timeout.connect(_earn_passive)

func _style(fill: Color, edge: Color, radius: int = 22) -> StyleBoxFlat:
	var s := StyleBoxFlat.new()
	s.bg_color = fill
	s.border_color = edge
	s.set_border_width_all(5)
	s.set_corner_radius_all(radius)
	s.shadow_color = Color(0.04, 0.09, 0.22, 0.55)
	s.shadow_size = 8
	s.shadow_offset = Vector2(0, 5)
	s.content_margin_left = 12
	s.content_margin_right = 12
	s.content_margin_top = 12
	s.content_margin_bottom = 12
	return s

func _box(parent: Node3D, pos: Vector3, scale_size: Vector3, tint: Color) -> MeshInstance3D:
	var obj := MeshInstance3D.new()
	var mesh := BoxMesh.new()
	mesh.size = scale_size
	obj.mesh = mesh
	var mat := StandardMaterial3D.new()
	mat.albedo_color = tint
	mat.metallic = 0.12
	mat.roughness = 0.42
	obj.material_override = mat
	obj.position = pos
	parent.add_child(obj)
	return obj

func _build_world() -> void:
	var env_node := WorldEnvironment.new()
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color("#263d76")
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color("#ffffff")
	env.ambient_light_energy = 0.85
	env_node.environment = env
	add_child(env_node)
	var light := DirectionalLight3D.new()
	light.rotation_degrees = Vector3(-48, -30, 0)
	light.light_energy = 1.3
	add_child(light)
	var cam := Camera3D.new()
	cam.position = Vector3(0, 5.5, 9)
	add_child(cam)
	cam.look_at(Vector3(0, 0.8, 0), Vector3.UP)
	cam.current = true
	_box(self, Vector3(0, -0.65, 0), Vector3(14, 0.6, 14), Color("#76c7a0"))
	_box(self, Vector3(0, -0.31, 0), Vector3(5.3, 0.16, 5.3), Color("#ffe2a1"))
	_box(self, Vector3(0, 0.08, 0), Vector3(3.8, 0.7, 3.8), Color("#254f95"))
	_box(self, Vector3(0, 0.47, 0), Vector3(3.5, 0.18, 3.5), Color("#ffd56d"))
	button_mesh = MeshInstance3D.new()
	var cyl := CylinderMesh.new()
	cyl.top_radius = 1.35
	cyl.bottom_radius = 1.45
	cyl.height = 0.52
	button_mesh.mesh = cyl
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color("#ff525f")
	mat.metallic = 0.18
	mat.roughness = 0.28
	button_mesh.material_override = mat
	button_mesh.position = Vector3(0, 0.84, 0)
	add_child(button_mesh)
	_box(self, Vector3(-3.7, 0.65, -2.7), Vector3(0.65, 1.7, 0.65), Color("#75b9ff"))
	_box(self, Vector3(3.7, 0.65, -2.7), Vector3(0.65, 1.7, 0.65), Color("#75b9ff"))

func _make_label(parent: Node, value: String, size: int, color: Color = Color.WHITE) -> Label:
	var label := Label.new()
	label.text = value
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", color)
	label.add_theme_color_override("font_shadow_color", Color("#17224a"))
	label.add_theme_constant_override("shadow_offset_x", 2)
	label.add_theme_constant_override("shadow_offset_y", 3)
	parent.add_child(label)
	return label

func _build_gui() -> void:
	var layer := CanvasLayer.new()
	add_child(layer)
	var root := Control.new()
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	layer.add_child(root)
	var top := PanelContainer.new()
	top.anchor_left = 0.035
	top.anchor_right = 0.965
	top.offset_top = 35
	top.offset_bottom = 224
	top.add_theme_stylebox_override("panel", _style(Color("#234b95"), Color("#ffcb64")))
	root.add_child(top)
	var stats := VBoxContainer.new()
	stats.add_theme_constant_override("separation", 5)
	top.add_child(stats)
	_make_label(stats, "BUTTON SIM", 34, Color("#ffe08b"))
	money_label = _make_label(stats, "", 36)
	mult_label = _make_label(stats, "", 28, Color("#fff0a3"))
	rate_label = _make_label(stats, "", 22, Color("#a8ffcf"))
	tap_area = Button.new()
	tap_area.anchor_left = 0.18
	tap_area.anchor_right = 0.82
	tap_area.anchor_top = 0.32
	tap_area.anchor_bottom = 0.75
	tap_area.text = "ЖМИ!"
	tap_area.add_theme_font_size_override("font_size", 54)
	tap_area.add_theme_color_override("font_color", Color.WHITE)
	tap_area.add_theme_color_override("font_shadow_color", Color("#8c203c"))
	tap_area.add_theme_constant_override("shadow_offset_y", 4)
	tap_area.add_theme_stylebox_override("normal", _style(Color(1, 0.3, 0.4, 0.14), Color(1, 1, 1, 0), 80))
	tap_area.add_theme_stylebox_override("pressed", _style(Color(1, 0.3, 0.4, 0.2), Color(1, 1, 1, 0), 80))
	tap_area.add_theme_stylebox_override("focus", StyleBoxEmpty.new())
	tap_area.pressed.connect(_tap)
	root.add_child(tap_area)
	panel = PanelContainer.new()
	panel.anchor_left = 0.045
	panel.anchor_right = 0.955
	panel.anchor_top = 0.28
	panel.anchor_bottom = 0.82
	panel.add_theme_stylebox_override("panel", _style(Color("#234b95"), Color("#ffd16e")))
	root.add_child(panel)
	var content := VBoxContainer.new()
	content.add_theme_constant_override("separation", 28)
	panel.add_child(content)
	panel_title = _make_label(content, "", 34, Color("#ffe08b"))
	panel_body = _make_label(content, "", 26)
	panel_body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var buy := Button.new()
	buy.name = "Buy"
	buy.text = "УЛУЧШИТЬ ДОХОД"
	buy.custom_minimum_size.y = 92
	buy.add_theme_font_size_override("font_size", 26)
	buy.add_theme_stylebox_override("normal", _style(Color("#30af69"), Color("#b8ff92")))
	buy.add_theme_stylebox_override("pressed", _style(Color("#21834d"), Color("#b8ff92")))
	buy.pressed.connect(_upgrade)
	content.add_child(buy)
	var dock := PanelContainer.new()
	dock.anchor_left = 0.015
	dock.anchor_right = 0.985
	dock.anchor_top = 1.0
	dock.anchor_bottom = 1.0
	dock.offset_top = -144
	dock.offset_bottom = -14
	dock.add_theme_stylebox_override("panel", _style(Color("#19396e"), Color("#f9cc72")))
	root.add_child(dock)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	dock.add_child(row)
	for i in range(4):
		var names := ["ГЛАВНАЯ", "КАПСУЛЫ", "УЛУЧШ.", "НАСТР."]
		var b := Button.new()
		b.text = names[i]
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.custom_minimum_size.y = 86
		b.add_theme_font_size_override("font_size", 19)
		b.add_theme_color_override("font_color", Color.WHITE)
		b.add_theme_stylebox_override("focus", StyleBoxEmpty.new())
		b.pressed.connect(_switch_tab.bind(i))
		row.add_child(b)
		dock_buttons.append(b)
	_switch_tab(0)

func _tap() -> void:
	multiplier += 1
	if click_anim and click_anim.is_running():
		click_anim.kill()
	button_mesh.scale = Vector3.ONE
	click_anim = create_tween()
	click_anim.tween_property(button_mesh, "scale", Vector3(1.1, 0.65, 1.1), 0.09)
	click_anim.tween_property(button_mesh, "scale", Vector3.ONE, 0.17).set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
	_refresh()
	_save()

func _earn_passive() -> void:
	money += base_income * float(multiplier) * 0.1
	_refresh()

func _upgrade() -> void:
	var cost := 50.0 * pow(2.0, upgrade_level)
	if money < cost:
		panel_body.text = "Недостаточно монет!\nНужно: " + str(int(ceil(cost)))
		return
	money -= cost
	upgrade_level += 1
	base_income += 1.0
	_refresh()
	_save()

func _switch_tab(tab: int) -> void:
	active_tab = tab
	panel.visible = tab != 0
	tap_area.visible = tab == 0
	button_mesh.visible = tab == 0
	for i in range(dock_buttons.size()):
		dock_buttons[i].add_theme_stylebox_override("normal", _style(Color("#f0a833") if i == tab else Color("#3578c3"), Color("#ffdf83") if i == tab else Color("#85c7ff"), 18))
		dock_buttons[i].add_theme_stylebox_override("pressed", _style(Color("#d98829"), Color("#ffdf83"), 18))
	if tab == 1:
		panel_title.text = "КАПСУЛЫ"
		panel_body.text = "Скоро здесь будут 3D-капсулы и питомцы."
	elif tab == 2:
		panel_title.text = "УЛУЧШЕНИЯ"
		panel_body.text = "Увеличь базовый доход за монеты."
	elif tab == 3:
		panel_title.text = "НАСТРОЙКИ"
		panel_body.text = "Button Sim • Первая версия"
	if panel.visible:
		panel.get_node("VBoxContainer/Buy").visible = tab == 2
	_refresh()

func _refresh() -> void:
	if money_label == null:
		return
	money_label.text = "МОНЕТЫ: " + str(int(floor(money)))
	mult_label.text = "МНОЖИТЕЛЬ: x" + str(multiplier)
	rate_label.text = "+" + str(int(base_income * multiplier)) + " монет/сек"
	if active_tab == 2:
		var cost := int(ceil(50.0 * pow(2.0, upgrade_level)))
		panel_body.text = "Базовый доход: " + str(int(base_income)) + "/сек\nСледующее улучшение: " + str(cost) + " монет"

func _save() -> void:
	var file := FileAccess.open(SAVE_FILE, FileAccess.WRITE)
	if file:
		file.store_string(JSON.stringify({"money": money, "multiplier": multiplier, "base_income": base_income, "upgrade_level": upgrade_level}))

func _load_save() -> void:
	if not FileAccess.file_exists(SAVE_FILE):
		return
	var file := FileAccess.open(SAVE_FILE, FileAccess.READ)
	if file == null:
		return
	var parsed = JSON.parse_string(file.get_as_text())
	if parsed is Dictionary:
		money = float(parsed.get("money", 0.0))
		multiplier = maxi(1, int(parsed.get("multiplier", 1)))
		base_income = maxf(1.0, float(parsed.get("base_income", 1.0)))
		upgrade_level = maxi(0, int(parsed.get("upgrade_level", 0)))

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST or what == NOTIFICATION_APPLICATION_PAUSED:
		_save()
