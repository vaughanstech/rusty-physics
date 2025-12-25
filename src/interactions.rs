use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;
use crate::game::{CursorDistance, FlyCamera};

#[derive(Component)]
pub struct ExampleLabel {
    pub entity: Entity,
}

#[derive(Component)]
pub struct ImpulseCoords;

#[derive(Component)]
pub struct WreckerCoords;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionModeType {
    Click,
    Impulse,
    Wrecker,
}

#[derive(Resource, PartialEq)]
pub struct InteractionMode(pub InteractionModeType);

#[derive(Component)]
pub struct ImpulseCursor;

#[derive(Component)]
pub struct WreckerCursor;

#[derive(Resource)]
pub struct ImpulseSettings {
    pub blast_radius: f32,
    pub max_force: f32
}
impl Default for ImpulseSettings {
    fn default() -> Self {
        Self { blast_radius: 50.0, max_force: 100.0 }
    }
}

pub fn draw_impulse_cursor(
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
                node.top = px(viewport_position.unwrap_or(Vec2::new(-100.0, -100.0)).y); // gracefully handle instances where viewport position in x and y are not available
                node.left = px(viewport_position.unwrap_or(Vec2::new(-100.0, -100.0)).x);

                text.0 = format!("┌─ Impulse: {}", position_text.clone());
            } else { // gracefully handle instances where viewport position may not be available
                // hide the label if the entity is not visible to the camera
                node.top = px(-100.0);
                node.left = px(-100.0);
            }
        }
        // Draw a small white sphere gizmo
        cursor_entity_transform.translation = point;
    }
}

pub fn draw_wrecker_cursor(
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
                node.top = px(viewport_position.unwrap_or(Vec2::new(-100.0, -100.0)).y); // gracefully handle instances where viewport position in x and y are not available
                node.left = px(viewport_position.unwrap_or(Vec2::new(-100.0, -100.0)).x);

                text.0 = format!("┌─ Wrecker: {}", position_text.clone());
            } else { // gracefully handle instances where viewport position may not be available
                node.top = px(-100.0);
                node.left = px(-100.0);
            }
        }
    } else {
        linear_velocity.0 = Vec3::ZERO;
    }
}

pub fn apply_force(
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

pub fn set_impulse_cursor_visibility<const VISIBLE: bool>(
    mut query: Query<&mut Visibility, With<ImpulseCursor>>,
) {
    for mut visibility in & mut query {
        *visibility = if VISIBLE { Visibility::Visible } else { Visibility::Hidden };
    }
}

pub fn set_wrecker_cursor_visibility<const VISIBLE: bool>(
    mut query: Query<&mut Visibility, With<WreckerCursor>>,
) {
    for mut visibility in & mut query {
        *visibility = if VISIBLE { Visibility::Visible } else { Visibility::Hidden };
    }
}

pub mod interactive_menu {
    use bevy::prelude::* ;
    use bevy_asset::{AssetServer};
    use bevy_egui::{EguiContexts, egui};
    use crate::entity_pipeline::*;
    use crate::interactions::*;

    #[derive(Component)]
    pub struct Ground;


    #[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum MapTag {
        Flat,
        Ramp,
        RectTank,
    }

    #[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum ShapeTag {
        Cube,
        Sphere,
        Cone,
        Torus,
        Cylinder,
        SMCUBE,
    }

    #[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum StructureTag {
        CubeTower
    }

    pub fn interactive_menu(
        mut contexts: EguiContexts,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        maps: Query<(Entity, &MapTag)>,
        shapes: Query<(Entity, &ShapeTag)>,
        structures: Query<(Entity, &StructureTag)>,
        mut interaction_mode: ResMut<InteractionMode>,
        mut impulse_settings: ResMut<ImpulseSettings>,
        mut wrecker_query: Query<&mut Transform, With<WreckerCursor>>,
    ) -> Result {
        egui::Window::new("Rusty Physics Interactive Menu")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(contexts.ctx_mut()?, |ui| {
                ui.label("Keybinds:");
                ui.label("Toggle Debug Renders: Q");
                ui.label("Enable Click Mode: C");
                ui.label("Enable Impulse Mode: I");
                ui.label("Enable Wrecker Mode: B");
                ui.label("(+) and (-): Up and Down Arrow (respectively)");

                ui.separator();
                ui.label("Interactive Mode");
                ui.horizontal(|ui| {
                    let is_click_mode = interaction_mode.0 == InteractionModeType::Click;
                    if ui.selectable_label(is_click_mode, "Click Mode").clicked() || keyboard_input.just_pressed(KeyCode::KeyC) {
                        interaction_mode.0 = InteractionModeType::Click;
                    }

                    let is_impulse_mode = interaction_mode.0 == InteractionModeType::Impulse;
                    if ui.selectable_label(is_impulse_mode, "Impulse Mode").clicked() || keyboard_input.just_pressed(KeyCode::KeyI) {
                        interaction_mode.0 = InteractionModeType::Impulse;
                    }
                    let is_wrecker_mode = interaction_mode.0 == InteractionModeType::Wrecker;
                    if ui.selectable_label(is_wrecker_mode, "Wrecker Mode").clicked() || keyboard_input.just_pressed(KeyCode::KeyB) {
                        interaction_mode.0 = InteractionModeType::Wrecker;
                    }
                });
                if interaction_mode.0 == InteractionModeType::Impulse {
                    ui.label("Impulse Settings");
                    ui.horizontal(|ui| {
                        ui.label(format!("Blast Radius: {}", &impulse_settings.blast_radius));
                        ui.add(egui::DragValue::new(&mut impulse_settings.blast_radius).speed(0.1));
                        if ui.button("-").clicked() || keyboard_input.just_pressed(KeyCode::ArrowDown) {
                            impulse_settings.blast_radius -= 1.0;
                        }
                        if ui.button("+").clicked() || keyboard_input.just_pressed(KeyCode::ArrowUp) {
                            impulse_settings.blast_radius += 1.0;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Max Force: {}", &impulse_settings.max_force));
                        ui.add(egui::DragValue::new(&mut impulse_settings.max_force).speed(0.1));
                        if ui.button("-").clicked() || keyboard_input.just_pressed(KeyCode::ArrowDown) {
                            impulse_settings.max_force -= 1.0;
                        }
                        if ui.button("+").clicked() || keyboard_input.just_pressed(KeyCode::ArrowUp) {
                            impulse_settings.max_force += 1.0;
                        }
                    });
                }
                if interaction_mode.0 == InteractionModeType::Wrecker {
                    ui.label("Wrecker Settings");
                    ui.horizontal(|ui| {
                        for mut transform in &mut wrecker_query {
                            ui.label(format!("Wrecker Scale: {}", &transform.scale));
                            if ui.button("-").clicked() || keyboard_input.just_pressed(KeyCode::ArrowDown) {
                                transform.scale -= 1.0;
                            }
                            if ui.button("+").clicked() || keyboard_input.just_pressed(KeyCode::ArrowUp) {
                                transform.scale += 1.0;
                            }
                        }
                    });
                }

                ui.separator();
                ui.label("Spawn Maps");
                ui.horizontal(|ui| {
                    for tag in [MapTag::Flat, MapTag::Ramp, MapTag::RectTank] {
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
                                MapTag::RectTank => commands.spawn(
                                    (SceneRoot(
                                    asset_server.load(
                                        GltfAssetLabel::Scene(2)
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
                    for tag in [ShapeTag::Cube, ShapeTag::Sphere, ShapeTag::Cone, ShapeTag::Torus, ShapeTag::Cylinder, ShapeTag::SMCUBE] {
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
                                            GltfAssetLabel::Scene(4)
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
                                            GltfAssetLabel::Scene(6)
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
                                .observe(on_shape_scene_spawn),
                                ShapeTag::SMCUBE => commands.spawn((
                                    SceneRoot(
                                        asset_server.load(
                                            GltfAssetLabel::Scene(3)
                                            .from_asset("shapes.glb"),
                                    )),
                                    Transform::from_xyz(0.0, 10.0, 0.0),
                                    ShapeTag::SMCUBE,
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
}
