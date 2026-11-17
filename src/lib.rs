//! Button Sim — WolfCraft Rust/wgpu-based portrait clicker.
pub mod game;
mod pet_render;
mod pet_ui;

use std::{sync::Arc, time::Instant};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}};
use wolfcraft_common::gpu::GpuContext;

struct Graphics {
    gpu: GpuContext,
    egui: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gfx: Option<Graphics>,
    game: game::Game,
    last_tick: Option<Instant>,
    save_path: Option<std::path::PathBuf>,
    last_save: Option<Instant>,
}

fn save_game(game: &game::Game, path: Option<&std::path::Path>) {
    let Some(path) = path else { return; };
    let result = (|| -> std::io::Result<()> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let temp = path.with_extension("tmp");
        std::fs::write(&temp, game.save_text())?;
        std::fs::rename(temp, path)?;
        Ok(())
    })();
    if let Err(error) = result { log::warn!("Cannot save Button Sim: {error}"); }
}
impl App {
    fn with_save_path(path: std::path::PathBuf) -> Self {
        let game = std::fs::read_to_string(&path).ok().and_then(|text| game::Game::from_save(&text)).unwrap_or_default();
        Self { game, save_path: Some(path), ..Self::default() }
    }
}

fn amount(n: f64) -> String {
    if n >= 1_000_000_000.0 { format!("{:.1}B", n / 1_000_000_000.0) }
    else if n >= 1_000_000.0 { format!("{:.1}M", n / 1_000_000.0) }
    else if n >= 10_000.0 { format!("{:.1}K", n / 1_000.0) }
    else { format!("{:.0}", n.floor()) }
}

fn aero_card(p: &egui::Painter, r: egui::Rect, accent: egui::Color32) {
    use egui::{Color32 as C, Stroke, Rect, vec2};
    for (spread, alpha) in [(6.0, 10), (3.0, 22), (1.0, 40)] {
        p.rect_stroke(r.expand(spread), 16.0 + spread, Stroke::new(2.0, C::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha)));
    }
    p.rect_filled(r, 16.0, C::from_rgba_unmultiplied(35, 63, 104, 210));
    let shine = Rect::from_min_max(r.min + vec2(2.0, 2.0), egui::pos2(r.right() - 2.0, r.center().y));
    p.rect_filled(shine, 14.0, C::from_rgba_unmultiplied(172, 223, 255, 20));
    p.rect_stroke(r, 16.0, Stroke::new(1.0, accent.gamma_multiply(0.7)));
    p.line_segment([r.left_top() + vec2(16.0, 2.0), r.right_top() + vec2(-16.0, 2.0)], Stroke::new(1.0, C::from_rgba_unmultiplied(227, 250, 255, 130)));
}

fn nav_icon(p: &egui::Painter, center: egui::Pos2, index: usize, color: egui::Color32) {
    use egui::{vec2, Rect, Stroke};
    let s = Stroke::new(2.0, color);
    match index {
        0 => {
            p.line_segment([center + vec2(-11.0, -1.0), center + vec2(0.0, -11.0)], s);
            p.line_segment([center + vec2(0.0, -11.0), center + vec2(11.0, -1.0)], s);
            p.rect_stroke(Rect::from_center_size(center + vec2(0.0, 4.0), vec2(15.0, 14.0)), 2.0, s);
        }
        1 => {
            p.rect_stroke(Rect::from_center_size(center, vec2(17.0, 25.0)), 8.0, s);
            p.line_segment([center + vec2(-8.0, 0.0), center + vec2(8.0, 0.0)], s);
            p.circle_filled(center, 3.0, color);
        }
        2 => {
            p.line_segment([center + vec2(0.0, 11.0), center + vec2(0.0, -10.0)], s);
            p.line_segment([center + vec2(-8.0, -2.0), center + vec2(0.0, -10.0)], s);
            p.line_segment([center + vec2(8.0, -2.0), center + vec2(0.0, -10.0)], s);
        }
        3 => {
            for (x, h) in [(-8.0, 9.0), (0.0, 16.0), (8.0, 23.0)] {
                p.rect_filled(Rect::from_min_size(center + vec2(x - 2.0, 11.0 - h), vec2(4.0, h)), 1.0, color);
            }
        }
        _ => {
            p.circle_stroke(center, 10.0, s);
            p.line_segment([center + vec2(10.0, -10.0), center + vec2(10.0, 0.0)], s);
            p.line_segment([center + vec2(10.0, 0.0), center + vec2(0.0, 0.0)], s);
        }
    }
}

fn ui(ctx: &egui::Context, game: &mut game::Game) {
    use egui::{Color32 as C, RichText as T, Stroke, pos2, vec2, Rect, Align2, FontId, Sense};
    let navy = C::from_rgb(9, 14, 33);
    let panel = C::from_rgba_unmultiplied(30, 53, 87, 220);
    let gold = C::from_rgb(99, 241, 255);
    let muted = C::from_rgb(161, 190, 220);
    let green = C::from_rgb(133, 255, 209);
    let width = ctx.screen_rect().width();
    let was_paused = game.paused;
    let blocked = was_paused || game.hatch.is_some() || game.pets_open;
    egui::TopBottomPanel::top("profile_header")
        .frame(egui::Frame::none().fill(navy).inner_margin(egui::Margin::same(12.0)))
        .show(ctx, |ui| {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 108.0), Sense::hover());
            let p = ui.painter();
            p.line_segment([pos2(r.left()-12.0,r.bottom()+11.0),pos2(r.right()+12.0,r.bottom()+11.0)], Stroke::new(1.0, C::from_rgb(47,71,100)));
            let avatar = pos2(r.left() + 42.0, r.top() + 24.0);
            for radius in [29.0, 26.0, 23.0] {
                p.circle_stroke(avatar, radius, Stroke::new(2.0, C::from_rgba_unmultiplied(80, 225, 255, 25)));
            }
            p.circle_filled(avatar, 22.0, C::from_rgb(39, 71, 119));
            p.circle_stroke(avatar, 22.0, Stroke::new(2.0, gold));
            p.circle_filled(avatar + vec2(0.0, -5.0), 7.0, C::from_rgb(217, 232, 249));
            p.rect_filled(Rect::from_center_size(avatar + vec2(0.0, 10.0), vec2(23.0, 12.0)), 6.0, C::from_rgb(217, 232, 249));
            p.text(pos2(r.left() + 42.0, r.top() + 59.0), Align2::CENTER_CENTER, game.rank(), FontId::proportional(11.0), gold);
            let card = Rect::from_min_max(pos2(r.left() + 98.0, r.top() + 4.0), pos2(r.right() - 53.0, r.top() + 65.0));
            p.rect_filled(card, 0.0, C::from_rgb(19, 35, 59));
            let money_icon = pos2(card.left() + 21.0, card.center().y);
            p.rect_filled(Rect::from_center_size(money_icon + vec2(-2.0, 3.0), vec2(27.0, 17.0)), 1.0, C::from_rgb(43, 126, 109));
            p.rect_filled(Rect::from_center_size(money_icon, vec2(27.0, 17.0)), 1.0, green);
            p.rect_stroke(Rect::from_center_size(money_icon, vec2(21.0, 11.0)), 0.0, Stroke::new(1.0, navy));
            p.circle_filled(money_icon, 3.5, navy);
            let text_x = card.left() + 43.0;
            p.text(pos2(text_x, card.top() + 12.0), Align2::LEFT_CENTER, "ДЕНЬГИ", FontId::proportional(10.0), muted);
            p.text(pos2(text_x, card.top() + 31.0), Align2::LEFT_CENTER, amount(game.coins), FontId::proportional(22.0), C::WHITE);
            p.text(pos2(text_x, card.bottom() - 10.0), Align2::LEFT_CENTER, format!("+{} / сек", amount(game.coins_per_second())), FontId::proportional(10.0), green);
            let rebirth_card = Rect::from_min_max(pos2(card.left(), r.top() + 72.0), pos2(card.right(), r.top() + 102.0));
            p.rect_filled(rebirth_card, 0.0, C::from_rgb(36, 29, 68));
            let rebirth_icon = pos2(money_icon.x, rebirth_card.center().y);
            let purple = C::from_rgb(211, 165, 255);
            p.circle_stroke(rebirth_icon, 8.0, Stroke::new(1.8, purple));
            p.line_segment([rebirth_icon + vec2(8.0,-8.0), rebirth_icon + vec2(8.0,0.0)], Stroke::new(2.0,purple));
            p.line_segment([rebirth_icon + vec2(8.0,0.0), rebirth_icon], Stroke::new(2.0,purple));
            p.text(pos2(text_x, rebirth_card.center().y - 6.0), Align2::LEFT_CENTER, "РЕБЁРТЫ", FontId::proportional(9.0), muted);
            p.text(pos2(text_x, rebirth_card.center().y + 7.0), Align2::LEFT_CENTER, game.rebirths.to_string(), FontId::proportional(13.0), C::WHITE);
            let pause = Rect::from_min_size(pos2(r.right() - 42.0, r.top() + 10.0), vec2(42.0, 42.0));
            p.rect_filled(pause, 0.0, C::from_rgba_unmultiplied(150, 105, 255, 35));
            for x in [-5.0, 5.0] {
                p.rect_filled(Rect::from_center_size(pause.center() + vec2(x, 0.0), vec2(4.0, 16.0)), 1.0, gold);
            }
            if ui.interact(pause, ui.id().with("pause"), Sense::click()).clicked() { game.paused = !game.paused; }
        });

    egui::TopBottomPanel::bottom("navigation").exact_height(170.0)
        .frame(egui::Frame::none().fill(navy).inner_margin(egui::Margin::same(8.0)))
        .show(ctx, |ui| {
            let dock = ui.max_rect();
            ui.painter().line_segment([pos2(dock.left()-8.0,dock.top()-8.0),pos2(dock.right()+8.0,dock.top()-8.0)], Stroke::new(1.0,C::from_rgb(47,71,100)));
            let team_count = game.equipped_pets.len();
            let attack_label = if team_count == 0 { "АВТОАТАКА · ВЫБРАТЬ ПИТОМЦЕВ".to_owned() }
                else { format!("АВТОАТАКА · {} · {}/3", if game.auto_attack { "ВКЛ" } else { "ВЫКЛ" }, team_count) };
            let attack_color = if game.auto_attack { green } else { gold };
            if ui.add_enabled(!blocked && !game.paused,
                egui::Button::new(T::new(attack_label).size(13.0).strong().color(navy))
                    .fill(attack_color).rounding(0.0).min_size(vec2(ui.available_width(), 42.0))).clicked() {
                if team_count == 0 { game.pets_open = true; } else { game.toggle_auto_attack(); }
            }
            ui.add_space(5.0);
            let (level_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
            let bar = Rect::from_min_size(level_rect.left_top() + vec2(10.0, 14.0), vec2(level_rect.width() - 20.0, 6.0));
            ui.painter().rect_filled(bar, 3.0, C::from_rgb(35, 49, 77));
            if game.level_progress() > 0.0 {
                ui.painter().rect_filled(Rect::from_min_size(bar.min, vec2(bar.width() * game.level_progress(), 6.0)), 3.0, gold);
            }
            ui.painter().text(level_rect.left_top() + vec2(10.0, 6.0), Align2::LEFT_CENTER, format!("УРОВЕНЬ {}", game.level()), FontId::proportional(10.0), C::WHITE);
            ui.painter().text(level_rect.right_top() + vec2(-10.0, 6.0), Align2::RIGHT_CENTER, format!("{} / 100 XP", game.taps % 100), FontId::proportional(10.0), gold);
            ui.add_enabled_ui(!blocked && !game.paused, |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.columns(5, |cols| {
                    for (i, (label, tab)) in [
                        ("Главная", game::Tab::Home),
                        ("Капсулы", game::Tab::Capsules),
                        ("Улучшения", game::Tab::Upgrades),
                        ("Статистика", game::Tab::Statistics),
                        ("Ребёрты", game::Tab::Rebirths),
                    ].into_iter().enumerate() {
                        let col = &mut cols[i];
                        let (rect, response) = col.allocate_exact_size(vec2(col.available_width(), 68.0), Sense::click());
                        let active = game.tab == tab;
                        let p = col.painter();
                        if active { p.rect_filled(rect.shrink(3.0), 0.0, C::from_rgba_unmultiplied(98, 226, 255, 30)); }
                        if active { p.rect_filled(Rect::from_min_size(rect.min + vec2(12.0, 0.0), vec2((rect.width() - 24.0).max(4.0), 3.0)), 2.0, gold); }
                        nav_icon(p, rect.center() - vec2(0.0, 10.0), i, if active { gold } else { muted });
                        p.text(pos2(rect.center().x, rect.bottom() - 15.0), Align2::CENTER_CENTER, label, FontId::proportional(if width < 370.0 { 9.0 } else { 11.0 }), if active { C::WHITE } else { muted });
                        if response.clicked() { game.tab = tab; }
                    }
                });
            });
        });

    egui::CentralPanel::default().frame(egui::Frame::none().fill(C::from_rgb(10, 17, 38)).inner_margin(egui::Margin::same(18.0)))
        .show(ctx, |ui| {
            let scene = ui.max_rect();
            let p = ui.painter();
            for (center, color) in [
                (scene.left_top() + vec2(15.0, 130.0), C::from_rgb(31, 143, 235)),
                (scene.right_bottom() - vec2(15.0, 130.0), C::from_rgb(168, 44, 230)),
            ] {
                for radius in [180.0, 140.0, 100.0, 60.0] {
                    p.circle_filled(center, radius, C::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 8));
                }
            }
            for row in 0..12 {
                let y = scene.bottom() - row as f32 * 28.0;
                p.line_segment([pos2(scene.left(), y), pos2(scene.right(), y)], Stroke::new(1.0, C::from_rgba_unmultiplied(88, 169, 255, 12)));
            }
            ui.visuals_mut().override_text_color = Some(C::WHITE);
            ui.add_enabled_ui(!blocked && !game.paused, |ui| {
                egui::ScrollArea::vertical().id_salt("scene_scroll").show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(18.0);
                        match game.tab {
                            game::Tab::Home => {
                                ui.label(T::new("BUTTON SIM").size(12.0).color(muted).strong());
                                ui.add_space(10.0);
                                ui.label(T::new(format!("Множитель ×{}", game.multiplier)).size(27.0).strong().color(gold));
                                ui.label(T::new("Каждое нажатие увеличивает доход").size(13.0).color(muted));
                                ui.add_space(26.0);
                                let size = (ui.available_width() * 0.64).min(220.0);
                                let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), size + 142.0), Sense::click());
                                let center = rect.center() - vec2(0.0, 10.0);
                                let pet_impact = (0..game.equipped_pets.len()).any(|slot| game.pet_is_striking(slot));
                                let pressed = response.is_pointer_button_down_on() || pet_impact;
                                let top = center + vec2(0.0, if pressed { 10.0 } else { 0.0 });
                                let p = ui.painter();
                                for spread in [22.0, 15.0, 8.0] {
                                    p.circle_stroke(center + vec2(0.0, 9.0), size * 0.48 + spread, Stroke::new(5.0, C::from_rgba_unmultiplied(52, 226, 255, 15)));
                                }
                                p.circle_filled(center + vec2(0.0, 22.0), size * 0.49, C::from_rgb(6, 12, 29));
                                p.circle_filled(center + vec2(0.0, 9.0), size * 0.48, C::from_rgb(33, 56, 93));
                                p.circle_stroke(center + vec2(0.0, 9.0), size * 0.48, Stroke::new(3.0, gold));
                                p.circle_filled(center + vec2(0.0, 10.0), size * 0.41, C::from_rgb(76, 25, 107));
                                p.circle_filled(top, size * 0.41, C::from_rgb(214, 70, 203));
                                p.circle_stroke(top, size * 0.40, Stroke::new(2.0, C::from_rgb(255, 170, 246)));
                                p.circle_filled(top - vec2(size * 0.07, size * 0.17), size * 0.25, C::from_rgba_unmultiplied(250, 202, 255, 26));
                                p.circle_filled(top - vec2(size * 0.1, size * 0.15), size * 0.16, C::from_rgba_unmultiplied(255, 185, 191, 90));
                                p.text(top, Align2::CENTER_CENTER, "ЖМИ!", FontId::proportional(36.0), C::WHITE);
                                pet_ui::attack_scene(p, rect, center, size, game);
                                if response.clicked() && (response.interact_pointer_pos().map(|point| point.distance(center) <= size * 0.48).unwrap_or(false)) { game.tap(); }
                                ui.add_space(22.0);
                                ui.label(T::new(format!("Бонус ребёртов ×{}", amount(game.rebirth_bonus()))).color(green));
                                if !game.equipped_pets.is_empty() {
                                    ui.label(T::new(format!("Команда {}/3 · доход ×{:.1}", game.equipped_pets.len(), game.pet_bonus())).color(gold));
                                }
                                ui.add_space(60.0);
                            }
                            game::Tab::Capsules => { pet_ui::shop(ui, game); }
                            game::Tab::Upgrades => {
                                ui.heading("Улучшения");
                                ui.add_space(20.0);
                                ui.label(format!("Уровень дохода: {}", game.upgrades));
                                ui.label(format!("Базовый доход: {} / сек", amount(game.base_income)));
                                ui.add_space(18.0);
                                let can_buy = game.coins >= game.upgrade_price();
                                if ui.add_enabled(can_buy, egui::Button::new(T::new(format!("Улучшить · {} денег", amount(game.upgrade_price()))).color(navy).strong()).fill(gold).rounding(12.0).min_size(vec2(ui.available_width().min(340.0), 58.0))).clicked() { game.upgrade(); }
                                if !can_buy { ui.label(T::new("Накопи деньги для улучшения").color(muted)); }
                            }
                            game::Tab::Statistics => {
                                ui.heading("Статистика");
                                ui.add_space(22.0);
                                for (label, value) in [
                                    ("Нажатия", game.taps.to_string()),
                                    ("Заработано денег", amount(game.total_earned)),
                                    ("Доход в секунду", amount(game.coins_per_second())),
                                    ("Уровень", game.level().to_string()),
                                    ("Ранг", game.rank().to_owned()),
                                    ("Ребёрты", game.rebirths.to_string()),
                                    ("Питомцы", game.pets.len().to_string()),
                                    ("Удары питомцев", game.auto_hits.to_string()),
                                    ("Бонус питомца", format!("×{:.1}", game.pet_bonus())),
                                    ("Время в игре", format!("{} мин", (game.played_seconds / 60.0) as u64)),
                                ] {
                                    egui::Frame::none().fill(panel).stroke(Stroke::new(1.0, C::from_rgb(55, 100, 136))).rounding(10.0).inner_margin(egui::Margin::same(14.0)).show(ui, |ui| {
                                        ui.set_min_width((width - 70.0).max(180.0));
                                        ui.horizontal(|ui| {
                                            ui.label(T::new(label).color(muted));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(T::new(value).strong().color(gold)); });
                                        });
                                    });
                                    ui.add_space(7.0);
                                }
                            }
                            game::Tab::Rebirths => {
                                ui.heading("Ребёрты");
                                ui.add_space(20.0);
                                ui.label(T::new(format!("Постоянный бонус ×{}", amount(game.rebirth_bonus()))).size(23.0).color(green));
                                ui.add_space(12.0);
                                ui.label("Начни заново с удвоенным доходом.");
                                ui.label(T::new("Деньги, множитель и улучшения сбросятся.\nУровень, ранг и статистика останутся.").size(13.0).color(muted));
                                ui.add_space(24.0);
                                let can_buy = game.coins >= game.rebirth_price();
                                if ui.add_enabled(can_buy, egui::Button::new(T::new(format!("Ребёрт · {} денег", amount(game.rebirth_price()))).color(navy).strong()).fill(gold).rounding(12.0).min_size(vec2(ui.available_width().min(340.0), 58.0))).clicked() { game.rebirth(); }
                                if !can_buy { ui.label(T::new(format!("Не хватает {} денег", amount(game.rebirth_price() - game.coins))).color(muted)); }
                            }
                        }
                    });
                });
            });
        });
    pet_ui::overlays(ctx, game);
    if game.paused {
        egui::Window::new("Пауза").anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .collapsible(false).resizable(false).default_width((width - 64.0).min(310.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label("Доход и нажатия остановлены");
                    ui.add_space(18.0);
                    if ui.add_sized(vec2(ui.available_width(), 48.0), egui::Button::new("Продолжить").fill(gold)).clicked() { game.paused = false; }
                    ui.add_space(10.0);
                });
            });
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() { return; }
        let window = self.window.get_or_insert_with(|| Arc::new(event_loop.create_window(
            Window::default_attributes().with_title("Button Sim").with_inner_size(winit::dpi::LogicalSize::new(420.0, 780.0))
        ).expect("window"))).clone();
        let gpu = GpuContext::new(window.clone(), true, false).expect("WolfCraft GPU");
        let egui = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("ButtonSans".into(), egui::FontData::from_static(include_bytes!("../assets/fonts/ButtonSans-Regular.ttf")).into());
        fonts.families.get_mut(&egui::FontFamily::Proportional).expect("proportional font family").insert(0, "ButtonSans".into());
        egui.set_fonts(fonts);
        let state = egui_winit::State::new(egui.clone(), egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, None);
        let renderer = egui_wgpu::Renderer::new(&gpu.device, gpu.config.format, None, 1, false);
        self.gfx = Some(Graphics { gpu, egui, state, renderer });
        self.last_tick = Some(Instant::now());
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::CloseRequested) { save_game(&self.game, self.save_path.as_deref()); event_loop.exit(); return; }
        let Some(g) = self.gfx.as_mut() else { return };
        let response = g.state.on_window_event(&g.gpu.window, &event);
        match event {
            WindowEvent::Resized(size) => g.gpu.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_tick.replace(now).unwrap_or(now)).as_secs_f64().min(0.25);
                self.game.tick(dt);
                let before = (self.game.pets.len(), self.game.equipped_pets.clone(), self.game.auto_attack, self.game.upgrades, self.game.rebirths);
                let input = g.state.take_egui_input(&g.gpu.window);
                let output = g.egui.run(input, |ctx| ui(ctx, &mut self.game));
                let after = (self.game.pets.len(), self.game.equipped_pets.clone(), self.game.auto_attack, self.game.upgrades, self.game.rebirths);
                if before != after || self.last_save.map(|t| now.duration_since(t).as_secs() >= 5).unwrap_or(true) {
                    save_game(&self.game, self.save_path.as_deref());
                    self.last_save = Some(now);
                }
                g.state.handle_platform_output(&g.gpu.window, output.platform_output);
                let size = g.gpu.window.inner_size();
                if size.width == 0 || size.height == 0 { return; }
                let pixels_per_point = g.egui.pixels_per_point();
                let shapes = g.egui.tessellate(output.shapes, pixels_per_point);
                let descriptor = egui_wgpu::ScreenDescriptor { size_in_pixels: [size.width, size.height], pixels_per_point };
                for (id, delta) in &output.textures_delta.set {
                    g.renderer.update_texture(&g.gpu.device, &g.gpu.queue, *id, delta);
                }
                let Ok(frame) = g.gpu.surface.as_ref().expect("active surface").get_current_texture() else { g.gpu.resize(size.width, size.height); return; };
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = g.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("button-sim") });
                g.renderer.update_buffers(&g.gpu.device, &g.gpu.queue, &mut encoder, &shapes, &descriptor);
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("button-sim-ui"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view, resolve_target: None,
                            ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.25, b: 0.45, a: 1.0 }), store: wgpu::StoreOp::Store },
                        })],
                        depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
                    });
                    g.renderer.render(&mut pass.forget_lifetime(), &shapes, &descriptor);
                }
                g.gpu.queue.submit(Some(encoder.finish()));
                frame.present();
                for id in &output.textures_delta.free { g.renderer.free_texture(id); }
                g.gpu.window.request_redraw();
            }
            _ => { if response.repaint { g.gpu.window.request_redraw(); } }
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) { save_game(&self.game, self.save_path.as_deref()); self.gfx = None; self.last_tick = None; }
}

pub fn run_desktop() {
    let event_loop = EventLoop::new().expect("event loop");
    let root = std::env::var_os("XDG_DATA_HOME").map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| std::path::PathBuf::from(p).join(".local/share")))
        .unwrap_or_else(std::env::temp_dir);
    let mut app = App::with_save_path(root.join("button-sim/save.txt"));
    event_loop.run_app(&mut app).expect("application loop");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::{event_loop::EventLoopBuilder, platform::android::EventLoopBuilderExtAndroid};
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let save_path = android_app.internal_data_path().unwrap_or_else(std::env::temp_dir).join("button-sim-save.txt");
    let event_loop = EventLoopBuilder::default().with_android_app(android_app).build().expect("android event loop");
    let mut app = App::with_save_path(save_path);
    event_loop.run_app(&mut app).expect("application loop");
}
