
use bevy::{app::{App, Update}, ecs::{component::Component, schedule::IntoScheduleConfigs, system::{Res, ResMut}}, input::{ButtonInput, keyboard::KeyCode}, state::{app::AppExtStates, condition::in_state, state::{NextState, OnEnter, State, States}}};
use strum::{IntoEnumIterator, EnumIter};

use crate::{GameState, SimulationState, menus::pause_menu::InGameMenuState};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States, EnumIter)]
pub enum LevelState {
    ONE,
    TWO,
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
        ))
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
    use bevy::{ecs::{component::Component, message::MessageReader, query::With, resource::Resource, system::{Query, Res, ResMut}}, input::{ButtonInput, keyboard::KeyCode, mouse::{MouseButton, MouseMotion, MouseWheel}}, math::{Quat, Vec2, Vec3}, time::Time, transform::components::Transform};

    use crate::{interactions::CursorDistance, levels::LevelsFlyCamera};

    #[derive(Resource, Default)]
    pub(crate) struct LevelsCameraOrientation {
        pub(crate) yaw: f32,
        pub(crate) pitch: f32,
    }

    #[derive(Resource)]
    pub(crate) struct LevelsCameraSettings {
        pub(crate) speed: f32,
        pub(crate) sensitivity: f32,
        pub(crate) zoom_speed: f32,
    }

    #[derive(Resource)]
    pub(crate) struct LevelsCursorDistance(pub(crate) f32);

    pub(crate) fn keyboard_movement(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        time: Res<Time>,
        settings: Res<LevelsCameraSettings>,
        mut query: Query<&mut Transform, With<LevelsFlyCamera>>,
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
    pub(crate) fn mouse_look(
        mut mouse_events: MessageReader<MouseMotion>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        settings: Res<LevelsCameraSettings>,
        mut orientation: ResMut<LevelsCameraOrientation>,
        mut query: Query<&mut Transform, With<LevelsFlyCamera>>,
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
    pub(crate) fn mouse_scroll(
        mut scroll_events: MessageReader<MouseWheel>,
        time: Res<Time>,
        settings: Res<LevelsCameraSettings>,
        mut query: Query<&mut Transform, With<LevelsFlyCamera>>,
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
