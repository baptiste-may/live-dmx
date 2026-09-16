use live_dmx_lib::output::color_routing::{
    closest_color_position, effective_color_preference, map_temperature_to_dmx,
    rgb_to_color_temperature, ColorChannelSummary,
};
use live_dmx_lib::output::ChannelType;
use live_dmx_lib::show::ColorPreference;

fn summary(mix: bool, wheel: bool, macro_: bool, temp: bool) -> ColorChannelSummary {
    ColorChannelSummary {
        has_mix: mix,
        has_wheel: wheel,
        has_macro: macro_,
        has_temp: temp,
    }
}

fn channels(types: &[ChannelType]) -> ColorChannelSummary {
    ColorChannelSummary::from_channels(types)
}

#[test]
fn test_color_channel_summary_from_channels() {
    let s = channels(&[
        ChannelType::Red,
        ChannelType::Green,
        ChannelType::ColorWheel,
        ChannelType::ColorTemperature,
        ChannelType::Other,
    ]);
    assert!(s.has_mix);
    assert!(s.has_wheel);
    assert!(!s.has_macro);
    assert!(s.has_temp);

    let none = channels(&[ChannelType::Dimmer, ChannelType::Other]);
    assert_eq!(none, summary(false, false, false, false));
}

#[test]
fn test_effective_preference_mix_fallback_chain() {
    // Preferences fall back in a deterministic order when a class is missing.
    let wheel_only = summary(false, true, false, false);
    let mac_only = summary(false, false, true, false);
    let temp_only = summary(false, false, false, true);
    let empty = summary(false, false, false, false);

    assert_eq!(
        effective_color_preference(ColorPreference::ColorMix, wheel_only),
        ColorPreference::ColorWheel
    );
    assert_eq!(
        effective_color_preference(ColorPreference::ColorMix, mac_only),
        ColorPreference::ColorPresets
    );
    assert_eq!(
        effective_color_preference(ColorPreference::ColorMix, temp_only),
        ColorPreference::Cct
    );
    assert_eq!(
        effective_color_preference(ColorPreference::ColorMix, empty),
        ColorPreference::ColorMix
    );
}

#[test]
fn test_effective_preference_wheel_fallback_chain() {
    assert_eq!(
        effective_color_preference(
            ColorPreference::ColorWheel,
            summary(true, false, false, false)
        ),
        ColorPreference::ColorMix
    );
    assert_eq!(
        effective_color_preference(
            ColorPreference::ColorWheel,
            summary(false, false, true, false)
        ),
        ColorPreference::ColorPresets
    );
    assert_eq!(
        effective_color_preference(
            ColorPreference::ColorWheel,
            summary(false, false, false, true)
        ),
        ColorPreference::Cct
    );
    assert_eq!(
        effective_color_preference(
            ColorPreference::ColorWheel,
            summary(false, false, false, false)
        ),
        ColorPreference::ColorWheel
    );
}

#[test]
fn test_effective_preference_presets_and_cct() {
    assert_eq!(
        effective_color_preference(
            ColorPreference::ColorPresets,
            summary(false, true, false, false),
        ),
        ColorPreference::ColorWheel
    );
    assert_eq!(
        effective_color_preference(
            ColorPreference::ColorPresets,
            summary(false, false, false, true),
        ),
        ColorPreference::Cct
    );
    assert_eq!(
        effective_color_preference(ColorPreference::Cct, summary(false, true, false, false)),
        ColorPreference::ColorWheel
    );
}

#[test]
fn test_closest_color_position_no_capabilities_uses_fallback() {
    // Pure red → red fallback slot (idx 1): (1.5/8)*255 = 47.8 → 48.
    assert_eq!(closest_color_position(255, 0, 0, &[]), 48);
    // Pure green → slot idx 4: (4.5/8)*255 = 143.4 → 143.
    assert_eq!(closest_color_position(0, 255, 0, &[]), 143);
    // Pure blue → slot idx 6: (6.5/8)*255 = 207.2 → 207.
    assert_eq!(closest_color_position(0, 0, 255, &[]), 207);
}

#[test]
fn test_closest_color_position_with_capabilities() {
    let capabilities = vec![
        serde_json::json!({
            "dmxFrom": 0,
            "dmxTo": 10,
            "color": "#ff0000"
        }),
        serde_json::json!({
            "dmxFrom": 85,
            "dmxTo": 95,
            "color": "#0000ff"
        }),
        serde_json::json!({
            "dmxFrom": 170,
            "dmxTo": 180,
            "colors": ["#00ff00", "#ffff00"]
        }),
    ];

    // Red request should land on the first capability slot (midpoint 5).
    assert_eq!(closest_color_position(255, 0, 0, &capabilities), 5);
    // Blue request → second slot midpoint 90.
    assert_eq!(closest_color_position(0, 0, 255, &capabilities), 90);
    // Green request → third capability (midpoint 175).
    assert_eq!(closest_color_position(0, 255, 0, &capabilities), 175);
    // Yellow request → closer to third cap (#ffff00) than red/blue.
    assert_eq!(closest_color_position(255, 255, 0, &capabilities), 175);
}

#[test]
fn test_closest_color_position_rejects_malformed_color_data() {
    // Non-ASCII "hex" strings must not panic and should fall back gracefully.
    let capabilities = vec![serde_json::json!({
        "dmxFrom": 0,
        "dmxTo": 10,
        "color": "\u{e9}\u{e9}\u{e9}"
    })];
    // Must not panic; malformed capability is skipped and the fallback (red slot) is used.
    assert_eq!(closest_color_position(255, 0, 0, &capabilities), 48);

    // Capabilities without a usable dmx range are skipped (fallback used → red slot = 48).
    let no_range = vec![serde_json::json!({"color": "#ff0000"})];
    assert_eq!(closest_color_position(255, 0, 0, &no_range), 48);
}

#[test]
fn test_rgb_to_color_temperature_known_value() {
    // Daylight-ish white ~ around 5500K (loose bound).
    let cct = rgb_to_color_temperature(255, 250, 240);
    assert!((5000.0..6500.0).contains(&cct), "got {cct}");

    // Warm orange pushes to the low end.
    let warm = rgb_to_color_temperature(255, 180, 80);
    assert!(warm < 4500.0, "got {warm}");
}

#[test]
fn test_rgb_to_color_temperature_clamped() {
    // Pure black should not crash; returns the 3200K fallback.
    assert_eq!(rgb_to_color_temperature(0, 0, 0), 3200.0);
    // Extreme blue/pure colors are clamped into the 2000..10000 range.
    let blue = rgb_to_color_temperature(0, 0, 255);
    assert!(blue >= 2000.0 && blue <= 10000.0, "got {blue}");
}

#[test]
fn test_map_temperature_to_dmx_bounds() {
    assert_eq!(map_temperature_to_dmx(2000.0), 0);
    assert_eq!(map_temperature_to_dmx(10000.0), 255);
    assert_eq!(map_temperature_to_dmx(0.0), 0);
    assert_eq!(map_temperature_to_dmx(50000.0), 255);

    // Midpoint: (6000-2000)/8000*255 = 127.5 → rounds to 128.
    assert_eq!(map_temperature_to_dmx(6000.0), 128);
}
