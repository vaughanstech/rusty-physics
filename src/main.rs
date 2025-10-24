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
    restitution: f32, // bounciness factor (0 = no bounce, 1 = perfect bounce)
}

// Used for dynamic plane surfaces
#[derive(Component)]
pub struct PlaneCollider {
    pub normal: Vec3,
    pub point: Vec3,
    pub restitution: f32,
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
        .add_systems(Update, (keyboard_movement, mouse_look, mouse_scroll, apply_gravity, integrate_motion, floor_collision, dynamic_collisions, draw_debug_colliders, cube_plane_collisions))
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

    // ramp plane parameters
    let slope_angle = -0.4; // radians (rotations around Z axis)
    let slope_length = 20.0;
    let slope_width = 10.0;
    let ramp_origin = Vec3::new(0.0, 0.0, 0.0); // where ramp is centered in world

    // compute ramp endpoints in world space
    let rotation = Quat::from_rotation_z(slope_angle);
    // let local_forward = Vec3::Y; // plane's local forward
    // let slope_dir = rotation * local_forward; // world direction along slope
    // let half_length_vec = slope_dir * (slope_length * 0.5);
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
        PlaneCollider {
            normal: Vec3::Y, // local up direction
            point: Vec3::ZERO, // local origin
            restitution: 0.4,
        }
    ));

    // compute where to place flat plane
    let flat_length = 20.0;
    let flat_width = 20.0;
    let flat_center = bottom_edge - Vec3::new(flat_length * 0.5 - flat_length, 0.0, 0.0);

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(flat_length, flat_width).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        Transform::from_translation(flat_center),
        PlaneCollider {
            normal: Vec3::Y, // local up direction
            point: Vec3::ZERO, // local origin
            restitution: 0.4,
        }
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
        Transform::from_translation(Vec3::new(5.0, 5.0, 0.0)),
        Rotates,
        Velocity::default(),
        Collider {
            half_extents: Vec3::splat(0.25),
            is_static: false,
            restitution: 0.2,
        },
    ));

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

// An axis-aligned bounding box (AABB) is a box aligned to the world's X/Y/Z axes
// We need to check for intersection between two AABBs
fn aabb_intersect(
    pos_a: Vec3,
    half_a: Vec3,
    pos_b: Vec3,
    half_b: Vec3
) -> Option<(Vec3, f32)> {
    // Calculate overlap on each axis
    let delta = pos_b - pos_a;
    let overlap_x = half_a.x + half_b.x - delta.x.abs();
    let overlap_y = half_a.y + half_b.y - delta.y.abs();
    let overlap_z = half_a.z + half_b.z - delta.z.abs();

    // if all overlaps > 0, we have a collision
    if overlap_x > 0.0 && overlap_y > 0.0 && overlap_z > 0.0 {
        // return smallest overlap axis as the collision normal * penetration depth
        if overlap_x < overlap_y && overlap_x < overlap_z {
            Some((Vec3::new(delta.x.signum(), 0.0, 0.0), overlap_x))
        } else if overlap_y < overlap_z {
            Some((Vec3::new(0.0, delta.y.signum(), 0.0), overlap_y))
        } else {
            Some((Vec3::new(0.0, 0.0, delta.z.signum()), overlap_z))
        }
    } else {
        None
    }
}

// Check all pairs of entities and resolve collisions
// This is unoptimized, use spatial partitioning (broad-phase) for optimization
fn dynamic_collisions(
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &Collider)>
) {
    // Collect entities to avoid borrow conflicts
    let entities: Vec<_> = query.iter_mut().map(|(e, _, _, _)| e).collect();

    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            if let Ok([mut a, mut b]) = query.get_many_mut([entities[i], entities[j]]) {
                // skip static objects
                if a.3.is_static && b.3.is_static {
                    continue;
                }

                // Extract positions and half extents
                let pos_a = a.1.translation;
                let pos_b = b.1.translation;
                let half_a = a.3.half_extents;
                let half_b = b.3.half_extents;

                // check intersection
                if let Some((normal, depth)) = aabb_intersect(pos_a, half_a, pos_b, half_b) {
                    // Compute relative velocity
                    let rel_vel = b.2.linear - a.2.linear;
                    let vel_along_normal = rel_vel.dot(normal);
                    // Compute bounce impulse
                    if vel_along_normal < 0.0 {
                        let restitution = (a.3.restitution + b.3.restitution) * 1.0;
                        let impulse_mag = -(1.0 + restitution) * vel_along_normal * 1.0;

                        let impulse = normal * impulse_mag;

                        // Apply impulse (change velocity)
                        if !a.3.is_static {
                            a.2.linear -= impulse;
                        }
                        if !b.3.is_static {
                            b.2.linear += impulse;
                        }
                    }
                    
                    // Direction from A to B
                    // adding bias to over-crrect slightly to ensure separation visually
                    let bias = 0.001;
                    let correction_strength = 0.5; // smaller = more "elastic" collisions
                    let separation = normal * (depth + bias) * correction_strength;

                    // Separate objects based on their static/dynamic state
                    // Separate along collision normal
                    if a.3.is_static {
                        b.1.translation += separation;
                    } else if b.3.is_static {
                        a.1.translation -= separation;
                    } else {
                        // Both are dynamic, split the correction
                        a.1.translation -= separation * 0.5;
                        b.1.translation += separation * 0.5;
                    }

                    // Dampen small residual vertical velocity to prevent jitter
                    if a.2.linear.length_squared() < 0.0001 {
                        a.2.linear = Vec3::ZERO;
                    }
                    if b.2.linear.length_squared() < 0.0001 {
                        b.2.linear = Vec3::ZERO;
                    }
                }
            }
        }
    }
}

fn cube_plane_collisions(
    mut cubes: Query<(&mut Transform, &mut Velocity, &Collider), Without<PlaneCollider>>,
    planes: Query<(&Transform, &PlaneCollider)>,
) {
    for (mut cube_transform, mut velocity, cube_collider) in cubes.iter_mut() {
        for (plane_transform, plane) in planes.iter() {
            // Compute the world-space plane normal
            let normal = (plane_transform.rotation * plane.normal).normalize();
            let plane_point = plane_transform.translation;

            // Distance from cube center to plane
            let cube_center = cube_transform.translation;
            let distance = normal.dot(cube_center - plane_point);

            // Contact threshold: half cube height
            if distance < cube_collider.half_extents.y {
                // collision
                let penetration = cube_collider.half_extents.y - distance;

                // correct position
                cube_transform.translation += normal * penetration;

                // compute bounce
                let vel_along_normal = velocity.linear.dot(normal);
                if vel_along_normal < 0.0 {
                    let restitution = (cube_collider.restitution + plane.restitution) * 0.5;
                    velocity.linear -= normal * (1.0 + restitution) * vel_along_normal;
                }

                // add sideways friction
                let tangent = (velocity.linear - normal * velocity.linear.dot(normal)) * 0.8;
                velocity.linear = tangent;
            }
        }
    }
}

// Using debug rendering system to understand colliders on objects
fn draw_debug_colliders(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Collider)>,
    plane_query: Query<(&Transform, &PlaneCollider)>
) {
    // Draw planes
    for (transform, plane) in &plane_query {
        let world_normal = transform.rotation * plane.normal.normalize();
        let world_point = transform.transform_point(plane.point);
        gizmos.sphere(world_point, 0.1, Color::srgb(0.0, 0.3, 0.3));
        gizmos.line(world_point, world_point + world_normal * 2.0, Color::srgb(1.0, 0.3, 0.3));

         // Draw a grid-like visualization for the plane’s surface
        let plane_size = 10.0;
        let right = transform.rotation * Vec3::X * plane_size;
        let forward = transform.rotation * Vec3::Z * plane_size;

        // Draw 4 corner lines to represent the plane
        let p1 = world_point + right + forward;
        let p2 = world_point + right - forward;
        let p3 = world_point - right - forward;
        let p4 = world_point - right + forward;

        gizmos.line(p1, p2, Color::srgb(0.0, 1.0, 0.0));
        gizmos.line(p2, p3, Color::srgb(0.0, 1.0, 0.0));
        gizmos.line(p3, p4, Color::srgb(0.0, 1.0, 0.0));
        gizmos.line(p4, p1, Color::srgb(0.0, 1.0, 0.0));
    }

    // Draw cubes
    for (transform, cube) in &query {
        let center = transform.translation;
        let hx = cube.half_extents.x;
        let hy = cube.half_extents.y;
        let hz = cube.half_extents.z;

        // Generate cube corners
        let mut corners = vec![];
        for &x in &[-hx, hx] {
            for &y in &[-hy, hy] {
                for &z in &[-hz, hz] {
                    corners.push(transform.transform_point(Vec3::new(x, y, z)));
                }
            }
        }

        // Wireframe edges
        let edges = [
            (0, 1), (0, 2), (0, 4),
            (3, 1), (3, 2), (3, 7),
            (5, 1), (5, 4), (5, 7),
            (6, 2), (6, 4), (6, 7),
        ];
        for &(a, b) in &edges {
            gizmos.line(corners[a], corners[b], Color::srgb(0.3, 0.8, 1.0));
        }
    }
    // for (transform, collider) in &query {
    //     let half = collider.half_extents;

    //     // Compute box corners (8 corners)
    //     let corners = [
    //         Vec3::new(-half.x, -half.y, -half.z),
    //         Vec3::new(half.x, -half.y, -half.z),
    //         Vec3::new(half.x, half.y, -half.z),
    //         Vec3::new(-half.x, half.y, -half.z),
    //         Vec3::new(-half.x, -half.y, half.z),
    //         Vec3::new(half.x, -half.y, half.z),
    //         Vec3::new(half.x, half.y, half.z),
    //         Vec3::new(-half.x, half.y, half.z),
    //     ];

    //     // Transform corners to world space
    //     let world_corners: Vec<Vec3> = corners
    //         .iter()
    //         .map(|c| transform.translation + *c)
    //         .collect();

    //     // Graw 12 edges of the box
    //     let edges = [
    //         (0, 1), (1, 2), (2, 3), (3, 0), // bottom
    //         (4, 5), (5, 6), (6, 7), (7, 4), // top
    //         (0, 4), (1, 5), (2, 6), (3, 7), // sides
    //     ];

    //     let color = if collider.is_static {
    //         Color::srgb(0.3, 0.8, 1.0) // light blue for static
    //     } else {
    //         Color::srgb(1.0, 0.3, 0.3) // red for dynamic
    //     };

    //     for (a,b) in edges {
    //         gizmos.line(world_corners[a], world_corners[b], color);
    //     }
    // }
}

// Update system - rotates the cube each frame
// fn rotate_cube(mut query: Query<&mut Transform, With<Rotates>>, time: Res<Time>) {
//     for mut transform in &mut query {
//         transform.rotate_y(1.0 * time.delta_secs());
//     }
// }
