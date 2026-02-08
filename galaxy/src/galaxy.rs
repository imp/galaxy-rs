use std::collections::HashMap;

use bevy::prelude::*;

use crate::planet::Planet;
use crate::planet::PlanetId;
use crate::planet::Position;

/// The galaxy containing all planets
#[derive(Debug, Resource)]
pub struct Galaxy {
    planets: HashMap<PlanetId, Planet>,
    next_planet_id: u32,
    width: f64,
    height: f64,
}

impl Galaxy {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            planets: HashMap::new(),
            next_planet_id: 0,
            width,
            height,
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    /// Add a planet to the galaxy
    pub fn add_planet(&mut self, position: Position, size: u32, owner: Option<u32>) -> PlanetId {
        let id = PlanetId(self.next_planet_id);
        self.next_planet_id += 1;

        let planet = if let Some(owner_id) = owner {
            Planet::new_home_planet(id, position, size, owner_id)
        } else {
            Planet::new(id, position, size, owner)
        };
        self.planets.insert(id, planet);
        id
    }

    /// Get a planet by ID
    pub fn get_planet(&self, id: PlanetId) -> Option<&Planet> {
        self.planets.get(&id)
    }

    /// Get a mutable reference to a planet
    pub fn get_planet_mut(&mut self, id: PlanetId) -> Option<&mut Planet> {
        self.planets.get_mut(&id)
    }

    /// Get all planets
    pub fn planets(&self) -> impl Iterator<Item = &Planet> {
        self.planets.values()
    }

    /// Get all planets (mutable)
    pub fn planets_mut(&mut self) -> impl Iterator<Item = &mut Planet> {
        self.planets.values_mut()
    }

    /// Get planets owned by a specific race
    pub fn planets_owned_by(&self, race_id: u32) -> impl Iterator<Item = &Planet> {
        self.planets
            .values()
            .filter(move |p| p.owner() == Some(race_id))
    }

    /// Get uninhabited planets
    pub fn uninhabited_planets(&self) -> impl Iterator<Item = &Planet> {
        self.planets.values().filter(|p| p.owner().is_none())
    }

    /// Count planets owned by a race
    pub fn count_planets_owned_by(&self, race_id: u32) -> usize {
        self.planets_owned_by(race_id).count()
    }

    /// Process production for all planets
    pub fn execute_production(&mut self) {
        for planet in self.planets.values_mut() {
            planet.execute_production();
        }
    }
}
