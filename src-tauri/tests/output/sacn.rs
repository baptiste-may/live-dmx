use live_dmx_lib::output::protocols::sacn::build_sacn_packet;

#[test]
fn test_build_sacn_packet_structure() {
    let dmx_data = vec![200u8; 512];
    let packet = build_sacn_packet(1, &dmx_data, 77, None).unwrap();

    assert_eq!(packet.len(), 638);

    // Root layer preamble
    assert_eq!(&packet[0..2], &0x0010u16.to_be_bytes());
    // ACN Packet Identifier
    assert_eq!(&packet[4..16], b"ASC-E1.17\0\0\0");

    // Framing layer sequence number
    assert_eq!(packet[111], 77);
    // Universe
    assert_eq!(&packet[113..115], &1u16.to_be_bytes());

    // DMP layer start code
    assert_eq!(packet[125], 0x00);
    // Channel data
    assert_eq!(&packet[126..638], &dmx_data[..]);
}

#[test]
fn test_build_sacn_packet_universe_encoding() {
    let packet = build_sacn_packet(258, &[], 1, None).unwrap();
    let universe = u16::from_be_bytes([packet[113], packet[114]]);
    assert_eq!(universe, 258);
}

#[test]
fn test_build_sacn_packet_rejects_invalid_universe() {
    assert!(build_sacn_packet(0, &[], 1, None).is_err());
    assert!(build_sacn_packet(64000, &[], 1, None).is_err());
}
