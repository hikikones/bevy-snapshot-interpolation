use std::net::SocketAddr;

use bevy::{ecs::schedule::ShouldRun, prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::spawn::Spawn;

use super::{ServerPacket, Snapshot};
use transport::{ClientTransport, ClientTransportEvent, DeliveryMethod, NetId, Transport};

pub(super) struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClientEvent>().add_system_set(
            SystemSet::new()
                .with_run_criteria(client_run_criteria)
                .with_system(update)
                .with_system(events),
        );
    }
}

pub struct Client {
    transport: Box<dyn ClientTransport>,
    players: HashMap<NetId, RemotePlayer>,
}

impl Client {
    pub fn new(transport: Transport, addr: Option<SocketAddr>) -> Self {
        Self {
            transport: transport.client(addr),
            players: HashMap::default(),
        }
    }

    pub fn get_id(&self) -> NetId {
        self.transport.get_id()
    }

    pub fn _is_connected(&self) -> bool {
        self.transport.is_connected()
    }

    pub fn is_host(&self) -> bool {
        self.transport.get_id() == 0
    }

    pub fn connect(&mut self, addr: SocketAddr) {
        self.transport.connect(addr);
    }

    pub fn send(&mut self, packet: ClientPacket, delivery: DeliveryMethod) {
        let bytes = bincode::serialize(&packet).unwrap();
        self.transport.send(bytes, delivery);
    }
}

fn client_run_criteria(client: Option<Res<Client>>) -> ShouldRun {
    if client.is_some() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn update(mut client: ResMut<Client>, mut client_evw: EventWriter<ClientTransportEvent>) {
    client.transport.poll();
    client.transport.receive(&mut client_evw);
}

fn events(
    mut client_transport_evr: EventReader<ClientTransportEvent>,
    mut client_evw: EventWriter<ClientEvent>,
    mut client: ResMut<Client>,
) {
    for event in client_transport_evr.iter() {
        match event {
            ClientTransportEvent::Connected(id) => {
                client.players.insert(*id, RemotePlayer);
                client_evw.send(ClientEvent::Connected);
                println!("[C] Connected({:?})", id);
            }
            ClientTransportEvent::Disconnected => {
                client_evw.send(ClientEvent::Disconnected);
                println!("[C] Disconnected");
            }
            ClientTransportEvent::Message(bytes) => {
                let packet: ServerPacket = bincode::deserialize(bytes).unwrap();
                println!("[C] {:?}", packet);
                match packet {
                    ServerPacket::PlayerConnected(id) => {
                        client.players.insert(id, RemotePlayer);
                        client_evw.send(ClientEvent::PlayerConnected(id));
                    }
                    ServerPacket::PlayerDisconnected(id) => {
                        client.players.remove(&id);
                        client_evw.send(ClientEvent::PlayerDisconnected(id));
                    }
                    ServerPacket::State(spawns) => {
                        client_evw.send(ClientEvent::State(spawns));
                    }
                    ServerPacket::Snapshot(snapshot) => {
                        if !client.is_host() {
                            client_evw.send(ClientEvent::Snapshot(snapshot));
                        }
                    }
                    ServerPacket::SpawnObstacle(id, pos) => {
                        client_evw.send(ClientEvent::SpawnObstacle(id, pos));
                    }
                }
            }
        }
    }
}

pub struct RemotePlayer;

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientPacket {
    Ready,
    Input(Vec2),
}

#[derive(Debug)]
pub enum ClientEvent {
    Connected,
    Disconnected,
    PlayerConnected(NetId),
    PlayerDisconnected(NetId),
    State(Box<[Spawn]>),
    Snapshot(Snapshot),
    SpawnObstacle(NetId, Vec3),
}
