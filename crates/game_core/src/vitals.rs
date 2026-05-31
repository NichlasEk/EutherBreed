#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApothecaryVitals {
    pub health: i32,
    pub ammo: i32,
    pub bio_samples: i32,
}

impl ApothecaryVitals {
    pub const fn new(health: i32, ammo: i32, bio_samples: i32) -> Self {
        Self {
            health,
            ammo,
            bio_samples,
        }
    }

    pub fn apply_damage(&mut self, amount: i32) -> DamageOutcome {
        if amount <= 0 || self.health == 0 {
            return DamageOutcome::Alive;
        }

        self.health = (self.health - amount).max(0);

        if self.health == 0 {
            DamageOutcome::Dead
        } else {
            DamageOutcome::Alive
        }
    }

    pub fn spend_round(&mut self) -> bool {
        if self.ammo <= 0 {
            return false;
        }

        self.ammo -= 1;
        true
    }

    pub fn collect_bio_sample(&mut self) {
        self.bio_samples += 1;
    }

    pub fn add_ammo(&mut self, amount: i32) {
        if amount > 0 {
            self.ammo += amount;
        }
    }

    pub fn heal(&mut self, amount: i32, max_health: i32) {
        if amount > 0 {
            self.health = (self.health + amount).min(max_health);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageOutcome {
    Alive,
    Dead,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_clamps_health_to_zero() {
        let mut vitals = ApothecaryVitals::new(10, 4, 0);

        assert_eq!(vitals.apply_damage(20), DamageOutcome::Dead);
        assert_eq!(vitals.health, 0);
    }

    #[test]
    fn spending_round_decrements_ammo() {
        let mut vitals = ApothecaryVitals::new(100, 2, 0);

        assert!(vitals.spend_round());
        assert_eq!(vitals.ammo, 1);
    }

    #[test]
    fn spending_round_fails_when_empty() {
        let mut vitals = ApothecaryVitals::new(100, 0, 0);

        assert!(!vitals.spend_round());
        assert_eq!(vitals.ammo, 0);
    }

    #[test]
    fn collecting_sample_increments_counter() {
        let mut vitals = ApothecaryVitals::new(100, 2, 0);

        vitals.collect_bio_sample();

        assert_eq!(vitals.bio_samples, 1);
    }

    #[test]
    fn adding_ammo_ignores_negative_values() {
        let mut vitals = ApothecaryVitals::new(100, 2, 0);

        vitals.add_ammo(-1);
        vitals.add_ammo(4);

        assert_eq!(vitals.ammo, 6);
    }

    #[test]
    fn healing_clamps_to_max_health() {
        let mut vitals = ApothecaryVitals::new(80, 2, 0);

        vitals.heal(40, 100);

        assert_eq!(vitals.health, 100);
    }
}
