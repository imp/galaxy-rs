use bevy::prelude::*;
use rand::Rng;

use crate::race::RaceId;
use crate::race::Technology;
use crate::ship::Ship;

/// Result of a combat encounter
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub attacker_survived: bool,
    pub defender_survived: bool,
    pub attacker_damage_dealt: f64,
    pub defender_damage_dealt: f64,
    pub rounds: u32,
}

/// Combat system for ship-to-ship battles
#[derive(Debug, Default, Resource)]
pub struct CombatSystem;

impl CombatSystem {
    pub fn new() -> Self {
        Self
    }

    /// Resolve combat between two ships using GalaxyNG probabilistic formulas
    ///
    /// Kill probability: p[kill] = (log4(attack/defence) + 1) / 2
    /// Combat continues until one or both ships are destroyed
    pub fn resolve_combat(
        attacker: &mut Ship,
        attacker_tech: &Technology,
        defender: &mut Ship,
        defender_tech: &Technology,
    ) -> CombatResult {
        let mut rounds = 0;
        let mut attacker_total_damage = 0.0;
        let mut defender_total_damage = 0.0;
        let mut rng = rand::thread_rng();

        while !attacker.is_destroyed() && !defender.is_destroyed() {
            rounds += 1;

            // Calculate attack and defence strengths
            let attacker_attack = attacker.attack_strength(attacker_tech.weapon_level() as f64);
            let attacker_defence = attacker.defence_strength(attacker_tech.shield_level() as f64);

            let defender_attack = defender.attack_strength(defender_tech.weapon_level() as f64);
            let defender_defence = defender.defence_strength(defender_tech.shield_level() as f64);

            // Attacker tries to kill defender
            if attacker_attack > 0.0 && defender_defence > 0.0 {
                let p_attacker_kills =
                    Self::calculate_kill_probability(attacker_attack, defender_defence);
                if rng.gen_range(0.0..1.0) < p_attacker_kills {
                    let damage = defender.current_hull();
                    defender.take_damage(damage);
                    attacker_total_damage += damage;
                }
            }

            // Defender tries to kill attacker (if still alive)
            if !defender.is_destroyed() && defender_attack > 0.0 && attacker_defence > 0.0 {
                let p_defender_kills =
                    Self::calculate_kill_probability(defender_attack, attacker_defence);
                if rng.gen_range(0.0..1.0) < p_defender_kills {
                    let damage = attacker.current_hull();
                    attacker.take_damage(damage);
                    defender_total_damage += damage;
                }
            }

            // Prevent infinite loops - max 100 rounds
            if rounds >= 100 {
                break;
            }
        }

        CombatResult {
            attacker_survived: !attacker.is_destroyed(),
            defender_survived: !defender.is_destroyed(),
            attacker_damage_dealt: attacker_total_damage,
            defender_damage_dealt: defender_total_damage,
            rounds,
        }
    }

    /// Calculate kill probability using GalaxyNG formula:
    /// p[kill] = (log4(attack/defence) + 1) / 2
    ///
    /// Clamped to [0.0, 1.0] to ensure valid probability
    fn calculate_kill_probability(attack: f64, defence: f64) -> f64 {
        if attack <= 0.0 || defence <= 0.0 {
            return 0.0;
        }

        let ratio = attack / defence;
        // log4(x) = ln(x) / ln(4)
        let log4_ratio = ratio.ln() / 4.0_f64.ln();
        let p = (log4_ratio + 1.0) / 2.0;

        // Clamp to valid probability range
        p.clamp(0.0, 1.0)
    }

    /// Check if two ships should engage in combat based on their owners
    pub fn should_engage(ship1_owner: RaceId, ship2_owner: RaceId) -> bool {
        ship1_owner != ship2_owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::PlanetId;
    use crate::race::Technology;
    use crate::ship::ShipDesign;
    use crate::ship::ShipId;

    #[test]
    fn test_kill_probability_formula() {
        // Test the probability formula directly

        // Equal attack and defence (ratio=1.0) -> log4(1) = 0 -> p = 0.5
        let p = CombatSystem::calculate_kill_probability(10.0, 10.0);
        assert!((p - 0.5).abs() < 0.01);

        // Attack 4× defence (ratio=4.0) -> log4(4) = 1 -> p = 1.0
        let p = CombatSystem::calculate_kill_probability(40.0, 10.0);
        assert!((p - 1.0).abs() < 0.01);

        // Attack 0.25× defence (ratio=0.25) -> log4(0.25) = -1 -> p = 0.0
        let p = CombatSystem::calculate_kill_probability(10.0, 40.0);
        assert!((p - 0.0).abs() < 0.01);

        // Zero attack or defence should give 0 probability
        assert_eq!(CombatSystem::calculate_kill_probability(0.0, 10.0), 0.0);
        assert_eq!(CombatSystem::calculate_kill_probability(10.0, 0.0), 0.0);
    }

    #[test]
    fn test_combat_probabilistic() {
        // Run many combats to verify probabilistic behavior
        // Use balanced ships where kills are possible (higher weapons relative to
        // shields)
        let design1 = ShipDesign::new(1.0, 1, 3.0, 2.0, 0.0);
        let design2 = ShipDesign::new(1.0, 1, 3.0, 2.0, 0.0);

        let tech = Technology::new();

        let mut attacker_wins = 0;
        let mut defender_wins = 0;
        let mut both_survive = 0;
        let mut mutual_destruction = 0;

        for _ in 0..100 {
            let mut ship1 = Ship::new(ShipId(0), RaceId(0), design1, PlanetId(0));
            let mut ship2 = Ship::new(ShipId(1), RaceId(1), design2, PlanetId(1));

            let result = CombatSystem::resolve_combat(&mut ship1, &tech, &mut ship2, &tech);

            if result.attacker_survived && !result.defender_survived {
                attacker_wins += 1;
            } else if !result.attacker_survived && result.defender_survived {
                defender_wins += 1;
            } else if result.attacker_survived && result.defender_survived {
                both_survive += 1;
            } else {
                mutual_destruction += 1;
            }
        }

        // With equal ships and probabilistic combat, we should see varied outcomes
        // Due to randomness, at least 2 different outcomes should occur
        let total_outcomes = [
            attacker_wins,
            defender_wins,
            both_survive,
            mutual_destruction,
        ];
        let unique_outcomes = total_outcomes.iter().filter(|&&x| x > 0).count();
        assert!(
            unique_outcomes >= 2,
            "Combat should have varied outcomes. Got: attacker={}, defender={}, both={}, mutual={}",
            attacker_wins,
            defender_wins,
            both_survive,
            mutual_destruction
        );
    }

    #[test]
    fn test_combat_strong_vs_weak() {
        let strong_design = ShipDesign::new(2.0, 1, 10.0, 20.0, 0.0);
        let weak_design = ShipDesign::new(1.0, 1, 0.5, 1.0, 0.0);

        let tech = Technology::new();

        let mut strong_wins = 0;

        for _ in 0..20 {
            let mut strong = Ship::new(ShipId(0), RaceId(0), strong_design, PlanetId(0));
            let mut weak = Ship::new(ShipId(1), RaceId(1), weak_design, PlanetId(1));

            let result = CombatSystem::resolve_combat(&mut strong, &tech, &mut weak, &tech);

            if result.attacker_survived && !result.defender_survived {
                strong_wins += 1;
            }
        }

        // Strong ship should win most of the time
        assert!(strong_wins >= 15, "Strong ship should win most combats");
    }

    #[test]
    fn test_combat_with_technology() {
        let design = ShipDesign::new(2.0, 1, 2.0, 2.0, 0.0);

        let mut tech_high = Technology::new();
        tech_high.advance(crate::race::TechnologyType::Weapon);
        tech_high.advance(crate::race::TechnologyType::Weapon);

        let tech_low = Technology::new();

        let mut high_tech_wins = 0;

        for _ in 0..20 {
            let mut ship1 = Ship::new(ShipId(0), RaceId(0), design, PlanetId(0));
            let mut ship2 = Ship::new(ShipId(1), RaceId(1), design, PlanetId(1));

            let result =
                CombatSystem::resolve_combat(&mut ship1, &tech_high, &mut ship2, &tech_low);

            if result.attacker_survived && !result.defender_survived {
                high_tech_wins += 1;
            }
        }

        // Higher tech should win most of the time
        assert!(high_tech_wins >= 15, "Higher tech should win most combats");
    }

    #[test]
    fn test_combat_rounds_limit() {
        // Create ships that can't kill each other easily
        let design = ShipDesign::new(1.0, 1, 0.1, 100.0, 0.0);

        let tech = Technology::new();

        let mut ship1 = Ship::new(ShipId(0), RaceId(0), design, PlanetId(0));
        let mut ship2 = Ship::new(ShipId(1), RaceId(1), design, PlanetId(1));

        let result = CombatSystem::resolve_combat(&mut ship1, &tech, &mut ship2, &tech);

        // Should terminate even if no one dies
        assert!(result.rounds <= 100, "Combat should not exceed 100 rounds");
    }

    #[test]
    fn test_should_engage_different_races() {
        assert!(CombatSystem::should_engage(RaceId(0), RaceId(1)));
        assert!(!CombatSystem::should_engage(RaceId(0), RaceId(0)));
    }
}
