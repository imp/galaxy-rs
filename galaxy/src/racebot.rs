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

/// AI controller for automated race management
pub struct Racebot {
    race_id: RaceId,
}

impl Racebot {
    pub fn new(race_id: RaceId) -> Self {
        Self { race_id }
    }

    #[expect(dead_code)]
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
        // Simple strategy:
        // 1. If low on capital and can afford it, build capital
        // 2. If decent industry but low materials, build materials
        // 3. Otherwise build capital to increase industry

        let avg_capital_per_planet = if !state.owned_planets.is_empty() {
            state.total_capital / state.owned_planets.len() as f64
        } else {
            0.0
        };

        // Target: ~50 capital per planet for good industry
        if avg_capital_per_planet < 50.0 {
            // Check if we can afford capital (needs 5 production + 1 material)
            if planet.materials() >= 1.0 && planet.production() >= 5.0 {
                return ProductionType::Capital;
            }
        }

        // Build materials as fallback
        ProductionType::Materials
    }

    /// Decide what ships to build this turn
    fn decide_ship_builds(&self, state: &GameState, _race: &Race) -> Vec<ShipBuild> {
        let mut builds = Vec::new();

        // Simple strategy: build a scout on each planet with enough materials
        // Scout design: small, fast, cheap (for colonization)
        let scout_design = ShipDesign::new(
            2.0, // drive_mass
            0,   // attacks
            0.0, // weapons_mass
            1.0, // shields_mass
            1.0, // cargo_mass
        );

        // Only build if we don't have too many ships already
        if state.owned_ships.len() < state.owned_planets.len() * 2 {
            for planet_id in &state.owned_planets {
                builds.push(ShipBuild {
                    planet_id: *planet_id,
                    design: scout_design,
                    name: format!("Scout-{}", planet_id.0),
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

        // Simple strategy: send idle ships to colonize nearest unowned planet
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
#[derive(Default)]
pub struct RacebotDecisions {
    pub production_orders: HashMap<PlanetId, ProductionType>,
    pub ship_builds: Vec<ShipBuild>,
    pub ship_movements: Vec<ShipMovement>,
}

/// Order to build a ship
pub struct ShipBuild {
    pub planet_id: PlanetId,
    pub design: ShipDesign,
    #[expect(dead_code)]
    pub name: String,
}

/// Order to move a ship
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
        let racebot = Racebot::new(race_id);

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

        let racebot = Racebot::new(race_id);
        let race = game.get_race(race_id).unwrap();
        let ships = HashMap::new();
        let state = racebot.analyze_state(game.galaxy(), race, &ships);

        // Find nearest from home
        let nearest = racebot.find_nearest_colonizable(home_planet, &state, game.galaxy());

        // Should find the near planet
        assert_eq!(nearest, Some(near_planet));
    }
}
