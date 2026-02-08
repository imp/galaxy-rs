use crate::race::TechnologyType;
use bevy::prelude::*;

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

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[allow(dead_code)]
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
    materials: f64,
    tech_focus: TechFocus,
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

    pub fn id(&self) -> PlanetId {
        self.id
    }

    #[allow(dead_code)]
    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn owner(&self) -> Option<u32> {
        self.owner
    }

    #[allow(dead_code)]
    pub fn set_owner(&mut self, owner: Option<u32>) {
        self.owner = owner;
    }

    pub fn materials(&self) -> f64 {
        self.materials
    }

    pub fn tech_focus(&self) -> TechFocus {
        self.tech_focus
    }

    pub fn set_tech_focus(&mut self, focus: TechFocus) {
        self.tech_focus = focus;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum TechFocus {
    None,
    Research(TechnologyType),
}
