use std::{time::Duration};

use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{DefaultPlugins, gltf::GltfMeshExtras, prelude::*, scene::SceneInstanceReady, time::common_conditions::on_timer };
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
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_cubes.run_if(on_timer(Duration::from_secs(1))))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let transform = Transform::from_xyz(20.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y);
    // Camera: positioned above and behind the origin, looking down
    commands.spawn((
        Camera3d::default(),
        ExampleViewports::_PerspectiveMain,
        transform,
    ));

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
            .from_asset("everything.glb"),
    );
    commands.insert_resource(CubeSceneHandle(cube_scene_handle));

    commands.spawn(SceneRoot(
        asset_server.load(
            GltfAssetLabel::Scene(1)
                .from_asset("everything.glb"),
        )
    ))
    .observe(on_level_scene_spawn);
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
                ));
            }
            BCollider::Cuboid => {
                let size = data.cube_size.expect(
                    "Cuboid collider must have cube_size",
                );
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => RigidBody::Static,
                        BRigidBody::Dynamic => RigidBody::Dynamic,
                    },
                    Collider::cuboid(size.x, size.y, size.z)
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