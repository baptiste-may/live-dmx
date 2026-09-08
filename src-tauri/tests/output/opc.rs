use live_dmx_lib::output::protocols::{build_opc_packet, OpcSender, OPC_PORT};

#[test]
fn test_build_opc_packet() {
    let data = vec![255, 128, 64, 32];
    let packet = build_opc_packet(0, &data);

    // 4-byte header + 4-byte data
    assert_eq!(packet.len(), 8);
    assert_eq!(packet[0], 0); // Channel
    assert_eq!(packet[1], 0); // Command 0 (Set pixel colors)
    assert_eq!(packet[2], 0); // Length high byte
    assert_eq!(packet[3], 4); // Length low byte
    assert_eq!(&packet[4..8], &[255, 128, 64, 32]);
}

#[test]
fn test_opc_sender_initialization_and_send() {
    assert_eq!(OPC_PORT, 7890);
    let mut sender = OpcSender::new();
    let data = [255, 255, 255];
    // When no OPC server is listening on port 7890, send returns Err
    assert!(sender.send_universe(1, &data).is_err());

    // Out of range universes return Err
    assert!(sender.send_universe(0, &data).is_err());
    assert!(sender.send_universe(257, &data).is_err());
}
