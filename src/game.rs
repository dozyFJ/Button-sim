//! Clicker economy, capsule drops and persistent player collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Home, Capsules, Upgrades, Statistics, Rebirths }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetKind { Cat, Dog, Bunny, Fox, Dragon, Unicorn }
impl PetKind {
    pub fn name(self) -> &'static str { match self { Self::Cat=>"Котик", Self::Dog=>"Щенок", Self::Bunny=>"Зайчик", Self::Fox=>"Лисёнок", Self::Dragon=>"Дракон", Self::Unicorn=>"Единорог" } }
    pub fn rarity(self) -> &'static str { match self { Self::Cat|Self::Dog=>"Обычный", Self::Bunny=>"Редкий", Self::Fox=>"Эпический", _=>"Легендарный" } }
    pub fn evolution_tier(self) -> u8 {
        match self { Self::Cat | Self::Dog => 1, Self::Bunny => 2, Self::Fox => 3, Self::Dragon => 4, Self::Unicorn => 5 }
    }
    pub fn damage(self) -> u64 {
        match self { Self::Cat => 2, Self::Dog => 3, Self::Bunny => 5, Self::Fox => 8, Self::Dragon => 12, Self::Unicorn => 16 }
    }
    pub fn bonus(self) -> f64 { match self { Self::Cat=>1.2, Self::Dog=>1.3, Self::Bunny=>1.6, Self::Fox=>2.0, Self::Dragon=>3.0, Self::Unicorn=>4.0 } }
    pub fn color(self) -> [u8;3] { match self { Self::Cat=>[124,178,255], Self::Dog=>[227,176,117], Self::Bunny=>[245,186,226], Self::Fox=>[255,139,69], Self::Dragon=>[135,102,242], Self::Unicorn=>[132,246,226] } }
    fn from_id(id: usize) -> Option<Self> { [Self::Cat,Self::Dog,Self::Bunny,Self::Fox,Self::Dragon,Self::Unicorn].get(id).copied() }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capsule { Meadow, Neon }
impl Capsule {
    pub fn name(self)-> &'static str { match self { Self::Meadow=>"Луговое яйцо",Self::Neon=>"Солнечное яйцо" } }
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
pub struct ActiveBoost {
    pub name: String,
    pub expires_at: f64,
}
#[derive(Debug, Clone)]
pub struct Game {
    pub coins:f64, pub multiplier:u64, pub base_income:f64, pub upgrades:u32,
    pub tab:Tab, pub paused:bool, pub taps:u64, pub total_earned:f64,
    pub played_seconds:f64, pub rebirths:u32,
    pub pets:Vec<PetKind>, pub equipped_pet:Option<usize>, pub hatch:Option<Hatch>, pub pets_open:bool,
    pub equipped_pets:Vec<usize>, pub auto_attack:bool, pub attack_clock:f64, pub auto_hits:u64,
    pub grid_size:usize, pub selected_switch:usize, pub switches:Vec<bool>,
    pub interface_open:bool,
    pub dark_mode: bool,
    pub active_boosts: Vec<ActiveBoost>,
    revealed_switches: usize,
    rebirth_damage: u64,
    pub last_reward: Option<(usize, f64)>,
    switch_damage: Vec<u64>,
    pet_xp: std::collections::BTreeMap<usize, u64>,
    rng:u64,
}
impl Default for Game {
    fn default()->Self { Self { coins:0.0,multiplier:1,base_income:1.0,upgrades:0,tab:Tab::Home,paused:false,
        taps:0,total_earned:0.0,played_seconds:0.0,rebirths:0,pets:Vec::new(),equipped_pet:None,hatch:None,pets_open:false,equipped_pets:Vec::new(),auto_attack:false,attack_clock:0.0,auto_hits:0,grid_size:32,selected_switch:0,switches:vec![false;1024],interface_open:false,dark_mode:false,active_boosts:Vec::new(),revealed_switches:5,rebirth_damage:0,last_reward:None,switch_damage:vec![0;1024],pet_xp:Default::default(),
        rng:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d|d.as_nanos() as u64).unwrap_or(1).max(1) } }
}
impl Game {
    pub fn tap(&mut self) { self.tap_switch(self.selected_switch); }
    pub fn switch_max_hp(index: usize) -> u64 { 6 + 2 * index as u64 }
    pub fn switch_price(index: usize) -> f64 { 5.0 * (index + 1) as f64 }
    pub fn switch_hp(&self, index: usize) -> u64 { Self::switch_max_hp(index) - self.switch_damage[index] }
    fn damage_switch(&mut self, damage: u64) {
        let target = self.selected_switch;
        self.switch_damage[target] = self.switch_damage[target].saturating_add(damage).min(Self::switch_max_hp(target));
        self.taps = self.taps.saturating_add(1);
        self.switches[target] = !self.switches[target];
    }
    pub fn visible_switches(&self) -> usize {
        self.revealed_switches.min(self.grid_size * self.grid_size).min(self.switches.len())
    }
    fn reveal_affordable_switches(&mut self) {
        while self.revealed_switches < self.switches.len() {
            let step = (self.revealed_switches - 4) as f64;
            if self.coins < 1_000.0 * step * step { break; }
            self.revealed_switches += 1;
        }
    }
    pub fn switch_required_level(index:usize)->u64 { if index<5 {1} else {(index-3) as u64} }
    pub fn switch_required_rebirths(index: usize) -> u32 {
        if index < 5 { 0 } else { 1 + ((index - 5) / 10) as u32 }
    }
    pub fn switch_unlocked(&self,index:usize)->bool {
        index < self.visible_switches()
            && self.level() >= Self::switch_required_level(index)
            && self.rebirths >= Self::switch_required_rebirths(index)
    }
    pub fn set_grid_size(&mut self,size:usize)->bool {
        if self.paused||self.hatch.is_some()||![8,32].contains(&size) {return false;}
        self.grid_size=size;if self.selected_switch>=size*size {self.selected_switch=0;self.attack_clock=0.0;} true
    }
    pub fn tap_switch(&mut self,index:usize)->bool {
        if self.paused||self.pets_open||self.interface_open||self.hatch.is_some()||!self.switch_unlocked(index) {return false;}
        let can_hit = self.switch_hp(index) > 0;
        let price = Self::switch_price(index);
        if !can_hit && self.coins < price { return false; }
        if self.selected_switch!=index {self.selected_switch=index;self.attack_clock=0.0;}
        if can_hit {
            self.damage_switch(1);
            return true;
        }
        self.buy_switch_reward(index)
    }
    fn buy_switch_reward(&mut self, index: usize) -> bool {
        let price = Self::switch_price(index);
        if self.switch_hp(index) != 0 || self.coins < price { return false; }
        self.coins -= price;
        self.multiplier = self.multiplier.saturating_add(1);
        self.switch_damage[index] = 0;
        self.last_reward = Some((index, self.played_seconds));
        self.attack_clock = 0.0;
        true
    }
    pub fn rebirth_tile_max_hp(index: u32) -> u64 { 20 + 10 * u64::from(index) }
    pub fn rebirth_tile_hp(&self) -> u64 { Self::rebirth_tile_max_hp(self.rebirths) - self.rebirth_damage }
    fn damage_rebirth_tile(&mut self, damage: u64) {
        self.rebirth_damage = self.rebirth_damage.saturating_add(damage).min(Self::rebirth_tile_max_hp(self.rebirths));
        self.taps = self.taps.saturating_add(1);
    }
    pub fn tap_rebirth_tile(&mut self) -> bool {
        if self.paused || self.hatch.is_some() || self.interface_open || self.pets_open { return false; }
        if self.rebirth_tile_hp() == 0 { return self.rebirth(); }
        self.damage_rebirth_tile(1);
        true
    }
    pub fn tick(&mut self,seconds:f64) {
        if self.paused||!seconds.is_finite() { return; }
        let dt=seconds.max(0.0);
        let income=self.coins_per_second()*dt;
        self.coins+=income; self.total_earned+=income; self.played_seconds+=dt;
        self.reveal_affordable_switches();
        if let Some(h)=&mut self.hatch { h.elapsed+=dt; }
        if self.attack_running() {
            let target = self.selected_switch;
            let rebirth_tile = self.tab == Tab::Rebirths;
            if if rebirth_tile { self.rebirth_tile_hp() == 0 } else { self.switch_hp(target) == 0 } {
                if rebirth_tile { self.rebirth(); } else { self.buy_switch_reward(target); }
                return;
            }
            let before = self.attack_clock;
            self.attack_clock += dt;
            let mut next = [f64::INFINITY; 3];
            for (slot, time) in next.iter_mut().take(self.equipped_pets.len()).enumerate() {
                let offset = 0.7 + slot as f64 * 0.18;
                *time = offset + (before - offset).max(0.0).floor() + 1.0;
                if *time <= before { *time += 1.0; }
            }
            while let Some((slot, time)) = next.iter().copied().enumerate()
                .filter(|(_, time)| time.is_finite() && *time <= self.attack_clock)
                .min_by(|a, b| a.1.total_cmp(&b.1))
            {
                let index = self.equipped_pets[slot];
                let xp = self.pet_xp.entry(index).or_default();
                *xp = xp.saturating_add(1);
                let damage = self.pets[index].damage();
                self.auto_hits = self.auto_hits.saturating_add(1);
                if rebirth_tile {
                    self.damage_rebirth_tile(damage);
                    if self.rebirth_tile_hp() == 0 { self.rebirth(); break; }
                } else {
                    self.damage_switch(damage);
                    if self.switch_hp(target) == 0 { self.buy_switch_reward(target); break; }
                }
                next[slot] = time + 1.0;
            }
        }
    }
    pub fn level(&self)->u64 { self.taps/100+1 }
    pub fn level_progress(&self)->f32 { (self.taps%100) as f32/100.0 }
    pub fn rank(&self)-> &'static str { match self.level() { 1..=4=>"НОВИЧОК",5..=14=>"ИСКАТЕЛЬ",15..=29=>"МАСТЕР",_=>"ЛЕГЕНДА" } }
    pub fn rebirth_bonus(&self)->f64 { 2.0_f64.powi(self.rebirths.min(30) as i32) }
    pub fn active_pet(&self)->Option<PetKind> { self.equipped_pet.and_then(|i|self.pets.get(i)).copied() }
    pub fn pet_bonus(&self)->f64 {
        1.0+self.equipped_pets.iter().filter_map(|&i|self.pets.get(i)).map(|p|p.bonus()-1.0).sum::<f64>()
    }
    pub fn attack_running(&self)->bool {
        self.auto_attack && !self.paused && self.hatch.is_none() && !self.pets_open
            && !self.interface_open && !self.equipped_pets.is_empty()
            && (self.tab == Tab::Rebirths || self.tab == Tab::Home && self.switch_unlocked(self.selected_switch))
    }
    pub fn toggle_auto_attack(&mut self)->bool {
        if self.paused||self.hatch.is_some()||self.pets_open||self.equipped_pets.is_empty() { return false; }
        self.auto_attack=!self.auto_attack;self.attack_clock=0.0;
        if self.auto_attack && self.tab != Tab::Rebirths { self.tab=Tab::Home; }
        true
    }
    pub fn attack_phase(&self,slot:usize)->f32 {
        ((self.attack_clock-0.7-slot as f64*0.18).max(0.0)%1.0) as f32
    }
    pub fn pet_is_striking(&self,slot:usize)->bool {
        self.auto_attack&&self.attack_clock>=1.7+slot as f64*0.18&&self.attack_phase(slot)<0.2
    }
    pub fn coins_per_second(&self)->f64 { self.base_income*self.multiplier as f64*self.rebirth_bonus()*self.pet_bonus() }
    pub fn upgrade_price(&self)->f64 { 50.0*2.0_f64.powi(self.upgrades.min(30) as i32) }
    pub fn upgrade(&mut self)->bool {
        let cost=self.upgrade_price();
        if self.paused||self.hatch.is_some()||self.coins<cost { return false; }
        self.coins-=cost; self.upgrades=self.upgrades.saturating_add(1); self.base_income+=1.0; true
    }
    pub fn rebirth_price_at(rebirths: u32) -> f64 { 10_000.0 * 10.0_f64.powi(rebirths.min(30) as i32) }
    pub fn rebirth_price(&self)->f64 { Self::rebirth_price_at(self.rebirths) }
    pub fn rebirth(&mut self)->bool {
        if self.paused||self.hatch.is_some()||self.coins<self.rebirth_price() { return false; }
        self.reveal_affordable_switches();
        self.coins=0.0; self.multiplier=1; self.base_income=1.0; self.upgrades=0;
        self.rebirths=self.rebirths.saturating_add(1); self.rebirth_damage=0; self.attack_clock=0.0; true
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
        if !self.equipped_pets.contains(&index) {
            if self.equipped_pets.len()==3 { self.equipped_pets.pop(); }
            self.equipped_pets.push(index);
            self.attack_clock=0.0;
        }
        self.equipped_pet=self.equipped_pets.first().copied(); true
    }
    pub fn unequip_pet(&mut self,index:usize)->bool {
        if self.paused||self.hatch.is_some() { return false; }
        let Some(slot)=self.equipped_pets.iter().position(|&i|i==index) else { return false; };
        self.equipped_pets.remove(slot);self.equipped_pet=self.equipped_pets.first().copied();
        self.attack_clock=0.0;if self.equipped_pets.is_empty() { self.auto_attack=false; } true
    }
    pub fn pet_xp(&self, index: usize) -> u64 {
        self.pet_xp.get(&index).copied().unwrap_or(0)
    }
    pub fn equip_best_pet(&mut self) -> bool {
        let best = (0..self.pets.len()).filter(|index| !self.equipped_pets.contains(index))
            .max_by_key(|&index| (self.pets[index].damage(), self.pet_xp(index), std::cmp::Reverse(index)));
        best.is_some_and(|index| self.equip_pet(index))
    }
    pub fn unequip_all_pets(&mut self) {
        if self.paused || self.hatch.is_some() { return; }
        self.equipped_pets.clear();
        self.equipped_pet = None;
        self.auto_attack = false;
        self.attack_clock = 0.0;
    }
    pub fn delete_pets(&mut self, mut indices: Vec<usize>) -> bool {
        if self.paused || self.hatch.is_some() || indices.is_empty()
            || indices.iter().any(|&index| index >= self.pets.len()) { return false; }
        indices.sort_unstable();
        indices.dedup();
        for index in indices.into_iter().rev() { self.delete_pet(index); }
        true
    }
    pub fn delete_pet(&mut self,index:usize)->bool {
        if self.paused||self.hatch.is_some()||index>=self.pets.len() {return false;}
        self.pets.remove(index);
        let mut later_xp = self.pet_xp.split_off(&index);
        later_xp.remove(&index);
        self.pet_xp.extend(later_xp.into_iter().map(|(slot, xp)| (slot - 1, xp)));
        self.equipped_pets.retain(|&i|i!=index);
        for i in &mut self.equipped_pets {if *i>index {*i-=1;}}
        self.equipped_pet=self.equipped_pets.first().copied();self.attack_clock=0.0;
        if self.equipped_pets.is_empty() {self.auto_attack=false;}
        true
    }
    pub fn craft_result(kind:PetKind)->Option<PetKind> {
        match kind {PetKind::Cat|PetKind::Dog=>Some(PetKind::Bunny),PetKind::Bunny=>Some(PetKind::Fox),PetKind::Fox=>Some(PetKind::Dragon),PetKind::Dragon=>Some(PetKind::Unicorn),PetKind::Unicorn=>None}
    }
    pub fn craft_materials(&self,index:usize)->Option<Vec<usize>> {
        let kind=*self.pets.get(index)?;Self::craft_result(kind)?;
        let mut materials=vec![index];
        for (i,&pet) in self.pets.iter().enumerate() {if i!=index&&pet==kind {materials.push(i);if materials.len()==3 {return Some(materials);}}}
        None
    }
    pub fn craft_pet(&mut self,index:usize)->Option<usize> {
        if self.paused||self.hatch.is_some() {return None;}
        let result=Self::craft_result(*self.pets.get(index)?)?;
        let mut materials=self.craft_materials(index)?;
        let equipped=materials.iter().any(|i|self.equipped_pets.contains(i));let auto=self.auto_attack;
        materials.sort_unstable_by(|a,b|b.cmp(a));
        for i in materials {self.delete_pet(i);}
        let index=self.pets.len();self.pets.push(result);
        if equipped {self.equip_pet(index);self.auto_attack=auto;}
        Some(index)
    }
    pub fn switch_rarity(index:usize)->&'static str {
        match index {0..=4=>"Старт",5..=31=>"Обычн.",32..=127=>"Редкая",128..=511=>"Эпик",_=>"Легенд."}
    }
    pub fn finish_hatch(&mut self,equip:bool)->bool {
        let Some(h)=&self.hatch else { return false; };
        if self.paused||h.elapsed<3.0 { return false; }
        let index=h.inventory_index; self.hatch=None;
        if equip { self.equip_pet(index); } true
    }
    pub fn save_text(&self)->String {
        let pets=self.pets.iter().map(|p|(*p as u8).to_string()).collect::<Vec<_>>().join(",");
        let damage = self.switch_damage.iter().enumerate().filter(|(_, damage)| **damage > 0)
            .map(|(index, damage)| format!("{index}:{damage}")).collect::<Vec<_>>().join(",");
        let xp = self.pet_xp.iter().map(|(index, xp)| format!("{index}:{xp}"))
            .collect::<Vec<_>>().join(",");
        format!("BUTTON_SIM_1\n{} {} {} {} {} {} {} {}\n{}\n{}\n{}\n{}\n{}\n{} {}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            self.coins,self.multiplier,self.base_income,self.upgrades,self.taps,self.total_earned,self.played_seconds,self.rebirths,
            pets,self.equipped_pet.map(|i|i.to_string()).unwrap_or_else(||"-".into()),
            self.equipped_pets.iter().map(|i|i.to_string()).collect::<Vec<_>>().join(","),
            self.auto_attack,self.auto_hits,self.grid_size,self.selected_switch,
            self.switches.iter().map(|b|if *b {'1'} else {'0'}).collect::<String>(), self.dark_mode,damage,xp,self.revealed_switches,self.rebirth_damage)
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
        if let Some(team)=lines.next() {
            if !team.is_empty() {
                for id in team.split(',') {
                    let i:usize=id.parse().ok()?;
                    if i>=g.pets.len()||g.equipped_pets.contains(&i)||g.equipped_pets.len()>=3 { return None; }
                    g.equipped_pets.push(i);
                }
            }
        } else if let Some(i)=g.equipped_pet { g.equipped_pets.push(i); }
        g.equipped_pet=g.equipped_pets.first().copied();
        g.auto_attack=lines.next().map(str::parse).transpose().ok()?.unwrap_or(false)&&!g.equipped_pets.is_empty();
        g.auto_hits=lines.next().map(str::parse).transpose().ok()?.unwrap_or(0);
        if let Some(board)=lines.next() {
            let mut fields=board.split_whitespace();
            g.grid_size=fields.next()?.parse().ok()?;g.selected_switch=fields.next()?.parse().ok()?;
            if ![8,32].contains(&g.grid_size)||g.selected_switch>=g.grid_size*g.grid_size {return None;}
            let states=lines.next()?;if states.len()!=1024||!states.bytes().all(|b|b==b'0'||b==b'1') {return None;}
            g.switches=states.bytes().map(|b|b==b'1').collect();
        }
        g.dark_mode = lines.next() == Some("true");
        if let Some(damage) = lines.next().filter(|line| !line.is_empty()) {
            for entry in damage.split(',') {
                let (index, damage) = entry.split_once(':')?;
                let index: usize = index.parse().ok()?;
                let damage: u64 = damage.parse().ok()?;
                if index >= g.switch_damage.len() || damage > Self::switch_max_hp(index) { return None; }
                g.switch_damage[index] = damage;
            }
        }
        if let Some(xp) = lines.next().filter(|line| !line.is_empty()) {
            for entry in xp.split(',') {
                let (index, xp) = entry.split_once(':')?;
                let index: usize = index.parse().ok()?;
                if index >= g.pets.len() { return None; }
                g.pet_xp.insert(index, xp.parse().ok()?);
            }
        }
        g.revealed_switches = if let Some(count) = lines.next().filter(|line| !line.is_empty()) {
            count.parse::<usize>().ok()?.clamp(5, g.switches.len())
        } else {
            let previously_used = g.switches.iter().zip(&g.switch_damage)
                .rposition(|(active, damage)| *active || *damage > 0)
                .map_or(5, |index| index + 1);
            previously_used.max(g.selected_switch + 1).max(5)
        };
        if let Some(damage) = lines.next().filter(|line| !line.is_empty()) {
            // A previous release briefly stored a colon-delimited cooldown list here.
            if !damage.contains(':') {
                g.rebirth_damage = damage.parse::<u64>().ok()?.min(Self::rebirth_tile_max_hp(g.rebirths));
            }
        }
        g.reveal_affordable_switches();
        if !g.switch_unlocked(g.selected_switch) { g.selected_switch = 0; }
        g.grid_size=32;
        Some(g)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn multiplier_increases_passive_income() { let mut g=Game::default();for _ in 0..6 { g.tap(); } g.coins=5.0; g.tap(); g.tick(2.0);assert_eq!(g.coins,4.0); }
    #[test] fn pause_freezes_economy_and_actions() {
        let mut g=Game{paused:true,coins:20_000.0,..Game::default()};g.tap();g.tick(3.0);
        assert!(!g.upgrade());assert!(!g.rebirth());assert!(!g.buy_capsule(Capsule::Meadow));
        assert_eq!(g.coins,20_000.0);assert_eq!(g.taps,0);assert_eq!(g.played_seconds,0.0);
    }
    #[test] fn rebirth_keeps_profile() {
        let mut g=Game::default();assert!(!g.rebirth());g.taps=100;
        g.coins=10_000.0;assert!(g.rebirth());assert_eq!(g.coins_per_second(),2.0);assert_eq!(g.level(),2);
    }
    #[test] fn level_rollover() {
        let mut g=Game::default();g.taps=98;g.tap();assert_eq!(g.level(),1);
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

    #[test]
    fn theme_preference_survives_reload_and_defaults_to_light_for_old_saves() {
        let game = Game { dark_mode: true, ..Game::default() };
        let save = game.save_text();
        assert!(Game::from_save(&save).unwrap().dark_mode);
        let old_save = save.lines().take(9).collect::<Vec<_>>().join("\n");
        assert!(!Game::from_save(&old_save).unwrap().dark_mode);
    }

    #[test]
    fn pet_xp_follows_the_pet_through_save_and_bulk_deletion() {
        let mut game = Game::default();
        game.pets = vec![PetKind::Dog, PetKind::Cat, PetKind::Dragon];
        assert!(game.equip_best_pet());
        assert_eq!(game.equipped_pets, vec![2]);
        assert!(game.toggle_auto_attack());
        game.tick(2.0);
        assert_eq!(game.pet_xp(2), 1);
        let mut game = Game::from_save(&game.save_text()).unwrap();
        assert!(!game.delete_pets(vec![0, 9]));
        assert!(game.delete_pets(vec![1, 0, 1]));
        assert_eq!(game.pets, vec![PetKind::Dragon]);
        assert_eq!(game.pet_xp(0), 1);
        assert_eq!(game.equipped_pets, vec![0]);
        game.unequip_all_pets();
        assert!(game.equipped_pets.is_empty() && !game.auto_attack);
    }

    #[test] fn auto_attack_requires_equipped_pet_and_travel() {
        let mut g=Game::default();assert!(!g.toggle_auto_attack());
        g.pets.push(PetKind::Cat);g.equip_pet(0);assert!(g.toggle_auto_attack());
        g.tick(1.69);assert_eq!(g.auto_hits,0);g.tick(0.02);
        assert_eq!(g.auto_hits,1);assert_eq!(g.switch_hp(0),4);assert_eq!(g.multiplier,1);assert!(g.pet_is_striking(0));
    }
    #[test]
    fn pets_can_defeat_and_purchase_the_current_rebirth_tile() {
        let mut game = Game::default();
        game.tab = Tab::Rebirths;
        game.coins = 10_000.0;
        game.pets.push(PetKind::Dragon);
        game.equip_pet(0);
        game.toggle_auto_attack();
        game.tick(1.8);
        assert_eq!(game.rebirth_tile_hp(), 8);
        game = Game::from_save(&game.save_text()).unwrap();
        game.tab = Tab::Rebirths;
        game.tick(1.8);
        assert_eq!(game.rebirths, 1);
        assert_eq!(game.coins, 0.0);
        assert_eq!(game.rebirth_tile_hp(), 30);
        assert_eq!(game.switch_hp(0), 6);
    }

    #[test]
    fn pets_collect_depleted_rewards_when_the_balance_is_sufficient() {
        let mut game = Game::default();
        for _ in 0..5 { game.tap(); }
        game.pets.push(PetKind::Cat);
        game.equip_pet(0);
        game.toggle_auto_attack();
        game.tick(1.8);
        assert_eq!(game.auto_hits, 1);
        assert_eq!(game.switch_hp(0), 0);
        assert_eq!(game.multiplier, 1);
        game.coins = Game::switch_price(0);
        game.tick(0.0);
        assert_eq!(game.multiplier, 2);
        assert_eq!(game.coins, 0.0);
        assert_eq!(game.switch_hp(0), 6);
        game.tick(0.0);
        assert_eq!(game.multiplier, 2);
    }

    #[test] fn team_hits_are_staggered_and_timestep_independent() {
        let mut a=Game::default();a.pets=vec![PetKind::Cat,PetKind::Dog,PetKind::Bunny];
        for i in 0..3 {a.equip_pet(i);}
        a.taps = 10_000;
        a.rebirths = 5;
        a.coins = 2_000_000.0;
        a.tick(0.0);
        a.selected_switch = 45;
        a.toggle_auto_attack();let mut b=a.clone();
        a.tick(5.1);for _ in 0..510 {b.tick(0.01);}
        assert_eq!(a.auto_hits,12);assert_eq!(a.auto_hits,b.auto_hits);assert_eq!(a.multiplier,b.multiplier);assert_eq!(a.switch_hp(45),b.switch_hp(45));
        assert!((a.pet_bonus()-2.1).abs()<0.001);
    }
    #[test] fn pause_inventory_and_hatch_suspend_attacks_without_catchup() {
        let mut g=Game::default();g.pets.push(PetKind::Cat);g.equip_pet(0);g.toggle_auto_attack();
        g.tick(1.5);g.paused=true;g.tick(10.0);assert_eq!(g.attack_clock,1.5);
        g.paused=false;g.pets_open=true;g.tick(10.0);assert_eq!(g.auto_hits,0);
        g.pets_open=false;g.coins=1000.0;g.buy_capsule(Capsule::Meadow);g.tick(3.0);assert_eq!(g.auto_hits,0);
        g.finish_hatch(false);g.tick(0.3);assert_eq!(g.auto_hits,1);
        g.unequip_pet(0);assert!(!g.auto_attack);
    }
    #[test] fn team_save_supports_old_format_and_restarts_travel() {
        let old="BUTTON_SIM_1\n1000 1 1 0 0 0 0 0\n0,1\n1\n";
        let mut g=Game::from_save(old).unwrap();assert_eq!(g.equipped_pets,vec![1]);
        g.equip_pet(0);g.toggle_auto_attack();g.tick(2.0);
        let restored=Game::from_save(&g.save_text()).unwrap();
        assert_eq!(restored.equipped_pets,g.equipped_pets);assert!(restored.auto_attack);
        assert_eq!(restored.auto_hits,g.auto_hits);assert_eq!(restored.attack_clock,0.0);
    }


    #[test]
    fn multiplier_tiles_reveal_at_balance_milestones_and_stay_visible() {
        let mut game = Game::default();
        assert_eq!(game.visible_switches(), 5);
        game.coins = 999.0;
        game.tick(0.0);
        assert_eq!(game.visible_switches(), 5);
        game.tick(1.0);
        assert_eq!(game.visible_switches(), 6);
        assert_eq!(game.coins, 1_000.0);
        game.coins = 4_000.0;
        game.tick(0.0);
        assert_eq!(game.visible_switches(), 7);
        game.coins = 0.0;
        let restored = Game::from_save(&game.save_text()).unwrap();
        assert_eq!(restored.visible_switches(), 7);
        assert!(!restored.switch_unlocked(5));
    }

    #[test]
    fn depleted_tiles_sell_one_reward_and_restore_hp_immediately() {
        let mut game = Game::default();
        for _ in 0..6 { assert!(game.tap_switch(0)); }
        assert_eq!(game.switch_hp(0), 0);
        assert_eq!(game.multiplier, 1);
        assert!(!game.tap_switch(0));
        game.coins = Game::switch_price(0) * 2.0;
        assert!(game.tap_switch(0));
        assert_eq!(game.coins, Game::switch_price(0));
        assert_eq!(game.multiplier, 2);
        assert_eq!(game.switch_hp(0), 6);
        assert!(game.tap_switch(0));
        assert_eq!(game.switch_hp(0), 5);
        assert_eq!(game.multiplier, 2);
        assert_eq!(game.coins, Game::switch_price(0));
    }

    #[test] fn starter_switches_and_level_locks() {
        let mut g=Game::default();for i in 0..5 {assert!(g.switch_unlocked(i));}
        assert!(!g.tap_switch(5));assert_eq!(g.taps,0);
        g.taps=100;
        assert!(!g.tap_switch(5));
        g.coins=g.rebirth_price();assert!(g.rebirth());
        assert!(g.tap_switch(5));assert_eq!(g.selected_switch,5);
        assert!(!g.tap_switch(1024));assert_eq!(g.taps,101);
    }
    #[test] fn board_state_saves_and_autoattacks_selected_tile() {
        let mut g=Game::default();g.tap_switch(3);assert!(g.switches[3]);
        g.pets.push(PetKind::Cat);g.equip_pet(0);g.toggle_auto_attack();g.tick(1.8);
        assert!(!g.switches[3]);assert_eq!(g.switch_hp(3),9);assert_eq!(g.auto_hits,1);assert!(!g.switches[0]);
        g.set_grid_size(32);let restored=Game::from_save(&g.save_text()).unwrap();
        assert_eq!(restored.grid_size,32);assert_eq!(restored.selected_switch,3);assert_eq!(restored.switches,g.switches);assert_eq!(restored.switch_hp(3),9);
        assert!(!g.set_grid_size(16));
    }

    #[test] fn deleting_pet_remaps_equipped_indices() {
        let mut g=Game::default();g.pets=vec![PetKind::Cat,PetKind::Dog,PetKind::Fox];g.equip_pet(1);g.equip_pet(2);g.toggle_auto_attack();
        assert!(g.delete_pet(0));assert_eq!(g.pets,vec![PetKind::Dog,PetKind::Fox]);assert_eq!(g.equipped_pets,vec![0,1]);assert!(g.auto_attack);
        g.delete_pet(0);g.delete_pet(0);assert!(!g.auto_attack);assert!(g.equipped_pets.is_empty());assert!(!g.delete_pet(0));
    }
    #[test] fn craft_consumes_three_identical_pets_and_preserves_other_team_members() {
        let mut g=Game::default();g.pets=vec![PetKind::Cat,PetKind::Dog,PetKind::Cat,PetKind::Cat];g.equip_pet(0);g.equip_pet(1);g.toggle_auto_attack();
        let index=g.craft_pet(2).unwrap();assert_eq!(g.pets,vec![PetKind::Dog,PetKind::Bunny]);assert_eq!(index,1);assert_eq!(g.equipped_pets,vec![0,1]);assert!(g.auto_attack);
        let restored=Game::from_save(&g.save_text()).unwrap();assert_eq!(restored.pets,g.pets);assert_eq!(restored.equipped_pets,g.equipped_pets);
    }
    #[test] fn invalid_craft_and_paused_deletion_leave_inventory_intact() {
        let mut g=Game::default();g.pets=vec![PetKind::Cat,PetKind::Cat,PetKind::Unicorn];let before=g.pets.clone();
        assert!(g.craft_pet(0).is_none());assert!(g.craft_pet(2).is_none());assert_eq!(g.pets,before);
        g.paused=true;assert!(!g.delete_pet(0));assert!(g.craft_pet(0).is_none());assert_eq!(g.pets,before);
    }
}
