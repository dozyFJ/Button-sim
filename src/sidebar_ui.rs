//! Portrait HUD with a compact currency header and a scrolling pill dock.
use egui::{Align2, Color32 as C, FontId, RichText as T, Sense, vec2};
use crate::{amount, game::{Game, Tab}, gui_state::{GuiState, Modal, Panel}, nav_icon};

const INK: C = C::from_rgb(39, 54, 41);
const GREEN: C = C::from_rgb(93, 161, 71);
pub const HEADER: f32 = 88.0;
pub const DOCK: f32 = 64.0;

fn pill(ui: &mut egui::Ui, label: &str, icon: usize, active: bool) -> bool {
    let color = if active { C::BLACK } else { INK };
    let text = ui.painter().layout_no_wrap(label.to_owned(), FontId::proportional(13.0), color);
    let (rect, response) = ui.allocate_exact_size(vec2(51.0 + text.size().x, 40.0), Sense::click());
    let painter = ui.painter();
    let fill = if active {
        GREEN
    } else if response.hovered() {
        C::from_rgb(216, 229, 192)
    } else {
        C::from_rgb(230, 233, 207)
    };
    painter.rect_filled(rect, 20.0, fill);
    let icon_center = rect.left_center() + vec2(22.0, 0.0);
    if icon < 6 {
        nav_icon(painter, icon_center, icon, color);
    } else {
        painter.text(icon_center, Align2::CENTER_CENTER, if icon == 6 { "⚙" } else { "$" }, FontId::proportional(21.0), color);
    }
    let text_position = rect.left_center() + vec2(41.0, -text.size().y / 2.0);
    painter.galley(text_position, text, color);
    response.clicked()
}

pub fn draw(ctx: &egui::Context, game: &mut Game, state: &mut GuiState) {
    let enabled = !game.paused && game.hatch.is_none();
    egui::TopBottomPanel::top("portrait_header").exact_height(HEADER).resizable(false)
        .frame(egui::Frame::none().fill(C::from_rgb(246,238,211)).inner_margin(egui::Margin::same(8.0))).show(ctx, |ui| {
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
                        ui.label(T::new(format!("$ {}", amount(game.coins))).size(19.0).strong().color(INK));
                        ui.add_space(12.0);
                        ui.label(T::new(format!("↻ {}", game.rebirths)).size(17.0).color(INK)).on_hover_text("Ребёрты");
                        ui.add_space(12.0);
                        ui.label(T::new(format!("+{} / сек", amount(game.coins_per_second()))).size(13.0).color(INK));
                    });
                });
            if ui.add_sized(vec2(38.0, 34.0), egui::Button::new(if game.paused { "▶" } else { "Ⅱ" }))
                .on_hover_text("Пауза").clicked()
            {
                game.paused = !game.paused;
            }
        });
        ui.horizontal(|ui| {
            let xp = format!("Уровень {}  ·  {} / 100 XP", game.level(), game.taps % 100);
            ui.add(egui::ProgressBar::new(game.level_progress())
                .desired_width((ui.available_width() - 44.0).max(0.0)).fill(GREEN).text(xp));
            if ui.add_enabled(enabled, egui::Button::new("▥").min_size(vec2(36.0, 24.0)))
                .on_hover_text("Статистика").clicked()
            {
                state.profile(true);
            }
        });
    });
    egui::TopBottomPanel::bottom("portrait_dock").exact_height(DOCK).resizable(false)
        .frame(egui::Frame::none().fill(C::from_rgb(246,238,211)).inner_margin(egui::Margin::same(8.0))).show(ctx,|ui| {
        ui.horizontal_centered(|ui| {
            if ui.add_enabled(state.sidebar_scroll>0.0,egui::Button::new("‹").min_size(vec2(26.0,40.0))).clicked() {state.sidebar_scroll=(state.sidebar_scroll-148.0).max(0.0);}
            let width=(ui.available_width()-34.0).max(80.0);
            ui.allocate_ui(vec2(width,40.0),|ui| {
                let out=egui::ScrollArea::horizontal().id_salt("portrait_dock_scroll").max_width(width).auto_shrink([false,false]).scroll_offset(vec2(state.sidebar_scroll,0.0))
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui,|ui| {
                    ui.add_enabled_ui(enabled,|ui| {ui.horizontal(|ui| {
                        if pill(ui,"Главная",0,game.tab==Tab::Home&&!state.busy()) {state.close();game.tab=Tab::Home;}
                        if pill(ui,"Капсулы",1,game.tab==Tab::Capsules&&!state.busy()) {state.close();game.tab=Tab::Capsules;}
                        if pill(ui,"Улучшения",2,state.modal==Modal::Upgrades) {state.modal(Modal::Upgrades);}
                        if pill(ui,"Питомцы",5,state.panel==Panel::Pets) {if state.panel==Panel::Pets {state.close();} else {state.pets();}}
                        if pill(ui,"Ребёрты",4,state.modal==Modal::Rebirths) {state.modal(Modal::Rebirths);}
                        if pill(ui,"Настройки",6,state.modal==Modal::Settings) {state.modal(Modal::Settings);}
                        if pill(ui,"Магазин",7,state.modal==Modal::Shop) {state.modal(Modal::Shop);}
                    });});
                });
                state.sidebar_scroll=out.state.offset.x;state.sidebar_max=(out.content_size.x-out.inner_rect.width()).max(0.0);
            });
            if ui.add_enabled(state.sidebar_scroll<state.sidebar_max,egui::Button::new("›").min_size(vec2(26.0,40.0))).clicked() {state.sidebar_scroll=(state.sidebar_scroll+148.0).min(state.sidebar_max);}
        });
    });
}
