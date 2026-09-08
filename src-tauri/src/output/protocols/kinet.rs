use std::net::UdpSocket;

pub const KINET_PORT: u16 = 6038;

pub fn build_kinet_packet(universe: u16, data: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(20 + 512);
    // Magic: 0x0401DC4A (KiNET v1 header magic)
    packet.extend_from_slice(&[0x04, 0x01, 0xDC, 0x4A]);
    // Version: 1 (little-endian)
    packet.extend_from_slice(&1u16.to_le_bytes());
    // Type: 0x0101 (DMX-out)
    packet.extend_from_slice(&[0x01, 0x01]);
    // Sequence: 0
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    // Port / Output (0 = DMX out)
    packet.push(0x00);
    // Flags / Pad:
    packet.push(0x00);
    // Timer:
    packet.extend_from_slice(&[0x00, 0x00]);
    // Universe (4 bytes little-endian):
    packet.extend_from_slice(&(universe as u32).to_le_bytes());
    // DMX Payload (up to 512 bytes):
    let len = data.len().min(512);
    packet.extend_from_slice(&data[..len]);
    if len < 512 {
        packet.resize(20 + 512, 0);
    }
    packet
}

pub struct KinetSender {
    socket: Option<UdpSocket>,
}

impl Default for KinetSender {
    fn default() -> Self {
        Self::new()
    }
}

impl KinetSender {
    pub fn new() -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").ok().inspect(|s| {
            let _ = s.set_broadcast(true);
        });
        Self { socket }
    }

    pub fn send_universe(&mut self, universe: u16, data: &[u8]) {
        let packet = build_kinet_packet(universe, data);
        if let Some(socket) = &self.socket {
            let _ = socket.send_to(&packet, ("127.0.0.1", KINET_PORT));
            let _ = socket.send_to(&packet, ("255.255.255.255", KINET_PORT));
        }
    }
}
