use std::collections::HashMap;

use bevy::{ color::palettes::css::SILVER, input::mouse::{MouseMotion, MouseWheel}, prelude::*, render::mesh::{Indices, PrimitiveTopology}};
use bevy_asset::RenderAssetUsages;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
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

// Tag to identify individual Cubes that are spawned
#[derive(Component)]
#[allow(dead_code)]
struct CubeId(u32);

// Lookup table resource for quickly accessing Cubes that are spawned
#[derive(Resource, Default)]
struct CubeMap(HashMap<u32, Entity>);

// Counter to keep track of the number of cube instances spawned
#[derive(Resource, Default)]
struct CubeCounter(u32);

// Tag to identify individual Pyramids that are spawned
#[derive(Component)]
#[allow(dead_code)]
struct PyramidId(u32);

// Lookup table resource for quickly accessing Pyramids that are spawned
#[derive(Resource, Default)]
struct PyramidMap(HashMap<u32, Entity>);

// Counter to keep track of the number of pyramid instances spawned
#[derive(Resource, Default)]
struct PyramidCounter(u32);

fn main() {
    // Entry point of the application
    App::new()
        // Load all default Bevy plugins (window, renderer, input, etc.)
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
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
        .insert_resource(CubeCounter::default())
        .insert_resource(CubeMap::default())
        .insert_resource(PyramidCounter::default())
        .insert_resource(PyramidMap::default())
        // Run this system once at startup
        .add_systems(Startup, (setup_camera, setup_lighting))
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, interactive_menu)
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

    // let cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

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

    // commands.spawn((
    //     Mesh3d(cube.clone()),
    //     MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
    //     Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
    //     Rotates,
    //     Velocity::default(),
    //     RigidBody::Fixed,
    //     Collider::cuboid(0.25, 0.25, 0.25),
    // ))
    // .insert(Restitution::coefficient(0.7))
    // .insert(GravityScale(1.0));

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
    let transform = Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y);
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

// Generates a mesh for a square-based pyramid
fn create_pyramid_mesh() -> Mesh {
    let base_size = 1.0;
    let height = 1.0;

    let half = base_size / 2.0;

    // --- Base (facing down) ---
    let base_vertices = vec![
        ([ half, 0.0,  half], [0.0, -1.0, 0.0], [0.0, 0.0]), // 0
        ([-half, 0.0,  half], [0.0, -1.0, 0.0], [1.0, 0.0]), // 1
        ([-half, 0.0, -half], [0.0, -1.0, 0.0], [1.0, 1.0]), // 2
        ([ half, 0.0, -half], [0.0, -1.0, 0.0], [0.0, 1.0]), // 3
    ];

    // --- Sides ---
    // We'll compute normals per face using cross products.
    let apex = Vec3::new(0.0, height, 0.0);
    let base_points = [
        Vec3::new( half, 0.0,  half), // front-right
        Vec3::new(-half, 0.0,  half), // front-left
        Vec3::new(-half, 0.0, -half), // back-left
        Vec3::new( half, 0.0, -half), // back-right
    ];

    // Helper to compute face normal
    fn face_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
        (b - a).cross(c - a).normalize()
    }

    let mut side_vertices = Vec::new();
    for i in 0..4 {
        let p0 = base_points[i];
        let p1 = base_points[(i + 1) % 4];
        let normal = face_normal(p0, p1, apex);
        // Triangle vertices for this face
        side_vertices.push((p0.to_array(), normal.to_array(), [0.0, 0.0]));
        side_vertices.push((p1.to_array(), normal.to_array(), [1.0, 0.0]));
        side_vertices.push((apex.to_array(), normal.to_array(), [0.5, 1.0]));
    }

    let mut vertices = Vec::new();
    vertices.extend(base_vertices);
    vertices.extend(side_vertices);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    let positions: Vec<[f32; 3]> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<[f32; 3]> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<[f32; 2]> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    // --- Indices ---
    // Base: two triangles
    let mut indices = vec![
        0, 2, 1,
        0, 3, 2,
    ];

    // Sides: each face adds 3 vertices, so offset starts at 4
    for i in 0..4 {
        let base_index = 4 + i * 3;
        indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
        ]);
    }

    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/* Todo:
- set gravity scale during runtime ✅
- support for different shapes (triangle, circle)
- selectable entities
- draggable entities
- default maps
- impulse effect on entites
- block structures
*/
fn interactive_menu(
    mut contexts: EguiContexts,
    mut commands: Commands, // used to spawn entities
    mut meshes: ResMut<Assets<Mesh>>, // resource for managing meshes
    mut materials: ResMut<Assets<StandardMaterial>>, // Resource for materials
    mut query: Query<&mut Transform>,
    mut cube_counter: ResMut<CubeCounter>,
    mut cube_map: ResMut<CubeMap>,
    mut pyramid_counter: ResMut<PyramidCounter>,
    mut pyramid_map: ResMut<PyramidMap>,
    mut grav_scale: Query<&mut GravityScale>,
) -> Result {
    let cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
    let pyramid = meshes.add(create_pyramid_mesh());
    egui::Window::new("Rusty Physics Interactive Menu")
        .resizable(true)
        .vscroll(true)
        .default_open(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label("Label!");

            if ui.button("Button!").clicked() {
                println!("boom!")
            }

            // these should be buttons in the future
            ui.label("Keybinds:");
            ui.label("Restart Simulation: R");
            ui.label("Enable Gravity: G");

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Spawn Cubes")
            });
            if ui.button("Spawn Cube").clicked() {
                cube_counter.0 += 1;
                let id = cube_counter.0;

                let cube_entity = commands.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                    Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
                    Rotates,
                    Velocity::default(),
                    RigidBody::Fixed,
                    Collider::cuboid(0.25, 0.25, 0.25),
                    CubeId(id),
                ))
                .insert(Restitution::coefficient(0.7))
                .insert(GravityScale(1.0))
                .id();

                cube_map.0.insert(id, cube_entity);
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Cube Entities")
            });
            for (id, &entity) in cube_map.0.iter() {
                if let Ok(mut transform) = query.get_mut(entity) {
                    // grab cube information
                    let mut x = transform.translation.x;
                    let mut y = transform.translation.y;
                    let mut z = transform.translation.z;
                    let mut pos = transform.translation;

                    // identify cube
                    ui.collapsing(format!("Cube {}: ", id), |ui| {
                        ui.collapsing("Modify Cube Position", |ui| {
                            // modify x position
                            ui.horizontal(|ui| {
                                ui.label(format!("X Position: {}", x));
                                ui.add(egui::DragValue::new(&mut x).speed(0.1));
                                if ui.button("-").clicked() {
                                    pos.x -= 1.0;
                                }
                                if ui.button("+").clicked() {
                                    pos.x += 1.0;
                                }
                            });

                            // modify y position
                            ui.horizontal(|ui| {
                                ui.label(format!("Y Position: {}", y));
                                ui.add(egui::DragValue::new(&mut y).speed(0.1));
                                if ui.button("-").clicked() {
                                    pos.y -= 1.0;
                                }
                                if ui.button("+").clicked() {
                                    pos.y += 1.0;
                                }
                            });

                            // modify z position
                            ui.horizontal(|ui| {
                                ui.label(format!("Y Position: {}", z));
                                ui.add(egui::DragValue::new(&mut z).speed(0.1));
                                if ui.button("-").clicked() {
                                    pos.z -= 1.0;
                                }
                                if ui.button("+").clicked() {
                                    pos.z += 1.0;
                                }
                            });

                            // reset cube position (0.0, 10.0, 0.0)
                            if ui.button("Reset Cube Position").clicked() {
                                pos.x = 0.0;
                                pos.y = 10.0;
                                pos.z = 0.0;
                            }

                            transform.translation = pos;
                        });

                        ui.collapsing("Modify Cube Gravity", |ui| {
                            for mut grav_scale in grav_scale.iter_mut() {
                                ui.horizontal(|ui| {
                                    ui.label("Current Gravity: ");
                                    ui.add(egui::DragValue::new(&mut grav_scale.0).speed(0.001));
                                });
                                if ui.button("Mars Gravity").clicked() {
                                    grav_scale.0 = 0.38;
                                }
                                if ui.button("Moon Gravity").clicked() {
                                    grav_scale.0 = 0.165;
                                }
                                if ui.button("Venus Gravity").clicked() {
                                    grav_scale.0 = 0.91;
                                }
                                if ui.button("Mercury Gravity").clicked() {
                                    grav_scale.0 = 0.38;
                                }
                            }
                        });

                        // delete cube
                        if ui.button("Delete").clicked() {
                                commands.entity(entity).despawn();
                        }
                    });
                }
            }

            ui.separator();
            ui.label("Spawn Pyramids");
            if ui.button("Spawn Pyramid").clicked() {
                pyramid_counter.0 += 1;
                let id = pyramid_counter.0;
                let base_size = 1.0;
                let height = 1.0;
                let vertices = vec![
                    Vec3::new(base_size / 2.0, 0.0, base_size / 2.0),
                    Vec3::new(-base_size / 2.0, 0.0, base_size / 2.0),
                    Vec3::new(-base_size / 2.0, 0.0, -base_size / 2.0),
                    Vec3::new(base_size / 2.0, 0.0, -base_size / 2.0),
                    Vec3::new(0.0, height, 0.0),
                ];

                let pyramid_collider = Collider::convex_hull(&vertices)
                    .expect("Failed to create convex hull collider");

                let pyramid_entity = commands.spawn((
                    Mesh3d(pyramid.clone()),
                    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                    Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
                    Rotates,
                    Velocity::default(),
                    RigidBody::Fixed,
                    pyramid_collider,
                ))
                .insert(Restitution::coefficient(0.7))
                .insert(GravityScale(1.0))
                .id();

                pyramid_map.0.insert(id, pyramid_entity);
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Pyramid Entities")
            });
            for (id, &entity) in pyramid_map.0.iter() {
                if let Ok(mut transform) = query.get_mut(entity) {
                    // grab pyramid information
                    let mut x = transform.translation.x;
                    let mut y = transform.translation.y;
                    let mut z = transform.translation.z;
                    let mut pos = transform.translation;

                    // identify pyramid
                    ui.collapsing(format!("Pyramid {}: ", id), |ui| {
                        ui.collapsing("Modify Pyramid Position", |ui| {
                            // modify x position
                            ui.horizontal(|ui| {
                                ui.label(format!("X Position: {}", x));
                                ui.add(egui::DragValue::new(&mut x).speed(0.1));
                                if ui.button("-").clicked() {
                                    pos.x -= 1.0;
                                }
                                if ui.button("+").clicked() {
                                    pos.x += 1.0;
                                }
                            });

                            // modify y position
                            ui.horizontal(|ui| {
                                ui.label(format!("Y Position: {}", y));
                                ui.add(egui::DragValue::new(&mut y).speed(0.1));
                                if ui.button("-").clicked() {
                                    pos.y -= 1.0;
                                }
                                if ui.button("+").clicked() {
                                    pos.y += 1.0;
                                }
                            });

                            // modify z position
                            ui.horizontal(|ui| {
                                ui.label(format!("Y Position: {}", z));
                                ui.add(egui::DragValue::new(&mut z).speed(0.1));
                                if ui.button("-").clicked() {
                                    pos.z -= 1.0;
                                }
                                if ui.button("+").clicked() {
                                    pos.z += 1.0;
                                }
                            });

                            // reset pyramid position (0.0, 10.0, 0.0)
                            if ui.button("Reset Pyramid Position").clicked() {
                                pos.x = 0.0;
                                pos.y = 10.0;
                                pos.z = 0.0;
                            }

                            transform.translation = pos;
                        });

                        ui.collapsing("Modify Pyramid Gravity", |ui| {
                            for mut grav_scale in grav_scale.iter_mut() {
                                ui.horizontal(|ui| {
                                    ui.label("Current Gravity: ");
                                    ui.add(egui::DragValue::new(&mut grav_scale.0).speed(0.001));
                                });
                                if ui.button("Mars Gravity").clicked() {
                                    grav_scale.0 = 0.38;
                                }
                                if ui.button("Moon Gravity").clicked() {
                                    grav_scale.0 = 0.165;
                                }
                                if ui.button("Venus Gravity").clicked() {
                                    grav_scale.0 = 0.91;
                                }
                                if ui.button("Mercury Gravity").clicked() {
                                    grav_scale.0 = 0.38;
                                }
                            }
                        });

                        // delete pyramid
                        if ui.button("Delete").clicked() {
                                commands.entity(entity).despawn();
                        }
                    });
                }
            }
        });
    Ok(())
}

// Update system - rotates the cube each frame
// fn rotate_cube(mut query: Query<&mut Transform, With<Rotates>>, time: Res<Time>) {
//     for mut transform in &mut query {
//         transform.rotate_y(1.0 * time.delta_secs());
//     }
// }
