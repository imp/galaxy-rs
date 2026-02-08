use crate::race::TechnologyType;
use std::fmt;

/// Unique identifier for a planet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlanetId(pub u32);

impl fmt::Display for PlanetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Planet{}", self.0)
    }
}

/// Position of a planet in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[allow(dead_code)]
    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// A planet in the galaxy
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Planet {
    pub id: PlanetId,
    pub position: Position,
    pub size: u32,
    pub owner: Option<u32>, // Race ID
    pub materials: f64,
    pub tech_focus: TechFocus,
}

impl Planet {
    pub fn new(id: PlanetId, position: Position, size: u32, owner: Option<u32>) -> Self {
        Self {
            id,
            position,
            size,
            owner,
            materials: 0.0,
            tech_focus: TechFocus::None,
        }
    }

    /// Calculate material production per turn based on planet size
    pub fn material_production(&self) -> f64 {
        self.size as f64
    }

    /// Produce materials for this turn
    pub fn produce_materials(&mut self) {
        if self.owner.is_some() {
            self.materials += self.material_production();
        }
    }

    /// Consume materials for ship construction
    #[allow(dead_code)]
    pub fn consume_materials(&mut self, amount: f64) -> bool {
        if self.materials >= amount {
            self.materials -= amount;
            true
        } else {
            false
        }
    }
}

/// Technology focus for a planet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TechFocus {
    None,
    Research(TechnologyType),
}
