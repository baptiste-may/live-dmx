use live_dmx_lib::output::protocols::{build_kinet_packet, KinetSender, KINET_PORT};

#[test]
fn test_build_kinet_packet() {
    let mut data = vec![0u8; 512];
    data[0] = 255;
    data[511] = 127;

    let packet = build_kinet_packet(1, &data);
    // 20-byte header + 512-byte payload = 532 bytes
    assert_eq!(packet.len(), 532);

    // KiNET magic: 0x0401DC4A
    assert_eq!(&packet[0..4], &[0x04, 0x01, 0xDC, 0x4A]);
    // Version: 1 (little-endian)
    assert_eq!(&packet[4..6], &[0x01, 0x00]);
    // Type: 0x0101 (DMX out)
    assert_eq!(&packet[6..8], &[0x01, 0x01]);
    // Port:
    assert_eq!(packet[12], 0);
    // Universe (4 bytes little-endian):
    assert_eq!(&packet[16..20], &[0x01, 0x00, 0x00, 0x00]);
    // Data check:
    assert_eq!(packet[20], 255);
    assert_eq!(packet[531], 127);
}

#[test]
fn test_kinet_sender_initialization_and_send() {
    assert_eq!(KINET_PORT, 6038);
    let mut sender = KinetSender::new();
    let data = [255, 0, 128];
    // Sending should not panic or fail
    sender.send_universe(1, &data);
}
