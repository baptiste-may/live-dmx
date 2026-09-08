use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

pub const OPC_PORT: u16 = 7890;

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

pub struct OpcSender {
    stream: Option<TcpStream>,
    last_reconnect_attempt: Option<Instant>,
}

impl Default for OpcSender {
    fn default() -> Self {
        Self::new()
    }
}

impl OpcSender {
    pub fn new() -> Self {
        Self {
            stream: None,
            last_reconnect_attempt: None,
        }
    }

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
