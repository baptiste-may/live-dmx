//! Open Pixel Control (OPC) output: packet builder and a lazily connecting TCP sender.
//!
//! Each universe maps to one OPC channel on a server at `127.0.0.1`. See
//! `docs/contributing/dmx-output.md` for the protocol overview.

use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

/// TCP port of the OPC server.
pub const OPC_PORT: u16 = 7890;

/// Builds an OPC "set pixel colors" message for one channel.
///
/// Layout: channel byte, command byte `0x00` (set 8-bit pixel colors), 16-bit
/// big-endian payload length, then the payload. `data` is truncated to 65535
/// bytes, the maximum the length field can express.
pub fn build_opc_packet(channel: u8, data: &[u8]) -> Vec<u8> {
    let len = data.len().min(65535) as u16;
    let mut packet = Vec::with_capacity(4 + len as usize);
    packet.push(channel);
    packet.push(0x00); // Command 0: Set 8-bit pixel colors
    packet.push((len >> 8) as u8); // Length high byte
    packet.push((len & 0xFF) as u8); // Length low byte
    packet.extend_from_slice(&data[..len as usize]);
    packet
}

/// OPC client that keeps one TCP connection to the local server.
///
/// The connection is opened lazily and dropped on write errors.
pub struct OpcSender {
    /// Open connection to the OPC server, if any.
    stream: Option<TcpStream>,
    /// Time of the last connection attempt, used to limit retries to one per second.
    last_reconnect_attempt: Option<Instant>,
}

impl Default for OpcSender {
    fn default() -> Self {
        Self::new()
    }
}

impl OpcSender {
    /// Creates a sender without a connection; it connects on the first send.
    pub fn new() -> Self {
        Self {
            stream: None,
            last_reconnect_attempt: None,
        }
    }

    /// Sends one universe as OPC channel `universe - 1`.
    ///
    /// When disconnected, tries to connect to `127.0.0.1:7890` (20 ms connect
    /// timeout, at most once per second) and attempts to enable `TCP_NODELAY` plus
    /// a 20 ms write timeout. The connect timeout bounds the connection attempt,
    /// but a failed write-timeout configuration is ignored, so the write is not
    /// guaranteed to return within 20 ms. On a write failure the connection is
    /// dropped so a later call reconnects.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `universe` is outside `1..=256`, if no connection could
    /// be established (including while waiting out the retry delay), or if the
    /// write to the server fails.
    pub fn send_universe(&mut self, universe: u16, data: &[u8]) -> Result<(), String> {
        if !(1..=256).contains(&universe) {
            return Err(format!(
                "OPC universe {universe} out of range (must be 1..=256)"
            ));
        }
        let channel = (universe - 1) as u8;
        let packet = build_opc_packet(channel, data);

        if self.stream.is_none() {
            let should_try_connect = match self.last_reconnect_attempt {
                Some(last) => last.elapsed() > Duration::from_secs(1),
                None => true,
            };

            if should_try_connect {
                self.last_reconnect_attempt = Some(Instant::now());
                let addr: SocketAddr = ([127, 0, 0, 1], OPC_PORT).into();
                if let Ok(s) = TcpStream::connect_timeout(&addr, Duration::from_millis(20)) {
                    let _ = s.set_nodelay(true);
                    let _ = s.set_write_timeout(Some(Duration::from_millis(20)));
                    self.stream = Some(s);
                }
            }
        }

        let stream = match &mut self.stream {
            Some(s) => s,
            None => return Err("OPC connection not established".to_string()),
        };

        if let Err(e) = stream.write_all(&packet) {
            self.stream = None;
            return Err(format!("Failed to write to OPC server: {e}"));
        }

        Ok(())
    }
}
