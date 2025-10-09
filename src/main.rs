use bevy::prelude::*;

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

fn main() {
    // Entry point of the application
    App::new()
        // Load all default Bevy plugins (window, renderer, input, etc.)
        .add_plugins(DefaultPlugins)
        // Run this system once at startup
        .add_systems(Startup, setup)
        // Run this system every frame
        .add_systems(Update, rotate_cube)
        // Begin the engine's main loop
        .run();
}

// Component marker for any entity that should rotate
#[derive(Component)]
struct Rotates;

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
    ));

    // Light: bright white light aabove the cube
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Cube: mesh + material + transform + custom component
    commands.spawn((
        // Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Rotates,
    ));
}

// Update system - rotates the cube each frame
fn rotate_cube(mut query: Query<&mut Transform, With<Rotates>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(1.0 * time.delta_secs());
    }
}
