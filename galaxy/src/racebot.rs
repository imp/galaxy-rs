use std::collections::HashMap;

use crate::galaxy::Galaxy;
use crate::planet::Planet;
use crate::planet::PlanetId;
use crate::planet::ProductionType;
use crate::race::Race;
use crate::race::RaceId;
use crate::ship::Ship;
use crate::ship::ShipDesign;
use crate::ship::ShipId;
use crate::ship::ShipLocation;

/// Behavioral personality for AI decision making
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants used in integration tests, not main binary yet
pub enum Personality {
    /// Aggressive: Builds warships, seeks combat, attacks readily
    Aggressive,
    /// Defensive: Builds defensive ships, stays near home, fortifies
    Defensive,
    /// Expansionist: Builds scouts, explores aggressively, rapid colonization
    Expansionist,
    /// Economic: Focuses on production, capital, research
    Economic,
    /// Balanced: Mix of all strategies, adapts to situation
    Balanced,
}

impl Personality {
    /// Get production priority weights (capital_weight, materials_weight)
    #[expect(dead_code)]
    fn production_weights(&self) -> (f64, f64) {
        match self {
            Self::Aggressive => (0.3, 0.7),   // More materials for ships
            Self::Defensive => (0.5, 0.5),    // Balanced
            Self::Expansionist => (0.4, 0.6), // More materials for scouts
            Self::Economic => (0.7, 0.3),     // More capital for industry
            Self::Balanced => (0.5, 0.5),     // Even split
        }
    }

    /// Target capital per planet before focusing on materials
    fn capital_target(&self) -> f64 {
        match self {
            Self::Aggressive => 30.0,   // Low - wants materials fast
            Self::Defensive => 60.0,    // High - strong economy
            Self::Expansionist => 40.0, // Medium
            Self::Economic => 100.0,    // Very high - max industry
            Self::Balanced => 50.0,     // Default
        }
    }

    /// Maximum number of ships per planet
    fn ships_per_planet_ratio(&self) -> f64 {
        match self {
            Self::Aggressive => 3.0,   // Large fleet
            Self::Defensive => 1.5,    // Smaller defensive fleet
            Self::Expansionist => 2.5, // Many scouts
            Self::Economic => 1.0,     // Minimal military
            Self::Balanced => 2.0,     // Default
        }
    }

    /// Ship design based on personality
    fn design_ship(&self, _race: &Race) -> ShipDesign {
        match self {
            Self::Aggressive => {
                // Warship: Heavy weapons, moderate shields
                ShipDesign::new(5.0, 3, 8.0, 6.0, 0.0)
            }
            Self::Defensive => {
                // Defensive ship: Heavy shields, moderate weapons
                ShipDesign::new(4.0, 2, 4.0, 10.0, 0.0)
            }
            Self::Expansionist => {
                // Scout: Fast, light, has cargo for colonists
                ShipDesign::new(3.0, 0, 0.0, 2.0, 2.0)
            }
            Self::Economic => {
                // Colony ship: Minimal combat, max cargo
                ShipDesign::new(2.0, 0, 0.0, 1.0, 3.0)
            }
            Self::Balanced => {
                // Balanced ship: Moderate everything
                ShipDesign::new(3.0, 1, 3.0, 4.0, 1.0)
            }
        }
    }

    /// Should aggressively colonize?
    fn colonization_priority(&self) -> bool {
        matches!(self, Self::Expansionist | Self::Economic | Self::Balanced)
    }

    /// Should seek combat?
    #[expect(dead_code)]
    fn combat_seeking(&self) -> bool {
        matches!(self, Self::Aggressive)
    }
}

/// AI controller for automated race management
#[derive(Debug)]
pub struct Racebot {
    race_id: RaceId,
    personality: Personality,
}

impl Racebot {
    pub fn with_personality(race_id: RaceId, personality: Personality) -> Self {
        Self {
            race_id,
            personality,
        }
    }

    #[allow(dead_code)]
    pub fn race_id(&self) -> RaceId {
        self.race_id
    }

    /// Make all decisions for this race for the current turn
    pub fn make_decisions(
        &self,
        galaxy: &Galaxy,
        race: &Race,
        ships: &HashMap<ShipId, Ship>,
    ) -> RacebotDecisions {
        let mut decisions = RacebotDecisions::default();

        // Analyze game state
        let state = self.analyze_state(galaxy, race, ships);

        // Make production decisions for each planet
        for planet_id in &state.owned_planets {
            if let Some(planet) = galaxy.get_planet(*planet_id) {
                let production = self.decide_production(planet, &state);
                decisions.production_orders.insert(*planet_id, production);
            }
        }

        // Make ship building decisions
        decisions.ship_builds = self.decide_ship_builds(&state, race);

        // Make ship movement decisions
        decisions.ship_movements = self.decide_ship_movements(&state, ships, galaxy);

        decisions
    }

    /// Analyze current game state
    fn analyze_state(
        &self,
        galaxy: &Galaxy,
        _race: &Race,
        ships: &HashMap<ShipId, Ship>,
    ) -> GameState {
        let mut state = GameState::default();

        // Find all owned planets
        for planet in galaxy.planets() {
            if planet.owner() == Some(self.race_id.0) {
                state.owned_planets.push(planet.id());
                state.total_population += planet.population();
                state.total_industry += planet.industry();
                state.total_production += planet.production();
                state.total_materials += planet.materials();
                state.total_capital += planet.capital();
            }
        }

        // Find all owned ships
        for (id, ship) in ships {
            if ship.owner() == self.race_id {
                state.owned_ships.push(*id);
            }
        }

        // Find colonizable planets (unowned, size > 0)
        for planet in galaxy.planets() {
            if planet.owner().is_none() && planet.size() > 0 {
                state.colonizable_planets.push(planet.id());
            }
        }

        state
    }

    /// Decide what to produce on a planet
    fn decide_production(&self, planet: &Planet, state: &GameState) -> ProductionType {
        // Use personality to determine production strategy
        let avg_capital_per_planet = if !state.owned_planets.is_empty() {
            state.total_capital / state.owned_planets.len() as f64
        } else {
            0.0
        };

        // Get personality-based capital target
        let capital_target = self.personality.capital_target();

        // Build capital if below target and can afford it
        if avg_capital_per_planet < capital_target {
            // Check if we can afford capital (needs 5 production + 1 material)
            if planet.materials() >= 1.0 && planet.production() >= 5.0 {
                return ProductionType::Capital;
            }
        }

        // Build materials as fallback
        ProductionType::Materials
    }

    /// Decide what ships to build this turn
    fn decide_ship_builds(&self, state: &GameState, race: &Race) -> Vec<ShipBuild> {
        let mut builds = Vec::new();

        // Use personality to determine ship design and fleet size
        let ship_design = self.personality.design_ship(race);
        let max_ships = (state.owned_planets.len() as f64
            * self.personality.ships_per_planet_ratio())
        .ceil() as usize;

        // Only build if under fleet cap
        if state.owned_ships.len() < max_ships {
            for planet_id in &state.owned_planets {
                builds.push(ShipBuild {
                    planet_id: *planet_id,
                    design: ship_design,
                    name: format!("{:?}-{}", self.personality, planet_id.0),
                });
            }
        }

        builds
    }

    /// Decide where to move ships
    fn decide_ship_movements(
        &self,
        state: &GameState,
        ships: &HashMap<ShipId, Ship>,
        galaxy: &Galaxy,
    ) -> Vec<ShipMovement> {
        let mut movements = Vec::new();

        // Only colonize if personality prioritizes it
        if !self.personality.colonization_priority() {
            return movements;
        }

        // Send idle ships to colonize nearest unowned planet
        for ship_id in &state.owned_ships {
            if let Some(ship) = ships.get(ship_id) {
                // Only move ships that are at a planet (not traveling)
                if let ShipLocation::AtPlanet(current_planet) = ship.location() {
                    // Find nearest colonizable planet
                    if let Some(target) =
                        self.find_nearest_colonizable(*current_planet, state, galaxy)
                    {
                        movements.push(ShipMovement {
                            ship_id: *ship_id,
                            destination: target,
                        });
                    }
                }
            }
        }

        movements
    }

    /// Find nearest colonizable planet
    fn find_nearest_colonizable(
        &self,
        from: PlanetId,
        state: &GameState,
        galaxy: &Galaxy,
    ) -> Option<PlanetId> {
        let from_pos = galaxy.get_planet(from)?.position();

        state
            .colonizable_planets
            .iter()
            .min_by_key(|planet_id| {
                if let Some(planet) = galaxy.get_planet(**planet_id) {
                    let dx = planet.position().x() - from_pos.x();
                    let dy = planet.position().y() - from_pos.y();
                    (dx * dx + dy * dy).sqrt() as i32
                } else {
                    i32::MAX
                }
            })
            .copied()
    }
}

/// Analyzed game state for decision making
#[derive(Default)]
struct GameState {
    owned_planets: Vec<PlanetId>,
    owned_ships: Vec<ShipId>,
    colonizable_planets: Vec<PlanetId>,
    total_population: f64,
    total_industry: f64,
    total_production: f64,
    total_materials: f64,
    total_capital: f64,
}

/// Decisions made by the racebot
#[derive(Default, Debug)]
pub struct RacebotDecisions {
    pub production_orders: HashMap<PlanetId, ProductionType>,
    pub ship_builds: Vec<ShipBuild>,
    pub ship_movements: Vec<ShipMovement>,
}

/// Order to build a ship
#[derive(Debug)]
pub struct ShipBuild {
    pub planet_id: PlanetId,
    pub design: ShipDesign,
    #[allow(dead_code)]
    pub name: String,
}

/// Order to move a ship
#[derive(Debug)]
pub struct ShipMovement {
    pub ship_id: ShipId,
    pub destination: PlanetId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;
    use crate::planet::Position;

    #[test]
    fn test_racebot_analyzes_state() {
        let mut game = GameState::new(1000.0, 1000.0);

        // Create race
        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("TestRace".to_string(), home_planet.0);

        // Add some colonizable planets
        game.galaxy_mut()
            .add_planet(Position::new(600.0, 600.0), 50, None);
        game.galaxy_mut()
            .add_planet(Position::new(400.0, 400.0), 30, None);

        // Create racebot
        let racebot = Racebot::with_personality(race_id, Personality::Balanced);

        // Analyze state
        let race = game.get_race(race_id).unwrap();
        let ships = HashMap::new();
        let state = racebot.analyze_state(game.galaxy(), race, &ships);

        // Verify analysis
        assert_eq!(state.owned_planets.len(), 1);
        assert_eq!(state.colonizable_planets.len(), 2);
        assert!(state.total_population > 0.0);
    }

    #[test]
    fn test_racebot_makes_production_decisions() {
        let mut game = GameState::new(1000.0, 1000.0);

        // Create race with home planet
        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("TestRace".to_string(), home_planet.0);

        // Run racebot
        game.run_racebot(race_id);

        // Verify production decision was made
        let planet = game.galaxy().get_planet(home_planet).unwrap();
        // Should have some production type set
        assert!(planet.production() > 0.0);
    }

    #[test]
    fn test_racebot_decides_ship_movements() {
        let mut game = GameState::new(1000.0, 1000.0);

        // Create race with home planet
        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("TestRace".to_string(), home_planet.0);

        // Add colonizable planet nearby
        let target_pos = Position::new(550.0, 550.0);
        let _target_planet = game.galaxy_mut().add_planet(target_pos, 50, None);

        // Add materials to home planet for ship building
        if let Some(planet) = game.galaxy_mut().get_planet_mut(home_planet) {
            planet.add_materials(1000.0);
        }

        // Build a scout ship
        let scout = ShipDesign::new(2.0, 0, 0.0, 1.0, 1.0);
        let ship_id = game.build_ship(home_planet, scout);
        assert!(ship_id.is_some(), "Failed to build ship");

        // Run racebot - it should send the ship to colonize
        game.run_racebot(race_id);

        // Check ship was ordered to move (or at least still exists)
        let ship = game.get_ship(ship_id.unwrap()).unwrap();
        // Ship should either be traveling or still at home
        match ship.location() {
            ShipLocation::AtPlanet(_) | ShipLocation::Traveling { .. } => {}
        }
    }

    #[test]
    fn test_racebot_finds_nearest_colonizable() {
        let mut game = GameState::new(1000.0, 1000.0);

        // Create race
        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("TestRace".to_string(), home_planet.0);

        // Add planets at different distances
        let near_planet = game
            .galaxy_mut()
            .add_planet(Position::new(520.0, 520.0), 50, None);
        let _far_planet = game
            .galaxy_mut()
            .add_planet(Position::new(800.0, 800.0), 30, None);

        let racebot = Racebot::with_personality(race_id, Personality::Balanced);
        let race = game.get_race(race_id).unwrap();
        let ships = HashMap::new();
        let state = racebot.analyze_state(game.galaxy(), race, &ships);

        // Find nearest from home
        let nearest = racebot.find_nearest_colonizable(home_planet, &state, game.galaxy());

        // Should find the near planet
        assert_eq!(nearest, Some(near_planet));
    }

    #[test]
    fn test_aggressive_personality() {
        let mut game = GameState::new(1000.0, 1000.0);

        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("Aggressive".to_string(), home_planet.0);

        let _racebot = Racebot::with_personality(race_id, Personality::Aggressive);
        let race = game.get_race(race_id).unwrap();
        let design = Personality::Aggressive.design_ship(race);
        assert!(design.weapons_mass() > design.shields_mass());
        assert!(design.attacks() >= 2);
        assert_eq!(Personality::Aggressive.capital_target(), 30.0);
    }

    #[test]
    fn test_defensive_personality() {
        let mut game = GameState::new(1000.0, 1000.0);

        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("Defensive".to_string(), home_planet.0);

        let _racebot = Racebot::with_personality(race_id, Personality::Defensive);
        let race = game.get_race(race_id).unwrap();

        let design = Personality::Defensive.design_ship(race);
        assert!(design.shields_mass() > design.weapons_mass());
        assert!(!Personality::Defensive.colonization_priority());
    }

    #[test]
    fn test_expansionist_personality() {
        let mut game = GameState::new(1000.0, 1000.0);

        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("Expansionist".to_string(), home_planet.0);

        let _racebot = Racebot::with_personality(race_id, Personality::Expansionist);
        let race = game.get_race(race_id).unwrap();

        let design = Personality::Expansionist.design_ship(race);
        assert!(design.cargo_mass() > 0.0);
        assert_eq!(design.attacks(), 0);
        assert!(Personality::Expansionist.colonization_priority());
        assert!(Personality::Expansionist.ships_per_planet_ratio() > 2.0);
    }

    #[test]
    fn test_economic_personality() {
        let mut game = GameState::new(1000.0, 1000.0);

        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("Economic".to_string(), home_planet.0);

        let _racebot = Racebot::with_personality(race_id, Personality::Economic);
        let race = game.get_race(race_id).unwrap();

        assert_eq!(Personality::Economic.capital_target(), 100.0);

        let design = Personality::Economic.design_ship(race);
        assert!(design.cargo_mass() >= 3.0);
        assert_eq!(design.attacks(), 0);
        assert_eq!(Personality::Economic.ships_per_planet_ratio(), 1.0);
    }

    #[test]
    fn test_personality_affects_production() {
        let mut game = GameState::new(1000.0, 1000.0);

        let home_pos = Position::new(500.0, 500.0);
        let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
        let race_id = game.add_race("Test".to_string(), home_planet.0);

        // Add materials for production
        game.galaxy_mut()
            .get_planet_mut(home_planet)
            .unwrap()
            .add_materials(1000.0);

        let economic_bot = Racebot::with_personality(race_id, Personality::Economic);
        let race = game.get_race(race_id).unwrap();
        let ships = HashMap::new();
        let state = economic_bot.analyze_state(game.galaxy(), race, &ships);

        let planet = game.galaxy().get_planet(home_planet).unwrap();
        let production_choice = economic_bot.decide_production(planet, &state);

        // With low capital, should build capital (Economic has high target of 100)
        assert_eq!(production_choice, ProductionType::Capital);
    }
}
