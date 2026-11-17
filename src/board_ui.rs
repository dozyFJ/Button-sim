//! One vertically scrolling field; metadata is part of the animated switch face.
use egui::{Color32 as C,Rect,Stroke,Sense,Shape,pos2,vec2};
use crate::game::Game;
fn draw_floor(ui: &egui::Ui, view: Rect, dark: bool) {
    let painter = ui.painter().with_clip_rect(view);
    let base = crate::theme_color(dark, [119,159,85], [24,26,46]);
    let grid = crate::theme_color(dark, [102,140,75], [33,36,65]);
    let stud_top = crate::theme_color(dark, [143,181,108], [53,55,94]);
    let stud_side = crate::theme_color(dark, [92,128,64], [16,18,37]);
    painter.rect_filled(view, 0.0, base);

    let tile = 22.0;
    let columns = (view.width() / tile).ceil() as usize;
    let rows = (view.height() / tile).ceil() as usize;
    for column in 1..columns {
        let x = view.left() + column as f32 * tile;
        painter.line_segment([pos2(x, view.top()), pos2(x, view.bottom())], Stroke::new(1.0, grid));
    }
    for row in 0..rows {
        let y = view.top() + row as f32 * tile;
        painter.line_segment([pos2(view.left(), y), pos2(view.right(), y)], Stroke::new(1.0, grid));
        for column in 0..columns {
            let center = pos2(view.left() + (column as f32 + 0.5) * tile, y + tile * 0.5);
            let stud = Rect::from_center_size(center, vec2(7.0, 5.0));
            painter.rect_filled(stud.translate(vec2(0.0, 2.0)), 1.0, stud_side);
            painter.rect_filled(stud, 1.0, stud_top);
        }
    }
}

pub fn show(ui:&mut egui::Ui,game:&mut Game) {
    let dark = game.dark_mode;
    let view = ui.available_rect_before_wrap();
    draw_floor(ui, view, dark);
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
                let depth = vec2(3.0, 8.0);
                let shadow = face.map(|point| point + depth + vec2(2.0, 2.0));
                p.add(Shape::convex_polygon(shadow.to_vec(), C::from_black_alpha(38), Stroke::NONE));
                p.add(Shape::convex_polygon(
                    vec![face[2], face[3], face[3] + depth, face[2] + depth],
                    crate::theme_color(dark, [125,116,85], [18,18,39]),
                    Stroke::NONE,
                ));
                p.add(Shape::convex_polygon(
                    vec![face[1], face[2], face[2] + depth, face[1] + depth],
                    crate::theme_color(dark, [150,132,90], [49,43,94]),
                    Stroke::NONE,
                ));
                let fill = if !unlocked {
                    crate::theme_color(dark, [165,170,150], [40,39,61])
                } else if game.switches[index] {
                    crate::theme_color(dark, [140,191,105], [29,110,105])
                } else {
                    crate::theme_color(dark, [231,190,119], [61,48,110])
                };
                let border = if selected { C::from_rgb(230,148,36) } else {
                    crate::theme_color(dark, [104,119,76], [124,111,183])
                };
                p.add(Shape::convex_polygon(face.to_vec(), fill, Stroke::new(if selected { 3.0 } else { 1.0 }, border)));
                let highlight = C::from_white_alpha(if dark { 52 } else { 92 });
                for (y, width) in [(0.0, 13.0), (3.0, 7.0), (6.0, 3.0)] {
                    p.rect_filled(Rect::from_min_size(face[0] + vec2(5.0, 5.0 + y), vec2(width, 3.0)), 0.0, highlight);
                }
                let max_hp = Game::switch_max_hp(index);
                let hp = game.switch_hp(index);
                crate::surface_gui::paint(p, face, &[
                    format!("#{}", index + 1), format!("{hp}/{max_hp}"), "НАГР +1".into(),
                    format!("УР {}", Game::switch_required_level(index)),
                    format!("Р {}", Game::switch_rarity(index)),
                ], crate::theme_color(dark, [45,62,38], [247,243,255]), hp as f32 / max_hp as f32);
                if response.clicked() {game.tap_switch(index);}
                if selected&&!game.equipped_pets.is_empty() {
                    crate::pet_ui::attack_scene(p,Rect::from_center_size(r.center(),r.size()),r.center(),side*0.62,game);
                }
            }
        });}
    });
}
