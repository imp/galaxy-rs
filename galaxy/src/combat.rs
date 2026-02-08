use bevy::prelude::*;

use crate::race::RaceId;
use crate::ship::Ship;

/// Result of a combat encounter
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CombatResult {
    pub attacker_survived: bool,
    pub defender_survived: bool,
    pub attacker_damage_dealt: u32,
    pub defender_damage_dealt: u32,
}

/// Combat system for ship-to-ship battles
#[derive(Debug, Default, Resource)]
pub struct CombatSystem;

impl CombatSystem {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// Resolve combat between two ships
    pub fn resolve_combat(attacker: &mut Ship, defender: &mut Ship) -> CombatResult {
        let attacker_damage = attacker.attack_power();
        let defender_damage = defender.attack_power();

        // Both ships attack simultaneously
        attacker.take_damage(defender_damage);
        defender.take_damage(attacker_damage);

        CombatResult {
            attacker_survived: !attacker.is_destroyed(),
            defender_survived: !defender.is_destroyed(),
            attacker_damage_dealt: attacker_damage,
            defender_damage_dealt: defender_damage,
        }
    }

    /// Check if two ships should engage in combat based on their owners
    #[allow(dead_code)]
    pub fn should_engage(ship1_owner: RaceId, ship2_owner: RaceId) -> bool {
        ship1_owner != ship2_owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::PlanetId;
    use crate::ship::ShipDesign;
    use crate::ship::ShipId;

    #[test]
    fn test_combat_both_survive() {
        let design1 = ShipDesign::new(100, 5, 2, 5); // Strong hull, moderate attack
        let design2 = ShipDesign::new(100, 5, 2, 5); // Equal ships

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, &mut ship2);

        // Both should survive with damage
        assert!(result.attacker_survived);
        assert!(result.defender_survived);
        assert!(ship1.current_hull() < design1.hull_strength());
        assert!(ship2.current_hull() < design2.hull_strength());
    }

    #[test]
    fn test_combat_attacker_destroyed() {
        let design1 = ShipDesign::new(5, 5, 1, 1); // Weak ship
        let design2 = ShipDesign::new(100, 5, 5, 10); // Strong ship

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, &mut ship2);

        // Weak ship should be destroyed
        assert!(!result.attacker_survived);
        assert!(result.defender_survived);
        assert!(ship1.is_destroyed());
    }

    #[test]
    fn test_combat_defender_destroyed() {
        let design1 = ShipDesign::new(100, 5, 5, 10); // Strong ship
        let design2 = ShipDesign::new(5, 5, 1, 1); // Weak ship

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, &mut ship2);

        // Weak ship should be destroyed
        assert!(result.attacker_survived);
        assert!(!result.defender_survived);
        assert!(ship2.is_destroyed());
    }

    #[test]
    fn test_combat_mutual_destruction() {
        let design = ShipDesign::new(10, 5, 5, 5); // Equal, fragile ships

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, &mut ship2);

        // Both should be destroyed
        assert!(!result.attacker_survived);
        assert!(!result.defender_survived);
        assert!(ship1.is_destroyed());
        assert!(ship2.is_destroyed());
    }

    #[test]
    fn test_should_engage_different_races() {
        assert!(CombatSystem::should_engage(RaceId(0), RaceId(1)));
    }

    #[test]
    fn test_should_not_engage_same_race() {
        assert!(!CombatSystem::should_engage(RaceId(0), RaceId(0)));
    }
}
