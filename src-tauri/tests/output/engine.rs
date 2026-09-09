use live_dmx_lib::output::LiveEngine;
use live_dmx_lib::show::{PatchedFixture, ShowData};

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
