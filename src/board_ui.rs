//! Perspective ground field. Scroll moves the view along the board, not a list of flat buttons.
use egui::{Color32 as C, RichText as T, Rect, Stroke, Sense, Align2, FontId, Shape, pos2, vec2};
use crate::game::{Game,Tab};

pub fn show(ui:&mut egui::Ui,game:&mut Game) {
    let mut focus=false;
    ui.horizontal_wrapped(|ui| {
        ui.label(T::new("ПОЛЕ КНОПОК").size(14.0).strong());
        for size in [8,32] {
            if ui.selectable_label(game.grid_size==size,format!("{size}×{size}")).clicked() {game.set_grid_size(size);}
        }
        ui.label(T::new(format!("×{} · цель №{}",game.multiplier,game.selected_switch+1)).color(C::from_rgb(42,83,40)));
        focus=ui.button("К цели").clicked();
    });
    let columns=game.grid_size;
    let height=(ui.available_height()-20.0).max(80.0);
    let bias=if ui.available_width()>ui.available_height()*1.3 {3.5} else {1.5};
    let world=vec2(columns as f32*110.0,columns as f32*110.0);
    let mut scroll=egui::ScrollArea::both().id_salt(("ground_field",columns)).auto_shrink([false,false]).max_height(height);
    if focus {
        scroll=scroll.scroll_offset(vec2(((game.selected_switch%columns) as f32-bias).max(0.0)*110.0,(game.selected_switch/columns) as f32*110.0));
    }
    scroll.show_viewport(ui,|ui,viewport| {
        ui.set_min_size(world);
        let view=ui.clip_rect();
        let p=ui.painter().with_clip_rect(view);
        p.rect_filled(view,0.0,C::from_rgb(160,205,225));
        p.rect_filled(Rect::from_min_max(pos2(view.left(),view.top()+view.height()*0.12),view.right_bottom()),0.0,C::from_rgb(121,166,82));
        let camera_x=(bias+viewport.min.x/110.0).min(columns as f32-0.5);
        let camera_z=viewport.min.y/110.0;
        let focal=(view.width()*0.85).min(view.height()*0.95);
        let horizon=view.top()+view.height()*0.12;
        let project=|x:f32,y:f32,z:f32| {
            let depth=(z-camera_z+3.0).max(0.3);
            pos2(view.center().x+(x-camera_x)*focal/depth,horizon+(2.5-y)*focal/depth)
        };
        // Parallel ground rails converge toward the horizon.
        for x in 0..=columns {
            p.line_segment([project(x as f32-0.5,0.0,camera_z-0.5),project(x as f32-0.5,0.0,camera_z+9.0)],
                Stroke::new(1.0,C::from_rgba_unmultiplied(71,110,48,80)));
        }
        let mut selected=None;
        for row in (0..columns).rev() {
            let depth=row as f32-camera_z+3.0;
            if !(2.2..10.0).contains(&depth) {continue;}
            for col in 0..columns {
                let index=row*columns+col;
                let x=col as f32;let z=row as f32;
                let corners=[
                    project(x-0.42,0.06,z-0.42),project(x+0.42,0.06,z-0.42),
                    project(x+0.42,0.06,z+0.42),project(x-0.42,0.06,z+0.42),
                ];
                let bounds=Rect::from_points(&corners).expand(4.0);
                if !view.intersects(bounds) {continue;}
                let unlocked=game.switch_unlocked(index);
                let active=game.selected_switch==index;
                let color=if !unlocked {C::from_rgb(109,117,104)} else if active {C::from_rgb(227,165,47)} else {C::from_rgb(82,107,63)};
                let turf=[project(x-0.5,0.0,z-0.5),project(x+0.5,0.0,z-0.5),project(x+0.5,0.0,z+0.5),project(x-0.5,0.0,z+0.5)];
                p.add(Shape::convex_polygon(turf.to_vec(),if (row+col)%2==0 {C::from_rgb(124,170,83)} else {C::from_rgb(132,177,90)},Stroke::new(0.5,C::from_rgb(112,154,71))));
                p.add(Shape::convex_polygon(corners.to_vec(),C::from_rgb(199,190,158),Stroke::new(if active {2.0} else {1.0},color)));
                let t=ui.ctx().animate_bool_with_time(ui.id().with(("rocker",index)),game.switches[index],0.16);
                let front=0.12+0.16*t;let back=0.28-0.16*t;
                let rocker=[
                    project(x-0.27,front,z-0.27),project(x+0.27,front,z-0.27),
                    project(x+0.27,back,z+0.27),project(x-0.27,back,z+0.27),
                ];
                p.add(Shape::convex_polygon(vec![corners[0],corners[1],rocker[1],rocker[0]],C::from_rgb(111,111,84),Stroke::NONE));
                p.add(Shape::convex_polygon(rocker.to_vec(),if !unlocked {C::from_rgb(147,153,140)} else if game.switches[index] {C::from_rgb(96,167,76)} else {C::from_rgb(210,107,73)},Stroke::new(1.0,color)));
                let center=project(x,0.26,z);
                p.text(center,Align2::CENTER_CENTER,if game.switches[index] {"I"} else {"O"},FontId::proportional((focal/depth*0.16).clamp(9.0,16.0)),C::from_rgb(252,245,221));
                // Surface label is a billboard: it remains facing the player.
                let label=project(x,0.5,z+0.36);
                let plate=Rect::from_center_size(label-vec2(0.0,6.0),vec2(if unlocked {34.0} else {48.0},if unlocked {16.0} else {28.0}));
                p.rect_filled(plate,0.0,C::from_rgb(246,238,208));
                p.rect_stroke(plate,0.0,Stroke::new(1.0,C::from_rgb(137,145,104)));
                p.text(label,Align2::CENTER_BOTTOM,format!("#{}",index+1),FontId::proportional((focal/depth*0.18).clamp(9.0,14.0)),C::from_rgb(50,66,42));
                if !unlocked {
                    p.text(label+vec2(0.0,11.0),Align2::CENTER_BOTTOM,format!("Ур. {}",Game::switch_required_level(index)),FontId::proportional(9.0),C::from_rgb(81,91,67));
                }
                let response=ui.interact(bounds,ui.id().with(("ground_switch",index)),Sense::click());
                if response.clicked()&&unlocked {game.tap_switch(index);}
                if active {selected=Some((center,focal/depth));}
            }
        }
        if let Some((center,scale))=selected {
            let stage=Rect::from_center_size(center,vec2(scale*2.6,scale*2.4));
            crate::pet_ui::attack_scene(&p,stage,center,scale*0.82,game);
        }
    });
    if game.tab==Tab::Home {ui.label(T::new("Тяни поле в стороны и вперёд · первые 5 кнопок открыты").size(10.0).color(C::from_rgb(60,86,50)));}
}

