use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use bevy::{prelude::EventWriter, utils::HashMap};
use bytes::Bytes;
use laminar::{Config, Packet, Socket, SocketEvent};

use crate::{
    client::{ClientTransport, ClientTransportEvent},
    server::{ServerTransport, ServerTransportEvent},
    DeliveryMethod, NetId,
};

pub struct LaminarServer {
    socket: Socket,
    connecting: HashMap<SocketAddr, NetId>,
    connected: HashMap<SocketAddr, NetId>,
    id_to_addr: HashMap<NetId, SocketAddr>,
    id_counter: NetId,
}

impl LaminarServer {
    // TODO: Return Result<Self, Error>
    pub fn bind(addr: Option<SocketAddr>) -> Self {
        let cfg = Config {
            heartbeat_interval: Some(Duration::from_secs_f32(1.0)),
            ..Default::default()
        };

        let socket = match addr {
            Some(addr) => Socket::bind_with_config(addr, cfg).unwrap(),
            None => Socket::bind_any_with_config(cfg).unwrap(),
        };

        Self {
            socket,
            connecting: HashMap::default(),
            connected: HashMap::default(),
            id_to_addr: HashMap::default(),
            id_counter: 0,
        }
    }
}

impl ServerTransport for LaminarServer {
    fn poll(&mut self) {
        self.socket.manual_poll(Instant::now());
    }

    fn receive(&mut self, server_evw: &mut EventWriter<ServerTransportEvent>) {
        while let Some(socket_event) = self.socket.recv() {
            match socket_event {
                SocketEvent::Connect(addr) => {
                    let id = *self.connecting.get(&addr).unwrap();
                    self.connected.insert(addr, id);
                    self.id_to_addr.insert(id, addr);
                    self.connecting.remove(&addr);
                    server_evw.send(ServerTransportEvent::Connected(id))
                }
                SocketEvent::Disconnect(addr) => {
                    let id = *self.connected.get(&addr).unwrap();
                    self.id_to_addr.remove(&id);
                    self.connected.remove(&addr);
                    server_evw.send(ServerTransportEvent::Disconnected(id))
                }
                SocketEvent::Timeout(_) => {
                    // TODO?
                }
                SocketEvent::Packet(packet) => {
                    if !self.connected.contains_key(&packet.addr()) {
                        // TODO: check password.
                        let _pw: String = bincode::deserialize(packet.payload()).unwrap();
                        //println!("PW: {}", pw);

                        self.connecting.insert(packet.addr(), self.id_counter);
                        send_packet(
                            &mut self.socket,
                            packet.addr(),
                            vec![self.id_counter],
                            DeliveryMethod::ReliableOrdered,
                        );
                        self.id_counter += 1;
                        continue;
                    }

                    let id = *self.connected.get(&packet.addr()).unwrap();
                    server_evw.send(ServerTransportEvent::Message(
                        id,
                        Bytes::copy_from_slice(packet.payload()),
                    ))
                }
            };
        }
    }

    fn send(&mut self, client_id: NetId, bytes: Vec<u8>, delivery: DeliveryMethod) {
        send_packet(
            &mut self.socket,
            self.id_to_addr[&client_id],
            bytes,
            delivery,
        );
    }

    fn send_to_all(&mut self, bytes: Vec<u8>, delivery: DeliveryMethod) {
        for addr in self.connected.keys() {
            send_packet(&mut self.socket, *addr, bytes.clone(), delivery);
        }
    }

    fn send_to_all_except(&mut self, client_id: NetId, bytes: Vec<u8>, delivery: DeliveryMethod) {
        for (addr, id) in self.connected.iter() {
            if *id != client_id {
                send_packet(&mut self.socket, *addr, bytes.clone(), delivery);
            }
        }
    }
}

pub struct LaminarClient {
    socket: Socket,
    server: Option<SocketAddr>,
    is_connecting: bool,
    is_connected: bool,
    id: NetId,
}

impl LaminarClient {
    // TODO: Return Result<Self, Error>
    pub fn bind(addr: Option<SocketAddr>) -> Self {
        let cfg = Config {
            heartbeat_interval: Some(Duration::from_secs_f32(1.0)),
            ..Default::default()
        };

        let socket = match addr {
            Some(addr) => Socket::bind_with_config(addr, cfg).unwrap(),
            None => Socket::bind_any_with_config(cfg).unwrap(),
        };

        Self {
            socket,
            server: None,
            is_connecting: false,
            is_connected: false,
            id: 0,
        }
    }
}

impl ClientTransport for LaminarClient {
    fn get_id(&self) -> NetId {
        self.id
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }

    fn connect(&mut self, addr: SocketAddr) {
        self.is_connecting = true;
        // TODO: add password to argument.
        let pw = bincode::serialize("password").unwrap();
        send_packet(&mut self.socket, addr, pw, DeliveryMethod::ReliableOrdered);
    }

    fn poll(&mut self) {
        self.socket.manual_poll(Instant::now());
    }

    fn receive(&mut self, client_evw: &mut EventWriter<ClientTransportEvent>) {
        while let Some(socket_event) = self.socket.recv() {
            match socket_event {
                SocketEvent::Connect(addr) => {
                    self.server = Some(addr);
                    self.is_connected = true;
                    continue;
                }
                SocketEvent::Disconnect(_) => {
                    self.is_connected = false;
                    self.is_connecting = false;
                    self.server = None;
                    client_evw.send(ClientTransportEvent::Disconnected);
                }
                SocketEvent::Timeout(_) => {
                    if self.is_connecting {
                        client_evw.send(ClientTransportEvent::Disconnected);
                    }
                    self.is_connected = false;
                    self.is_connecting = false;
                    self.server = None;
                }
                SocketEvent::Packet(packet) => {
                    if self.is_connecting && self.is_connected {
                        self.is_connecting = false;
                        self.id = packet.payload()[0];
                        client_evw.send(ClientTransportEvent::Connected(self.id));
                        continue;
                    }

                    client_evw.send(ClientTransportEvent::Message(Bytes::copy_from_slice(
                        packet.payload(),
                    )));
                }
            };
        }
    }

    fn send(&mut self, bytes: Vec<u8>, delivery: DeliveryMethod) {
        if let Some(server_addr) = self.server {
            send_packet(&mut self.socket, server_addr, bytes, delivery);
        }
    }
}

fn send_packet(socket: &mut Socket, addr: SocketAddr, bytes: Vec<u8>, delivery: DeliveryMethod) {
    match delivery {
        DeliveryMethod::ReliableOrdered => {
            socket
                .send(Packet::reliable_ordered(addr, bytes, None))
                .unwrap();
        }
        DeliveryMethod::UnreliableSequenced => {
            socket
                .send(Packet::unreliable_sequenced(addr, bytes, None))
                .unwrap();
        }
    };
}
