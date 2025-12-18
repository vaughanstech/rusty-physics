use avian3d::prelude::*;
use bevy::{ color, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::* };
use bevy_asset::{AssetServer};
use bevy_egui::EguiPrimaryContextPass;
use bevy_framepace::*;

use crate::{peripherals::*, interaction_modes::*, interactive_menu::*};
use super::SetMaxFps;

use super::GameState;

#[derive(Component)]
struct FpsText;

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
        .insert_resource(ImpulseSettings::default())
        .insert_resource(CameraOrientation::default())
        .insert_resource(InteractionMode(InteractionModeType::Click))
        .insert_resource(CursorDistance(10.0)) // set cursor distance on spawn
        .add_systems(EguiPrimaryContextPass, interactive_menu)
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
            set_max_fps,
            fps_counter,
        ));
}

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
        )).insert(WreckerCursor);
    };
    wrecker_label(wrecker_ball, "┌─ Wrecker: (0.00, 0.00, 0.00)");
}

/// Set the max framerate limit
fn set_max_fps(
    mut commands: Commands,
    mut settings: ResMut<FramepaceSettings>,
    fps_limit: Res<SetMaxFps>,
) {
    // Setting the Physics time equal to the max framerate
    commands.insert_resource(Time::<Fixed>::from_hz(fps_limit.fps));
    // Actually setting global max fps
    settings.limiter = Limiter::from_framerate(fps_limit.fps);
}

/// Tracks frames per second
fn fps_counter(
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
