use live_dmx_lib::output::{
    calculate_effect_factor, calculate_effect_intensity, calculate_effective_fract,
    compute_random_strobe_active, GroupEffectState, LiveEngine,
};
use live_dmx_lib::show::{PatchedFixture, ShowData};

#[test]
fn test_live_engine_white_flash_calculation() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Test Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue", "White", "Pan", "Tilt"]}),
        universe: 1,
        address: 1,
    });

    // Trigger White Flash for Group 1
    engine.set_flash(1, "master".to_string(), true);
    assert!(engine.is_flash_active(1, "master"));

    engine.compute_dmx(&show);
    let buf_flash = engine.universe_buffers.get(&1).unwrap().clone();
    for val in buf_flash.iter().take(5) {
        assert_eq!(*val, 255);
    }
    assert_eq!(buf_flash[5], 128); // Pan defaults to center (128)
    assert_eq!(buf_flash[6], 128); // Tilt defaults to center (128)

    // Release White Flash
    engine.set_flash(1, "master".to_string(), false);
    assert!(!engine.is_flash_active(1, "master"));
}

#[test]
fn test_live_engine_strobe_keeps_amber_uv_channels() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "RGBWA+UV Par".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": ["Dimmer", "Red", "Green", "Blue", "White", "Amber", "UV"]
        }),
        universe: 1,
        address: 1,
    });

    engine.strobe_rate = 50;
    // Amber color (hex) → r=255, g=170, b=0, which matches the Amber heuristic.
    engine.set_group_color(1, Some("#ffaa00".to_string()), Some(&show));
    engine.set_flash(1, "strobe".to_string(), true);

    let mut amber = Vec::new();
    for _ in 0..15 {
        engine.compute_dmx(&show);
        let buf = engine.universe_buffers.get(&1).unwrap();
        // Amber must track the strobe phase exactly like the Dimmer channel.
        assert_eq!(buf[5], buf[0]);
        assert_eq!(buf[6], 0); // UV stays 0 for an amber color
        amber.push(buf[5]);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    // ~75ms window at freq 10.5 covers both strobe phases.
    assert!(amber.contains(&255), "amber strobe phase never reached 255");
    assert!(amber.contains(&0), "amber strobe phase never reached 0");

    // Strobe must be released cleanly.
    engine.set_flash(1, "strobe".to_string(), false);
    assert!(!engine.is_flash_active(1, "strobe"));
}

#[test]
fn test_live_engine_simple_flash_calculation() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Test Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue", "White", "Pan", "Tilt"]}),
        universe: 1,
        address: 1,
    });

    // Set group 1 color to red
    engine.set_group_color(1, Some("red".to_string()), Some(&show));
    // Set group master to 0 so dimmer is normally 0
    engine.set_group_master(1, 0);

    engine.compute_dmx(&show);
    let buf_normal = engine.universe_buffers.get(&1).unwrap().clone();
    assert_eq!(buf_normal[0], 0); // Dimmer is 0
    assert_eq!(buf_normal[1], 255); // Red is 255
    assert_eq!(buf_normal[2], 0); // Green is 0
    assert_eq!(buf_normal[3], 0); // Blue is 0

    // Trigger Simple Flash
    engine.set_flash(1, "simple".to_string(), true);
    assert!(engine.is_flash_active(1, "simple"));

    engine.compute_dmx(&show);
    let buf_flash = engine.universe_buffers.get(&1).unwrap().clone();
    assert_eq!(buf_flash[0], 255); // Dimmer flashed to 255
    assert_eq!(buf_flash[1], 255); // Red stays 255
    assert_eq!(buf_flash[2], 0); // Green stays 0 (NOT white!)
    assert_eq!(buf_flash[3], 0); // Blue stays 0 (NOT white!)

    // Release Simple Flash
    engine.set_flash(1, "simple".to_string(), false);
    assert!(!engine.is_flash_active(1, "simple"));
}

#[test]
fn test_live_engine_flash_fade() {
    let mut engine = LiveEngine::new();

    engine.set_simple_flash_fade(1.5);
    engine.set_white_flash_fade(2.0);

    assert_eq!(engine.simple_flash_fade, 1.5);
    assert_eq!(engine.white_flash_fade, 2.0);

    engine.set_flash(1, "simple".to_string(), true);
    assert_eq!(engine.get_flash_factor(1, "simple"), 1.0);

    engine.set_flash(1, "simple".to_string(), false);
    // Right after release, transition should exist and factor should be > 0.0
    let factor = engine.get_flash_factor(1, "simple");
    assert!(factor > 0.8 && factor <= 1.0);

    // After reset, transitions and fade settings should clear
    engine.reset_all();
    assert_eq!(engine.simple_flash_fade, 0.0);
    assert_eq!(engine.white_flash_fade, 0.0);
    assert_eq!(engine.get_flash_factor(1, "simple"), 0.0);
}

#[test]
fn test_live_engine_effect_bpm() {
    let mut engine = LiveEngine::new();
    assert_eq!(engine.effect_bpm, 60);

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

    engine.set_group_effect(1, Some("pulse".to_string()), None, false);

    // With 0 BPM, effect should produce 0 intensity
    engine.effect_bpm = 0;
    engine.compute_dmx(&show);
    let buf_zero = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf_zero[0], 0);

    // With 60 BPM, effect computes with 60 BPM rate
    engine.effect_bpm = 60;
    engine.compute_dmx(&show);
    // Resetting all sets bpm back to default 60
    engine.reset_all();
    assert_eq!(engine.effect_bpm, 60);
    assert_eq!(engine.get_group_effect(1).width, 100);
}

#[test]
fn test_live_engine_effect_width() {
    let mut engine = LiveEngine::new();
    assert_eq!(engine.get_group_effect(1).width, 100);
    assert_eq!(engine.get_group_color_effect(1).width, 100);

    engine.set_group_effect_width(1, 50);
    assert_eq!(engine.get_group_effect(1).width, 50);

    engine.set_group_color_effect_width(1, 75);
    assert_eq!(engine.get_group_color_effect(1).width, 75);

    // Cannot exceed 100
    engine.set_group_effect_width(1, 150);
    assert_eq!(engine.get_group_effect(1).width, 100);

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

    engine.set_group_effect(1, Some("pulse".to_string()), None, false);

    // With 0 width, effect produces 0 intensity
    engine.set_group_effect_width(1, 0);
    engine.compute_dmx(&show);
    let buf_zero = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf_zero[0], 0);

    engine.reset_all();
    assert_eq!(engine.get_group_effect(1).width, 100);
    assert_eq!(engine.get_group_color_effect(1).width, 100);
}

#[test]
fn test_calculate_effect_intensity_types() {
    let state_none = GroupEffectState {
        effect: None,
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    assert_eq!(
        calculate_effect_intensity(&state_none, 0.0, 0, 1, 1, 120),
        0
    );

    let state_off = GroupEffectState {
        effect: Some("off".to_string()),
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    assert_eq!(calculate_effect_intensity(&state_off, 0.0, 0, 1, 1, 120), 0);

    let state_full = GroupEffectState {
        effect: Some("full".to_string()),
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    assert_eq!(
        calculate_effect_intensity(&state_full, 0.0, 0, 1, 1, 120),
        255
    );

    let state_chase = GroupEffectState {
        effect: Some("chase".to_string()),
        alignment: Some("left".to_string()),
        bounce: false,
        ..Default::default()
    };
    let intensity = calculate_effect_intensity(&state_chase, 0.25, 0, 4, 1, 60);
    assert!(intensity == 0 || intensity == 255);

    let state_ramp_up = GroupEffectState {
        effect: Some("ramp_up".to_string()),
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    let intensity_ramp = calculate_effect_intensity(&state_ramp_up, 0.5, 0, 1, 1, 60);
    assert!(intensity_ramp > 0);

    let state_ramp_down = GroupEffectState {
        effect: Some("ramp_down".to_string()),
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    let intensity_down = calculate_effect_intensity(&state_ramp_down, 0.5, 0, 1, 1, 60);
    assert!(intensity_down > 0);
}

#[test]
fn test_calculate_effect_factor() {
    let state_chase = GroupEffectState {
        effect: Some("chase".to_string()),
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    let factor = calculate_effect_factor(&state_chase, 0.25, 0, 1, 1, 60);
    assert!(factor == 0.0 || factor == 1.0);

    let state_pulse = GroupEffectState {
        effect: Some("pulse".to_string()),
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    let factor_pulse = calculate_effect_factor(&state_pulse, 0.25, 0, 1, 1, 60);
    assert!((0.0..=1.0).contains(&factor_pulse));
}

#[test]
fn test_calculate_effect_factor_width() {
    let state_pulse = GroupEffectState {
        effect: Some("pulse".to_string()),
        alignment: None,
        bounce: false,
        width: 50,
        ..Default::default()
    };
    // At elapsed 0.75 with 50% width, fixture is in inactive portion (> 0.5) -> 0.0
    let factor_50_late = calculate_effect_factor(&state_pulse, 0.75, 0, 1, 1, 60);
    assert_eq!(factor_50_late, 0.0);

    // With 0% width, factor is 0.0
    let state_pulse_0 = GroupEffectState {
        effect: Some("pulse".to_string()),
        alignment: None,
        bounce: false,
        width: 0,
        ..Default::default()
    };
    let factor_0 = calculate_effect_factor(&state_pulse_0, 0.25, 0, 1, 1, 60);
    assert_eq!(factor_0, 0.0);
}

#[test]
fn test_compute_random_strobe_active() {
    let _active = compute_random_strobe_active(1, 0.5, 50);
}

#[test]
fn test_calculate_effect_factor_sine() {
    let state_sine = GroupEffectState {
        effect: Some("sine".to_string()),
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    // At elapsed 0.0 with 60 BPM (cycle 1.0s), t = 0.0, sin(0) = 0, factor = 0.5
    let factor_sine = calculate_effect_factor(&state_sine, 0.0, 0, 1, 1, 60);
    assert!((factor_sine - 0.5).abs() < 0.01);

    // At elapsed 0.25 with 60 BPM, t = PI/2, sin(PI/2) = 1.0, factor = 1.0
    let factor_sine_peak = calculate_effect_factor(&state_sine, 0.25, 0, 1, 1, 60);
    assert!((factor_sine_peak - 1.0).abs() < 0.01);
}

#[test]
fn test_center_alignment_and_calculate_effective_fract() {
    let state_center = GroupEffectState {
        effect: Some("pulse".to_string()),
        alignment: Some("center".to_string()),
        bounce: false,
        ..Default::default()
    };

    // 3 fixtures: i=0 (left edge), i=1 (center), i=2 (right edge)
    let fract_left = calculate_effective_fract(&state_center, 0.0, 0, 3, 1, 60).unwrap();
    let fract_center = calculate_effective_fract(&state_center, 0.0, 1, 3, 2, 60).unwrap();
    let fract_right = calculate_effective_fract(&state_center, 0.0, 2, 3, 3, 60).unwrap();

    // Symmetrical outer fixtures should have equal phase offsets
    assert!((fract_left - fract_right).abs() < 0.01);
    // Center fixture has a distinct phase from the edges
    assert!((fract_center - fract_left).abs() > 0.1);

    // If BPM is 0, calculate_effective_fract returns None
    assert!(calculate_effective_fract(&state_center, 0.0, 0, 3, 1, 0).is_none());

    // If effect is None or off, returns None
    let state_none = GroupEffectState {
        effect: None,
        alignment: None,
        bounce: false,
        ..Default::default()
    };
    assert!(calculate_effective_fract(&state_none, 0.0, 0, 3, 1, 60).is_none());
}

#[test]
fn test_generate_shuffled_steps_permutation() {
    let count = 16;
    let steps = live_dmx_lib::output::generate_shuffled_steps(count, 0x12345678);
    assert_eq!(steps.len(), count);

    // Verify all numbers from 0..count appear exactly once (perfect permutation)
    let mut sorted = steps.clone();
    sorted.sort();
    let expected: Vec<usize> = (0..count).collect();
    assert_eq!(
        sorted, expected,
        "Permutation must contain each element 0..count exactly once"
    );

    // Verify different seeds give different permutations
    let steps_other = live_dmx_lib::output::generate_shuffled_steps(count, 0x87654321);
    assert_ne!(steps, steps_other);
}

#[test]
fn test_is_too_structured_rejects_left_right_wing_center() {
    use live_dmx_lib::output::is_too_structured;

    // Left (strictly ascending)
    assert!(is_too_structured(&[0, 1, 2]));
    assert!(is_too_structured(&[0, 1, 2, 3, 4]));

    // Right (strictly descending)
    assert!(is_too_structured(&[2, 1, 0]));
    assert!(is_too_structured(&[4, 3, 2, 1, 0]));

    // Cyclic left / right sweeps
    assert!(is_too_structured(&[1, 2, 3, 0]));
    assert!(is_too_structured(&[3, 0, 1, 2]));
    assert!(is_too_structured(&[2, 1, 0, 3]));

    // V-shape (wing) and inverted V-shape (center)
    assert!(is_too_structured(&[3, 1, 0, 2]));
    assert!(is_too_structured(&[0, 2, 3, 1]));

    // Interleaved outside-in
    assert!(is_too_structured(&[0, 3, 1, 2]));

    // Truly chaotic permutations should NOT be rejected
    assert!(!is_too_structured(&[0, 2, 1]));
    assert!(!is_too_structured(&[1, 3, 0, 2]));
    assert!(!is_too_structured(&[2, 0, 3, 1]));
    assert!(!is_too_structured(&[3, 0, 2, 4, 1]));
}

#[test]
fn test_generate_shuffled_steps_forced_random() {
    use live_dmx_lib::output::{generate_shuffled_steps, is_too_structured};

    // Test across various fixture / pixel rig sizes and 50 different seeds
    for count in [3, 4, 5, 8, 10, 16, 50] {
        for seed_idx in 0..50 {
            let seed = (seed_idx as u64)
                .wrapping_mul(0x9e3779b97f4a7c15)
                .wrapping_add(0x517cc1b727220a95);
            let steps = generate_shuffled_steps(count, seed);

            // 1. Must be a complete permutation of 0..count
            let mut sorted = steps.clone();
            sorted.sort();
            let expected: Vec<usize> = (0..count).collect();
            assert_eq!(sorted, expected);

            // 2. Must NEVER be structured (no accidental left, right, wing, center)
            assert!(
                !is_too_structured(&steps),
                "Permutation {:?} for count {} with seed 0x{:x} was too structured",
                steps,
                count,
                seed
            );

            // 3. Must never be pure left (0..count) or right ((0..count).rev())
            assert_ne!(steps, expected, "Permutation was identical to left sweep");
            let rev_expected: Vec<usize> = (0..count).rev().collect();
            assert_ne!(
                steps, rev_expected,
                "Permutation was identical to right sweep"
            );
        }
    }
}
