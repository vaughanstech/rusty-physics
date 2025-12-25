mod entity_pipeline;
mod game;
mod interactions;
mod menus;

use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{DefaultPlugins, diagnostic::{FrameTimeDiagnosticsPlugin}, prelude::* };
use bevy_framepace::*;

use crate::{game::setup_camera, interactions::interactive_menu::cleanup_entities};

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
        .insert_resource(SetFps::Medium)
        .add_systems(Startup, setup_camera)
        .add_plugins((menus::main_menu::menu_plugin, game::game_plugin, menus::pause_menu::pause_menu_plugin))
        .add_systems(Update, (game::set_max_fps, game::fps_counter))
        .add_systems(OnEnter(GameState::Menu), cleanup_entities)
        .run();
}
