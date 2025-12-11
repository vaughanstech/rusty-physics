use bevy::{ input::mouse::{MouseMotion, MouseWheel}, prelude::* };

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
pub struct FlyCamera;

#[derive(Resource)]
pub struct CameraSettings {
    pub speed: f32, // camera movement speed
    pub sensitivity: f32, // mouse movement sensitivity
    pub zoom_speed: f32, // mouse scroll sensitivity
}

#[derive(Resource, Default)]
pub struct CameraOrientation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Resource)]
pub struct CursorDistance(pub f32);

/// Initializes 3D camera
pub fn setup_camera(
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
pub fn keyboard_movement(
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
pub fn mouse_look(
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
pub fn mouse_scroll(
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

pub fn scroll_control(
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