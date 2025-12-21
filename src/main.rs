mod peripherals;
mod entity_pipeline;
mod game;
mod interaction_modes;
mod interactive_menu;
mod menu;

use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{DefaultPlugins, diagnostic::{FrameTimeDiagnosticsPlugin}, prelude::* };
use bevy_framepace::*;

use peripherals::*;
use crate::{game::pause_menu, interaction_modes::{InteractionMode, InteractionModeType}};

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
    Paused,
}

#[derive(Resource)]
struct SetMaxFps {
    fps: f64,
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
        .insert_resource(SetMaxFps {
            fps: 120.0,
        })
        .add_systems(Startup, setup_camera)
        .add_plugins((menu::menu_plugin, game::game_plugin, pause_menu::pause_menu_plugin))
        .add_systems(Update, (game::set_max_fps, game::fps_counter))
        .run();
}

// fn setup(
//     mut commands: Commands,
// ) {
//     commands.spawn(Camera2d);
// }
