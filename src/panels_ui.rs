use egui::{RichText as T,Rect,vec2,pos2};
use crate::{game::Game,gui_state::{GuiState,Panel,Modal},amount};

pub fn frame(ctx: &egui::Context)->egui::Frame {let style = ctx.style(); egui::Frame::none().fill(style.visuals.window_fill).stroke(style.visuals.window_stroke).inner_margin(egui::Margin::same(10.0))}
pub fn heading(ui:&mut egui::Ui,title:&str)->bool {
    let mut close=false;ui.horizontal(|ui| {ui.label(T::new(title).size(19.0).strong());ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {close=ui.button("X").clicked();});});ui.separator();close
}
fn card(ui:&mut egui::Ui,title:&str,value:&str) {
    egui::Frame::none().fill(crate::theme_color(ui.visuals().dark_mode, [231,235,207], [40,53,46])).inner_margin(egui::Margin::same(10.0)).show(ui,|ui| {
        ui.set_min_width((ui.available_width()-2.0).max(50.0));ui.label(T::new(title).size(11.0).color(crate::theme_color(ui.visuals().dark_mode, [92,109,77], [172,192,180])));ui.label(T::new(value).size(18.0).strong());
    });ui.add_space(5.0);
}
pub fn draw(ctx:&egui::Context,workspace:Rect,game:&mut Game,state:&mut GuiState) {
    if state.panel==Panel::Profile {
        let width=if workspace.width()<600.0 {workspace.width()-16.0} else {(workspace.width()*0.48).clamp(270.0,380.0)};
        let rect=Rect::from_min_max(pos2(workspace.right()-width-8.0,workspace.top()+8.0),workspace.right_bottom()-vec2(8.0,8.0));
        egui::Window::new("profile_panel").title_bar(false).fixed_rect(rect).resizable(false).collapsible(false).frame(frame(ctx)).show(ctx,|ui| {
            if heading(ui,"Мой профиль") {state.close();}
            ui.horizontal(|ui| {ui.selectable_value(&mut state.profile_stats,false,"Профиль");ui.selectable_value(&mut state.profile_stats,true,"Статистика");});
            egui::ScrollArea::vertical().id_salt("profile_cards").show(ui,|ui| {
                if !state.profile_stats {
                    card(ui,"Игрок",game.rank());card(ui,"Уровень",&game.level().to_string());
                    ui.add(egui::ProgressBar::new(game.level_progress()).desired_width(ui.available_width()).fill(ui.visuals().selection.bg_fill).text(format!("{} / 100 XP",game.taps%100)));
                    card(ui,"Деньги",&amount(game.coins));card(ui,"Ребёрты",&game.rebirths.to_string());
                    card(ui,"Команда питомцев",&format!("{} / 3 · ×{:.1}",game.equipped_pets.len(),game.pet_bonus()));
                } else {
                    for (title,value) in [("Нажатия",game.taps.to_string()),("Заработано денег",amount(game.total_earned)),("Доход / сек",amount(game.coins_per_second())),("Уровень",game.level().to_string()),("Ребёрты",game.rebirths.to_string()),("Питомцы",game.pets.len().to_string()),("Удары питомцев",game.auto_hits.to_string()),("В игре",format!("{} мин",(game.played_seconds/60.0) as u64))] {card(ui,title,&value);}
                }
            });
        });
    }
    if state.modal!=Modal::None {
        let rect=Rect::from_center_size(workspace.center(),vec2((workspace.width()-28.0).min(560.0),(workspace.height()-24.0).min(470.0)));
        let modal=state.modal;
        egui::Window::new("game_modal").title_bar(false).fixed_rect(rect).resizable(false).collapsible(false).frame(frame(ctx)).show(ctx,|ui| {
            if heading(ui,match modal {Modal::Upgrades=>"Улучшения",Modal::Rebirths=>"Ребёрты",Modal::Settings=>"Настройки",Modal::Shop=>"Магазин",Modal::None=>""}) {state.close();}
            egui::ScrollArea::vertical().id_salt(("modal_cards",modal==Modal::Upgrades)).show(ui,|ui| {
                if modal==Modal::Settings {
                    ui.checkbox(&mut game.dark_mode, "Тёмная тема");
                    ui.label("Масштаб интерфейса");
                    ui.horizontal(|ui| {
                        for (label,scale) in [("Меньше",0.9),("Обычный",1.0),("Крупнее",1.1)] {if ui.button(label).clicked() {ctx.set_zoom_factor(scale);}}
                    });
                    if ui.button("Пауза и обновление игры").clicked() {state.close();game.paused=true;}
                } else if modal==Modal::Shop {
                    ui.label("Капсулы за игровые деньги");
                    if ui.button("Перейти к капсулам").clicked() {state.close();game.tab=crate::game::Tab::Capsules;}
                    ui.separator();
                    for capsule in [crate::game::Capsule::Meadow,crate::game::Capsule::Neon] {
                        ui.label(T::new(capsule.name()).strong());
                        if ui.add_enabled(!game.paused&&game.coins>=capsule.price()&&game.pets.len()<1000,egui::Button::new(format!("Купить · {} денег",amount(capsule.price()))).fill(ui.visuals().selection.bg_fill)).clicked() {game.buy_capsule(capsule);state.close();}
                        ui.add_space(8.0);
                    }
                } else if modal==Modal::Upgrades {
                    for step in 0..4 {
                        let level=game.upgrades+step;
                        egui::Frame::none().fill(crate::theme_color(ui.visuals().dark_mode, [231,235,207], [40,53,46])).inner_margin(egui::Margin::same(12.0)).show(ui,|ui| {
                            ui.set_min_width((ui.available_width()-2.0).max(120.0));ui.label(T::new(format!("Доход · уровень {}",level+1)).strong());
                            ui.label(format!("Базовый доход {} / сек → {} / сек",amount(game.base_income+step as f64),amount(game.base_income+step as f64+1.0)));
                            let cost=50.0*2.0_f64.powi(level.min(30) as i32);
                            if step==0 {
                                if ui.add_enabled(!game.paused&&game.coins>=cost,egui::Button::new(format!("Улучшить · {} денег",amount(cost))).fill(ui.visuals().selection.bg_fill)).clicked() {game.upgrade();}
                                if game.coins<cost {ui.label(T::new(format!("Не хватает {} денег",amount(cost-game.coins))).size(11.0));}
                            } else {ui.add_enabled(false,egui::Button::new(format!("После предыдущего · {} денег",amount(cost))));}
                        });ui.add_space(8.0);
                    }
                } else {
                    ui.label("Ребёрт сбросит деньги, множитель и улучшения. Питомцы, уровень и статистика останутся.");
                    for step in 0..3 {
                        let count=game.rebirths+step;
                        egui::Frame::none().fill(crate::theme_color(ui.visuals().dark_mode, [245,226,185], [62,51,36])).inner_margin(egui::Margin::same(12.0)).show(ui,|ui| {
                            ui.set_min_width((ui.available_width()-2.0).max(120.0));ui.label(T::new(format!("Ребёрт {}",count+1)).strong());
                            ui.label(format!("Постоянный бонус ×{} → ×{}",amount(2.0_f64.powi(count.min(30) as i32)),amount(2.0_f64.powi((count+1).min(30) as i32))));
                            let cost=10_000.0*10.0_f64.powi(count.min(30) as i32);
                            if step==0 {
                                if ui.add_enabled(!game.paused&&game.coins>=cost,egui::Button::new(format!("Сделать ребёрт · {} денег",amount(cost))).fill(crate::theme_color(ui.visuals().dark_mode, [225,176,77], [111,77,30]))).clicked() {game.rebirth();}
                                if game.coins<cost {ui.label(T::new(format!("Не хватает {} денег",amount(cost-game.coins))).size(11.0));}
                            } else {ui.add_enabled(false,egui::Button::new(format!("Следующий · {} денег",amount(cost))));}
                        });ui.add_space(8.0);
                    }
                }
            });
        });
    }
}
