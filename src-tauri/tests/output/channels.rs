use live_dmx_lib::output::{
    classify_channel, extract_fixture_channel_count, get_fixture_channel_name, ChannelType,
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
    assert_eq!(classify_channel("Dimmer fine"), ChannelType::Other);
    assert_eq!(classify_channel("Red Fine"), ChannelType::Other);
    assert_eq!(classify_channel("Green LSB"), ChannelType::Other);
    assert_eq!(classify_channel("Pan (16 bit)"), ChannelType::Other);
    assert_eq!(classify_channel("Pan"), ChannelType::Other);
    assert_eq!(classify_channel("Tilt"), ChannelType::Other);
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
