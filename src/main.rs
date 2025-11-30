use std::{time::Duration};

use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{DefaultPlugins, color, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, gltf::GltfMeshExtras, input::mouse::{MouseMotion, MouseWheel}, prelude::*, scene::SceneInstanceReady };
use bevy_asset::{AssetServer};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui::{self, TextStyle}};
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
}

#[derive(Resource, PartialEq)]
struct InteractionMode(InteractionModeType);

#[derive(Resource)]
struct CursorDistance(f32);

#[derive(Component)]
struct GizmoCoordinateLabel;

#[derive(Component)]
struct ImpulseCursorGizmo;

#[derive(Component)]
struct XCoord;
#[derive(Component)]
struct YCoord;
#[derive(Component)]
struct ZCoord;

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
        .insert_resource(CameraOrientation::default())
        .insert_resource(InteractionMode(InteractionModeType::Click))
        .insert_resource(CursorDistance(10.0))
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
            ).run_if(resource_equals(InteractionMode(InteractionModeType::Click))),
            // Cursor Control/Draw runs only in Impulse Mode
            (
                impulse_mode_scroll_control, // System to update cursor distance
                draw_cursor,                // System to draw the gizmo
                update_gizmo_label,
                set_impulse_cursor_visibility::<true>,
            ).run_if(resource_equals(InteractionMode(InteractionModeType::Impulse))),
            toggle_debug_render_state,
            set_max_fps,
            fps_counter,
            update_label,
        ))
        .run();
}

#[derive(Component)]
struct ExampleLabel {
    entity: Entity,
}

#[derive(Component)]
struct ExampleDisplay;

fn setup(
    mut commands: Commands,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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

    let cube = commands.spawn((
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(1)
                    .from_asset("shapes.glb"),   
        )),
        Transform::from_xyz(0.0, 10.0, 0.0),
        ShapeTag::Cube,
    )).id();

    let text_style = TextFont {
        ..Default::default()
    };

    let label_text_style = (text_style.clone(), TextColor(color::palettes::css::ORANGE.into()));

    commands.spawn((
        Text::default(),
        text_style,
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            ..default()
        },
        ExampleDisplay,
    ));


    let mut label = |entity: Entity, label: &str| {
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
            )],
        ));
    };
    label(cube, "┌─ Cube");

    // commands.spawn((
    //     Text::new("TEST"),
    //     TextFont {
    //         font_size: 50.0,
    //         ..Default::default()
    //     },
    //     TextColor(Color::WHITE),
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    // ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.1).mesh().uv(23, 16))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Hidden,
    ))
    .insert(ImpulseCursorGizmo);

    // let mut gizmo = GizmoAsset::new();

    // gizmo.sphere(Isometry3d::IDENTITY, 0.1, Color::WHITE);

    // let parent_gizmo = commands.spawn((
    //     Gizmo {
    //         handle: gizmo_assets.add(gizmo),
    //         line_config: GizmoLineConfig {
    //             width: 5.,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     },
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    //     Visibility::Hidden,
    // )).insert(ImpulseCursorGizmo).id();

    // let child_text = commands.spawn((
    //     Text::new("Impulse Position:"),
    //     TextFont {
    //         font_size: 16.0,
    //         ..Default::default()
    //     },
    //     TextColor(Color::WHITE),
    //     Visibility::Hidden,
    // )).insert(ImpulseCursorGizmo).id();
    // // .with_child((
    // //     TextSpan::default(),
    // //     TextFont {
    // //         font_size: 16.0,
    // //         ..Default::default()
    // //     },
    // //     TextColor(Color::WHITE),
    // // )).id();

    // commands.entity(parent_gizmo).add_child(child_text);

    // Gizmo Coordinate Label (Initially hidden)
    // commands.spawn((
    //     Text::new("X: "),
    //     TextFont {
    //         font_size: 16.0,
    //         ..Default::default()
    //     },
    // )).with_child((
    //     TextSpan::default(),
    //     TextFont {
    //         font_size: 16.0,
    //         ..Default::default()
    //     },
    //     TextColor(Color::WHITE),
    //     XCoord,
    // ))
    // .insert(GizmoCoordinateLabel)
    // .insert(Visibility::Visible);

    // commands.spawn((
    //     Text::new("Y: "),
    //     TextFont {
    //         font_size: 16.0,
    //         ..Default::default()
    //     },
    // )).with_child((
    //     TextSpan::default(),
    //     TextFont {
    //         font_size: 16.0,
    //         ..Default::default()
    //     },
    //     TextColor(Color::WHITE),
    //     YCoord,
    // ))
    // .insert(GizmoCoordinateLabel)
    // .insert(Visibility::Visible);

    // commands.spawn((
    //     Text::new("Z: "),
    //     TextFont {
    //         font_size: 16.0,
    //         ..Default::default()
    //     },
    // )).with_child((
    //     TextSpan::default(),
    //     TextFont {
    //         font_size: 16.0,
    //         ..Default::default()
    //     },
    //     TextColor(Color::WHITE),
    //     ZCoord,
    // ))
    // .insert(GizmoCoordinateLabel)
    // .insert(Visibility::Visible);
}

fn update_label(
    mut labels: Query<(&mut Node, &ExampleLabel)>,
    labeled: Query<&GlobalTransform>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<FlyCamera>>,
) {
    let (camera, camera_transform) = *camera_query;

    for mut transform in &mut query {
        for (mut node, label) in &mut labels {
            let world_position = labeled.get(label.entity).unwrap().translation() + Vec3::Y;

            let viewport_position = camera.world_to_viewport(camera_transform, world_position).unwrap();
            node.top = px(viewport_position.y);
            node.left = px(viewport_position.x);
        }
    }
}

/// Set the max framerate limit
fn set_max_fps(
    mut settings: ResMut<FramepaceSettings>,
    fps_limit: Res<SetMaxFps>
) {
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

fn impulse_mode_scroll_control(
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

fn draw_gizmos(
    mut gizmos: Gizmos
) {
    gizmos.sphere(Vec3::new(0.0, 0.0, 0.0), 0.1, Color::WHITE);
}

fn draw_cursor(
    distance: Res<CursorDistance>,
    camera_query: Single<(&Camera, &GlobalTransform), With<FlyCamera>>,
    window: Single<&Window>,
    mut gizmo_query: Query<&mut Transform, With<ImpulseCursorGizmo>>,
) {
    // If the system runs, the mode is Impulse, so we draw the cursor
    let current_distance = distance.0;
    let (camera, camera_transform) = *camera_query;
    let Ok(mut gizmo_transform) = gizmo_query.single_mut() else {
        return;
    };
    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
    {
        // Calculate the point on the ray at the current stored distance
        let point = ray.get_point(current_distance);

        // Draw a small white sphere gizmo
        gizmo_transform.translation = point
    }
}

fn update_gizmo_label(
    distance: Res<CursorDistance>,
    camera_query: Single<(&Camera, &GlobalTransform), With<FlyCamera>>,
    window: Single<&Window>,
    mut x_query: Query<(&mut TextSpan, &mut Node), (With<XCoord>, With<GizmoCoordinateLabel>, Without<YCoord>, Without<ZCoord>)>,
    mut y_query: Query<(&mut TextSpan, &mut Node), (With<YCoord>, With<GizmoCoordinateLabel>, Without<XCoord>, Without<ZCoord>)>,
    mut z_query: Query<(&mut TextSpan, &mut Node), (With<ZCoord>, With<GizmoCoordinateLabel>, Without<XCoord>, Without<YCoord>)>,
) {
    let current_distance = distance.0;
    let (camera, camera_transform) = *camera_query;

    let Ok((mut x_text, mut x_style)) = x_query.single_mut() else { return; };
    let Ok((mut y_text, mut y_style)) = y_query.single_mut() else { return; };
    let Ok((mut z_text, mut z_style)) = z_query.single_mut() else { return; };

    
    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
    {
        // Calculate the 3D world point of the gizmo
        let world_point = ray.get_point(current_distance);

        // Project the world point back to viewport coordinates
        if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_point) {
            x_text.0 = format!("{:.2}", world_point.x);
            y_text.0 = format!("{:.2}", world_point.y);
            z_text.0 = format!("{:.2}", world_point.z);
            info!("X: {:?}", x_text);

            x_style.left = Val::Px(viewport_pos.x + 10.0);
            x_style.top = Val::Px(viewport_pos.y + 10.0);

            y_style.left = Val::Px(viewport_pos.x + 10.0);
            y_style.top = Val::Px(viewport_pos.y + 10.0);

            z_style.left = Val::Px(viewport_pos.x + 10.0);
            z_style.top = Val::Px(viewport_pos.y + 10.0);
            return;
        }
    }

    error!("Could not calculate position!");
    x_style.left = Val::Px(-1000.0);
    x_style.top = Val::Px(-1000.0);

    y_style.left = Val::Px(-1000.0);
    y_style.top = Val::Px(-1000.0);

    z_style.left = Val::Px(-1000.0);
    z_style.top = Val::Px(-1000.0);
}

fn set_gizmo_label_visibility<const VISIBLE: bool>(
    mut query: Query<&mut Visibility, With<GizmoCoordinateLabel>>,
) {
    for mut visibility in &mut query {
        *visibility = if VISIBLE { Visibility::Visible } else { Visibility::Hidden };
    }
}

fn set_impulse_cursor_visibility<const VISIBLE: bool>(
    mut query: Query<&mut Visibility, With<ImpulseCursorGizmo>>,
) {
    for mut visibility in & mut query {
        *visibility = if VISIBLE { Visibility::Visible } else { Visibility::Hidden };
    }
}

// fn spawn_cubes(
//     mut commands: Commands,
//     cube_handle: Res<CubeSceneHandle>,
// ) {
//     commands.spawn((
//         SceneRoot(
//             cube_handle.0.clone(),
//         ),
//         Transform::from_xyz(0.0, 10.0, 0.0),
//     )).observe(on_cube_scene_spawn);
// }
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
            });

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
                                        GltfAssetLabel::Scene(4)
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
