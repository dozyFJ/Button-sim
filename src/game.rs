//! Clicker economy, capsule drops and persistent player collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Home, Capsules, Upgrades, Statistics, Rebirths }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetKind { Cat, Dog, Bunny, Fox, Dragon, Unicorn }
impl PetKind {
    pub fn name(self) -> &'static str { match self { Self::Cat=>"Котик", Self::Dog=>"Щенок", Self::Bunny=>"Зайчик", Self::Fox=>"Лисёнок", Self::Dragon=>"Дракон", Self::Unicorn=>"Единорог" } }
    pub fn rarity(self) -> &'static str { match self { Self::Cat|Self::Dog=>"Обычный", Self::Bunny=>"Редкий", Self::Fox=>"Эпический", _=>"Легендарный" } }
    pub fn bonus(self) -> f64 { match self { Self::Cat=>1.2, Self::Dog=>1.3, Self::Bunny=>1.6, Self::Fox=>2.0, Self::Dragon=>3.0, Self::Unicorn=>4.0 } }
    pub fn color(self) -> [u8;3] { match self { Self::Cat=>[124,178,255], Self::Dog=>[227,176,117], Self::Bunny=>[245,186,226], Self::Fox=>[255,139,69], Self::Dragon=>[135,102,242], Self::Unicorn=>[132,246,226] } }
    fn from_id(id: usize) -> Option<Self> { [Self::Cat,Self::Dog,Self::Bunny,Self::Fox,Self::Dragon,Self::Unicorn].get(id).copied() }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capsule { Meadow, Neon }
impl Capsule {
    pub fn name(self)-> &'static str { match self { Self::Meadow=>"Луговая капсула",Self::Neon=>"Неоновая капсула" } }
    pub fn price(self)->f64 { match self { Self::Meadow=>250.0,Self::Neon=>2500.0 } }
    pub fn drops(self)-> &'static [(PetKind,u8)] { match self {
        Self::Meadow=>&[(PetKind::Cat,45),(PetKind::Dog,35),(PetKind::Bunny,17),(PetKind::Fox,3)],
        Self::Neon=>&[(PetKind::Bunny,45),(PetKind::Fox,35),(PetKind::Dragon,17),(PetKind::Unicorn,3)] } }
    fn roll(self, roll:u64)->PetKind {
        let mut threshold=0;
        for &(pet,weight) in self.drops() { threshold+=weight as u64; if roll%100<threshold { return pet; } }
        self.drops()[0].0
    }
}
#[derive(Debug, Clone)]
pub struct Hatch { pub capsule:Capsule, pub inventory_index:usize, pub elapsed:f64 }
#[derive(Debug, Clone)]
pub struct Game {
    pub coins:f64, pub multiplier:u64, pub base_income:f64, pub upgrades:u32,
    pub tab:Tab, pub paused:bool, pub taps:u64, pub total_earned:f64,
    pub played_seconds:f64, pub rebirths:u32,
    pub pets:Vec<PetKind>, pub equipped_pet:Option<usize>, pub hatch:Option<Hatch>, pub pets_open:bool,
    rng:u64,
}
impl Default for Game {
    fn default()->Self { Self { coins:0.0,multiplier:1,base_income:1.0,upgrades:0,tab:Tab::Home,paused:false,
        taps:0,total_earned:0.0,played_seconds:0.0,rebirths:0,pets:Vec::new(),equipped_pet:None,hatch:None,pets_open:false,
        rng:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d|d.as_nanos() as u64).unwrap_or(1).max(1) } }
}
impl Game {
    pub fn tap(&mut self) {
        if self.paused||self.hatch.is_some()||self.pets_open { return; }
        self.multiplier=self.multiplier.saturating_add(1); self.taps=self.taps.saturating_add(1);
    }
    pub fn tick(&mut self,seconds:f64) {
        if self.paused||!seconds.is_finite() { return; }
        let dt=seconds.max(0.0); let income=self.coins_per_second()*dt;
        self.coins+=income; self.total_earned+=income; self.played_seconds+=dt;
        if let Some(h)=&mut self.hatch { h.elapsed+=dt; }
    }
    pub fn level(&self)->u64 { self.taps/100+1 }
    pub fn level_progress(&self)->f32 { (self.taps%100) as f32/100.0 }
    pub fn rank(&self)-> &'static str { match self.level() { 1..=4=>"НОВИЧОК",5..=14=>"ИСКАТЕЛЬ",15..=29=>"МАСТЕР",_=>"ЛЕГЕНДА" } }
    pub fn rebirth_bonus(&self)->f64 { 2.0_f64.powi(self.rebirths.min(30) as i32) }
    pub fn active_pet(&self)->Option<PetKind> { self.equipped_pet.and_then(|i|self.pets.get(i)).copied() }
    pub fn pet_bonus(&self)->f64 { self.active_pet().map(PetKind::bonus).unwrap_or(1.0) }
    pub fn coins_per_second(&self)->f64 { self.base_income*self.multiplier as f64*self.rebirth_bonus()*self.pet_bonus() }
    pub fn upgrade_price(&self)->f64 { 50.0*2.0_f64.powi(self.upgrades.min(30) as i32) }
    pub fn upgrade(&mut self)->bool {
        let cost=self.upgrade_price();
        if self.paused||self.hatch.is_some()||self.coins<cost { return false; }
        self.coins-=cost; self.upgrades=self.upgrades.saturating_add(1); self.base_income+=1.0; true
    }
    pub fn rebirth_price(&self)->f64 { 10_000.0*10.0_f64.powi(self.rebirths.min(30) as i32) }
    pub fn rebirth(&mut self)->bool {
        if self.paused||self.hatch.is_some()||self.coins<self.rebirth_price() { return false; }
        self.coins=0.0; self.multiplier=1; self.base_income=1.0; self.upgrades=0;
        self.rebirths=self.rebirths.saturating_add(1); true
    }
    pub fn buy_capsule(&mut self,capsule:Capsule)->bool {
        if self.paused||self.hatch.is_some()||self.pets.len()>=1000||self.coins<capsule.price() { return false; }
        self.rng^=self.rng<<13; self.rng^=self.rng>>7; self.rng^=self.rng<<17;
        self.coins-=capsule.price(); let index=self.pets.len();
        self.pets.push(capsule.roll(self.rng));
        self.hatch=Some(Hatch{capsule,inventory_index:index,elapsed:0.0}); true
    }
    pub fn equip_pet(&mut self,index:usize)->bool {
        if self.paused||index>=self.pets.len() { return false; }
        self.equipped_pet=Some(index); true
    }
    pub fn finish_hatch(&mut self,equip:bool)->bool {
        let Some(h)=&self.hatch else { return false; };
        if self.paused||h.elapsed<3.0 { return false; }
        let index=h.inventory_index; self.hatch=None;
        if equip { self.equip_pet(index); } true
    }
    pub fn save_text(&self)->String {
        let pets=self.pets.iter().map(|p|(*p as u8).to_string()).collect::<Vec<_>>().join(",");
        format!("BUTTON_SIM_1\n{} {} {} {} {} {} {} {}\n{}\n{}\n",
            self.coins,self.multiplier,self.base_income,self.upgrades,self.taps,self.total_earned,self.played_seconds,self.rebirths,
            pets,self.equipped_pet.map(|i|i.to_string()).unwrap_or_else(||"-".into()))
    }
    pub fn from_save(text:&str)->Option<Self> {
        let mut lines=text.lines(); if lines.next()? !="BUTTON_SIM_1" { return None; }
        let mut v=lines.next()?.split_whitespace(); let mut g=Self::default();
        g.coins=v.next()?.parse().ok()?; g.multiplier=v.next()?.parse().ok()?;
        g.base_income=v.next()?.parse().ok()?; g.upgrades=v.next()?.parse().ok()?;
        g.taps=v.next()?.parse().ok()?; g.total_earned=v.next()?.parse().ok()?;
        g.played_seconds=v.next()?.parse().ok()?; g.rebirths=v.next()?.parse().ok()?;
        if ![g.coins,g.base_income,g.total_earned,g.played_seconds].iter().all(|x|x.is_finite()&&*x>=0.0)||g.multiplier==0||g.base_income<1.0 { return None; }
        let pets=lines.next()?; if !pets.is_empty() {
            for id in pets.split(',').take(1001) { g.pets.push(PetKind::from_id(id.parse().ok()?)?); }
        }
        if g.pets.len()>1000 { return None; }
        let equipped=lines.next()?; if equipped!="-" {
            let i:usize=equipped.parse().ok()?; if i>=g.pets.len() { return None; } g.equipped_pet=Some(i);
        }
        Some(g)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn multiplier_increases_passive_income() { let mut g=Game::default();g.tap();g.tick(2.0);assert_eq!(g.coins,4.0); }
    #[test] fn pause_freezes_economy_and_actions() {
        let mut g=Game{paused:true,coins:20_000.0,..Game::default()};g.tap();g.tick(3.0);
        assert!(!g.upgrade());assert!(!g.rebirth());assert!(!g.buy_capsule(Capsule::Meadow));
        assert_eq!(g.coins,20_000.0);assert_eq!(g.taps,0);assert_eq!(g.played_seconds,0.0);
    }
    #[test] fn rebirth_keeps_profile() {
        let mut g=Game::default();assert!(!g.rebirth());for _ in 0..100 { g.tap(); }
        g.coins=10_000.0;assert!(g.rebirth());assert_eq!(g.coins_per_second(),2.0);assert_eq!(g.level(),2);
    }
    #[test] fn level_rollover() {
        let mut g=Game::default();for _ in 0..99 { g.tap(); }assert_eq!(g.level(),1);
        g.tap();assert_eq!(g.level(),2);assert_eq!(g.level_progress(),0.0);
    }
    #[test] fn capsule_charges_once_and_equips_after_animation() {
        let mut g=Game::default();assert!(!g.buy_capsule(Capsule::Meadow));g.coins=1000.0;
        assert!(g.buy_capsule(Capsule::Meadow));assert_eq!(g.coins,750.0);assert_eq!(g.pets.len(),1);
        assert!(!g.buy_capsule(Capsule::Meadow));assert!(!g.finish_hatch(true));
        g.tick(3.0);assert!(g.finish_hatch(true));assert_eq!(g.equipped_pet,Some(0));
        assert_eq!(g.pet_bonus(),g.pets[0].bonus());assert!(!g.equip_pet(2));
    }
    #[test] fn displayed_drop_weights_match_every_roll() {
        for c in [Capsule::Meadow,Capsule::Neon] { for &(pet,w) in c.drops() {
            assert_eq!((0..100).filter(|&r|c.roll(r)==pet).count(),w as usize);
        } }
    }
    #[test] fn pause_freezes_hatch_and_rebirth_keeps_pets() {
        let mut g=Game{coins:100_000.0,..Game::default()};g.buy_capsule(Capsule::Neon);
        g.paused=true;g.tick(4.0);assert_eq!(g.hatch.as_ref().unwrap().elapsed,0.0);
        g.paused=false;g.tick(3.0);g.finish_hatch(true);assert!(g.rebirth());
        assert_eq!(g.pets.len(),1);assert_eq!(g.equipped_pet,Some(0));
    }
    #[test] fn save_preserves_purchase_and_equipped_pet() {
        let mut g=Game{coins:5000.0,..Game::default()};g.buy_capsule(Capsule::Neon);
        let restored=Game::from_save(&g.save_text()).unwrap();
        assert_eq!(restored.coins,2500.0);assert_eq!(restored.pets,g.pets);assert!(restored.hatch.is_none());
        g.tick(3.0);g.finish_hatch(true);let restored=Game::from_save(&g.save_text()).unwrap();
        assert_eq!(restored.equipped_pet,Some(0));assert!(Game::from_save("broken").is_none());
    }
}
