//! KiNET (Color Kinetics) DMX-out output: packet builder and a best-effort UDP sender.
//!
//! See `docs/contributing/dmx-output.md` for the protocol overview.

use std::net::UdpSocket;

/// UDP port used by KiNET devices.
pub const KINET_PORT: u16 = 6038;

/// Builds a KiNET v1 "DMX out" packet carrying one universe.
///
/// Layout (all multi-byte fields little-endian except the magic): 4-byte magic
/// `0x0401DC4A`, 2-byte version (1), 2-byte type `0x0101` (DMX out), 4-byte
/// sequence (always 0), 1-byte port (0), 1-byte flags (0), 2-byte timer (0),
/// 4-byte universe, then 512 channels. `data` is truncated or zero-padded to
/// 512 channels, so the frame is always 532 bytes. The universe is written as
/// received (no 0/1-based conversion).
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

/// Best-effort KiNET transmitter over a single UDP socket.
///
/// Send errors are ignored; if the socket could not be bound, sending is a no-op.
pub struct KinetSender {
    /// Ephemeral UDP socket with broadcast enabled, or `None` if binding failed.
    socket: Option<UdpSocket>,
}

impl Default for KinetSender {
    fn default() -> Self {
        Self::new()
    }
}

impl KinetSender {
    /// Binds a UDP socket on `0.0.0.0:0` and enables broadcast.
    ///
    /// A bind failure is swallowed and yields an inactive sender.
    pub fn new() -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").ok().inspect(|s| {
            let _ = s.set_broadcast(true);
        });
        Self { socket }
    }

    /// Sends one universe to `127.0.0.1:6038` and `255.255.255.255:6038`.
    ///
    /// Send errors are ignored.
    pub fn send_universe(&mut self, universe: u16, data: &[u8]) {
        let packet = build_kinet_packet(universe, data);
        if let Some(socket) = &self.socket {
            let _ = socket.send_to(&packet, ("127.0.0.1", KINET_PORT));
            let _ = socket.send_to(&packet, ("255.255.255.255", KINET_PORT));
        }
    }
}
