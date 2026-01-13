mod entity_pipeline;
mod game;
mod interactions;
mod levels;
mod menus;

use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{DefaultPlugins, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::*};
use bevy_framepace::*;

use crate::interactions::interactive_menu::cleanup_entities;

/// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
    Levels,
}

/// Used to toggle between the different FPS limits, can be set in the Settings menu
/// - **Low** = 30.0 FPS
/// - **Medium** = 60.0 FPS
/// - **High** = 120.0 FPS
/// - **Uncapped** = No Limit on FPS
#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
pub enum SetFps {
    Low,
    Medium,
    High,
    Uncapped,
}

#[derive(Component)]
pub struct FpsText;

/// State used to track and toggle the game in its Running and Paused states
/// - This is a subsystem to the `GameState`
/// - The game will primarily run in each GameState (e.g. `Game`, `Levels`) but the `SimulationState` is used inside of them to display the in-game menu
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum SimulationState {
    #[default]
    Running,
    Paused,
}

#[derive(Component)]
struct SetupCamera;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),
            FramepacePlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .init_state::<GameState>()
        .init_state::<SimulationState>()
        .insert_resource(SetFps::High)
        .add_systems(Startup, fps_text)
        .add_systems(Update, (log_state_changes, set_max_fps, fps_counter))
        .add_systems(OnEnter(GameState::Menu), setup)
        .add_plugins((menus::main_menu::menu_plugin, game::game_plugin, menus::pause_menu::pause_menu_plugin, levels::levels_plugin))
        .add_systems(OnEnter(GameState::Menu), cleanup_entities)
        .add_systems(OnExit(GameState::Menu), cleanup_setup)
        .run();
}

fn fps_text(
    mut commands: Commands,
) {
    commands.spawn((
        Text::new("FPS: "),
        TextFont {
            font_size: 20.0,
            ..default()
        },
    ))
    .with_child((
        TextSpan::default(),
        TextFont {
            font_size: 20.0,
            ..Default::default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        FpsText,
    ));
}

/// Set the max framerate limit
fn set_max_fps(
    mut commands: Commands,
    mut settings: ResMut<FramepaceSettings>,
    fps_limit: Res<SetFps>,
) {
    let fps = match *fps_limit {
        SetFps::Low => 30.0,
        SetFps::Medium => 60.0,
        SetFps::High => 120.0,
        SetFps::Uncapped => 240.0,
    };
    // Setting the Physics time equal to the max framerate
    commands.insert_resource(Time::<Fixed>::from_hz(fps));
    settings.limiter = Limiter::from_framerate(fps);
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

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d::default(),
        SetupCamera,
    ));
}

fn log_state_changes(
    mut reader: MessageReader<StateTransitionEvent<GameState>>
) {
    for event in reader.read() {
        info!("State changed from {:?} -> {:?}", event.exited, event.entered);
    }
}

fn cleanup_setup(
    mut commands: Commands,
    cam_query: Query<Entity, With<SetupCamera>>,
) {
    for entity in &cam_query {
        commands.entity(entity).despawn();
    }
    dbg!("Cleaning up MainMenu entities");
}
