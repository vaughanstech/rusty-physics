use avian3d::prelude::*;
use bevy::{ color, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::* };
use bevy_asset::{AssetServer};
use bevy_egui::{EguiPrimaryContextPass, EguiPlugin};
use bevy_framepace::*;

use crate::{interactions::{interactive_menu::*, *}, menus::pause_menu::{InGameMenuState, PauseFromState}};
use super::SetFps;

use super::GameState;

#[derive(Component)]
pub struct FpsText;

pub fn game_plugin(
    app: &mut App,
) {
    app
        .add_systems(OnEnter(GameState::Game), game_setup)
        .insert_resource(CameraSettings {
            speed: 8.0,
            sensitivity: 0.002,
            zoom_speed: 30.0,
        })
        .add_plugins((
            EguiPlugin::default(),
        ))
        .insert_resource(ImpulseSettings::default())
        .insert_resource(CameraOrientation::default())
        .insert_resource(CursorDistance(10.0)) // set cursor distance on spawn
        .insert_resource(InteractionMode(InteractionModeType::Click))
        .add_systems(EguiPrimaryContextPass, interactive_menu.run_if(in_state(GameState::Game)))
        .add_systems(Update, (
            // spawn_cubes.run_if(on_timer(Duration::from_secs(1))),
            keyboard_movement,
            mouse_look,
            // Camera Zoom/Scroll runs only in Click Mode
            (
                mouse_scroll,
                set_impulse_cursor_visibility::<false>,
                set_wrecker_cursor_visibility::<false>,
            ).run_if(resource_equals(InteractionMode(InteractionModeType::Click))),
            // Cursor Control/Draw runs only in Impulse && Wrecker Mode
            (
                scroll_control, // System to update cursor distance
                draw_impulse_cursor,                // System to draw the gizmo
                apply_force,
                set_impulse_cursor_visibility::<true>,
                set_wrecker_cursor_visibility::<false>,
            ).run_if(resource_equals(InteractionMode(InteractionModeType::Impulse))),
            (
                scroll_control,
                draw_wrecker_cursor,
                set_impulse_cursor_visibility::<false>,
                set_wrecker_cursor_visibility::<true>,
            ).run_if(resource_equals(InteractionMode(InteractionModeType::Wrecker))),
            toggle_debug_render_state,
            game_action,
        ).run_if(in_state(GameState::Game)))
        .add_systems(OnExit(GameState::Game), cleanup_game);
}

#[derive(Component)]
struct OnGameScreen;

fn game_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // Light: bright white light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        OnGameScreen,
    ));

    commands.spawn((
        Text::new("FPS: "),
        TextFont {
            font_size: 42.0,
            ..default()
        },
    ))
    .with_child((
        TextSpan::default(),
        TextFont {
            font_size: 33.0,
            ..Default::default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        FpsText,
    ));

    let impulse_ball = commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(5).from_asset("shapes.glb"))
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Hidden,
        OnGameScreen,
    ))
    .insert(ImpulseCursor).id();

    let sphere = meshes.add(Sphere::new(0.5));

    let wrecker_ball = commands.spawn((
        Mesh3d(sphere.clone()),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
        Transform::from_xyz(0.0, 10.0, 0.0),
        Collider::sphere(0.5),
        RigidBody::Kinematic,
        Visibility::Hidden,
        OnGameScreen,
    ))
    .insert(WreckerCursor).id();

    let text_style = TextFont {
        font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
        ..Default::default()
    };

    let label_text_style = (text_style.clone(), TextColor(color::palettes::css::ORANGE.into()));


    let mut impulse_label = |entity: Entity, label: &str| {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            ExampleLabel { entity },
            children![(
                Text::new(label),
                label_text_style.clone(),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::ZERO,
                    ..Default::default()
                },
                TextLayout::default().with_no_wrap(),
                ImpulseCoords,
            )],
            Visibility::Hidden,
            OnGameScreen,
        )).insert(ImpulseCursor);
    };
    impulse_label(impulse_ball, "┌─ Impulse: (0.00, 0.00, 0.00)");

    let mut wrecker_label = |entity: Entity, label: &str| {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            ExampleLabel { entity },
            children![(
                Text::new(label),
                label_text_style.clone(),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::ZERO,
                    ..Default::default()
                },
                TextLayout::default().with_no_wrap(),
                WreckerCoords,
            )],
            Visibility::Hidden,
            OnGameScreen,
        )).insert(WreckerCursor);
    };
    wrecker_label(wrecker_ball, "┌─ Wrecker: (0.00, 0.00, 0.00)");
}


/// Cleans up all objects/entities that are created in the game_setup() function
/// - This runs when the user leaves the GameState::Game state
fn cleanup_game(mut commands: Commands, query: Query<Entity, With<OnGameScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Set the max framerate limit
pub fn set_max_fps(
    mut commands: Commands,
    mut settings: ResMut<FramepaceSettings>,
    fps_limit: Res<SetFps>,
) {
    let fps = match *fps_limit {
        SetFps::Low => 30.0,
        SetFps::Medium => 60.0,
        SetFps::High => 120.0,
        SetFps::Uncapped => 0.0,
    };
    
    // Actually setting global max fps
    if *fps_limit == SetFps::Uncapped {
        settings.limiter = Limiter::Off;
    } else {
        // Setting the Physics time equal to the max framerate
        commands.insert_resource(Time::<Fixed>::from_hz(fps));
        settings.limiter = Limiter::from_framerate(fps);
    }
}

/// Tracks frames per second
pub fn fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
            && let Some(value) = fps.smoothed() 
        {
            // update the value of the second section
            **span = format!("{value:.2}");
        }
    }
}

/// Cross-system function used to toggle between the Game state and the Pause state
pub fn game_action(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    pause_state: Res<State<PauseFromState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<InGameMenuState>>,
    mut pause_from_state: ResMut<NextState<PauseFromState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Game => {
                game_state.set(GameState::Paused);
                pause_from_state.set(PauseFromState::Game);
                
                info!("Pausing Game");
            }
            GameState::Paused => {
                if *pause_state.get() == PauseFromState::Game {
                    game_state.set(GameState::Game);
                    menu_state.set(InGameMenuState::Disabled);
                    pause_from_state.set(PauseFromState::Disabled);
                    info!("Resuming Game");
                    return;
                }
            }
            _ => {}
        }
        // for mut menu_toggle in menu_toggle_query.iter_mut() {
        //     *menu_toggle = match *menu_toggle {
        //         TogglePauseMenu::Disabled => {
        //             game_state.set(GameState::Paused);
        //             info!("Game state = Paused");
        //             TogglePauseMenu::Enabled
        //         }
        //         TogglePauseMenu::Enabled => {
        //             game_state.set(GameState::Game);
        //             menu_state.set(InGameMenuState::Disabled);
        //             info!("Game state = Game");
        //             TogglePauseMenu::Disabled
        //         }
        //     }
        // }
    }
}

fn toggle_debug_render_state(
    // mut debug_render_state: ResMut<DebugRenderState>,
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        let (config, _) = gizmo_config_store
            .config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
    }
}

use bevy::input::mouse::{MouseMotion, MouseWheel};

#[derive(Component)]
enum ExampleViewports {
    _PerspectiveMain,
    _PerspectiveStretched,
    _PerspectiveMoving,
    _PerspectiveControl,
    _OrthographicMain,
    _OrthographicStretched,
    _OrthographicMoving,
    _OrthographicControl,
}

#[derive(Component)]
pub struct FlyCamera;

#[derive(Resource)]
pub struct CameraSettings {
    pub speed: f32, // camera movement speed
    pub sensitivity: f32, // mouse movement sensitivity
    pub zoom_speed: f32, // mouse scroll sensitivity
}

#[derive(Resource, Default)]
pub struct CameraOrientation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Resource)]
pub struct CursorDistance(pub f32);

/// Initializes 3D camera
pub fn setup_camera(
    mut commands: Commands,
    mut orientation: ResMut<CameraOrientation>,
) {
    let position = Vec3::new(0.0, 10.0, 20.0);

    // Initializing Camera Orientation:
    // - Calculate yaw and pitch from the camera's starting Transform and use those values to initialize the CameraOrientation resource

    // Yaw (Y-axis rotation, horizontal look)
    orientation.yaw = 0.0;

    // Pitch (X-axis rotation, vertical look)
    // let horizontal_length = Vec2::new(direction.x, direction.z).length();
    orientation.pitch = 0.0;

    // Construct the rotation from the calculated yaw and pitch
    let rotation = Quat::from_axis_angle(Vec3::Y, orientation.yaw) * Quat::from_axis_angle(Vec3::X, orientation.pitch);

    // Create the initial transform using the calculated position
    let transform = Transform::from_translation(position).with_rotation(rotation);
    
    // Camera: initially positioned above and looking at origin
    commands.spawn((
        Camera3d::default(),
        ExampleViewports::_PerspectiveMain,
        transform,
        FlyCamera,
        // OnGameScreen,
        // OnInGameMenuScreen,
        // crate::levels::OnLevelScreen,
        // DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
    ));
}

/// Handles keyboard input for movement
pub fn keyboard_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;

        // local forward and right vectors relative to camera
        let forward = -transform.local_z();
        let right = transform.right();

        // WASD movement
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += *forward;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= *forward;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= *right;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += *right;
        }

        // Up/Down
        if keyboard_input.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * settings.speed * time.delta_secs();
        }
    }
}

/// Handles mouse movement for looking around
pub fn mouse_look(
    mut mouse_events: MessageReader<MouseMotion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    settings: Res<CameraSettings>,
    mut orientation: ResMut<CameraOrientation>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
) {
    let mut delta = Vec2::ZERO;
    if mouse_input.pressed(MouseButton::Middle) {
        for event in mouse_events.read() {
            delta += event.delta;
        }
    }

    if delta.length_squared() == 0.0 {
        return;
    }

    // update yaw and pitch
    orientation.yaw -= delta.x * settings.sensitivity;
    orientation.pitch -= delta.y * settings.sensitivity;
    orientation.pitch = orientation.pitch.clamp(-1.54, 1.54); // prevent flipping

    // apply rotation to camera transformation
    for mut transform in &mut query {
        transform.rotation = Quat::from_axis_angle(Vec3::Y, orientation.yaw) * Quat::from_axis_angle(Vec3::X, orientation.pitch);
    }
}

/// Handles mouse scroll wheel for zooming in/out of camera
pub fn mouse_scroll(
    mut scroll_events: MessageReader<MouseWheel>,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
) {
    let mut scroll_delta = 0.0;
    for event in scroll_events.read() {
        // scroll up = zoom in
        scroll_delta += event.y
    }

    if scroll_delta.abs() < f32::EPSILON {
        return;
    }

    for mut transform in &mut query {
        let forward = transform.forward();
        transform.translation += forward * scroll_delta * settings.zoom_speed * time.delta_secs();
    }
}

pub fn scroll_control(
    mut scroll_events: MessageReader<MouseWheel>,
    mut distance: ResMut<CursorDistance>,
) {
    let mut scroll_delta = 0.0;
    for event in scroll_events.read() {
        scroll_delta += event.y;
    }

    if scroll_delta.abs() > f32::EPSILON {
        distance.0 -= scroll_delta * 0.5;
        distance.0 = distance.0.clamp(1.0, 50.0);
    }
}
