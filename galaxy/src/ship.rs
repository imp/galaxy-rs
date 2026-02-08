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

/// Ship design specification (GalaxyNG format)
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct ShipDesign {
    drive_mass: f64,
    attacks: u32,
    weapons_mass: f64,
    shields_mass: f64,
    cargo_mass: f64,
}

#[allow(dead_code)]
impl ShipDesign {
    pub fn new(
        drive_mass: f64,
        attacks: u32,
        weapons_mass: f64,
        shields_mass: f64,
        cargo_mass: f64,
    ) -> Self {
        Self {
            drive_mass,
            attacks,
            weapons_mass,
            shields_mass,
            cargo_mass,
        }
    }

    pub fn drive_mass(&self) -> f64 {
        self.drive_mass
    }

    pub fn attacks(&self) -> u32 {
        self.attacks
    }

    pub fn weapons_mass(&self) -> f64 {
        self.weapons_mass
    }

    pub fn shields_mass(&self) -> f64 {
        self.shields_mass
    }

    pub fn cargo_mass(&self) -> f64 {
        self.cargo_mass
    }

    /// Calculate ship mass: D + W + S + C + (attacks-1) × W/2
    pub fn ship_mass(&self) -> f64 {
        let base = self.drive_mass + self.weapons_mass + self.shields_mass + self.cargo_mass;
        let additional_attacks = if self.attacks > 1 {
            (self.attacks - 1) as f64 * self.weapons_mass / 2.0
        } else {
            0.0
        };
        base + additional_attacks
    }

    /// Calculate material cost: ship mass × 1 (1 material per mass unit)
    pub fn material_cost(&self) -> f64 {
        self.ship_mass()
    }

    /// Calculate speed: 20 × drive_tech × (drive_mass / (ship_mass + cargo))
    pub fn speed(&self, drive_tech: f64, cargo_carried: f64) -> f64 {
        if self.drive_mass == 0.0 {
            return 0.0; // Immobile ships
        }
        20.0 * drive_tech * (self.drive_mass / (self.ship_mass() + cargo_carried))
    }

    /// Calculate attack strength: weapons_mass × weapons_tech
    pub fn attack_strength(&self, weapons_tech: f64) -> f64 {
        self.weapons_mass * weapons_tech
    }

    /// Calculate defence strength: (shields × shields_tech / (mass +
    /// cargo)^(1/3)) × 30^(1/3)
    pub fn defence_strength(&self, shields_tech: f64, cargo_carried: f64) -> f64 {
        if self.shields_mass == 0.0 {
            return 0.0;
        }
        let total_mass = self.ship_mass() + cargo_carried;
        (self.shields_mass * shields_tech / total_mass.powf(1.0 / 3.0)) * 30.0_f64.powf(1.0 / 3.0)
    }

    /// Calculate cargo capacity: cargo_mass + cargo_mass²/10
    pub fn base_cargo_capacity(&self) -> f64 {
        self.cargo_mass + (self.cargo_mass * self.cargo_mass) / 10.0
    }
}

/// A spaceship
#[derive(Debug, Clone, Component)]
pub struct Ship {
    id: ShipId,
    owner: RaceId,
    design: ShipDesign,
    current_hull: f64,
    location: ShipLocation,
}

#[allow(dead_code)]
impl Ship {
    pub fn new(id: ShipId, owner: RaceId, design: ShipDesign, location: PlanetId) -> Self {
        Self {
            id,
            owner,
            current_hull: design.shields_mass(), // Hull = shields mass
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
    pub fn current_hull(&self) -> f64 {
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
        self.current_hull <= 0.0
    }

    /// Take damage to the ship
    pub fn take_damage(&mut self, damage: f64) {
        self.current_hull = (self.current_hull - damage).max(0.0);
    }

    /// Calculate travel speed based on design and technology
    pub fn travel_speed(&self, drive_tech: f64) -> f64 {
        self.design.speed(drive_tech, 0.0) // No cargo for now
    }

    /// Calculate attack strength with technology
    pub fn attack_strength(&self, weapons_tech: f64) -> f64 {
        self.design.attack_strength(weapons_tech)
    }

    /// Calculate defence strength with technology
    pub fn defence_strength(&self, shields_tech: f64) -> f64 {
        self.design.defence_strength(shields_tech, 0.0) // No cargo for now
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

impl ShipLocation {
    /// Returns the planet ID if the ship is at a planet
    pub fn planet_id(&self) -> Option<PlanetId> {
        match self {
            Self::AtPlanet(id) => Some(*id),
            Self::Traveling { .. } => None,
        }
    }

    /// Returns true if the ship is traveling
    #[allow(dead_code)]
    pub fn is_traveling(&self) -> bool {
        matches!(self, Self::Traveling { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ship_mass_calculation() {
        // Example from GalaxyNG manual: Fighter 2.48, 1, 1.20, 1.27, 0.00
        let fighter = ShipDesign::new(2.48, 1, 1.20, 1.27, 0.0);
        // Mass = 2.48 + 1.20 + 1.27 + 0.0 = 4.95
        assert!((fighter.ship_mass() - 4.95).abs() < 0.01);
    }

    #[test]
    fn test_ship_mass_with_multiple_attacks() {
        // Gunship 4.00, 2, 2.00, 4.00, 0.00
        let gunship = ShipDesign::new(4.0, 2, 2.0, 4.0, 0.0);
        // Mass = 4.0 + 2.0 + 4.0 + 0.0 + (2-1) × 2.0/2 = 11.0
        assert!((gunship.ship_mass() - 11.0).abs() < 0.01);
    }

    #[test]
    fn test_speed_calculation() {
        // Drone 1.00, 0, 0.00, 0.00, 0.00 should be speed 20.0 at tech 1.0
        let drone = ShipDesign::new(1.0, 0, 0.0, 0.0, 0.0);
        let speed = drone.speed(1.0, 0.0);
        assert!((speed - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_speed_with_cargo() {
        // Hauler 2.00, 0, 0.00, 0.00, 1.00
        let hauler = ShipDesign::new(2.0, 0, 0.0, 0.0, 1.0);
        // Ship mass = 3.0
        // Speed with no cargo = 20 × 1.0 × (2.0 / 3.0) = 13.33
        let speed_empty = hauler.speed(1.0, 0.0);
        assert!((speed_empty - 13.33).abs() < 0.1);

        // Speed with cargo = 20 × 1.0 × (2.0 / (3.0 + 1.1)) = 9.75
        let speed_loaded = hauler.speed(1.0, 1.1);
        assert!((speed_loaded - 9.75).abs() < 0.1);
    }

    #[test]
    fn test_cargo_capacity() {
        // Freighter with cargo_mass 10.0
        let freighter = ShipDesign::new(30.0, 0, 0.0, 9.5, 10.0);
        // Capacity = 10.0 + 10.0²/10 = 10.0 + 10.0 = 20.0
        assert_eq!(freighter.base_cargo_capacity(), 20.0);
    }

    #[test]
    fn test_attack_and_defence() {
        let battleship = ShipDesign::new(33.0, 3, 25.0, 16.0, 1.0);

        // Attack = weapons_mass × weapons_tech = 25.0 × 2.0 = 50.0
        assert_eq!(battleship.attack_strength(2.0), 50.0);

        // Defence calculation is more complex, just check it returns something
        let defence = battleship.defence_strength(2.0, 0.0);
        assert!(defence > 0.0);
    }
}
