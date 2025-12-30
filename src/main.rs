mod entity_pipeline;
mod game;
mod interactions;
mod levels;
mod menus;

use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{DefaultPlugins, diagnostic::{FrameTimeDiagnosticsPlugin}, prelude::* };
use bevy_framepace::*;

use crate::interactions::interactive_menu::cleanup_entities;

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
    Levels,
    Paused,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
enum SetFps {
    Low,
    Medium,
    High,
    Uncapped,
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
        .insert_resource(SetFps::High)
        .add_systems(OnEnter(GameState::Menu), setup)
        .add_plugins((menus::main_menu::menu_plugin, game::game_plugin, menus::pause_menu::pause_menu_plugin, levels::levels_plugin))
        .add_systems(Update, (game::set_max_fps, game::fps_counter))
        .add_systems(OnEnter(GameState::Menu), cleanup_entities)
        .add_systems(OnExit(GameState::Menu), cleanup_setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d::default(),
        SetupCamera,
    ));
}

fn cleanup_setup(
    mut commands: Commands,
    cam_query: Query<Entity, With<SetupCamera>>,
) {
    for entity in &cam_query {
        commands.entity(entity).despawn();
    }
}
