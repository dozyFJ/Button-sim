//! Portrait HUD: a compact header and a centered, horizontally scrolling dock.
use egui::{Color32 as C,RichText as T,Rect,Sense,Align2,FontId,vec2,pos2};
use crate::{game::{Game,Tab},gui_state::{GuiState,Panel,Modal},amount,nav_icon};
const INK:C=C::from_rgb(39,54,41);const GREEN:C=C::from_rgb(93,161,71);
pub const HEADER:f32=132.0;pub const DOCK:f32=88.0;
fn tile(ui:&mut egui::Ui,label:&str,icon:usize,active:bool)->bool {
    let (r,response)=ui.allocate_exact_size(vec2(66.0,64.0),Sense::click());let p=ui.painter();let color=if active {C::WHITE} else {INK};
    p.rect_filled(r,14.0,if active {GREEN} else {C::from_rgb(230,233,207)});
    if icon<6 {nav_icon(p,r.center()-vec2(0.0,9.0),icon,color);} else {
        p.text(r.center()-vec2(0.0,10.0),Align2::CENTER_CENTER,if icon==6 {"⚙"} else {"$"},FontId::proportional(26.0),color);
    }
    p.text(pos2(r.center().x,r.bottom()-12.0),Align2::CENTER_CENTER,label,FontId::proportional(9.0),color);response.clicked()
}
pub fn draw(ctx:&egui::Context,game:&mut Game,state:&mut GuiState) {
    let enabled=!game.paused&&game.hatch.is_none();
    egui::TopBottomPanel::top("portrait_header").exact_height(HEADER).resizable(false)
        .frame(egui::Frame::none().fill(C::from_rgb(246,238,211)).inner_margin(egui::Margin::same(8.0))).show(ctx,|ui| {
        ui.horizontal_top(|ui| {
            ui.allocate_ui(vec2(62.0,96.0),|ui| {
                let (r,response)=ui.allocate_exact_size(vec2(60.0,70.0),Sense::click());let p=ui.painter();
                let avatar=Rect::from_center_size(r.center()-vec2(0.0,9.0),vec2(40.0,40.0));
                p.rect_filled(avatar,10.0,C::from_rgb(115,174,196));
                p.rect_filled(Rect::from_center_size(avatar.center()-vec2(0.0,7.0),vec2(14.0,14.0)),3.0,C::from_rgb(255,226,175));
                p.rect_filled(Rect::from_center_size(avatar.center()+vec2(0.0,10.0),vec2(26.0,12.0)),3.0,C::from_rgb(66,112,153));
                p.text(r.center_bottom()-vec2(0.0,10.0),Align2::CENTER_CENTER,"Профиль",FontId::proportional(10.0),INK);
                if response.clicked()&&enabled {if state.panel==Panel::Profile {state.close();} else {state.profile(false);}}
                ui.label(T::new(game.rank()).size(9.0).color(INK));
            });
            let currency_width=(ui.available_width()-66.0).max(110.0);
            ui.allocate_ui(vec2(currency_width,112.0),|ui| {
                ui.set_width(currency_width);
                egui::ScrollArea::vertical().id_salt("header_currencies").max_height(112.0).auto_shrink([false,false]).show(ui,|ui| {
                    egui::Frame::none().fill(C::from_rgb(226,235,198)).rounding(12.0).inner_margin(egui::Margin::same(7.0)).show(ui,|ui| {
                        ui.set_width((currency_width-20.0).max(70.0));
                        ui.horizontal(|ui| {ui.label(T::new("$").size(24.0).strong().color(GREEN));ui.vertical(|ui| {ui.label(T::new("ДЕНЬГИ").size(10.0));ui.label(T::new(amount(game.coins)).size(23.0).strong());});});
                    });
                    ui.label(T::new(format!("Уровень {}",game.level())).size(11.0));
                    ui.add(egui::ProgressBar::new(game.level_progress()).desired_width(currency_width-8.0).fill(GREEN).text(format!("{} / 100 XP",game.taps%100)));
                    ui.add_space(8.0);
                    ui.label(T::new(format!("Ребёрты · {}",game.rebirths)).strong());
                    ui.label(format!("+{} денег / сек",amount(game.coins_per_second())));
                    ui.label(T::new("Прокрути вверх к деньгам ↑").size(9.0));
                });
            });
            ui.vertical(|ui| {
                if ui.add_enabled(enabled,egui::Button::new(T::new("▥").size(20.0)).min_size(vec2(46.0,34.0))).on_hover_text("Статистика").clicked() {state.profile(true);}
                if ui.add_sized(vec2(46.0,34.0),egui::Button::new(if game.paused {"▶"} else {"Ⅱ"})).on_hover_text("Пауза").clicked() {game.paused=!game.paused;}
                ui.label(T::new("↓ ещё").size(9.0));
            });
        });
    });
    egui::TopBottomPanel::bottom("portrait_dock").exact_height(DOCK).resizable(false)
        .frame(egui::Frame::none().fill(C::from_rgb(246,238,211)).inner_margin(egui::Margin::same(8.0))).show(ctx,|ui| {
        ui.horizontal_centered(|ui| {
            if ui.add_enabled(state.sidebar_scroll>0.0,egui::Button::new("‹").min_size(vec2(26.0,60.0))).clicked() {state.sidebar_scroll=(state.sidebar_scroll-148.0).max(0.0);}
            let width=(ui.available_width()-34.0).max(80.0);
            ui.allocate_ui(vec2(width,68.0),|ui| {
                let out=egui::ScrollArea::horizontal().id_salt("portrait_dock_scroll").max_width(width).auto_shrink([false,false]).scroll_offset(vec2(state.sidebar_scroll,0.0))
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui,|ui| {
                    ui.add_enabled_ui(enabled,|ui| {ui.horizontal(|ui| {
                        if tile(ui,"Главная",0,game.tab==Tab::Home&&!state.busy()) {state.close();game.tab=Tab::Home;}
                        if tile(ui,"Настройки",6,state.modal==Modal::Settings) {state.modal(Modal::Settings);}
                        if tile(ui,"Магазин",7,state.modal==Modal::Shop) {state.modal(Modal::Shop);}
                        if tile(ui,"Улучшения",2,state.modal==Modal::Upgrades) {state.modal(Modal::Upgrades);}
                        if tile(ui,"Капсулы",1,game.tab==Tab::Capsules&&!state.busy()) {state.close();game.tab=Tab::Capsules;}
                        if tile(ui,"Питомцы",5,state.panel==Panel::Pets) {if state.panel==Panel::Pets {state.close();} else {state.pets();}}
                        if tile(ui,"Ребёрты",4,state.modal==Modal::Rebirths) {state.modal(Modal::Rebirths);}
                    });});
                });
                state.sidebar_scroll=out.state.offset.x;state.sidebar_max=(out.content_size.x-out.inner_rect.width()).max(0.0);
            });
            if ui.add_enabled(state.sidebar_scroll<state.sidebar_max,egui::Button::new("›").min_size(vec2(26.0,60.0))).clicked() {state.sidebar_scroll=(state.sidebar_scroll+148.0).min(state.sidebar_max);}
        });
    });
}
