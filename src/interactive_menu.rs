use bevy::prelude::* ;
use bevy_asset::{AssetServer};
use bevy_egui::{EguiContexts, egui};
use crate::entity_pipeline::*;
use crate::interaction_modes::*;

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