use bevy::prelude::*;

/// Unique identifier for a race
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct RaceId(pub u32);

impl std::fmt::Display for RaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Race{}", self.0)
    }
}

/// Technology types that can be advanced
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
#[allow(dead_code)]
pub enum TechnologyType {
    Drive,
    Weapon,
    Shield,
}

/// Technology levels for a race
#[derive(Debug, Clone, Component)]
pub struct Technology {
    drive_level: u32,
    weapon_level: u32,
    shield_level: u32,
}

#[allow(dead_code)]
impl Technology {
    pub fn new() -> Self {
        Self {
            drive_level: 1,
            weapon_level: 1,
            shield_level: 1,
        }
    }

    pub fn drive_level(&self) -> u32 {
        self.drive_level
    }

    pub fn weapon_level(&self) -> u32 {
        self.weapon_level
    }

    pub fn shield_level(&self) -> u32 {
        self.shield_level
    }

    pub fn get_level(&self, tech_type: TechnologyType) -> u32 {
        match tech_type {
            TechnologyType::Drive => self.drive_level,
            TechnologyType::Weapon => self.weapon_level,
            TechnologyType::Shield => self.shield_level,
        }
    }

    pub fn advance(&mut self, tech_type: TechnologyType) {
        match tech_type {
            TechnologyType::Drive => self.drive_level += 1,
            TechnologyType::Weapon => self.weapon_level += 1,
            TechnologyType::Shield => self.shield_level += 1,
        }
    }

    /// Calculate effort required to advance technology based on planet size
    pub fn effort_required(planet_size: u32, current_level: u32) -> f64 {
        (planet_size as f64) * (current_level as f64)
    }
}

impl Default for Technology {
    fn default() -> Self {
        Self::new()
    }
}

/// A race in the galaxy
#[derive(Debug, Clone, Component)]
pub struct Race {
    id: RaceId,
    name: String,
    technology: Technology,
    home_planet_id: u32,
    tech_progress: TechProgress,
}

#[allow(dead_code)]
impl Race {
    pub fn new(id: RaceId, name: String, home_planet_id: u32) -> Self {
        Self {
            id,
            name,
            technology: Technology::new(),
            home_planet_id,
            tech_progress: TechProgress::new(),
        }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> RaceId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn technology(&self) -> &Technology {
        &self.technology
    }

    #[allow(dead_code)]
    pub fn home_planet_id(&self) -> u32 {
        self.home_planet_id
    }

    /// Add research effort to a technology type
    pub fn add_research(&mut self, tech_type: TechnologyType, effort: f64) {
        self.tech_progress.add_effort(tech_type, effort);

        // Check if we can advance the technology
        let current_level = self.technology.get_level(tech_type);
        let required = Technology::effort_required(100, current_level); // Base calculation

        if self.tech_progress.get_effort(tech_type) >= required {
            self.technology.advance(tech_type);
            self.tech_progress.reset(tech_type);
        }
    }
}

/// Tracks research progress toward next technology level
#[derive(Debug, Clone)]
struct TechProgress {
    drive_progress: f64,
    weapon_progress: f64,
    shield_progress: f64,
}

impl TechProgress {
    fn new() -> Self {
        Self {
            drive_progress: 0.0,
            weapon_progress: 0.0,
            shield_progress: 0.0,
        }
    }

    fn add_effort(&mut self, tech_type: TechnologyType, effort: f64) {
        match tech_type {
            TechnologyType::Drive => self.drive_progress += effort,
            TechnologyType::Weapon => self.weapon_progress += effort,
            TechnologyType::Shield => self.shield_progress += effort,
        }
    }

    fn get_effort(&self, tech_type: TechnologyType) -> f64 {
        match tech_type {
            TechnologyType::Drive => self.drive_progress,
            TechnologyType::Weapon => self.weapon_progress,
            TechnologyType::Shield => self.shield_progress,
        }
    }

    fn reset(&mut self, tech_type: TechnologyType) {
        match tech_type {
            TechnologyType::Drive => self.drive_progress = 0.0,
            TechnologyType::Weapon => self.weapon_progress = 0.0,
            TechnologyType::Shield => self.shield_progress = 0.0,
        }
    }
}
