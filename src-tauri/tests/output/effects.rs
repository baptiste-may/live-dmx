use live_dmx_lib::output::{
    calculate_effect_factor, calculate_effect_intensity, compute_random_strobe_active,
    GroupEffectState, LiveEngine,
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
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue", "White"]}),
        universe: 1,
        address: 1,
    });

    // Trigger White Flash for Group 1
    engine.set_flash(1, "master".to_string(), true);
    assert!(engine.is_flash_active(1, "master"));

    engine.compute_dmx(&show);
    let buf_flash = engine.universe_buffers.get(&1).unwrap().clone();
    for i in 0..5 {
        assert_eq!(buf_flash[i], 255);
    }

    // Release White Flash
    engine.set_flash(1, "master".to_string(), false);
    assert!(!engine.is_flash_active(1, "master"));
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
}

#[test]
fn test_calculate_effect_intensity_types() {
    let state_none = GroupEffectState {
        effect: None,
        alignment: None,
        bounce: false,
    };
    assert_eq!(
        calculate_effect_intensity(&state_none, 0.0, 0, 1, 1, 120),
        0
    );

    let state_off = GroupEffectState {
        effect: Some("off".to_string()),
        alignment: None,
        bounce: false,
    };
    assert_eq!(calculate_effect_intensity(&state_off, 0.0, 0, 1, 1, 120), 0);

    let state_full = GroupEffectState {
        effect: Some("full".to_string()),
        alignment: None,
        bounce: false,
    };
    assert_eq!(
        calculate_effect_intensity(&state_full, 0.0, 0, 1, 1, 120),
        255
    );

    let state_chase = GroupEffectState {
        effect: Some("chase".to_string()),
        alignment: Some("left".to_string()),
        bounce: false,
    };
    let intensity = calculate_effect_intensity(&state_chase, 0.25, 0, 4, 1, 60);
    assert!(intensity == 0 || intensity == 255);

    let state_ramp_up = GroupEffectState {
        effect: Some("ramp_up".to_string()),
        alignment: None,
        bounce: false,
    };
    let intensity_ramp = calculate_effect_intensity(&state_ramp_up, 0.5, 0, 1, 1, 60);
    assert!(intensity_ramp > 0);

    let state_ramp_down = GroupEffectState {
        effect: Some("ramp_down".to_string()),
        alignment: None,
        bounce: false,
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
    };
    let factor = calculate_effect_factor(&state_chase, 0.25, 0, 1, 1, 60);
    assert!(factor == 0.0 || factor == 1.0);

    let state_pulse = GroupEffectState {
        effect: Some("pulse".to_string()),
        alignment: None,
        bounce: false,
    };
    let factor_pulse = calculate_effect_factor(&state_pulse, 0.25, 0, 1, 1, 60);
    assert!((0.0..=1.0).contains(&factor_pulse));
}

#[test]
fn test_compute_random_strobe_active() {
    let _active = compute_random_strobe_active(1, 0.5, 50);
}
