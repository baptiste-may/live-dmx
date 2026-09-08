use std::collections::HashMap;
use std::time::{Duration, Instant};

use serialport::SerialPort;

use super::types::PhysicalInterface;

pub fn build_enttec_pro_packet(dmx_data: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(518);
    packet.push(0x7E); // Start of message
    packet.push(0x06); // Label: Output Only Send DMX Packet Request

    let length = (dmx_data.len().min(512) + 1) as u16;
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

pub struct PhysicalDmxSender {
    open_ports: HashMap<String, Box<dyn SerialPort>>,
    cached_interfaces: Vec<PhysicalInterface>,
    last_interface_scan: Option<Instant>,
}

impl Default for PhysicalDmxSender {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalDmxSender {
    pub fn new() -> Self {
        Self {
            open_ports: HashMap::new(),
            cached_interfaces: Vec::new(),
            last_interface_scan: None,
        }
    }

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

    pub fn send_to_target(&mut self, target: &str, dmx_data: &[u8]) -> Result<(), String> {
        if let Some(port) = self.resolve_port(target) {
            self.send_universe(&port, dmx_data)
        } else {
            Err(format!("Interface target '{target}' not found"))
        }
    }

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
