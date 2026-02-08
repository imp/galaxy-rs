use std::collections::HashMap;

use bevy::prelude::*;

use crate::galaxy::Galaxy;
use crate::planet::PlanetId;
use crate::planet::TechFocus;
use crate::race::Race;
use crate::race::RaceId;
use crate::race::TechnologyType;
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

    /// Add a new race to the game
    pub fn add_race(&mut self, name: String, home_planet_id: u32) -> RaceId {
        let id = RaceId(self.next_race_id);
        self.next_race_id += 1;

        let race = Race::new(id, name, home_planet_id);
        self.races.insert(id, race);
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
    #[allow(dead_code)]
    pub fn races(&self) -> impl Iterator<Item = &Race> {
        self.races.values()
    }

    /// Process one turn of the game
    pub fn advance_turn(&mut self) {
        self.turn += 1;

        // 1. Produce materials on all planets
        self.galaxy.produce_materials();

        // 2. Process technology advancement per planet
        self.process_technology_advancement();

        // 3. Process ship movement
        self.process_ship_movement();
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
                let speed = ship.travel_speed();
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
}
