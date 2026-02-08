use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::game_state::GameState;

const BACKGROUND_COLOR: Color = Color::srgb(0.05, 0.05, 0.1);
const PLANET_BASE_RADIUS: f32 = 3.0;
const SHIP_SIZE: f32 = 8.0;
const PAN_SPEED: f32 = 500.0;
const ZOOM_SPEED: f32 = 0.1;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 5.0;

#[derive(Component, Debug)]
struct MainCamera;

#[derive(Component, Debug)]
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

#[derive(Component, Debug)]
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

#[derive(Component)]
struct TurnText;

#[derive(Component)]
struct InfoText;

#[derive(Debug)]
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(Startup, (setup_camera, setup_ui))
            .add_systems(
                Update,
                (
                    spawn_planets,
                    spawn_ships,
                    update_ui,
                    handle_input,
                    camera_controls,
                ),
            );
    }
}

fn setup_camera(mut commands: Commands<'_, '_>, game_state: Res<'_, GameState>) {
    // Calculate initial camera position to center on galaxy
    let galaxy_width = game_state.galaxy().width() as f32;
    let galaxy_height = game_state.galaxy().height() as f32;

    commands.spawn((
        Camera2d,
        MainCamera,
        Transform::from_xyz(galaxy_width / 2.0, galaxy_height / 2.0, 0.0),
    ));
}

fn setup_ui(mut commands: Commands<'_, '_>) {
    // UI root
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|parent| {
            // Top bar - turn counter
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Turn: 0"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TurnText,
                    ));
                });

            // Bottom bar - instructions
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(
                            "SPACE: Advance Turn | Arrow Keys: Pan | Mouse Wheel: Zoom | ESC: Quit",
                        ),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                    parent.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        InfoText,
                    ));
                });
        });
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
            Transform::from_xyz(pos.x() as f32, pos.y() as f32, 0.0),
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
                Transform::from_xyz(pos.x() as f32 + 10.0, pos.y() as f32 + 10.0, 1.0),
            ));
        }
    }
}

fn update_ui(
    game_state: Res<'_, GameState>,
    mut turn_query: Query<'_, '_, &mut Text, (With<TurnText>, Without<InfoText>)>,
    mut info_query: Query<'_, '_, &mut Text, (With<InfoText>, Without<TurnText>)>,
) {
    if game_state.is_changed() {
        // Update turn counter
        if let Ok(mut text) = turn_query.get_single_mut() {
            **text = format!("Turn: {}", game_state.turn());
        }

        // Update info text
        if let Ok(mut text) = info_query.get_single_mut() {
            let total_planets = game_state.galaxy().planets().count();
            let total_ships = game_state.ships().count();
            let races = game_state.races().count();
            **text = format!(
                "Races: {} | Planets: {} | Ships: {}",
                races, total_planets, total_ships
            );
        }
    }
}

fn handle_input(
    keyboard: Res<'_, ButtonInput<KeyCode>>,
    mut game_state: ResMut<'_, GameState>,
    mut exit: EventWriter<'_, AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_state.advance_turn();
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn camera_controls(
    keyboard: Res<'_, ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<'_, '_, MouseWheel>,
    mut camera_query: Query<
        '_,
        '_,
        (&mut Transform, &mut OrthographicProjection),
        With<MainCamera>,
    >,
    time: Res<'_, Time>,
) {
    let Ok((mut transform, mut projection)) = camera_query.get_single_mut() else {
        return;
    };

    // Handle panning with arrow keys
    let mut pan_direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowLeft) {
        pan_direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        pan_direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        pan_direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        pan_direction.y -= 1.0;
    }

    if pan_direction != Vec3::ZERO {
        // Pan speed scales with zoom level
        let pan_speed = PAN_SPEED * projection.scale * time.delta_secs();
        transform.translation += pan_direction.normalize() * pan_speed;
    }

    // Handle zoom with mouse wheel
    for event in scroll_events.read() {
        let zoom_delta = -event.y * ZOOM_SPEED;
        projection.scale = (projection.scale + zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
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
