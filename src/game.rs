use bevy::prelude::*;

use physics::prelude::*;
use transport::DeliveryMethod;

use crate::{
    cleanup::Cleanup,
    network::*,
    run_criteria::game_server_run_criteria,
    spawn::{Despawn, Spawn, SpawnName},
    AppState,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Game)
                .with_system(on_enter_game)
                .with_system(setup_light)
                .with_system(setup_level),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(on_disconnect_event)
                .with_system(on_client_connection_event)
                .with_system(on_client_state_event)
                .with_system(on_client_spawn_obstacle),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(game_server_run_criteria)
                .with_system(on_server_connection_event)
                .with_system(on_server_ready_event)
                .with_system(on_server_spawn_obstacle),
        );
    }
}

fn on_enter_game(mut client: ResMut<Client>) {
    println!("\n---------- Game ----------");
    println!("Press 'Q' to quit.");
    if client.is_host() {
        println!("Press 'S' to spawn obstacle.");
    }
    println!();

    client.send(ClientPacket::Ready, DeliveryMethod::ReliableOrdered);
}

fn on_disconnect_event(
    keyboard: Res<Input<KeyCode>>,
    mut client_evr: EventReader<ClientEvent>,
    mut app_state: ResMut<State<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Q) {
        app_state.set(AppState::Menu).unwrap();
    } else {
        for event in client_evr.iter() {
            match event {
                ClientEvent::Disconnected => {
                    app_state.set(AppState::Menu).unwrap();
                }
                _ => {}
            }
        }
    }
}

fn on_client_state_event(mut client_evr: EventReader<ClientEvent>, mut commands: Commands) {
    for event in client_evr.iter() {
        match event {
            ClientEvent::State(spawns) => {
                for spawn in spawns.iter() {
                    commands.spawn().insert(*spawn);
                }
            }
            _ => {}
        }
    }
}

fn on_client_connection_event(
    mut client_evr: EventReader<ClientEvent>,
    mut commands: Commands,
    network_id_q: Query<(Entity, &NetworkId)>,
) {
    for event in client_evr.iter() {
        match event {
            ClientEvent::PlayerConnected(id) => {
                commands.spawn().insert(Spawn {
                    id: *id,
                    name: SpawnName::Player,
                    position: Vec3::ZERO,
                });
            }
            ClientEvent::PlayerDisconnected(id) => {
                for (entity, net_id) in network_id_q.iter() {
                    if net_id.value() == *id {
                        commands.entity(entity).insert(Despawn);
                        return;
                    }
                }
            }
            _ => {}
        }
    }
}

fn on_server_connection_event(
    mut server_evr: EventReader<ServerEvent>,
    mut server: ResMut<Server>,
) {
    for event in server_evr.iter() {
        match event {
            ServerEvent::PlayerConnected(id) => {
                server.send_to_all_except(
                    *id,
                    ServerPacket::PlayerConnected(*id),
                    DeliveryMethod::ReliableOrdered,
                );
            }
            ServerEvent::PlayerDisconnected(id) => {
                server.send_to_all_except(
                    *id,
                    ServerPacket::PlayerDisconnected(*id),
                    DeliveryMethod::ReliableOrdered,
                );
            }
            _ => {}
        }
    }
}

fn on_server_ready_event(
    mut server_evr: EventReader<ServerEvent>,
    mut server: ResMut<Server>,
    actor_q: Query<(&NetworkId, &SpawnName, &Transform)>,
) {
    for event in server_evr.iter() {
        match event {
            ServerEvent::PlayerReady(id) => {
                let mut actors = Vec::new();
                actors.push(Spawn {
                    id: *id,
                    name: SpawnName::Player,
                    position: Vec3::ZERO,
                });
                for (net_id, name, transform) in actor_q.iter() {
                    if net_id.value() == *id {
                        continue;
                    }
                    actors.push(Spawn {
                        id: net_id.value(),
                        name: *name,
                        position: transform.translation,
                    })
                }
                server.send(
                    *id,
                    ServerPacket::State(actors.into_boxed_slice()),
                    DeliveryMethod::ReliableOrdered,
                );
            }
            _ => {}
        }
    }
}

fn on_server_spawn_obstacle(keyboard: Res<Input<KeyCode>>, mut server: ResMut<Server>) {
    if keyboard.just_pressed(KeyCode::S) {
        let id = server.generate_id();
        let x = fastrand::i32(-10..=10);
        let z = fastrand::i32(-10..=10);
        let pos = Vec3::new(x as f32, 0.0, z as f32);
        server.send_to_all(
            ServerPacket::SpawnObstacle(id, pos),
            DeliveryMethod::ReliableOrdered,
        );
    }
}

fn on_client_spawn_obstacle(mut client_evr: EventReader<ClientEvent>, mut commands: Commands) {
    for event in client_evr.iter() {
        match event {
            ClientEvent::SpawnObstacle(id, pos) => {
                commands.spawn().insert(Spawn {
                    id: *id,
                    name: SpawnName::Obstacle,
                    position: *pos,
                });
            }
            _ => {}
        }
    }
}

fn setup_light(mut commands: Commands) {
    commands
        .spawn_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::WHITE,
                illuminance: 15000.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(Cleanup);
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    client: Res<Client>,
) {
    let size = Vec3::new(20.0, 1.0, 20.0);
    let ground = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Cube::default())),
            material: materials.add(Color::DARK_GRAY.into()),
            transform: Transform {
                translation: -Vec3::Y * 0.5,
                scale: size,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .id();

    if client.is_host() {
        commands.entity(ground).insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5).into(),
            position: (-Vec3::Y * size.y * 0.5).into(),
            material: ColliderMaterial {
                friction: 0.0,
                restitution: 0.0,
                ..Default::default()
            }
            .into(),
            flags: ColliderFlags {
                collision_groups: InteractionGroups::new(u32::MAX, u32::MAX),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        });
    }
}
