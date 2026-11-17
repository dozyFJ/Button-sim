//! Full-width isometric ground canvas with drag, zoom and depth-ordered 3D switches.
use egui::{Color32 as C, RichText as T, Rect, Stroke, Sense, Align2, FontId, Shape, pos2, vec2};
use crate::{game::Game,isometric::{MapCamera,quad_contains}};

pub fn show(ui:&mut egui::Ui,game:&mut Game) {
    let view=ui.available_rect_before_wrap();
    let center=[view.center().x,view.center().y];
    let camera_id=egui::Id::new(("isometric_camera",game.grid_size));
    let mut camera=ui.ctx().data_mut(|data|data.get_temp::<MapCamera>(camera_id)).unwrap_or(MapCamera{
        x:3.5,z:3.5,zoom:(view.width()/12.5).min(view.height()/8.2).clamp(24.0,90.0)
    });
    let mut focus=false;
    if !game.pets_open&&game.hatch.is_none()&&!game.paused {
        egui::Area::new(egui::Id::new("field_controls")).fixed_pos(view.min+vec2(8.0,6.0)).order(egui::Order::Foreground).movable(false).show(ui.ctx(),|ui| {
            ui.set_max_width((ui.ctx().screen_rect().width()-24.0).max(250.0));
            egui::Frame::none().fill(C::from_rgb(246,238,211)).inner_margin(egui::Margin::same(5.0)).show(ui,|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(T::new("КАРТА").size(13.0).strong());
                    for size in [8,32] {if ui.selectable_label(game.grid_size==size,format!("{size}×{size}")).clicked() {game.set_grid_size(size);}}
                    if ui.small_button("-").clicked() {camera.zoom_at(1.0/1.18,center,center);}
                    if ui.small_button("+").clicked() {camera.zoom_at(1.18,center,center);}
                    ui.label(T::new(format!("×{} · цель №{}",game.multiplier,game.selected_switch+1)).color(C::from_rgb(42,83,40)));
                    focus=ui.button("К цели").clicked();
                });
            });
        });
    }
    if focus {camera.x=(game.selected_switch%game.grid_size) as f32;camera.z=(game.selected_switch/game.grid_size) as f32;}
    let (canvas,response)=ui.allocate_exact_size(view.size(),Sense::click_and_drag());
    if response.dragged() {
        let delta=ui.ctx().input(|input|input.pointer.delta());
        camera.pan([delta.x,delta.y]);
    }
    if response.hovered() {
        let (zoom,wheel,pointer)=ui.ctx().input(|i|(i.zoom_delta(),i.smooth_scroll_delta.y,i.pointer.hover_pos()));
        let factor=if (zoom-1.0).abs()>0.001 {zoom} else {(wheel*0.002).exp()};
        let pointer=pointer.unwrap_or(canvas.center());
        camera.zoom_at(factor,[pointer.x,pointer.y],center);
    }
    let columns=game.grid_size;
    camera.x=camera.x.clamp(-2.0,columns as f32+1.0);camera.z=camera.z.clamp(-2.0,columns as f32+1.0);
    ui.ctx().data_mut(|data|data.insert_temp(egui::Id::new(("isometric_camera",columns)),camera));
    let p=ui.painter().with_clip_rect(canvas);
    p.rect_filled(canvas,0.0,C::from_rgb(121,166,82));
    let project=|x:f32,y:f32,z:f32| {let v=camera.project([x,y,z],center);pos2(v[0],v[1])};
    let pointer=if response.clicked() {response.interact_pointer_pos().map(|p|[p.x,p.y])} else {None};
    let mut picked=None;let mut selected=None;
    for depth in 0..(columns*2-1) {
        for col in 0..columns {
            if depth<col {continue;}let row=depth-col;if row>=columns {continue;}
            let index=row*columns+col;let x=col as f32;let z=row as f32;
            let corners=[project(x-0.42,0.08,z-0.42),project(x+0.42,0.08,z-0.42),project(x+0.42,0.08,z+0.42),project(x-0.42,0.08,z+0.42)];
            if !canvas.intersects(Rect::from_points(&corners).expand(40.0)) {continue;}
            let turf=[project(x-0.5,0.0,z-0.5),project(x+0.5,0.0,z-0.5),project(x+0.5,0.0,z+0.5),project(x-0.5,0.0,z+0.5)];
            p.add(Shape::convex_polygon(turf.to_vec(),if (row+col)%2==0 {C::from_rgb(124,170,83)} else {C::from_rgb(132,177,90)},Stroke::new(0.6,C::from_rgb(107,148,66))));
            let unlocked=game.switch_unlocked(index);let active=game.selected_switch==index;
            let outline=if active {C::from_rgb(227,165,47)} else {C::from_rgb(100,117,77)};
            p.add(Shape::convex_polygon(corners.map(|v|v+vec2(3.0,4.0)).to_vec(),C::from_rgba_unmultiplied(44,67,32,48),Stroke::NONE));
            p.add(Shape::convex_polygon(vec![project(x+0.42,0.0,z-0.42),project(x+0.42,0.0,z+0.42),corners[2],corners[1]],C::from_rgb(143,137,111),Stroke::NONE));
            p.add(Shape::convex_polygon(vec![project(x+0.42,0.0,z+0.42),project(x-0.42,0.0,z+0.42),corners[3],corners[2]],C::from_rgb(171,162,130),Stroke::NONE));
            p.add(Shape::convex_polygon(corners.to_vec(),C::from_rgb(205,195,161),Stroke::new(if active {2.5} else {1.0},outline)));
            let t=ui.ctx().animate_bool_with_time(ui.id().with(("rocker",index)),game.switches[index],0.16);
            let front=0.12+0.16*t;let back=0.28-0.16*t;
            let rocker=[project(x-0.27,front,z-0.27),project(x+0.27,front,z-0.27),project(x+0.27,back,z+0.27),project(x-0.27,back,z+0.27)];
            p.add(Shape::convex_polygon(vec![corners[1],corners[2],rocker[2],rocker[1]],C::from_rgb(111,111,84),Stroke::NONE));
            p.add(Shape::convex_polygon(vec![corners[2],corners[3],rocker[3],rocker[2]],C::from_rgb(136,130,99),Stroke::NONE));
            p.add(Shape::convex_polygon(rocker.to_vec(),if !unlocked {C::from_rgb(147,153,140)} else if game.switches[index] {C::from_rgb(96,167,76)} else {C::from_rgb(210,107,73)},Stroke::new(1.0,outline)));
            let tile_center=project(x,0.22,z);
            p.text(tile_center,Align2::CENTER_CENTER,if game.switches[index] {"I"} else {"O"},FontId::proportional((camera.zoom*0.2).clamp(9.0,17.0)),C::from_rgb(252,245,221));
            let label=project(x,0.55,z);
            let plate=Rect::from_center_size(label-vec2(0.0,6.0),vec2((camera.zoom*1.4).clamp(34.0,48.0),if unlocked {16.0} else {28.0}));
            p.rect_filled(plate,0.0,C::from_rgb(246,238,208));p.rect_stroke(plate,0.0,Stroke::new(1.0,C::from_rgb(137,145,104)));
            p.text(label,Align2::CENTER_BOTTOM,format!("#{}",index+1),FontId::proportional((camera.zoom*0.2).clamp(9.0,13.0)),C::from_rgb(50,66,42));
            if !unlocked {p.text(label+vec2(0.0,11.0),Align2::CENTER_BOTTOM,format!("Ур.{}",Game::switch_required_level(index)),FontId::proportional(8.0),C::from_rgb(81,91,67));}
            if let Some(pointer)=pointer {
                if quad_contains(pointer,corners.map(|v|[v.x,v.y]))||quad_contains(pointer,rocker.map(|v|[v.x,v.y])) {picked=Some(index);}
            }
            if active {selected=Some(tile_center);}
        }
    }
    if let Some(index)=picked {game.tap_switch(index);}
    if let Some(target)=selected {
        let stage=Rect::from_center_size(target,vec2(camera.zoom*2.6,camera.zoom*2.4));
        crate::pet_ui::attack_scene(&p,stage,target,camera.zoom*0.82,game);
    }
}

