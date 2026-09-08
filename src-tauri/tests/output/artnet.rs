use live_dmx_lib::output::protocols::artnet::{
    build_artnet_packet, ARTNET_HEADER, OP_OUTPUT, PROTOCOL_VERSION,
};

#[test]
fn test_build_artnet_packet_header_and_length() {
    let dmx_data = vec![255u8; 512];
    let packet = build_artnet_packet(1, &dmx_data, 42);

    assert_eq!(packet.len(), 530);
    assert_eq!(&packet[0..8], ARTNET_HEADER);

    // OpCode: 0x5000 in little-endian (0x00, 0x50)
    let opcode = (packet[8] as u16) | ((packet[9] as u16) << 8);
    assert_eq!(opcode, OP_OUTPUT);

    // ProtVer: 14 in big-endian (0x00, 0x0E)
    let version = ((packet[10] as u16) << 8) | (packet[11] as u16);
    assert_eq!(version, PROTOCOL_VERSION);

    // Sequence
    assert_eq!(packet[12], 42);

    // Length: 512 in big-endian (0x02, 0x00)
    let length = ((packet[16] as u16) << 8) | (packet[17] as u16);
    assert_eq!(length, 512);

    // Data verification
    assert_eq!(&packet[18..530], &dmx_data[..]);
}

#[test]
fn test_build_artnet_packet_short_data_padded() {
    let dmx_data = vec![128u8; 10];
    let packet = build_artnet_packet(1, &dmx_data, 1);

    assert_eq!(packet.len(), 530);
    assert_eq!(&packet[18..28], &[128u8; 10]);
    assert_eq!(&packet[28..530], &[0u8; 502]);
}

#[test]
fn test_build_artnet_packet_port_address() {
    // Universe 1 -> Port-Address 0
    let packet_u1 = build_artnet_packet(1, &[], 0);
    assert_eq!(packet_u1[14], 0);
    assert_eq!(packet_u1[15], 0);

    // Universe 2 -> Port-Address 1
    let packet_u2 = build_artnet_packet(2, &[], 0);
    assert_eq!(packet_u2[14], 1);
    assert_eq!(packet_u2[15], 0);

    // Universe 257 -> Port-Address 256 (SubUni 0, Net 1)
    let packet_u257 = build_artnet_packet(257, &[], 0);
    assert_eq!(packet_u257[14], 0);
    assert_eq!(packet_u257[15], 1);
}
