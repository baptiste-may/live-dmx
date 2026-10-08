use live_dmx_lib::output::{
    classify_channel, extract_fixture_channel_count, get_fixture_channel_capabilities,
    get_fixture_channel_name, ChannelType,
};
use live_dmx_lib::show::PatchedFixture;

#[test]
fn test_classify_channel() {
    assert_eq!(classify_channel("Dimmer"), ChannelType::Dimmer);
    assert_eq!(classify_channel("Master Intensity"), ChannelType::Dimmer);
    assert_eq!(classify_channel("Red"), ChannelType::Red);
    assert_eq!(classify_channel("Green"), ChannelType::Green);
    assert_eq!(classify_channel("Blue"), ChannelType::Blue);
    assert_eq!(classify_channel("White"), ChannelType::White);
    assert_eq!(classify_channel("Amber"), ChannelType::Amber);
    assert_eq!(classify_channel("Cyan"), ChannelType::Cyan);
    assert_eq!(classify_channel("Magenta"), ChannelType::Magenta);
    assert_eq!(classify_channel("Yellow"), ChannelType::Yellow);
    assert_eq!(classify_channel("UV"), ChannelType::UV);
    assert_eq!(classify_channel("Strobe"), ChannelType::Strobe);
    assert_eq!(classify_channel("Color Wheel"), ChannelType::ColorWheel);
    assert_eq!(classify_channel("Color Macro"), ChannelType::ColorMacro);
    assert_eq!(classify_channel("Color Preset"), ChannelType::ColorMacro);
    assert_eq!(classify_channel("Colors"), ChannelType::ColorMacro);
    assert_eq!(classify_channel("Color Slot"), ChannelType::ColorMacro);
    assert_eq!(classify_channel("CTO"), ChannelType::ColorTemperature);
    assert_eq!(classify_channel("CCT"), ChannelType::ColorTemperature);
    assert_eq!(
        classify_channel("Color Temperature"),
        ChannelType::ColorTemperature
    );
    assert_eq!(classify_channel("Dimmer fine"), ChannelType::Other);
    assert_eq!(classify_channel("Red Fine"), ChannelType::Other);
    assert_eq!(classify_channel("Green LSB"), ChannelType::Other);
    assert_eq!(classify_channel("Pan (16 bit)"), ChannelType::PanFine);
    assert_eq!(classify_channel("Pan"), ChannelType::Pan);
    assert_eq!(classify_channel("Pan Fine"), ChannelType::PanFine);
    assert_eq!(classify_channel("Tilt"), ChannelType::Tilt);
    assert_eq!(classify_channel("Tilt Fine"), ChannelType::TiltFine);
    assert_eq!(classify_channel("XYZ_Y"), ChannelType::PositionXYZ);
    assert_eq!(classify_channel("Lime"), ChannelType::Lime);
    assert_eq!(classify_channel("ColorAdd_GY"), ChannelType::Lime);
    assert_eq!(classify_channel("DimmerCurve"), ChannelType::Other);
    assert_eq!(classify_channel("Dimmer Mode"), ChannelType::Other);
    assert_eq!(classify_channel("Dimmer Speed"), ChannelType::Other);
    assert_eq!(classify_channel("Dimmer Time"), ChannelType::Other);
}

#[test]
fn test_fixture_channel_capabilities() {
    let fixture = PatchedFixture {
        id: 1,
        groups: vec![0],
        name: "Multi Capabilities".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "Dimmer",
                {"name": "Color Wheel", "capabilities": [{"dmxRange": [1, 5], "name": "Red"}]},
                {"channel": "Green"}
            ]
        }),
        universe: 1,
        address: 1,
    };

    let caps = get_fixture_channel_capabilities(&fixture, 1);
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0]["name"], "Red");
    assert_eq!(caps[0]["dmxRange"][0], 1);

    // Plain string channels and unknown indices yield no capabilities
    assert!(get_fixture_channel_capabilities(&fixture, 0).is_empty());
    assert!(get_fixture_channel_capabilities(&fixture, 2).is_empty());
    assert!(get_fixture_channel_capabilities(&fixture, 9).is_empty());
}

#[test]
fn test_fixture_channel_helpers() {
    let fixture = PatchedFixture {
        id: 1,
        groups: vec![0],
        name: "Test Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "Dimmer",
                {"name": "Red"},
                {"channel": "Green"},
                "Blue"
            ]
        }),
        universe: 1,
        address: 1,
    };

    assert_eq!(extract_fixture_channel_count(&fixture), 4);
    assert_eq!(
        get_fixture_channel_name(&fixture, 0),
        Some("Dimmer".to_string())
    );
    assert_eq!(
        get_fixture_channel_name(&fixture, 1),
        Some("Red".to_string())
    );
    assert_eq!(
        get_fixture_channel_name(&fixture, 2),
        Some("Green".to_string())
    );
    assert_eq!(
        get_fixture_channel_name(&fixture, 3),
        Some("Blue".to_string())
    );
    assert_eq!(get_fixture_channel_name(&fixture, 4), None);
}
