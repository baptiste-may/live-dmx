use live_dmx_lib::output::LiveEngine;
use live_dmx_lib::show::{
    ColorPreference, GroupSettings, PatchedFixture, PixelPreference, ShowData,
};

#[test]
fn test_live_engine_default_state_and_blackout() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Test Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer"]}),
        universe: 1,
        address: 1,
    });

    // Default effect is None -> Blackout (0 intensity)
    engine.compute_dmx(&show);
    let buf_default = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf_default[0], 0);

    // Set Group 1 effect to Full -> 255 intensity
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf_full = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf_full[0], 255);
}

#[test]
fn test_live_engine_group_zero_all_control() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Spot 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer"]}),
        universe: 1,
        address: 1,
    });

    // Group 0 ("All") effect sets full intensity on fixture 1 (even though fixture is in group 1)
    engine.set_group_effect(0, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255);
}

#[test]
fn test_live_engine_manual_dmx_overrides() {
    let mut engine = LiveEngine::new();
    let show = ShowData::default();

    engine.manual_channels.insert((1, 10), 128);
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[9], 128);
}

#[test]
fn test_live_engine_rgb_fixture_without_dimmer() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "RGB Par".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Red", "Green", "Blue"]}),
        universe: 1,
        address: 1,
    });

    // Color is blue, effect is Full
    engine.set_color_fade_speed(0.0);
    engine.set_group_color(1, Some("blue".to_string()), Some(&show));
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 0); // Red
    assert_eq!(buf[1], 0); // Green
    assert_eq!(buf[2], 255); // Blue

    // Effect is None (Blackout) -> Blue should dim to 0
    engine.set_group_effect(1, None, None, false);
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 0);
    assert_eq!(buf[1], 0);
    assert_eq!(buf[2], 0);
}

#[test]
fn test_live_engine_group_master_scaling() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer"]}),
        universe: 1,
        address: 1,
    });

    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_master(1, 50);
    assert_eq!(engine.get_group_master(1), 50);

    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 127);
}

#[test]
fn test_live_engine_group_max_power_scales_master() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer"]}),
        universe: 1,
        address: 1,
    });

    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_master(1, 100);

    // Without a max power setting the group is not limited
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255);

    // max_power scales the group master down: 100% master * 50% limit = 50%
    show.groups.settings.insert(
        1,
        GroupSettings {
            max_power: 50,
            ..GroupSettings::default()
        },
    );

    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 127);

    // The global "All" group limit multiplies on top of the group limit
    show.groups.settings.insert(
        0,
        GroupSettings {
            max_power: 25,
            ..GroupSettings::default()
        },
    );

    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 31);
}

#[test]
fn test_live_engine_global_max_power_applied_once_without_groups() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![],
        name: "Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer"]}),
        universe: 1,
        address: 1,
    });

    engine.set_group_effect(0, Some("full".to_string()), None, false);
    engine.set_group_master(0, 100);

    show.groups.settings.insert(
        0,
        GroupSettings {
            max_power: 50,
            ..GroupSettings::default()
        },
    );

    // The "All" limit applies once even though primary_group_id defaults to 0
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 127);
}

#[test]
fn test_live_engine_max_power_caps_simple_flash() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer"]}),
        universe: 1,
        address: 1,
    });

    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_master(1, 100);
    engine.set_flash(1, "simple".to_string(), true);

    // Without max power the flash reaches full
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255);

    // With a max power limit the flash is capped at the limit
    show.groups.settings.insert(
        1,
        GroupSettings {
            max_power: 50,
            ..GroupSettings::default()
        },
    );

    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 128);
}

#[test]
fn test_live_engine_max_power_caps_strobe() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer"]}),
        universe: 1,
        address: 1,
    });

    show.groups.settings.insert(
        1,
        GroupSettings {
            max_power: 50,
            ..GroupSettings::default()
        },
    );

    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_master(1, 100);
    engine.set_flash(1, "strobe".to_string(), true);

    // Sample the strobe until both phases were seen: the on-phase must open at
    // the max power ceiling and never at full.
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    let (mut saw_on, mut saw_off) = (false, false);
    while std::time::Instant::now() < deadline && !(saw_on && saw_off) {
        engine.compute_dmx(&show);
        match engine.universe_buffers.get(&1).unwrap()[0] {
            0 => saw_off = true,
            128 => saw_on = true,
            other => panic!("strobe dimmer exceeded the max power ceiling: {other}"),
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(saw_on, "strobe phase never reached the 50% ceiling");
    assert!(saw_off, "strobe phase never went dark");
}

#[test]
fn test_live_engine_buttons_and_reset() {
    let mut engine = LiveEngine::new();
    engine.set_button("btn_1".to_string(), true);
    assert_eq!(engine.button_states.get("btn_1"), Some(&true));

    engine.reset_all();
    assert!(engine.button_states.is_empty());
    assert_eq!(engine.strobe_rate, 50);
    assert_eq!(engine.random_strobe_rate, 50);
    assert_eq!(engine.effect_bpm, 60);
    assert_eq!(engine.get_group_color(0), Some("white"));
    assert_eq!(engine.get_group_color(1), None);
}

#[test]
fn test_live_engine_color_effects_and_secondary_colors() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "RGB Par 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue"]}),
        universe: 1,
        address: 1,
    });

    // Set intensity to full
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    // Primary color = Blue, Secondary color = Red
    engine.set_group_color(1, Some("blue".to_string()), Some(&show));
    engine.set_group_secondary_color(1, Some("red".to_string()));

    assert_eq!(engine.get_group_color(1), Some("blue"));
    assert_eq!(engine.get_group_secondary_color(1), Some("red"));

    // No color effect -> primary color (Blue: 0, 0, 255)
    engine.compute_dmx(&show);
    let buf_base = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf_base[0], 255); // Dimmer
    assert_eq!(buf_base[1], 0); // Red
    assert_eq!(buf_base[2], 0); // Green
    assert_eq!(buf_base[3], 255); // Blue

    // Set color effect to chase
    engine.set_group_color_effect(1, Some("chase".to_string()), None, false);
    assert_eq!(
        engine.get_group_color_effect(1).effect,
        Some("chase".to_string())
    );

    engine.compute_dmx(&show);
    let buf_chase = engine.universe_buffers.get(&1).unwrap();
    // In chase, color is either 100% blue or 100% red
    assert_eq!(buf_chase[0], 255);
    assert!(
        (buf_chase[1] == 0 && buf_chase[3] == 255) || (buf_chase[1] == 255 && buf_chase[3] == 0)
    );

    // Test color effect BPM and Width
    engine.set_color_effect_bpm(130);
    assert_eq!(engine.color_effect_bpm, 130);
    engine.set_group_color_effect_width(1, 75);
    assert_eq!(engine.get_group_color_effect(1).width, 75);

    // Reset color effect
    engine.reset_group_color_effect(1);
    assert_eq!(engine.get_group_color_effect(1).effect, None);
    assert_eq!(engine.get_group_color_effect(1).width, 100);

    // Reset all
    engine.reset_all();
    assert_eq!(engine.get_group_secondary_color(1), None);
    assert_eq!(engine.color_effect_bpm, 60);
    assert_eq!(engine.get_group_color_effect(1).width, 100);
}

#[test]
fn test_live_engine_rainbow_color_effect() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "RGB Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue"]}),
        universe: 1,
        address: 1,
    });

    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_color_effect(1, Some("rainbow".to_string()), None, false);
    engine.set_color_effect_bpm(60);

    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255);
    assert!(buf[1] > 0 || buf[2] > 0 || buf[3] > 0);
}

#[test]
fn test_live_engine_universe_buffers_auto_create_and_get_dmx() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // Add universe 5 to output configuration
    show.output
        .universes
        .push(live_dmx_lib::show::UniverseOutputConfig {
            universe: 5,
            output: "none".to_string(),
            enabled: true,
        });

    // Add a fixture patched to universe 6
    show.patch.fixtures.push(PatchedFixture {
        id: 42,
        groups: vec![1],
        name: "Univ 6 Dimmer".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer"]}),
        universe: 6,
        address: 10,
    });

    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);

    // Universe 5 should exist and be 512 bytes of 0
    let u5_buf = engine.universe_buffers.get(&5);
    assert!(u5_buf.is_some());
    assert_eq!(u5_buf.unwrap().len(), 512);

    // Universe 6 should exist and channel 10 (idx 9) should be 255
    let u6_buf = engine.universe_buffers.get(&6);
    assert!(u6_buf.is_some());
    assert_eq!(u6_buf.unwrap()[9], 255);
}

#[test]
fn test_live_engine_color_effect_does_not_get_stuck_on_secondary_color() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "RGB Par 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue"]}),
        universe: 1,
        address: 1,
    });

    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    // Primary color = Blue, Secondary color = Red
    engine.set_group_color(1, Some("blue".to_string()), Some(&show));
    engine.set_group_secondary_color(1, Some("red".to_string()));

    // Activate sine color effect
    engine.set_group_color_effect(1, Some("sine".to_string()), None, false);
    engine.set_color_effect_bpm(60);

    // Run multiple frames of compute_dmx and observe that secondary color (Red) is actively mixed
    let mut observed_secondary = false;
    for _ in 0..20 {
        engine.compute_dmx(&show);
        let buf = engine.universe_buffers.get(&1).unwrap();
        if buf[1] > 0 {
            observed_secondary = true;
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    assert!(
        observed_secondary,
        "Effect should actively mix secondary color"
    );

    // Now reset color effect: must return to 100% primary color (Blue), NOT stuck on secondary color (Red)!
    engine.reset_group_color_effect(1);
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255); // Dimmer
    assert_eq!(buf[1], 0); // Red must be 0 (not stuck on secondary!)
    assert_eq!(buf[2], 0); // Green
    assert_eq!(buf[3], 255); // Blue must be 255 (primary color preserved!)
}

#[test]
fn test_red_blue_pulse_channels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "RGBAW Par 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": [
            "Dimmer",
            "Red",
            "Green",
            "Blue",
            "White",
            "Amber",
            "Yellow"
        ]}),
        universe: 1,
        address: 1,
    });

    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_color(1, Some("red".to_string()), Some(&show));
    engine.set_group_secondary_color(1, Some("blue".to_string()));
    engine.set_group_color_effect(1, Some("pulse".to_string()), None, false);
    engine.set_color_effect_bpm(120);

    // Track red and blue values across pulse cycle
    let mut min_blue = 255u8;
    let mut max_blue = 0u8;

    // Compute across multiple frames: Red and Blue should blend only between R and B.
    // Green, White, Amber, and Yellow channels must strictly remain 0.
    for _ in 0..30 {
        engine.compute_dmx(&show);
        let buf = engine.universe_buffers.get(&1).unwrap();
        assert_eq!(buf[0], 255); // Dimmer

        let b = buf[3];
        if b < min_blue {
            min_blue = b;
        }
        if b > max_blue {
            max_blue = b;
        }

        assert_eq!(buf[2], 0, "Green channel must be 0 for red and blue pulse");
        assert_eq!(buf[4], 0, "White channel must be 0 for red and blue pulse");
        assert_eq!(buf[5], 0, "Amber channel must be 0 for red and blue pulse");
        assert_eq!(buf[6], 0, "Yellow channel must be 0 for red and blue pulse");

        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    assert!(
        max_blue > 0,
        "Pulse effect should modulate secondary blue channel"
    );
    assert_eq!(
        min_blue, 0,
        "Pulse effect should return to pure primary red channel"
    );
}

#[test]
fn test_fixture_color_wheel_routing() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Wheel Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Color Wheel"]}),
        universe: 1,
        address: 1,
    });

    show.groups.settings.insert(
        1,
        GroupSettings {
            color_preference: ColorPreference::ColorWheel,
            ..GroupSettings::default()
        },
    );

    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_color(1, Some("red".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255); // Dimmer
    assert!(buf[1] > 0, "Color wheel channel should receive a position");
}

#[test]
fn test_wheel_only_fixture_falls_back_to_wheel_routing() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Wheel Par".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Color Wheel"]}),
        universe: 1,
        address: 1,
    });

    // No settings configured: effective routing must fall back to wheel
    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_color(1, Some("blue".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255);
    assert!(buf[1] > 0);
}

#[test]
fn test_color_mix_preference_zeroes_wheel_channel() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Hybrid Par".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue", "Color Wheel"]}),
        universe: 1,
        address: 1,
    });

    show.groups.settings.insert(
        1,
        GroupSettings {
            color_preference: ColorPreference::ColorMix,
            ..GroupSettings::default()
        },
    );

    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_color(1, Some("blue".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[1], 0); // Red
    assert_eq!(buf[2], 0); // Green
    assert_eq!(buf[3], 255); // Blue
    assert_eq!(buf[4], 0, "Wheel channel must stay at 0 under Color Mix");
}

#[test]
fn test_fixture_cct_routing() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "CCT Wash".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "CCT"]}),
        universe: 1,
        address: 1,
    });

    show.groups.settings.insert(
        1,
        GroupSettings {
            color_preference: ColorPreference::Cct,
            ..GroupSettings::default()
        },
    );

    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_color(1, Some("white".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255); // Dimmer
    let cct_value = buf[1];
    assert!(
        cct_value > 0 && cct_value < 255,
        "CCT channel should map white to an in-range temperature, got {cct_value}"
    );
}

#[test]
fn test_cct_preference_falls_back_to_rgb_on_mix_fixture() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "RGB Wash".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue"]}),
        universe: 1,
        address: 1,
    });

    // No CCT channel available: CCT preference must fall back to Color Mix
    show.groups.settings.insert(
        1,
        GroupSettings {
            color_preference: ColorPreference::Cct,
            ..GroupSettings::default()
        },
    );

    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_color(1, Some("green".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[2], 255); // Green
    assert_eq!(buf[1], 0); // Red
}

#[test]
fn test_moving_head_pan_tilt_defaults_and_shutter_open() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Test Moving Head".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "Dimmer",
                "Pan",
                "Pan Fine",
                "Tilt",
                "Tilt Fine",
                "Strobe",
                "Red",
                "Green",
                "Blue",
                "Lime"
            ],
            "channelDefaults": [0, 128, 0, 128, 0, 0, 0, 0, 0, 0],
            "shutterOpenValues": {"5": 32}
        }),
        universe: 1,
        address: 1,
    });

    // 1. Initial / dark state (Dimmer = 0)
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 0); // Dimmer closed
    assert_eq!(buf[1], 128); // Pan coarse center
    assert_eq!(buf[2], 0); // Pan fine
    assert_eq!(buf[3], 128); // Tilt coarse center
    assert_eq!(buf[4], 0); // Tilt fine
    assert_eq!(buf[5], 0); // Shutter closed when intensity is 0

    // 2. Active full intensity (Dimmer = 255, Color = White)
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_color(1, Some("white".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf_active = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf_active[0], 255); // Dimmer open
    assert_eq!(buf_active[1], 128); // Pan stays centered
    assert_eq!(buf_active[2], 0);
    assert_eq!(buf_active[3], 128); // Tilt stays centered
    assert_eq!(buf_active[4], 0);
    assert_eq!(buf_active[5], 32); // Shutter opens to open value!
    assert_eq!(buf_active[6], 255); // Red
    assert_eq!(buf_active[7], 255); // Green
    assert_eq!(buf_active[8], 255); // Blue
    assert_eq!(buf_active[9], 0); // Lime stays 0 on neutral white!
}

#[test]
fn test_position_xyz_unaffected_by_intensity() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "DMX Mover".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "XYZ_Y",
                "XYZ_Y Fine"
            ]
        }),
        universe: 1,
        address: 1,
    });

    // 1. With no effect / master 0
    engine.set_group_master(1, 0);
    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255, "XYZ_Y should default to 255 (top)");
    assert_eq!(buf[1], 255, "XYZ_Y Fine should default to 255");

    // 2. With full intensity on group 1
    engine.set_group_master(1, 100);
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.compute_dmx(&show);
    let buf_active = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(
        buf_active[0], 255,
        "XYZ_Y must not be modulated by intensity"
    );
    assert_eq!(buf_active[1], 255);
}

#[test]
fn test_pixel_preference_single_keeps_all_pixels_identical() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // Fixture with Tilt (ch0), Dimmer (ch1), Shutter (ch2), and 4 RGB pixels (ch3..14)
    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Pixel Bar 4".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "Tilt",
                "Dimmer",
                "Shutter",
                "Red 1", "Green 1", "Blue 1",
                "Red 2", "Green 2", "Blue 2",
                "Red 3", "Green 3", "Blue 3",
                "Red 4", "Green 4", "Blue 4",
            ]
        }),
        universe: 1,
        address: 1,
    });

    // Group 1 settings: Single
    show.groups.settings.insert(
        1,
        GroupSettings {
            pixel_preference: PixelPreference::Single,
            ..Default::default()
        },
    );

    engine.set_group_color(1, Some("red".to_string()), Some(&show));
    engine.set_group_effect(1, Some("full".to_string()), None, false);
    engine.set_group_master(1, 100);

    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    // Tilt is centered (128)
    assert_eq!(buf[0], 128);
    // Master Dimmer is 255
    assert_eq!(buf[1], 255);
    // Shutter is open (255)
    assert_eq!(buf[2], 255);

    // All 4 pixels are identical Red: (255, 0, 0)
    for p in 0..4 {
        let base = 3 + p * 3;
        assert_eq!(buf[base], 255, "Pixel {} red", p);
        assert_eq!(buf[base + 1], 0, "Pixel {} green", p);
        assert_eq!(buf[base + 2], 0, "Pixel {} blue", p);
    }
}

#[test]
fn test_pixel_preference_individual_splits_pixels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // Fixture with Dimmer (ch0), Shutter (ch1), and 4 RGB pixels (ch2..13)
    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Pixel Bar 4".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "Dimmer",
                "Shutter",
                "Red 1", "Green 1", "Blue 1",
                "Red 2", "Green 2", "Blue 2",
                "Red 3", "Green 3", "Blue 3",
                "Red 4", "Green 4", "Blue 4",
            ]
        }),
        universe: 1,
        address: 1,
    });

    // Group 1 settings: Individual
    show.groups.settings.insert(
        1,
        GroupSettings {
            pixel_preference: PixelPreference::Individual,
            ..Default::default()
        },
    );

    engine.set_group_color(1, Some("white".to_string()), Some(&show));
    // Set a ramp_up effect with left alignment and width 100
    engine.set_group_effect(
        1,
        Some("ramp_up".to_string()),
        Some("left".to_string()),
        false,
    );
    engine.set_group_effect_width(1, 100);
    engine.set_group_master(1, 100);

    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    // In Individual mode, at t=0 with left alignment and ramp_up:
    // Pixel 0: phase 0/4 => 0
    // Pixel 1: phase 1/4 => 64
    // Pixel 2: phase 2/4 => 128
    // Pixel 3: phase 3/4 => 191
    assert_eq!(buf[2], 0, "Pixel 0 red");
    assert_eq!(buf[5], 64, "Pixel 1 red");
    assert_eq!(buf[8], 128, "Pixel 2 red");
    assert_eq!(buf[11], 191, "Pixel 3 red");

    // A max power limit halves the per-pixel dimming
    show.groups.settings.insert(
        1,
        GroupSettings {
            pixel_preference: PixelPreference::Individual,
            max_power: 50,
            ..Default::default()
        },
    );

    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[2], 0, "Pixel 0 red");
    assert_eq!(buf[5], 32, "Pixel 1 red");
    assert_eq!(buf[8], 64, "Pixel 2 red");
    assert_eq!(buf[11], 95, "Pixel 3 red");
}

#[test]
fn test_pixel_preference_grouped_zones() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // Fixture with Dimmer (ch0), Shutter (ch1), and 4 RGB pixels (ch2..13)
    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Pixel Bar 4".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "Dimmer",
                "Shutter",
                "Red 1", "Green 1", "Blue 1",
                "Red 2", "Green 2", "Blue 2",
                "Red 3", "Green 3", "Blue 3",
                "Red 4", "Green 4", "Blue 4",
            ]
        }),
        universe: 1,
        address: 1,
    });

    // Group 1 settings: Grouped with 2 zones (pixels 0,1 in zone 0; pixels 2,3 in zone 1)
    show.groups.settings.insert(
        1,
        GroupSettings {
            pixel_preference: PixelPreference::Grouped,
            pixel_group_count: 2,
            ..Default::default()
        },
    );

    engine.set_group_color(1, Some("white".to_string()), Some(&show));
    // Set a ramp_up effect with left alignment and width 100
    engine.set_group_effect(
        1,
        Some("ramp_up".to_string()),
        Some("left".to_string()),
        false,
    );
    engine.set_group_effect_width(1, 100);
    engine.set_group_master(1, 100);

    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    // Zone 0 (steps 0/2 => 0): Pixel 0 and Pixel 1 must be 0
    assert_eq!(buf[2], 0, "Pixel 0 red");
    assert_eq!(buf[5], 0, "Pixel 1 red");

    // Zone 1 (steps 1/2 => 128): Pixel 2 and Pixel 3 must be 128
    assert_eq!(buf[8], 128, "Pixel 2 red");
    assert_eq!(buf[11], 128, "Pixel 3 red");

    // A max power limit halves the zone dimming
    show.groups.settings.insert(
        1,
        GroupSettings {
            pixel_preference: PixelPreference::Grouped,
            pixel_group_count: 2,
            max_power: 50,
            ..Default::default()
        },
    );

    engine.compute_dmx(&show);
    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[2], 0, "Pixel 0 red");
    assert_eq!(buf[5], 0, "Pixel 1 red");
    assert_eq!(buf[8], 64, "Pixel 2 red");
    assert_eq!(buf[11], 64, "Pixel 3 red");
}

#[test]
fn test_epix_strip_individual_global_random_chase() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    // 5-section RGB Epix strip (15 channels, 5 pixels, no dimmer channel)
    let epix_channels = vec![
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
        id: 401,
        groups: vec![4],
        name: "EPIX 401".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "epix_strip",
            "name": "Epix Strip IP",
            "manufacturerKey": "chauvet"
        }),
        mode: serde_json::json!({
            "name": "5 Section Mode",
            "channels": epix_channels.clone()
        }),
        universe: 1,
        address: 1,
    });

    show.patch.fixtures.push(PatchedFixture {
        id: 402,
        groups: vec![4],
        name: "EPIX 402".to_string(),
        fixture_type: serde_json::json!({
            "fixtureKey": "epix_strip",
            "name": "Epix Strip IP",
            "manufacturerKey": "chauvet"
        }),
        mode: serde_json::json!({
            "name": "5 Section Mode",
            "channels": epix_channels
        }),
        universe: 1,
        address: 16,
    });

    // Configure group 4 as Individual
    show.groups.settings.insert(
        4,
        GroupSettings {
            pixel_preference: PixelPreference::Individual,
            ..Default::default()
        },
    );

    engine.set_group_color(4, Some("white".to_string()), Some(&show));
    engine.set_group_effect(
        4,
        Some("ramp_up".to_string()),
        Some("left".to_string()),
        false,
    );
    engine.set_group_effect_width(4, 100);
    engine.set_group_master(4, 100);

    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    // 10 pixels total across 2 fixtures.
    // Fixture 401 pixels 0..4 (channels 0..14)
    // Fixture 402 pixels 0..4 (channels 15..29)
    // Check that each pixel has a different intensity!
    let p0 = buf[0];
    let p1 = buf[3];
    let p2 = buf[6];
    let p3 = buf[9];
    let p4 = buf[12];
    let p5 = buf[15];
    let p6 = buf[18];
    let p7 = buf[21];
    let p8 = buf[24];
    let p9 = buf[27];

    println!(
        "Pixels: {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
        p0, p1, p2, p3, p4, p5, p6, p7, p8, p9
    );

    assert_eq!(p0, 0, "P0");
    assert_eq!(p1, 26, "P1");
    assert_eq!(p2, 51, "P2");
    assert_eq!(p3, 77, "P3");
    assert_eq!(p4, 102, "P4");
    assert_eq!(p5, 128, "P5");
    assert_eq!(p6, 153, "P6");
    assert_eq!(p7, 179, "P7");
    assert_eq!(p8, 204, "P8");
    assert_eq!(p9, 230, "P9");
}

#[test]
fn test_epix_strip_individual_global_grandma2_shuffled_rng_phase() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 501,
        groups: vec![5],
        name: "Epix 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "Pixel 1_Red", "Pixel 1_Green", "Pixel 1_Blue",
                "Pixel 2_Red", "Pixel 2_Green", "Pixel 2_Blue",
                "Pixel 3_Red", "Pixel 3_Green", "Pixel 3_Blue",
                "Pixel 4_Red", "Pixel 4_Green", "Pixel 4_Blue",
                "Pixel 5_Red", "Pixel 5_Green", "Pixel 5_Blue",
            ]
        }),
        universe: 1,
        address: 1,
    });

    show.patch.fixtures.push(PatchedFixture {
        id: 502,
        groups: vec![5],
        name: "Epix 2".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": [
                "Pixel 1_Red", "Pixel 1_Green", "Pixel 1_Blue",
                "Pixel 2_Red", "Pixel 2_Green", "Pixel 2_Blue",
                "Pixel 3_Red", "Pixel 3_Green", "Pixel 3_Blue",
                "Pixel 4_Red", "Pixel 4_Green", "Pixel 4_Blue",
                "Pixel 5_Red", "Pixel 5_Green", "Pixel 5_Blue",
            ]
        }),
        universe: 1,
        address: 16,
    });

    show.groups.settings.insert(
        5,
        GroupSettings {
            pixel_preference: PixelPreference::Individual,
            ..Default::default()
        },
    );

    engine.set_group_color(5, Some("white".to_string()), Some(&show));
    engine.set_group_effect(
        5,
        Some("ramp_up".to_string()),
        Some("rng".to_string()),
        false,
    );
    engine.set_group_effect_width(5, 100);
    engine.set_group_master(5, 100);

    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    let intensities = vec![
        buf[0], buf[3], buf[6], buf[9], buf[12], buf[15], buf[18], buf[21], buf[24], buf[27],
    ];

    // grandMA2 shuffle guarantee: all 10 items get distinct evenly distributed phase slots
    let mut sorted = intensities.clone();
    sorted.sort();
    assert_eq!(
        sorted,
        vec![0, 26, 51, 77, 102, 128, 153, 179, 204, 230],
        "All 10 pixels across both fixtures must receive distinct unique phase slots"
    );

    // Verify it is truly shuffled (not trivial left-to-right)
    assert_ne!(
        intensities, sorted,
        "Shuffled order must not be identical to linear left-to-right"
    );
}
