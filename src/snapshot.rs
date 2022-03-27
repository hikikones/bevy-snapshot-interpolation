use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashMap};

use crate::{network::*, run_criteria::game_client_exclusive_run_criteria, AppState};

use super::run_criteria::game_server_run_criteria;

pub struct SnapshotPlugin;

impl Plugin for SnapshotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SnapshotBuffer::default())
            .add_system_set(
                SystemSet::on_enter(AppState::Game).with_system(clear_buffer_on_enter_game),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(game_server_run_criteria)
                    .with_system(send_snapshots),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(game_client_exclusive_run_criteria)
                    .with_system(network_entity_transform_sync_setup)
                    .with_system(buffer_snapshot)
                    .with_system(lerp),
            );
    }
}

//
// Server
//

const SERVER_SEND_RATE: f32 = 1.0 / 20.0;

fn send_snapshots(
    mut send_rate_timer: Local<f32>,
    mut sequence: Local<u32>,
    time: Res<Time>,
    q: Query<(&NetworkId, &Transform)>,
    mut server: ResMut<Server>,
) {
    *send_rate_timer += time.delta_seconds();

    if *send_rate_timer > SERVER_SEND_RATE {
        *send_rate_timer -= SERVER_SEND_RATE;

        let mut transforms = Vec::new();
        for (net_id, transform) in q.iter() {
            let net_transform = NetworkTransform {
                position: transform.translation,
                rotation: transform.rotation,
            };
            transforms.push((net_id.value(), net_transform));
        }

        if transforms.is_empty() {
            return;
        }

        server.send_to_all(
            ServerPacket::Snapshot(Snapshot {
                sequence: *sequence,
                transforms: transforms.into_boxed_slice(),
            }),
            DeliveryMethod::UnreliableSequenced,
        );

        *sequence += 1;
    }
}

//
// Client
//

#[derive(Default)]
struct SnapshotBuffer(VecDeque<Snapshot>);

#[derive(Default, Component)]
struct Lerp {
    from_pos: Vec3,
    from_rot: Quat,
    to_pos: Vec3,
    to_rot: Quat,
}

fn network_entity_transform_sync_setup(
    entity_q: Query<(Entity, &Transform), Added<NetworkId>>,
    mut commands: Commands,
) {
    for (entity, transform) in entity_q.iter() {
        commands.entity(entity).insert(Lerp {
            from_pos: transform.translation,
            to_pos: transform.translation,
            ..Default::default()
        });
    }
}

fn clear_buffer_on_enter_game(mut buffer: ResMut<SnapshotBuffer>) {
    buffer.0.clear();
}

fn buffer_snapshot(mut client_evr: EventReader<ClientEvent>, mut buffer: ResMut<SnapshotBuffer>) {
    for event in client_evr.iter() {
        match event {
            ClientEvent::Snapshot(snapshot) => {
                buffer.0.push_back(snapshot.clone());
            }
            _ => {}
        }
    }
}

fn lerp(
    mut send_rate_timer: Local<f32>,
    mut previous_sequence: Local<u32>,
    mut current_sequence: Local<u32>,
    time: Res<Time>,
    mut buffer: ResMut<SnapshotBuffer>,
    mut lerp_q: Query<(&mut Transform, &mut Lerp, &NetworkId)>,
) {
    if buffer.0.is_empty() {
        return;
    }

    const BUFFER_SIZE_TARGET: u8 = 2;
    let buffer_scalar = 1.0 - (BUFFER_SIZE_TARGET as f32 - buffer.0.len() as f32) / 10.0;

    *send_rate_timer += time.delta_seconds() * buffer_scalar;

    let sequence_scalar = (*current_sequence - *previous_sequence).clamp(1, 4);
    let lerp_duration = SERVER_SEND_RATE * sequence_scalar as f32;

    if *send_rate_timer > lerp_duration {
        *send_rate_timer -= lerp_duration;

        if let Some(snapshot) = buffer.0.pop_front() {
            *previous_sequence = *current_sequence;
            *current_sequence = snapshot.sequence;

            let net_transforms = snapshot
                .transforms
                .iter()
                .map(|s| s.clone())
                .collect::<HashMap<NetId, NetworkTransform>>();

            for (_, mut lerp, id) in lerp_q.iter_mut() {
                lerp.from_pos = lerp.to_pos;
                lerp.from_rot = lerp.to_rot;

                if let Some(net_transform) = net_transforms.get(&id.value()) {
                    lerp.to_pos = net_transform.position;
                    lerp.to_rot = net_transform.rotation;
                }
            }
        }
    }

    let t = *send_rate_timer / lerp_duration;
    for (mut transform, lerp, _) in lerp_q.iter_mut() {
        transform.translation = Vec3::lerp(lerp.from_pos, lerp.to_pos, t);
        transform.rotation = Quat::slerp(lerp.from_rot, lerp.to_rot, t);
    }
}
