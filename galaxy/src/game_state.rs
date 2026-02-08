use std::collections::HashMap;

use bevy::prelude::*;

use crate::combat::CombatSystem;
use crate::diplomacy::Diplomacy;
use crate::galaxy::Galaxy;
use crate::planet::PlanetId;
use crate::planet::TechFocus;
use crate::race::Race;
use crate::race::RaceId;
use crate::race::TechnologyType;
use crate::racebot::Personality;
use crate::racebot::Racebot;
use crate::ship::Ship;
use crate::ship::ShipDesign;
use crate::ship::ShipId;
use crate::ship::ShipLocation;

/// The main game state
#[derive(Debug, Resource)]
pub struct GameState {
    galaxy: Galaxy,
    races: HashMap<RaceId, Race>,
    ships: HashMap<ShipId, Ship>,
    diplomacy: Diplomacy,
    ai_personalities: HashMap<RaceId, Personality>,
    next_race_id: u32,
    next_ship_id: u32,
    turn: u32,
}

impl GameState {
    pub fn new(galaxy_width: f64, galaxy_height: f64) -> Self {
        Self {
            galaxy: Galaxy::new(galaxy_width, galaxy_height),
            races: HashMap::new(),
            ships: HashMap::new(),
            diplomacy: Diplomacy::new(),
            ai_personalities: HashMap::new(),
            next_race_id: 0,
            next_ship_id: 0,
            turn: 0,
        }
    }

    pub fn galaxy(&self) -> &Galaxy {
        &self.galaxy
    }

    pub fn galaxy_mut(&mut self) -> &mut Galaxy {
        &mut self.galaxy
    }

    pub fn turn(&self) -> u32 {
        self.turn
    }

    pub fn diplomacy(&self) -> &Diplomacy {
        &self.diplomacy
    }

    pub fn diplomacy_mut(&mut self) -> &mut Diplomacy {
        &mut self.diplomacy
    }

    /// Add a new race to the game
    pub fn add_race(&mut self, name: String, home_planet_id: u32) -> RaceId {
        let id = RaceId(self.next_race_id);
        self.next_race_id += 1;

        let race = Race::new(id, name, home_planet_id);
        self.races.insert(id, race);
        id
    }

    /// Add a new AI-controlled race with specified personality
    pub fn add_ai_race(
        &mut self,
        name: String,
        home_planet_id: u32,
        personality: Personality,
    ) -> RaceId {
        let id = RaceId(self.next_race_id);
        self.next_race_id += 1;

        let race = Race::new_ai(id, name, home_planet_id);
        self.races.insert(id, race);
        self.ai_personalities.insert(id, personality);
        id
    }

    /// Get a race by ID
    pub fn get_race(&self, id: RaceId) -> Option<&Race> {
        self.races.get(&id)
    }

    /// Get a mutable reference to a race
    pub fn get_race_mut(&mut self, id: RaceId) -> Option<&mut Race> {
        self.races.get_mut(&id)
    }

    /// Get all races
    pub fn races(&self) -> impl Iterator<Item = &Race> {
        self.races.values()
    }

    /// Process one turn of the game
    pub fn advance_turn(&mut self) {
        self.turn += 1;

        // 0. Process AI decisions for all AI-controlled races
        self.process_ai_turns();

        // 1. Execute production on all planets
        self.galaxy.execute_production();

        // 2. Process technology advancement per planet
        self.process_technology_advancement();

        // 3. Process ship movement
        self.process_ship_movement();

        // 4. Process combat encounters
        self.process_combat();

        // 5. Grow population on all planets
        self.process_population_growth();
    }

    fn process_population_growth(&mut self) {
        for planet in self.galaxy.planets_mut() {
            planet.grow_population();
        }
    }

    fn process_technology_advancement(&mut self) {
        // Collect planet data first to avoid borrow checker issues
        let planet_research: Vec<(u32, TechnologyType, u32)> = self
            .galaxy
            .planets()
            .filter(|p| p.owner().is_some())
            .filter_map(|p| match p.tech_focus() {
                TechFocus::Research(tech_type) => Some((p.owner().unwrap(), tech_type, p.size())),
                TechFocus::None => None,
            })
            .collect();

        // Apply research to races
        for (race_id, tech_type, planet_size) in planet_research {
            if let Some(race) = self.get_race_mut(RaceId(race_id)) {
                let effort = planet_size as f64;
                race.add_research(tech_type, effort);
            }
        }
    }

    /// Set technology focus for a planet
    pub fn set_planet_tech_focus(&mut self, planet_id: PlanetId, focus: TechFocus) {
        if let Some(planet) = self.galaxy.get_planet_mut(planet_id) {
            planet.set_tech_focus(focus);
        }
    }

    /// Check victory conditions - returns the winning race if any
    pub fn check_victory(&self) -> Option<RaceId> {
        let mut max_planets = 0;
        let mut winner = None;

        for race in self.races.values() {
            let planet_count = self.galaxy.count_planets_owned_by(race.id().0);
            if planet_count > max_planets {
                max_planets = planet_count;
                winner = Some(race.id());
            }
        }

        // Only return winner if they have significantly more planets
        // (You could add more sophisticated victory conditions)
        winner
    }

    /// Run the game simulation for a maximum number of turns
    /// Returns the winning race or None if no victory by max_turns
    pub fn run_simulation(&mut self, max_turns: u32) -> Option<RaceId> {
        for _ in 0..max_turns {
            self.advance_turn();

            // Check for victory (could add more sophisticated checks)
            if let Some(winner) = self.check_victory() {
                let winner_planets = self.galaxy.count_planets_owned_by(winner.0);
                let total_planets = self.galaxy.planets().count();

                // Win if you control majority of planets
                if winner_planets > total_planets / 2 {
                    return Some(winner);
                }
            }
        }

        None // No winner within max_turns
    }

    /// Build a ship at a planet
    pub fn build_ship(&mut self, planet_id: PlanetId, design: ShipDesign) -> Option<ShipId> {
        let planet = self.galaxy.get_planet(planet_id)?;
        let owner_id = planet.owner()?;

        let cost = design.material_cost();

        // Check if planet has enough materials
        let planet = self.galaxy.get_planet_mut(planet_id)?;
        if planet.consume_materials(cost) {
            let ship_id = ShipId(self.next_ship_id);
            self.next_ship_id += 1;

            let ship = Ship::new(ship_id, RaceId(owner_id), design, planet_id);
            self.ships.insert(ship_id, ship);

            return Some(ship_id);
        }

        None
    }

    /// Get a ship by ID
    pub fn get_ship(&self, id: ShipId) -> Option<&Ship> {
        self.ships.get(&id)
    }

    /// Get all ships
    pub fn ships(&self) -> impl Iterator<Item = &Ship> {
        self.ships.values()
    }

    /// Order a ship to travel to a destination planet
    pub fn order_ship_travel(&mut self, ship_id: ShipId, destination: PlanetId) -> bool {
        let ship = match self.ships.get_mut(&ship_id) {
            Some(s) => s,
            None => return false,
        };

        // Get current location
        let origin = match ship.location() {
            ShipLocation::AtPlanet(planet_id) => *planet_id,
            ShipLocation::Traveling { .. } => return false, // Already traveling
        };

        // Can't travel to same planet
        if origin == destination {
            return false;
        }

        // Verify destination exists
        if self.galaxy.get_planet(destination).is_none() {
            return false;
        }

        // Start travel
        ship.set_location(ShipLocation::Traveling {
            from: origin,
            to: destination,
            progress: 0.0,
        });

        true
    }

    fn process_ship_movement(&mut self) {
        // Collect ship movements to process
        let movements: Vec<(ShipId, PlanetId, PlanetId, f64, f64)> = self
            .ships
            .iter()
            .filter_map(|(id, ship)| {
                if let ShipLocation::Traveling { from, to, progress } = ship.location() {
                    // Calculate distance between planets
                    let from_planet = self.galaxy.get_planet(*from)?;
                    let to_planet = self.galaxy.get_planet(*to)?;
                    let distance = from_planet.position().distance_to(to_planet.position());
                    Some((*id, *from, *to, *progress, distance))
                } else {
                    None
                }
            })
            .collect();

        for (ship_id, from, to, progress, distance) in movements {
            if let Some(ship) = self.ships.get_mut(&ship_id) {
                // Get drive technology for this ship's owner
                let drive_tech = self
                    .races
                    .get(&ship.owner())
                    .map_or(1.0, |r| r.technology().drive_level() as f64);

                let speed = ship.travel_speed(drive_tech);
                let new_progress = progress + (speed / distance.max(1.0));

                if new_progress >= 1.0 {
                    // Ship arrived
                    ship.set_location(ShipLocation::AtPlanet(to));

                    // Check if planet is uninhabited and colonize it
                    if let Some(planet) = self.galaxy.get_planet_mut(to)
                        && planet.owner().is_none()
                    {
                        planet.set_owner(Some(ship.owner().0));
                    }
                } else {
                    // Continue traveling
                    ship.set_location(ShipLocation::Traveling {
                        from,
                        to,
                        progress: new_progress,
                    });
                }
            }
        }
    }

    fn process_combat(&mut self) {
        // Find ships at the same planet that should fight
        let mut combat_pairs: Vec<(ShipId, ShipId)> = Vec::new();

        // Group ships by planet
        let mut ships_at_planets: HashMap<PlanetId, Vec<(ShipId, RaceId)>> = HashMap::new();

        for (ship_id, ship) in &self.ships {
            if let ShipLocation::AtPlanet(planet_id) = ship.location() {
                ships_at_planets
                    .entry(*planet_id)
                    .or_default()
                    .push((*ship_id, ship.owner()));
            }
        }

        // Find hostile pairs
        for ships in ships_at_planets.values() {
            for i in 0..ships.len() {
                for j in (i + 1)..ships.len() {
                    let (ship1_id, race1) = ships[i];
                    let (ship2_id, race2) = ships[j];

                    if self.diplomacy.should_attack(race1, race2) {
                        combat_pairs.push((ship1_id, ship2_id));
                        // Mark races as hostile if they weren't already
                        self.diplomacy.make_hostile(race1, race2);
                    }
                }
            }
        }

        // Process combat
        let mut ships_to_remove = Vec::new();

        for (ship1_id, ship2_id) in combat_pairs {
            // Skip if either ship was already destroyed in a previous combat
            if !self.ships.contains_key(&ship1_id) || !self.ships.contains_key(&ship2_id) {
                continue;
            }

            // Remove both ships temporarily to get mutable access
            let mut ship1 = self.ships.remove(&ship1_id).unwrap();
            let mut ship2 = self.ships.remove(&ship2_id).unwrap();

            // Get technology for both races
            let default_tech = crate::race::Technology::new();
            let ship1_tech = self
                .races
                .get(&ship1.owner())
                .map_or(&default_tech, |r| r.technology());

            let ship2_tech = self
                .races
                .get(&ship2.owner())
                .map_or(&default_tech, |r| r.technology());

            // Resolve combat
            let result =
                CombatSystem::resolve_combat(&mut ship1, ship1_tech, &mut ship2, ship2_tech);

            // Put survivors back
            if result.attacker_survived {
                self.ships.insert(ship1_id, ship1);
            } else {
                ships_to_remove.push(ship1_id);
            }

            if result.defender_survived {
                self.ships.insert(ship2_id, ship2);
            } else {
                ships_to_remove.push(ship2_id);
            }
        }
    }

    /// Execute racebot decisions
    pub fn execute_racebot_decisions(
        &mut self,
        race_id: RaceId,
        decisions: crate::racebot::RacebotDecisions,
    ) {
        // Apply production orders
        for (planet_id, production_type) in decisions.production_orders {
            if let Some(planet) = self.galaxy.get_planet_mut(planet_id)
                && planet.owner() == Some(race_id.0)
            {
                planet.set_production_type(production_type);
            }
        }

        // Build ships
        for ship_build in decisions.ship_builds {
            self.build_ship(ship_build.planet_id, ship_build.design);
        }

        // Move ships
        for ship_movement in decisions.ship_movements {
            self.order_ship_travel(ship_movement.ship_id, ship_movement.destination);
        }
    }

    /// Run racebot for a specific race
    pub fn run_racebot(&mut self, race_id: RaceId) {
        // Get personality if stored, otherwise use Balanced
        let personality = self
            .ai_personalities
            .get(&race_id)
            .copied()
            .unwrap_or(Personality::Balanced);

        // Create racebot with appropriate personality
        let racebot = Racebot::with_personality(race_id, personality);

        // Get race reference
        let race = match self.races.get(&race_id) {
            Some(r) => r,
            None => return,
        };

        // Make decisions (immutable borrows)
        let decisions = racebot.make_decisions(&self.galaxy, race, &self.ships);

        // Execute decisions (mutable borrows)
        self.execute_racebot_decisions(race_id, decisions);
    }

    /// Process AI turns for all AI-controlled races
    fn process_ai_turns(&mut self) {
        // Collect AI race IDs first (to avoid borrow checker issues)
        let ai_races: Vec<RaceId> = self
            .races
            .values()
            .filter(|r| r.is_ai_controlled())
            .map(|r| r.id())
            .collect();

        // Run racebot for each AI race
        for race_id in ai_races {
            self.run_racebot(race_id);
        }
    }
}
