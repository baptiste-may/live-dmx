use std::net::UdpSocket;

use rosc::{OscMessage, OscPacket, OscType};

pub const OSC_PORT: u16 = 8000;

pub fn build_osc_dmx_packet(universe: u16, data: &[u8]) -> Vec<u8> {
    let osc_packet = OscPacket::Message(OscMessage {
        addr: format!("/dmx/universe/{universe}"),
        args: vec![OscType::Blob(data.to_vec())],
    });
    rosc::encoder::encode(&osc_packet).unwrap_or_default()
}

pub struct OscSender {
    socket: Option<UdpSocket>,
}

impl Default for OscSender {
    fn default() -> Self {
        Self::new()
    }
}

impl OscSender {
    pub fn new() -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").ok().inspect(|s| {
            let _ = s.set_broadcast(true);
        });
        Self { socket }
    }

    pub fn send_universe(&mut self, universe: u16, data: &[u8]) {
        let packet = build_osc_dmx_packet(universe, data);
        if let Some(socket) = &self.socket {
            let _ = socket.send_to(&packet, ("127.0.0.1", OSC_PORT));
            let _ = socket.send_to(&packet, ("255.255.255.255", OSC_PORT));
        }
    }
}
