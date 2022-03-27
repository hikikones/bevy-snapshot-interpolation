use std::net::SocketAddr;

use bevy::prelude::*;

mod client;
mod laminar;
mod server;

pub use self::laminar::*;
pub use client::*;
pub use server::*;

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ClientTransportPlugin)
            .add_plugin(ServerTransportPlugin);
    }
}

pub type NetId = u8;

#[derive(Debug, Clone, Copy)]
pub enum Transport {
    Laminar,
}

impl Transport {
    pub fn server(&self, addr: Option<SocketAddr>) -> Box<dyn ServerTransport> {
        match self {
            Transport::Laminar => Box::new(LaminarServer::bind(addr)),
        }
    }

    pub fn client(&self, addr: Option<SocketAddr>) -> Box<dyn ClientTransport> {
        match self {
            Transport::Laminar => Box::new(LaminarClient::bind(addr)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeliveryMethod {
    // Reliable,
    ReliableOrdered,
    // ReliableSequenced,
    // Unreliable,
    UnreliableSequenced,
}
