//! Art-Net (ArtDmx) output: packet builder and a best-effort UDP sender.
//!
//! Frames are broadcast on the standard Art-Net port and mirrored to loopback;
//! see `docs/contributing/dmx-output.md` for the protocol overview.

use artnet_protocol::{ArtCommand, Output, PortAddress};
use std::net::UdpSocket;

/// Standard Art-Net UDP port (`0x1936`).
pub const ARTNET_PORT: u16 = 6454;
/// Eight-byte packet identifier that starts every Art-Net packet (`"Art-Net"` + NUL).
pub const ARTNET_HEADER: &[u8; 8] = b"Art-Net\0";
/// Art-Net OpCode of an `ArtDmx` (DMX output) packet, sent little-endian on the wire.
pub const OP_OUTPUT: u16 = 0x5000;
/// Art-Net protocol revision carried in the packet header (big-endian on the wire).
pub const PROTOCOL_VERSION: u16 = 14;

/// Builds an `ArtDmx` packet for one universe.
///
/// `universe` is the 1-based DMX universe used by the app; it is converted to
/// the 0-based Art-Net Port-Address (`universe - 1`, with 0 mapped to 0). A
/// Port-Address outside the valid 15-bit range falls back to 0. `data` is
/// zero-padded or truncated to exactly 512 channels, so the resulting frame is
/// always 530 bytes (18-byte header + 512 channels). `sequence` is written as-is
/// (0 means "sequencing disabled" in Art-Net). Returns an empty `Vec` if
/// serialization fails.
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

/// Best-effort Art-Net transmitter over a single UDP socket.
///
/// Send errors are ignored; if the socket could not be bound, sending is a no-op.
pub struct ArtNetSender {
    /// Ephemeral UDP socket with broadcast enabled, or `None` if binding failed.
    socket: Option<UdpSocket>,
    /// Next sequence number, shared by all universes; cycles through 1..=255
    /// (0 is skipped because it disables sequencing).
    sequence: u8,
}

impl Default for ArtNetSender {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtNetSender {
    /// Binds a UDP socket on `0.0.0.0:0` and enables broadcast.
    ///
    /// A bind failure is swallowed and yields an inactive sender.
    pub fn new() -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").ok().inspect(|s| {
            let _ = s.set_broadcast(true);
        });
        Self {
            socket,
            sequence: 1,
        }
    }

    /// Sends one universe (1-based) to `255.255.255.255:6454` and `127.0.0.1:6454`.
    ///
    /// Advances the sequence counter even when the socket is unavailable.
    /// Non-blocking in practice; send errors are ignored.
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
