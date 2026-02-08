use bevy::prelude::*;

use crate::race::TechnologyType;

/// Unique identifier for a planet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct PlanetId(pub u32);

impl std::fmt::Display for PlanetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Planet{}", self.0)
    }
}

/// Position of a planet in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct Position {
    x: f64,
    y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// A planet in the galaxy
#[derive(Debug, Clone, Component)]
pub struct Planet {
    id: PlanetId,
    position: Position,
    size: u32,
    owner: Option<u32>, // Race ID
    population: f64,
    industry: f64,
    resources: f64,
    materials: f64,
    capital: f64,
    colonists: f64,
    tech_focus: TechFocus,
    production_type: ProductionType,
}

impl Planet {
    pub fn new(id: PlanetId, position: Position, size: u32, owner: Option<u32>) -> Self {
        Self {
            id,
            position,
            size,
            owner,
            population: 0.0,
            industry: 0.0,
            resources: 1.0,
            materials: 0.0,
            capital: 0.0,
            colonists: 0.0,
            tech_focus: TechFocus::None,
            production_type: ProductionType::None,
        }
    }

    /// Create a home planet (fully populated, resources 10.0)
    pub fn new_home_planet(id: PlanetId, position: Position, size: u32, owner: u32) -> Self {
        let size_f = size as f64;
        Self {
            id,
            position,
            size,
            owner: Some(owner),
            population: size_f,
            industry: size_f,
            resources: 10.0,
            materials: 0.0,
            capital: 0.0,
            colonists: 0.0,
            tech_focus: TechFocus::None,
            production_type: ProductionType::Materials,
        }
    }

    pub fn id(&self) -> PlanetId {
        self.id
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn owner(&self) -> Option<u32> {
        self.owner
    }

    pub fn set_owner(&mut self, owner: Option<u32>) {
        self.owner = owner;
    }

    pub fn materials(&self) -> f64 {
        self.materials
    }

    pub fn capital(&self) -> f64 {
        self.capital
    }

    pub fn colonists(&self) -> f64 {
        self.colonists
    }

    pub fn population(&self) -> f64 {
        self.population
    }

    pub fn industry(&self) -> f64 {
        self.industry
    }

    pub fn resources(&self) -> f64 {
        self.resources
    }

    pub fn set_resources(&mut self, resources: f64) {
        self.resources = resources;
    }

    pub fn tech_focus(&self) -> TechFocus {
        self.tech_focus
    }

    pub fn set_tech_focus(&mut self, focus: TechFocus) {
        self.tech_focus = focus;
    }

    /// Calculate production capacity: Industry + (Population - Industry)/4
    pub fn production(&self) -> f64 {
        self.industry + (self.population - self.industry) / 4.0
    }

    /// Grow population by 8% per turn, capped by planet size
    pub fn grow_population(&mut self) {
        if self.owner.is_none() {
            return;
        }

        const GROWTH_RATE: f64 = 0.08;
        let new_population = self.population * (1.0 + GROWTH_RATE);
        let max_population = self.size as f64;

        if new_population <= max_population {
            self.population = new_population;
        } else {
            // Excess population becomes colonists (8 population = 1 colonist)
            let excess = new_population - max_population;
            self.colonists += excess / 8.0;
            self.population = max_population;
        }

        // Grow industry if we have capital (independent of population growth)
        if self.capital > 0.0 && self.industry < self.population {
            let capital_to_use = (self.population - self.industry).min(self.capital);
            self.industry += capital_to_use;
            self.capital -= capital_to_use;
        }
    }

    /// Add materials to stockpile
    pub fn add_materials(&mut self, amount: f64) {
        self.materials += amount;
    }

    /// Add capital to stockpile
    pub fn add_capital(&mut self, amount: f64) {
        self.capital += amount;
    }

    /// Add colonists to stockpile
    pub fn add_colonists(&mut self, amount: f64) {
        self.colonists += amount;
    }

    /// Calculate material production per turn: production × resources
    pub fn material_production(&self) -> f64 {
        self.production() * self.resources
    }

    /// Produce materials for this turn
    pub fn produce_materials(&mut self) {
        if self.owner.is_some() {
            self.materials += self.material_production();
        }
    }

    /// Consume materials for ship construction or capital production
    pub fn consume_materials(&mut self, amount: f64) -> bool {
        if self.materials >= amount {
            self.materials -= amount;
            true
        } else {
            false
        }
    }

    /// Consume capital
    pub fn consume_capital(&mut self, amount: f64) -> bool {
        if self.capital >= amount {
            self.capital -= amount;
            true
        } else {
            false
        }
    }

    /// Set production type
    pub fn set_production_type(&mut self, production_type: ProductionType) {
        self.production_type = production_type;
    }

    /// Get current production type
    pub fn production_type(&self) -> ProductionType {
        self.production_type
    }

    /// Execute production for this turn based on production_type
    /// GalaxyNG formulas:
    /// - Materials: production × resources
    /// - Capital: 1 capital requires 5 production + 1 material (auto-diverts
    ///   production to materials if needed)
    pub fn execute_production(&mut self) {
        if self.owner.is_none() {
            return;
        }

        let prod = self.production();

        match self.production_type {
            ProductionType::None => {}
            ProductionType::Materials => {
                self.materials += prod * self.resources;
            }
            ProductionType::Capital => {
                // 1 capital = 5 production + 1 material
                // If we don't have materials, auto-divert some production to make them
                let capital_per_prod = 1.0 / 5.0; // 0.2 capital per production
                let materials_needed_per_prod = capital_per_prod; // 0.2 materials per production

                if self.materials >= prod * materials_needed_per_prod {
                    // We have enough materials stockpiled
                    let capital_produced = prod * capital_per_prod;
                    self.capital += capital_produced;
                    self.materials -= capital_produced; // 1:1 ratio
                } else {
                    // Need to produce some materials
                    // With resources R, producing X materials takes X/R production
                    // Remaining production (prod - X/R) makes capital
                    // That capital needs X materials
                    // So: X = (prod - X/R) * 0.2
                    // Solve for X: X = (prod * 0.2 * R) / (1 + 0.2 * R)
                    let materials_from_stockpile = self.materials;
                    let prod_for_stockpile_capital = materials_from_stockpile * 5.0;

                    if prod_for_stockpile_capital >= prod {
                        // We can use stockpile for all production
                        let capital_produced = prod / 5.0;
                        self.capital += capital_produced;
                        self.materials -= capital_produced;
                    } else {
                        // Use stockpile first, then auto-produce materials
                        let capital_from_stockpile = materials_from_stockpile;
                        self.capital += capital_from_stockpile;
                        self.materials = 0.0;

                        let remaining_prod = prod - (capital_from_stockpile * 5.0);
                        // With remaining production, make materials and capital
                        // materials_produced = remaining_prod * R / (1 + 0.2*R)
                        let materials_produced =
                            (remaining_prod * self.resources) / (1.0 + 0.2 * self.resources);
                        let capital_produced =
                            (remaining_prod - materials_produced / self.resources) / 5.0;

                        self.capital += capital_produced;
                        // Materials are immediately consumed for capital
                    }
                }
            }
            ProductionType::Research(_tech_type) => {
                // TODO: Implement technology research
                // For now, just do nothing (will be added in separate ticket)
            }
            ProductionType::Ships(_ship_type_id) => {
                // TODO: Implement ship building with material costs
                // For now, just do nothing (will be integrated with ship
                // system)
            }
        }
    }
}

/// Production type for a planet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]

pub enum ProductionType {
    None,
    Materials,
    Capital,
    Research(TechnologyType),
    Ships(ShipTypeId),
}

/// Temporary ID for ship types (will be replaced with proper ship type system
/// later)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipTypeId(pub u32);

/// Technology focus for a planet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]

pub enum TechFocus {
    None,
    Research(TechnologyType),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_population_growth() {
        let mut planet = Planet::new_home_planet(
            PlanetId(1),
            Position::new(100.0, 100.0),
            200, // Size larger than population
            0,
        );

        // Set starting population below size
        planet.population = 100.0;
        planet.industry = 100.0;

        // Grow population (8% growth)
        planet.grow_population();
        assert!((planet.population() - 108.0).abs() < 0.1);
    }

    #[test]
    fn test_population_excess_becomes_colonists() {
        let mut planet = Planet::new_home_planet(
            PlanetId(1),
            Position::new(100.0, 100.0),
            100, // Size = population
            0,
        );

        assert_eq!(planet.population(), 100.0);
        assert_eq!(planet.colonists(), 0.0);

        // Population is at size, so growth should create colonists
        planet.grow_population();
        assert_eq!(planet.population(), 100.0); // Capped at size
        assert!(planet.colonists() > 0.0); // Excess became colonists (108-100)/8 = 1.0
        assert!((planet.colonists() - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_production_formula() {
        let mut planet = Planet::new_home_planet(PlanetId(1), Position::new(100.0, 100.0), 500, 0);

        // Production = Industry + (Population - Industry) / 4
        // With 500 pop and 500 ind: 500 + 0 = 500
        assert_eq!(planet.production(), 500.0);

        // Manually set different values
        planet.population = 500.0;
        planet.industry = 250.0;
        // With 500 pop and 250 ind: 250 + 250/4 = 312.5
        assert_eq!(planet.production(), 312.5);
    }

    #[test]
    fn test_material_production_with_resources() {
        let mut planet = Planet::new_home_planet(PlanetId(1), Position::new(100.0, 100.0), 100, 0);

        // Home planet has resources 10.0, production 100.0
        // Material production = 100.0 * 10.0 = 1000.0
        assert_eq!(planet.material_production(), 1000.0);

        // Change resources
        planet.set_resources(5.0);
        assert_eq!(planet.material_production(), 500.0);
    }

    #[test]
    fn test_capital_increases_industry() {
        let mut planet = Planet::new_home_planet(PlanetId(1), Position::new(100.0, 100.0), 500, 0);

        planet.population = 500.0;
        planet.industry = 200.0;
        planet.add_capital(100.0);

        // Growing should use capital to increase industry (up to population level)
        planet.grow_population();

        // Industry should have increased toward population
        assert!(planet.industry() > 200.0);
        // Some capital should have been used
        assert!(planet.capital() < 100.0);
    }
}

#[test]
fn test_capital_production_with_materials() {
    let mut planet = Planet::new_home_planet(PlanetId(1), Position::new(100.0, 100.0), 100, 0);

    planet.set_production_type(ProductionType::Capital);
    planet.add_materials(100.0); // Plenty of materials

    // Production = 100, need 500 production for 100 capital
    // With 100 materials stockpiled, can make 20 capital (5 prod each)
    planet.execute_production();

    assert_eq!(planet.capital(), 20.0);
    assert_eq!(planet.materials(), 80.0); // Used 20 materials
}

#[test]
fn test_capital_production_auto_materials() {
    let mut planet = Planet::new_home_planet(PlanetId(1), Position::new(100.0, 100.0), 100, 0);

    planet.set_production_type(ProductionType::Capital);
    planet.set_resources(10.0);
    // No materials stockpile, production = 100, resources = 10

    planet.execute_production();

    // Some capital should be produced (with auto-material generation)
    assert!(planet.capital() > 0.0);
    // With prod=100, resources=10, should make ~13.3 capital
    assert!((planet.capital() - 13.3).abs() < 0.5);
}

#[test]
fn test_materials_production() {
    let mut planet = Planet::new_home_planet(PlanetId(1), Position::new(100.0, 100.0), 100, 0);

    planet.set_production_type(ProductionType::Materials);
    planet.set_resources(5.0);

    planet.execute_production();

    // Production = 100, resources = 5.0
    // Materials = 100 × 5.0 = 500
    assert_eq!(planet.materials(), 500.0);
}
