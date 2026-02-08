mod diplomacy;
mod galaxy;
mod game_state;
mod planet;
mod race;
mod ship;

use diplomacy::Relationship;
use game_state::GameState;
use planet::Position;
use planet::TechFocus;
use race::TechnologyType;
use ship::ShipDesign;

fn main() {
    println!("=== GALAXY - Space Simulator ===\n");

    // Create a new game
    let mut game = GameState::new(1000.0, 1000.0);

    // Add home planets for two races
    let home1_id = game
        .galaxy_mut()
        .add_planet(Position::new(100.0, 100.0), 100, Some(0));
    let home2_id = game
        .galaxy_mut()
        .add_planet(Position::new(900.0, 900.0), 100, Some(1));
    let _neutral = game
        .galaxy_mut()
        .add_planet(Position::new(500.0, 500.0), 150, None);

    // Create races
    let race1 = game.add_race("Humans".to_string(), home1_id.0);
    let race2 = game.add_race("Aliens".to_string(), home2_id.0);

    // Set technology focus
    game.set_planet_tech_focus(home1_id, TechFocus::Research(TechnologyType::Drive));
    game.set_planet_tech_focus(home2_id, TechFocus::Research(TechnologyType::Weapon));

    println!("Game initialized - Turn {}", game.turn());
    println!(
        "Created galaxy {}x{}",
        game.galaxy().width(),
        game.galaxy().height()
    );
    println!("{} home: {}", race1, home1_id);
    println!("{} home: {}", race2, home2_id);

    // Simulate a few turns
    for _ in 0..3 {
        game.advance_turn();

        if let Some(planet) = game.galaxy().get_planet(home1_id) {
            println!(
                "\nTurn {}: {} materials: {:.0}",
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

    // Test ship exploration
    println!("\n--- Ship Exploration Test ---");
    if let Some(ship_id) = game.build_ship(home1_id, ShipDesign::new(5, 10, 1, 1)) {
        println!("Built explorer {}", ship_id);

        // Send ship to neutral planet
        if game.order_ship_travel(ship_id, _neutral) {
            println!("Ship ordered to explore {}", _neutral);

            // Simulate travel
            for _i in 1..=5 {
                game.advance_turn();
                if let Some(ship) = game.get_ship(ship_id) {
                    match ship.location() {
                        ship::ShipLocation::Traveling { to, progress, .. } => {
                            println!(
                                "  Turn {}: Ship traveling to {} - {:.0}% complete",
                                game.turn(),
                                to,
                                progress * 100.0
                            );
                        }
                        ship::ShipLocation::AtPlanet(pid) => {
                            println!("  Turn {}: Ship arrived at {}", game.turn(), pid);
                            if let Some(planet) = game.galaxy().get_planet(*pid)
                                && let Some(owner) = planet.owner()
                            {
                                println!("    Planet colonized by race {}", owner);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    // Test diplomacy system
    println!("\n--- Diplomacy Test ---");
    println!(
        "Humans vs Aliens: {:?}",
        game.diplomacy().get_relationship(race1, race2)
    );

    // Make them hostile
    game.diplomacy_mut().make_hostile(race1, race2);
    println!(
        "After declaring war: {:?}",
        game.diplomacy().get_relationship(race1, race2)
    );
    println!(
        "Should attack: {}",
        game.diplomacy().should_attack(race1, race2)
    );

    // Make them friendly
    game.diplomacy_mut()
        .set_relationship(race1, race2, Relationship::Friendly);
    println!(
        "After alliance: {:?}",
        game.diplomacy().get_relationship(race1, race2)
    );
    println!(
        "Should attack: {}",
        game.diplomacy().should_attack(race1, race2)
    );

    println!("\n=== Simulation Complete ===");
}
