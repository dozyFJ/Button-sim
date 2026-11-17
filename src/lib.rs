//! Button Sim — WolfCraft Rust/wgpu-based landscape clicker.
pub mod game;
mod pet_render;
mod pet_ui;
mod updates;
mod board_ui;
mod isometric;
mod save_format;

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
    update_error: Option<String>,
    boot_healthy: bool,
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
        5 => {
            p.circle_filled(center+vec2(0.0,5.0),6.0,color);
            for v in [vec2(-9.0,-3.0),vec2(0.0,-8.0),vec2(9.0,-3.0)] {p.circle_filled(center+v,3.5,color);}
        }
        _ => {
            p.circle_stroke(center, 10.0, s);
            p.line_segment([center + vec2(10.0, -10.0), center + vec2(10.0, 0.0)], s);
            p.line_segment([center + vec2(10.0, 0.0), center + vec2(0.0, 0.0)], s);
        }
    }
}

fn ui(ctx: &egui::Context, game: &mut game::Game, update_error: Option<&str>) -> bool {
    let mut update_requested = false;
    use egui::{Color32 as C, RichText as T, Stroke, pos2, vec2, Rect, Align2, FontId, Sense};
    let navy = C::from_rgb(39, 54, 41);
    let panel = C::from_rgb(250, 243, 222);
    let gold = C::from_rgb(232, 175, 63);
    let muted = C::from_rgb(86, 102, 78);
    let green = C::from_rgb(93, 161, 71);
    let width = ctx.screen_rect().width();
    let was_paused = game.paused;
    let blocked = was_paused || game.hatch.is_some() || game.pets_open;
    egui::TopBottomPanel::top("game_header").exact_height(110.0)
        .frame(egui::Frame::none().fill(C::from_rgb(246, 238, 211)).inner_margin(egui::Margin {left:10.0,right:10.0,top:6.0,bottom:6.0}))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::hover());
            let p = ui.painter();
            let profile_width = if width < 700.0 {92.0} else {118.0};
            let avatar = Rect::from_min_size(r.min + vec2(0.0,4.0),vec2(34.0,34.0));
            p.rect_filled(avatar,0.0,C::from_rgb(115,174,196));
            p.rect_stroke(avatar,0.0,Stroke::new(2.0,navy));
            p.rect_filled(Rect::from_min_size(avatar.min+vec2(11.0,6.0),vec2(12.0,12.0)),0.0,C::from_rgb(255,226,175));
            p.rect_filled(Rect::from_min_size(avatar.min+vec2(7.0,20.0),vec2(20.0,11.0)),0.0,C::from_rgb(66,112,153));
            p.text(pos2(r.left()+42.0,r.top()+14.0),Align2::LEFT_CENTER,"ИГРОК",FontId::proportional(11.0),navy);
            p.text(pos2(r.left()+42.0,r.top()+30.0),Align2::LEFT_CENTER,game.rank(),FontId::proportional(if width<700.0 {8.0} else {10.0}),muted);
            let remaining = (r.width()-profile_width-64.0).max(240.0);
            let money_width = remaining*0.42;
            let rebirth_width = remaining*0.24;
            let attack_width = remaining*0.34;
            let card = Rect::from_min_size(pos2(r.left()+profile_width,r.top()),vec2(money_width,44.0));
            p.rect_filled(card,0.0,C::from_rgb(229,235,197));
            p.rect_stroke(card,0.0,Stroke::new(1.0,C::from_rgb(158,176,122)));
            let icon=card.left_center()+vec2(20.0,0.0);
            p.rect_filled(Rect::from_center_size(icon+vec2(-2.0,3.0),vec2(25.0,16.0)),0.0,C::from_rgb(83,131,57));
            p.rect_filled(Rect::from_center_size(icon,vec2(25.0,16.0)),0.0,C::from_rgb(137,189,86));
            p.circle_filled(icon,3.5,C::from_rgb(244,212,99));
            let text_x=card.left()+40.0;
            p.text(pos2(text_x,card.top()+8.0),Align2::LEFT_CENTER,"ДЕНЬГИ",FontId::proportional(9.0),muted);
            p.text(pos2(text_x,card.top()+23.0),Align2::LEFT_CENTER,amount(game.coins),FontId::proportional(18.0),navy);
            p.text(pos2(text_x,card.bottom()-7.0),Align2::LEFT_CENTER,format!("+{} / сек",amount(game.coins_per_second())),FontId::proportional(9.0),muted);
            let rebirth=Rect::from_min_size(card.right_top()+vec2(6.0,0.0),vec2(rebirth_width,44.0));
            p.rect_filled(rebirth,0.0,C::from_rgb(246,221,174));
            p.rect_stroke(rebirth,0.0,Stroke::new(1.0,C::from_rgb(196,159,98)));
            let ri=rebirth.left_center()+vec2(18.0,0.0);
            p.circle_stroke(ri,8.0,Stroke::new(2.0,C::from_rgb(155,102,40)));
            p.line_segment([ri+vec2(8.0,-8.0),ri+vec2(8.0,0.0)],Stroke::new(2.0,C::from_rgb(155,102,40)));
            p.line_segment([ri+vec2(8.0,0.0),ri],Stroke::new(2.0,C::from_rgb(155,102,40)));
            p.text(rebirth.left_top()+vec2(35.0,12.0),Align2::LEFT_CENTER,"РЕБЁРТЫ",FontId::proportional(9.0),muted);
            p.text(rebirth.left_top()+vec2(35.0,29.0),Align2::LEFT_CENTER,game.rebirths.to_string(),FontId::proportional(17.0),navy);
            let attack=Rect::from_min_size(rebirth.right_top()+vec2(6.0,0.0),vec2(attack_width,44.0));
            p.rect_filled(attack,0.0,if game.auto_attack {green} else {C::from_rgb(215,225,206)});
            p.rect_stroke(attack,0.0,Stroke::new(1.0,C::from_rgb(112,139,92)));
            let fg=if game.auto_attack {C::WHITE} else {navy};
            p.text(attack.center()-vec2(0.0,8.0),Align2::CENTER_CENTER,"АВТОАТАКА",FontId::proportional(11.0),fg);
            let status=if game.equipped_pets.is_empty() {"Выбрать питомцев".into()} else {format!("{} · {}/3",if game.auto_attack {"ВКЛ"} else {"ВЫКЛ"},game.equipped_pets.len())};
            p.text(attack.center()+vec2(0.0,9.0),Align2::CENTER_CENTER,status,FontId::proportional(10.0),fg);
            let pause=Rect::from_min_size(pos2(r.right()-40.0,r.top()),vec2(40.0,44.0));
            p.rect_filled(pause,0.0,C::from_rgb(203,215,224));
            for x in [-5.0,5.0] {p.rect_filled(Rect::from_center_size(pause.center()+vec2(x,0.0),vec2(4.0,17.0)),0.0,navy);}
            if ui.interact(pause,ui.id().with("pause"),Sense::click()).clicked() {game.paused=!game.paused;}
            if !blocked&&!game.paused&&ui.interact(attack,ui.id().with("auto_attack"),Sense::click()).clicked() {
                if game.equipped_pets.is_empty() {game.pets_open=true;} else {game.toggle_auto_attack();}
            }
            let (xp,_) = ui.allocate_exact_size(vec2(ui.available_width(),14.0),Sense::hover());
            let track=Rect::from_min_max(xp.min+vec2(0.0,5.0),xp.max-vec2(0.0,3.0));
            ui.painter().rect_filled(track,0.0,C::from_rgb(196,205,172));
            if game.level_progress()>0.0 {ui.painter().rect_filled(Rect::from_min_size(track.min,vec2(track.width()*game.level_progress(),track.height())),0.0,green);}
            ui.painter().text(xp.center(),Align2::CENTER_CENTER,format!("УРОВЕНЬ {} · {}/100 XP",game.level(),game.taps%100),FontId::proportional(9.0),navy);
            let (menu_row,_) = ui.allocate_exact_size(vec2(ui.available_width(),30.0),Sense::hover());
            let strip_width=menu_row.width().min(760.0);
            let start=menu_row.center().x-strip_width*0.5;
            let cell_width=(strip_width-5.0*4.0)/6.0;
            let menu_enabled=!game.paused&&game.hatch.is_none();
            for (i,label) in ["Главная","Капсулы","Улучшения","Статистика","Ребёрты","Питомцы"].into_iter().enumerate() {
                let tab=[game::Tab::Home,game::Tab::Capsules,game::Tab::Upgrades,game::Tab::Statistics,game::Tab::Rebirths].get(i).copied();
                let active=if i==5 {game.pets_open} else {!game.pets_open&&tab==Some(game.tab)};
                let rect=Rect::from_min_size(pos2(start+i as f32*(cell_width+4.0),menu_row.top()),vec2(cell_width,30.0));
                let fg=if active {C::WHITE} else {navy};
                ui.painter().rect_filled(rect,0.0,if active {green} else {C::from_rgb(225,224,202)});
                nav_icon(ui.painter(),rect.left_center()+vec2(15.0,0.0),i,fg);
                ui.painter().text(rect.center()+vec2(12.0,0.0),Align2::CENTER_CENTER,label,FontId::proportional(if width<800.0 {10.0} else {11.0}),fg);
                if menu_enabled&&ui.interact(rect,ui.id().with(("header_menu",i)),Sense::click()).clicked() {
                    if i==5 {game.pets_open=!game.pets_open;} else if let Some(tab)=tab {game.tab=tab;game.pets_open=false;}
                }
            }
        });

    egui::CentralPanel::default().frame(egui::Frame::none().fill(C::from_rgb(166, 207, 224)).inner_margin(egui::Margin::same(if game.tab == game::Tab::Home {0.0} else {10.0})))
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(navy);
            ui.add_enabled_ui(!blocked && !game.paused, |ui| {
                if game.tab == game::Tab::Home { board_ui::show(ui, game); return; }
                egui::ScrollArea::vertical().id_salt("scene_scroll").show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(18.0);
                        match game.tab {
                            game::Tab::Home => { board_ui::show(ui, game); }
                            game::Tab::Capsules => { pet_ui::shop(ui, game); }
                            game::Tab::Upgrades => {
                                ui.heading("Улучшения");
                                ui.add_space(20.0);
                                ui.label(format!("Уровень дохода: {}", game.upgrades));
                                ui.label(format!("Базовый доход: {} / сек", amount(game.base_income)));
                                ui.add_space(18.0);
                                let can_buy = game.coins >= game.upgrade_price();
                                if ui.add_enabled(can_buy, egui::Button::new(T::new(format!("Улучшить · {} денег", amount(game.upgrade_price()))).color(navy).strong()).fill(gold).rounding(0.0).min_size(vec2(ui.available_width().min(340.0), 58.0))).clicked() { game.upgrade(); }
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
                                    egui::Frame::none().fill(panel).stroke(Stroke::new(1.0, C::from_rgb(155, 170, 133))).rounding(0.0).inner_margin(egui::Margin::same(14.0)).show(ui, |ui| {
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
                                if ui.add_enabled(can_buy, egui::Button::new(T::new(format!("Ребёрт · {} денег", amount(game.rebirth_price()))).color(navy).strong()).fill(gold).rounding(0.0).min_size(vec2(ui.available_width().min(340.0), 58.0))).clicked() { game.rebirth(); }
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
                    let build = option_env!("GITHUB_SHA").unwrap_or("development");
                    ui.label(format!("Button Sim {} · {}", env!("CARGO_PKG_VERSION"), &build[..build.len().min(8)]));
                    if ui.add_sized(vec2(ui.available_width(), 44.0), egui::Button::new("Обновить игру (OTA)")).clicked() { update_requested = true; }
                    if let Some(error) = update_error { ui.colored_label(C::LIGHT_RED, error); }
                    ui.add_space(18.0);
                    if ui.add_sized(vec2(ui.available_width(), 48.0), egui::Button::new("Продолжить").fill(gold)).clicked() { game.paused = false; }
                    ui.add_space(10.0);
                });
            });
    }
    update_requested
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() { return; }
        let window = self.window.get_or_insert_with(|| Arc::new(event_loop.create_window(
            Window::default_attributes().with_title("Button Sim").with_inner_size(winit::dpi::LogicalSize::new(960.0, 540.0))
        ).expect("window"))).clone();
        let gpu = GpuContext::new(window.clone(), true, false).expect("WolfCraft GPU");
        let egui = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("ButtonSans".into(), egui::FontData::from_static(include_bytes!("../assets/fonts/ButtonSans-Regular.ttf")).into());
        fonts.families.get_mut(&egui::FontFamily::Proportional).expect("proportional font family").insert(0, "ButtonSans".into());
        egui.set_fonts(fonts);
        let mut visuals = egui::Visuals::light();
        visuals.panel_fill = egui::Color32::from_rgb(246,238,211);
        visuals.window_fill = egui::Color32::from_rgb(250,244,222);
        visuals.selection.bg_fill = egui::Color32::from_rgb(93,161,71);
        visuals.override_text_color = Some(egui::Color32::from_rgb(39,54,41));
        egui.set_visuals(visuals);
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
                let before = (self.game.pets.len(), self.game.equipped_pets.clone(), self.game.auto_attack, self.game.upgrades, self.game.rebirths, self.game.grid_size, self.game.selected_switch);
                let input = g.state.take_egui_input(&g.gpu.window);
                let mut update_requested = false;
                let update_error = self.update_error.clone();
                let output = g.egui.run(input, |ctx| { update_requested = ui(ctx, &mut self.game, update_error.as_deref()); });
                if update_requested {
                    save_game(&self.game, self.save_path.as_deref());
                    self.update_error = updates::open().err();
                }
                let after = (self.game.pets.len(), self.game.equipped_pets.clone(), self.game.auto_attack, self.game.upgrades, self.game.rebirths, self.game.grid_size, self.game.selected_switch);
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
                if !self.boot_healthy { updates::mark_ready(); self.boot_healthy = true; }
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
    updates::configure(android_app.clone());
    let save_path = android_app.internal_data_path().unwrap_or_else(std::env::temp_dir).join("button-sim-save.txt");
    let event_loop = EventLoopBuilder::default().with_android_app(android_app).build().expect("android event loop");
    let mut app = App::with_save_path(save_path);
    event_loop.run_app(&mut app).expect("application loop");
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn landscape_scene_fills_space_below_single_header() {
        for (width,height) in [(640.0,360.0),(800.0,360.0),(960.0,540.0),(1280.0,720.0)] {
            for tab in [game::Tab::Home,game::Tab::Capsules,game::Tab::Statistics] {
                let ctx=egui::Context::default();
                let mut game=game::Game::default();game.tab=tab;
                let output=ctx.run(egui::RawInput {screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(width,height))),..Default::default()},|ctx| {ui(ctx,&mut game,None);});
                let scene=output.shapes.iter().find_map(|shape| match &shape.shape {
                    egui::Shape::Rect(rect) if rect.fill==egui::Color32::from_rgb(166,207,224)=>Some(rect.rect),_=>None
                }).expect("scene background");
                assert!(scene.top()>=100.0&&scene.top()<=116.0,"header height: {scene:?}");
                assert!((scene.bottom()-height).abs()<1.0,"unexpected bottom dock: {scene:?}");
                assert!(scene.height()>=height*0.65,"scene too small: {scene:?}");
                let menu:Vec<_>=output.shapes.iter().filter_map(|shape|match &shape.shape {
                    egui::Shape::Rect(rect) if (rect.rect.height()-30.0).abs()<0.1&&rect.rect.top()>60.0&&rect.rect.bottom()<110.0=>Some(rect.rect),_=>None
                }).collect();
                assert_eq!(menu.len(),6,"expected six items in the header");
                let left=menu.iter().map(|r|r.left()).fold(f32::INFINITY,f32::min);
                let right=menu.iter().map(|r|r.right()).fold(f32::NEG_INFINITY,f32::max);
                assert!(((left+right)*0.5-width*0.5).abs()<1.0,"menu is not centered");
                let pets:Vec<_>=output.shapes.iter().filter_map(|shape|match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text=="Питомцы"&&text.pos.y<110.0=>Some(text.pos),_=>None
                }).collect();
                assert_eq!(pets.len(),1,"pets should have one entry point in the header");
                assert!(pets[0].y<110.0,"pets button is not in the header");
                if tab==game::Tab::Home {
                    let field=output.shapes.iter().find_map(|shape|match &shape.shape {
                        egui::Shape::Rect(rect) if rect.fill==egui::Color32::from_rgb(121,166,82)=>Some(rect.rect),_=>None
                    }).expect("ground field viewport");
                    assert!(field.left()<=1.0&&field.right()>=width-1.0,"field does not fill screen width: {field:?}");
                }
            }
        }
    }
}
