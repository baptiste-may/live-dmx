use artnet_protocol::{ArtCommand, Output, PortAddress};
use std::net::UdpSocket;

pub const ARTNET_PORT: u16 = 6454;
pub const ARTNET_HEADER: &[u8; 8] = b"Art-Net\0";
pub const OP_OUTPUT: u16 = 0x5000;
pub const PROTOCOL_VERSION: u16 = 14;

pub fn build_artnet_packet(universe: u16, data: &[u8], sequence: u8) -> Vec<u8> {
    let port_address = if universe > 0 { universe - 1 } else { 0 };
    let mut dmx_data = data.to_vec();
    if dmx_data.len() < 512 {
        dmx_data.resize(512, 0);
    } else if dmx_data.len() > 512 {
        dmx_data.truncate(512);
    }

    let port_address_val = PortAddress::try_from(port_address).unwrap_or(PortAddress::from(0));
    let command = ArtCommand::Output(Output {
        port_address: port_address_val,
        sequence,
        data: dmx_data.into(),
        ..Output::default()
    });

    command.write_to_buffer().unwrap_or_default()
}

pub struct ArtNetSender {
    socket: Option<UdpSocket>,
    sequence: u8,
}

impl Default for ArtNetSender {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtNetSender {
    pub fn new() -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").ok().inspect(|s| {
            let _ = s.set_broadcast(true);
        });
        Self {
            socket,
            sequence: 1,
        }
    }

    pub fn send_universe(&mut self, universe: u16, data: &[u8]) {
        let packet = build_artnet_packet(universe, data, self.sequence);
        self.sequence = if self.sequence == 255 {
            1
        } else {
            self.sequence + 1
        };

        if let Some(socket) = &self.socket {
            let _ = socket.send_to(&packet, ("255.255.255.255", ARTNET_PORT));
            let _ = socket.send_to(&packet, ("127.0.0.1", ARTNET_PORT));
        }
    }
}
