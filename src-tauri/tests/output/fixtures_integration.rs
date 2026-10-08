use std::collections::HashMap;

use live_dmx_lib::output::LiveEngine;
use live_dmx_lib::show::{EmbeddedGdtfDefinition, EmbeddedGdtfMode, PatchedFixture, ShowData};

#[test]
fn test_integration_mac_aura_pxl_all_channels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // Setup MAC Aura PXL definition with GDTF defaults and shutterOpenValues
    let mut shutter_map = HashMap::new();
    shutter_map.insert(0, 201); // 0-based index (channel 0): open is 201

    let mode = EmbeddedGdtfMode {
        name: "Compact".to_string(),
        short_name: None,
        channel_count: 17,
        channels: vec![
            "Strobe".to_string(),
            "Dimmer".to_string(),
            "Dimmer Fine".to_string(),
            "ColorRGB_Red".to_string(),
            "ColorRGB_Red Fine".to_string(),
            "ColorRGB_Green".to_string(),
            "ColorRGB_Green Fine".to_string(),
            "ColorRGB_Blue".to_string(),
            "ColorRGB_Blue Fine".to_string(),
            "CTC".to_string(),
            "Color1".to_string(),
            "Zoom".to_string(),
            "Pan".to_string(),
            "Pan Fine".to_string(),
            "Tilt".to_string(),
            "Tilt Fine".to_string(),
            "Control1".to_string(),
        ],
        channel_defaults: Some(vec![
            30, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 128, 128, 0, 128, 0, 0,
        ]),
        shutter_open_values: Some(shutter_map),
        pan_bounds: Some([-270, 270]),
        tilt_bounds: Some([-135, 135]),
    };

    show.fixture_definitions.insert(
        "martin professional@mac aura pxl".to_string(),
        EmbeddedGdtfDefinition {
            name: "MAC Aura PXL".to_string(),
            short_name: None,
            manufacturer: "Martin Professional".to_string(),
            description: None,
            fixture_type_id: None,
            rid: None,
            modes: vec![mode],
        },
    );

    // Patch fixture (initially without duplicated shutterOpenValues to test hydration)
    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "MAC Aura PXL 1".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "mac_aura_pxl",
            "name": "MAC Aura PXL",
            "manufacturerKey": "martin_professional"
        }),
        mode: serde_json::json!({
            "name": "Compact",
            "channels": [
                "Strobe", "Dimmer", "Dimmer Fine",
                "ColorRGB_Red", "ColorRGB_Red Fine",
                "ColorRGB_Green", "ColorRGB_Green Fine",
                "ColorRGB_Blue", "ColorRGB_Blue Fine",
                "CTC", "Color1", "Zoom",
                "Pan", "Pan Fine", "Tilt", "Tilt Fine", "Control1"
            ]
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout state (Effect = None)
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    // Shutter must be default 30 (Open/No function, NOT 255 fast random strobe!)
    assert_eq!(buf[0], 30, "Blackout: Shutter should be default open 30");
    // Dimmer must be 0
    assert_eq!(buf[1], 0, "Blackout: Dimmer must be 0");
    assert_eq!(buf[2], 0, "Blackout: Dimmer fine must be 0");
    // Position must be centered (Pan=128, Tilt=128)
    assert_eq!(buf[12], 128, "Blackout: Pan must be centered");
    assert_eq!(buf[14], 128, "Blackout: Tilt must be centered");
    // Zoom must be default 128
    assert_eq!(buf[11], 128, "Blackout: Zoom must be default 128");

    // 2. Full Intensity with White (Group 1 Effect = "full", White color)
    engine.set_group_color(1, Some("white".to_string()), Some(&show));
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    // Shutter must be 201 (GDTF open shutter value, NOT 255 fast random strobe!)
    assert_eq!(buf[0], 201, "Full: Shutter must be 201 (open)");
    assert_eq!(buf[1], 255, "Full: Dimmer must be 255");
    // RGB must be full white
    assert_eq!(buf[3], 255, "Full: Red must be 255");
    assert_eq!(buf[5], 255, "Full: Green must be 255");
    assert_eq!(buf[7], 255, "Full: Blue must be 255");

    // 3. Color mixing: Blue
    engine.set_color_fade_speed(0.0);
    engine.set_group_color(1, Some("blue".to_string()), Some(&show));
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 201, "Blue: Shutter must remain 201");
    assert_eq!(buf[1], 255, "Blue: Dimmer must be 255");
    assert_eq!(buf[3], 0, "Blue: Red must be 0");
    assert_eq!(buf[5], 0, "Blue: Green must be 0");
    assert_eq!(buf[7], 255, "Blue: Blue must be 255");
}

#[test]
fn test_integration_robe_ibolt_all_channels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    let mut shutter_map = HashMap::new();
    shutter_map.insert(34, 224); // 0-based offset (channel 34) for Shutter: open is 224

    let mut channels = vec![
        "Pan".to_string(),
        "Pan Fine".to_string(),
        "Tilt".to_string(),
        "Tilt Fine".to_string(),
        "PanRotate".to_string(),
        "PositionMSpeed".to_string(),
        "Control1".to_string(),
        "Channel 8".to_string(),
        "Color1".to_string(),
        "Color1 Fine".to_string(),
        "Cyan".to_string(),
        "Magenta".to_string(),
        "Yellow".to_string(),
    ];
    // Fill channels up to 34
    for i in 14..=34 {
        channels.push(format!("Channel {}", i));
    }
    channels.push("Shutter".to_string()); // Ch 35 (0-indexed 34)
    channels.push("Dimmer".to_string()); // Ch 36 (0-indexed 35)
    channels.push("Dimmer Fine".to_string()); // Ch 37 (0-indexed 36)

    let mut defaults = vec![128, 0, 128, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0];
    defaults.extend(std::iter::repeat_n(0, 21));
    defaults.push(32); // Default Shutter is 32 (Open)
    defaults.push(0); // Default Dimmer is 0
    defaults.push(0);

    let mode = EmbeddedGdtfMode {
        name: "Mode 1 - Standard 16 bit".to_string(),
        short_name: None,
        channel_count: 37,
        channels: channels.clone(),
        channel_defaults: Some(defaults),
        shutter_open_values: Some(shutter_map),
        pan_bounds: Some([-270, 270]),
        tilt_bounds: Some([-135, 135]),
    };

    show.fixture_definitions.insert(
        "robe lighting@robin ibolt".to_string(),
        EmbeddedGdtfDefinition {
            name: "Robin iBolt".to_string(),
            short_name: None,
            manufacturer: "Robe Lighting".to_string(),
            description: None,
            fixture_type_id: None,
            rid: None,
            modes: vec![mode],
        },
    );

    show.patch.fixtures.push(PatchedFixture {
        id: 2,
        groups: vec![2],
        name: "Robin iBolt 1".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "robin_ibolt",
            "name": "Robin iBolt",
            "manufacturerKey": "robe_lighting"
        }),
        mode: serde_json::json!({
            "name": "Mode 1 - Standard 16 bit",
            "channels": channels
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout: Pan/Tilt center, Shutter open (32), Dimmer 0
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 128, "Blackout: Pan must be centered");
    assert_eq!(buf[2], 128, "Blackout: Tilt must be centered");
    assert_eq!(buf[7], 5, "Blackout: LidarControl must be default 5");
    assert_eq!(
        buf[34], 32,
        "Blackout: Shutter must be default 32 (NOT 255 blackout!)"
    );
    assert_eq!(buf[35], 0, "Blackout: Dimmer must be 0");

    // 2. Full Intensity: Shutter must be 224 (OPEN! NOT 255 blackout!), Dimmer 255
    engine.set_group_effect(2, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(
        buf[34], 224,
        "Full: Shutter must be 224 (Open, NOT 255 blackout!)"
    );
    assert_eq!(buf[35], 255, "Full: Dimmer must be 255");
    assert_eq!(buf[0], 128, "Full: Pan must be centered");
    assert_eq!(buf[2], 128, "Full: Tilt must be centered");

    // 3. CMY Color mixing: Red (Cyan=0, Magenta=255, Yellow=255)
    engine.set_color_fade_speed(0.0);
    engine.set_group_color(2, Some("red".to_string()), Some(&show));
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[34], 224, "Red: Shutter must remain 224");
    assert_eq!(buf[35], 255, "Red: Dimmer must be 255");
    assert_eq!(buf[10], 0, "Red CMY: Cyan must be 0");
    assert_eq!(buf[11], 255, "Red CMY: Magenta must be 255");
    assert_eq!(buf[12], 255, "Red CMY: Yellow must be 255");
}

#[test]
fn test_integration_glp_x4_bar_20_all_channels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // Mode Dual Pixel High Resolution has 9 base channels + 20 RGBW pixels (89 total channels)
    // In GDTF, channel 6 was Dummy Function / NoFeature.
    let mut channels = vec![
        "NoFeature".to_string(),      // Channel 1: Tilt (raw GDTF NoFeature)
        "NoFeature Fine".to_string(), // Channel 2: Tilt Fine (raw GDTF NoFeature Fine)
        "NoFeature".to_string(),      // Channel 3: Shutter!
        "Dimmer".to_string(),
        "Dimmer Fine".to_string(),
        "NoFeature".to_string(), // Channel 6: CTO
        "Zoom".to_string(),
        "Zoom Fine".to_string(),
        "Control".to_string(),
    ];
    // Add 20 pixels of RGBW
    for _ in 1..=20 {
        channels.push("Red".to_string());
        channels.push("Green".to_string());
        channels.push("Blue".to_string());
        channels.push("White".to_string());
    }

    let mode = EmbeddedGdtfMode {
        name: "Dual Pixel High Resolution".to_string(),
        short_name: None,
        channel_count: channels.len(),
        channels: channels.clone(),
        channel_defaults: None,
        shutter_open_values: None,
        pan_bounds: None,
        tilt_bounds: Some([-105, 105]),
    };

    show.fixture_definitions.insert(
        "glp@dummy - impression x4 bar 20".to_string(),
        EmbeddedGdtfDefinition {
            name: "Dummy - Impression X4 Bar 20".to_string(),
            short_name: None,
            manufacturer: "GLP".to_string(),
            description: None,
            fixture_type_id: None,
            rid: None,
            modes: vec![mode],
        },
    );

    show.patch.fixtures.push(PatchedFixture {
        id: 3,
        groups: vec![3],
        name: "Impression X4 Bar 20 1".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "impression_x4_bar_20",
            "name": "Impression X4 Bar 20",
            "gdtfSpec": "GLP@Impression X4 Bar 20",
            "manufacturerKey": "glp"
        }),
        mode: serde_json::json!({
            "name": "Dual Pixel High Resolution",
            "channels": channels
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout: Master Dimmer = 0, Tilt = 128
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 128, "Blackout: Tilt must be centered");
    // In blackout, Master Dimmer (ch 4) is 0 so the fixture is completely blacked out
    assert_eq!(buf[3], 0, "Blackout: Master Dimmer must be 0");

    // 2. Full Intensity with White:
    // Master Dimmer must be 255, Shutter must be 255 (OPEN! NOT 0 closed!), and all 20 pixels lit!
    engine.set_group_color(3, Some("white".to_string()), Some(&show));
    engine.set_group_effect(3, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 128, "Full: Tilt must be centered");
    assert_eq!(buf[2], 255, "Full: Shutter (ch 3) must be 255 (OPEN!)");
    assert_eq!(buf[3], 255, "Full: Master Dimmer must be 255");

    for p in 0..20 {
        let base = 9 + p * 4;
        assert_eq!(buf[base], 255, "Full: Pixel {} Red must be 255", p + 1);
        assert_eq!(
            buf[base + 1],
            255,
            "Full: Pixel {} Green must be 255",
            p + 1
        );
        assert_eq!(buf[base + 2], 255, "Full: Pixel {} Blue must be 255", p + 1);
        assert_eq!(
            buf[base + 3],
            255,
            "Full: Pixel {} White must be 255",
            p + 1
        );
    }

    // 3. Green color:
    engine.set_color_fade_speed(0.0);
    engine.set_group_color(3, Some("green".to_string()), Some(&show));
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[2], 255, "Green: Shutter must be 255 (Open)");
    assert_eq!(buf[3], 255, "Green: Master Dimmer must be 255");
    for p in 0..20 {
        let base = 9 + p * 4;
        assert_eq!(buf[base], 0, "Green: Pixel {} Red must be 0", p + 1);
        assert_eq!(
            buf[base + 1],
            255,
            "Green: Pixel {} Green must be 255",
            p + 1
        );
        assert_eq!(buf[base + 2], 0, "Green: Pixel {} Blue must be 0", p + 1);
        assert_eq!(buf[base + 3], 0, "Green: Pixel {} White must be 0", p + 1);
    }
}

#[test]
fn test_integration_glp_jdc2_ip_all_channels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // JDC2 IP Mode M1 - Dual Strobe (24 channels):
    // Channels 1-2: Tilt, Tilt Fine
    // Channel 3: Control1
    // Channel 4: Mix Priority
    // Channels 5-6: Dimmer Beam, Dimmer Beam Fine (Central White Strobe Tube)
    // Channel 7: StrobeDuration Beam
    // Channel 8: Shutter Beam
    // Channel 9: StrobeModeEffect Beam
    // Channels 10-11: Dimmer Plate, Dimmer Plate Fine (Wash Plates)
    // Channel 12: StrobeDuration Plate
    // Channel 13: Shutter Plate
    // Channel 14: StrobeModeEffect Plate
    // Channel 15: CTC
    // Channels 16-18: Red, Green, Blue (Plate)
    // Channels 19-20: Dimmer Sub Module
    // Channel 21: White (Beam)
    // Channels 22-24: Red, Green, Blue (Plate)
    let channels = vec![
        "Tilt".to_string(),
        "Tilt Fine".to_string(),
        "Control1".to_string(),
        "Channel 4".to_string(),
        "Dimmer Beam".to_string(), // Ch 5 (0-index 4)
        "Dimmer Beam Fine".to_string(),
        "StrobeDuration".to_string(),
        "Strobe".to_string(),
        "Channel 9".to_string(),
        "Dimmer Plate".to_string(), // Ch 10 (0-index 9)
        "Dimmer Plate Fine".to_string(),
        "StrobeDuration".to_string(),
        "Strobe".to_string(),
        "Channel 14".to_string(),
        "CTC".to_string(),
        "Red".to_string(), // Ch 16 (0-index 15)
        "Green".to_string(),
        "Blue".to_string(),
        "Dimmer".to_string(), // Ch 19
        "Dimmer Fine".to_string(),
        "White (Beam)".to_string(), // Ch 21 (0-index 20)
        "Red".to_string(),          // Ch 22
        "Green".to_string(),
        "Blue".to_string(),
    ];

    let mode = EmbeddedGdtfMode {
        name: "M1 - Dual Strobe".to_string(),
        short_name: None,
        channel_count: 24,
        channels: channels.clone(),
        channel_defaults: Some(vec![
            127, 255, 0, 191, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 255,
            255,
        ]),
        shutter_open_values: None,
        pan_bounds: None,
        tilt_bounds: Some([-92, 92]),
    };

    show.fixture_definitions.insert(
        "glp@jdc2 ip".to_string(),
        EmbeddedGdtfDefinition {
            name: "JDC2 IP".to_string(),
            short_name: None,
            manufacturer: "GLP".to_string(),
            description: None,
            fixture_type_id: None,
            rid: None,
            modes: vec![mode],
        },
    );

    show.patch.fixtures.push(PatchedFixture {
        id: 4,
        groups: vec![4],
        name: "JDC2 IP 1".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "jdc2_ip",
            "name": "JDC2 IP",
            "manufacturerKey": "glp"
        }),
        mode: serde_json::json!({
            "name": "M1 - Dual Strobe",
            "channels": channels
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout: Tilt center, all dimmers 0
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 127, "Blackout: Tilt default");
    assert_eq!(buf[4], 0, "Blackout: Dimmer Beam must be 0");
    assert_eq!(buf[9], 0, "Blackout: Dimmer Plate must be 0");

    // 2. Full Intensity Wash with Red color:
    // CRITICAL: Dimmer Beam (ch 5) and White Beam (ch 21) must STAY 0!
    // Dimmer Plate (ch 10) must be 255, Plate RGB must be Red (255, 0, 0)!
    engine.set_color_fade_speed(0.0);
    engine.set_group_color(4, Some("red".to_string()), Some(&show));
    engine.set_group_effect(4, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();

    // Central white strobe tube MUST BE OFF:
    assert_eq!(
        buf[4], 0,
        "Full Wash: Dimmer Beam MUST be 0 (center strobe bar ignored)"
    );
    assert_eq!(buf[5], 0, "Full Wash: Dimmer Beam Fine MUST be 0");
    assert_eq!(
        buf[20], 0,
        "Full Wash: White (Beam) MUST be 0 (center strobe bar ignored)"
    );

    // Wash Plates MUST BE FULL RED:
    assert_eq!(buf[9], 255, "Full Wash: Dimmer Plate must be 255");
    assert_eq!(buf[15], 255, "Full Wash: Main Plate Red must be 255");
    assert_eq!(buf[16], 0, "Full Wash: Main Plate Green must be 0");
    assert_eq!(buf[17], 0, "Full Wash: Main Plate Blue must be 0");

    assert_eq!(buf[21], 255, "Full Wash: Sub Plate Red must be 255");
    assert_eq!(buf[22], 0, "Full Wash: Sub Plate Green must be 0");
    assert_eq!(buf[23], 0, "Full Wash: Sub Plate Blue must be 0");
}

#[test]
fn test_integration_dmx_mover_untouched_by_intensity() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // Capture Visualiser DMX Mover (16-bit Y mode): 2 channels PositionXYZ
    show.patch.fixtures.push(PatchedFixture {
        id: 5,
        groups: vec![5],
        name: "DMX Mover 1".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "dmx_mover",
            "name": "DMX Mover",
            "manufacturerKey": "capture_visualiser"
        }),
        mode: serde_json::json!({
            "name": "16-bit Y",
            "channels": ["PositionXYZ", "PositionXYZ Fine"]
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout: Position stays default (255 / max at top)
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255, "DMX Mover: Y position stays at top 255");

    // 2. Full Intensity effect: DMX Mover must NOT change and stay at top 255
    engine.set_group_effect(5, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(
        buf[0], 255,
        "DMX Mover: Y position stays at top 255 even on Full effect"
    );
}

#[test]
fn test_integration_chauvet_epix_strip_all_channels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // 5 Section Mode: 15 channels (5 x RGB)
    let channels = vec![
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
    ];

    show.patch.fixtures.push(PatchedFixture {
        id: 6,
        groups: vec![6],
        name: "Epix Strip IP 1".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "epix_strip_ip",
            "name": "Epix Strip IP",
            "manufacturerKey": "chauvet"
        }),
        mode: serde_json::json!({
            "name": "5 Section Mode",
            "channels": channels
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout: All 15 channels must be 0
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    for (ch, &val) in buf.iter().enumerate().take(15) {
        assert_eq!(val, 0, "Blackout: Epix Strip ch {} must be 0", ch + 1);
    }

    // 2. Full Intensity White: All 15 channels must be 255
    engine.set_group_color(6, Some("white".to_string()), Some(&show));
    engine.set_group_effect(6, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    for (ch, &val) in buf.iter().enumerate().take(15) {
        assert_eq!(val, 255, "Full White: Epix Strip ch {} must be 255", ch + 1);
    }

    // 3. Full Intensity Red: Red channels 255, Green/Blue 0
    engine.set_color_fade_speed(0.0);
    engine.set_group_color(6, Some("red".to_string()), Some(&show));
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    for s in 0..5 {
        assert_eq!(buf[s * 3], 255, "Red: Section {} Red must be 255", s + 1);
        assert_eq!(buf[s * 3 + 1], 0, "Red: Section {} Green must be 0", s + 1);
        assert_eq!(buf[s * 3 + 2], 0, "Red: Section {} Blue must be 0", s + 1);
    }
}

#[test]
fn test_integration_gigapix_wash_neutral_white() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // GigaPix Wash mode (15 channels, no Master Dimmer):
    // Pan, Pan Fine, Tilt, Tilt Fine, Red, Green, Blue, Lime, CTB, Zoom, ...
    let channels = vec![
        "Pan".to_string(),
        "Pan Fine".to_string(),
        "Tilt".to_string(),
        "Tilt Fine".to_string(),
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
        "Lime".to_string(),
        "CTB".to_string(),
        "Zoom".to_string(),
    ];

    show.patch.fixtures.push(PatchedFixture {
        id: 7,
        groups: vec![7],
        name: "GigaPix 1".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "gigapix",
            "name": "GigaPix",
            "manufacturerKey": "high_end_systems"
        }),
        mode: serde_json::json!({
            "name": "Wash",
            "channels": channels
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout: Red, Green, Blue must be 0 (no master dimmer)
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 128, "Blackout: Pan centered");
    assert_eq!(buf[2], 128, "Blackout: Tilt centered");
    assert_eq!(buf[4], 0, "Blackout: Red must be 0");
    assert_eq!(buf[5], 0, "Blackout: Green must be 0");
    assert_eq!(buf[6], 0, "Blackout: Blue must be 0");
    assert_eq!(buf[7], 0, "Blackout: Lime must be 0");

    // 2. Full Intensity White: Red, Green, Blue at 255, Lime at 0 (pure clean white, NOT warm!)
    engine.set_group_color(7, Some("white".to_string()), Some(&show));
    engine.set_group_effect(7, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[4], 255, "Full White: Red must be 255");
    assert_eq!(buf[5], 255, "Full White: Green must be 255");
    assert_eq!(buf[6], 255, "Full White: Blue must be 255");
    assert_eq!(
        buf[7], 0,
        "Full White: Lime MUST be 0 so white is clean and pure!"
    );
}

#[test]
fn test_integration_cameo_opus_sp5p_all_channels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // 34CH Mode Standard:
    // Pan, Pan Fine, Tilt, Tilt Fine, Dimmer, Dimmer Fine, Strobe, Cyan, Magenta, Yellow, ...
    let mut channels = vec![
        "Pan".to_string(),
        "Pan Fine".to_string(),
        "Tilt".to_string(),
        "Tilt Fine".to_string(),
        "Dimmer".to_string(),
        "Dimmer Fine".to_string(),
        "Strobe".to_string(),
        "Cyan".to_string(),
        "Magenta".to_string(),
        "Yellow".to_string(),
    ];
    for i in 11..=34 {
        channels.push(format!("Channel {}", i));
    }

    show.patch.fixtures.push(PatchedFixture {
        id: 8,
        groups: vec![8],
        name: "OPUS SP5P 1".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "opus_sp5p",
            "name": "OPUS SP5P",
            "manufacturerKey": "cameo"
        }),
        mode: serde_json::json!({
            "name": "34CH Mode Standard",
            "channels": channels
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout: Dimmer 0, Pan/Tilt centered
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 128, "Blackout: Pan centered");
    assert_eq!(buf[2], 128, "Blackout: Tilt centered");
    assert_eq!(buf[4], 0, "Blackout: Dimmer must be 0");

    // 2. Full Intensity White: Dimmer 255, Shutter 255 (Open), CMY all 0 (no absorption filters)
    engine.set_group_color(8, Some("white".to_string()), Some(&show));
    engine.set_group_effect(8, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[4], 255, "Full: Dimmer must be 255");
    assert_eq!(buf[6], 255, "Full: Shutter must be 255");
    assert_eq!(buf[7], 0, "Full White: Cyan filter must be 0");
    assert_eq!(buf[8], 0, "Full White: Magenta filter must be 0");
    assert_eq!(buf[9], 0, "Full White: Yellow filter must be 0");

    // 3. Full Intensity Blue: Dimmer 255, Cyan 255, Magenta 255, Yellow 0
    engine.set_color_fade_speed(0.0);
    engine.set_group_color(8, Some("blue".to_string()), Some(&show));
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[4], 255, "Blue: Dimmer must be 255");
    assert_eq!(buf[7], 255, "Blue CMY: Cyan must be 255");
    assert_eq!(buf[8], 255, "Blue CMY: Magenta must be 255");
    assert_eq!(buf[9], 0, "Blue CMY: Yellow must be 0");
}

#[test]
fn test_integration_ayrton_eaglestrike_shutter_open_1_indexed() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // Ayrton EagleStrike: 1-indexed shutterOpenValues {"6": 246}
    let mut shutter_map = HashMap::new();
    shutter_map.insert(6, 246); // 1-indexed channel 6: Open is 246

    let channels = vec![
        "Pan".to_string(),
        "Pan Fine".to_string(),
        "Tilt".to_string(),
        "Tilt Fine".to_string(),
        "PanRotate".to_string(),
        "Strobe".to_string(), // Ch 6 (0-indexed 5)
        "Dimmer".to_string(), // Ch 7 (0-indexed 6)
        "Dimmer Fine".to_string(),
    ];

    let mode = EmbeddedGdtfMode {
        name: "Standard_540".to_string(),
        short_name: None,
        channel_count: channels.len(),
        channels: channels.clone(),
        channel_defaults: Some(vec![128, 0, 128, 0, 128, 10, 0, 0]), // default strobe is 10 (closed/LED OFF)
        shutter_open_values: Some(shutter_map),
        pan_bounds: Some([-270, 270]),
        tilt_bounds: Some([-135, 135]),
    };

    show.fixture_definitions.insert(
        "ayrton@eaglestrike".to_string(),
        EmbeddedGdtfDefinition {
            name: "EagleStrike LT".to_string(),
            short_name: None,
            manufacturer: "Ayrton".to_string(),
            description: None,
            fixture_type_id: None,
            rid: None,
            modes: vec![mode],
        },
    );

    show.patch.fixtures.push(PatchedFixture {
        id: 301,
        groups: vec![1],
        name: "EagleStrike 301".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "eaglestrike",
            "name": "EagleStrike LT",
            "manufacturerKey": "ayrton"
        }),
        mode: serde_json::json!({
            "name": "Standard_540",
            "channels": channels
        }),
        universe: 1,
        address: 1,
    });

    // 1. Blackout: Dimmer 0, Shutter default 10 (Closed)
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[5], 10, "Blackout: Strobe default 10 (closed)");
    assert_eq!(buf[6], 0, "Blackout: Dimmer 0");

    // 2. Full Intensity: Shutter MUST be 246 (OPEN!), Dimmer 255
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[5], 246, "Full: Strobe MUST be 246 (Open!)");
    assert_eq!(buf[6], 255, "Full: Dimmer 255");
    assert_eq!(buf[7], 255, "Full: Dimmer Fine 255");
}
