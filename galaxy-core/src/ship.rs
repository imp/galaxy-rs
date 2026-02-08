use std::fmt;

use bevy::prelude::*;

use crate::planet::PlanetId;
use crate::race::RaceId;

/// Unique identifier for a ship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct ShipId(pub u32);

impl fmt::Display for ShipId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ShipId").field(&self.0).finish()
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

    /// Calculate base cargo capacity: cargo_mass + cargo_mass²/10
    pub fn base_cargo_capacity(&self) -> f64 {
        self.cargo_mass + (self.cargo_mass * self.cargo_mass) / 10.0
    }

    /// Calculate cargo capacity with technology: base_capacity × cargo_tech
    pub fn cargo_capacity(&self, cargo_tech: f64) -> f64 {
        self.base_cargo_capacity() * cargo_tech
    }
}

/// Cargo types that ships can carry
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // Used in tests and future game mechanics // Used in tests and future game mechanics
pub enum CargoType {
    Colonists,
    Materials,
    Capital,
}

/// A spaceship
#[derive(Debug, Clone, Component)]
pub struct Ship {
    id: ShipId,
    owner: RaceId,
    design: ShipDesign,
    current_hull: f64,
    location: ShipLocation,
    cargo_colonists: f64,
    cargo_materials: f64,
    cargo_capital: f64,
}

impl Ship {
    pub fn new(id: ShipId, owner: RaceId, design: ShipDesign, location: PlanetId) -> Self {
        Self {
            id,
            owner,
            current_hull: design.shields_mass(), // Hull = shields mass
            design,
            location: ShipLocation::AtPlanet(location),
            cargo_colonists: 0.0,
            cargo_materials: 0.0,
            cargo_capital: 0.0,
        }
    }

    pub fn id(&self) -> ShipId {
        self.id
    }

    pub fn owner(&self) -> RaceId {
        self.owner
    }

    pub fn design(&self) -> &ShipDesign {
        &self.design
    }

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

    /// Get total cargo mass
    pub fn total_cargo(&self) -> f64 {
        self.cargo_colonists + self.cargo_materials + self.cargo_capital
    }

    /// Get available cargo space
    pub fn available_cargo(&self, cargo_tech: f64) -> f64 {
        let capacity = self.design.cargo_capacity(cargo_tech);
        (capacity - self.total_cargo()).max(0.0)
    }

    /// Load cargo onto ship (returns amount actually loaded)
    pub fn load_cargo(&mut self, cargo_type: CargoType, amount: f64, cargo_tech: f64) -> f64 {
        let available = self.available_cargo(cargo_tech);
        let to_load = amount.min(available);

        match cargo_type {
            CargoType::Colonists => self.cargo_colonists += to_load,
            CargoType::Materials => self.cargo_materials += to_load,
            CargoType::Capital => self.cargo_capital += to_load,
        }

        to_load
    }

    /// Unload cargo from ship (returns amount actually unloaded)
    pub fn unload_cargo(&mut self, cargo_type: CargoType, amount: f64) -> f64 {
        match cargo_type {
            CargoType::Colonists => {
                let unload = amount.min(self.cargo_colonists);
                self.cargo_colonists -= unload;
                unload
            }
            CargoType::Materials => {
                let unload = amount.min(self.cargo_materials);
                self.cargo_materials -= unload;
                unload
            }
            CargoType::Capital => {
                let unload = amount.min(self.cargo_capital);
                self.cargo_capital -= unload;
                unload
            }
        }
    }

    /// Get cargo amount by type
    pub fn cargo(&self, cargo_type: CargoType) -> f64 {
        match cargo_type {
            CargoType::Colonists => self.cargo_colonists,
            CargoType::Materials => self.cargo_materials,
            CargoType::Capital => self.cargo_capital,
        }
    }

    /// Calculate travel speed based on design, technology, and cargo
    pub fn travel_speed(&self, drive_tech: f64) -> f64 {
        self.design.speed(drive_tech, self.total_cargo())
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

#[cfg(test)]
mod cargo_tests {
    use super::*;

    #[test]
    fn test_cargo_capacity_with_tech() {
        // Freighter with cargo_mass = 10.0
        let design = ShipDesign::new(5.0, 0, 0.0, 5.0, 10.0);

        // Base capacity = 10 + 100/10 = 20
        assert_eq!(design.base_cargo_capacity(), 20.0);

        // With tech level 1.0: 20 × 1.0 = 20
        assert_eq!(design.cargo_capacity(1.0), 20.0);

        // With tech level 2.0: 20 × 2.0 = 40
        assert_eq!(design.cargo_capacity(2.0), 40.0);
    }

    #[test]
    fn test_load_cargo() {
        let design = ShipDesign::new(5.0, 0, 0.0, 5.0, 5.0);
        let mut ship = Ship::new(ShipId(1), RaceId(0), design, PlanetId(0));

        // Capacity = 5 + 25/10 = 7.5 at tech 1.0
        assert_eq!(ship.available_cargo(1.0), 7.5);

        // Load 3.0 colonists
        let loaded = ship.load_cargo(CargoType::Colonists, 3.0, 1.0);
        assert_eq!(loaded, 3.0);
        assert_eq!(ship.cargo(CargoType::Colonists), 3.0);
        assert_eq!(ship.available_cargo(1.0), 4.5);

        // Load 2.0 materials
        let loaded = ship.load_cargo(CargoType::Materials, 2.0, 1.0);
        assert_eq!(loaded, 2.0);
        assert_eq!(ship.total_cargo(), 5.0);

        // Try to load more than available (2.5 available, try 5.0)
        let loaded = ship.load_cargo(CargoType::Capital, 5.0, 1.0);
        assert_eq!(loaded, 2.5);
        assert_eq!(ship.total_cargo(), 7.5);
        assert_eq!(ship.available_cargo(1.0), 0.0);
    }

    #[test]
    fn test_unload_cargo() {
        let design = ShipDesign::new(5.0, 0, 0.0, 5.0, 10.0);
        let mut ship = Ship::new(ShipId(1), RaceId(0), design, PlanetId(0));

        // Load cargo
        ship.load_cargo(CargoType::Materials, 10.0, 1.0);
        assert_eq!(ship.cargo(CargoType::Materials), 10.0);

        // Unload 4.0
        let unloaded = ship.unload_cargo(CargoType::Materials, 4.0);
        assert_eq!(unloaded, 4.0);
        assert_eq!(ship.cargo(CargoType::Materials), 6.0);

        // Try to unload more than available
        let unloaded = ship.unload_cargo(CargoType::Materials, 10.0);
        assert_eq!(unloaded, 6.0);
        assert_eq!(ship.cargo(CargoType::Materials), 0.0);
    }

    #[test]
    fn test_cargo_affects_speed() {
        let design = ShipDesign::new(10.0, 0, 0.0, 5.0, 5.0);
        let mut ship = Ship::new(ShipId(1), RaceId(0), design, PlanetId(0));

        // Empty ship speed
        let empty_speed = ship.travel_speed(1.0);

        // Load cargo
        ship.load_cargo(CargoType::Materials, 5.0, 1.0);

        // Loaded ship should be slower
        let loaded_speed = ship.travel_speed(1.0);
        assert!(loaded_speed < empty_speed);
    }

    #[test]
    fn test_cargo_types_separate() {
        let design = ShipDesign::new(5.0, 0, 0.0, 5.0, 10.0);
        let mut ship = Ship::new(ShipId(1), RaceId(0), design, PlanetId(0));

        ship.load_cargo(CargoType::Colonists, 3.0, 1.0);
        ship.load_cargo(CargoType::Materials, 5.0, 1.0);
        ship.load_cargo(CargoType::Capital, 2.0, 1.0);

        assert_eq!(ship.cargo(CargoType::Colonists), 3.0);
        assert_eq!(ship.cargo(CargoType::Materials), 5.0);
        assert_eq!(ship.cargo(CargoType::Capital), 2.0);
        assert_eq!(ship.total_cargo(), 10.0);
    }

    #[test]
    fn test_cargo_tech_increases_capacity() {
        let design = ShipDesign::new(5.0, 0, 0.0, 5.0, 10.0);
        let mut ship = Ship::new(ShipId(1), RaceId(0), design, PlanetId(0));

        // Base capacity = 10 + 100/10 = 20
        // At tech 1.0: capacity = 20
        assert_eq!(ship.available_cargo(1.0), 20.0);

        // At tech 2.0: capacity = 40
        assert_eq!(ship.available_cargo(2.0), 40.0);

        // Load 25.0 at tech 2.0 (should fit)
        let loaded = ship.load_cargo(CargoType::Materials, 25.0, 2.0);
        assert_eq!(loaded, 25.0);

        // But at tech 1.0, this would exceed capacity
        assert!(ship.total_cargo() > ship.design.cargo_capacity(1.0));
    }
}
