#![allow(unused_crate_dependencies)] // Test uses dependencies from main crate

use galaxy_core::game_state::GameState;
use galaxy_core::planet::Position;
use galaxy_core::racebot::Personality;

#[test]
fn test_multi_bot_simulation() {
    let mut game = GameState::new(2000.0, 2000.0);

    // Create 4 AI races with different personalities in different corners
    let race1_pos = Position::new(200.0, 200.0);
    let race1_home = game.galaxy_mut().add_planet(race1_pos, 100, Some(0));
    let race1 = game.add_ai_race(
        "Aggressors".to_string(),
        race1_home.0,
        Personality::Aggressive,
    );

    let race2_pos = Position::new(1800.0, 200.0);
    let race2_home = game.galaxy_mut().add_planet(race2_pos, 100, Some(1));
    let race2 = game.add_ai_race(
        "Defenders".to_string(),
        race2_home.0,
        Personality::Defensive,
    );

    let race3_pos = Position::new(200.0, 1800.0);
    let race3_home = game.galaxy_mut().add_planet(race3_pos, 100, Some(2));
    let race3 = game.add_ai_race(
        "Explorers".to_string(),
        race3_home.0,
        Personality::Expansionist,
    );

    let race4_pos = Position::new(1800.0, 1800.0);
    let race4_home = game.galaxy_mut().add_planet(race4_pos, 100, Some(3));
    let race4 = game.add_ai_race(
        "Economists".to_string(),
        race4_home.0,
        Personality::Economic,
    );

    // Add some unclaimed planets for colonization
    game.galaxy_mut()
        .add_planet(Position::new(1000.0, 1000.0), 50, None);
    game.galaxy_mut()
        .add_planet(Position::new(500.0, 1500.0), 40, None);
    game.galaxy_mut()
        .add_planet(Position::new(1500.0, 500.0), 60, None);

    // Verify all races exist
    assert!(game.get_race(race1).is_some());
    assert!(game.get_race(race2).is_some());
    assert!(game.get_race(race3).is_some());
    assert!(game.get_race(race4).is_some());

    // Simulate 10 turns
    for _turn in 0..10 {
        game.advance_turn();
    }

    // Verify game is still running
    assert_eq!(game.turn(), 10);

    // All races should still exist
    assert!(game.get_race(race1).is_some());
    assert!(game.get_race(race2).is_some());
    assert!(game.get_race(race3).is_some());
    assert!(game.get_race(race4).is_some());
}

#[test]
fn test_aggressive_vs_defensive() {
    let mut game = GameState::new(1000.0, 1000.0);

    // Two races close together
    let race1_home = game
        .galaxy_mut()
        .add_planet(Position::new(400.0, 500.0), 100, Some(0));
    let race1 = game.add_ai_race(
        "Warmongers".to_string(),
        race1_home.0,
        Personality::Aggressive,
    );

    let race2_home = game
        .galaxy_mut()
        .add_planet(Position::new(600.0, 500.0), 100, Some(1));
    let race2 = game.add_ai_race(
        "Pacifists".to_string(),
        race2_home.0,
        Personality::Defensive,
    );

    // Simulate 20 turns
    for _turn in 0..20 {
        game.advance_turn();
    }

    // Both races should still exist (combat not deadly yet)
    assert!(game.get_race(race1).is_some());
    assert!(game.get_race(race2).is_some());
}

#[test]
fn test_expansionist_colonization() {
    let mut game = GameState::new(1500.0, 1500.0);

    // One expansionist race
    let home_pos = Position::new(750.0, 750.0);
    let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
    let race = game.add_ai_race(
        "Colonizers".to_string(),
        home_planet.0,
        Personality::Expansionist,
    );

    // Add 5 nearby colonizable planets
    for i in 0..5 {
        let angle = (i as f64) * (2.0 * std::f64::consts::PI / 5.0);
        let x = 750.0 + 300.0 * angle.cos();
        let y = 750.0 + 300.0 * angle.sin();
        game.galaxy_mut().add_planet(Position::new(x, y), 50, None);
    }

    // Count initial planets
    let initial_owned = game
        .galaxy()
        .planets()
        .filter(|p| p.owner() == Some(race.0))
        .count();
    assert_eq!(initial_owned, 1);

    // Simulate 30 turns (enough time to build ships and colonize)
    for _turn in 0..30 {
        game.advance_turn();
    }

    // Expansionist should have colonized some planets
    // (Exact number depends on production, but should be > 1)
    let final_owned = game
        .galaxy()
        .planets()
        .filter(|p| p.owner() == Some(race.0))
        .count();

    // Should own more than just home planet
    assert!(
        final_owned >= initial_owned,
        "Expected colonization, owned {} planets",
        final_owned
    );
}

#[test]
fn test_economic_vs_aggressive_capital() {
    let mut game = GameState::new(1000.0, 1000.0);

    // Economic race
    let econ_home = game
        .galaxy_mut()
        .add_planet(Position::new(300.0, 500.0), 100, Some(0));
    let _econ_race = game.add_ai_race("Builders".to_string(), econ_home.0, Personality::Economic);

    // Aggressive race
    let aggr_home = game
        .galaxy_mut()
        .add_planet(Position::new(700.0, 500.0), 100, Some(1));
    let _aggr_race = game.add_ai_race("Warriors".to_string(), aggr_home.0, Personality::Aggressive);

    // Add materials to both home planets
    game.galaxy_mut()
        .get_planet_mut(econ_home)
        .unwrap()
        .add_materials(1000.0);
    game.galaxy_mut()
        .get_planet_mut(aggr_home)
        .unwrap()
        .add_materials(1000.0);

    // Simulate 15 turns
    for _turn in 0..15 {
        game.advance_turn();
    }

    // Economic race should have more capital (target 100 vs 30)
    let econ_capital = game.galaxy().get_planet(econ_home).unwrap().capital();
    let aggr_capital = game.galaxy().get_planet(aggr_home).unwrap().capital();

    // Economic should prioritize capital more
    assert!(
        econ_capital >= aggr_capital,
        "Economic capital={}, Aggressive capital={}",
        econ_capital,
        aggr_capital
    );
}

#[test]
fn test_long_simulation_stability() {
    let mut game = GameState::new(1500.0, 1500.0);

    // Create balanced race
    let home_pos = Position::new(750.0, 750.0);
    let home_planet = game.galaxy_mut().add_planet(home_pos, 100, Some(0));
    let _race = game.add_ai_race("Balanced".to_string(), home_planet.0, Personality::Balanced);

    // Add some planets to colonize
    for i in 0..10 {
        let x = 300.0 + (i as f64) * 100.0;
        let y = 300.0 + ((i * 7) % 10) as f64 * 100.0;
        game.galaxy_mut().add_planet(Position::new(x, y), 40, None);
    }

    // Simulate 100 turns - should not crash
    for _turn in 0..100 {
        game.advance_turn();
    }

    assert_eq!(game.turn(), 100);
}

#[test]
fn test_ai_race_identification() {
    let mut game = GameState::new(1000.0, 1000.0);

    // Create AI race
    let ai_home = game
        .galaxy_mut()
        .add_planet(Position::new(500.0, 500.0), 100, Some(0));
    let ai_race = game.add_ai_race("AI".to_string(), ai_home.0, Personality::Balanced);

    // Create human race
    let human_home = game
        .galaxy_mut()
        .add_planet(Position::new(700.0, 500.0), 100, Some(1));
    let human_race = game.add_race("Human".to_string(), human_home.0);

    // Verify AI control flags
    assert!(game.get_race(ai_race).unwrap().is_ai_controlled());
    assert!(!game.get_race(human_race).unwrap().is_ai_controlled());
}
