mod galaxy;
mod game_state;
mod planet;
mod race;
mod ship;

use game_state::GameState;
use planet::{Position, TechFocus};
use race::TechnologyType;
use ship::ShipDesign;

fn main() {
    println!("=== GALAXY - Space Simulator ===\n");
    
    // Create a new game
    let mut game = GameState::new(1000.0, 1000.0);
    
    // Add home planets for two races
    let home1_id = game.galaxy.add_planet(Position::new(100.0, 100.0), 100, Some(0));
    let home2_id = game.galaxy.add_planet(Position::new(900.0, 900.0), 100, Some(1));
    let _neutral = game.galaxy.add_planet(Position::new(500.0, 500.0), 150, None);
    
    // Create races
    let race1 = game.add_race("Humans".to_string(), home1_id.0);
    let race2 = game.add_race("Aliens".to_string(), home2_id.0);
    
    // Set technology focus
    game.set_planet_tech_focus(home1_id, TechFocus::Research(TechnologyType::Drive));
    game.set_planet_tech_focus(home2_id, TechFocus::Research(TechnologyType::Weapon));
    
    println!("Game initialized - Turn {}", game.turn);
    println!("Created galaxy {}x{}", game.galaxy.width, game.galaxy.height);
    println!("{} home: {}", race1, home1_id);
    println!("{} home: {}", race2, home2_id);
    
    // Simulate a few turns
    for _ in 0..3 {
        game.advance_turn();
        
        if let Some(planet) = game.galaxy.get_planet(home1_id) {
            println!("\nTurn {}: {} materials: {:.0}", 
                game.turn, planet.id, planet.materials);
        }
        
        if let Some(race) = game.get_race_mut(race1) {
            println!("  {} tech - Drive:{} Weapon:{} Shield:{}", 
                race.name,
                race.technology.drive_level,
                race.technology.weapon_level,
                race.technology.shield_level);
        }
    }
    
    // Try building a ship
    let design = ShipDesign::new(10, 5, 2, 3);
    println!("\nShip design cost: {:.0} materials", design.material_cost());
    
    if let Some(ship_id) = game.build_ship(home1_id, design) {
        println!("Built {}", ship_id);
        if let Some(ship) = game.get_ship(ship_id) {
            println!("  Hull: {}, Engine: {}, Cannons: {}x{}", 
                ship.design.hull_strength,
                ship.design.engine_power,
                ship.design.cannon_count,
                ship.design.cannon_power);
        }
    }
    
    if let Some(planet) = game.galaxy.get_planet(home1_id) {
        println!("Planet {} materials remaining: {:.0}", planet.id, planet.materials);
    }
    
    println!("\n=== Simulation Complete ===");
}
