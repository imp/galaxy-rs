use crate::race::RaceId;
use crate::planet::PlanetId;
use std::fmt;

/// Unique identifier for a ship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShipId(pub u32);

impl fmt::Display for ShipId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ship{}", self.0)
    }
}

/// Ship design specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipDesign {
    pub hull_strength: u32,
    pub engine_power: u32,
    pub cannon_count: u32,
    pub cannon_power: u32,
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

    /// Calculate total material cost for this ship design
    pub fn material_cost(&self) -> f64 {
        let hull_cost = self.hull_strength as f64 * 2.0;
        let engine_cost = self.engine_power as f64 * 3.0;
        let weapon_cost = (self.cannon_count * self.cannon_power) as f64 * 1.5;
        hull_cost + engine_cost + weapon_cost
    }

    /// Apply race technology bonuses to ship design
    #[allow(dead_code)]
    pub fn with_tech_bonus(hull: u32, engine: u32, cannons: u32, 
                          drive_tech: u32, weapon_tech: u32, shield_tech: u32) -> Self {
        Self {
            hull_strength: hull * shield_tech,
            engine_power: engine * drive_tech,
            cannon_count: cannons,
            cannon_power: weapon_tech,
        }
    }
}

/// A spaceship
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Ship {
    pub id: ShipId,
    pub owner: RaceId,
    pub design: ShipDesign,
    pub current_hull: u32,
    pub location: ShipLocation,
}

impl Ship {
    pub fn new(id: ShipId, owner: RaceId, design: ShipDesign, location: PlanetId) -> Self {
        Self {
            id,
            owner,
            current_hull: design.hull_strength,
            design,
            location: ShipLocation::AtPlanet(location),
        }
    }

    /// Check if ship is destroyed
    #[allow(dead_code)]
    pub fn is_destroyed(&self) -> bool {
        self.current_hull == 0
    }

    /// Take damage to the ship
    #[allow(dead_code)]
    pub fn take_damage(&mut self, damage: u32) {
        self.current_hull = self.current_hull.saturating_sub(damage);
    }

    /// Calculate travel speed based on engine power
    #[allow(dead_code)]
    pub fn travel_speed(&self) -> f64 {
        self.design.engine_power as f64 * 10.0
    }

    /// Calculate attack power
    #[allow(dead_code)]
    pub fn attack_power(&self) -> u32 {
        self.design.cannon_count * self.design.cannon_power
    }
}

/// Ship location - either at a planet or traveling
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ShipLocation {
    AtPlanet(PlanetId),
    Traveling { from: PlanetId, to: PlanetId, progress: f64 },
}
