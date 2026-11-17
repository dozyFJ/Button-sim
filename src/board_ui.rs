//! One vertically scrolling field; metadata is part of the animated switch face.
use egui::{Color32 as C,Rect,Stroke,Sense,Shape,vec2};
use crate::game::Game;
pub fn show(ui:&mut egui::Ui,game:&mut Game) {
    let dark = game.dark_mode;
    let view=ui.available_rect_before_wrap();ui.painter().rect_filled(view,0.0,crate::theme_color(dark, [121,166,82], [18,29,26]));
    let columns=(view.width()/155.0).floor().clamp(2.0,6.0) as usize;
    let gap=10.0;let side=(view.width()-16.0-(columns-1) as f32*gap)/columns as f32;
    let height=side*0.86;let count=game.switches.len();let rows=count.div_ceil(columns);
    egui::ScrollArea::vertical().id_salt("vertical_switch_field").auto_shrink([false,false]).max_height(view.height())
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).show_rows(ui,height+gap,rows,|ui,range| {
        for row in range {ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x=gap;
            for col in 0..columns {
                let index=row*columns+col;if index>=count {break;}
                let (r,response)=ui.allocate_exact_size(vec2(side,height),Sense::click());let p=ui.painter();
                if !ui.is_rect_visible(r) {continue;}
                let unlocked=game.switch_unlocked(index);let selected=game.selected_switch==index;
                let t=ui.ctx().animate_bool_with_time(ui.id().with(("switch",index)),game.switches[index],0.16);
                let tilt=(t-0.5)*8.0;
                let face=[r.left_top()+vec2(9.0,8.0+tilt),r.right_top()+vec2(-9.0,8.0+tilt),r.right_bottom()+vec2(-4.0,-16.0-tilt),r.left_bottom()+vec2(4.0,-16.0-tilt)];
                let shadow=face.map(|v|v+vec2(3.0,6.0));p.add(Shape::convex_polygon(shadow.to_vec(),C::from_rgba_unmultiplied(43,67,31,60),Stroke::NONE));
                p.add(Shape::convex_polygon(vec![face[2],face[3],face[3]+vec2(0.0,8.0),face[2]+vec2(0.0,8.0)],crate::theme_color(dark, [125,116,85], [12,19,17]),Stroke::NONE));
                p.add(Shape::convex_polygon(face.to_vec(),if !unlocked {crate::theme_color(dark, [165,170,150], [42,49,47])} else if game.switches[index] {crate::theme_color(dark, [140,191,105], [36,91,64])} else {crate::theme_color(dark, [231,190,119], [53,58,46])},Stroke::new(if selected {3.0_f32} else {1.0_f32},if selected {C::from_rgb(230,148,36)} else {crate::theme_color(dark, [104,119,76], [94,110,97])})));
                crate::surface_gui::paint(p,face,&[format!("#{}",index+1),"ЦЕНА 0".into(),"НАГР +1".into(),format!("УР {}",Game::switch_required_level(index)),format!("Р {}",Game::switch_rarity(index))],crate::theme_color(dark, [45,62,38], [232,239,230]));
                if response.clicked() {game.tap_switch(index);}
                if selected&&!game.equipped_pets.is_empty() {
                    crate::pet_ui::attack_scene(p,Rect::from_center_size(r.center(),r.size()),r.center(),side*0.62,game);
                }
            }
        });}
    });
}
