use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_extensions::*;
use physics::prelude::*;
use transport::DeliveryMethod;

use crate::{
    network::{Client, ClientPacket, NetworkId, ServerEvent},
    run_criteria::game_server_run_criteria,
    AppState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::Game).with_system(send_input))
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(game_server_run_criteria)
                    .with_system(server_player_setup)
                    .with_system(server_input_event)
                    .with_system(server_move_players)
                    .with_system(server_rotate_players),
            )
            .add_system_to_stage(CoreStage::PostUpdate, sync_position);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct LocalPlayer;

//
// Client
//

const CLIENT_SEND_RATE: f32 = 1.0 / 20.0;

fn send_input(
    mut send_rate_timer: Local<f32>,
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut client: ResMut<Client>,
) {
    *send_rate_timer += time.delta_seconds();

    if *send_rate_timer > CLIENT_SEND_RATE {
        *send_rate_timer -= CLIENT_SEND_RATE;

        let mut input = Vec2::ZERO;

        if keyboard.pressed(KeyCode::Left) {
            input.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::Right) {
            input.x += 1.0;
        }
        if keyboard.pressed(KeyCode::Up) {
            input.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::Down) {
            input.y += 1.0;
        }

        client.send(
            ClientPacket::Input(input.normalize_or_zero()),
            DeliveryMethod::UnreliableSequenced,
        );
    }
}

//
// Server
//

#[derive(Default, Component)]
struct CurrentInput(Vec3);

fn server_player_setup(player_added_q: Query<Entity, Added<Player>>, mut commands: Commands) {
    for entity in player_added_q.iter() {
        commands.entity(entity).insert(CurrentInput::default());
    }
}

fn server_input_event(
    mut server_evr: EventReader<ServerEvent>,
    mut input_q: Query<(&mut CurrentInput, &NetworkId), With<Player>>,
) {
    for event in server_evr.iter() {
        match event {
            ServerEvent::PlayerInput(player_id, input) => {
                for (mut current_input, id) in input_q.iter_mut() {
                    if *player_id == id.value() {
                        current_input.0 = Vec3::new(input.x, 0.0, input.y);
                        return;
                    }
                }
            }
            _ => {}
        };
    }
}

fn server_move_players(
    mut player_q: Query<(&mut RigidBodyVelocityComponent, &CurrentInput), With<Player>>,
    time: Res<Time>,
) {
    for (mut velocity, input) in player_q.iter_mut() {
        let max_speed = 10.0;
        let max_acc = 100.0;
        let max_delta = max_acc * time.delta_seconds();
        let target = input.0 * max_speed;

        let mut vel: Vec3 = velocity.linvel.into();
        vel.x.move_towards(target.x, max_delta);
        vel.z.move_towards(target.z, max_delta);

        velocity.linvel = vel.into();
    }
}

fn server_rotate_players(
    mut player_q: Query<(&mut Transform, &CurrentInput), With<Player>>,
    time: Res<Time>,
) {
    for (mut transform, input) in player_q.iter_mut() {
        if input.0 == Vec3::ZERO {
            continue;
        }

        let dir = f32::atan2(-input.0.x, -input.0.z);
        let forward = f32::atan2(-transform.forward().x, -transform.forward().z);
        let mut angle = dir - forward;

        if angle < -PI {
            angle += PI * 2.0;
        } else if angle > PI {
            angle -= PI * 2.0;
        }

        transform.rotate(Quat::from_rotation_y(angle * 10.0 * time.delta_seconds()));
    }
}

fn sync_position(mut q: Query<(&mut Transform, &RigidBodyPositionComponent), With<Player>>) {
    for (mut t, pos) in q.iter_mut() {
        t.translation = pos.position.translation.vector.into();
    }
}
