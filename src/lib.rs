//! Button Sim — WolfCraft Rust/wgpu-based landscape clicker.
pub mod game;
mod pet_render;
mod pet_ui;
mod updates;
mod board_ui;
mod gui_state;
mod sidebar_ui;
mod panels_ui;
mod inventory_ui;
mod surface_gui;
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

fn theme_color(dark: bool, light: [u8; 3], night: [u8; 3]) -> egui::Color32 {
    let [r, g, b] = if dark { night } else { light };
    egui::Color32::from_rgb(r, g, b)
}

fn apply_theme(ctx: &egui::Context, dark: bool) {
    let mut visuals = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    visuals.panel_fill = theme_color(dark, [246,238,211], [24,23,42]);
    visuals.window_fill = theme_color(dark, [250,244,222], [32,29,53]);
    visuals.selection.bg_fill = theme_color(dark, [93,161,71], [83,56,166]);
    visuals.override_text_color = Some(theme_color(dark, [39,54,41], [241,237,255]));
    ctx.set_visuals(visuals);
}

fn board_swipe(
    ctx: &egui::Context, field: egui::Rect, controls: egui::Rect,
    start: &mut Option<(egui::TouchId, egui::Pos2)>,
) -> Option<game::Tab> {
    ctx.input(|input| {
        for event in &input.events {
            let egui::Event::Touch { id, phase, pos, .. } = event else { continue; };
            match phase {
                egui::TouchPhase::Start if field.contains(*pos) && !controls.contains(*pos) => {
                    *start = Some((*id, *pos));
                }
                egui::TouchPhase::End | egui::TouchPhase::Cancel if start.is_some_and(|(touch, _)| touch == *id) => {
                    let (_, origin) = start.take().unwrap();
                    let delta = *pos - origin;
                    if *phase == egui::TouchPhase::End && delta.x.abs() >= 55.0 && delta.x.abs() >= 1.65 * delta.y.abs() {
                        return Some(if delta.x > 0.0 { game::Tab::Rebirths } else { game::Tab::Home });
                    }
                }
                _ => {}
            }
        }
        None
    })
}

fn ui(ctx:&egui::Context,game:&mut game::Game,update_error:Option<&str>)->bool {
    use egui::{Color32 as C,RichText as T,Rect,vec2};
    use gui_state::{GuiState,Panel,Modal};
    if ctx.style().visuals.dark_mode != game.dark_mode || ctx.style().visuals.override_text_color.is_none() {
        apply_theme(ctx, game.dark_mode);
    }
    let mut state=GuiState::load(ctx);
    match game.tab {
        game::Tab::Statistics=>{state.profile(true);game.tab=game::Tab::Home;}
        game::Tab::Upgrades=>{state.modal(Modal::Upgrades);game.tab=game::Tab::Home;}
        _=>{}
    }
    if game.hatch.is_some() {
        game.interface_open=true;
        pet_ui::overlays(ctx,game);
        return false;
    }
    sidebar_ui::draw(ctx,game,&mut state);
    game.pets_open=state.panel==Panel::Pets;game.interface_open=state.busy();
    let workspace=ctx.available_rect();
    let blocked=game.paused||game.hatch.is_some()||game.interface_open;
    egui::CentralPanel::default().frame(egui::Frame::none().fill(theme_color(game.dark_mode, [166,207,224], [14,15,29])).inner_margin(egui::Margin::same(if game.tab==game::Tab::Capsules {10.0} else {0.0}))).show(ctx,|ui| {
        ui.add_enabled_ui(!blocked,|ui| {
            if game.tab==game::Tab::Capsules {
                pet_ui::shop(ui,game);
            } else {board_ui::show(ui,game);}
        });
    });
    panels_ui::draw(ctx,workspace,game,&mut state);
    inventory_ui::draw(ctx,workspace,game,&mut state);
    if matches!(game.tab, game::Tab::Home | game::Tab::Rebirths) && !blocked {
        let previous = game.tab;
        let controls = pet_ui::board_controls(ctx, workspace, game, &mut state);
        if !state.busy() && game.tab == previous {
            if let Some(tab) = board_swipe(ctx, workspace, controls, &mut state.board_swipe) {
                game.tab = tab;
            }
        } else {
            state.board_swipe = None;
        }
    } else {
        state.board_swipe = None;
    }
    let mut update_requested=false;
    if game.paused {
        let rect=Rect::from_center_size(workspace.center(),vec2((workspace.width()-24.0).min(350.0),(workspace.height()-24.0).min(260.0)));
        egui::Window::new("Пауза").fixed_rect(rect).collapsible(false).resizable(false).show(ctx,|ui| {
            ui.vertical_centered(|ui| {
                ui.label("Доход и нажатия остановлены");
                let build=option_env!("GITHUB_SHA").unwrap_or("development");ui.label(format!("Button Sim {} · {}",env!("CARGO_PKG_VERSION"),&build[..build.len().min(8)]));
                if ui.add_sized(vec2(ui.available_width(),44.0),egui::Button::new("Обновить игру (OTA)")).clicked() {update_requested=true;}
                if let Some(error)=update_error {ui.colored_label(C::LIGHT_RED,error);}
                if ui.add_sized(vec2(ui.available_width(),44.0),egui::Button::new(T::new("Продолжить").strong())).clicked() {game.paused=false;}
            });
        });
    }
    game.pets_open=state.panel==Panel::Pets;game.interface_open=state.busy();state.store(ctx);
    update_requested
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() { return; }
        let window = self.window.get_or_insert_with(|| Arc::new(event_loop.create_window(
            Window::default_attributes().with_title("Button Sim").with_inner_size(winit::dpi::LogicalSize::new(420.0, 860.0))
        ).expect("window"))).clone();
        let gpu = GpuContext::new(window.clone(), true, false).expect("WolfCraft GPU");
        let egui = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("ButtonSans".into(), egui::FontData::from_static(include_bytes!("../assets/fonts/ButtonSans-Regular.ttf")).into());
        fonts.families.get_mut(&egui::FontFamily::Proportional).expect("proportional font family").insert(0, "ButtonSans".into());
        egui.set_fonts(fonts);
        apply_theme(&egui, self.game.dark_mode);
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
                let before = (self.game.pets.len(), self.game.equipped_pets.clone(), self.game.auto_attack, self.game.upgrades, self.game.rebirths, self.game.grid_size, self.game.selected_switch, self.game.dark_mode);
                let input = g.state.take_egui_input(&g.gpu.window);
                let mut update_requested = false;
                let update_error = self.update_error.clone();
                let output = g.egui.run(input, |ctx| { update_requested = ui(ctx, &mut self.game, update_error.as_deref()); });
                if update_requested {
                    save_game(&self.game, self.save_path.as_deref());
                    self.update_error = updates::open().err();
                }
                let after = (self.game.pets.len(), self.game.equipped_pets.clone(), self.game.auto_attack, self.game.upgrades, self.game.rebirths, self.game.grid_size, self.game.selected_switch, self.game.dark_mode);
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
    fn render(ctx:&egui::Context,game:&mut game::Game,width:f32,height:f32)->egui::FullOutput {
        ctx.run(egui::RawInput{screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(width,height))),..Default::default()},|ctx|{ui(ctx,game,None);})
    }
    #[test] fn portrait_header_dock_leave_a_full_width_playfield() {
        for (width,height) in [(360.0,740.0),(390.0,844.0),(430.0,932.0),(540.0,960.0)] {
            let ctx=egui::Context::default();let mut game=game::Game::default();render(&ctx,&mut game,width,height);let out=render(&ctx,&mut game,width,height);
            let scene=out.shapes.iter().find_map(|s|match &s.shape {egui::Shape::Rect(r) if r.fill==egui::Color32::from_rgb(166,207,224)=>Some(r.rect),_=>None}).expect("workspace");
            assert!(scene.left()<1.0&&scene.right()>=width-1.0,"full width {scene:?}");
            assert!((scene.top()-sidebar_ui::HEADER).abs()<2.0,"header {scene:?}");
            assert!((scene.bottom()-(height-sidebar_ui::DOCK)).abs()<2.0,"dock {scene:?}");
            assert!(out.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Автоатака")));
            let label = |prefix: &str| out.shapes.iter().find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text.starts_with(prefix) => {
                    Some(egui::Rect::from_min_size(text.pos, text.galley.size()))
                }
                _ => None,
            }).unwrap();
            let tile = label("Главная");
            let xp = label("Уровень");
            assert!(tile.bottom() < xp.top() && xp.bottom() < height - 14.0);
            let progress = label("0 / 100 XP");
            assert!(xp.right() < progress.left() && (xp.center().y - progress.center().y).abs() < 2.0);
            assert!((xp.left() + progress.right() - width).abs() < 2.0);
        }
    }
    #[test]
    fn balance_is_visible_next_to_profile_at_phone_widths() {
        for dark in [false, true] {
            for width in [280.0, 360.0, 540.0] {
                let ctx = egui::Context::default();
                let mut game = game::Game::default();
                game.coins = 11_400_000.0;
                game.dark_mode = dark;
                render(&ctx, &mut game, width, 740.0);
                let output = render(&ctx, &mut game, width, 740.0);
                let (balance, clip) = output.shapes.iter().find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text == format!("$ {}", amount(game.coins)) =>
                        Some((egui::Rect::from_min_size(text.pos, text.galley.size()), shape.clip_rect)),
                    _ => None,
                }).expect("balance in header");
                assert!(balance.left() >= 46.0 && balance.left() < 75.0, "{balance:?}");
                assert!(balance.bottom() < sidebar_ui::HEADER && clip.contains_rect(balance), "{balance:?}, {clip:?}");
            }
        }
    }
    #[test]
    fn second_board_shows_rebirths_and_current_page() {
        let ctx = egui::Context::default();
        let mut game = game::Game::default();
        game.tab = game::Tab::Rebirths;
        render(&ctx, &mut game, 360.0, 740.0);
        let out = render(&ctx, &mut game, 360.0, 740.0);
        let labels: Vec<_> = out.shapes.iter().filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        }).collect();
        assert!(labels.contains(&"Плитки ребёрта"));
        assert!(labels.contains(&"Страница 2/2"));
        assert!(!labels.contains(&"Автоатака"));
    }
    #[test] fn profile_and_inventory_are_context_frames_to_the_right_of_navigation() {
        let ctx=egui::Context::default();let mut g=game::Game::default();let mut state=gui_state::GuiState::default();state.profile(true);state.store(&ctx);
        render(&ctx,&mut g,960.0,720.0);assert!(g.interface_open);assert!(!g.pets_open);
        state.pets();state.store(&ctx);g.pets=vec![game::PetKind::Cat,game::PetKind::Cat,game::PetKind::Cat];
        render(&ctx,&mut g,960.0,720.0);let out=render(&ctx,&mut g,960.0,720.0);assert!(g.pets_open);assert!(g.interface_open);
        assert!(out.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Удалить")));
    }
    #[test] fn hatch_hides_game_gui_and_centers_the_scene() {
        for elapsed in [1.5,3.1] {
            let ctx=egui::Context::default();let mut g=game::Game::default();g.coins=500.0;g.buy_capsule(game::Capsule::Meadow);g.hatch.as_mut().unwrap().elapsed=elapsed;
            render(&ctx,&mut g,960.0,540.0);let out=render(&ctx,&mut g,960.0,540.0);
            let texts:Vec<_>=out.shapes.iter().filter_map(|s|match &s.shape {egui::Shape::Text(t)=>Some(t.galley.job.text.as_str()),_=>None}).collect();
            assert!(!texts.iter().any(|t|["Главная","МОЙ ПРОФИЛЬ","Автоатака"].contains(t)));
            assert!(texts.contains(&if elapsed<3.0 {"ОТКРЫТИЕ КАПСУЛЫ"} else {"НОВЫЙ ПИТОМЕЦ!"}));
            let background=out.shapes.iter().find_map(|s|match &s.shape {egui::Shape::Rect(r) if r.fill==egui::Color32::from_rgb(166,207,224)=>Some(r.rect),_=>None}).unwrap();
            assert!(background.left()<1.0&&background.width()>959.0);
        }
    }

}
