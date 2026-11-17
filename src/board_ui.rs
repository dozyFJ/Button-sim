//! One vertically scrolling field; metadata is part of the animated switch face.
use egui::{Color32 as C,Rect,Stroke,Sense,Shape,pos2,vec2};
use crate::{amount, game::{Game, Tab}};
fn draw_floor(ui: &egui::Ui, view: Rect, dark: bool) {
    let painter = ui.painter().with_clip_rect(view);
    let base = crate::theme_color(dark, [119,159,85], [16,29,53]);
    let grid = crate::theme_color(dark, [102,140,75], [25,44,76]);
    let stud_top = crate::theme_color(dark, [143,181,108], [42,66,111]);
    let stud_side = crate::theme_color(dark, [92,128,64], [9,19,36]);
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
    let rebirths = game.tab == Tab::Rebirths;
    draw_floor(ui, ui.available_rect_before_wrap(), dark);
    if rebirths {
        egui::Frame::none().fill(ui.visuals().panel_fill).inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("Плитки ребёрта").strong());
            ui.label(format!("Ребёрты: {}  ·  Множитель ×{}", game.rebirths, amount(game.multiplier as f64)));
        });
    }
    let view = ui.available_rect_before_wrap();
    let columns=(view.width()/155.0).floor().clamp(2.0,6.0) as usize;
    let gap=10.0;let side=(view.width()-16.0-(columns-1) as f32*gap)/columns as f32;
    let height = side * 0.86;
    let count = if rebirths { (game.rebirths as usize + 12).max(32) } else { game.visible_switches() };
    let rows = count.div_ceil(columns);
    egui::ScrollArea::vertical().id_salt(("vertical_switch_field", rebirths)).auto_shrink([false,false]).max_height((view.height() - 65.0).max(0.0))
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).show_rows(ui,height+gap,rows,|ui,range| {
        for row in range {ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x=gap;
            for col in 0..columns {
                let index=row*columns+col;if index>=count {break;}
                let (r,response)=ui.allocate_exact_size(vec2(side,height),Sense::click());let p=ui.painter();
                if !ui.is_rect_visible(r) {continue;}
                let completed = rebirths && index < game.rebirths as usize;
                let selected = if rebirths { index == game.rebirths as usize } else { index == game.selected_switch };
                let unlocked = if rebirths { completed || selected } else { game.switch_unlocked(index) };
                let active = if rebirths { completed } else { game.switches[index] };
                let t = ui.ctx().animate_bool_with_time(ui.id().with(("switch", index)), active, 0.16);
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
                    crate::theme_color(dark, [150,132,90], [31,51,90]),
                    Stroke::NONE,
                ));
                let fill = if !unlocked {
                    crate::theme_color(dark, [157,160,163], [53,55,63])
                } else if active {
                    crate::theme_color(dark, [140,191,105], [63,92,30])
                } else {
                    crate::theme_color(dark, [231,190,119], [36,63,115])
                };
                let border = if !unlocked {
                    crate::theme_color(dark, [105,108,112], [112,115,123])
                } else if selected { crate::theme_color(dark, [230,148,36], [172,225,78]) } else {
                    crate::theme_color(dark, [104,119,76], [104,138,199])
                };
                p.add(Shape::convex_polygon(face.to_vec(), fill, Stroke::new(if selected { 3.0 } else { 1.0 }, border)));
                if dark && unlocked {
                    let ceiling = face[0].lerp(face[3], 0.40);
                    let right_edge = face[1].lerp(face[2], 0.40);
                    p.add(Shape::convex_polygon(vec![face[0], face[1], right_edge, ceiling], C::from_white_alpha(19), Stroke::NONE));
                    let rim = C::from_rgba_unmultiplied(181,203,255,101);
                    p.line_segment([face[0], face[1]], Stroke::new(1.0_f32, rim));
                }
                let highlight = C::from_white_alpha(if dark { 52 } else { 92 });
                for (y, width) in [(0.0, 13.0), (3.0, 7.0), (6.0, 3.0)] {
                    p.rect_filled(Rect::from_min_size(face[0] + vec2(5.0, 5.0 + y), vec2(width, 3.0)), 0.0, highlight);
                }
                let ink = if unlocked { crate::theme_color(dark, [45,62,38], [241,247,255]) } else { crate::theme_color(dark, [38,41,45], [231,232,238]) };
                if rebirths {
                    let price = Game::rebirth_price_at(index as u32);
                    let max_hp = Game::rebirth_tile_max_hp(index as u32);
                    let hp = if selected { game.rebirth_tile_hp() } else { max_hp };
                    let status = if completed { "ГОТОВО".into() } else if selected && hp == 0 {
                        "КУПИ РЕБЕРТ".into()
                    } else if selected { format!("{hp}/{max_hp}") } else { "ЗАКРЫТО".into() };
                    crate::surface_gui::paint(p, face, &[
                        format!("РЕБЕРТ {}", index + 1), status,
                        format!("ЦЕНА {}", amount(price)), "ДОХОД X2".into(),
                    ], ink, selected.then_some(hp as f32 / max_hp as f32), !unlocked);
                    if selected && response.clicked() { game.tap_rebirth_tile(); }
                } else {
                    let max_hp = Game::switch_max_hp(index);
                    let hp = game.switch_hp(index);
                    let (status, reward, progress) = if hp == 0 {
                        ("ГОТОВО".into(), format!("КУПИ {}", amount(Game::switch_price(index))), 0.0)
                    } else {
                        (format!("{hp}/{max_hp}"), "НАГР +1".into(), hp as f32 / max_hp as f32)
                    };
                    crate::surface_gui::paint(p, face, &[
                        format!("#{}", index + 1), status, reward,
                        format!("УР {}", Game::switch_required_level(index)),
                        format!("РБ {} · {}", Game::switch_required_rebirths(index), Game::switch_rarity(index)),
                    ], ink, unlocked.then_some(progress), !unlocked);
                    if unlocked && response.clicked() { game.tap_switch(index); }
                }
                if !unlocked {
                    let center = face[1].lerp(face[2], 0.20) - vec2(17.0, 0.0);
                    let lock = crate::theme_color(dark, [53,57,64], [230,232,239]);
                    p.circle_filled(center, 11.0, crate::theme_color(dark, [212,215,218], [34,36,42]));
                    p.rect_stroke(Rect::from_center_size(center - vec2(0.0, 3.0), vec2(7.0, 9.0)), 3.0, Stroke::new(1.5, lock));
                    p.rect_filled(Rect::from_center_size(center + vec2(0.0, 2.5), vec2(11.0, 8.0)), 2.0, lock);
                }
                if !rebirths {
                    if let Some((_, started)) = game.last_reward.filter(|(reward, _)| *reward == index) {
                        let t = ((game.played_seconds - started) / 0.65) as f32;
                        if (0.0..1.0).contains(&t) {
                            let position = r.center() - vec2(0.0, 12.0 + t * 22.0);
                            let font = egui::FontId::proportional(15.0);
                            p.text(position + vec2(1.0, 1.0), egui::Align2::CENTER_CENTER,
                                "+1 к множителю", font.clone(), C::BLACK.gamma_multiply(1.0 - t));
                            p.text(position, egui::Align2::CENTER_CENTER,
                                "+1 к множителю", font, C::WHITE.gamma_multiply(1.0 - t));
                        }
                    }
                }
                let target_alive = if rebirths { game.rebirth_tile_hp() > 0 } else { game.switch_hp(index) > 0 };
                if unlocked && selected && target_alive && !game.equipped_pets.is_empty() {
                    crate::pet_ui::attack_scene(p,Rect::from_center_size(r.center(),r.size()),r.center(),side*0.62,game);
                }
            }
        });}
    });
}
