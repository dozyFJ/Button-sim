use egui::{Color32 as C,RichText as T,Rect,Sense,Stroke,Align2,vec2,pos2};
use crate::{game::{Game,PetKind,Tab},gui_state::{GuiState,Panel,PetAction},pet_render,panels_ui};
const GREEN:C=C::from_rgb(93,161,71);
pub fn draw(ctx:&egui::Context,workspace:Rect,game:&mut Game,state:&mut GuiState) {
    if state.panel!=Panel::Pets {return;}
    let rect=workspace.shrink(8.0);
    egui::Window::new("pets_inventory").title_bar(false).fixed_rect(rect).resizable(false).collapsible(false).frame(panels_ui::frame(ctx)).show(ctx,|ui| {
        if panels_ui::heading(ui,"Питомцы") {state.close();}
        ui.horizontal_wrapped(|ui| {ui.label(T::new(format!("Надето {}/3",game.equipped_pets.len())).strong());ui.label(format!("Инвентарь {}/1000",game.pets.len()));
            ui.label("Поиск");ui.add(egui::TextEdit::singleline(&mut state.search).hint_text("Имя или редкость").desired_width(160.0));
        });
        ui.horizontal_wrapped(|ui| {
            for (i,title) in ["Инвентарь","Надеты","Крафт"].into_iter().enumerate() {ui.selectable_value(&mut state.pet_tab,i,title);}
            egui::ComboBox::from_id_salt("pet_rarity_filter").selected_text(["Все","Обычные","Редкие","Эпические","Легендарные"][state.rarity]).show_ui(ui,|ui| {for (i,title) in ["Все","Обычные","Редкие","Эпические","Легендарные"].into_iter().enumerate() {ui.selectable_value(&mut state.rarity,i,title);}});
        });ui.separator();
        if state.selected_pet.is_none_or(|i|i>=game.pets.len()) {state.selected_pet=if game.pets.is_empty() {None} else {Some(0)};}
        let body=ui.available_rect_before_wrap();
        let (left,right)=if body.width()<520.0 {
            let split=body.top()+body.height()*0.48;
            (Rect::from_min_max(body.min,pos2(body.right(),split)),Rect::from_min_max(pos2(body.left(),split+8.0),body.max))
        } else {
            let detail_width=(body.width()*0.30).clamp(165.0,260.0);let split=body.right()-detail_width-10.0;
            (Rect::from_min_max(body.min,pos2(split,body.bottom())),Rect::from_min_max(pos2(split+10.0,body.top()),body.max))
        };
        ui.allocate_ui_at_rect(left,|ui| {
            ui.set_clip_rect(left);let query=state.search.trim().to_lowercase();
            let filtered:Vec<usize>=game.pets.iter().enumerate().filter_map(|(i,&pet)| {
                let rank=match pet {PetKind::Cat|PetKind::Dog=>1,PetKind::Bunny=>2,PetKind::Fox=>3,_=>4};
                let matches=format!("{} {}",pet.name(),pet.rarity()).to_lowercase().contains(&query);
                if matches&&(state.rarity==0||state.rarity==rank)&&(state.pet_tab!=1||game.equipped_pets.contains(&i))&&(state.pet_tab!=2||Game::craft_result(pet).is_some()) {Some(i)} else {None}
            }).collect();
            if filtered.is_empty() {ui.label(if game.pets.is_empty() {"Нет питомцев. Открой капсулу."} else {"По запросу ничего не найдено."});
                if game.pets.is_empty()&&ui.button("К капсулам").clicked() {state.close();game.tab=Tab::Capsules;}
            }
            let columns=((left.width()/90.0).floor() as usize).max(2);let tile=(left.width()-8.0)/columns as f32;
            egui::ScrollArea::vertical().id_salt("pet_slots").auto_shrink([false,false]).max_height(ui.available_height()).show(ui,|ui| {
                for row in filtered.chunks(columns) {ui.horizontal(|ui| {ui.spacing_mut().item_spacing.x=3.0;
                    for &index in row {
                        let pet=game.pets[index];let (r,response)=ui.allocate_exact_size(vec2(tile-3.0,94.0),Sense::click());let p=ui.painter();let selected=state.selected_pet==Some(index);let equipped=game.equipped_pets.contains(&index);
                        p.rect_filled(r,0.0,crate::theme_color(ui.visuals().dark_mode, [230,235,207], [40,53,46]));p.rect_stroke(r,0.0,Stroke::new(if selected {3.0_f32} else {1.0_f32},if selected {C::from_rgb(221,166,52)} else {crate::theme_color(ui.visuals().dark_mode, [153,173,117], [94,110,97])}));
                        pet_render::pet(p,Rect::from_min_max(r.min+vec2(5.0,3.0),r.max-vec2(5.0,30.0)),pet,game.played_seconds as f32);
                        p.text(r.left_top()+vec2(5.0,5.0),Align2::LEFT_TOP,(index+1).to_string(),egui::FontId::proportional(9.0),ui.visuals().text_color());
                        if equipped {p.circle_filled(r.right_top()+vec2(-8.0,8.0),4.0,GREEN);}
                        p.text(r.center_bottom()-vec2(0.0,21.0),Align2::CENTER_CENTER,pet.name(),egui::FontId::proportional(10.0),ui.visuals().text_color());
                        p.text(r.center_bottom()-vec2(0.0,8.0),Align2::CENTER_CENTER,format!("×{:.1}",pet.bonus()),egui::FontId::proportional(9.0),ui.visuals().text_color());
                        if response.clicked()&&state.pending.is_none() {state.selected_pet=Some(index);}
                    }
                });ui.add_space(4.0);}
            });
        });
        ui.allocate_ui_at_rect(right,|ui| {
            ui.set_clip_rect(right);egui::ScrollArea::vertical().id_salt("selected_pet_actions").auto_shrink([false,false]).show(ui,|ui| {
                if let Some(index)=state.selected_pet.filter(|&i|i<game.pets.len()) {
                    let pet=game.pets[index];ui.label(T::new(pet.name()).size(18.0).strong());
                    let (r,_)=ui.allocate_exact_size(vec2(ui.available_width(),85.0),Sense::hover());pet_render::pet(ui.painter(),r,pet,game.played_seconds as f32);
                    ui.label(format!("{} · доход ×{:.1}",pet.rarity(),pet.bonus()));ui.separator();
                    ui.add_enabled_ui(!game.paused&&state.pending.is_none(),|ui| {
                        let equipped=game.equipped_pets.contains(&index);
                        let label=if equipped {"Снять"} else if game.equipped_pets.len()==3 {"Надеть · заменить 3-го"} else {"Надеть"};
                        if ui.add_sized(vec2(ui.available_width(),32.0),egui::Button::new(label).fill(ui.visuals().selection.bg_fill)).clicked() {if equipped {game.unequip_pet(index);} else {game.equip_pet(index);}}
                        if ui.add_sized(vec2(ui.available_width(),30.0),egui::Button::new("Удалить")).clicked() {state.pending=Some(PetAction::Delete(index));}
                        let can_craft=game.craft_materials(index).is_some();
                        if ui.add_enabled(can_craft,egui::Button::new("Крафт · 3 одинаковых").min_size(vec2(ui.available_width(),32.0))).clicked() {state.pending=Some(PetAction::Craft(index));}
                        if let Some(next)=Game::craft_result(pet) {ui.label(T::new(format!("3 × {} → {}",pet.name(),next.name())).size(10.0));} else {ui.label("Высшая редкость");}
                    });
                } else {ui.label("Выбери питомца в сетке");}
            });
        });
    });
    if let Some(action)=state.pending {
        let index=match action {PetAction::Delete(i)|PetAction::Craft(i)=>i};
        if index>=game.pets.len() {state.pending=None;return;}
        let pet=game.pets[index];let rect=Rect::from_center_size(workspace.center(),vec2((workspace.width()-32.0).min(390.0),(workspace.height()-24.0).min(220.0)));
        egui::Window::new("pet_confirmation").title_bar(false).fixed_rect(rect).resizable(false).collapsible(false).frame(panels_ui::frame(ctx)).show(ctx,|ui| {
            ui.heading(if matches!(action,PetAction::Delete(_)) {"Удалить питомца?"} else {"Крафт питомца"});
            match action {
                PetAction::Delete(_) => {ui.label(format!("{} будет удалён из инвентаря. Действие нельзя отменить.",pet.name()));}
                PetAction::Craft(_) => {if let Some(next)=Game::craft_result(pet) {ui.label(format!("Потратить 3 × {} и получить {} (×{:.1})?",pet.name(),next.name(),next.bonus()));}}
            }
            ui.add_space(12.0);ui.horizontal(|ui| {
                if ui.button("Отмена").clicked() {state.pending=None;}
                if ui.add_enabled(!game.paused,egui::Button::new("Подтвердить")).clicked() {
                    match action {PetAction::Delete(i)=>{game.delete_pet(i);state.selected_pet=if game.pets.is_empty() {None} else {Some(i.min(game.pets.len()-1))};},PetAction::Craft(i)=>{state.selected_pet=game.craft_pet(i);}}
                    state.pending=None;
                }
            });
        });
    }
}
