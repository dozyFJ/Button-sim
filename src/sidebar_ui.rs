//! Portrait HUD with a compact currency header and centered navigation tiles.
use egui::{Align2, Color32 as C, FontId, RichText as T, Sense, vec2};
use crate::{amount, game::{Game, Tab}, gui_state::{GuiState, Modal, Panel}, nav_icon};

const GREEN: C = C::from_rgb(93, 161, 71);
pub const HEADER: f32 = 92.0;
pub const DOCK: f32 = 74.0;
const TILE: f32 = 54.0;
const GAP: f32 = 5.0;
const NAV_ITEMS: usize = 7;
const ARROW: f32 = 24.0;

fn tile(ui: &mut egui::Ui, label: &str, icon: usize, active: bool) -> bool {
    let dark = ui.visuals().dark_mode;
    let color = if active && !dark { C::BLACK } else { ui.visuals().text_color() };
    let (rect, response) = ui.allocate_exact_size(vec2(TILE, TILE), Sense::click());
    let painter = ui.painter();
    let fill = if active {
        crate::theme_color(dark, [93,161,71], [45,104,72])
    } else if response.hovered() {
        crate::theme_color(dark, [216,229,192], [52,67,59])
    } else {
        crate::theme_color(dark, [230,233,207], [37,49,43])
    };
    painter.rect_filled(rect, 9.0, fill);
    let icon_center = rect.center() - vec2(0.0, 8.0);
    if icon < 6 {
        nav_icon(painter, icon_center, icon, color);
    } else {
        painter.text(icon_center, Align2::CENTER_CENTER, if icon == 6 { "⚙" } else { "$" }, FontId::proportional(21.0), color);
    }
    painter.text(rect.center_bottom() - vec2(0.0, 9.0), Align2::CENTER_CENTER, label, FontId::proportional(10.0), color);
    response.clicked()
}

pub fn draw(ctx: &egui::Context, game: &mut Game, state: &mut GuiState) {
    let enabled = !game.paused && game.hatch.is_none();
    egui::TopBottomPanel::top("portrait_header").exact_height(HEADER).resizable(false)
        .frame(egui::Frame::none().fill(ctx.style().visuals.panel_fill).inner_margin(egui::Margin::same(8.0))).show(ctx, |ui| {
        ui.horizontal(|ui| {
            let profile = ui.add_enabled(enabled, egui::Button::new("").min_size(vec2(38.0, 34.0)))
                .on_hover_text("Профиль");
            let center = profile.rect.center();
            let stroke = egui::Stroke::new(1.6, ui.style().interact(&profile).fg_stroke.color);
            ui.painter().circle_stroke(center - vec2(0.0, 5.0), 3.5, stroke);
            ui.painter().add(egui::Shape::line(
                [vec2(-7.0, 7.0), vec2(-6.0, 3.0), vec2(-3.0, 1.0), vec2(3.0, 1.0), vec2(6.0, 3.0), vec2(7.0, 7.0)]
                    .into_iter().map(|offset| center + offset).collect(),
                stroke,
            ));
            if profile.clicked() {
                if state.panel == Panel::Profile { state.close(); } else { state.profile(false); }
            }
            let currency_width = (ui.available_width() - 46.0).max(0.0);
            egui::ScrollArea::horizontal().id_salt("header_currencies").max_width(currency_width)
                .auto_shrink([false, true]).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.set_min_height(34.0);
                        let values = [
                            (format!("$ {}", amount(game.coins)), 19.0),
                            (format!("↻ {}", game.rebirths), 17.0),
                            (format!("+{} / сек", amount(game.coins_per_second())), 13.0),
                        ];
                        let spacing = 10.0;
                        ui.spacing_mut().item_spacing.x = spacing;
                        let text_width = values.iter().map(|(text, size)| {
                            ui.painter().layout_no_wrap(text.clone(), FontId::proportional(*size), ui.visuals().text_color()).size().x
                        }).sum::<f32>() + 2.0 * spacing;
                        let padding = (currency_width - text_width - spacing - 6.0).max(0.0) * 0.5;
                        if padding > 0.0 { ui.add_space(padding); }
                        for (text, size) in values {
                            ui.label(T::new(text).size(size));
                        }
                    });
                });
            let pause = ui.add(egui::Button::new("").min_size(vec2(38.0, 34.0)))
                .on_hover_text(if game.paused { "Продолжить" } else { "Пауза" });
            let center = pause.rect.center();
            let color = ui.style().interact(&pause).fg_stroke.color;
            if game.paused {
                ui.painter().add(egui::Shape::convex_polygon(
                    vec![center + vec2(-4.0,-6.0), center + vec2(6.0,0.0), center + vec2(-4.0,6.0)],
                    color, egui::Stroke::NONE,
                ));
            } else {
                for x in [-3.0, 3.0] {
                    ui.painter().line_segment([center + vec2(x,-5.0), center + vec2(x,5.0)], egui::Stroke::new(2.0, color));
                }
            }
            if pause.clicked() { game.paused = !game.paused; }
        });
        ui.horizontal(|ui| {
            let xp = format!("Уровень {}  ·  {} / 100 XP", game.level(), game.taps % 100);
            let width = (ui.available_width() - 44.0).max(0.0);
            let (pocket, _) = ui.allocate_exact_size(vec2(width, 30.0), Sense::hover());
            let dark = ui.visuals().dark_mode;
            let painter = ui.painter();
            painter.rect_filled(pocket, 8.0, crate::theme_color(dark, [230,233,207], [37,49,43]));
            painter.text(pocket.center() - vec2(0.0, 4.0), Align2::CENTER_CENTER, xp, FontId::proportional(12.0), ui.visuals().text_color());
            let track = egui::Rect::from_min_size(pocket.left_bottom() + vec2(8.0,-8.0), vec2((width-16.0).max(0.0), 3.0));
            painter.rect_filled(track, 1.5, crate::theme_color(dark, [197,209,177], [58,74,65]));
            let filled = egui::Rect::from_min_size(track.min, vec2(track.width()*game.level_progress(), track.height()));
            painter.rect_filled(filled, 1.5, GREEN);
            let stats = ui.add_enabled(enabled, egui::Button::new("").min_size(vec2(36.0, 26.0)))
                .on_hover_text("Статистика");
            nav_icon(ui.painter(), stats.rect.center(), 3, ui.style().interact(&stats).fg_stroke.color);
            if stats.clicked() { state.profile(true); }
        });
    });
    egui::TopBottomPanel::bottom("portrait_dock").exact_height(DOCK).resizable(false)
        .frame(egui::Frame::none().fill(ctx.style().visuals.panel_fill).inner_margin(egui::Margin::same(8.0))).show(ctx,|ui| {
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = GAP;
            let available = ui.available_width();
            let fit = ((available - 2.0 * ARROW - GAP) / (TILE + GAP)).floor().max(1.0) as usize;
            let visible = fit.min(NAV_ITEMS);
            let width = visible as f32 * (TILE + GAP) - GAP;
            let group_width = width + 2.0 * (ARROW + GAP);
            ui.add_space((available - group_width).max(0.0) * 0.5);
            let step = (visible.saturating_sub(1).max(1) as f32) * (TILE + GAP);
            if ui.add_enabled(state.sidebar_scroll>0.0,egui::Button::new("‹").min_size(vec2(ARROW,TILE))).clicked() {
                state.sidebar_scroll=(state.sidebar_scroll-step).max(0.0);
            }
            ui.allocate_ui(vec2(width,TILE),|ui| {
                let out=egui::ScrollArea::horizontal().id_salt("portrait_dock_scroll").max_width(width).auto_shrink([false,false]).scroll_offset(vec2(state.sidebar_scroll,0.0))
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui,|ui| {
                    ui.add_enabled_ui(enabled,|ui| {ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = GAP;
                        if tile(ui,"Главная",0,game.tab==Tab::Home&&!state.busy()) {state.close();game.tab=Tab::Home;}
                        if tile(ui,"Капсулы",1,game.tab==Tab::Capsules&&!state.busy()) {state.close();game.tab=Tab::Capsules;}
                        if tile(ui,"Апгрейд",2,state.modal==Modal::Upgrades) {state.modal(Modal::Upgrades);}
                        if tile(ui,"Питомцы",5,state.panel==Panel::Pets) {if state.panel==Panel::Pets {state.close();} else {state.pets();}}
                        if tile(ui,"Ребёрты",4,state.modal==Modal::Rebirths) {state.modal(Modal::Rebirths);}
                        if tile(ui,"Настройки",6,state.modal==Modal::Settings) {state.modal(Modal::Settings);}
                        if tile(ui,"Магазин",7,state.modal==Modal::Shop) {state.modal(Modal::Shop);}
                    });});
                });
                state.sidebar_scroll=out.state.offset.x;state.sidebar_max=(out.content_size.x-out.inner_rect.width()).max(0.0);
            });
            if ui.add_enabled(state.sidebar_scroll<state.sidebar_max,egui::Button::new("›").min_size(vec2(ARROW,TILE))).clicked() {state.sidebar_scroll=(state.sidebar_scroll+step).min(state.sidebar_max);}
        });
    });
}
