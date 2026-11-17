//! Portrait HUD with a compact currency header and centered navigation tiles.
use egui::{Align2, Color32 as C, FontId, Rect, Sense, pos2, vec2};
use crate::{amount, game::{Game, Tab}, gui_state::{GuiState, Modal, Panel}, nav_icon};

pub const HEADER: f32 = 54.0;
const DOCK_ROW: f32 = 70.0;
const XP_POCKET: f32 = 32.0;
const BOTTOM_MARGIN: f32 = 14.0;
pub const DOCK: f32 = DOCK_ROW + XP_POCKET + BOTTOM_MARGIN;
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
        crate::theme_color(dark, [93,161,71], [103,63,189])
    } else if response.hovered() {
        crate::theme_color(dark, [216,229,192], [64,59,108])
    } else {
        crate::theme_color(dark, [230,233,207], [42,40,75])
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
        let header_right = ui.max_rect().right();
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
            let pause_rect = Rect::from_min_size(pos2(header_right - 38.0, profile.rect.top()), vec2(38.0, 34.0));
            let pause = ui.put(pause_rect, egui::Button::new("").min_size(pause_rect.size()))
                .on_hover_text(if game.paused { "Продолжить" } else { "Пауза" });

            let balance_left = profile.rect.right() + 6.0;
            let max_width = (pause_rect.left() - balance_left - 6.0).max(0.0);
            let painter = ui.painter();
            let text = format!("$ {}", amount(game.coins));
            let color = ui.visuals().text_color();
            let measured = painter.layout_no_wrap(text.clone(), FontId::proportional(19.0), color);
            let width = (measured.size().x + 22.0).max(68.0).min(max_width);
            let font_size = 19.0 * ((width - 16.0).max(0.0) / measured.size().x).min(1.0);
            let balance = Rect::from_min_size(pos2(balance_left, profile.rect.top()), vec2(width, 34.0));
            painter.rect_filled(balance, 8.0, crate::theme_color(ui.visuals().dark_mode, [230,233,207], [53,45,93]));
            painter.text(balance.left_center() + vec2(8.0, 0.0), Align2::LEFT_CENTER, text, FontId::proportional(font_size), color);
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
    });
    egui::TopBottomPanel::bottom("portrait_dock").exact_height(DOCK).resizable(false)
        .frame(egui::Frame::none()).show(ctx, |ui| {
        let dock = ui.available_rect_before_wrap();
        ui.allocate_rect(dock, Sense::hover());
        let dark = ui.visuals().dark_mode;
        let strip = Rect::from_min_size(dock.min, vec2(dock.width(), DOCK_ROW));
        let fill = ui.visuals().panel_fill;
        ui.painter().rect_filled(dock, 0.0, crate::theme_color(dark, [213,223,186], [14,15,29]));
        ui.painter().rect_filled(strip, 0.0, fill);

        let controls = 2.0 * (ARROW + GAP);
        let fit = ((strip.width() - 16.0 - controls + GAP) / (TILE + GAP)).floor().max(1.0) as usize;
        let visible = fit.min(NAV_ITEMS);
        let viewport_width = visible as f32 * (TILE + GAP) - GAP;
        let center = strip.center();
        let viewport = Rect::from_center_size(center, vec2(viewport_width, TILE));
        let left = Rect::from_min_size(viewport.left_top() - vec2(ARROW + GAP, 0.0), vec2(ARROW, TILE));
        let right = Rect::from_min_size(viewport.right_top() + vec2(GAP, 0.0), vec2(ARROW, TILE));
        let max_offset = (NAV_ITEMS - visible) as f32 * (TILE + GAP);
        let step = (visible.saturating_sub(1).max(1) as f32) * (TILE + GAP);
        let left_clicked = ui.put(left, egui::Button::new("‹").sense(if state.sidebar_scroll > 0.0 { Sense::click() } else { Sense::hover() })).clicked();
        let right_clicked = ui.put(right, egui::Button::new("›").sense(if state.sidebar_scroll < max_offset { Sense::click() } else { Sense::hover() })).clicked();
        if left_clicked { state.sidebar_scroll = (state.sidebar_scroll - step).max(0.0); }
        if right_clicked { state.sidebar_scroll = (state.sidebar_scroll + step).min(max_offset); }

        ui.allocate_ui_at_rect(viewport, |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(viewport));
            ui.set_width(viewport_width);
            let out = egui::ScrollArea::horizontal().id_salt("portrait_dock_scroll")
                .max_width(viewport_width).max_height(TILE).auto_shrink([false, false])
                .scroll_offset(vec2(state.sidebar_scroll.min(max_offset), 0.0))
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.add_enabled_ui(enabled, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = GAP;
                            if tile(ui,"Главная",0,game.tab==Tab::Home&&!state.busy()) {state.close();game.tab=Tab::Home;}
                            if tile(ui,"Капсулы",1,game.tab==Tab::Capsules&&!state.busy()) {state.close();game.tab=Tab::Capsules;}
                            if tile(ui,"Апгрейд",2,state.modal==Modal::Upgrades) {state.modal(Modal::Upgrades);}
                            if tile(ui,"Питомцы",5,state.panel==Panel::Pets) {if state.panel==Panel::Pets {state.close();} else {state.pets();}}
                            if tile(ui,"Ребёрты",4,game.tab==Tab::Rebirths&&!state.busy()) {state.close();game.tab=Tab::Rebirths;}
                            if tile(ui,"Настройки",6,state.modal==Modal::Settings) {state.modal(Modal::Settings);}
                            if tile(ui,"Магазин",7,state.modal==Modal::Shop) {state.modal(Modal::Shop);}
                        });
                    });
                });
            state.sidebar_scroll = out.state.offset.x;
        });

        let width = (dock.width() * 0.62).min(230.0);
        let pocket = Rect::from_min_size(pos2(dock.center().x - width * 0.5, strip.bottom()), vec2(width, XP_POCKET));
        let painter = ui.painter();
        painter.rect_filled(pocket, egui::Rounding { nw: 0.0, ne: 0.0, sw: 12.0, se: 12.0 }, fill);
        let xp = format!("Уровень {}  ·  {} / 100 XP", game.level(), game.taps % 100);
        painter.text(pocket.center() - vec2(0.0, 5.0), Align2::CENTER_CENTER, xp, FontId::proportional(12.0), ui.visuals().text_color());
        let track = Rect::from_min_size(pocket.left_bottom() + vec2(10.0, -10.0), vec2((width - 20.0).max(0.0), 4.0));
        painter.rect_filled(track, 2.0, crate::theme_color(dark, [197,209,177], [63,57,102]));
        let filled = Rect::from_min_size(track.min, vec2(track.width() * game.level_progress(), track.height()));
        painter.rect_filled(filled, 2.0, crate::theme_color(dark, [93,161,71], [82,226,186]));
    });
}
