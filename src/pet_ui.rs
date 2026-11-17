use egui::{Color32 as C, RichText as T, vec2, Rect, Align2, Sense, Stroke, pos2};
use crate::{game::{Game,Capsule},pet_render,amount,aero_card};

pub fn shop(ui:&mut egui::Ui,game:&mut Game) {
    ui.heading("Капсулы");
    ui.label(T::new("Купи яйцо и открой своего питомца").color(C::LIGHT_BLUE));
    let time=game.played_seconds as f32;
    for capsule in [Capsule::Meadow,Capsule::Neon] {
        ui.add_space(16.0);
        egui::Frame::none().fill(C::from_rgba_unmultiplied(31,54,91,210))
            .stroke(Stroke::new(1.0,C::from_rgb(91,189,222))).rounding(18.0)
            .inner_margin(egui::Margin::same(14.0)).show(ui,|ui| {
                ui.set_min_width((ui.available_width()-4.0).max(150.0));
                ui.vertical_centered(|ui| {
                    ui.label(T::new(capsule.name()).size(21.0).strong().color(C::WHITE));
                    let (r,_)=ui.allocate_exact_size(vec2(170.0,180.0),Sense::hover());
                    pet_render::capsule(ui.painter(),r,capsule==Capsule::Neon,time,0.0);
                    ui.horizontal_wrapped(|ui| {
                        for &(pet,chance) in capsule.drops() {
                            ui.label(T::new(format!("{} {}%",pet.name(),chance)).size(11.0).color(C::from_rgb(180,208,232)));
                        }
                    });
                    ui.add_space(10.0);
                    if ui.add_enabled(game.coins>=capsule.price()&&game.pets.len()<1000,
                        egui::Button::new(T::new(format!("Купить · {} денег",amount(capsule.price()))).strong().color(C::from_rgb(8,25,44)))
                            .fill(C::from_rgb(109,241,255)).rounding(12.0).min_size(vec2(ui.available_width(),48.0))).clicked() {
                        game.buy_capsule(capsule);
                    }
                    if game.pets.len()>=1000 { ui.label("Коллекция заполнена"); }
                });
            });
    }
}
pub fn overlays(ctx:&egui::Context,game:&mut Game) {
    let screen=ctx.screen_rect();
    let cyan=C::from_rgb(109,241,255);
    if game.hatch.is_none()&&!game.pets_open&&!game.paused {
        egui::Area::new(egui::Id::new("pets_float")).anchor(Align2::RIGHT_BOTTOM,vec2(-16.0,-134.0)).order(egui::Order::Foreground).show(ctx,|ui| {
            let (r,response)=ui.allocate_exact_size(vec2(88.0,64.0),Sense::click());
            aero_card(ui.painter(),r.shrink(3.0),C::from_rgb(221,151,255));
            let c=r.center()-vec2(0.0,10.0);
            ui.painter().circle_filled(c+vec2(0.0,4.0),7.0,cyan);
            for v in [vec2(-10.0,-4.0),vec2(0.0,-10.0),vec2(10.0,-4.0)] { ui.painter().circle_filled(c+v,4.0,cyan); }
            ui.painter().text(r.center()+vec2(0.0,19.0),Align2::CENTER_CENTER,"Питомцы",egui::FontId::proportional(13.0),C::WHITE);
            if response.clicked() { game.pets_open=true; }
        });
    }
    if game.pets_open {
        let mut open=true;
        let rect=Rect::from_min_max(screen.left_top()+vec2(14.0,140.0),screen.right_bottom()-vec2(14.0,128.0));
        egui::Window::new(format!("Питомцы · {}",game.pets.len())).open(&mut open).fixed_rect(rect)
            .collapsible(false).resizable(false).show(ctx,|ui| {
                ui.add_enabled_ui(!game.paused,|ui| {
                    ui.label(T::new("Активен 1 питомец · бонус к доходу").color(cyan));
                    if game.pets.is_empty() {
                        ui.add_space(25.0);ui.label("Коллекция пока пуста");
                        if ui.button("К капсулам").clicked() { game.tab=crate::game::Tab::Capsules;game.pets_open=false; }
                    } else {
                        let pets=game.pets.clone();
                        egui::ScrollArea::vertical().id_salt("pet_inventory").show(ui,|ui| {
                            for row in (0..pets.len()).step_by(2) {
                                ui.columns(2,|cols| {
                                    for (col,column) in cols.iter_mut().enumerate() {
                                        let index=row+col;if index>=pets.len() { continue; }let pet=pets[index];
                                        egui::Frame::none().fill(C::from_rgb(32,48,78)).rounding(12.0).inner_margin(egui::Margin::same(8.0)).show(column,|ui| {
                                            ui.vertical_centered(|ui| {
                                                let size=ui.available_width().min(150.0);
                                                let (r,_)=ui.allocate_exact_size(vec2(size,size),Sense::hover());
                                                pet_render::pet(ui.painter(),r,pet,game.played_seconds as f32);
                                                ui.label(T::new(pet.name()).strong().color(C::WHITE));
                                                ui.label(T::new(format!("{} · ×{:.1}",pet.rarity(),pet.bonus())).size(10.0).color(cyan));
                                                let active=game.equipped_pet==Some(index);
                                                if ui.add_enabled(!active,egui::Button::new(if active {"Применён"} else {"Применить"})).clicked() { game.equip_pet(index); }
                                            });
                                        });
                                    }
                                });ui.add_space(8.0);
                            }
                        });
                    }
                });
            });
        if !open { game.pets_open=false; }
    }
    if let Some(hatch)=game.hatch.clone() {
        let elapsed=hatch.elapsed as f32;
        let reveal=elapsed>=3.0;
        let pet=game.pets[hatch.inventory_index];
        egui::Window::new("Открытие капсулы").id(egui::Id::new("hatch_overlay")).title_bar(false)
            .fixed_rect(screen.shrink(10.0)).resizable(false).collapsible(false)
            .frame(egui::Frame::none().fill(C::from_rgb(12,18,43)).rounding(20.0).stroke(Stroke::new(2.0,cyan)).inner_margin(egui::Margin::same(18.0)))
            .show(ctx,|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label(T::new(if reveal {"НОВЫЙ ПИТОМЕЦ!"} else {"ОТКРЫТИЕ КАПСУЛЫ"}).size(24.0).strong().color(cyan));
                    ui.label(T::new(if reveal {pet.rarity()} else {hatch.capsule.name()}).color(C::WHITE));
                    let size=(screen.width()-76.0).min((screen.height()-280.0).max(100.0)).min(370.0);
                    let (r,_)=ui.allocate_exact_size(vec2(size,size),Sense::hover());
                    if reveal {
                        let p=ui.painter();
                        for i in 0..16 {
                            let a=i as f32*std::f32::consts::TAU/16.0+elapsed*0.15;
                            p.line_segment([r.center()+vec2(a.cos(),a.sin())*size*0.2,r.center()+vec2(a.cos(),a.sin())*size*0.48],Stroke::new(2.0,C::from_rgba_unmultiplied(148,234,255,60)));
                        }
                        pet_render::pet(p,r,pet,elapsed);
                        ui.label(T::new(pet.name()).size(28.0).strong().color(C::WHITE));
                        ui.label(T::new(format!("Бонус дохода ×{:.1}",pet.bonus())).color(cyan));
                        ui.add_space(16.0);
                        ui.add_enabled_ui(!game.paused,|ui| {
                            if ui.add_sized(vec2(ui.available_width(),50.0),egui::Button::new(T::new("Применить").strong().color(C::from_rgb(8,25,44))).fill(cyan)).clicked() { game.finish_hatch(true); }
                            if ui.add_sized(vec2(ui.available_width(),42.0),egui::Button::new("В коллекцию")).clicked() { game.finish_hatch(false); }
                        });
                    } else {
                        let shake=if elapsed<0.7 {0.0} else {(elapsed-0.7)*7.0};
                        pet_render::capsule(ui.painter(),r,hatch.capsule==Capsule::Neon,elapsed,shake);
                        ui.label(T::new(if elapsed<0.7 {"Приготовься…"} else if elapsed<2.4 {"ТРЯСЁМ! · SHAKE"} else {"Кто же внутри?"}).size(24.0).strong().color(C::WHITE));
                        ui.add(egui::ProgressBar::new(elapsed/3.0).fill(cyan).show_percentage());
                    }
                });
            });
        ctx.request_repaint();
    }
}
