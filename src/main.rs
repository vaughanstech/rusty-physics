
use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{DefaultPlugins, color, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, gltf::GltfMeshExtras, input::mouse::{MouseMotion, MouseWheel}, prelude::*, scene::SceneInstanceReady };
use bevy_asset::{AssetServer};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui, input::EguiWantsInput};
use bevy_framepace::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
enum ExampleViewports {
    _PerspectiveMain,
    _PerspectiveStretched,
    _PerspectiveMoving,
    _PerspectiveControl,
    _OrthographicMain,
    _OrthographicStretched,
    _OrthographicMoving,
    _OrthographicControl,
}

#[derive(Component)]
struct FlyCamera;

#[derive(Resource)]
struct CameraSettings {
    speed: f32, // camera movement speed
    sensitivity: f32, // mouse movement sensitivity
    zoom_speed: f32, // mouse scroll sensitivity
}

#[derive(Resource)]
struct SetMaxFps {
    fps: f64,
}

#[derive(Resource, Default)]
struct CameraOrientation {
    yaw: f32,
    pitch: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct BMeshExtras {
    collider: BCollider,
    rigid_body: BRigidBody,
    cube_size: Option<Vec3>,
    radius: Option<f32>,
    height: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
enum BCollider {
    TrimeshFromMesh,
    ConvexHull,
    Cuboid,
    Sphere,
    Cylinder,
}

#[derive(Debug, Serialize, Deserialize)]
enum BRigidBody {
    Static,
    Dynamic,
}

#[derive(Component)]
struct FpsText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractionModeType {
    Click,
    Impulse,
    Wrecker,
}

#[derive(Resource, PartialEq)]
struct InteractionMode(InteractionModeType);

#[derive(Resource)]
struct CursorDistance(f32);

#[derive(Component)]
struct ImpulseCursor;

#[derive(Component)]
struct WreckerCursor;

#[derive(Resource)]
struct ImpulseSettings {
    blast_radius: f32,
    max_force: f32
}
impl Default for ImpulseSettings {
    fn default() -> Self {
        Self { blast_radius: 50.0, max_force: 100.0 }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),
            FramepacePlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            EguiPlugin::default(),
        ))
        .insert_resource(CameraSettings {
            speed: 8.0,
            sensitivity: 0.002,
            zoom_speed: 30.0,
        })
        .insert_resource(SetMaxFps {
            fps: 120.0,
        })
        .insert_resource(ImpulseSettings::default())
        .insert_resource(CameraOrientation::default())
        .insert_resource(InteractionMode(InteractionModeType::Click))
        .insert_resource(CursorDistance(10.0)) // set cursor distance on spawn
        .add_systems(Startup, (setup, setup_camera))
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
            // Cursor Control/Draw runs only in Impulse Mode
            (
                scroll_control, // System to update cursor distance
                draw_cursor,                // System to draw the gizmo
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
        ))
        .run();
}

#[derive(Component)]
struct ExampleLabel {
    entity: Entity,
}

#[derive(Component)]
struct ImpulseCoords;

#[derive(Component)]
struct WreckerCoords;

fn setup(
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
            asset_server.load(GltfAssetLabel::Scene(4).from_asset("shapes.glb"))
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
        DebugRender::default().with_collider_color(Color::srgb(1.0, 1.0, 0.0)),
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

/// Initializes 3D camera
fn setup_camera(
    mut commands: Commands,
    mut orientation: ResMut<CameraOrientation>,
) {
    let position = Vec3::new(0.0, 10.0, 20.0);

    // Initializing Camera Orientation:
    // - Calculate yaw and pitch from the camera's starting Transform and use those values to initialize the CameraOrientation resource

    // Yaw (Y-axis rotation, horizontal look)
    orientation.yaw = 0.0;

    // Pitch (X-axis rotation, vertical look)
    // let horizontal_length = Vec2::new(direction.x, direction.z).length();
    orientation.pitch = 0.0;

    // Construct the rotation from the calculated yaw and pitch
    let rotation = Quat::from_axis_angle(Vec3::Y, orientation.yaw) * Quat::from_axis_angle(Vec3::X, orientation.pitch);

    // Create the initial transform using the calculated position
    let transform = Transform::from_translation(position).with_rotation(rotation);
    
    // Camera: initially positioned above and looking at origin
    commands.spawn((
        Camera3d::default(),
        ExampleViewports::_PerspectiveMain,
        transform,
        FlyCamera,
        // DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
    ));
}

/// Handles keyboard input for movement
fn keyboard_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
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
fn mouse_look(
    mut mouse_events: MessageReader<MouseMotion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    settings: Res<CameraSettings>,
    mut orientation: ResMut<CameraOrientation>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
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
fn mouse_scroll(
    mut scroll_events: MessageReader<MouseWheel>,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
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

fn scroll_control(
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

/// Process all entities within an newly loaded scene instance
/// and apply physics components based on GLTF extras
fn process_gltf_descendants(
    trigger_entity: Entity,
    mut commands: Commands,
    children: Query<&Children>,
    extras: &Query<&GltfMeshExtras>,
) {
    info!("Processing scene descendants for entity {:?}", trigger_entity);

    // Iterate through the scene to check entities
    for entity in children.iter_descendants(trigger_entity) {
        // If the entity has a GltfMeshExtras component, apply physics
        let Ok(gltf_mesh_extras) = extras.get(entity) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<BMeshExtras>(&gltf_mesh_extras.value) else {
            error!("Couldn't deserialize extras!");
            continue;
        };

        match data.collider {
            BCollider::TrimeshFromMesh => {
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => RigidBody::Static,
                        BRigidBody::Dynamic => RigidBody::Dynamic,
                    },
                    ColliderConstructor::TrimeshFromMesh,
                    DebugRender::default().with_collider_color(Color::srgb(0.0, 0.0, 1.0)),
                ));
            }
            BCollider::ConvexHull => {
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => RigidBody::Static,
                        BRigidBody::Dynamic => RigidBody::Dynamic,
                    },
                    ColliderConstructor::ConvexHullFromMesh,
                    DebugRender::default().with_collider_color(Color::srgb(1.0, 1.0, 1.0)),
                ));
            }
            BCollider::Cuboid => {
                let size = data.cube_size.expect(
                    "Cuboid collider must have cube_size",
                );
                // Scale the defined size by the entity's scale to avoid wrong collider size
                let scaled_size = size * 2.0;
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => RigidBody::Static,
                        BRigidBody::Dynamic => RigidBody::Dynamic,
                    },
                    Mass(100.0),
                    CenterOfMass::default(),
                    Collider::cuboid(scaled_size.x, scaled_size.y, scaled_size.z),
                    DebugRender::default().with_collider_color(Color::srgb(0.0, 1.0, 0.0)),
                ));
            }
            BCollider::Sphere => {
                let size = data.radius.expect(
                    "Sphere collider must have sphere_radius"
                );
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => RigidBody::Static,
                        BRigidBody::Dynamic => RigidBody::Dynamic,
                    },
                    Collider::sphere(size),
                    DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
                ));
            }
            BCollider::Cylinder => {
                let radius = data.radius.expect(
                    "Cylinder collider must have radius"
                );
                let height = data.height.expect(
                    "Cylinder collider must have height"
                );
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => RigidBody::Static,
                        BRigidBody::Dynamic => RigidBody::Dynamic,
                    },
                    Collider::cylinder(radius, height*2.0),
                    DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 1.0)),
                ));
            }
        }
    }
}

/// A dedicated observer system for the initial, one-time level setup (Scene 1).
fn on_level_scene_spawn(
    trigger: On<SceneInstanceReady>,
    commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    info!("LEVEL SCENE READY: Running physics setup for the main level. (ONE TIME)");
    process_gltf_descendants(
        trigger.entity,
        commands,
        children,
        &extras,
    );
}

/// A dedicated observer system for the repetitive structure spawns (Scene 0).
fn on_structure_scene_spawn(
    trigger: On<SceneInstanceReady>,
    commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    info!("STRUCTURE SCENE READY: Running physics setup for a new shape.");
    process_gltf_descendants(
        trigger.entity,
        commands,
        children,
        &extras,
    );
}

/// A dedicated observer system for the repetitive shape spawns (Scene 0).
fn on_shape_scene_spawn(
    trigger: On<SceneInstanceReady>,
    commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    info!("SHAPE SCENE READY: Running physics setup for a new shape.");
    process_gltf_descendants(
        trigger.entity,
        commands,
        children,
        &extras,
    );
}

#[derive(Component)]
struct Ground;

fn draw_cursor(
    distance: Res<CursorDistance>,
    mut labels: Query<(&mut Node, &ExampleLabel)>,
    mut text: Single<&mut Text, With<ImpulseCoords>>,
    labeled: Query<&GlobalTransform>,
    camera_query: Single<(&Camera, &GlobalTransform), With<FlyCamera>>,
    window: Single<&Window>,
    mut cursor_entity_query: Query<&mut Transform, With<ImpulseCursor>>,
) {
    // If the system runs, the mode is Impulse, so we draw the cursor
    let current_distance = distance.0;
    let (camera, camera_transform) = *camera_query;
    let Ok(mut cursor_entity_transform) = cursor_entity_query.single_mut() else {
        return;
    };
    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
    {
        // Calculate the point on the ray at the current stored distance
        let point = ray.get_point(current_distance);

        let position_vector = point;

        let position_text = format!(
            "({:.2}, {:.2}, {:.2})",
            position_vector.x, position_vector.y, position_vector.z
        );

        for (mut node, label) in &mut labels {
            let world_position = labeled.get(label.entity).unwrap().translation() + Vec3::Y * 0.5;
            if let Some(viewport_position) = Some(camera.world_to_viewport(camera_transform, world_position)){
                node.top = px(viewport_position.unwrap_or(Vec2::new(-100.0, -100.0)).y);
                node.left = px(viewport_position.unwrap_or(Vec2::new(-100.0, -100.0)).x);

                text.0 = format!("┌─ Impulse: {}", position_text.clone());
            } else {
                // hide the label if the entity is not visible to the camera
                node.top = px(-100.0);
                node.left = px(-100.0);
            }
        }
        // Draw a small white sphere gizmo
        cursor_entity_transform.translation = point;
    }
}

fn draw_wrecker_cursor(
    distance: Res<CursorDistance>,
    mut labels: Query<(&mut Node, &ExampleLabel)>,
    mut text: Single<&mut Text, With<WreckerCoords>>,
    labeled: Query<&GlobalTransform>,
    camera_query: Single<(&Camera, &GlobalTransform), With<FlyCamera>>,
    window: Single<&Window>,
    time: Res<Time>,
    mut wrecker_entity_query: Query<(&mut LinearVelocity, &mut Transform), With<WreckerCursor>>,
) {
    // If the system runs, the mode is Impulse, so we draw the cursor
    let current_distance = distance.0;
    let (camera, camera_transform) = *camera_query;
    let Ok((mut linear_velocity, wrecker_entity_transform)) = wrecker_entity_query.single_mut() else {
        return;
    };
    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
    {
        // Calculate the point on the ray at the current stored distance
        let point = ray.get_point(current_distance);

        // Calculate the distance vector to the target point
        let target_position = point;
        let current_position = wrecker_entity_transform.translation;
        let delta = target_position - current_position;

        // Caculate the required velocity
        let velocity = delta / time.delta_secs();
        // info!("Velocity: {}", velocity);

        // Set the LinearVelocity component
        linear_velocity.0 = velocity;

        let position_vector = current_position;

        let position_text = format!(
            "({:.2}, {:.2}, {:.2})",
            position_vector.x, position_vector.y, position_vector.z
        );

        for (mut node, label) in &mut labels {
            let world_position = labeled.get(label.entity).unwrap().translation() + Vec3::Y * 0.5;
            if let Some(viewport_position) = Some(camera.world_to_viewport(camera_transform, world_position)) {
                node.top = px(viewport_position.unwrap_or(Vec2::new(-100.0, -100.0)).y);
                node.left = px(viewport_position.unwrap_or(Vec2::new(-100.0, -100.0)).x);

                text.0 = format!("┌─ Wrecker: {}", position_text.clone());
            } else {
                node.top = px(-100.0);
                node.left = px(-100.0);
            }
        }
    } else {
        linear_velocity.0 = Vec3::ZERO;
    }
}

fn apply_force(
    distance: Res<CursorDistance>,
    mut forces: Query<(&Transform, Forces), (With<RigidBody>, Without<WreckerCursor>)>,
    impulse_settings: Res<ImpulseSettings>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform), With<FlyCamera>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    egui_ctx: Res<EguiWantsInput>,
) {
    if egui_ctx.is_pointer_over_area() {
        return;
    }
    // let current_distance = distance.0;
    let (camera, camera_transform) = *camera_query;
    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
        && mouse_input.just_pressed(MouseButton::Left)
    {
        let point = ray.get_point(distance.0);
        for (body_transform , mut impulse_comp) in &mut forces {
            // Vector pointing from point to rigid_body
            let direction_vec = body_transform.translation - point;

            // Set distance
            let distance = direction_vec.length();
            // info!("Distance: {}", distance);

            // Check radius and avoid division by zero if distance is 0
            if distance > 0.0 && distance < impulse_settings.blast_radius {
                // Linear falloff factor: 1.0 at center, 0.0 at edge
                let falloff = 1.0 - (distance / impulse_settings.blast_radius);

                // Calculate the impulse vector: Direction * Max Force * Falloff
                // info!("Normalized direction vector: {}", direction_vec.normalize());
                let impulse = direction_vec.normalize() * impulse_settings.max_force * falloff;

                // Apply the impulse
                impulse_comp.apply_linear_impulse(impulse);
            }
        }
    }

}

fn set_impulse_cursor_visibility<const VISIBLE: bool>(
    mut query: Query<&mut Visibility, With<ImpulseCursor>>,
) {
    for mut visibility in & mut query {
        *visibility = if VISIBLE { Visibility::Visible } else { Visibility::Hidden };
    }
}

fn set_wrecker_cursor_visibility<const VISIBLE: bool>(
    mut query: Query<&mut Visibility, With<WreckerCursor>>,
) {
    for mut visibility in & mut query {
        *visibility = if VISIBLE { Visibility::Visible } else { Visibility::Hidden };
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum MapTag {
    Flat,
    Ramp,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum ShapeTag {
    Cube,
    Sphere,
    Cone,
    Torus,
    Cylinder,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum StructureTag {
    CubeTower
}
fn interactive_menu(
    mut contexts: EguiContexts,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    maps: Query<(Entity, &MapTag)>,
    shapes: Query<(Entity, &ShapeTag)>,
    structures: Query<(Entity, &StructureTag)>,
    mut interaction_mode: ResMut<InteractionMode>,
    mut impulse_settings: ResMut<ImpulseSettings>,
) -> Result {
    egui::Window::new("Rusty Physics Interactive Menu")
        .resizable(true)
        .vscroll(true)
        .default_open(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label("Keybinds:");
            ui.label("Toggle Debug Renders: Q");

            ui.separator();
            ui.label("Interactive Mode");
            ui.horizontal(|ui| {
                let is_click_mode = interaction_mode.0 == InteractionModeType::Click;
                if ui.selectable_label(is_click_mode, "Click Mode").clicked() {
                    interaction_mode.0 = InteractionModeType::Click;
                }

                let is_impulse_mode = interaction_mode.0 == InteractionModeType::Impulse;
                if ui.selectable_label(is_impulse_mode, "Impulse Mode").clicked() {
                    interaction_mode.0 = InteractionModeType::Impulse;
                }
                let is_wrecker_mode = interaction_mode.0 == InteractionModeType::Wrecker;
                if ui.selectable_label(is_wrecker_mode, "Wrecker Mode").clicked() {
                    interaction_mode.0 = InteractionModeType::Wrecker;
                }
            });
            if interaction_mode.0 == InteractionModeType::Impulse {
                ui.label("Impulse Settings");
                ui.horizontal(|ui| {
                    ui.label(format!("Blast Radius: {}", &impulse_settings.blast_radius));
                    ui.add(egui::DragValue::new(&mut impulse_settings.blast_radius).speed(0.1));
                    if ui.button("-").clicked() {
                        impulse_settings.blast_radius -= 1.0;
                    }
                    if ui.button("+").clicked() {
                        impulse_settings.blast_radius += 1.0;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Max Force: {}", &impulse_settings.max_force));
                    ui.add(egui::DragValue::new(&mut impulse_settings.max_force).speed(0.1));
                    if ui.button("-").clicked() {
                        impulse_settings.max_force -= 1.0;
                    }
                    if ui.button("+").clicked() {
                        impulse_settings.max_force += 1.0;
                    }
                });
            }

            ui.separator();
            ui.label("Spawn Maps");
            ui.horizontal(|ui| {
                for tag in [MapTag::Flat, MapTag::Ramp] {
                    let label = format!("{:?}", tag);
                    if ui.button(label).clicked() {
                        for (entity, _) in maps.iter() {
                            commands.entity(entity).despawn();
                        }

                        match tag {
                            MapTag::Flat => commands.spawn(
                                (SceneRoot(
                                asset_server.load(
                                    GltfAssetLabel::Scene(0)
                                        .from_asset("maps.glb"),
                                )),
                                MapTag::Flat,
                                Ground,
                            ))
                            .observe(on_level_scene_spawn),
                            MapTag::Ramp => commands.spawn(
                                (SceneRoot(
                                asset_server.load(
                                    GltfAssetLabel::Scene(1)
                                        .from_asset("maps.glb"),
                                )),
                                MapTag::Ramp,
                                Ground,
                            ))
                            .observe(on_level_scene_spawn),
                        };
                    }
                }
                
            });

            ui.separator();
            ui.label("Spawn Shapes");
            ui.horizontal(|ui| {
                for tag in [ShapeTag::Cube, ShapeTag::Sphere, ShapeTag::Cone, ShapeTag::Torus, ShapeTag::Cylinder] {
                    let label = format!("{:?}", tag);
                    if ui.button(label).clicked() {
                        match tag {
                            ShapeTag::Cube => commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(1)
                                            .from_asset("shapes.glb"),   
                                )),
                                Transform::from_xyz(0.0, 10.0, 0.0),
                                ShapeTag::Cube,
                            ))
                            .observe(on_shape_scene_spawn),
                            ShapeTag::Sphere => commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(3)
                                        .from_asset("shapes.glb"),
                                )),
                                Transform::from_xyz(0.0, 10.0, 0.0),
                                ShapeTag::Sphere,
                            ))
                            .observe(on_shape_scene_spawn),
                            ShapeTag::Cone => commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(0)
                                        .from_asset("shapes.glb"),
                                )),
                                Transform::from_xyz(0.0, 10.0, 0.0),
                                ShapeTag::Cone,
                            ))
                            .observe(on_shape_scene_spawn),
                            ShapeTag::Torus => commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(5)
                                        .from_asset("shapes.glb"),
                                    )),
                                Transform::from_xyz(0.0, 10.0, 0.0),
                                ShapeTag::Torus,
                            ))
                            .observe(on_shape_scene_spawn),
                            ShapeTag::Cylinder => commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(2)
                                        .from_asset("shapes.glb"),
                                )),
                                Transform::from_xyz(0.0, 10.0, 0.0),
                                ShapeTag::Cylinder,
                            ))
                            .observe(on_shape_scene_spawn)
                        };
                    }
                }

            });
            if ui.button("Delete Shapes").clicked() {
                for (entity, _) in shapes.iter() {
                    commands.entity(entity).despawn();
                }
            };

            ui.separator();
            ui.label("Spawn Structures");
            ui.horizontal(|ui| {
                for tag in [StructureTag::CubeTower] {
                    let label = format!("{:?}", tag);
                    if ui.button(label).clicked() {
                        for (entity, _) in structures.iter() {
                            commands.entity(entity).despawn();
                        }

                        match tag {
                            StructureTag::CubeTower => commands.spawn((
                                SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(0)
                                        .from_asset("structures.glb"),
                                )),
                                Transform::from_xyz(0.0, 0.1, 0.0),
                                StructureTag::CubeTower,
                            ))
                            .observe(on_structure_scene_spawn),
                        };
                    }
                }
            });
        });
    Ok(())
}
