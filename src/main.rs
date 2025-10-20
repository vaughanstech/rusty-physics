use bevy::{color::palettes::css::SILVER, input::mouse::{MouseMotion, MouseWheel}, prelude::*};

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

// Velocity of an entity in world space
#[derive(Component, Default, Debug)]
struct Velocity {
    linear: Vec3,
}

// Acceleration (optional, for forces)
#[derive(Component, Default, Debug)]
struct Acceleration {
    _linear: Vec3,
}

// A simple collider (AABB)
#[derive(Component, Debug)]
struct Collider {
    half_extents: Vec3, // half-size in x, y, z
    is_static: bool,
}

// Global physics settings (gravity, timestep, etc.)
#[derive(Resource)]
struct PhysicsSettings {
    gravity: Vec3,
    damping: f32,
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
        .insert_resource(PhysicsSettings {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            damping: 0.98,
        })
        .insert_resource(CameraOrientation::default())
        // Run this system once at startup
        .add_systems(Startup, setup)
        // Run this system every frame
        .add_systems(Update, (keyboard_movement, mouse_look, mouse_scroll, apply_gravity, integrate_motion, floor_collision))
        // Begin the engine's main loop
        .run();
}



// Runs once when the app starts, sets up the scene
fn setup(
    mut commands: Commands, // used to spawn entities
    mut meshes: ResMut<Assets<Mesh>>, // resource for managing meshes
    mut materials: ResMut<Assets<StandardMaterial>>, // Resource for materials
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

    // Light: bright white light above the cube
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Floor: Flat mesh that object should sit on
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        Collider {
            half_extents: Vec3::new(10.0, 0.1, 10.0),
            is_static: true,
        }
    ));

    let cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

    // Cube: mesh + material + transform + custom component
    for x in -1..2 {
        for z in -1..2 {
            commands.spawn((
                // Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                Mesh3d(cube.clone()),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                Transform::from_translation(Vec3::new(x as f32, 5.0, z as f32)),
                Rotates,
                Velocity::default(),
                Collider {
                    half_extents: Vec3::splat(0.25),
                    is_static: false,
                }
            ));
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
    mut mouse_events: MessageReader<MouseMotion>,
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
    mut scroll_events: MessageReader<MouseWheel>,
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

fn apply_gravity(
    settings: Res<PhysicsSettings>,
    mut query: Query<(&mut Velocity, &Collider), Without<Acceleration>>,
    time: Res<Time>,
) {
    for (mut vel, col) in &mut query {
        if !col.is_static {
            vel.linear += settings.gravity * time.delta_secs();
            vel.linear *= settings.damping; // simple air friction
        }
    }
}

fn integrate_motion(
    mut query: Query<(&mut Transform, &Velocity, &Collider)>,
    time: Res<Time>,
) {
    for (mut transform, vel, col) in &mut query {
        if !col.is_static {
            transform.translation += vel.linear * time.delta_secs();
        }
    }
}

fn floor_collision(
    mut query: Query<(&mut Transform, &mut Velocity, &Collider)>
) {
    for (mut transform, mut vel, col) in &mut query {
        if col.is_static {
            continue;
        }

        let bottom = transform.translation.y - col.half_extents.y;
        let floor_y = 0.0; // hardcoded ground level

        if bottom < floor_y {
            // snap to floor
            transform.translation.y = floor_y + col.half_extents.y;
            // simple bounce (invert Y velocity)
            vel.linear.y = 0.0;
        }
    }
}

// Update system - rotates the cube each frame
// fn rotate_cube(mut query: Query<&mut Transform, With<Rotates>>, time: Res<Time>) {
//     for mut transform in &mut query {
//         transform.rotate_y(1.0 * time.delta_secs());
//     }
// }
