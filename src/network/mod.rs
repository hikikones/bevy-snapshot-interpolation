use bevy::prelude::*;

pub use transport::{DeliveryMethod, NetId, Transport};

mod client;
mod server;

pub use client::*;
pub use server::*;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ServerPlugin).add_plugin(ClientPlugin);
    }
}

#[derive(Component)]
pub struct NetworkId(NetId);

impl NetworkId {
    pub fn new(id: NetId) -> Self {
        Self(id)
    }
    pub fn value(&self) -> NetId {
        self.0
    }
}
