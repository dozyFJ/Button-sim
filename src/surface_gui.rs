//! Tiny pixel glyphs drawn as mesh geometry directly on a moving switch face.
use egui::{Color32,Mesh,Painter,Pos2,Shape};
fn glyph(c:char)->[u8;7] {match c {
'X'=>[17,17,10,4,10,17,17],'K'=>[17,18,20,24,20,18,17],'M'=>[17,27,21,21,17,17,17],'B'=>[30,17,17,30,17,17,30],
'З'=>[30,1,1,14,1,1,30],'В'=>[30,17,17,30,17,17,30],'Х'=>[17,17,10,4,10,17,17],'·'=>[0,0,0,4,0,0,0],
'0'=>[14,17,19,21,25,17,14],'1'=>[4,12,4,4,4,4,14],'2'=>[14,17,1,2,4,8,31],'3'=>[30,1,1,14,1,1,30],'4'=>[2,6,10,18,31,2,2],
'5'=>[31,16,16,30,1,1,30],'6'=>[14,16,16,30,17,17,14],'7'=>[31,1,2,4,8,8,8],'8'=>[14,17,17,14,17,17,14],'9'=>[14,17,17,15,1,1,14],
'/'=>[1,2,2,4,8,8,16],'#'=>[10,10,31,10,31,10,10],'+'=>[0,4,4,31,4,4,0],'.'=>[0,0,0,0,0,6,6],
'Ц'=>[18,18,18,18,18,31,1],'Е'=>[31,16,16,30,16,16,31],'Н'=>[17,17,17,31,17,17,17],'А'=>[14,17,17,31,17,17,17],
'Г'=>[31,16,16,16,16,16,16],'Р'=>[30,17,17,30,16,16,16],'У'=>[17,17,17,15,1,17,14],
'С'=>[15,16,16,16,16,16,15],'Т'=>[31,4,4,4,4,4,4],'О'=>[14,17,17,17,17,17,14],'Б'=>[31,16,16,30,17,17,30],
'Ы'=>[17,17,17,29,21,21,29],'Ч'=>[17,17,17,15,1,1,1],'Д'=>[6,10,10,10,10,31,17],'К'=>[17,18,20,24,20,18,17],
'Э'=>[30,1,1,15,1,1,30],'Я'=>[15,17,17,15,5,9,17],'И'=>[17,19,21,25,17,17,17],'П'=>[31,17,17,17,17,17,17],'Л'=>[7,9,9,9,9,17,17],_=>[0;7]}}
pub fn paint(p:&Painter,face:[Pos2;4],lines:&[String],color:Color32,health:Option<f32>) {
    let warp=|x:f32,y:f32| {let u=x/96.0;let v=y/80.0;let a=face[0].lerp(face[1],u);let b=face[3].lerp(face[2],u);a.lerp(b,v)};
    let mut mesh=Mesh::default();
    for (row,line) in lines.iter().enumerate() {
        let width = line.chars().count() as f32 * 6.0;
        let scale = (88.0 / width.max(1.0)).min(1.0);
        let start = (96.0 - width * scale) * 0.5;
        for (col,ch) in line.chars().enumerate() {for (y,bits) in glyph(ch.to_uppercase().next().unwrap_or(ch)).iter().enumerate() {for x in 0..5 {
            if bits&(1<<(4-x))==0 {continue;}
            let px=start+(col as f32*6.0+x as f32)*scale;let py=4.0+row as f32*15.0+y as f32;
            let base=mesh.vertices.len() as u32;
            for pos in [warp(px,py),warp(px+scale,py),warp(px+scale,py+1.0),warp(px,py+1.0)] {
                mesh.vertices.push(egui::epaint::Vertex{pos,uv:egui::epaint::WHITE_UV,color});
            }
            mesh.indices.extend_from_slice(&[base,base+1,base+2,base,base+2,base+3]);
        }}}
    }
    p.add(Shape::mesh(mesh));
    let Some(health) = health else { return; };
    let bar = |end: f32| vec![warp(17.0,73.0), warp(end,73.0), warp(end,77.0), warp(17.0,77.0)];
    p.add(Shape::convex_polygon(bar(79.0), Color32::from_rgb(72,47,47), egui::Stroke::NONE));
    if health > 0.0 {
        p.add(Shape::convex_polygon(bar(17.0 + 62.0 * health), Color32::from_rgb(241,100,100), egui::Stroke::NONE));
    }
}
