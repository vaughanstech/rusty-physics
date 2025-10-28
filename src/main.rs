use bevy::{color::palettes::css::SILVER, input::mouse::{MouseMotion, MouseWheel}, prelude::*};
use bevy_rapier3d::prelude::*;

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

// Used for dynamic plane surfaces
#[derive(Component)]
pub struct PlaneCollider {
    pub normal: Vec3,
    pub point: Vec3,
    pub restitution: f32,
}

// Component marker for any entity that should rotate
#[derive(Component)]
struct Rotates;

// Component used to tag the camera entity
#[derive(Component)]
struct FlyCamera;

// Configuration resource for the camera controller
#[derive(Resource)]
struct CameraSettings {
    speed: f32, // movement speed
    sensitivity: f32, // mouse look sensitivity
    zoom_speed: f32,
}

// State resource to store current yaw/pitch for smooth rotation
#[derive(Resource, Default)]
struct CameraOrientation {
    yaw: f32,
    pitch: f32,
}



fn main() {
    // Entry point of the application
    App::new()
        // Load all default Bevy plugins (window, renderer, input, etc.)
        .add_plugins(DefaultPlugins)
        // Insert camera controller settings
        .insert_resource(CameraSettings {
            speed: 8.0,
            sensitivity: 0.002,
            zoom_speed: 5.0,
        })
        // Insert physics settings
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(CameraOrientation::default())
        // Run this system once at startup
        .add_systems(Startup, (setup_camera, setup_lighting))
        .add_systems(Startup, setup)
        // Run this system every frame
        .add_systems(Update, (keyboard_movement, mouse_look, mouse_scroll, restart_scene_on_key, toggle_gravity))
        // Begin the engine's main loop
        .run();
}



// Runs once when the app starts, sets up the scene
fn setup(
    mut commands: Commands, // used to spawn entities
    mut meshes: ResMut<Assets<Mesh>>, // resource for managing meshes
    mut materials: ResMut<Assets<StandardMaterial>>, // Resource for materials
) {

    // ramp plane parameters
    let slope_angle = -0.4; // radians (rotations around Z axis)
    let slope_length = 20.0;
    let slope_width = 10.0;
    let ramp_origin = Vec3::new(0.0, 0.0, 0.0); // where ramp is centered in world

    // compute ramp endpoints in world space
    let rotation = Quat::from_rotation_z(slope_angle);
    let half_len = slope_length * 0.5;

    // Compute the ramp's local-to-world transform
    let ramp_transform = Transform {
        translation: ramp_origin,
        rotation,
        ..default()
    };
    // Local bottom edge point (along -Z)
    let local_bottom_edge = Vec3::new(half_len, 0.0, 0.0);

    // Convert that local point to world space
    let bottom_edge = ramp_transform.transform_point(local_bottom_edge);

    // Floor: Flat mesh that object should sit on
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(slope_length, slope_width).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        ramp_transform,
        Collider::cuboid(10.0, 0.0, 5.0)
    ));

    // compute where to place flat plane
    let flat_length = 20.0;
    let flat_width = 20.0;
    let flat_center = bottom_edge - Vec3::new(flat_length * 0.5 - flat_length, 0.0, 0.0);

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(flat_length, flat_width).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        Transform::from_translation(flat_center),
        Collider::cuboid(10.0, 0.0, 10.0)
    ));

    let cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

    // Cube: mesh + material + transform + custom component
    // for x in -1..2 {
    //     for z in -1..2 {
    //         commands.spawn((
    //             // Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //             Mesh3d(cube.clone()),
    //             MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
    //             Transform::from_translation(Vec3::new(x as f32, 30.0, z as f32)),
    //             Rotates,
    //             Velocity::default(),
    //             Collider {
    //                 half_extents: Vec3::splat(0.25),
    //                 is_static: false,
    //                 restitution: 0.2,
    //             }
    //         ));
    //     }
        
    // }

    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
        Rotates,
        Velocity::default(),
        RigidBody::Fixed,
        Collider::cuboid(0.25, 0.25, 0.25),
    ))
    .insert(Restitution::coefficient(0.7))
    .insert(GravityScale(1.0));

    // commands.spawn((
    //     Mesh3d(cube.clone()),
    //     MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
    //     Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
    //     Rotates,
    //     Velocity::default(),
    //     Collider {
    //         half_extents: Vec3::splat(0.25),
    //         is_static: false,
    //         restitution: 0.2,
    //     },
    // ));
}

fn setup_camera(
    mut commands: Commands,
) {
    let transform = Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    // Camera: positioned above and behind the origin, looking down
    commands.spawn((
        Camera3d::default(),
        Camera::default(),
        ExampleViewports::_PerspectiveMain,
        transform,
        FlyCamera,
    ));
}

fn setup_lighting(
    mut commands: Commands,
) {
    // Light: bright white light above the cube
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

fn restart_scene_on_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<Entity, (With<RigidBody>, Without<Camera>, Without<Window>)>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        // Despawn all entities except the camera
        for entity in &query {
            commands.entity(entity).despawn();
        }

        // Recreate the scene
        setup(commands, meshes, materials);
        
        info!("Scene Restarted!");
    }
}

fn toggle_gravity(
    mut query: Query<&mut RigidBody>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyG) {
        for mut rigid_body in query.iter_mut() {
            *rigid_body = match *rigid_body {
                RigidBody::Dynamic => RigidBody::Fixed,
                _ => RigidBody::Dynamic
            }
        }
    }
}

// Handles keyboard input for movement
fn keyboard_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;

        // Local forward and right vectors relative to camera
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

// Handles mouse movement for looking around
fn mouse_look(
    mut mouse_events: EventReader<MouseMotion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    settings: Res<CameraSettings>,
    mut orientation: ResMut<CameraOrientation>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
) {
    let mut delta = Vec2::ZERO;
    if mouse_input.pressed(MouseButton::Left) {
        for event in mouse_events.read() {
            delta += event.delta;
        }
    }

    if delta.length_squared() == 0.0 {
        return;
    }

    // Update yaw and pitch
    orientation.yaw -= delta.x * settings.sensitivity;
    orientation.pitch -= delta.y * settings.sensitivity;
    orientation.pitch = orientation.pitch.clamp(-1.54, 1.54); // prevent flipping

    // Apply rotation to camera transformation
    for mut transform in &mut query {
        transform.rotation = Quat::from_axis_angle(Vec3::Y, orientation.yaw) * Quat::from_axis_angle(Vec3::X, orientation.pitch);
    }
}

fn mouse_scroll(
    mut scroll_events: EventReader<MouseWheel>,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
) {
    let mut scroll_delta = 0.0;
    for event in scroll_events.read() {
        // Scroll "up" is positive
        scroll_delta += event.y;
    }

    if scroll_delta.abs() < f32::EPSILON {
        return;
    }

    for mut transform in &mut query {
        let forward = transform.forward();
        transform.translation += forward * scroll_delta * settings.zoom_speed * time.delta_secs();
    }
}

// Update system - rotates the cube each frame
// fn rotate_cube(mut query: Query<&mut Transform, With<Rotates>>, time: Res<Time>) {
//     for mut transform in &mut query {
//         transform.rotate_y(1.0 * time.delta_secs());
//     }
// }
