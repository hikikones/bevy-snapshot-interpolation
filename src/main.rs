use bevy::prelude::*;

mod camera;
mod cleanup;
mod game;
mod menu;
mod network;
mod obstacle;
mod player;
mod run_criteria;
mod snapshot;
mod spawn;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Menu,
    Game,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Snapshot Interpolation".into(),
            width: 1024.0,
            height: 720.0,
            vsync: false,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_state(AppState::Menu)
        .add_plugins(DefaultPlugins)
        .add_plugin(transport::TransportPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(network::NetworkPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(spawn::SpawnPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(obstacle::ObstaclePlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(cleanup::CleanupPlugin)
        .add_plugin(snapshot::SnapshotPlugin)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
