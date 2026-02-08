use bevy::prelude::*;

use crate::game_state::GameState;

const BACKGROUND_COLOR: Color = Color::srgb(0.05, 0.05, 0.1);
const PLANET_BASE_RADIUS: f32 = 3.0;
const SHIP_SIZE: f32 = 8.0;
const ZOOM_SCALE: f32 = 2.0;

#[derive(Component)]
pub struct PlanetMarker {
    planet_id: u32,
}

impl PlanetMarker {
    pub fn new(planet_id: u32) -> Self {
        Self { planet_id }
    }

    pub fn planet_id(&self) -> u32 {
        self.planet_id
    }
}

#[derive(Component)]
pub struct ShipMarker {
    ship_id: u32,
}

impl ShipMarker {
    pub fn new(ship_id: u32) -> Self {
        Self { ship_id }
    }

    pub fn ship_id(&self) -> u32 {
        self.ship_id
    }
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (spawn_planets, spawn_ships, update_ui));
    }
}

fn setup_camera(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
}

fn spawn_planets(
    mut commands: Commands<'_, '_>,
    game_state: Res<'_, GameState>,
    existing: Query<'_, '_, &PlanetMarker>,
) {
    if !game_state.is_changed() {
        return;
    }

    let existing_ids: std::collections::HashSet<_> =
        existing.iter().map(|m| m.planet_id()).collect();

    let galaxy = game_state.galaxy();
    for planet in galaxy.planets() {
        if existing_ids.contains(&planet.id().0) {
            continue;
        }

        let pos = planet.position();
        let size = planet.size() as f32;
        let radius = PLANET_BASE_RADIUS * (size / 100.0).sqrt();

        let color = if let Some(owner_id) = planet.owner() {
            race_color(owner_id)
        } else {
            Color::srgb(0.5, 0.5, 0.5)
        };

        commands.spawn((
            PlanetMarker::new(planet.id().0),
            Sprite {
                color,
                custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                ..default()
            },
            Transform::from_xyz(
                pos.x() as f32 * ZOOM_SCALE,
                pos.y() as f32 * ZOOM_SCALE,
                0.0,
            ),
        ));
    }
}

fn spawn_ships(
    mut commands: Commands<'_, '_>,
    game_state: Res<'_, GameState>,
    existing: Query<'_, '_, &ShipMarker>,
) {
    if !game_state.is_changed() {
        return;
    }

    let existing_ids: std::collections::HashSet<_> = existing.iter().map(|m| m.ship_id()).collect();

    let galaxy = game_state.galaxy();
    for ship in game_state.ships() {
        if existing_ids.contains(&ship.id().0) {
            continue;
        }

        if let Some(planet_id) = ship.location().planet_id()
            && let Some(planet) = galaxy.get_planet(planet_id)
        {
            let pos = planet.position();
            let color = race_color(ship.owner().0);

            commands.spawn((
                ShipMarker::new(ship.id().0),
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(SHIP_SIZE, SHIP_SIZE)),
                    ..default()
                },
                Transform::from_xyz(
                    pos.x() as f32 * ZOOM_SCALE + 10.0,
                    pos.y() as f32 * ZOOM_SCALE + 10.0,
                    1.0,
                ),
            ));
        }
    }
}

fn update_ui(game_state: Res<'_, GameState>) {
    if game_state.is_changed() {
        // UI updates will go here
        // For now just track turns
    }
}

fn race_color(race_id: u32) -> Color {
    let colors = [
        Color::srgb(1.0, 0.3, 0.3), // Red
        Color::srgb(0.3, 0.3, 1.0), // Blue
        Color::srgb(0.3, 1.0, 0.3), // Green
        Color::srgb(1.0, 1.0, 0.3), // Yellow
        Color::srgb(1.0, 0.3, 1.0), // Magenta
        Color::srgb(0.3, 1.0, 1.0), // Cyan
    ];
    colors[race_id as usize % colors.len()]
}
