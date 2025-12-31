
use bevy::{app::{App, Update}, ecs::{schedule::IntoScheduleConfigs, system::{Res, ResMut}}, input::{ButtonInput, keyboard::KeyCode}, state::{app::AppExtStates, condition::in_state, state::{NextState, OnEnter, State, States}}};

use crate::{GameState, SimulationState, menus::pause_menu::InGameMenuState};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum LevelState {
    ONE,
    #[default]
    Disabled,
}

pub fn levels_plugin(
    app: &mut App,
) {
    app
        .init_state::<LevelState>()
        .add_systems(OnEnter(GameState::Levels), levels_setup)
        .add_plugins(level_one::level_one_plugin)
        .add_systems(Update, level_action.run_if(in_state(GameState::Levels)));
}

fn levels_setup(
    mut levels_state: ResMut<NextState<LevelState>>,
) {
    levels_state.set(LevelState::ONE);
}

/// Cross-system function used to toggle between the Game state and the Pause state
pub fn level_action(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<SimulationState>>,
    mut next_state: ResMut<NextState<SimulationState>>,
    mut paused_menu_state: ResMut<NextState<InGameMenuState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if *state.get() == SimulationState::Running {
            next_state.set(SimulationState::Paused);
        } else {
            next_state.set(SimulationState::Running);
            paused_menu_state.set(InGameMenuState::Disabled);
        }
    }
}

mod level_one {
    use std::time::Duration;

    use bevy::{prelude::*, time::common_conditions::on_timer};

    use crate::{entity_pipeline::{on_shape_scene_spawn, on_structure_scene_spawn}, game::{ExampleViewports, SavedCameraTransforms}, levels::LevelState};

    pub fn level_one_plugin(
        app: &mut App,
    ) {
        app
            .add_systems(OnEnter(LevelState::ONE), (level_one_setup, initialize_cam))
            .add_systems(Update, spawn_cubes.run_if(on_timer(Duration::from_secs(1)).and(in_state(LevelState::ONE))))
            // .add_systems(OnExit(GameState::Levels), level_one_camera_cleanup)
            .add_systems(OnExit(LevelState::ONE), level_one_cleanup);
    }

    #[derive(Component)]
    pub struct OnLevelOneScreen;

    #[derive(Component)]
    struct LevelOneCamera;

    fn initialize_cam(
        mut commands: Commands,
        saved_cam: Res<SavedCameraTransforms>,
    ) {
        let transform = saved_cam.0.get("lvl_one_cam_last_pos")
        .cloned()
        .unwrap_or(Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y));

        commands.spawn((
            Camera3d::default(),
            ExampleViewports::_PerspectiveMain,
            transform,
            LevelOneCamera,
            OnLevelOneScreen,
        ));
    }

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
