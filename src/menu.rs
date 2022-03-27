use std::net::SocketAddr;

use bevy::prelude::*;

use crate::{network::*, AppState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Menu).with_system(on_enter_menu))
            .add_system_set(
                SystemSet::on_update(AppState::Menu)
                    .with_system(on_update_menu)
                    .with_system(on_connecting),
            );
    }
}

fn on_enter_menu(mut commands: Commands) {
    println!("\n---------- Menu ----------");
    println!("Press 'H' to host, or 'J' to join.\n");
    remove_server_and_client(&mut commands);
}

fn on_update_menu(keyboard_input: Res<Input<KeyCode>>, mut commands: Commands) {
    let server_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

    if keyboard_input.just_pressed(KeyCode::H) {
        let server = Server::new(Transport::Laminar, Some(server_addr));
        let mut client = Client::new(Transport::Laminar, None);
        client.connect(server_addr);
        commands.insert_resource(server);
        commands.insert_resource(client);
    } else if keyboard_input.just_pressed(KeyCode::J) {
        let mut client = Client::new(Transport::Laminar, None);
        client.connect(server_addr);
        commands.insert_resource(client);
    }
}

fn on_connecting(
    mut client_evr: EventReader<ClientEvent>,
    mut app_state: ResMut<State<AppState>>,
    mut commands: Commands,
) {
    for event in client_evr.iter() {
        match event {
            ClientEvent::Connected => {
                app_state.set(AppState::Game).unwrap();
            }
            ClientEvent::Disconnected => {
                remove_server_and_client(&mut commands);
            }
            _ => {}
        }
    }
}

fn remove_server_and_client(commands: &mut Commands) {
    commands.remove_resource::<Server>();
    commands.remove_resource::<Client>();
}
