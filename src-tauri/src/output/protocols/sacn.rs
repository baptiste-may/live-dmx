use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::SystemTime;

pub const SACN_PORT: u16 = 5568;
const ACN_PACKET_IDENTIFIER: &[u8; 12] = b"ASC-E1.17\0\0\0";
pub const DEFAULT_CID: [u8; 16] = [
    0x4c, 0x69, 0x76, 0x65, 0x44, 0x4d, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
];

pub fn generate_uuid_v4() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    let time_nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    bytes[0..8].copy_from_slice(&time_nanos.to_le_bytes()[..8]);
    let pid = std::process::id();
    bytes[8..12].copy_from_slice(&pid.to_le_bytes());
    bytes[12..16].copy_from_slice(&0x5a17e011u32.to_le_bytes());

    // RFC 4122 variant & version 4 bits:
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 1 (RFC 4122)
    bytes
}

pub fn build_sacn_packet(
    universe: u16,
    data: &[u8],
    sequence: u8,
    cid: Option<&[u8; 16]>,
) -> Result<Vec<u8>, String> {
    if !(1..=63999).contains(&universe) {
        return Err(format!(
            "Universe {universe} out of sACN valid range (1..=63999)"
        ));
    }

    let mut packet = vec![0u8; 638];

    // Root Layer (38 bytes)
    // 0..1: Preamble Size
    packet[0..2].copy_from_slice(&0x0010u16.to_be_bytes());
    // 2..3: Postamble Size
    packet[2..4].copy_from_slice(&0x0000u16.to_be_bytes());
    // 4..15: ACN Packet Identifier
    packet[4..16].copy_from_slice(ACN_PACKET_IDENTIFIER);
    // 16..17: Flags and Length (0x7000 | (638 - 16) = 0x726E)
    packet[16..18].copy_from_slice(&(0x7000u16 | 622u16).to_be_bytes());
    // 18..21: Vector (Root: VECTOR_ROOT_E131_DATA = 0x00000004)
    packet[18..22].copy_from_slice(&0x00000004u32.to_be_bytes());
    // 22..37: CID (16 bytes UUID)
    let actual_cid = cid.unwrap_or(&DEFAULT_CID);
    packet[22..38].copy_from_slice(actual_cid);

    // Framing Layer (77 bytes: 38..115)
    // 38..39: Flags and Length (0x7000 | (638 - 38) = 0x7258)
    packet[38..40].copy_from_slice(&(0x7000u16 | 600u16).to_be_bytes());
    // 40..43: Vector (Framing: VECTOR_E131_DATA_PACKET = 0x00000002)
    packet[40..44].copy_from_slice(&0x00000002u32.to_be_bytes());
    // 44..107: Source Name (64 bytes null-terminated UTF-8)
    let source_name = b"Live-DMX Controller";
    packet[44..44 + source_name.len()].copy_from_slice(source_name);
    // 108: Priority (100 is default)
    packet[108] = 100;
    // 109..110: Synchronization Address (0 = none)
    packet[109..111].copy_from_slice(&0u16.to_be_bytes());
    // 111: Sequence Number
    packet[111] = sequence;
    // 112: Options (0)
    packet[112] = 0;
    // 113..114: Universe
    packet[113..115].copy_from_slice(&universe.to_be_bytes());

    // DMP Layer (523 bytes: 115..638)
    // 115..116: Flags and Length (0x7000 | (638 - 115) = 0x720B)
    packet[115..117].copy_from_slice(&(0x7000u16 | 523u16).to_be_bytes());
    // 117: Vector (DMP: VECTOR_DMP_SET_PROPERTY = 0x02)
    packet[117] = 0x02;
    // 118: Address Type & Data Type (0xa1)
    packet[118] = 0xa1;
    // 119..120: First Property Address (0x0000)
    packet[119..121].copy_from_slice(&0u16.to_be_bytes());
    // 121..122: Address Increment (0x0001)
    packet[121..123].copy_from_slice(&1u16.to_be_bytes());
    // 123..124: Property value count (513: 1 start code + 512 channels)
    packet[123..125].copy_from_slice(&513u16.to_be_bytes());
    // 125: DMX Start Code (0x00)
    packet[125] = 0x00;

    // 126..637: 512 DMX channels
    let len_to_copy = data.len().min(512);
    packet[126..126 + len_to_copy].copy_from_slice(&data[..len_to_copy]);

    Ok(packet)
}

pub struct SacnSender {
    socket: UdpSocket,
    sequences: HashMap<u16, u8>,
    cid: [u8; 16],
}

impl SacnSender {
    pub fn new() -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let _ = socket.set_broadcast(true);
        let cid = generate_uuid_v4();

        Ok(Self {
            socket,
            sequences: HashMap::new(),
            cid,
        })
    }

    pub fn send_universe(&mut self, universe: u16, data: &[u8]) -> std::io::Result<()> {
        if !(1..=63999).contains(&universe) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Universe {universe} out of sACN valid range (1..=63999)"),
            ));
        }

        let seq = self.sequences.entry(universe).or_insert(0);
        let current_seq = *seq;
        *seq = current_seq.wrapping_add(1);

        let packet = build_sacn_packet(universe, data, current_seq, Some(&self.cid))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        let high_byte = (universe >> 8) as u8;
        let low_byte = (universe & 0xFF) as u8;
        let multicast_addr = format!("239.255.{high_byte}.{low_byte}:{SACN_PORT}");
        self.socket.send_to(&packet, multicast_addr)?;
        self.socket.send_to(&packet, ("127.0.0.1", SACN_PORT))?;
        Ok(())
    }
}
