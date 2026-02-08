use bevy::prelude::*;

use crate::planet::PlanetId;
use crate::race::RaceId;

/// Unique identifier for a ship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct ShipId(pub u32);

impl std::fmt::Display for ShipId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ship{}", self.0)
    }
}

/// Ship design specification
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct ShipDesign {
    hull_strength: u32,
    engine_power: u32,
    cannon_count: u32,
    cannon_power: u32,
}

impl ShipDesign {
    pub fn new(hull: u32, engine: u32, cannon_count: u32, cannon_power: u32) -> Self {
        Self {
            hull_strength: hull,
            engine_power: engine,
            cannon_count,
            cannon_power,
        }
    }

    pub fn hull_strength(&self) -> u32 {
        self.hull_strength
    }

    pub fn engine_power(&self) -> u32 {
        self.engine_power
    }

    pub fn cannon_count(&self) -> u32 {
        self.cannon_count
    }

    pub fn cannon_power(&self) -> u32 {
        self.cannon_power
    }

    /// Calculate total material cost for this ship design
    pub fn material_cost(&self) -> f64 {
        let hull_cost = self.hull_strength as f64 * 2.0;
        let engine_cost = self.engine_power as f64 * 3.0;
        let weapon_cost = (self.cannon_count * self.cannon_power) as f64 * 1.5;
        hull_cost + engine_cost + weapon_cost
    }

    /// Apply race technology bonuses to ship design
    #[allow(dead_code)]
    pub fn with_tech_bonus(
        hull: u32,
        engine: u32,
        cannons: u32,
        drive_tech: u32,
        weapon_tech: u32,
        shield_tech: u32,
    ) -> Self {
        Self {
            hull_strength: hull * shield_tech,
            engine_power: engine * drive_tech,
            cannon_count: cannons,
            cannon_power: weapon_tech,
        }
    }
}

/// A spaceship
#[derive(Debug, Clone, Component)]
pub struct Ship {
    id: ShipId,
    owner: RaceId,
    design: ShipDesign,
    current_hull: u32,
    location: ShipLocation,
}

impl Ship {
    pub fn new(id: ShipId, owner: RaceId, design: ShipDesign, location: PlanetId) -> Self {
        Self {
            id,
            owner,
            current_hull: design.hull_strength(),
            design,
            location: ShipLocation::AtPlanet(location),
        }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> ShipId {
        self.id
    }

    pub fn owner(&self) -> RaceId {
        self.owner
    }

    pub fn design(&self) -> &ShipDesign {
        &self.design
    }

    #[allow(dead_code)]
    pub fn current_hull(&self) -> u32 {
        self.current_hull
    }

    pub fn location(&self) -> &ShipLocation {
        &self.location
    }

    pub fn set_location(&mut self, location: ShipLocation) {
        self.location = location;
    }

    /// Check if ship is destroyed
    pub fn is_destroyed(&self) -> bool {
        self.current_hull == 0
    }

    /// Take damage to the ship
    pub fn take_damage(&mut self, damage: u32) {
        self.current_hull = self.current_hull.saturating_sub(damage);
    }

    /// Calculate travel speed based on engine power
    pub fn travel_speed(&self) -> f64 {
        self.design.engine_power() as f64 * 10.0
    }

    /// Calculate attack power
    pub fn attack_power(&self) -> u32 {
        self.design.cannon_count() * self.design.cannon_power()
    }
}

/// Ship location - either at a planet or traveling
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum ShipLocation {
    AtPlanet(PlanetId),
    Traveling {
        from: PlanetId,
        to: PlanetId,
        progress: f64,
    },
}
