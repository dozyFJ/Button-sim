//! Orthographic map camera: a tilted top-down view with no vanishing point.
#[derive(Clone, Copy, Debug)]
pub struct MapCamera {pub x:f32,pub z:f32,pub zoom:f32}
const X:f32=std::f32::consts::FRAC_1_SQRT_2;
const Z:f32=0.45;
impl MapCamera {
    pub fn project(&self,point:[f32;3],center:[f32;2])->[f32;2] {
        let x=point[0]-self.x;let z=point[2]-self.z;
        [center[0]+(x-z)*self.zoom*X,center[1]+(x+z)*self.zoom*Z-point[1]*self.zoom*0.85]
    }
    pub fn ground_at(&self,screen:[f32;2],center:[f32;2])->[f32;2] {
        let u=(screen[0]-center[0])/(self.zoom*X);let v=(screen[1]-center[1])/(self.zoom*Z);
        [self.x+(u+v)*0.5,self.z+(v-u)*0.5]
    }
    pub fn pan(&mut self,delta:[f32;2]) {
        let u=delta[0]/(self.zoom*X);let v=delta[1]/(self.zoom*Z);
        self.x-=(u+v)*0.5;self.z-=(v-u)*0.5;
    }
    pub fn zoom_at(&mut self,factor:f32,screen:[f32;2],center:[f32;2]) {
        if !factor.is_finite()||factor<=0.0 {return;}
        let before=self.ground_at(screen,center);self.zoom=(self.zoom*factor).clamp(24.0,160.0);
        let after=self.ground_at(screen,center);self.x+=before[0]-after[0];self.z+=before[1]-after[1];
    }
}
pub fn quad_contains(point:[f32;2],quad:[[f32;2];4])->bool {
    let mut sign=0.0_f32;
    for i in 0..4 {
        let a=quad[i];let b=quad[(i+1)%4];
        let cross=(b[0]-a[0])*(point[1]-a[1])-(b[1]-a[1])*(point[0]-a[0]);
        if cross.abs()<0.001 {continue;}
        if sign!=0.0&&cross.signum()!=sign {return false;}sign=cross.signum();
    }
    sign!=0.0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ground_picking_matches_projection() {
        let c=MapCamera{x:3.5,z:6.0,zoom:73.0};let center=[400.0,250.0];
        for x in 0..32 {for z in 0..32 {
            let p=c.project([x as f32,0.0,z as f32],center);let w=c.ground_at(p,center);
            assert!((w[0]-x as f32).abs()<0.0001&&(w[1]-z as f32).abs()<0.0001);
        }}
    }
    #[test] fn dragging_moves_ground_with_pointer() {
        let mut c=MapCamera{x:3.5,z:3.5,zoom:60.0};let center=[400.0,250.0];
        let before=c.project([2.0,0.0,1.0],center);c.pan([45.0,-30.0]);let after=c.project([2.0,0.0,1.0],center);
        assert!((after[0]-before[0]-45.0).abs()<0.001);assert!((after[1]-before[1]+30.0).abs()<0.001);
    }
    #[test] fn zoom_keeps_the_point_under_the_pointer() {
        let mut c=MapCamera{x:3.5,z:3.5,zoom:60.0};let center=[400.0,250.0];let pointer=[330.0,180.0];
        let before=c.ground_at(pointer,center);c.zoom_at(1.7,pointer,center);let after=c.ground_at(pointer,center);
        assert!((before[0]-after[0]).abs()<0.001&&(before[1]-after[1]).abs()<0.001);
    }
    #[test] fn tile_hit_test_rejects_empty_corners_of_bounding_box() {
        let q=[[0.0,-10.0],[20.0,0.0],[0.0,10.0],[-20.0,0.0]];
        assert!(quad_contains([0.0,0.0],q));assert!(quad_contains([10.0,0.0],q));
        assert!(!quad_contains([19.0,9.0],q));assert!(!quad_contains([21.0,0.0],q));
    }
}
