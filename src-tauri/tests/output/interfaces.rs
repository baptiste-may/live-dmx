use live_dmx_lib::output::interfaces::{
    build_enttec_pro_packet, default_physical_interfaces, discover_physical_interfaces,
    PhysicalDmxSender,
};

#[test]
fn test_default_physical_interfaces() {
    let interfaces = default_physical_interfaces();
    assert_eq!(interfaces.len(), 3);
    assert_eq!(interfaces[0].id, "physical-1");
    assert_eq!(interfaces[0].status, "connected");
    assert_eq!(interfaces[0].fps, 44);

    #[cfg(target_os = "windows")]
    {
        assert_eq!(interfaces[0].port, "COM3");
        assert_eq!(interfaces[1].port, "COM4");
        assert_eq!(interfaces[2].port, "COM5");
    }
    #[cfg(target_os = "macos")]
    {
        assert!(interfaces[0].port.starts_with("/dev/cu."));
        assert!(interfaces[1].port.starts_with("/dev/cu."));
        assert!(interfaces[2].port.starts_with("/dev/cu."));
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        assert_eq!(interfaces[0].port, "/dev/ttyUSB0");
        assert_eq!(interfaces[1].port, "/dev/ttyUSB1");
        assert_eq!(interfaces[2].port, "/dev/ttyUSB2");
    }
}

#[test]
fn test_discover_physical_interfaces() {
    let discovered = discover_physical_interfaces().unwrap();
    for interface in &discovered {
        assert!(interface.id.starts_with("physical-"));
        assert!(!interface.name.is_empty());
        assert!(!interface.port.is_empty());
    }
}

#[test]
fn test_build_enttec_pro_packet() {
    let mut data = vec![0u8; 512];
    data[0] = 255;
    data[511] = 128;

    let packet = build_enttec_pro_packet(&data);
    // 1 (start 0x7E) + 1 (label 0x06) + 2 (len 513) + 1 (dmx start code 0x00) + 512 (payload) + 1 (end 0xE7) = 518 bytes
    assert_eq!(packet.len(), 518);
    assert_eq!(packet[0], 0x7E);
    assert_eq!(packet[1], 0x06);
    assert_eq!(packet[2], 0x01); // 513 LSB
    assert_eq!(packet[3], 0x02); // 513 MSB (0x0201 = 513)
    assert_eq!(packet[4], 0x00); // DMX Start Code
    assert_eq!(packet[5], 255); // First channel
    assert_eq!(packet[516], 128); // 512th channel
    assert_eq!(packet[517], 0xE7); // End of message
}

#[test]
fn test_physical_sender_resolve_port() {
    let mut sender = PhysicalDmxSender::new();
    // Direct port names pass through directly on any platform
    assert_eq!(
        sender.resolve_port("/dev/ttyUSB0"),
        Some("/dev/ttyUSB0".to_string())
    );
    assert_eq!(sender.resolve_port("COM3"), Some("COM3".to_string()));
    assert_eq!(
        sender.resolve_port("/dev/cu.usbserial-1"),
        Some("/dev/cu.usbserial-1".to_string())
    );
}
