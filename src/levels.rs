
use bevy::{app::{App, Update}, ecs::{component::Component, entity::Entity, query::With, schedule::IntoScheduleConfigs, system::{Commands, Query, Res, ResMut}}, input::{ButtonInput, keyboard::KeyCode}, log::info, state::{app::AppExtStates, condition::in_state, state::{NextState, OnEnter, OnExit, State, States}}};

use crate::{GameState, menus::pause_menu::{InGameMenuState, PauseFromState}};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum LevelState {
    ONE,
    #[default]
    Disabled,
}

#[derive(Component)]
pub struct OnLevelScreen;

pub fn levels_plugin(
    app: &mut App,
) {
    app
        .init_state::<LevelState>()
        .add_systems(OnEnter(GameState::Levels), levels_setup)
        .add_plugins(level_one::level_one_plugin)
        .add_systems(Update, level_action.run_if(in_state(GameState::Levels)))
        .add_systems(OnExit(GameState::Levels), levels_cleanup);
}

fn levels_setup(
    mut levels_state: ResMut<NextState<LevelState>>,
) {
    levels_state.set(LevelState::ONE);
}

/// Cross-system function used to toggle between the Game state and the Pause state
pub fn level_action(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    pause_state: Res<State<PauseFromState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<InGameMenuState>>,
    mut pause_from_state: ResMut<NextState<PauseFromState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Levels => {
                game_state.set(GameState::Paused);
                pause_from_state.set(PauseFromState::Levels);
                
                info!("Pausing Game");
            }
            GameState::Paused => {
                if *pause_state.get() == PauseFromState::Levels {
                    game_state.set(GameState::Levels);
                    menu_state.set(InGameMenuState::Disabled);
                    pause_from_state.set(PauseFromState::Disabled);
                    info!("Resuming Game");
                    return;
                }
            }
            _ => {}
        }
    }
}

fn levels_cleanup(
        mut commands: Commands,
        query: Query<Entity, With<OnLevelScreen>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }

mod level_one {
    use std::time::Duration;

    use bevy::{prelude::*, time::common_conditions::on_timer};

    use crate::{entity_pipeline::{on_shape_scene_spawn, on_structure_scene_spawn}, levels::LevelState};

    pub fn level_one_plugin(
        app: &mut App,
    ) {
        app
            .add_systems(OnEnter(LevelState::ONE), level_one_setup)
            .add_systems(Update, spawn_cubes.run_if(on_timer(Duration::from_secs(1)).and(in_state(LevelState::ONE))))
            .add_systems(OnExit(LevelState::ONE), level_one_cleanup);
    }

    #[derive(Component)]
    pub struct OnLevelOneScreen;

    fn level_one_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(1)
                    .from_asset("maps.glb"),
                )
            ),
            OnLevelOneScreen,
        )).observe(on_structure_scene_spawn);
    }

    fn level_one_cleanup(
        mut commands: Commands,
        query: Query<Entity, With<OnLevelOneScreen>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }

    fn spawn_cubes(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(1)
                        .from_asset("shapes.glb"),
                ),
            ),
            Transform::from_xyz(0.0, 10.0, 0.0),
            OnLevelOneScreen,
        )).observe(on_shape_scene_spawn);
    }
}
