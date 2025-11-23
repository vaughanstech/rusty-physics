use std::{time::Duration};

use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{DefaultPlugins, gltf::GltfMeshExtras, input::mouse::{MouseMotion, MouseWheel}, prelude::*, scene::SceneInstanceReady, time::common_conditions::on_timer };
use bevy_asset::{AssetServer, Handle};
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

#[derive(Resource)]
struct CubeSceneHandle(Handle<Scene>);

#[derive(Component)]
struct FlyCamera;

#[derive(Resource)]
struct CameraSettings {
    speed: f32, // camera movement speed
    sensitivity: f32, // mouse movement sensitivity
    zoom_spped: f32, // mouse scroll sensitivity
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
}

#[derive(Debug, Serialize, Deserialize)]
enum BCollider {
    TrimeshFromMesh,
    Cuboid,
}

#[derive(Debug, Serialize, Deserialize)]
enum BRigidBody {
    Static,
    Dynamic,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .insert_resource(CameraSettings {
            speed: 8.0,
            sensitivity: 0.002,
            zoom_spped: 5.0,
        })
        .insert_resource(CameraOrientation::default())
        .add_systems(Startup, (setup, setup_camera))
        .add_systems(Update, (
            spawn_cubes.run_if(on_timer(Duration::from_secs(1))),
            keyboard_movement,
            mouse_look,
            mouse_scroll,
            toggle_debug_render_state,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
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

    let cube_scene_handle = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("shapes.glb"),
    );
    commands.insert_resource(CubeSceneHandle(cube_scene_handle));

    commands.spawn(SceneRoot(
        asset_server.load(
            GltfAssetLabel::Scene(0)
                .from_asset("maps.glb"),
        )
    ))
    .observe(on_level_scene_spawn);
}

/// Initializes 3D camera
fn setup_camera(
    mut commands: Commands,
) {
    let transform = Transform::from_xyz(20.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y);
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
        let forward = transform.forward();
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
        transform.translation += forward * scroll_delta * settings.zoom_spped * time.delta_secs();
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

/// A dedicated observer system for the repetitive cube spawns (Scene 0).
fn on_cube_scene_spawn(
    trigger: On<SceneInstanceReady>,
    commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    info!("CUBE SCENE READY: Running physics setup for a new cube.");
    process_gltf_descendants(
        trigger.entity,
        commands,
        children,
        &extras,
    );
}

fn spawn_cubes(
    mut commands: Commands,
    cube_handle: Res<CubeSceneHandle>,
) {
    commands.spawn((
        SceneRoot(
            cube_handle.0.clone(),
        ),
        Transform::from_xyz(0.0, 10.0, 0.0),
    )).observe(on_cube_scene_spawn);
}