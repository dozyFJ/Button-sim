use egui::{Align2, Color32 as C, Rect, RichText as T, Sense, Stroke, pos2, vec2};
use crate::{game::{Game, Tab}, gui_state::{GuiState, Panel, PetAction}, panels_ui, pet_render};

const GREEN: C = C::from_rgb(93, 161, 71);

pub fn draw(ctx: &egui::Context, workspace: Rect, game: &mut Game, state: &mut GuiState) {
    if state.panel != Panel::Pets { return; }
    egui::Window::new("pets_inventory").title_bar(false).fixed_rect(workspace.shrink(8.0))
        .resizable(false).collapsible(false).frame(panels_ui::frame(ctx)).show(ctx, |ui| {
        if panels_ui::heading(ui, "Питомцы") {
            state.close();
            return;
        }
        if state.selected_pet.is_none_or(|i| i >= game.pets.len()) {
            state.selected_pet = (!game.pets.is_empty()).then_some(0);
        }
        if let Some(index) = state.selected_pet {
            details(ui, game, state, index);
            ui.separator();
        }
        filters(ui, game, state);
        if state.pet_multi_delete {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Выбрано: {}", state.pet_marked.len()));
                if ui.button("Отмена").clicked() {
                    state.pet_multi_delete = false;
                    state.pet_marked.clear();
                }
                if ui.add_enabled(!state.pet_marked.is_empty() && state.pending.is_none() && !game.paused,
                    egui::Button::new("Удалить выбранных")).clicked()
                {
                    state.pending = Some(PetAction::DeleteMany);
                }
            });
        }
        grid(ui, game, state);
    });
    confirm(ctx, workspace, game, state);
}

fn details(ui: &mut egui::Ui, game: &mut Game, state: &mut GuiState, index: usize) {
    let pet = game.pets[index];
    let available = ui.available_rect_before_wrap();
    let width = available.width();
    let height = (available.height() * 0.42).clamp(125.0, 170.0);
    let (card, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let split = card.left() + (width * 0.37).min(240.0);
    let left = Rect::from_min_max(card.min, pos2(split, card.bottom()));
    let right = Rect::from_min_max(pos2(split + 9.0, card.top()), card.max);

    ui.allocate_ui_at_rect(left, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(left));
        ui.vertical_centered(|ui| {
            ui.label(T::new(pet.name()).size(17.0).strong());
            let (viewport, _) = ui.allocate_exact_size(vec2((left.width() - 4.0).max(0.0), height - 53.0), Sense::hover());
            let dark = ui.visuals().dark_mode;
            let painter = ui.painter().with_clip_rect(viewport);
            painter.rect_filled(viewport, 0.0, crate::theme_color(dark, [195,220,223], [38,49,79]));
            let floor = Rect::from_min_max(pos2(viewport.left(), viewport.top() + viewport.height() * 0.64), viewport.max);
            painter.rect_filled(floor, 0.0, crate::theme_color(dark, [154,177,123], [52,63,94]));
            painter.rect_filled(Rect::from_center_size(viewport.center() + vec2(0.0, viewport.height() * 0.29),
                vec2(viewport.width() * 0.48, 9.0)), 5.0, C::from_black_alpha(65));
            pet_render::pet(&painter, viewport.shrink(5.0), pet, game.played_seconds as f32);
            painter.rect_stroke(viewport, 0.0, Stroke::new(1.0, crate::theme_color(dark, [103,122,87], [133,117,198])));
            ui.label(T::new(format!("{} · тир {}", pet.rarity(), pet.evolution_tier())).size(11.0));
        });
    });

    ui.allocate_ui_at_rect(right, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(right));
        ui.set_width(right.width());
        egui::ScrollArea::horizontal().id_salt("pet_boosts").max_width(right.width()).show(ui, |ui| {
            ui.horizontal(|ui| {
                for label in [format!("Доход ×{:.1}", pet.bonus()), format!("Урон {}", pet.damage())] {
                    egui::Frame::none().fill(ui.visuals().selection.bg_fill).rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(7.0, 4.0)).show(ui, |ui| { ui.label(T::new(label).strong()); });
                }
            });
        });
        ui.add_space(4.0);
        ui.add_enabled_ui(!game.paused && state.pending.is_none(), |ui| {
            egui::ScrollArea::horizontal().id_salt("pet_actions").max_width(right.width()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let equipped = game.equipped_pets.contains(&index);
                    if ui.button(if equipped { "Снять" } else { "Надеть" }).clicked() {
                        if equipped { game.unequip_pet(index); } else { game.equip_pet(index); }
                    }
                    if ui.add_enabled(!state.pet_multi_delete, egui::Button::new("Удалить")).clicked() { state.pending = Some(PetAction::Delete(index)); }
                    if ui.add_enabled(!state.pet_multi_delete && game.craft_materials(index).is_some(), egui::Button::new("Крафт"))
                        .on_hover_text("Нужны три одинаковых питомца").clicked()
                    { state.pending = Some(PetAction::Craft(index)); }
                    if ui.button("Надеть лучшего").clicked() { game.equip_best_pet(); }
                    if ui.add_enabled(!game.equipped_pets.is_empty(), egui::Button::new("Снять всех")).clicked() {
                        game.unequip_all_pets();
                    }
                    if ui.selectable_label(state.pet_multi_delete, "Мультиудаление").clicked() {
                        state.pet_multi_delete = !state.pet_multi_delete;
                        state.pet_marked.clear();
                    }
                });
            });
        });
        ui.add_space(6.0);
        let xp = game.pet_xp(index);
        ui.add(egui::ProgressBar::new((xp % 100) as f32 / 100.0).desired_width(right.width())
            .text(format!("Ур. {} · {} / 100 XP", xp / 100 + 1, xp % 100)));
    });
}

fn filters(ui: &mut egui::Ui, game: &Game, state: &mut GuiState) {
    ui.horizontal_wrapped(|ui| {
        ui.label(T::new(format!("Надето {}/3", game.equipped_pets.len())).strong());
        ui.label(format!("Инвентарь {}/1000", game.pets.len()));
        ui.add(egui::TextEdit::singleline(&mut state.search).hint_text("Имя или редкость").desired_width(130.0));
    });
    ui.horizontal_wrapped(|ui| {
        for (i, title) in ["Инвентарь", "Надеты", "Крафт"].into_iter().enumerate() {
            ui.selectable_value(&mut state.pet_tab, i, title);
        }
        let rarities = ["Все", "Обычные", "Редкие", "Эпические", "Легендарные"];
        egui::ComboBox::from_id_salt("pet_rarity_filter").selected_text(rarities[state.rarity]).show_ui(ui, |ui| {
            for (i, title) in rarities.into_iter().enumerate() { ui.selectable_value(&mut state.rarity, i, title); }
        });
    });
    ui.separator();
}

fn grid(ui: &mut egui::Ui, game: &mut Game, state: &mut GuiState) {
    let query = state.search.trim().to_lowercase();
    let filtered: Vec<_> = game.pets.iter().enumerate().filter_map(|(index, &pet)| {
        let rank = usize::from(pet.evolution_tier().min(4));
        let matches = format!("{} {}", pet.name(), pet.rarity()).to_lowercase().contains(&query)
            && (state.rarity == 0 || state.rarity == rank)
            && (state.pet_tab != 1 || game.equipped_pets.contains(&index))
            && (state.pet_tab != 2 || Game::craft_result(pet).is_some());
        matches.then_some(index)
    }).collect();
    if filtered.is_empty() {
        ui.label(if game.pets.is_empty() { "Нет питомцев. Открой капсулу." } else { "По запросу ничего не найдено." });
        if game.pets.is_empty() && ui.button("К капсулам").clicked() { state.close(); game.tab = Tab::Capsules; }
        return;
    }
    let width = ui.available_width() - 10.0;
    let columns = ((width / 90.0).floor() as usize).max(2);
    let tile = (width - (columns - 1) as f32 * 4.0) / columns as f32;
    egui::ScrollArea::vertical().id_salt("pet_slots").auto_shrink([false, false]).show_rows(ui, 94.0,
        filtered.len().div_ceil(columns), |ui, rows| {
        for row in rows {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for &index in filtered.iter().skip(row * columns).take(columns) {
                    let pet = game.pets[index];
                    let (rect, response) = ui.allocate_exact_size(vec2(tile, 94.0), Sense::click());
                    let p = ui.painter();
                    let marked = state.pet_marked.contains(&index);
                    let selected = if state.pet_multi_delete { marked } else { state.selected_pet == Some(index) };
                    let dark = ui.visuals().dark_mode;
                    p.rect_filled(rect, 5.0, crate::theme_color(dark, [230,235,207], [43,39,72]));
                    p.rect_stroke(rect, 5.0, Stroke::new(if selected { 2.0 } else { 1.0 },
                        if selected { C::from_rgb(221,166,52) } else { crate::theme_color(dark, [153,173,117], [105,91,162]) }));
                    pet_render::pet(p, Rect::from_min_max(rect.min + vec2(5.0,3.0), rect.max - vec2(5.0,30.0)), pet, game.played_seconds as f32);
                    let ink = ui.visuals().text_color();
                    let number = if marked { format!("✓ {}", index + 1) } else { (index + 1).to_string() };
                    p.text(rect.left_top() + vec2(5.0,5.0), Align2::LEFT_TOP, number, egui::FontId::proportional(10.0), ink);
                    if game.equipped_pets.contains(&index) { p.circle_filled(rect.right_top() + vec2(-8.0,8.0), 4.0, GREEN); }
                    p.text(rect.center_bottom() - vec2(0.0,21.0), Align2::CENTER_CENTER, pet.name(), egui::FontId::proportional(10.0), ink);
                    p.text(rect.center_bottom() - vec2(0.0,8.0), Align2::CENTER_CENTER, format!("×{:.1}", pet.bonus()), egui::FontId::proportional(9.0), ink);
                    if response.clicked() && state.pending.is_none() {
                        if state.pet_multi_delete {
                            if let Some(position) = state.pet_marked.iter().position(|&entry| entry == index) {
                                state.pet_marked.swap_remove(position);
                            } else { state.pet_marked.push(index); }
                        } else { state.selected_pet = Some(index); }
                    }
                }
            });
        }
    });
}

fn confirm(ctx: &egui::Context, workspace: Rect, game: &mut Game, state: &mut GuiState) {
    let Some(action) = state.pending else { return; };
    let message = match action {
        PetAction::Delete(i) => game.pets.get(i).map(|pet| format!("Удалить питомца «{}»?", pet.name())),
        PetAction::Craft(i) => game.pets.get(i).and_then(|pet| Game::craft_result(*pet)
            .map(|next| format!("Обменять 3 × {} на {} (×{:.1})?", pet.name(), next.name(), next.bonus()))),
        PetAction::DeleteMany => Some(format!("Удалить выбранных питомцев ({})?", state.pet_marked.len())),
    };
    let Some(message) = message else { state.pending = None; return; };
    let rect = Rect::from_center_size(workspace.center(), vec2((workspace.width() - 32.0).min(390.0), 128.0));
    egui::Window::new("pet_confirmation").title_bar(false).fixed_rect(rect)
        .resizable(false).collapsible(false).frame(panels_ui::frame(ctx)).show(ctx, |ui| {
        ui.heading(message);
        if matches!(action, PetAction::Delete(_) | PetAction::DeleteMany) { ui.label("Действие нельзя отменить."); }
        ui.horizontal(|ui| {
            if ui.button("Отмена").clicked() { state.pending = None; }
            if ui.add_enabled(!game.paused, egui::Button::new("Подтвердить")).clicked() {
                match action {
                    PetAction::Delete(index) => { if game.delete_pet(index) { state.selected_pet = None; } }
                    PetAction::Craft(index) => { state.selected_pet = game.craft_pet(index); }
                    PetAction::DeleteMany => {
                        if game.delete_pets(state.pet_marked.clone()) {
                            state.pet_marked.clear();
                            state.pet_multi_delete = false;
                            state.selected_pet = None;
                        }
                    }
                }
                state.pending = None;
            }
        });
    });
}
