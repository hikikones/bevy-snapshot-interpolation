use std::net::SocketAddr;

use bevy::{ecs::schedule::ShouldRun, prelude::*, utils::HashMap};
use bincode;
use serde::{Deserialize, Serialize};

use crate::spawn::Spawn;

use super::ClientPacket;
use transport::{DeliveryMethod, NetId, ServerTransport, ServerTransportEvent, Transport};

pub(super) struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerEvent>().add_system_set(
            SystemSet::new()
                .with_run_criteria(server_run_criteria)
                .with_system(update)
                .with_system(events),
        );
    }
}

pub struct Server {
    transport: Box<dyn ServerTransport>,
    players: HashMap<NetId, ServerPlayer>,
    id_counter: NetId,
}

impl Server {
    pub fn new(transport: Transport, addr: Option<SocketAddr>) -> Self {
        Self {
            transport: transport.server(addr),
            players: HashMap::default(),
            id_counter: u8::MAX,
        }
    }

    pub fn generate_id(&mut self) -> NetId {
        let id = self.id_counter;
        self.id_counter -= 1;
        id
    }

    pub fn send(&mut self, client_id: u8, packet: ServerPacket, delivery: DeliveryMethod) {
        let bytes = bincode::serialize(&packet).unwrap();
        self.transport.send(client_id, bytes, delivery);
    }

    pub fn send_to_all(&mut self, packet: ServerPacket, delivery: DeliveryMethod) {
        let bytes = bincode::serialize(&packet).unwrap();
        self.transport.send_to_all(bytes, delivery);
    }

    pub fn send_to_all_except(
        &mut self,
        client_id: NetId,
        packet: ServerPacket,
        delivery: DeliveryMethod,
    ) {
        let bytes = bincode::serialize(&packet).unwrap();
        self.transport
            .send_to_all_except(client_id, bytes, delivery);
    }
}

fn server_run_criteria(server: Option<Res<Server>>) -> ShouldRun {
    if server.is_some() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn update(mut server: ResMut<Server>, mut server_evw: EventWriter<ServerTransportEvent>) {
    server.transport.poll();
    server.transport.receive(&mut server_evw);
}

fn events(
    mut server_transport_evr: EventReader<ServerTransportEvent>,
    mut server_evw: EventWriter<ServerEvent>,
    mut server: ResMut<Server>,
) {
    for event in server_transport_evr.iter() {
        match event {
            ServerTransportEvent::Connected(id) => {
                println!("[S] Connected({:?})", id);
                server.players.insert(*id, ServerPlayer);
                server_evw.send(ServerEvent::PlayerConnected(*id));
            }
            ServerTransportEvent::Disconnected(id) => {
                println!("[S] Disconnected({:?})", id);
                server.players.remove(id);
                server_evw.send(ServerEvent::PlayerDisconnected(*id));
            }
            ServerTransportEvent::Message(id, bytes) => {
                let packet: ClientPacket = bincode::deserialize(bytes).unwrap();
                println!("[S] {:?}", packet);
                match packet {
                    ClientPacket::Ready => {
                        server_evw.send(ServerEvent::PlayerReady(*id));
                    }
                    ClientPacket::Input(input) => {
                        server_evw.send(ServerEvent::PlayerInput(*id, input));
                    }
                }
            }
        }
    }
}

pub struct ServerPlayer;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerPacket {
    PlayerConnected(NetId),
    PlayerDisconnected(NetId),
    State(Box<[Spawn]>),
    Snapshot(Snapshot),
    SpawnObstacle(NetId, Vec3),
}

#[derive(Debug)]
pub enum ServerEvent {
    PlayerConnected(NetId),
    PlayerDisconnected(NetId),
    PlayerReady(NetId),
    PlayerInput(NetId, Vec2),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub sequence: u32,
    pub transforms: Box<[(NetId, NetworkTransform)]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTransform {
    pub position: Vec3,
    pub rotation: Quat,
}
