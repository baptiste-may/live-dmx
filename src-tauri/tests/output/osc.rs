use live_dmx_lib::output::protocols::{build_osc_dmx_packet, OscSender};

#[test]
fn test_build_osc_dmx_packet() {
    let data = vec![255, 128, 64, 0];
    let packet_bytes = build_osc_dmx_packet(1, &data);
    assert!(!packet_bytes.is_empty());
    // OSC packets must be a multiple of 4 bytes
    assert_eq!(packet_bytes.len() % 4, 0);

    // Address is "/dmx/universe/1\0" (16 bytes aligned)
    assert_eq!(&packet_bytes[0..16], b"/dmx/universe/1\0");

    // Type tag is ",b\0\0" (4 bytes aligned)
    assert_eq!(&packet_bytes[16..20], b",b\0\0");

    // Blob length is 4 bytes big-endian
    assert_eq!(&packet_bytes[20..24], &[0, 0, 0, 4]);

    // Blob payload
    assert_eq!(&packet_bytes[24..28], &[255, 128, 64, 0]);
}

#[test]
fn test_osc_sender_initialization_and_send() {
    let mut sender = OscSender::new();
    let data = [100, 200, 50];
    // Sending should not panic or fail even if no network peer is listening
    sender.send_universe(2, &data);
}
