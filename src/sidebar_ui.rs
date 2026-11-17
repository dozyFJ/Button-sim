use egui::{Color32 as C,RichText as T,Rect,Sense,Stroke,Align2,FontId,vec2,pos2};
use crate::{game::{Game,Tab},gui_state::{GuiState,Panel,Modal},amount,nav_icon};
const INK:C=C::from_rgb(39,54,41);
const GREEN:C=C::from_rgb(93,161,71);
pub fn width(screen_width:f32)->f32 {if screen_width<720.0 {104.0} else {116.0}}
fn square(ui:&mut egui::Ui,label:&str,icon:usize,active:bool)->bool {
    let side=ui.available_width().min(88.0);
    let (r,response)=ui.allocate_exact_size(vec2(side,side),Sense::click());
    let p=ui.painter();let color=if active {C::WHITE} else {INK};
    p.rect_filled(r.translate(vec2(0.0,2.0)),0.0,C::from_rgb(153,160,121));
    p.rect_filled(r,0.0,if active {GREEN} else {C::from_rgb(232,231,206)});
    p.rect_stroke(r,0.0,Stroke::new(1.0_f32,C::from_rgb(143,159,112)));
    if icon<6 {nav_icon(p,r.center()-vec2(0.0,8.0),icon,color);} else if icon==6 {
        p.circle_stroke(r.center()-vec2(0.0,8.0),12.0,Stroke::new(2.0_f32,color));
        p.line_segment([r.center()+vec2(-17.0,-8.0),r.center()+vec2(17.0,-8.0)],Stroke::new(2.0_f32,color));
        p.line_segment([r.center()+vec2(0.0,-25.0),r.center()+vec2(0.0,9.0)],Stroke::new(2.0_f32,color));
    } else {for x in [-5.0,5.0] {p.rect_filled(Rect::from_center_size(r.center()+vec2(x,-8.0),vec2(5.0,20.0)),0.0,color);}}
    p.text(pos2(r.center().x,r.bottom()-14.0),Align2::CENTER_CENTER,label,FontId::proportional(10.0),color);
    response.clicked()
}
fn balance(ui:&mut egui::Ui,title:&str,value:&str,money:bool) {
    let (r,_)=ui.allocate_exact_size(vec2(ui.available_width(),49.0),Sense::hover());let p=ui.painter();
    p.rect_filled(r,0.0,if money {C::from_rgb(229,235,197)} else {C::from_rgb(246,221,174)});
    p.rect_stroke(r,0.0,Stroke::new(1.0_f32,C::from_rgb(164,176,128)));
    let c=r.left_center()+vec2(16.0,2.0);
    if money {p.rect_filled(Rect::from_center_size(c,vec2(22.0,14.0)),0.0,GREEN);p.circle_filled(c,3.0,C::from_rgb(244,212,99));}
    else {p.circle_stroke(c,8.0,Stroke::new(2.0_f32,C::from_rgb(155,102,40)));p.line_segment([c+vec2(8.0,-8.0),c+vec2(8.0,0.0)],Stroke::new(2.0_f32,INK));}
    p.text(pos2(r.left()+31.0,r.top()+12.0),Align2::LEFT_CENTER,title,FontId::proportional(8.0),INK);
    p.text(pos2(r.left()+31.0,r.top()+31.0),Align2::LEFT_CENTER,value,FontId::proportional(16.0),INK);
}
pub fn draw(ctx:&egui::Context,game:&mut Game,state:&mut GuiState) {
    egui::SidePanel::left("main_sidebar").exact_width(width(ctx.screen_rect().width())).resizable(false)
        .frame(egui::Frame::none().fill(C::from_rgb(246,238,211)).inner_margin(egui::Margin::same(8.0)))
        .show(ctx,|ui| {
            if ui.add_enabled(state.sidebar_scroll>0.0,egui::Button::new("↑").min_size(vec2(ui.available_width(),24.0))).clicked() {state.sidebar_scroll=(state.sidebar_scroll-200.0).max(0.0);}
            let height=(ui.available_height()-32.0).max(60.0);
            let out=egui::ScrollArea::vertical().id_salt("main_sidebar_scroll").max_height(height).auto_shrink([false,false]).scroll_offset(vec2(0.0,state.sidebar_scroll))
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui,|ui| {
                let (r,response)=ui.allocate_exact_size(vec2(ui.available_width(),76.0),Sense::click());let p=ui.painter();
                p.rect_filled(r,0.0,C::from_rgb(219,230,203));
                let avatar=Rect::from_center_size(r.center()-vec2(0.0,14.0),vec2(34.0,34.0));
                p.rect_filled(avatar,0.0,C::from_rgb(115,174,196));p.rect_stroke(avatar,0.0,Stroke::new(2.0_f32,INK));
                p.rect_filled(Rect::from_center_size(avatar.center()-vec2(0.0,6.0),vec2(12.0,12.0)),0.0,C::from_rgb(255,226,175));
                p.rect_filled(Rect::from_center_size(avatar.center()+vec2(0.0,9.0),vec2(22.0,12.0)),0.0,C::from_rgb(66,112,153));
                p.text(pos2(r.center().x,r.bottom()-23.0),Align2::CENTER_CENTER,"МОЙ ПРОФИЛЬ",FontId::proportional(9.0),INK);
                p.text(pos2(r.center().x,r.bottom()-9.0),Align2::CENTER_CENTER,game.rank(),FontId::proportional(9.0),INK);
                if response.clicked()&&!game.paused&&game.hatch.is_none() {if state.panel==Panel::Profile {state.close();} else {state.profile(false);}}
                ui.add_space(4.0);balance(ui,"ДЕНЬГИ",&amount(game.coins),true);ui.add_space(3.0);balance(ui,"РЕБЁРТЫ",&game.rebirths.to_string(),false);
                ui.label(T::new(format!("Уровень {}",game.level())).size(11.0).strong());
                ui.add(egui::ProgressBar::new(game.level_progress()).desired_width(ui.available_width()).fill(GREEN).text(format!("{}/100 XP",game.taps%100)));
                ui.separator();
                ui.add_enabled_ui(!game.paused&&game.hatch.is_none(),|ui| {
                    ui.vertical_centered(|ui| {
                        if square(ui,"Главная",0,game.tab==Tab::Home&&!state.busy()) {state.close();game.tab=Tab::Home;}
                        if square(ui,"Улучшения",2,state.modal==Modal::Upgrades) {state.modal(Modal::Upgrades);}
                        if square(ui,"Питомцы",5,state.panel==Panel::Pets) {if state.panel==Panel::Pets {state.close();} else {state.pets();}}
                        if square(ui,"Статистика",3,state.panel==Panel::Profile&&state.profile_stats) {state.profile(true);}
                        if square(ui,"Ребёрты",4,state.modal==Modal::Rebirths) {state.modal(Modal::Rebirths);}
                        if square(ui,"Капсулы",1,game.tab==Tab::Capsules&&!state.busy()) {state.close();game.tab=Tab::Capsules;}
                        if square(ui,if game.auto_attack {"Автоатака ВКЛ"} else {"Автоатака"},6,game.auto_attack) {
                            state.close();game.pets_open=false;game.interface_open=false;
                            if game.equipped_pets.is_empty() {state.pets();} else {game.toggle_auto_attack();}
                        }
                    });
                });
                ui.vertical_centered(|ui| {if square(ui,if game.paused {"Продолжить"} else {"Пауза"},7,game.paused) {game.paused=!game.paused;}});
            });
            state.sidebar_scroll=out.state.offset.y;state.sidebar_max=(out.content_size.y-out.inner_rect.height()).max(0.0);
            if ui.add_enabled(state.sidebar_scroll<state.sidebar_max,egui::Button::new("↓").min_size(vec2(ui.available_width(),24.0))).clicked() {state.sidebar_scroll=(state.sidebar_scroll+200.0).min(state.sidebar_max);}
        });
}
