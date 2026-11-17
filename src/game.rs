//! Clicker economy, profile progression and navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Home, Capsules, Upgrades, Statistics, Rebirths }

#[derive(Debug, Clone)]
pub struct Game {
    pub coins: f64,
    pub multiplier: u64,
    pub base_income: f64,
    pub upgrades: u32,
    pub tab: Tab,
    pub paused: bool,
    pub taps: u64,
    pub total_earned: f64,
    pub played_seconds: f64,
    pub rebirths: u32,
}
impl Default for Game {
    fn default() -> Self {
        Self { coins: 0.0, multiplier: 1, base_income: 1.0, upgrades: 0,
            tab: Tab::Home, paused: false, taps: 0, total_earned: 0.0,
            played_seconds: 0.0, rebirths: 0 }
    }
}
impl Game {
    pub fn tap(&mut self) {
        if self.paused { return; }
        self.multiplier = self.multiplier.saturating_add(1);
        self.taps = self.taps.saturating_add(1);
    }
    pub fn tick(&mut self, seconds: f64) {
        if self.paused || !seconds.is_finite() { return; }
        let dt = seconds.max(0.0);
        let income = self.coins_per_second() * dt;
        self.coins += income;
        self.total_earned += income;
        self.played_seconds += dt;
    }
    pub fn level(&self) -> u64 { self.taps / 100 + 1 }
    pub fn level_progress(&self) -> f32 { (self.taps % 100) as f32 / 100.0 }
    pub fn rank(&self) -> &'static str {
        match self.level() { 1..=4 => "НОВИЧОК", 5..=14 => "ИСКАТЕЛЬ", 15..=29 => "МАСТЕР", _ => "ЛЕГЕНДА" }
    }
    pub fn rebirth_bonus(&self) -> f64 { 2.0_f64.powi(self.rebirths.min(30) as i32) }
    pub fn coins_per_second(&self) -> f64 {
        self.base_income * self.multiplier as f64 * self.rebirth_bonus()
    }
    pub fn upgrade_price(&self) -> f64 { 50.0 * 2.0_f64.powi(self.upgrades.min(30) as i32) }
    pub fn upgrade(&mut self) -> bool {
        let cost = self.upgrade_price();
        if self.paused || self.coins < cost { return false; }
        self.coins -= cost;
        self.upgrades = self.upgrades.saturating_add(1);
        self.base_income += 1.0;
        true
    }
    pub fn rebirth_price(&self) -> f64 { 10_000.0 * 10.0_f64.powi(self.rebirths.min(30) as i32) }
    pub fn rebirth(&mut self) -> bool {
        if self.paused || self.coins < self.rebirth_price() { return false; }
        self.coins = 0.0;
        self.multiplier = 1;
        self.base_income = 1.0;
        self.upgrades = 0;
        self.rebirths = self.rebirths.saturating_add(1);
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiplier_increases_passive_income() {
        let mut g = Game::default();
        g.tap(); g.tick(2.0);
        assert_eq!(g.coins, 4.0);
    }
    #[test]
    fn pause_freezes_economy_and_actions() {
        let mut g = Game { paused: true, coins: 20_000.0, ..Game::default() };
        g.tap(); g.tick(3.0);
        assert!(!g.upgrade()); assert!(!g.rebirth());
        assert_eq!(g.coins, 20_000.0); assert_eq!(g.taps, 0);
        assert_eq!(g.played_seconds, 0.0);
    }
    #[test]
    fn rebirth_requires_money_resets_run_and_keeps_profile() {
        let mut g = Game::default();
        assert!(!g.rebirth());
        for _ in 0..100 { g.tap(); }
        g.coins = 10_000.0;
        g.total_earned = 12_000.0;
        assert!(g.rebirth());
        assert_eq!(g.coins, 0.0); assert_eq!(g.multiplier, 1);
        assert_eq!(g.coins_per_second(), 2.0);
        assert_eq!(g.level(), 2); assert_eq!(g.total_earned, 12_000.0);
        assert_eq!(g.rebirth_price(), 100_000.0);
    }
    #[test]
    fn level_progress_rolls_over_at_one_hundred_taps() {
        let mut g = Game::default();
        for _ in 0..99 { g.tap(); }
        assert_eq!(g.level(), 1); assert!(g.level_progress() > 0.98);
        g.tap();
        assert_eq!(g.level(), 2); assert_eq!(g.level_progress(), 0.0);
    }
}
