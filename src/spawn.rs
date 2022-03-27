use bevy::prelude::*;
use physics::prelude::*;
use serde::{Deserialize, Serialize};
use transport::NetId;

use crate::{
    cleanup::Cleanup,
    network::{Client, NetworkId},
    obstacle::ObstacleBundle,
    player::{LocalPlayer, Player},
    AppState,
};

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(spawn_event)
                .with_system(despawn_event),
        );
    }
}

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub enum SpawnName {
    Player,
    Obstacle,
}

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct Spawn {
    pub id: NetId,
    pub name: SpawnName,
    pub position: Vec3,
}

#[derive(Component)]
pub struct Despawn;

fn spawn_event(
    spawn_q: Query<(Entity, &Spawn)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    client: Res<Client>,
) {
    for (entity, spawn) in spawn_q.iter() {
        commands.entity(entity).despawn();

        match spawn.name {
            SpawnName::Player => {
                let player = commands
                    .spawn()
                    .insert(GlobalTransform::identity())
                    .insert(Transform {
                        translation: spawn.position,
                        ..Default::default()
                    })
                    .insert(Player)
                    .insert(NetworkId::new(spawn.id))
                    .insert(Cleanup)
                    .insert(spawn.name)
                    .with_children(|child| {
                        // Capsule
                        child.spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Capsule::default())),
                            material: materials.add(Color::WHITE.into()),
                            transform: Transform {
                                translation: Vec3::Y,
                                ..Default::default()
                            },
                            ..Default::default()
                        });

                        // Eyes
                        let eye_mesh =
                            meshes.add(Mesh::from(bevy::prelude::shape::Icosphere::default()));
                        let eye_material = materials.add(Color::BLACK.into());
                        let eye_left = Vec3::new(-0.2, 1.6, 0.0) - Vec3::Z * 0.4;
                        let eye_right = Vec3::new(-eye_left.x, eye_left.y, eye_left.z);
                        let eye_scale = Vec3::ONE * 0.15;
                        child.spawn_bundle(PbrBundle {
                            mesh: eye_mesh.clone(),
                            material: eye_material.clone(),
                            transform: Transform {
                                translation: eye_left,
                                scale: eye_scale,
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                        child.spawn_bundle(PbrBundle {
                            mesh: eye_mesh.clone(),
                            material: eye_material.clone(),
                            transform: Transform {
                                translation: eye_right,
                                scale: eye_scale,
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                    })
                    .id();

                if client.get_id() == spawn.id {
                    commands.entity(player).insert(LocalPlayer);
                }

                if client.is_host() {
                    commands
                        .entity(player)
                        .insert_bundle(RigidBodyBundle {
                            position: spawn.position.into(),
                            body_type: RigidBodyType::Dynamic.into(),
                            mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
                            ..Default::default()
                        })
                        .insert_bundle(ColliderBundle {
                            collider_type: ColliderType::Solid.into(),
                            shape: ColliderShape::capsule(
                                (Vec3::Y * 0.5).into(),
                                (Vec3::Y * 1.5).into(),
                                0.5,
                            )
                            .into(),
                            material: ColliderMaterial {
                                friction: 0.0,
                                restitution: 0.0,
                                ..Default::default()
                            }
                            .into(),
                            flags: ColliderFlags {
                                collision_groups: InteractionGroups::new(1 << 0, 1 << 0),
                                ..Default::default()
                            }
                            .into(),
                            ..Default::default()
                        });
                }
            }
            SpawnName::Obstacle => {
                let size = Vec3::new(1.0, 1.0, 1.0);
                let obstacle = commands
                    .spawn()
                    .insert(GlobalTransform::identity())
                    .insert(Transform {
                        translation: spawn.position,
                        ..Default::default()
                    })
                    .insert(NetworkId::new(spawn.id))
                    .insert(Cleanup)
                    .insert(spawn.name)
                    .with_children(|child| {
                        child.spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Cube::default())),
                            material: materials.add(Color::RED.into()),
                            transform: Transform {
                                translation: Vec3::Y,
                                scale: size,
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                    })
                    .id();

                if client.is_host() {
                    commands
                        .entity(obstacle)
                        .insert_bundle(ObstacleBundle::new(spawn.position))
                        .insert_bundle(RigidBodyBundle {
                            position: spawn.position.into(),
                            body_type: RigidBodyType::KinematicPositionBased.into(),
                            ..Default::default()
                        })
                        .insert_bundle(ColliderBundle {
                            collider_type: ColliderType::Solid.into(),
                            shape: ColliderShape::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5)
                                .into(),
                            material: ColliderMaterial {
                                friction: 0.0,
                                restitution: 0.0,
                                ..Default::default()
                            }
                            .into(),
                            flags: ColliderFlags {
                                collision_groups: InteractionGroups::new(1 << 0, 1 << 0),
                                ..Default::default()
                            }
                            .into(),
                            ..Default::default()
                        })
                        .insert(RigidBodyPositionSync::Discrete);
                }
            }
        }
    }
}

fn despawn_event(despawn_q: Query<Entity, Added<Despawn>>, mut commands: Commands) {
    for entity in despawn_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
