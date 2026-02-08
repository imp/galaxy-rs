mod combat;
mod diplomacy;
mod galaxy;
mod game_state;
mod init;
mod planet;
mod race;
mod ship;

use game_state::GameState;
use init::GameConfig;
use init::initialize_game;
use planet::TechFocus;
use race::TechnologyType;
use ship::ShipDesign;

fn main() {
    println!("=== GALAXY - Space Simulator ===\n");

    // Initialize game with random galaxy
    let config = GameConfig {
        galaxy_width: 1000.0,
        galaxy_height: 1000.0,
        num_races: 4,
        num_planets: 15,
    };

    let mut game = initialize_game(config);

    println!("Game initialized - Turn {}", game.turn());
    println!(
        "Created galaxy {}x{}",
        game.galaxy().width(),
        game.galaxy().height()
    );
    println!("Races: {}", game.races().count());
    println!("Planets: {}", game.galaxy().planets().count());

    // Show race info
    println!("\n--- Races ---");
    for race in game.races() {
        let planet_count = game.galaxy().count_planets_owned_by(race.id().0);
        println!("{}: {} - {} planets", race.id(), race.name(), planet_count);
    }

    // Show planet distribution
    println!("\n--- Planets ---");
    for planet in game.galaxy().planets() {
        let owner_name = if let Some(owner_id) = planet.owner() {
            game.races()
                .find(|r| r.id().0 == owner_id)
                .map_or("Unknown", |r| r.name())
        } else {
            "Uninhabited"
        };
        println!("{}: size {} - {}", planet.id(), planet.size(), owner_name);
    }

    // Get first two races for testing
    let races_vec: Vec<_> = game.races().map(|r| r.id()).collect();
    let race1 = races_vec[0];
    let race2 = races_vec[1];

    // Get home planets
    let home1_id = game
        .galaxy()
        .planets()
        .find(|p| p.owner() == Some(race1.0))
        .unwrap()
        .id();

    let home2_id = game
        .galaxy()
        .planets()
        .find(|p| p.owner() == Some(race2.0))
        .unwrap()
        .id();

    // Set technology focus
    game.set_planet_tech_focus(home1_id, TechFocus::Research(TechnologyType::Drive));
    game.set_planet_tech_focus(home2_id, TechFocus::Research(TechnologyType::Weapon));

    // Simulate a few turns
    println!("\n--- Simulation ---");
    for _ in 0..3 {
        game.advance_turn();

        if let Some(planet) = game.galaxy().get_planet(home1_id) {
            println!(
                "Turn {}: {} materials: {:.0}",
                game.turn(),
                planet.id(),
                planet.materials()
            );
        }

        if let Some(race) = game.get_race(race1) {
            let tech = race.technology();
            println!(
                "  {} tech - Drive:{} Weapon:{} Shield:{}",
                race.name(),
                tech.drive_level(),
                tech.weapon_level(),
                tech.shield_level()
            );
        }
    }

    // Try building a ship
    let design = ShipDesign::new(10, 5, 2, 3);
    println!(
        "\nShip design cost: {:.0} materials",
        design.material_cost()
    );

    if let Some(ship_id) = game.build_ship(home1_id, design) {
        println!("Built {}", ship_id);
        if let Some(ship) = game.get_ship(ship_id) {
            let ship_design = ship.design();
            println!(
                "  Hull: {}, Engine: {}, Cannons: {}x{}",
                ship_design.hull_strength(),
                ship_design.engine_power(),
                ship_design.cannon_count(),
                ship_design.cannon_power()
            );
        }
    }

    if let Some(planet) = game.galaxy().get_planet(home1_id) {
        println!(
            "Planet {} materials remaining: {:.0}",
            planet.id(),
            planet.materials()
        );
    }

    // Test diplomacy
    println!("\n--- Diplomacy Test ---");
    println!(
        "{} vs {}: {:?}",
        game.get_race(race1).unwrap().name(),
        game.get_race(race2).unwrap().name(),
        game.diplomacy().get_relationship(race1, race2)
    );

    // Declare war
    game.diplomacy_mut().make_hostile(race1, race2);
    println!(
        "After war declaration: {:?}",
        game.diplomacy().get_relationship(race1, race2)
    );

    println!("\n=== Simulation Complete ===");
}
