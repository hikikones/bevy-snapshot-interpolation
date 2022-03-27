use bevy::prelude::*;
use bytes::Bytes;

use crate::{DeliveryMethod, NetId};

pub(crate) struct ServerTransportPlugin;

impl Plugin for ServerTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerTransportEvent>();
    }
}

pub trait ServerTransport
where
    Self: Send + Sync,
{
    fn poll(&mut self);
    fn receive(&mut self, server_evw: &mut EventWriter<ServerTransportEvent>);
    fn send(&mut self, client_id: NetId, bytes: Vec<u8>, delivery: DeliveryMethod);
    fn send_to_all(&mut self, bytes: Vec<u8>, delivery: DeliveryMethod);
    fn send_to_all_except(&mut self, client_id: NetId, bytes: Vec<u8>, delivery: DeliveryMethod);
}

pub enum ServerTransportEvent {
    Connected(NetId),
    Disconnected(NetId),
    Message(NetId, Bytes),
}
