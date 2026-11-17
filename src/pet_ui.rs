use egui::{Color32 as C, RichText as T, vec2, Rect, Align2, Sense, Stroke, pos2};
use crate::{game::{Game,Capsule},pet_render,amount,cartoon_card};
const INK:C=C::from_rgb(39,54,41);
const GREEN:C=C::from_rgb(93,161,71);
fn capsule_card(ui:&mut egui::Ui,game:&mut Game,capsule:Capsule) {
    egui::Frame::none().fill(C::from_rgb(249,241,214)).stroke(Stroke::new(2.0,C::from_rgb(130,150,105)))
        .inner_margin(egui::Margin::same(10.0)).show(ui,|ui| {
            ui.vertical_centered(|ui| {
                ui.label(T::new(capsule.name()).size(19.0).strong().color(INK));
                let (r,_)=ui.allocate_exact_size(vec2(125.0,135.0),Sense::hover());
                pet_render::capsule(ui.painter(),r,capsule==Capsule::Neon,game.played_seconds as f32,0.0);
                ui.horizontal_wrapped(|ui| {for &(pet,chance) in capsule.drops() {
                    ui.label(T::new(format!("{} {}%",pet.name(),chance)).size(10.0).color(INK));
                }});
                if ui.add_enabled(game.coins>=capsule.price()&&game.pets.len()<1000,
                    egui::Button::new(T::new(format!("Купить · {} денег",amount(capsule.price()))).strong().color(C::WHITE))
                    .fill(GREEN).rounding(0.0).min_size(vec2(ui.available_width(),38.0))).clicked() {game.buy_capsule(capsule);}
                if game.pets.len()>=1000 {ui.label("Коллекция заполнена");}
            });
        });
}
pub fn shop(ui:&mut egui::Ui,game:&mut Game) {
    ui.heading("Капсулы");
    if ui.available_width()>540.0 {
        ui.columns(2,|cols| {capsule_card(&mut cols[0],game,Capsule::Meadow);capsule_card(&mut cols[1],game,Capsule::Neon);});
    } else {for c in [Capsule::Meadow,Capsule::Neon] {capsule_card(ui,game,c);ui.add_space(10.0);}}
}
pub fn overlays(ctx:&egui::Context,game:&mut Game) {
    let screen=ctx.screen_rect();
    if game.hatch.is_none()&&!game.pets_open&&!game.paused {
        egui::Area::new(egui::Id::new("pets_float")).anchor(Align2::RIGHT_BOTTOM,vec2(-12.0,-12.0)).order(egui::Order::Foreground).show(ctx,|ui| {
            let (r,response)=ui.allocate_exact_size(vec2(88.0,54.0),Sense::click());
            cartoon_card(ui.painter(),r.shrink(3.0),GREEN);
            let c=r.center()-vec2(0.0,8.0);
            ui.painter().circle_filled(c+vec2(0.0,4.0),6.0,GREEN);
            for v in [vec2(-9.0,-4.0),vec2(0.0,-9.0),vec2(9.0,-4.0)] {ui.painter().circle_filled(c+v,3.5,GREEN);}
            ui.painter().text(r.center()+vec2(0.0,17.0),Align2::CENTER_CENTER,"Питомцы",egui::FontId::proportional(12.0),INK);
            if response.clicked() {game.pets_open=true;}
        });
    }
    if game.pets_open {
        let mut open=true;
        let rect=Rect::from_min_max(screen.left_top()+vec2(16.0,118.0),screen.right_bottom()-vec2(16.0,10.0));
        egui::Window::new(format!("Питомцы · {}",game.pets.len())).open(&mut open).fixed_rect(rect).collapsible(false).resizable(false).show(ctx,|ui| {
            ui.add_enabled_ui(!game.paused,|ui| {
                ui.label(T::new(format!("Команда {}/3 · доход ×{:.1}",game.equipped_pets.len(),game.pet_bonus())).color(INK));
                if game.pets.is_empty() {
                    ui.label("Коллекция пока пуста");
                    if ui.button("К капсулам").clicked() {game.tab=crate::game::Tab::Capsules;game.pets_open=false;}
                } else {
                    let pets=game.pets.clone();let columns=if screen.width()>700.0 {4} else {2};
                    egui::ScrollArea::vertical().id_salt("pet_inventory").show(ui,|ui| {
                        for row in (0..pets.len()).step_by(columns) {
                            ui.columns(columns,|cols| {for (col,column) in cols.iter_mut().enumerate() {
                                let index=row+col;if index>=pets.len() {continue;}let pet=pets[index];
                                egui::Frame::none().fill(C::from_rgb(230,235,207)).inner_margin(egui::Margin::same(6.0)).show(column,|ui| {
                                    ui.vertical_centered(|ui| {
                                        let size=ui.available_width().min(88.0);
                                        let (r,_)=ui.allocate_exact_size(vec2(size,size),Sense::hover());
                                        pet_render::pet(ui.painter(),r,pet,game.played_seconds as f32);
                                        ui.label(T::new(pet.name()).strong().color(INK));
                                        ui.label(T::new(format!("{} · ×{:.1}",pet.rarity(),pet.bonus())).size(10.0).color(INK));
                                        let active=game.equipped_pets.contains(&index);
                                        let label=if active {"Снять"} else if game.equipped_pets.len()==3 {"Заменить 3-го"} else {"Применить"};
                                        if ui.button(label).clicked() {if active {game.unequip_pet(index);} else {game.equip_pet(index);}}
                                    });
                                });
                            }});ui.add_space(6.0);
                        }
                    });
                }
            });
        });if !open {game.pets_open=false;}
    }
    if let Some(hatch)=game.hatch.clone() {
        let elapsed=hatch.elapsed as f32;let reveal=elapsed>=3.0;let pet=game.pets[hatch.inventory_index];
        egui::Window::new("Открытие капсулы").id(egui::Id::new("hatch_overlay")).title_bar(false)
            .fixed_rect(screen.shrink(12.0)).resizable(false).collapsible(false).vscroll(true)
            .frame(egui::Frame::none().fill(C::from_rgb(250,242,216)).stroke(Stroke::new(2.0,GREEN)).inner_margin(egui::Margin::same(12.0)))
            .show(ctx,|ui| {
                ui.columns(2,|cols| {
                    cols[0].vertical_centered(|ui| {
                        let size=(screen.height()-80.0).min(ui.available_width()).clamp(100.0,300.0);
                        let (r,_)=ui.allocate_exact_size(vec2(size,size),Sense::hover());
                        if reveal {pet_render::pet(ui.painter(),r,pet,elapsed);} else {
                            pet_render::capsule(ui.painter(),r,hatch.capsule==Capsule::Neon,elapsed,if elapsed<0.7 {0.0} else {(elapsed-0.7)*7.0});
                        }
                    });
                    cols[1].vertical_centered(|ui| {
                        ui.label(T::new(if reveal {"НОВЫЙ ПИТОМЕЦ!"} else {"ОТКРЫТИЕ КАПСУЛЫ"}).size(20.0).strong().color(INK));
                        if reveal {
                            ui.label(T::new(pet.name()).size(25.0).strong().color(INK));
                            ui.label(format!("{} · доход ×{:.1}",pet.rarity(),pet.bonus()));
                            ui.add_space(12.0);
                            ui.add_enabled_ui(!game.paused,|ui| {
                                if ui.add_sized(vec2(ui.available_width(),44.0),egui::Button::new(T::new("Применить").strong().color(C::WHITE)).fill(GREEN)).clicked() {game.finish_hatch(true);}
                                if ui.add_sized(vec2(ui.available_width(),36.0),egui::Button::new("В коллекцию")).clicked() {game.finish_hatch(false);}
                            });
                        } else {
                            ui.label(hatch.capsule.name());ui.add_space(20.0);
                            ui.label(T::new(if elapsed<0.7 {"Приготовься…"} else if elapsed<2.4 {"ТРЯСЁМ!"} else {"Кто же внутри?"}).size(24.0).strong().color(INK));
                            ui.add(egui::ProgressBar::new(elapsed/3.0).fill(GREEN).show_percentage());
                        }
                    });
                });
            });ctx.request_repaint();
    }
}

/// Position and lunge use the same simulation clock that awards actual hits.
pub fn attack_scene(p:&egui::Painter,rect:Rect,center:egui::Pos2,button_size:f32,game:&Game) {
    let count=game.equipped_pets.len();
    for (slot,&index) in game.equipped_pets.iter().enumerate() {
        let Some(&pet)=game.pets.get(index) else {continue;};
        let angle=if count==1 {std::f32::consts::FRAC_PI_2}
            else if count==2 {if slot==0 {2.8} else {0.34}}
            else {[-2.7_f32,-0.44,1.57][slot]};
        let direction=vec2(angle.cos(),angle.sin());
        let radius=button_size*0.64;
        let target=center+direction*radius;
        let start=pos2(rect.left()+35.0+slot as f32*65.0,rect.bottom()-32.0);
        let travel=if game.auto_attack {(game.attack_clock as f32/0.7).clamp(0.0,1.0)} else {0.0};
        let smooth=travel*travel*(3.0-2.0*travel);
        let mut position=start+(target-start)*smooth;
        if game.auto_attack&&travel>=1.0 {
            let phase=game.attack_phase(slot);
            if game.attack_clock>=0.7+slot as f64*0.18 {
                // Anticipate the next simulation hit, then recoil on its frame.
                let lunge=if phase>0.78 {(phase-0.78)/0.22} else if game.pet_is_striking(slot) {1.0-phase/0.2} else {0.0};
                position-=direction*lunge.clamp(0.0,1.0)*button_size*0.16;
            }
        }
        let size=(button_size*0.36).clamp(54.0,78.0);
        let model=Rect::from_center_size(position,vec2(size,size));
        let tint=pet.color();let color=C::from_rgb(tint[0],tint[1],tint[2]);
        if game.auto_attack {
            p.line_segment([position,center+direction*button_size*0.43],Stroke::new(1.5,C::from_rgba_unmultiplied(tint[0],tint[1],tint[2],110)));
        }
        pet_render::pet(p,model,pet,game.played_seconds as f32+slot as f32);
        if game.pet_is_striking(slot) {
            let hit=center+direction*button_size*0.43;
            let phase=game.attack_phase(slot);
            for i in 0..6 {
                let angle=i as f32*std::f32::consts::TAU/6.0;
                let v=vec2(angle.cos(),angle.sin());
                p.line_segment([hit+v*4.0,hit+v*(8.0+phase*70.0)],Stroke::new(2.0,color));
            }
            p.text(hit-vec2(0.0,18.0+phase*45.0),Align2::CENTER_CENTER,"+1",egui::FontId::proportional(20.0),C::from_rgb(45,65,39));
        }
    }
}
