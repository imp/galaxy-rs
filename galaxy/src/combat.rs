use bevy::prelude::*;

use crate::race::RaceId;
use crate::ship::Ship;

/// Result of a combat encounter
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CombatResult {
    pub attacker_survived: bool,
    pub defender_survived: bool,
    pub attacker_damage_dealt: f64,
    pub defender_damage_dealt: f64,
}

/// Combat system for ship-to-ship battles
#[derive(Debug, Default, Resource)]
pub struct CombatSystem;

impl CombatSystem {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// Resolve combat between two ships (simple deterministic version)
    /// Takes weapon tech for damage calculation
    pub fn resolve_combat(
        attacker: &mut Ship,
        attacker_weapons_tech: f64,
        defender: &mut Ship,
        defender_weapons_tech: f64,
    ) -> CombatResult {
        let attacker_damage = attacker.attack_strength(attacker_weapons_tech);
        let defender_damage = defender.attack_strength(defender_weapons_tech);

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
        // Design: drive=2.0, attacks=1, weapons=1.0, shields=10.0, cargo=0.0
        let design1 = ShipDesign::new(2.0, 1, 1.0, 10.0, 0.0);
        let design2 = ShipDesign::new(2.0, 1, 1.0, 10.0, 0.0);

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, 1.0, &mut ship2, 1.0);

        // Both should survive with some damage
        assert!(result.attacker_survived);
        assert!(result.defender_survived);
        assert!(ship1.current_hull() < 10.0);
        assert!(ship2.current_hull() < 10.0);
    }

    #[test]
    fn test_combat_attacker_destroyed() {
        let design1 = ShipDesign::new(1.0, 1, 0.5, 1.0, 0.0); // Weak
        let design2 = ShipDesign::new(2.0, 1, 5.0, 10.0, 0.0); // Strong

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, 1.0, &mut ship2, 1.0);

        // Weak ship should be destroyed
        assert!(!result.attacker_survived);
        assert!(result.defender_survived);
        assert!(ship1.is_destroyed());
    }

    #[test]
    fn test_combat_defender_destroyed() {
        let design1 = ShipDesign::new(2.0, 1, 5.0, 10.0, 0.0); // Strong
        let design2 = ShipDesign::new(1.0, 1, 0.5, 1.0, 0.0); // Weak

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, 1.0, &mut ship2, 1.0);

        // Weak ship should be destroyed
        assert!(result.attacker_survived);
        assert!(!result.defender_survived);
        assert!(ship2.is_destroyed());
    }

    #[test]
    fn test_combat_mutual_destruction() {
        let design1 = ShipDesign::new(1.0, 1, 2.0, 2.0, 0.0);
        let design2 = ShipDesign::new(1.0, 1, 2.0, 2.0, 0.0);

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, 1.0, &mut ship2, 1.0);

        // Both should be destroyed
        assert!(!result.attacker_survived);
        assert!(!result.defender_survived);
    }

    #[test]
    fn test_combat_damage_calculation() {
        let design1 = ShipDesign::new(1.0, 2, 3.0, 10.0, 0.0); // 2 attacks, 3.0 weapons
        let design2 = ShipDesign::new(1.0, 1, 2.0, 10.0, 0.0); // 1 attack, 2.0 weapons

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, 2.0, &mut ship2, 1.0);

        // Attack strength = weapons_mass × weapons_tech
        // Ship1: 3.0 × 2.0 = 6.0
        // Ship2: 2.0 × 1.0 = 2.0
        assert_eq!(result.attacker_damage_dealt, 6.0);
        assert_eq!(result.defender_damage_dealt, 2.0);
    }

    #[test]
    fn test_should_engage_different_races() {
        assert!(CombatSystem::should_engage(RaceId(0), RaceId(1)));
        assert!(!CombatSystem::should_engage(RaceId(0), RaceId(0)));
    }
}
