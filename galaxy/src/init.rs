use rand::Rng;

use crate::game_state::GameState;
use crate::planet::Position;

/// Configuration for initializing a new game
#[derive(Debug, Clone)]
pub struct GameConfig {
    pub galaxy_width: f64,
    pub galaxy_height: f64,
    pub num_races: u32,
    pub num_planets: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            galaxy_width: 1000.0,
            galaxy_height: 1000.0,
            num_races: 4,
            num_planets: 20,
        }
    }
}

/// Initialize a new game with random galaxy generation
pub fn initialize_game(config: GameConfig) -> GameState {
    let mut rng = rand::thread_rng();
    let mut game = GameState::new(config.galaxy_width, config.galaxy_height);

    // Validate configuration
    if config.num_planets < config.num_races {
        panic!("Must have at least as many planets as races");
    }

    // Generate random positions for all planets
    let mut planet_positions = Vec::new();
    for _ in 0..config.num_planets {
        // Ensure planets are well-distributed
        let x = rng.gen_range(50.0..config.galaxy_width - 50.0);
        let y = rng.gen_range(50.0..config.galaxy_height - 50.0);
        planet_positions.push(Position::new(x, y));
    }

    // Create home planets for each race (first num_races planets)
    let race_names = generate_race_names(config.num_races);

    for i in 0..config.num_races {
        let position = planet_positions[i as usize];
        let planet_id = game.galaxy_mut().add_planet(position, 100, Some(i));

        // Create the race
        let race_name = race_names[i as usize].clone();
        game.add_race(race_name, planet_id.0);
    }

    // Create remaining planets (random size 10-300, random resources 0.01-10.00)
    for i in config.num_races..config.num_planets {
        let position = planet_positions[i as usize];
        let size = rng.gen_range(10..=300);
        let planet_id = game.galaxy_mut().add_planet(position, size, None);

        // Set random resources (average 1.0)
        let resources = rng.gen_range(0.01..=10.0);
        if let Some(planet) = game.galaxy_mut().get_planet_mut(planet_id) {
            planet.set_resources(resources);
        }
    }

    game
}

/// Generate random race names
fn generate_race_names(count: u32) -> Vec<String> {
    let prefixes = [
        "Zor", "Kar", "Thal", "Vex", "Nyx", "Drak", "Qua", "Xen", "Mor", "Lux", "Kor", "Zal",
        "Pyr", "Vok", "Rax", "Syl",
    ];

    let suffixes = [
        "ians", "ites", "oids", "ans", "ix", "ar", "on", "us", "el", "ak", "or", "im", "ax", "en",
        "um", "is",
    ];

    let mut rng = rand::thread_rng();
    let mut names = Vec::new();
    let mut used_names = std::collections::HashSet::new();

    for _ in 0..count {
        // Generate unique name
        loop {
            let prefix = prefixes[rng.gen_range(0..prefixes.len())];
            let suffix = suffixes[rng.gen_range(0..suffixes.len())];
            let name = format!("{}{}", prefix, suffix);

            if used_names.insert(name.clone()) {
                names.push(name);
                break;
            }
        }
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GameConfig::default();
        assert_eq!(config.galaxy_width, 1000.0);
        assert_eq!(config.galaxy_height, 1000.0);
        assert_eq!(config.num_races, 4);
        assert_eq!(config.num_planets, 20);
    }

    #[test]
    fn test_initialize_game() {
        let config = GameConfig {
            galaxy_width: 500.0,
            galaxy_height: 500.0,
            num_races: 3,
            num_planets: 10,
        };

        let game = initialize_game(config);

        // Check galaxy size
        assert_eq!(game.galaxy().width(), 500.0);
        assert_eq!(game.galaxy().height(), 500.0);

        // Check correct number of planets (accessible through iteration)
        let planet_count = game.galaxy().planets().count();
        assert_eq!(planet_count, 10);

        // Check that we have 3 races
        let race_count = game.races().count();
        assert_eq!(race_count, 3);
    }

    #[test]
    fn test_home_planets_size_100() {
        let config = GameConfig {
            galaxy_width: 1000.0,
            galaxy_height: 1000.0,
            num_races: 2,
            num_planets: 5,
        };

        let game = initialize_game(config);

        // First 2 planets should be size 100 (home planets)
        let home_planets: Vec<_> = game
            .galaxy()
            .planets()
            .filter(|p| p.owner().is_some())
            .collect();

        assert_eq!(home_planets.len(), 2);
        for planet in home_planets {
            assert_eq!(planet.size(), 100);
        }
    }

    #[test]
    fn test_uninhabited_planets_random_size() {
        let config = GameConfig {
            galaxy_width: 1000.0,
            galaxy_height: 1000.0,
            num_races: 2,
            num_planets: 10,
        };

        let game = initialize_game(config);

        // Should have 8 uninhabited planets
        let uninhabited: Vec<_> = game.galaxy().uninhabited_planets().collect();
        assert_eq!(uninhabited.len(), 8);

        // Check sizes are in valid range
        for planet in uninhabited {
            assert!(planet.size() >= 10);
            assert!(planet.size() <= 300);
        }
    }

    #[test]
    fn test_race_names_unique() {
        let names = generate_race_names(10);
        let unique_names: std::collections::HashSet<_> = names.iter().collect();

        assert_eq!(names.len(), 10);
        assert_eq!(unique_names.len(), 10); // All unique
    }

    #[test]
    #[should_panic(expected = "Must have at least as many planets as races")]
    fn test_invalid_config_panics() {
        let config = GameConfig {
            galaxy_width: 1000.0,
            galaxy_height: 1000.0,
            num_races: 10,
            num_planets: 5, // Less than races!
        };

        initialize_game(config);
    }
}
