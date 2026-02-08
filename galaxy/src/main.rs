use bevy::prelude::*;
use galaxy_core::init::GameConfig;
use galaxy_core::init::initialize_game;
use galaxy_core::rendering::RenderingPlugin;

fn main() {
    // Initialize game with random galaxy
    let config = GameConfig {
        galaxy_width: 1000.0,
        galaxy_height: 1000.0,
        num_races: 4,
        num_planets: 15,
    };

    let game = initialize_game(config);

    println!("=== GALAXY - Space Simulator ===");
    println!("Starting Bevy visualization...");
    println!(
        "Galaxy: {}x{}",
        game.galaxy().width(),
        game.galaxy().height()
    );
    println!("Races: {}", game.races().count());
    println!("Planets: {}", game.galaxy().planets().count());

    // Launch Bevy app
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GALAXY - Space Strategy".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RenderingPlugin)
        .insert_resource(game)
        .run();
}
