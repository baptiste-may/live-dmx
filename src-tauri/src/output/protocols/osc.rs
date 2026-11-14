//! OSC output: one OSC message per universe, sent over UDP.
//!
//! See `docs/contributing/dmx-output.md` for the protocol overview.

use std::net::UdpSocket;

use rosc::{OscMessage, OscPacket, OscType};

/// UDP port the OSC messages are sent to.
pub const OSC_PORT: u16 = 8000;

/// Encodes one universe as an OSC message addressed `/dmx/universe/{universe}`.
///
/// The message has a single blob argument holding `data` verbatim (no padding,
/// truncation or universe validation; the blob itself is padded to 4 bytes by
/// the OSC encoder). Returns an empty `Vec` if encoding fails.
pub fn build_osc_dmx_packet(universe: u16, data: &[u8]) -> Vec<u8> {
    let osc_packet = OscPacket::Message(OscMessage {
        addr: format!("/dmx/universe/{universe}"),
        args: vec![OscType::Blob(data.to_vec())],
    });
    rosc::encoder::encode(&osc_packet).unwrap_or_default()
}

/// Best-effort OSC transmitter over a single UDP socket.
///
/// Send errors are ignored; if the socket could not be bound, sending is a no-op.
pub struct OscSender {
    /// Ephemeral UDP socket with broadcast enabled, or `None` if binding failed.
    socket: Option<UdpSocket>,
}

impl Default for OscSender {
    fn default() -> Self {
        Self::new()
    }
}

impl OscSender {
    /// Binds a UDP socket on `0.0.0.0:0` and enables broadcast.
    ///
    /// A bind failure is swallowed and yields an inactive sender.
    pub fn new() -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").ok().inspect(|s| {
            let _ = s.set_broadcast(true);
        });
        Self { socket }
    }

    /// Sends one universe to `127.0.0.1:8000` and `255.255.255.255:8000`.
    ///
    /// Send errors are ignored.
    pub fn send_universe(&mut self, universe: u16, data: &[u8]) {
        let packet = build_osc_dmx_packet(universe, data);
        if let Some(socket) = &self.socket {
            let _ = socket.send_to(&packet, ("127.0.0.1", OSC_PORT));
            let _ = socket.send_to(&packet, ("255.255.255.255", OSC_PORT));
        }
    }
}
