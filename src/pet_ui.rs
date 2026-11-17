use egui::{Color32 as C, RichText as T, vec2, Rect, Align2, Sense, Stroke, pos2};
use crate::{game::{Game,Capsule},pet_render,amount};
const GREEN:C=C::from_rgb(93,161,71);
fn capsule_card(ui:&mut egui::Ui,game:&mut Game,capsule:Capsule) {
    egui::Frame::none().fill(crate::theme_color(ui.visuals().dark_mode, [249,241,214], [32,29,53])).rounding(18.0)
        .inner_margin(egui::Margin::same(10.0)).show(ui,|ui| {
            ui.vertical_centered(|ui| {
                ui.label(T::new(capsule.name()).size(19.0).strong());
                let (r,_)=ui.allocate_exact_size(vec2(125.0,135.0),Sense::hover());
                pet_render::capsule(ui.painter(),r,capsule==Capsule::Neon,game.played_seconds as f32,0.0);
                ui.horizontal_wrapped(|ui| {for &(pet,chance) in capsule.drops() {
                    ui.label(T::new(format!("{} {}%",pet.name(),chance)).size(10.0));
                }});
                if ui.add_enabled(game.coins>=capsule.price()&&game.pets.len()<1000,
                    egui::Button::new(T::new(format!("Купить · {} денег",amount(capsule.price()))).strong().color(C::WHITE))
                    .fill(ui.visuals().selection.bg_fill).rounding(0.0).min_size(vec2(ui.available_width(),38.0))).clicked() {game.buy_capsule(capsule);}
                if game.pets.len()>=1000 {ui.label("Коллекция заполнена");}
            });
        });
}
pub fn shop(ui:&mut egui::Ui,game:&mut Game) {
    ui.heading("Капсулы");
    let id=egui::Id::new("capsule_rarity");
    let mut selected=ui.ctx().data_mut(|d|d.get_temp::<usize>(id)).unwrap_or(0);
    let labels=["Все","Обычные","Редкие","Эпические","Легендарные"];
    egui::ScrollArea::horizontal().id_salt("capsule_pills").show(ui,|ui| {
        ui.horizontal(|ui| {for (i,label) in labels.iter().enumerate() {
            let active=selected==i;
            if ui.add(egui::Button::new(T::new(*label).color(if active {C::WHITE} else {ui.visuals().text_color()}))
                .fill(if active {ui.visuals().selection.bg_fill} else {crate::theme_color(ui.visuals().dark_mode, [239,242,218], [42,40,75])}).rounding(20.0).min_size(vec2(100.0,36.0))).clicked() {selected=i;}
        }});
    });
    ui.ctx().data_mut(|d|d.insert_temp(id,selected));
    ui.add_space(10.0);
    egui::Frame::none().fill(crate::theme_color(ui.visuals().dark_mode, [225,236,213], [24,23,42])).rounding(22.0).inner_margin(egui::Margin::same(12.0)).show(ui,|ui| {
        ui.set_width(ui.available_width());
        ui.label(T::new(labels[selected]).strong());
        egui::ScrollArea::vertical().id_salt(("capsule_list",selected)).show(ui,|ui| {
            for capsule in [Capsule::Meadow,Capsule::Neon] {
                if selected==0||capsule.drops().iter().any(|(pet,_)|match selected {1=>pet.rarity()=="Обычный",2=>pet.rarity()=="Редкий",3=>pet.rarity()=="Эпический",_=>pet.rarity()=="Легендарный"}) {
                    capsule_card(ui,game,capsule);ui.add_space(10.0);
                }
            }
        });
    });
}
/// A dedicated scene: no sidebar, game field, windows or dialog background.
pub fn overlays(ctx:&egui::Context,game:&mut Game) {
    let Some(hatch)=game.hatch.clone() else {return;};
    let elapsed=hatch.elapsed as f32;let reveal=elapsed>=3.0;
    let Some(&pet)=game.pets.get(hatch.inventory_index) else {return;};
    egui::CentralPanel::default().frame(egui::Frame::none().fill(crate::theme_color(game.dark_mode, [166,207,224], [14,15,29]))).show(ctx,|ui| {
        let width=ui.available_width();let height=ui.available_height();
        let model_size=(height-160.0).min(width*0.44).clamp(72.0,270.0);
        let content_height=model_size+if reveal {156.0} else {128.0};
        ui.add_space(((height-content_height)*0.5).max(0.0));
        ui.vertical_centered(|ui| {
            ui.label(T::new(if reveal {"НОВЫЙ ПИТОМЕЦ!"} else {"ОТКРЫТИЕ КАПСУЛЫ"}).size(19.0).strong());
            ui.label(T::new(if reveal {pet.name()} else {hatch.capsule.name()}).size(23.0).strong());
            let (r,_)=ui.allocate_exact_size(vec2(model_size,model_size),Sense::hover());
            if reveal {pet_render::pet(ui.painter(),r,pet,elapsed);} else {
                pet_render::capsule(ui.painter(),r,hatch.capsule==Capsule::Neon,elapsed,if elapsed<0.7 {0.0} else {((elapsed-0.7)*7.0).min(10.0)});
            }
            let controls=width.min(360.0)-24.0;
            if reveal {
                ui.label(T::new(format!("{} · доход ×{:.1}",pet.rarity(),pet.bonus())).size(16.0));
                ui.add_space(8.0);
                if ui.add_sized(vec2(controls,38.0),egui::Button::new(T::new("Применить").strong().color(C::WHITE)).fill(ui.visuals().selection.bg_fill).rounding(19.0)).clicked() {game.finish_hatch(true);}
                if ui.add_sized(vec2(controls,34.0),egui::Button::new("В коллекцию").rounding(17.0)).clicked() {game.finish_hatch(false);}
            } else {
                ui.label(T::new(if elapsed<0.7 {"Приготовься…"} else if elapsed<2.4 {"ТРЯСЁМ!"} else {"Кто же внутри?"}).size(20.0).strong());
                ui.add(egui::ProgressBar::new((elapsed/3.0).clamp(0.0,1.0)).desired_width(controls).fill(ui.visuals().selection.bg_fill).show_percentage());
            }
        });
    });
    ctx.request_repaint();
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
            p.text(hit-vec2(0.0,18.0+phase*45.0),Align2::CENTER_CENTER,format!("-{}",pet.damage()),egui::FontId::proportional(20.0),crate::theme_color(game.dark_mode, [45,65,39], [232,239,230]));
        }
    }
}

pub fn board_controls(ctx:&egui::Context,workspace:Rect,game:&mut Game,state:&mut crate::gui_state::GuiState) -> Rect {
    let first_page = game.tab == crate::game::Tab::Home;
    let width: f32 = if first_page { 220.0 } else { 72.0 };
    let size = vec2(width.min((workspace.width() - 16.0).max(0.0)), 50.0);
    egui::Area::new(egui::Id::new("canvas_page_controls")).order(egui::Order::Foreground)
        .fixed_pos(workspace.right_bottom() - size - vec2(8.0, 8.0)).show(ctx, |ui| {
        egui::Frame::none().fill(ctx.style().visuals.panel_fill).rounding(14.0).inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
            ui.set_width((size.x - 16.0).max(0.0));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                for (tab, label) in [(crate::game::Tab::Home, "Множитель"), (crate::game::Tab::Rebirths, "Ребёрты")] {
                    let active = game.tab == tab;
                    let (rect, response) = ui.allocate_exact_size(vec2(24.0, 34.0), Sense::click());
                    let ink = ui.visuals().text_color().gamma_multiply(if active { 1.0 } else { 0.42 });
                    ui.painter().circle_filled(rect.center(), if active { 4.0 } else { 3.0 }, ink);
                    if response.on_hover_text(label).clicked() { game.tab = tab; }
                }
                if !first_page { return; }
                ui.separator();
                ui.vertical(|ui| {ui.label(T::new("Автоатака").strong());ui.label(T::new(if game.auto_attack {"Вкл"} else {"Выкл"}).size(11.0));});
                let (rect,response)=ui.allocate_exact_size(vec2(56.0,30.0),Sense::click());
                let t=ctx.animate_bool(response.id,game.auto_attack);
                ui.painter().rect_filled(rect,15.0,if game.auto_attack {GREEN} else {C::from_rgb(155,165,151)});
                ui.painter().circle_filled(pos2(rect.left()+15.0+t*(rect.width()-30.0),rect.center().y),11.0,C::WHITE);
                if response.clicked() {
                    if game.equipped_pets.is_empty() {state.pets();} else {game.toggle_auto_attack();}
                }
            });
        });
    }).response.rect
}

pub fn boost_status(ctx: &egui::Context, workspace: Rect, controls: Rect, game: &Game) {
    if game.active_boosts.is_empty() { return; }
    let left = workspace.left() + 12.0;
    let right = controls.left() - 8.0;
    if right <= left { return; }
    let area = Rect::from_min_max(pos2(left, workspace.top()), pos2(right, workspace.bottom()));
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("boost_status")))
        .with_clip_rect(area);
    let active = game.active_boosts.iter().filter(|boost| boost.expires_at.is_finite() && boost.expires_at > game.played_seconds);
    for (row, boost) in active.enumerate() {
        let y = controls.center().y - 16.0 - row as f32 * 36.0;
        if y < area.top() { break; }
        let seconds = (boost.expires_at - game.played_seconds).ceil() as u64;
        let timer = format!("{}:{:02}", seconds / 60, seconds % 60);
        for (text, y, size) in [(boost.name.as_str(), y, 15.0), (timer.as_str(), y + 17.0, 12.0)] {
            let font = egui::FontId::proportional(size);
            let measured = painter.layout_no_wrap(text.to_owned(), font, C::WHITE);
            let size = size * (area.width() / measured.size().x.max(1.0)).min(1.0);
            let font = egui::FontId::proportional(size);
            let position = pos2(left, y);
            painter.text(position + vec2(1.0, 1.5), Align2::LEFT_TOP, text, font.clone(), C::from_black_alpha(235));
            painter.text(position, Align2::LEFT_TOP, text, font, C::WHITE);
        }
    }
}
