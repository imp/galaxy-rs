use std::collections::HashMap;

use bevy::prelude::*;

use crate::race::RaceId;

/// Relationship between two races
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Default)]
pub enum Relationship {
    /// Races are allies - ships will not attack
    Friendly,
    /// Races are at war - ships will attack on sight
    Hostile,
    /// Races are neutral - ships will not attack unless provoked
    #[default]
    Neutral,
}

/// Manages diplomatic relationships between all races
#[derive(Debug, Clone, Resource, Default)]
pub struct Diplomacy {
    // HashMap of (race1_id, race2_id) -> Relationship
    // We store relationships as ordered pairs where race1_id < race2_id
    relationships: HashMap<(u32, u32), Relationship>,
}

impl Diplomacy {
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    /// Get the relationship between two races
    pub fn get_relationship(&self, race1: RaceId, race2: RaceId) -> Relationship {
        // Can't have relationship with yourself
        if race1 == race2 {
            return Relationship::Friendly;
        }

        let key = Self::make_key(race1.0, race2.0);
        self.relationships.get(&key).copied().unwrap_or_default()
    }

    /// Set the relationship between two races
    pub fn set_relationship(&mut self, race1: RaceId, race2: RaceId, relationship: Relationship) {
        // Can't set relationship with yourself
        if race1 == race2 {
            return;
        }

        let key = Self::make_key(race1.0, race2.0);
        self.relationships.insert(key, relationship);
    }

    /// Make a race hostile toward another (due to attack)
    pub fn make_hostile(&mut self, attacker: RaceId, defender: RaceId) {
        // Both sides become hostile to each other
        self.set_relationship(attacker, defender, Relationship::Hostile);
    }

    /// Check if two races are hostile to each other
    pub fn are_hostile(&self, race1: RaceId, race2: RaceId) -> bool {
        self.get_relationship(race1, race2) == Relationship::Hostile
    }

    /// Check if two races are friendly
    pub fn are_friendly(&self, race1: RaceId, race2: RaceId) -> bool {
        self.get_relationship(race1, race2) == Relationship::Friendly
    }

    /// Check if ships should attack each other
    pub fn should_attack(&self, race1: RaceId, race2: RaceId) -> bool {
        self.are_hostile(race1, race2)
    }

    // Helper to create ordered key for HashMap
    fn make_key(id1: u32, id2: u32) -> (u32, u32) {
        if id1 < id2 { (id1, id2) } else { (id2, id1) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_relationship_is_neutral() {
        let diplomacy = Diplomacy::new();
        let race1 = RaceId(0);
        let race2 = RaceId(1);

        assert_eq!(
            diplomacy.get_relationship(race1, race2),
            Relationship::Neutral
        );
    }

    #[test]
    fn test_self_relationship_is_friendly() {
        let diplomacy = Diplomacy::new();
        let race = RaceId(0);

        assert_eq!(
            diplomacy.get_relationship(race, race),
            Relationship::Friendly
        );
    }

    #[test]
    fn test_set_and_get_relationship() {
        let mut diplomacy = Diplomacy::new();
        let race1 = RaceId(0);
        let race2 = RaceId(1);

        diplomacy.set_relationship(race1, race2, Relationship::Hostile);
        assert_eq!(
            diplomacy.get_relationship(race1, race2),
            Relationship::Hostile
        );

        // Should be symmetric
        assert_eq!(
            diplomacy.get_relationship(race2, race1),
            Relationship::Hostile
        );
    }

    #[test]
    fn test_make_hostile() {
        let mut diplomacy = Diplomacy::new();
        let race1 = RaceId(0);
        let race2 = RaceId(1);

        diplomacy.make_hostile(race1, race2);
        assert!(diplomacy.are_hostile(race1, race2));
        assert!(diplomacy.should_attack(race1, race2));
    }

    #[test]
    fn test_friendly_relationship() {
        let mut diplomacy = Diplomacy::new();
        let race1 = RaceId(0);
        let race2 = RaceId(1);

        diplomacy.set_relationship(race1, race2, Relationship::Friendly);
        assert!(diplomacy.are_friendly(race1, race2));
        assert!(!diplomacy.should_attack(race1, race2));
    }
}
