use avian3d::prelude::*;
use bevy::{ gltf::GltfMeshExtras, prelude::*, scene::SceneInstanceReady };
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BMeshExtras {
    pub collider: BCollider,
    pub rigid_body: BRigidBody,
    pub cube_size: Option<Vec3>,
    pub radius: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BCollider {
    TrimeshFromMesh,
    ConvexHull,
    Cuboid,
    Sphere,
    Cylinder,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BRigidBody {
    Static,
    Dynamic,
}

/// Process all entities within an newly loaded scene instance
/// and apply physics components based on GLTF extras
pub fn process_gltf_descendants(
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
                    Mass(100.0),
                    CenterOfMass::default(),
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
pub fn on_level_scene_spawn(
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
pub fn on_structure_scene_spawn(
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
pub fn on_shape_scene_spawn(
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