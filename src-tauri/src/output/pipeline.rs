use crate::output::profile::{ChannelResolution, CompiledFixtureProfile, LogicalAttribute};
use crate::show::ColorPreference;

/// Represents the normalized, fully evaluated state of a fixture for a single rendering frame.
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalFixtureState {
    /// Normalized master intensity: 0.0 .. 1.0 (effects * masters)
    pub intensity: f32,
    /// Normalized color in RGB: 0.0 .. 1.0
    pub color_rgb: (f32, f32, f32),
    /// Normalized white emitter accent: 0.0 .. 1.0
    pub white_accent: f32,
    /// Flash dimmer boost factor: 0.0 .. 1.0
    pub flash_dimmer_factor: f32,
    /// White flash boost factor: 0.0 .. 1.0
    pub white_flash_factor: f32,
    /// Whether the fixture is participating in active strobe
    pub is_in_strobe: bool,
    /// Instantaneous strobe gate value (0 or 255)
    pub strobe_val: u8,
    /// Color wheel precomputed slot DMX value if routed
    pub color_wheel_dmx: Option<u8>,
    /// Color macro / preset DMX value if routed
    pub color_macro_dmx: Option<u8>,
    /// Color temperature DMX value if routed
    pub color_temp_dmx: Option<u8>,
    /// Optional normalized Pan position override: 0.0 .. 1.0 (None preserves profile default)
    pub pan_norm: Option<f32>,
    /// Optional normalized Tilt position override: 0.0 .. 1.0 (None preserves profile default)
    pub tilt_norm: Option<f32>,
}

impl Default for LogicalFixtureState {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            color_rgb: (1.0, 1.0, 1.0),
            white_accent: 0.0,
            flash_dimmer_factor: 0.0,
            white_flash_factor: 0.0,
            is_in_strobe: false,
            strobe_val: 0,
            color_wheel_dmx: None,
            color_macro_dmx: None,
            color_temp_dmx: None,
            pan_norm: None,
            tilt_norm: None,
        }
    }
}

/// Pure DMX encoding function: translates `LogicalFixtureState` into DMX bytes
/// according to the fixture's compiled profile.
pub fn encode_fixture_dmx(
    profile: &CompiledFixtureProfile,
    state: &LogicalFixtureState,
    pixel_states: Option<&[LogicalFixtureState]>,
    effective_color_preference: ColorPreference,
    buf: &mut [u8],
) {
    if buf.is_empty() || profile.footprint == 0 {
        return;
    }

    let footprint = profile.footprint.min(buf.len());

    // 1. Initialize entire fixture slice with compiled defaults.
    // This immediately satisfies all Gobos, Zoom, Focus, Prism, Iris, and Control channels!
    buf[..footprint].copy_from_slice(&profile.default_dmx[..footprint]);

    // 2. Check if pixel individual state evaluation is active
    let has_pixel_states = pixel_states.is_some_and(|ps| !ps.is_empty());

    // Compute effective dimmer level for master state
    let master_effective_dimmer = if state.flash_dimmer_factor > 0.0 {
        state.intensity + (1.0 - state.intensity) * state.flash_dimmer_factor
    } else if state.is_in_strobe {
        if state.strobe_val > 0 {
            1.0
        } else {
            0.0
        }
    } else {
        state.intensity
    }
    .clamp(0.0, 1.0);

    let max_pixel_effective_dimmer = if has_pixel_states {
        pixel_states
            .map(|ps| {
                ps.iter()
                    .map(|p_state| {
                        if p_state.flash_dimmer_factor > 0.0 {
                            p_state.intensity
                                + (1.0 - p_state.intensity) * p_state.flash_dimmer_factor
                        } else if p_state.is_in_strobe {
                            if p_state.strobe_val > 0 {
                                1.0
                            } else {
                                0.0
                            }
                        } else {
                            p_state.intensity
                        }
                        .clamp(0.0, 1.0)
                    })
                    .fold(0.0, f32::max)
            })
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let effective_shutter_dimmer = if has_pixel_states {
        master_effective_dimmer.max(max_pixel_effective_dimmer)
    } else {
        master_effective_dimmer
    };

    // 3. Compute effective color mixing components
    let is_mix_routing = effective_color_preference == ColorPreference::ColorMix;

    // 4. Encode channels
    for ch in &profile.channels {
        match ch.attribute {
            LogicalAttribute::BeamDimmer | LogicalAttribute::BeamWhite => match ch.resolution {
                ChannelResolution::Bits8 { offset } => {
                    if offset < buf.len() {
                        buf[offset] = 0;
                    }
                }
                ChannelResolution::Bits16 { coarse, fine } => {
                    if coarse < buf.len() {
                        buf[coarse] = 0;
                    }
                    if fine < buf.len() {
                        buf[fine] = 0;
                    }
                }
            },
            LogicalAttribute::Dimmer => {
                let dim_level = if let Some(p) = ch.pixel_index {
                    let p_state = pixel_states.and_then(|ps| ps.get(p)).unwrap_or(state);
                    if p_state.flash_dimmer_factor > 0.0 {
                        p_state.intensity + (1.0 - p_state.intensity) * p_state.flash_dimmer_factor
                    } else if p_state.is_in_strobe {
                        if p_state.strobe_val > 0 {
                            1.0
                        } else {
                            0.0
                        }
                    } else {
                        p_state.intensity
                    }
                    .clamp(0.0, 1.0)
                } else if has_pixel_states {
                    // Master dimmer with pixel individual control:
                    // If any pixel is active, flash, or strobe, master dimmer is fully open (1.0)
                    if max_pixel_effective_dimmer > 0.0
                        || state.flash_dimmer_factor > 0.0
                        || (state.is_in_strobe && state.strobe_val > 0)
                    {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    master_effective_dimmer
                };

                match ch.resolution {
                    ChannelResolution::Bits8 { offset } => {
                        if offset < buf.len() {
                            buf[offset] = (dim_level * 255.0).round() as u8;
                        }
                    }
                    ChannelResolution::Bits16 { coarse, fine } => {
                        let dmx16 = (dim_level * 65535.0).round() as u16;
                        if coarse < buf.len() {
                            buf[coarse] = (dmx16 >> 8) as u8;
                        }
                        if fine < buf.len() {
                            buf[fine] = (dmx16 & 0xFF) as u8;
                        }
                    }
                }
            }
            LogicalAttribute::Shutter => {
                let offset = ch.resolution.primary_offset();
                if offset < buf.len() {
                    let shutter_open = ch.shutter_open_dmx.unwrap_or(255);
                    if state.is_in_strobe {
                        buf[offset] = if state.strobe_val > 0 {
                            shutter_open
                        } else {
                            0
                        };
                    } else if effective_shutter_dimmer > 0.0 || state.flash_dimmer_factor > 0.0 {
                        buf[offset] = shutter_open;
                    } else {
                        buf[offset] = ch.default_dmx;
                    }
                }
            }
            LogicalAttribute::Red
            | LogicalAttribute::Green
            | LogicalAttribute::Blue
            | LogicalAttribute::White
            | LogicalAttribute::Amber
            | LogicalAttribute::Lime
            | LogicalAttribute::Cyan
            | LogicalAttribute::Magenta
            | LogicalAttribute::Yellow
            | LogicalAttribute::UV => {
                if !is_mix_routing {
                    match ch.resolution {
                        ChannelResolution::Bits8 { offset } => {
                            if offset < buf.len() {
                                buf[offset] = 0;
                            }
                        }
                        ChannelResolution::Bits16 { coarse, fine } => {
                            if coarse < buf.len() {
                                buf[coarse] = 0;
                            }
                            if fine < buf.len() {
                                buf[fine] = 0;
                            }
                        }
                    }
                    continue;
                }

                let (ch_state, pixel_has_own_dimmer) = match ch.pixel_index {
                    Some(p) => {
                        let ps = pixel_states.and_then(|list| list.get(p)).unwrap_or(state);
                        let has_cell_dim = profile.pixels.get(p).is_some_and(|pix| {
                            pix.channel_indices.iter().any(|&c_idx| {
                                profile
                                    .channels
                                    .get(c_idx)
                                    .is_some_and(|c| c.attribute == LogicalAttribute::Dimmer)
                            })
                        });
                        (ps, has_cell_dim)
                    }
                    None => (state, profile.has_dimmer),
                };

                let ch_effective_dimmer = if ch_state.flash_dimmer_factor > 0.0 {
                    ch_state.intensity + (1.0 - ch_state.intensity) * ch_state.flash_dimmer_factor
                } else if ch_state.is_in_strobe {
                    if ch_state.strobe_val > 0 {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    ch_state.intensity
                }
                .clamp(0.0, 1.0);

                let color_gate = if ch_state.is_in_strobe {
                    if ch_state.strobe_val > 0 {
                        1.0
                    } else {
                        0.0
                    }
                } else if has_pixel_states {
                    if pixel_has_own_dimmer {
                        1.0
                    } else {
                        ch_effective_dimmer
                    }
                } else if !profile.has_dimmer {
                    ch_effective_dimmer
                } else {
                    1.0
                };

                let (mut r, mut g, mut b) = ch_state.color_rgb;
                if ch_state.white_flash_factor > 0.0 {
                    r += (1.0 - r) * ch_state.white_flash_factor;
                    g += (1.0 - g) * ch_state.white_flash_factor;
                    b += (1.0 - b) * ch_state.white_flash_factor;
                }

                let raw_val = match ch.attribute {
                    LogicalAttribute::Red => r,
                    LogicalAttribute::Green => g,
                    LogicalAttribute::Blue => b,
                    LogicalAttribute::White => ch_state.white_accent,
                    LogicalAttribute::Amber => {
                        if r > 0.78 && g > 0.39 && b < 0.20 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    LogicalAttribute::Lime => 0.0,
                    LogicalAttribute::Cyan => {
                        if profile.is_cmy {
                            1.0 - r
                        } else if g > 0.78 && b > 0.78 && r < 0.20 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    LogicalAttribute::Magenta => {
                        if profile.is_cmy {
                            1.0 - g
                        } else if r > 0.78 && b > 0.78 && g < 0.20 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    LogicalAttribute::Yellow => {
                        if profile.is_cmy {
                            1.0 - b
                        } else if r > 0.78 && b > 0.78 && b < 0.20 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    LogicalAttribute::UV => ch_state.white_flash_factor.max(0.0),
                    _ => 0.0,
                };
                let gated = (raw_val * color_gate).clamp(0.0, 1.0);
                match ch.resolution {
                    ChannelResolution::Bits8 { offset } => {
                        if offset < buf.len() {
                            buf[offset] = (gated * 255.0).round() as u8;
                        }
                    }
                    ChannelResolution::Bits16 { coarse, fine } => {
                        let dmx16 = (gated * 65535.0).round() as u16;
                        if coarse < buf.len() {
                            buf[coarse] = (dmx16 >> 8) as u8;
                        }
                        if fine < buf.len() {
                            buf[fine] = (dmx16 & 0xFF) as u8;
                        }
                    }
                }
            }
            LogicalAttribute::ColorWheel => {
                let offset = ch.resolution.primary_offset();
                if offset < buf.len() {
                    buf[offset] = state.color_wheel_dmx.unwrap_or(0);
                }
            }
            LogicalAttribute::ColorMacro => {
                let offset = ch.resolution.primary_offset();
                if offset < buf.len() {
                    buf[offset] = state.color_macro_dmx.unwrap_or(0);
                }
            }
            LogicalAttribute::ColorTemperature => {
                let offset = ch.resolution.primary_offset();
                if offset < buf.len() {
                    buf[offset] = state.color_temp_dmx.unwrap_or(0);
                }
            }
            LogicalAttribute::Pan => {
                if let Some(norm) = state.pan_norm {
                    match ch.resolution {
                        ChannelResolution::Bits8 { offset } => {
                            if offset < buf.len() {
                                buf[offset] = (norm * 255.0).round() as u8;
                            }
                        }
                        ChannelResolution::Bits16 { coarse, fine } => {
                            let val16 = (norm * 65535.0).round() as u16;
                            if coarse < buf.len() {
                                buf[coarse] = (val16 >> 8) as u8;
                            }
                            if fine < buf.len() {
                                buf[fine] = (val16 & 0xFF) as u8;
                            }
                        }
                    }
                }
            }
            LogicalAttribute::Tilt => {
                if let Some(norm) = state.tilt_norm {
                    match ch.resolution {
                        ChannelResolution::Bits8 { offset } => {
                            if offset < buf.len() {
                                buf[offset] = (norm * 255.0).round() as u8;
                            }
                        }
                        ChannelResolution::Bits16 { coarse, fine } => {
                            let val16 = (norm * 65535.0).round() as u16;
                            if coarse < buf.len() {
                                buf[coarse] = (val16 >> 8) as u8;
                            }
                            if fine < buf.len() {
                                buf[fine] = (val16 & 0xFF) as u8;
                            }
                        }
                    }
                }
            }
            LogicalAttribute::PositionXYZ => {
                let offset = ch.resolution.primary_offset();
                if offset < buf.len() {
                    buf[offset] = 255;
                }
            }
            LogicalAttribute::Other => {
                let offset = ch.resolution.primary_offset();
                if offset < buf.len() && offset == 0 && profile.treat_first_as_dimmer {
                    buf[offset] = (master_effective_dimmer * 255.0).round() as u8;
                }
            }
            _ => {
                // Gobo, Prism, Zoom, Focus, Control already populated by default_dmx
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::profile::compile_fixture_profile;

    #[test]
    fn test_encode_fixture_dimmer_16bit() {
        let channel_names = vec![
            "Dimmer".to_string(),
            "Dimmer Fine".to_string(),
            "Shutter".to_string(),
        ];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);

        let mut buf = vec![0u8; 3];
        let state = LogicalFixtureState {
            intensity: 0.5,
            ..Default::default()
        };

        encode_fixture_dmx(&profile, &state, None, ColorPreference::ColorMix, &mut buf);

        // 0.5 * 65535 = 32768 => coarse = 128, fine = 0
        assert_eq!(buf[0], 128);
        assert_eq!(buf[1], 0);
        // Shutter open
        assert_eq!(buf[2], 255);
    }

    #[test]
    fn test_encode_fixture_shutter_open_and_closed() {
        let channel_names = vec![
            "Dimmer".to_string(),
            "Shutter".to_string(),
            "Red".to_string(),
        ];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);

        let mut buf = vec![0u8; 3];
        let mut state = LogicalFixtureState {
            intensity: 0.0,
            ..Default::default()
        };

        // 1. Intensity 0: shutter should be closed (0)
        encode_fixture_dmx(&profile, &state, None, ColorPreference::ColorMix, &mut buf);
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 0);

        // 2. Intensity > 0: shutter should be open (255)
        state.intensity = 0.8;
        encode_fixture_dmx(&profile, &state, None, ColorPreference::ColorMix, &mut buf);
        assert_eq!(buf[0], 204);
        assert_eq!(buf[1], 255);
    }

    #[test]
    fn test_encode_fixture_strobe() {
        let channel_names = vec!["Dimmer".to_string(), "Shutter".to_string()];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);

        let mut buf = vec![0u8; 2];
        let mut state = LogicalFixtureState {
            is_in_strobe: true,
            strobe_val: 255,
            ..Default::default()
        };

        // Phase high (strobe_val = 255)
        encode_fixture_dmx(&profile, &state, None, ColorPreference::ColorMix, &mut buf);
        assert_eq!(buf[0], 255);
        assert_eq!(buf[1], 255);

        // Phase low (strobe_val = 0)
        state.strobe_val = 0;
        encode_fixture_dmx(&profile, &state, None, ColorPreference::ColorMix, &mut buf);
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 0);
    }

    #[test]
    fn test_encode_fixture_cmy_subtractive() {
        let channel_names = vec![
            "Dimmer".to_string(),
            "Cyan".to_string(),
            "Magenta".to_string(),
            "Yellow".to_string(),
        ];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);

        let mut buf = vec![0u8; 4];
        let mut state = LogicalFixtureState {
            intensity: 1.0,
            color_rgb: (1.0, 0.0, 0.0),
            ..Default::default()
        };

        encode_fixture_dmx(&profile, &state, None, ColorPreference::ColorMix, &mut buf);
        assert_eq!(buf[0], 255);
        assert_eq!(buf[1], 0);
        assert_eq!(buf[2], 255);
        assert_eq!(buf[3], 255);

        // White flash: flags pulled down toward 0
        state.white_flash_factor = 1.0;
        encode_fixture_dmx(&profile, &state, None, ColorPreference::ColorMix, &mut buf);
        assert_eq!(buf[1], 0);
        assert_eq!(buf[2], 0);
        assert_eq!(buf[3], 0);
    }

    #[test]
    fn test_encode_fixture_auxiliary_defaults() {
        let channel_names = vec![
            "Dimmer".to_string(),
            "Gobo".to_string(),
            "Focus".to_string(),
            "Zoom".to_string(),
        ];
        let defaults = vec![0, 15, 128, 42];
        let profile =
            compile_fixture_profile(&channel_names, Some(&defaults), None, None, None, None);

        let mut buf = vec![0u8; 4];
        let state = LogicalFixtureState::default();

        encode_fixture_dmx(&profile, &state, None, ColorPreference::ColorMix, &mut buf);
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 15);
        assert_eq!(buf[2], 128);
        assert_eq!(buf[3], 42);
    }

    #[test]
    fn test_encode_fixture_multi_pixel_individual() {
        // Multi-pixel fixture with master dimmer, shutter, and 2 RGB pixels:
        // Ch0: Dimmer, Ch1: Shutter, Ch2..4: Pix 0 (RGB), Ch5..7: Pix 1 (RGB)
        let channel_names = vec![
            "Dimmer".to_string(),
            "Shutter".to_string(),
            "Red 1".to_string(),
            "Green 1".to_string(),
            "Blue 1".to_string(),
            "Red 2".to_string(),
            "Green 2".to_string(),
            "Blue 2".to_string(),
        ];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);
        assert_eq!(profile.pixel_count, 2);

        let mut buf = vec![0u8; 8];
        let master_state = LogicalFixtureState {
            intensity: 0.0,
            ..Default::default()
        };
        let pixel_states = vec![
            // Pixel 0: Full Red, 100% intensity
            LogicalFixtureState {
                intensity: 1.0,
                color_rgb: (1.0, 0.0, 0.0),
                ..Default::default()
            },
            // Pixel 1: Full Green, 0% intensity (off)
            LogicalFixtureState {
                intensity: 0.0,
                color_rgb: (0.0, 1.0, 0.0),
                ..Default::default()
            },
        ];

        encode_fixture_dmx(
            &profile,
            &master_state,
            Some(&pixel_states),
            ColorPreference::ColorMix,
            &mut buf,
        );

        // Master dimmer is open because pixel 0 is active
        assert_eq!(buf[0], 255);
        // Shutter is open
        assert_eq!(buf[1], 255);
        // Pixel 0: Red 255, Green 0, Blue 0
        assert_eq!(buf[2], 255);
        assert_eq!(buf[3], 0);
        assert_eq!(buf[4], 0);
        // Pixel 1: 0% intensity => Red 0, Green 0, Blue 0
        assert_eq!(buf[5], 0);
        assert_eq!(buf[6], 0);
        assert_eq!(buf[7], 0);
    }

    #[test]
    fn test_encode_fixture_multi_cell_dimmer() {
        // 4-cell dimmer fixture
        let channel_names = vec![
            "Dimmer 1".to_string(),
            "Dimmer 2".to_string(),
            "Dimmer 3".to_string(),
            "Dimmer 4".to_string(),
        ];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);
        assert_eq!(profile.pixel_count, 4);

        let mut buf = vec![0u8; 4];
        let master_state = LogicalFixtureState::default();
        let pixel_states = vec![
            LogicalFixtureState {
                intensity: 1.0,
                ..Default::default()
            },
            LogicalFixtureState {
                intensity: 0.5,
                ..Default::default()
            },
            LogicalFixtureState {
                intensity: 0.0,
                ..Default::default()
            },
            LogicalFixtureState {
                intensity: 0.25,
                ..Default::default()
            },
        ];

        encode_fixture_dmx(
            &profile,
            &master_state,
            Some(&pixel_states),
            ColorPreference::ColorMix,
            &mut buf,
        );

        assert_eq!(buf[0], 255);
        assert_eq!(buf[1], 128);
        assert_eq!(buf[2], 0);
        assert_eq!(buf[3], 64);
    }
}
