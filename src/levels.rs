
use bevy::{app::{App, Update}, ecs::{component::Component, schedule::{IntoScheduleConfigs, common_conditions::not}, system::{Res, ResMut}}, input::{ButtonInput, keyboard::KeyCode}, state::{app::AppExtStates, condition::in_state, state::{NextState, OnEnter, State, States}}};
use strum::{IntoEnumIterator, EnumIter};

use crate::{GameState, SimulationState, levels::level_four::LevelFourState, menus::pause_menu::InGameMenuState};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States, EnumIter)]
pub enum LevelState {
    ONE,
    TWO,
    THREE,
    FOUR,
    #[default]
    Disabled,
}

#[derive(Component)]
pub struct LevelsFlyCamera;

pub fn levels_plugin(
    app: &mut App,
) {
    app
        .init_state::<LevelState>()
        .add_systems(OnEnter(GameState::Levels), levels_setup)
        .add_plugins((
            level_one::level_one_plugin,
            level_two::level_two_plugin,
            level_three::level_three_plugin,
            level_four::level_four_plugin,
        ))
        .add_systems(Update, level_action.run_if(in_state(GameState::Levels)).run_if(not(in_state(LevelFourState::Loading))));
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
    level_state: Res<State<LevelState>>,
    mut next_level_state: ResMut<NextState<LevelState>>,
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
    } else if keyboard_input.just_pressed(KeyCode::Tab) {
        let current_level = *level_state.get();

        // Gather all levels as a Vec, excluding Disabled
        let levels: Vec<_> = LevelState::iter().filter(|l| *l != LevelState::Disabled).collect();

        // Retrieve the index of the current level and move to the next
        if let Some(pos) = levels.iter().position(|&l| l == current_level) {
            let next_idx = (pos + 1) % levels.len();
            next_level_state.set(levels[next_idx]);
        }
    }
}

/// Collection of Helper functions/components/resources to assist in Level development
mod level_helpers {
    use bevy::{ecs::{message::MessageReader, system::ResMut}, input::mouse::MouseWheel};

    use crate::{interactions::CursorDistance};

    // #[derive(Resource, Default)]
    // pub(crate) struct LevelsCameraOrientation {
    //     pub(crate) yaw: f32,
    //     pub(crate) pitch: f32,
    // }

    // #[derive(Resource)]
    // pub(crate) struct LevelsCameraSettings {
    //     pub(crate) speed: f32,
    //     pub(crate) sensitivity: f32,
    //     pub(crate) zoom_speed: f32,
    // }

    // #[derive(Resource)]
    // pub(crate) struct LevelsCursorDistance(pub(crate) f32);

    // pub(crate) fn keyboard_movement(
    //     keyboard_input: Res<ButtonInput<KeyCode>>,
    //     time: Res<Time>,
    //     settings: Res<LevelsCameraSettings>,
    //     mut query: Query<&mut Transform, With<LevelsFlyCamera>>,
    // ) {
    //     for mut transform in &mut query {
    //         let mut direction = Vec3::ZERO;

    //         // local forward and right vectors relative to camera
    //         let forward = -transform.local_z();
    //         let right = transform.right();

    //         // WASD movement
    //         if keyboard_input.pressed(KeyCode::KeyW) {
    //             direction += *forward;
    //         }
    //         if keyboard_input.pressed(KeyCode::KeyS) {
    //             direction -= *forward;
    //         }
    //         if keyboard_input.pressed(KeyCode::KeyA) {
    //             direction -= *right;
    //         }
    //         if keyboard_input.pressed(KeyCode::KeyD) {
    //             direction += *right;
    //         }

    //         // Up/Down
    //         if keyboard_input.pressed(KeyCode::Space) {
    //             direction += Vec3::Y;
    //         }
    //         if keyboard_input.pressed(KeyCode::ShiftLeft) {
    //             direction -= Vec3::Y;
    //         }

    //         if direction.length_squared() > 0.0 {
    //             direction = direction.normalize();
    //             transform.translation += direction * settings.speed * time.delta_secs();
    //         }
    //     }
    // }

    // /// Handles mouse movement for looking around
    // pub(crate) fn mouse_look(
    //     mut mouse_events: MessageReader<MouseMotion>,
    //     mouse_input: Res<ButtonInput<MouseButton>>,
    //     settings: Res<LevelsCameraSettings>,
    //     mut orientation: ResMut<LevelsCameraOrientation>,
    //     mut query: Query<&mut Transform, With<LevelsFlyCamera>>,
    // ) {
    //     let mut delta = Vec2::ZERO;
    //     if mouse_input.pressed(MouseButton::Middle) {
    //         for event in mouse_events.read() {
    //             delta += event.delta;
    //         }
    //     }

    //     if delta.length_squared() == 0.0 {
    //         return;
    //     }

    //     // update yaw and pitch
    //     orientation.yaw -= delta.x * settings.sensitivity;
    //     orientation.pitch -= delta.y * settings.sensitivity;
    //     orientation.pitch = orientation.pitch.clamp(-1.54, 1.54); // prevent flipping

    //     // apply rotation to camera transformation
    //     for mut transform in &mut query {
    //         transform.rotation = Quat::from_axis_angle(Vec3::Y, orientation.yaw) * Quat::from_axis_angle(Vec3::X, orientation.pitch);
    //     }
    // }

    // /// Handles mouse scroll wheel for zooming in/out of camera
    // pub(crate) fn mouse_scroll(
    //     mut scroll_events: MessageReader<MouseWheel>,
    //     time: Res<Time>,
    //     settings: Res<LevelsCameraSettings>,
    //     mut query: Query<&mut Transform, With<LevelsFlyCamera>>,
    // ) {
    //     let mut scroll_delta = 0.0;
    //     for event in scroll_events.read() {
    //         // scroll up = zoom in
    //         scroll_delta += event.y
    //     }

    //     if scroll_delta.abs() < f32::EPSILON {
    //         return;
    //     }

    //     for mut transform in &mut query {
    //         let forward = transform.forward();
    //         transform.translation += forward * scroll_delta * settings.zoom_speed * time.delta_secs();
    //     }
    // }

    pub(crate) fn scroll_control(
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
}

/// Spawn Cubes
mod level_one {
    use std::time::Duration;

    use avian3d::prelude::{RigidBody, Sleeping};
    use bevy::{prelude::*, time::common_conditions::on_timer};

    use crate::{SimulationState, entity_pipeline::{on_level_scene_spawn, on_shape_scene_spawn}, game::ExampleViewports, levels::LevelState};

    pub fn level_one_plugin(
        app: &mut App,
    ) {
        app
            .insert_resource(EntityStats::default())
            .add_systems(OnEnter(LevelState::ONE), (level_one_setup, initialize_cam))
            .add_systems(Update, (
                spawn_cubes.run_if(on_timer(Duration::from_secs_f32(0.5))),
                rotate_level_one_cam,
            ).run_if(in_state(LevelState::ONE)).run_if(in_state(SimulationState::Running).or(in_state(SimulationState::Paused))))
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

    #[derive(Component)]
    struct ActiveEntityCountSpawnedText;

    #[derive(Resource, Debug)]
    struct EntityStats {
        count: i32,
        active_count: i32,
    }
    impl Default for EntityStats {
        fn default() -> Self {
            Self { 
                count: 0,
                active_count: 0,
            }
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
        entity_stats: Res<EntityStats>,
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
        )).observe(on_level_scene_spawn);

        commands.spawn((
            Text::new("Level One: Spawning Cubes"),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 30.0,
                ..default()
            },
            Node {
                margin: UiRect { 
                    left: auto(),
                    right: auto(),
                    top: Val::Px(20.0),
                    ..default()},
                ..default()
            },
            OnLevelOneScreen,
        ));

        commands.spawn((
            Text::new("Press TAB to go to next chapter"),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 20.0,
                ..default()
            },
            Node {
                position_type: PositionType::Relative,
                margin: UiRect {
                    left: auto(),
                    right: auto(),
                    ..default()
                },
                bottom: px(5),
                ..default()
            },
            OnLevelOneScreen,
        ));

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
                ..default()
            },
            OnLevelOneScreen,
        )).with_child((
            TextSpan::new(format!("{:?}", entity_stats.count)),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            EntitySpawnedText,
        ));

        commands.spawn((
            Text::new("Active Entities: "),
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
            },
            OnLevelOneScreen,
        )).with_child((
            TextSpan::new(format!("{:?}", entity_stats.active_count)),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            ActiveEntityCountSpawnedText,
        ));
    }

    /// Upon exiting Level One, this cleans up all entities so they are not spilled into another GameState
    fn level_one_cleanup(
        mut commands: Commands,
        query: Query<Entity, With<OnLevelOneScreen>>,
        mut entity_stats: ResMut<EntityStats>,
    ) {
        entity_stats.count = 0;
        entity_stats.active_count = 0;
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }

    /// Spawns Cubes entities into the scene
    fn spawn_cubes(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut query: Single<&mut TextSpan, With<EntitySpawnedText>>,
        mut query_active_entities_text: Single<&mut TextSpan, (With<ActiveEntityCountSpawnedText>, Without<EntitySpawnedText>)>,
        active_entities: Query<Entity, (With<RigidBody>, Without<Sleeping>)>,
        mut entity_count: ResMut<EntityStats>,
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
        let active_count = active_entities.iter().count();
        query.0 = format!("{:?}", &entity_count.count);
        query_active_entities_text.0 = format!("{:?}", &active_count);
    }

    // fn count_active_bodies(
    //     query: Query<Entity, (With<RigidBody>, Without<Sleeping>)>
    // ) {
    //     let active_count = query.iter().count();
    //     info!("Active Rigid Bodies: {}", active_count);
    // }
}

/// Impulse Force
mod level_two {
    use bevy::{app::{App, Update}, camera::{Camera3d, visibility::Visibility}, color::{self, Color}, ecs::{children, component::Component, entity::Entity, query::{Changed, With, Without}, schedule::{IntoScheduleConfigs, SystemCondition}, system::{Commands, Query, Res, ResMut, Single}}, gltf::GltfAssetLabel, light::PointLight, math::Vec3, prelude::SpawnRelated, scene::SceneRoot, state::{condition::in_state, state::{OnEnter, OnExit}}, text::{TextColor, TextFont, TextLayout, TextSpan}, transform::components::Transform, ui::{AlignItems, BackgroundColor, Display, FlexDirection, Interaction, JustifyContent, Node, PositionType, UiRect, Val, auto, percent, px, vh, widget::{Button, Text}}, utils::default};
    use bevy_asset::AssetServer;
    use rand::Rng;

    use crate::{SimulationState, entity_pipeline::{on_level_scene_spawn, on_structure_scene_spawn}, game::ExampleViewports, interactions::{CursorDistance, ExampleLabel, ImpulseCoords, ImpulseCursor, ImpulseSettings, apply_force, draw_impulse_cursor, set_impulse_cursor_visibility}, levels::{LevelState, LevelsFlyCamera, level_helpers::scroll_control}};

    pub fn level_two_plugin(
        app: &mut App,
    ) {
        app
            .add_systems(OnEnter(LevelState::TWO), (level_two_setup, initialize_cam))
            .insert_resource(ImpulseSettings::default())
            .insert_resource(CursorDistance(10.0))
            .add_systems(Update, (
                scroll_control,
                draw_impulse_cursor,
                apply_force,
                lvl_two_button_system,
                lvl_two_action_controls,
                track_impulse_settings,
                set_impulse_cursor_visibility::<true>,
            ).run_if(in_state(LevelState::TWO)).run_if(in_state(SimulationState::Running).or(in_state(SimulationState::Paused))))
            .add_systems(OnExit(LevelState::TWO), level_two_cleanup);
    }

    /// Tagged to the camera in Level Two
    #[derive(Component)]
    struct LevelTwoCamera;

    /// Tagged to all entities in the Level Two to handle cleanup when leaving State
    #[derive(Component)]
    struct OnLevelTwoScreen;

    /// Tag to buttons that are selected in the UI
    #[derive(Component)]
    struct SelectedOption;

    /// Tags the blast_strength text in the UI so it can be updated
    #[derive(Component)]
    struct BlastStrengthTag;

    /// Tags the blast_radius text in the UI so it can be updated
    #[derive(Component)]
    struct BlastRadiusTag;

    /// Tag the structures spawned in Level Two to handle resetting the scene
    #[derive(Component)]
    struct Lvl2StructureTag;

    /// All different actions that are tagged to the buttons in the UI
    #[derive(Component)]
    enum BlastControlsButtonAction {
        BlastRadiusDecrease,
        BlastRadiusIncrease,
        BlastStrengthDecrease,
        BlastStrengthIncrease,
        ResetScene,
    }

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    /// Create the LevelTwo Camera on setup
    fn initialize_cam(
        mut commands: Commands,
    ) {
        commands.spawn((
            Camera3d::default(),
            ExampleViewports::_PerspectiveMain,
            Transform::from_xyz(0.0, 10.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
            LevelTwoCamera,
            OnLevelTwoScreen,
            LevelsFlyCamera,
        ));
    }

    fn level_two_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        impulse_settings: Res<ImpulseSettings>,
    ) {
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 10.0, 4.0),
            OnLevelTwoScreen,
        ));
        commands.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0)
                    .from_asset("maps.glb"),
                )
            ),
            OnLevelTwoScreen,
        )).observe(on_level_scene_spawn);

        let impulse_ball = commands.spawn((
            SceneRoot(
                asset_server.load(GltfAssetLabel::Scene(5).from_asset("shapes.glb"))
            ),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Hidden,
            OnLevelTwoScreen,
        ))
        .insert(ImpulseCursor).id();

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
                OnLevelTwoScreen,
            )).insert(ImpulseCursor);
        };
        impulse_label(impulse_ball, "┌─ Impulse: (0.00, 0.00, 0.00)");

        let mut rng = rand::rng();
        // let structure_nums: Vec<i32> = (0..3).collect();
        // let coord_nums: Vec<i32> = (0..25).collect();
        for _ in 0..5 {
            commands.spawn((
                SceneRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(rng.random_range(0..2))
                        .from_asset("structures.glb")
                    )
                ),
                Transform::from_xyz(rng.random_range(-10.0..10.0), 0.0, rng.random_range(0.0..10.0)),
                OnLevelTwoScreen,
                Lvl2StructureTag,
            )).observe(on_structure_scene_spawn);
        }

        // justify-content: aligns items horizontally
        // align-items: aligns items vertically

        commands.spawn((
            Text::new("Level Two: Impulse Forces"),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 30.0,
                ..default()
            },
            Node {
                margin: UiRect {
                    left: auto(),
                    right: auto(),
                    top: Val::Px(20.0),
                    ..default()
                },
                ..default()
            },
            OnLevelTwoScreen,
        ));
        commands.spawn((
            Text::new(
                "Demonstrates impulse forces effects on dynamic rigid bodies."
            ),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 20.0,
                ..default()
            },
            Node {
                position_type: PositionType::Relative,
                margin: UiRect {
                    left: auto(),
                    right: auto(),
                    ..default()
                },
                ..default()
            },
            OnLevelTwoScreen,
        ));

        let button_node = Node {
            width: px(50),
            height: px(30),
            margin: UiRect::all(px(10)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_font = TextFont {
            font_size: 20.0,
            ..default()
        };

        commands.spawn((
            Node { // container
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                height: vh(100),
                width: percent(100.0),
                ..default()
            },
            OnLevelTwoScreen,
            children![
                (
                    Text::new("The cursor's coordinates indicate where the force where be applied.It is recommended to apply forces around the structures to see how they are effected. Go ahead and scroll the cursor to z=-10.0 to see the structures fall forward."),
                    TextFont {
                        font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                        font_size: 12.0,
                        ..default()
                    },
                ),
                (
                    Node { // box
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        width: percent(100),
                        height: percent(15),
                        ..default()
                    },
                    BackgroundColor(color::palettes::css::CRIMSON.into()),
                    children![
                        (
                            Node { // row-buttons
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                column_gap: px(75),
                                top: px(10),
                                bottom: px(20),
                                ..default()
                            },
                            children![
                                (
                                    Node { // blast-radius-buttons
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::FlexStart,
                                        ..default()
                                    },
                                    children![
                                        (
                                            Text::new("Blast Radius: "),
                                            TextFont {
                                                font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                font_size: 30.0,
                                                ..default()
                                            },
                                            children![(
                                                TextSpan::new(format!("{:?}", impulse_settings.blast_radius)),
                                                TextFont {
                                                    font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                    font_size: 30.0,
                                                    ..default()
                                                },
                                                BlastRadiusTag,
                                            )]
                                        ),
                                        (
                                            Node { // radius-button-row
                                                display: Display::Flex,
                                                flex_direction: FlexDirection::Row,
                                                column_gap: px(5.0),
                                                ..default()
                                            },
                                            children![
                                                (
                                                    Button,
                                                    button_node.clone(),
                                                    BackgroundColor(NORMAL_BUTTON),
                                                    BlastControlsButtonAction::BlastRadiusDecrease,
                                                    children![
                                                        (
                                                            Text::new("-"),
                                                            button_text_font.clone(),
                                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                                        )
                                                    ]
                                                ),
                                                (
                                                    Button,
                                                    button_node.clone(),
                                                    BackgroundColor(NORMAL_BUTTON),
                                                    BlastControlsButtonAction::BlastRadiusIncrease,
                                                    children![
                                                        (
                                                            Text::new("+"),
                                                            button_text_font.clone(),
                                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                                        )
                                                    ]
                                                ),
                                            ],
                                        )
                                    ]
                                ),
                                (
                                    Node { // blast-strength-buttons
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::FlexStart,
                                        ..default()
                                    },
                                    children![
                                        (
                                            Text::new("Blast Strength: "),
                                            TextFont {
                                                font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                font_size: 30.0,
                                                ..default()
                                            },
                                            children![(
                                                TextSpan::new(format!("{:?}", impulse_settings.max_force)),
                                                TextFont {
                                                    font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                    font_size: 30.0,
                                                    ..default()
                                                },
                                                BlastStrengthTag,
                                            )]
                                        ),
                                        (
                                            Node { // strength-button-row
                                                display: Display::Flex,
                                                flex_direction: FlexDirection::Row,
                                                column_gap: px(5.0),
                                                ..default()
                                            },
                                            children![
                                                (
                                                    Button,
                                                    button_node.clone(),
                                                    BackgroundColor(NORMAL_BUTTON),
                                                    BlastControlsButtonAction::BlastStrengthDecrease,
                                                    children![
                                                        (
                                                            Text::new("-"),
                                                            button_text_font.clone(),
                                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                                        )
                                                    ]
                                                ),
                                                (
                                                    Button,
                                                    button_node.clone(),
                                                    BackgroundColor(NORMAL_BUTTON),
                                                    BlastControlsButtonAction::BlastStrengthIncrease,
                                                    children![
                                                        (
                                                            Text::new("+"),
                                                            button_text_font.clone(),
                                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                                        )
                                                    ]
                                                ),
                                            ],
                                        ),
                                    ]
                                ),
                                (
                                    Node { // reset-scene-buttons
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::FlexStart,
                                        ..default()
                                    },
                                    children![
                                        (
                                            Text::new("Reset Scene"),
                                            TextFont {
                                                font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                font_size: 30.0,
                                                ..default()
                                            },
                                        ),
                                        (
                                            Button,
                                            button_node.clone(),
                                            BackgroundColor(NORMAL_BUTTON),
                                            BlastControlsButtonAction::ResetScene,
                                            children![
                                                (
                                                    Text::new(" "),
                                                    button_text_font.clone(),
                                                )
                                            ]
                                        )
                                    ]
                                ),
                            ]
                        ),
                    ]
                ),
            ]
        ));
    }

    /// System used for adding visual interactions to buttons in UI
    fn lvl_two_button_system(
        mut interaction_query: Query<(&Interaction, &mut BackgroundColor, Option<&SelectedOption>), (Changed<Interaction>, With<Button>)>
    ) {
        for (interaction, mut background_color, selected) in &mut interaction_query {
            *background_color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    /// Manages controls to update the value of blast_radius and blast_strength and support resetting the scene
    fn lvl_two_action_controls(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        interaction_query: Query<(&Interaction, &mut BlastControlsButtonAction), (Changed<Interaction>, With<Button>)>,
        mut impulse_settings: ResMut<ImpulseSettings>,
        structure_query: Query<Entity, With<Lvl2StructureTag>>,
    ) {
        for (interaction, blast_controls_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match blast_controls_button_action {
                    BlastControlsButtonAction::BlastRadiusDecrease => {
                        impulse_settings.blast_radius -= 5.0;
                    }
                    BlastControlsButtonAction::BlastRadiusIncrease => {
                        impulse_settings.blast_radius += 5.0;
                    }
                    BlastControlsButtonAction::BlastStrengthDecrease => {
                        impulse_settings.max_force -= 5.0;
                    }
                    BlastControlsButtonAction::BlastStrengthIncrease => {
                        impulse_settings.max_force += 5.0;
                    }
                    BlastControlsButtonAction::ResetScene => {
                        // Delete all structures in the level
                        for entity in &structure_query {
                            commands.entity(entity).despawn();
                        }

                        // run same logic to respawn all structures in random locations
                        let mut rng = rand::rng();
                        for _ in 0..5 {
                            commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(rng.random_range(0..2))
                                        .from_asset("structures.glb")
                                    )
                                ),
                                Transform::from_xyz(rng.random_range(-10.0..10.0), 0.0, rng.random_range(0.0..10.0)),
                                OnLevelTwoScreen,
                                Lvl2StructureTag,
                            )).observe(on_structure_scene_spawn);
                        }
                    }
                }
            }
        }
    }

    /// Used to update the value of blast_radius and blast_strength in the UI
    fn track_impulse_settings(
        mut blast_radius_text_query: Single<&mut TextSpan, (With<BlastRadiusTag>, Without<BlastStrengthTag>)>,
        mut blast_strength_text_query: Single<&mut TextSpan, (With<BlastStrengthTag>, Without<BlastRadiusTag>)>,
        impulse_settings: Res<ImpulseSettings>,
    ) {
        blast_radius_text_query.0 = format!("{:?}", impulse_settings.blast_radius);
        blast_strength_text_query.0 = format!("{:?}", impulse_settings.max_force);
    }

    fn level_two_cleanup(
        mut commands: Commands,
        query: Query<Entity, With<OnLevelTwoScreen>>,
        mut impulse_settings: ResMut<ImpulseSettings>,
    ) {
        impulse_settings.blast_radius = 50.0;
        impulse_settings.max_force = 100.0;
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}

/// Wrecker Ball
mod level_three {
    use avian3d::prelude::{Collider, RigidBody};
    use bevy::{color, prelude::*};
    use rand::Rng;

    use crate::{SimulationState, entity_pipeline::{on_level_scene_spawn, on_structure_scene_spawn}, game::ExampleViewports, interactions::{CursorDistance, ExampleLabel, WreckerCoords, WreckerCursor, center_cursor, draw_wrecker_cursor, set_wrecker_cursor_visibility}, levels::{LevelState, LevelsFlyCamera, level_helpers::scroll_control}};
    
    pub fn level_three_plugin(
        app: &mut App,
    ) {
        app
            .add_systems(OnEnter(LevelState::THREE), (level_three_setup, initialize_cam))
            .insert_resource(CursorDistance(10.0))
            .add_systems(Update, (
                scroll_control,
                draw_wrecker_cursor,
                lvl_three_button_system,
                lvl_three_action_controls,
                track_wrecker_settings,
                set_wrecker_cursor_visibility::<true>,
            ).run_if(in_state(LevelState::THREE)).run_if(in_state(SimulationState::Running).or(in_state(SimulationState::Paused))))
            .add_systems(OnExit(LevelState::THREE), (level_three_cleanup, center_cursor));
    }

    #[derive(Component)]
    struct OnLevelThreeScreen;

    #[derive(Component)]
    struct SelectedOption;

    #[derive(Component)]

    struct LevelThreeCamera;

    #[derive(Component)]
    struct Lvl3StructureTag;

    #[derive(Component)]
    struct WreckerBallScaleTag;

    #[derive(Component)]
    enum WreckerControlsBunttonAction {
        WreckerScaleDecrease,
        WreckerScaleIncrease,
        ResetScene,
    }

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    fn initialize_cam(
        mut commands: Commands,
    ) {
        commands.spawn((
            Camera3d::default(),
            ExampleViewports::_PerspectiveMain,
            Transform::from_xyz(0.0, 10.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
            LevelThreeCamera,
            OnLevelThreeScreen,
            LevelsFlyCamera,
        ));
    }

    fn level_three_setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 10.0, 4.0),
            OnLevelThreeScreen,
        ));
        commands.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0)
                    .from_asset("maps.glb"),
                )
            ),
            OnLevelThreeScreen,
        )).observe(on_level_scene_spawn);

        let mut rng = rand::rng();
        // let structure_nums: Vec<i32> = (0..3).collect();
        // let coord_nums: Vec<i32> = (0..25).collect();
        for _ in 0..5 {
            commands.spawn((
                SceneRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(rng.random_range(0..2))
                        .from_asset("structures.glb")
                    )
                ),
                Transform::from_xyz(rng.random_range(-10.0..10.0), 0.0, rng.random_range(0.0..10.0)),
                OnLevelThreeScreen,
                Lvl3StructureTag,
            )).observe(on_structure_scene_spawn);
        }

        let text_style = TextFont {
            font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
            ..Default::default()
        };
        
        let sphere = meshes.add(Sphere::new(0.5));
        let wrecker_ball = commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
            Transform::from_xyz(0.0, 10.0, 0.0),
            Collider::sphere(0.5),
            RigidBody::Kinematic,
            Visibility::Hidden,
            OnLevelThreeScreen,
        ))
        .insert(WreckerCursor).id();

        let label_text_style = (text_style.clone(), TextColor(color::palettes::css::ORANGE.into()));

        let mut wrecker_label = |entity: Entity, label: &str| {
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ExampleLabel { entity },
                children![(
                    Text::new(label),
                    label_text_style.clone(),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::ZERO,
                        ..default()
                    },
                    TextLayout::default().with_no_wrap(),
                    WreckerCoords,
                )],
                Visibility::Hidden,
                OnLevelThreeScreen,
            )).insert(WreckerCursor);
        };
        wrecker_label(wrecker_ball, "┌─ Wrecker: (0.00, 0.00, 0.00)");

        commands.spawn((
            Text::new("Level Three: Wrecker Ball"),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 30.0,
                ..default()
            },
            Node {
                margin: UiRect {
                    left: auto(),
                    right: auto(),
                    top: Val::Px(20.0),
                    ..default()
                },
                ..default()
            },
            OnLevelThreeScreen,
        ));
        commands.spawn((
            Text::new(
                "Demonstrates Kinematic effects on dynamic rigid bodies."
            ),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 20.0,
                ..default()
            },
            Node {
                position_type: PositionType::Relative,
                margin: UiRect {
                    left: auto(),
                    right: auto(),
                    ..default()
                },
                ..default()
            },
            OnLevelThreeScreen,
        ));

        let button_node = Node {
            width: px(50),
            height: px(30),
            margin: UiRect::all(px(10)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_font = TextFont {
            font_size: 20.0,
            ..default()
        };

        commands.spawn((
            Node { // container
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                height: vh(100),
                width: percent(100.0),
                ..default()
            },
            OnLevelThreeScreen,
            children![
                (
                    Text::new("The cursor's coordinates indicate where the Wrecker ball is at the moment. The amount of velocity you move your cursor will effect the amount of force the Wrecker ball has on rigid bodies."),
                    TextFont {
                        font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                        font_size: 12.0,
                        ..default()
                    },
                ),
                (
                    Node { // box
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        width: percent(100),
                        height: percent(15),
                        ..default()
                    },
                    BackgroundColor(color::palettes::css::CRIMSON.into()),
                    children![
                        (
                            Node { // row-buttons
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                column_gap: px(75),
                                top: px(10),
                                bottom: px(20),
                                ..default()
                            },
                            children![
                                (
                                   Node { // scale-buttons
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                   },
                                   children![
                                        (
                                            Text::new("Wrecker Scale: "),
                                            TextFont {
                                                font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                font_size: 30.0,
                                                ..default()
                                            },
                                            children![(
                                                TextSpan::new("1.0"),
                                                TextFont {
                                                    font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                    font_size: 30.0,
                                                    ..default()
                                                },
                                                WreckerBallScaleTag,
                                            )]
                                        ),
                                        (
                                            Node { // radius-button-row
                                                display: Display::Flex,
                                                flex_direction: FlexDirection::Row,
                                                column_gap: px(5.0),
                                                ..default()
                                            },
                                            children![
                                                (
                                                    Button,
                                                    button_node.clone(),
                                                    BackgroundColor(NORMAL_BUTTON),
                                                    WreckerControlsBunttonAction::WreckerScaleDecrease,
                                                    children![
                                                        (
                                                            Text::new("-"),
                                                            button_text_font.clone(),
                                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                                        )
                                                    ]
                                                ),
                                                (
                                                    Button,
                                                    button_node.clone(),
                                                    BackgroundColor(NORMAL_BUTTON),
                                                    WreckerControlsBunttonAction::WreckerScaleIncrease,
                                                    children![
                                                        (
                                                            Text::new("+"),
                                                            button_text_font.clone(),
                                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                                        )
                                                    ]
                                                ),
                                            ],
                                        ),
                                    ]
                                ),
                                (
                                    Node { // reset-scene
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    children![
                                        (
                                            Text::new("Reset Scene"),
                                            TextFont {
                                                font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                font_size: 30.0,
                                                ..default()
                                            },
                                        ),
                                        (
                                            Button,
                                            button_node.clone(),
                                            BackgroundColor(NORMAL_BUTTON),
                                            WreckerControlsBunttonAction::ResetScene,
                                            children![
                                                (
                                                    Text::new(" "),
                                                    button_text_font.clone(),
                                                )
                                            ]
                                        )
                                    ]
                                ),
                            ]
                        )
                    ]
                )
            ]
        ));
    }

    /// System used for adding visual interactions to buttons in UI
    fn lvl_three_button_system(
        mut interaction_query: Query<(&Interaction, &mut BackgroundColor, Option<&SelectedOption>), (Changed<Interaction>, With<Button>)>
    ) {
        for (interaction, mut background_color, selected) in &mut interaction_query {
            *background_color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    /// Manages controls to update the value of the wrecker ball scale and support resetting the scene
    fn lvl_three_action_controls(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        interaction_query: Query<(&Interaction, &mut WreckerControlsBunttonAction), (Changed<Interaction>, With<Button>)>,
        mut wrecker_query: Single<&mut Transform, With<WreckerCursor>>,
        structure_query: Query<Entity, With<Lvl3StructureTag>>,
    ) {
        for (interaction, wrecker_controls_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match wrecker_controls_button_action {
                    WreckerControlsBunttonAction::WreckerScaleDecrease => {
                        if wrecker_query.scale == vec3(1.0, 1.0, 1.0) {
                            return;
                        }
                        wrecker_query.scale -= 1.0;
                    }
                    WreckerControlsBunttonAction::WreckerScaleIncrease => {
                        wrecker_query.scale += 1.0;
                    }
                    WreckerControlsBunttonAction::ResetScene => {
                        // Delete all structures in the level
                        for entity in &structure_query {
                            commands.entity(entity).despawn();
                        }

                        // run same logic to respawn all structures in random locations
                        let mut rng = rand::rng();
                        for _ in 0..5 {
                            commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(rng.random_range(0..2))
                                        .from_asset("structures.glb")
                                    )
                                ),
                                Transform::from_xyz(rng.random_range(-10.0..10.0), 0.0, rng.random_range(0.0..10.0)),
                                OnLevelThreeScreen,
                                Lvl3StructureTag,
                            )).observe(on_structure_scene_spawn);
                        }
                    }
                }
            }
        }
    }

    fn track_wrecker_settings(
        wrecker_query: Single<&Transform, With<WreckerCursor>>,
        mut wrecker_scale_text_query: Single<&mut TextSpan, With<WreckerBallScaleTag>>,
    ) {
        wrecker_scale_text_query.0 = format!("{:.1}", wrecker_query.scale.x);
    }

    fn level_three_cleanup(
        mut commands: Commands,
        query: Query<Entity, With<OnLevelThreeScreen>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}

/// Asteroids
mod level_four {
    use std::time::Duration;

    use avian3d::prelude::{Collider, ConstantForce, Mass, RigidBody};
    use bevy::{color, prelude::*, time::common_conditions::on_timer};
    use bevy_asset::{AssetServer, Assets};
    use rand::Rng;
    use strum::EnumIter;

    use crate::{SimulationState, entity_pipeline::{StructureBlock, on_level_scene_spawn, on_structure_scene_spawn}, game::ExampleViewports, interactions::center_cursor, levels::LevelState};

    
    pub fn level_four_plugin(
        app: &mut App,
    ) {
        app
            .init_state::<LevelFourState>()
            .add_systems(OnEnter(LevelState::FOUR), level_four_setup)
            .add_systems(OnEnter(LevelFourState::Loading), setup_delay_timer)
            .add_systems(Update, check_delay_timer.run_if(in_state(LevelFourState::Loading)))
            .add_systems(OnEnter(LevelFourState::Running), (initialize_cam, cleanup_loading_screen, level_four_text))
            .add_systems(Update, (
                rotate_level_four_cam,
                spawn_asteroids.run_if(on_timer(Duration::from_secs_f32(0.2))),
                despawn_asteroids,
                lvl_four_button_system,
                lvl_four_action_controls
            ).run_if(in_state(LevelFourState::Running)).run_if(in_state(LevelState::FOUR)).run_if(in_state(SimulationState::Running).or(in_state(SimulationState::Paused))))
            .add_systems(OnExit(LevelState::FOUR), (level_four_cleanup, center_cursor));
    }

    /// Keeps track of internal states inside of Level 4
    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States, EnumIter)]
    pub(crate) enum LevelFourState {
        #[default]
        Start,
        Loading,
        Running,
    }

    /// Used to delay the execution of the level until all entities are loaded
    #[derive(Component)]
    struct DelayTimer(Timer);

    /// Tags entities loaded in Level four loading state
    #[derive(Component)]
    struct OnLevelFourLoadingScreen;

    /// Tags the primary 3D camera in Level Four
    #[derive(Component)]
    struct LevelFourCamera;

    /// Tags entities loaded in the Level four running state
    #[derive(Component)]
    struct OnLevelFourScreen;

    #[derive(Component)]
    struct SelectedOption;

    /// Tags structure entity in Level four
    #[derive(Component)]
    struct Lvl4StructureTag;

    /// Tags asteroid entities
    #[derive(Component)]
    struct AsteroidTag;

    #[derive(Component)]
    enum AsteroidsButtonControls {
        ResetScene,
    }

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    /// Creates the main 3D camera
    fn initialize_cam(
        mut commands: Commands,
    ) {
        commands.spawn((
            Camera3d::default(),
            ExampleViewports::_PerspectiveMain,
            Transform::from_xyz(0.0, 20.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y),
            LevelFourCamera,
            OnLevelFourScreen,
        ));
    }

    /// Continuously rotates the Camera around the origin
    fn rotate_level_four_cam(
        time: Res<Time>,
        mut query: Query<&mut Transform, With<LevelFourCamera>>,
    ) {
        let radius = 80.0;
        let speed = 0.2;
        let vertical_offset = 20.0;

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

    /// Loads all entities in Level four
    fn level_four_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut next_level_four_state: ResMut<NextState<LevelFourState>>,
    ) {
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 10.0, 4.0),
            OnLevelFourScreen,
        ));
        commands.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0)
                    .from_asset("maps.glb"),
                )
            ),
            OnLevelFourScreen,
        )).observe(on_level_scene_spawn);

        let mut rng = rand::rng();
        for _ in 0..75 {
            commands.spawn((
                SceneRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(rng.random_range(0..2))
                        .from_asset("structures.glb")
                    )
                ),
                Transform::from_xyz(rng.random_range(-50.0..50.0), 0.0, rng.random_range(-50.0..50.0)),
                OnLevelFourScreen,
                Lvl4StructureTag,
            )).observe(on_structure_scene_spawn);
        }
        next_level_four_state.set(LevelFourState::Loading);
    }

    /// Spawns all text to be loaded in Level four
    fn level_four_text(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn((
            Text::new("Level Four: Asteroids"),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 30.0,
                ..default()
            },
            Node {
                margin: UiRect {
                    left: auto(),
                    right: auto(),
                    top: Val::Px(20.0),
                    ..default()
                },
                ..default()
            },
            OnLevelFourScreen,
        ));
        commands.spawn((
            Text::new("Large scale wrecker simulation with 'Asteroids'."),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 20.0,
                ..default()
            },
            Node {
                position_type: bevy::ui::PositionType::Relative,
                margin: UiRect {
                    left: auto(),
                    right: auto(),
                    ..default()
                },
                ..default()
            },
            OnLevelFourScreen,
        ));

        let button_node = Node {
            width: px(50),
            height: px(30),
            margin: UiRect::all(px(10)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_font = TextFont {
            font_size: 20.0,
            ..default()
        };

        commands.spawn((
            Node { // container
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                height: vh(100),
                width: percent(100.0),
                ..default()
            },
            OnLevelFourScreen,
            children![
                (
                    Text::new("Asteroids spawn every 0.2 seconds in a random X position above the map and accelerate into structures. Who will be lucky to survive"),
                    TextFont {
                        font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                        font_size: 12.0,
                        ..default()
                    },
                ),
                (
                    Node { // box
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        width: percent(100),
                        height: percent(15),
                        ..default()
                    },
                    BackgroundColor(color::palettes::css::CRIMSON.into()),
                    children![ //row-buttons
                        (
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                column_gap: px(75),
                                top: px(10),
                                bottom: px(20),
                                ..default()
                            },
                            children![
                                (
                                    Node { // reset-scene
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    children![
                                        (
                                            Text::new("Reset Scene"),
                                            TextFont {
                                                font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
                                                font_size: 30.0,
                                                ..default()
                                            },
                                        ),
                                        (
                                            Button,
                                            button_node.clone(),
                                            BackgroundColor(NORMAL_BUTTON),
                                            AsteroidsButtonControls::ResetScene,
                                            children![
                                                (
                                                    Text::new(" "),
                                                    button_text_font.clone(),
                                                )
                                            ]
                                        )
                                    ]
                                )
                            ]
                        )
                    ]
                )
            ]
        ));
    }

    /// System used for adding visual interactions to buttons in UI
    fn lvl_four_button_system(
        mut interaction_query: Query<(&Interaction, &mut BackgroundColor, Option<&SelectedOption>), (Changed<Interaction>, With<Button>)>
    ) {
        for (interaction, mut background_color, selected) in &mut interaction_query {
            *background_color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    /// Manages controls to update the Asteroids scene based on button pressed
    fn lvl_four_action_controls(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut next_level_four_state: ResMut<NextState<LevelFourState>>,
        interaction_query: Query<(&Interaction, &mut AsteroidsButtonControls), (Changed<Interaction>, With<Button>)>,
        structure_query: Query<Entity, Or<(With<Lvl4StructureTag>, With<LevelFourCamera>)>>,
    ) {
        for (interaction, asteroid_controls_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match asteroid_controls_button_action {
                    AsteroidsButtonControls::ResetScene => {
                        for entity in &structure_query {
                            commands.entity(entity).despawn();
                        }
                        let mut rng = rand::rng();
                        for _ in 0..75 {
                            commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(rng.random_range(0..2))
                                        .from_asset("structures.glb")
                                    )
                                ),
                                Transform::from_xyz(rng.random_range(-50.0..50.0), 0.0, rng.random_range(-50.0..50.0)),
                                OnLevelFourScreen,
                                Lvl4StructureTag,
                            )).observe(on_structure_scene_spawn);
                        }
                        next_level_four_state.set(LevelFourState::Loading);
                    }
                }
            }
        }
    }

    /// Creates a timer preventing execution of level four until entities are done loading (4s)
    fn setup_delay_timer(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn((
            Camera3d::default(),
            OnLevelFourLoadingScreen,
            OnLevelFourScreen,
        ));
        commands.spawn((
            Text::new("Loading Level..."),
            TextFont {
                font: asset_server.load(r"fonts\FiraMono-Bold.ttf"),
                font_size: 40.0,
                ..default()
            },
            Node {
                margin: UiRect::all(auto()),
                ..default()
            },
            OnLevelFourScreen,
            OnLevelFourLoadingScreen,
        ));
        let mut timer = Timer::new(Duration::from_secs(4), TimerMode::Once);
        timer.tick(Duration::from_secs_f32(0.01));
        commands.spawn((DelayTimer(timer), OnLevelFourLoadingScreen));
    }

    /// Continuously checks Delay Timer until it is done with execution then transitons Level four state
    fn check_delay_timer(
        time: Res<Time>,
        mut commands: Commands,
        mut query: Query<(Entity, &mut DelayTimer)>,
        mut next_level_four_state: ResMut<NextState<LevelFourState>>,
    ) {
        for (entity, mut delay_timer) in &mut query {
            delay_timer.0.tick(time.delta());

            if delay_timer.0.is_finished() {
                info!("Delay finished! Moving to Running state");
                next_level_four_state.set(LevelFourState::Running);
                commands.entity(entity).despawn();
            }
        }
    }

    /// Spawns Asteroid entities into the scene
    fn spawn_asteroids(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let mut rng = rand::rng();
        // let radius = rng.random_range(0.5..2.0);
        let sphere = meshes.add(Sphere::new(0.5));
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
            Transform::from_xyz(rng.random_range(-100.0..0.0), 40.0, rng.random_range(-50.0..50.0)),
            Collider::sphere(0.5),
            Mass(500.0),
            ConstantForce::new(5000.0, 0.0, 0.0),
            RigidBody::Dynamic,
            AsteroidTag,
            OnLevelFourScreen,
            Lvl4StructureTag,
        ));
    }

    /// Despawns all asteroid entities when they fall below the floor
    fn despawn_asteroids(
        mut commands: Commands,
        query_asteroids: Query<(Entity, &GlobalTransform), Or<(With<Lvl4StructureTag>, With<StructureBlock>)>>,
    ) {
        for (entity, transform) in query_asteroids.iter() {
            if transform.translation().y < -0.5 {
                commands.entity(entity).despawn();
                info!("Despawned Rigid Body");
            }
        }
    }

    /// Cleans the loading screen entities when transitioning to running state
    fn cleanup_loading_screen(
        mut commands: Commands,
        query: Query<Entity, With<OnLevelFourLoadingScreen>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }

    /// Cleans all entities when level 4 
    fn level_four_cleanup(
        mut commands: Commands,
        query: Query<Entity, With<OnLevelFourScreen>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}
