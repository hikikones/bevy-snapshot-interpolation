use std::net::SocketAddr;

use bevy::prelude::*;
use bytes::Bytes;

use crate::{DeliveryMethod, NetId};

pub(crate) struct ClientTransportPlugin;

impl Plugin for ClientTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClientTransportEvent>();
    }
}

pub trait ClientTransport
where
    Self: Send + Sync,
{
    fn get_id(&self) -> NetId;
    fn is_connected(&self) -> bool;
    fn connect(&mut self, addr: SocketAddr);
    fn poll(&mut self);
    fn receive(&mut self, client_evw: &mut EventWriter<ClientTransportEvent>);
    fn send(&mut self, bytes: Vec<u8>, delivery: DeliveryMethod);
}

pub enum ClientTransportEvent {
    Connected(NetId),
    Disconnected,
    Message(Bytes),
}
