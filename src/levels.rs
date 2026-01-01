
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

    use crate::{entity_pipeline::{on_shape_scene_spawn, on_structure_scene_spawn}, game::ExampleViewports, levels::LevelState};

    pub fn level_one_plugin(
        app: &mut App,
    ) {
        app
        .insert_resource(EntityCount::default())
            .add_systems(OnEnter(LevelState::ONE), (level_one_setup, initialize_cam))
            .add_systems(Update, (
                spawn_cubes.run_if(on_timer(Duration::from_secs_f32(0.5))),
                rotate_level_one_cam,
            ).run_if(in_state(LevelState::ONE)))
            // .add_systems(OnExit(GameState::Levels), level_one_camera_cleanup)
            .add_systems(OnExit(LevelState::ONE), level_one_cleanup);
    }

    /// Tag used to keep track of all entities in the Level One scene
    #[derive(Component)]
    pub struct OnLevelOneScreen;

    /// Tag used specifically for the Level One camera
    #[derive(Component)]
    struct LevelOneCamera;

    /// Tag used specifically for text that updates the amount of entities spawned
    #[derive(Component)]
    struct EntitySpawnedText;

    #[derive(Resource, Debug)]
    struct EntityCount {
        count: i32,
    }
    impl Default for EntityCount {
        fn default() -> Self {
            Self { count: 0 }
        }
    }

    /// Creates the LevelOne Camera on Setup
    fn initialize_cam(
        mut commands: Commands,
    ) {
        commands.spawn((
            Camera3d::default(),
            ExampleViewports::_PerspectiveMain,
            Transform::from_xyz(0.0, 10.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
            LevelOneCamera,
            OnLevelOneScreen,
        ));
    }

    /// Continuously rotates the Camera around the origin
    fn rotate_level_one_cam(
        time: Res<Time>,
        mut query: Query<&mut Transform, With<LevelOneCamera>>,
    ) {
        let radius = 40.0;
        let speed = 0.5;
        let vertical_offset = 10.0;

        // Calculate the angle based on total elapsed time
        let angle = time.elapsed_secs() * speed;

        for mut transform in &mut query {
            // Calculate new circular position on the XZ plane
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            // Apply new translation
            transform.translation = Vec3::new(x, vertical_offset, z);

            // Ensure the camera always points at the origin
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }

    /// Additional configurations to be made upon entering Level One
    fn level_one_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        entity_count: Res<EntityCount>,
    ) {
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 10.0, 4.0),
            OnLevelOneScreen,
        ));
        commands.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(1)
                    .from_asset("maps.glb"),
                )
            ),
            OnLevelOneScreen,
        )).observe(on_structure_scene_spawn);

        commands.spawn((
            Text::new(format!("Cubes Spawned: ")),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 30.0,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                bottom: px(5),
                right: px(5),
                ..default()
            }
        )).with_child((
            TextSpan::new(format!("{:?}", entity_count.count)),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            EntitySpawnedText,
        ));
    }

    /// Upon exiting Level One, this cleans up all entities so they are not spilled into another GameState
    fn level_one_cleanup(
        mut commands: Commands,
        query: Query<Entity, With<OnLevelOneScreen>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }

    /// Spawns Cubes entities into the scene
    fn spawn_cubes(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut query: Single<&mut TextSpan, With<EntitySpawnedText>>,
        mut entity_count: ResMut<EntityCount>,
    ) {
        commands.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(1)
                        .from_asset("shapes.glb"),
                ),
            ),
            Transform::from_xyz(0.0, 20.0, 0.0),
            OnLevelOneScreen,
        )).observe(on_shape_scene_spawn);
        entity_count.count += 1;
        query.0 = format!("{:?}", &entity_count.count);
    }
}
