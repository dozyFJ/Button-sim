//! Button Sim — WolfCraft Rust/wgpu-based portrait clicker.
pub mod game;

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
}


fn amount(n: f64) -> String {
    if n >= 1_000_000_000.0 { format!("{:.1}B", n / 1_000_000_000.0) }
    else if n >= 1_000_000.0 { format!("{:.1}M", n / 1_000_000.0) }
    else if n >= 10_000.0 { format!("{:.1}K", n / 1_000.0) }
    else { format!("{:.0}", n.floor()) }
}

fn ui(ctx: &egui::Context, game: &mut game::Game) {
    use egui::{Color32 as C, RichText as T, Stroke, pos2, vec2, Rect, Align2, FontId, Sense};
    let navy = C::from_rgb(17, 29, 54);
    let panel = C::from_rgb(26, 43, 73);
    let gold = C::from_rgb(255, 206, 100);
    let muted = C::from_rgb(155, 176, 205);
    let green = C::from_rgb(99, 231, 184);
    let width = ctx.screen_rect().width();
    let was_paused = game.paused;
    egui::TopBottomPanel::top("profile_header")
        .frame(egui::Frame::none().fill(navy).inner_margin(egui::Margin::same(12.0)))
        .show(ctx, |ui| {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 108.0), Sense::hover());
            let p = ui.painter();
            let avatar = pos2(r.left() + 42.0, r.top() + 24.0);
            p.circle_filled(avatar, 22.0, C::from_rgb(56, 88, 130));
            p.circle_stroke(avatar, 22.0, Stroke::new(2.0, gold));
            p.circle_filled(avatar + vec2(0.0, -5.0), 7.0, C::from_rgb(217, 232, 249));
            p.rect_filled(Rect::from_center_size(avatar + vec2(0.0, 10.0), vec2(23.0, 12.0)), 6.0, C::from_rgb(217, 232, 249));
            p.text(pos2(r.left() + 42.0, r.top() + 59.0), Align2::CENTER_CENTER, game.rank(), FontId::proportional(11.0), gold);
            let bar = Rect::from_min_size(pos2(r.left(), r.top() + 73.0), vec2(84.0, 9.0));
            p.rect_filled(bar, 4.0, C::from_rgb(43, 62, 91));
            if game.level_progress() > 0.0 {
                p.rect_filled(Rect::from_min_size(bar.min, vec2(bar.width() * game.level_progress(), bar.height())), 4.0, green);
            }
            p.text(pos2(r.left() + 42.0, r.top() + 97.0), Align2::CENTER_CENTER, format!("Ур. {} · {}/100", game.level(), game.taps % 100), FontId::proportional(10.0), muted);
            let card = Rect::from_min_max(pos2(r.left() + 98.0, r.top() + 8.0), pos2(r.right() - 53.0, r.top() + 99.0));
            p.rect_filled(card, 15.0, panel);
            p.rect_stroke(card, 15.0, Stroke::new(1.0, C::from_rgb(57, 78, 108)));
            p.text(pos2(card.center().x, card.top() + 17.0), Align2::CENTER_CENTER, "МОНЕТЫ", FontId::proportional(11.0), gold);
            p.text(card.center() - vec2(0.0, 1.0), Align2::CENTER_CENTER, amount(game.coins), FontId::proportional(24.0), C::WHITE);
            p.text(pos2(card.center().x, card.bottom() - 23.0), Align2::CENTER_CENTER, format!("+{} / сек", amount(game.coins_per_second())), FontId::proportional(12.0), green);
            // Only coins exist in the economy. The single page indicator reflects that.
            p.circle_filled(pos2(card.center().x, card.bottom() - 9.0), 2.5, gold);
            let pause = Rect::from_min_size(pos2(r.right() - 42.0, r.top() + 10.0), vec2(42.0, 42.0));
            p.rect_filled(pause, 12.0, panel);
            for x in [-5.0, 5.0] {
                p.rect_filled(Rect::from_center_size(pause.center() + vec2(x, 0.0), vec2(4.0, 16.0)), 1.0, gold);
            }
            if ui.interact(pause, ui.id().with("pause"), Sense::click()).clicked() { game.paused = !game.paused; }
        });

    egui::TopBottomPanel::bottom("navigation")
        .frame(egui::Frame::none().fill(navy).inner_margin(egui::Margin::same(8.0)))
        .show(ctx, |ui| {
            ui.add_enabled_ui(!was_paused && !game.paused, |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.columns(5, |cols| {
                    for (i, (label, tab, icon)) in [
                        ("Главная", game::Tab::Home, "H"),
                        ("Капсулы", game::Tab::Capsules, "C"),
                        ("Улучшения", game::Tab::Upgrades, "+"),
                        ("Статистика", game::Tab::Statistics, "="),
                        ("Ребёрты", game::Tab::Rebirths, "R"),
                    ].into_iter().enumerate() {
                        let col = &mut cols[i];
                        let (rect, response) = col.allocate_exact_size(vec2(col.available_width(), 68.0), Sense::click());
                        let active = game.tab == tab;
                        let p = col.painter();
                        p.rect_filled(rect, 12.0, if active { C::from_rgb(50, 71, 100) } else { panel });
                        if active { p.rect_filled(Rect::from_min_size(rect.min + vec2(12.0, 0.0), vec2((rect.width() - 24.0).max(4.0), 3.0)), 2.0, gold); }
                        p.text(rect.center() - vec2(0.0, 10.0), Align2::CENTER_CENTER, icon, FontId::proportional(24.0), if active { gold } else { muted });
                        p.text(pos2(rect.center().x, rect.bottom() - 15.0), Align2::CENTER_CENTER, label, FontId::proportional(if width < 370.0 { 9.0 } else { 11.0 }), if active { C::WHITE } else { muted });
                        if response.clicked() { game.tab = tab; }
                    }
                });
            });
        });

    egui::CentralPanel::default().frame(egui::Frame::none().fill(C::from_rgb(22, 39, 64)).inner_margin(egui::Margin::same(18.0)))
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(C::WHITE);
            ui.add_enabled_ui(!was_paused && !game.paused, |ui| {
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
                                let size = (ui.available_width() * 0.8).min(280.0);
                                let (rect, response) = ui.allocate_exact_size(vec2(size, size + 24.0), Sense::click());
                                let center = rect.center() - vec2(0.0, 10.0);
                                let pressed = response.is_pointer_button_down_on();
                                let top = center + vec2(0.0, if pressed { 10.0 } else { 0.0 });
                                let p = ui.painter();
                                p.circle_filled(center + vec2(0.0, 22.0), size * 0.49, C::from_rgb(10, 21, 37));
                                p.circle_filled(center + vec2(0.0, 9.0), size * 0.48, C::from_rgb(47, 72, 98));
                                p.circle_stroke(center + vec2(0.0, 9.0), size * 0.48, Stroke::new(3.0, gold));
                                p.circle_filled(center + vec2(0.0, 10.0), size * 0.41, C::from_rgb(129, 33, 61));
                                p.circle_filled(top, size * 0.41, C::from_rgb(245, 80, 104));
                                p.circle_filled(top - vec2(size * 0.1, size * 0.15), size * 0.16, C::from_rgba_unmultiplied(255, 185, 191, 90));
                                p.text(top, Align2::CENTER_CENTER, "ЖМИ!", FontId::proportional(36.0), C::WHITE);
                                if response.clicked() { game.tap(); }
                                ui.add_space(22.0);
                                ui.label(T::new(format!("Бонус ребёртов ×{}", amount(game.rebirth_bonus()))).color(green));
                            }
                            game::Tab::Capsules => {
                                ui.heading("Капсулы");
                                ui.label(T::new("Коллекция питомцев").color(muted));
                                ui.add_space(30.0);
                                let (r, _) = ui.allocate_exact_size(vec2(150.0, 190.0), Sense::hover());
                                let p = ui.painter();
                                p.rect_filled(r.shrink(12.0), 60.0, C::from_rgb(72, 110, 164));
                                p.rect_filled(Rect::from_min_max(pos2(r.left()+12.0, r.center().y), r.right_bottom()-vec2(12.0,12.0)), 35.0, gold);
                                p.circle_filled(r.center(), 24.0, navy);
                                p.text(r.center(), Align2::CENTER_CENTER, "?", FontId::proportional(29.0), gold);
                                ui.add_space(24.0);
                                ui.label("Капсулы ещё не доступны");
                                ui.label(T::new("Открытие и питомцы появятся позже").size(12.0).color(muted));
                            }
                            game::Tab::Upgrades => {
                                ui.heading("Улучшения");
                                ui.add_space(20.0);
                                ui.label(format!("Уровень дохода: {}", game.upgrades));
                                ui.label(format!("Базовый доход: {} / сек", amount(game.base_income)));
                                ui.add_space(18.0);
                                let can_buy = game.coins >= game.upgrade_price();
                                if ui.add_enabled(can_buy, egui::Button::new(T::new(format!("Улучшить · {} монет", amount(game.upgrade_price()))).color(navy).strong()).fill(gold).rounding(12.0).min_size(vec2(ui.available_width().min(340.0), 58.0))).clicked() { game.upgrade(); }
                                if !can_buy { ui.label(T::new("Накопи монеты для улучшения").color(muted)); }
                            }
                            game::Tab::Statistics => {
                                ui.heading("Статистика");
                                ui.add_space(22.0);
                                for (label, value) in [
                                    ("Нажатия", game.taps.to_string()),
                                    ("Заработано монет", amount(game.total_earned)),
                                    ("Доход в секунду", amount(game.coins_per_second())),
                                    ("Уровень", game.level().to_string()),
                                    ("Ранг", game.rank().to_owned()),
                                    ("Ребёрты", game.rebirths.to_string()),
                                    ("Время в игре", format!("{} мин", (game.played_seconds / 60.0) as u64)),
                                ] {
                                    egui::Frame::none().fill(panel).rounding(10.0).inner_margin(egui::Margin::same(14.0)).show(ui, |ui| {
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
                                ui.label(T::new("Монеты, множитель и улучшения сбросятся.\nУровень, ранг и статистика останутся.").size(13.0).color(muted));
                                ui.add_space(24.0);
                                let can_buy = game.coins >= game.rebirth_price();
                                if ui.add_enabled(can_buy, egui::Button::new(T::new(format!("Ребёрт · {} монет", amount(game.rebirth_price()))).color(navy).strong()).fill(gold).rounding(12.0).min_size(vec2(ui.available_width().min(340.0), 58.0))).clicked() { game.rebirth(); }
                                if !can_buy { ui.label(T::new(format!("Не хватает {} монет", amount(game.rebirth_price() - game.coins))).color(muted)); }
                            }
                        }
                    });
                });
            });
        });
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
        let state = egui_winit::State::new(egui.clone(), egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, None);
        let renderer = egui_wgpu::Renderer::new(&gpu.device, gpu.config.format, None, 1, false);
        self.gfx = Some(Graphics { gpu, egui, state, renderer });
        self.last_tick = Some(Instant::now());
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::CloseRequested) { event_loop.exit(); return; }
        let Some(g) = self.gfx.as_mut() else { return };
        let response = g.state.on_window_event(&g.gpu.window, &event);
        match event {
            WindowEvent::Resized(size) => g.gpu.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_tick.replace(now).unwrap_or(now)).as_secs_f64().min(0.25);
                self.game.tick(dt);
                let input = g.state.take_egui_input(&g.gpu.window);
                let output = g.egui.run(input, |ctx| ui(ctx, &mut self.game));
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

    fn suspended(&mut self, _: &ActiveEventLoop) { self.gfx = None; }
}

pub fn run_desktop() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("application loop");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::{event_loop::EventLoopBuilder, platform::android::EventLoopBuilderExtAndroid};
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let event_loop = EventLoopBuilder::default().with_android_app(android_app).build().expect("android event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("application loop");
}
