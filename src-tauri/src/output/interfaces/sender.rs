//! Serial sender for physical DMX interfaces using Enttec DMX USB Pro framing.
//!
//! Ports are opened lazily, kept open between frames and dropped on write
//! errors. See `docs/contributing/dmx-output.md` for the framing table.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serialport::SerialPort;

use super::types::PhysicalInterface;

/// Builds an Enttec DMX USB Pro "Output Only Send DMX Packet" frame.
///
/// Layout: `0x7E` start byte, label `0x06`, 16-bit little-endian length
/// (`0x0201`, counting the start code and the padded 512-byte payload), DMX
/// start code `0x00`, the channels, then `0xE7` end byte. `dmx_data` is
/// truncated or zero-padded to 512 channels, so the frame is always 518 bytes.
pub fn build_enttec_pro_packet(dmx_data: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(518);
    packet.push(0x7E); // Start of message
    packet.push(0x06); // Label: Output Only Send DMX Packet Request

    // The payload is always padded to 512 channels, so the length field always
    // counts the start code plus that full payload.
    let length: u16 = 512 + 1;
    packet.push((length & 0xFF) as u8); // Length LSB
    packet.push(((length >> 8) & 0xFF) as u8); // Length MSB

    packet.push(0x00); // DMX Start Code
    let len_to_copy = dmx_data.len().min(512);
    packet.extend_from_slice(&dmx_data[..len_to_copy]);
    if len_to_copy < 512 {
        packet.resize(5 + 512, 0);
    }
    packet.push(0xE7); // End of message
    packet
}

/// Sends DMX frames to USB serial interfaces, caching open ports and discovery results.
pub struct PhysicalDmxSender {
    /// Open serial ports keyed by port path; entries are removed on write failure.
    open_ports: HashMap<String, Box<dyn SerialPort>>,
    /// Result of the last interface scan, used to resolve `physical-{n}` ids.
    cached_interfaces: Vec<PhysicalInterface>,
    /// Time of the last successful scan; the cache is refreshed after 5 seconds.
    last_interface_scan: Option<Instant>,
}

impl Default for PhysicalDmxSender {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalDmxSender {
    /// Creates a sender with no open ports and an empty interface cache.
    pub fn new() -> Self {
        Self {
            open_ports: HashMap::new(),
            cached_interfaces: Vec::new(),
            last_interface_scan: None,
        }
    }

    /// Resolves an output target to a serial port path.
    ///
    /// A `physical-{n}` id is looked up in the cached scan, which is refreshed
    /// when older than 5 seconds or when the id is missing (a rescan that fails
    /// keeps the previous cache). Returns `None` if the id is unknown. Any
    /// other string is treated as a port path and returned unchanged without
    /// validation. May enumerate serial ports, which can be slow.
    pub fn resolve_port(&mut self, target: &str) -> Option<String> {
        if target.starts_with("physical-") {
            let should_refresh = match self.last_interface_scan {
                Some(last) => last.elapsed() > Duration::from_secs(5),
                None => true,
            };
            if should_refresh || !self.cached_interfaces.iter().any(|i| i.id == target) {
                if let Ok(interfaces) = super::discovery::discover_physical_interfaces() {
                    self.cached_interfaces = interfaces;
                    self.last_interface_scan = Some(Instant::now());
                }
            }
            self.cached_interfaces
                .iter()
                .find(|i| i.id == target)
                .map(|i| i.port.clone())
        } else {
            Some(target.to_string())
        }
    }

    /// Resolves `target` (see [`Self::resolve_port`]) and sends one DMX frame to it.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `target` cannot be resolved to a port, or with any
    /// error from [`Self::send_universe`].
    pub fn send_to_target(&mut self, target: &str, dmx_data: &[u8]) -> Result<(), String> {
        if let Some(port) = self.resolve_port(target) {
            self.send_universe(&port, dmx_data)
        } else {
            Err(format!("Interface target '{target}' not found"))
        }
    }

    /// Writes one Enttec Pro frame to the serial port `port_name`.
    ///
    /// Opens the port on first use at 57600 baud with a 50 ms timeout and keeps
    /// it for later calls. The write is blocking, bounded by that timeout.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the port cannot be opened or the write fails; after a
    /// write failure the port is dropped so the next call reopens it.
    pub fn send_universe(&mut self, port_name: &str, dmx_data: &[u8]) -> Result<(), String> {
        let port = match self.open_ports.get_mut(port_name) {
            Some(p) => p,
            None => {
                let p = serialport::new(port_name, 57_600)
                    .timeout(Duration::from_millis(50))
                    .open()
                    .map_err(|e| format!("Failed to open serial port {port_name}: {e}"))?;
                self.open_ports.insert(port_name.to_string(), p);
                self.open_ports
                    .get_mut(port_name)
                    .ok_or_else(|| "Failed to retrieve opened serial port".to_string())?
            }
        };

        let packet = build_enttec_pro_packet(dmx_data);
        if let Err(e) = port.write_all(&packet) {
            self.open_ports.remove(port_name);
            return Err(format!("Failed to write to serial port {port_name}: {e}"));
        }
        Ok(())
    }
}
