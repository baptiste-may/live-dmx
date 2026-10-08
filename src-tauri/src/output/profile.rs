use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::output::color_routing::ColorChannelSummary;
use crate::show::{EmbeddedGdtfMode, PatchedFixture};

/// Normalized logical lighting attribute of a DMX channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalAttribute {
    Dimmer,
    BeamDimmer,
    BeamWhite,
    Pan,
    Tilt,
    PositionXYZ,
    Red,
    Green,
    Blue,
    White,
    Amber,
    Lime,
    Cyan,
    Magenta,
    Yellow,
    UV,
    Shutter,
    ColorWheel,
    ColorMacro,
    ColorTemperature,
    Gobo,
    Prism,
    Zoom,
    Focus,
    Control,
    Other,
}

/// DMX channel byte resolution and address pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelResolution {
    Bits8 { offset: usize },
    Bits16 { coarse: usize, fine: usize },
}

impl ChannelResolution {
    pub fn primary_offset(&self) -> usize {
        match *self {
            ChannelResolution::Bits8 { offset } => offset,
            ChannelResolution::Bits16 { coarse, .. } => coarse,
        }
    }
}

/// A compiled, ready-to-encode channel definition within a fixture's footprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledChannel {
    pub attribute: LogicalAttribute,
    pub resolution: ChannelResolution,
    pub default_dmx: u8,
    pub shutter_open_dmx: Option<u8>,
    pub pixel_index: Option<usize>,
}

/// A mapped pixel (cell) within a multi-cell or single fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledFixturePixel {
    pub index: usize,
    pub channel_indices: Vec<usize>,
}

/// A normalized, fully resolved fixture profile ready for high-performance DMX encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledFixtureProfile {
    pub footprint: usize,
    pub channels: Vec<CompiledChannel>,
    pub default_dmx: Vec<u8>,
    pub has_dimmer: bool,
    pub treat_first_as_dimmer: bool,
    pub is_cmy: bool,
    pub pan_bounds: Option<[i32; 2]>,
    pub tilt_bounds: Option<[i32; 2]>,
    pub color_summary: ColorChannelSummary,
    pub channel_capabilities: Vec<Vec<serde_json::Value>>,
    pub pixels: Vec<CompiledFixturePixel>,
    pub pixel_count: usize,
}

/// Classifies a raw channel name into a base `LogicalAttribute` and whether it is a fine/LSB channel.
pub fn parse_channel_name(name: &str) -> (LogicalAttribute, bool) {
    let lower = name.to_lowercase();
    let is_fine = lower.contains("fine")
        || lower.contains("lsb")
        || lower.contains("16 bit")
        || lower.contains("16bit")
        || lower.contains("16-bit");

    let is_curve_or_mode = lower.contains("curve")
        || lower.contains("mode")
        || lower.contains("speed")
        || lower.contains("time")
        || lower.contains("duration");

    let attr = if lower.contains("pan") {
        LogicalAttribute::Pan
    } else if lower.contains("tilt") {
        LogicalAttribute::Tilt
    } else if lower.starts_with("xyz_") || lower.contains("xyz") {
        LogicalAttribute::PositionXYZ
    } else if is_fine
        && !lower.contains("pan")
        && !lower.contains("tilt")
        && !lower.contains("dimmer")
        && !lower.contains("intensity")
        && !lower.contains("red")
        && !lower.contains("green")
        && !lower.contains("blue")
        && !lower.contains("white")
        && !lower.contains("cyan")
        && !lower.contains("magenta")
        && !lower.contains("yellow")
    {
        LogicalAttribute::Other
    } else if !is_curve_or_mode
        && lower.contains("beam")
        && (lower.contains("dimmer") || lower.contains("intensity"))
    {
        LogicalAttribute::BeamDimmer
    } else if lower.contains("beam") && lower.contains("white") {
        LogicalAttribute::BeamWhite
    } else if !is_curve_or_mode && (lower.contains("dimmer") || lower.contains("intensity")) {
        LogicalAttribute::Dimmer
    } else if lower == "lime"
        || lower.contains("green-yellow")
        || lower.contains("green yellow")
        || lower.contains("coloradd_gy")
    {
        LogicalAttribute::Lime
    } else if lower.contains("red") || lower == "r" || lower.contains("coloradd_r") {
        LogicalAttribute::Red
    } else if lower.contains("green") || lower == "g" || lower.contains("coloradd_g") {
        LogicalAttribute::Green
    } else if lower.contains("blue") || lower == "b" || lower.contains("coloradd_b") {
        LogicalAttribute::Blue
    } else if lower.contains("white") || lower == "w" || lower.contains("coloradd_w") {
        LogicalAttribute::White
    } else if lower.contains("amber") || lower == "a" || lower.contains("coloradd_a") {
        LogicalAttribute::Amber
    } else if lower.contains("cyan") || lower == "c" || lower.contains("colorsub_c") {
        LogicalAttribute::Cyan
    } else if lower.contains("magenta") || lower == "m" || lower.contains("colorsub_m") {
        LogicalAttribute::Magenta
    } else if lower.contains("yellow") || lower == "y" || lower.contains("colorsub_y") {
        LogicalAttribute::Yellow
    } else if lower.contains("uv") || lower.contains("ultraviolet") || lower.contains("coloradd_uv")
    {
        LogicalAttribute::UV
    } else if (lower.contains("strobe") || lower.contains("shutter")) && !is_curve_or_mode {
        LogicalAttribute::Shutter
    } else if lower.contains("wheel") {
        LogicalAttribute::ColorWheel
    } else if lower.contains("macro")
        || lower.contains("preset")
        || lower.contains("colors")
        || lower.contains("color slot")
    {
        LogicalAttribute::ColorMacro
    } else if lower.contains("cct")
        || lower.contains("cto")
        || lower.contains("ctb")
        || lower.contains("ctc")
        || lower.contains("color temperature")
    {
        LogicalAttribute::ColorTemperature
    } else if lower.contains("gobo") {
        LogicalAttribute::Gobo
    } else if lower.contains("prism") {
        LogicalAttribute::Prism
    } else if lower.contains("zoom") {
        LogicalAttribute::Zoom
    } else if lower.contains("focus") {
        LogicalAttribute::Focus
    } else if lower.contains("control") || lower.contains("reset") {
        LogicalAttribute::Control
    } else {
        LogicalAttribute::Other
    };

    (attr, is_fine)
}

/// Compiles raw channel definitions into a normalized `CompiledFixtureProfile`.
pub fn compile_fixture_profile(
    channel_names: &[String],
    channel_defaults: Option<&[u8]>,
    shutter_open_values: Option<&HashMap<usize, u8>>,
    pan_bounds: Option<[i32; 2]>,
    tilt_bounds: Option<[i32; 2]>,
    capabilities: Option<&[Vec<serde_json::Value>]>,
) -> CompiledFixtureProfile {
    let footprint = channel_names.len();
    if footprint == 0 {
        return CompiledFixtureProfile {
            footprint: 0,
            channels: Vec::new(),
            default_dmx: Vec::new(),
            has_dimmer: false,
            treat_first_as_dimmer: false,
            is_cmy: false,
            pan_bounds: None,
            tilt_bounds: None,
            color_summary: ColorChannelSummary::default(),
            channel_capabilities: Vec::new(),
            pixels: Vec::new(),
            pixel_count: 1,
        };
    }
    let mut default_dmx = vec![0u8; footprint];

    // 1. Initial pass: parse raw attributes and is_fine flags
    let mut raw_attrs = Vec::with_capacity(footprint);
    for (i, name) in channel_names.iter().enumerate() {
        let (mut attr, is_fine) = parse_channel_name(name);

        // Check if an explicit shutter open value exists for this exact channel index
        let has_explicit_shutter = shutter_open_values.is_some_and(|m| m.contains_key(&i));
        if has_explicit_shutter && attr == LogicalAttribute::Other {
            attr = LogicalAttribute::Shutter;
        }

        raw_attrs.push((attr, is_fine));
    }

    // Heuristic: motorized tilt bars / battens (e.g. GLP Impression X4 Bar, Robe Tetra)
    // where channels 0 and 1 are unassigned/NoFeature coarse and fine preceding Dimmer
    let has_tilt = raw_attrs.iter().any(|(a, _)| *a == LogicalAttribute::Tilt);
    let has_pan = raw_attrs.iter().any(|(a, _)| *a == LogicalAttribute::Pan);
    if !has_tilt
        && !has_pan
        && footprint >= 4
        && raw_attrs[0].0 == LogicalAttribute::Other
        && raw_attrs[1].1
    {
        raw_attrs[0] = (LogicalAttribute::Tilt, false);
        raw_attrs[1] = (LogicalAttribute::Tilt, true);
    }

    // Heuristic: placeholder channel immediately preceding master dimmer is designated as Shutter
    let is_placeholder = |name: &str| {
        let l = name.to_lowercase();
        l == "nofeature"
            || l.starts_with("channel ")
            || l.starts_with("dummy")
            || l.starts_with("raw")
    };

    let dimmer_indices: Vec<usize> = raw_attrs
        .iter()
        .enumerate()
        .filter_map(|(idx, (a, f))| {
            if *a == LogicalAttribute::Dimmer && !*f {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    let has_shutter = raw_attrs
        .iter()
        .any(|(a, _)| *a == LogicalAttribute::Shutter);
    if !has_shutter {
        if let Some(&first_dimmer_idx) = dimmer_indices.first() {
            if first_dimmer_idx > 0
                && raw_attrs[first_dimmer_idx - 1].0 == LogicalAttribute::Other
                && is_placeholder(&channel_names[first_dimmer_idx - 1])
            {
                raw_attrs[first_dimmer_idx - 1].0 = LogicalAttribute::Shutter;
            }
        }
    }

    // Heuristic: Multi-dimmer / Hybrid strobe-wash fixtures (e.g. GLP JDC1 / JDC2 IP, Chauvet Color STRIKE M)
    // In hybrid fixtures, a dimmer controlling a strobe tube (no color channels in its section) is designated as BeamDimmer,
    // and isolated White channels in RGB fixtures are designated as BeamWhite.
    let coarse_dimmer_indices: Vec<usize> = raw_attrs
        .iter()
        .enumerate()
        .filter_map(|(idx, (a, f))| {
            if *a == LogicalAttribute::Dimmer && !*f {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    let has_rgb = raw_attrs.iter().any(|(a, _)| {
        matches!(
            a,
            LogicalAttribute::Red | LogicalAttribute::Green | LogicalAttribute::Blue
        )
    });

    if coarse_dimmer_indices.len() > 1 && has_rgb {
        for i in 0..coarse_dimmer_indices.len() {
            let d_curr = coarse_dimmer_indices[i];
            let d_next = if i + 1 < coarse_dimmer_indices.len() {
                coarse_dimmer_indices[i + 1]
            } else {
                raw_attrs.len()
            };

            let has_rgb_in_section = (d_curr..d_next).any(|idx| {
                matches!(
                    raw_attrs[idx].0,
                    LogicalAttribute::Red | LogicalAttribute::Green | LogicalAttribute::Blue
                )
            });

            if !has_rgb_in_section {
                raw_attrs[d_curr].0 = LogicalAttribute::BeamDimmer;
                if d_curr + 1 < raw_attrs.len()
                    && raw_attrs[d_curr + 1].0 == LogicalAttribute::Dimmer
                    && raw_attrs[d_curr + 1].1
                {
                    raw_attrs[d_curr + 1].0 = LogicalAttribute::BeamDimmer;
                }
            }
        }

        let has_beam_dimmer = raw_attrs
            .iter()
            .any(|(a, _)| *a == LogicalAttribute::BeamDimmer);
        if has_beam_dimmer {
            for item in raw_attrs.iter_mut() {
                if item.0 == LogicalAttribute::White {
                    item.0 = LogicalAttribute::BeamWhite;
                }
            }
        }
    }

    // 2. Populate default DMX values across the entire footprint
    for i in 0..footprint {
        let (attr, is_fine) = raw_attrs[i];
        let explicit_default = channel_defaults.and_then(|d| d.get(i)).copied();

        default_dmx[i] = match explicit_default {
            Some(d) if d > 0 => d,
            _ => match attr {
                LogicalAttribute::Pan | LogicalAttribute::Tilt if !is_fine => 128,
                LogicalAttribute::PositionXYZ => 255,
                _ => 0,
            },
        };
    }

    // 3. Pair 16-bit coarse + fine channels and build CompiledChannel list
    let mut channels = Vec::new();
    let mut consumed_fine = vec![false; footprint];

    for i in 0..footprint {
        if consumed_fine[i] {
            continue;
        }

        let (attr, is_fine) = raw_attrs[i];
        if is_fine {
            // Unpaired fine channel
            channels.push(CompiledChannel {
                attribute: attr,
                resolution: ChannelResolution::Bits8 { offset: i },
                default_dmx: default_dmx[i],
                shutter_open_dmx: None,
                pixel_index: None,
            });
            continue;
        }

        // Check if immediately following channel is the corresponding fine counterpart
        let mut resolution = ChannelResolution::Bits8 { offset: i };
        if i + 1 < footprint {
            let (next_attr, next_fine) = raw_attrs[i + 1];
            if next_fine && next_attr == attr {
                resolution = ChannelResolution::Bits16 {
                    coarse: i,
                    fine: i + 1,
                };
                consumed_fine[i + 1] = true;
            }
        }

        let shutter_open_dmx = if attr == LogicalAttribute::Shutter {
            let open_val = shutter_open_values.and_then(|m| {
                m.get(&i).copied().or_else(|| {
                    if raw_attrs
                        .get(i + 1)
                        .is_none_or(|(a, _)| *a != LogicalAttribute::Shutter)
                    {
                        m.get(&(i + 1)).copied()
                    } else {
                        None
                    }
                })
            });

            if let Some(val) = open_val {
                if val > 15 {
                    Some(val)
                } else {
                    Some(255)
                }
            } else if default_dmx[i] > 15 {
                Some(default_dmx[i])
            } else {
                Some(255)
            }
        } else {
            None
        };

        channels.push(CompiledChannel {
            attribute: attr,
            resolution,
            default_dmx: default_dmx[i],
            shutter_open_dmx,
            pixel_index: None,
        });
    }

    let has_dimmer = raw_attrs
        .iter()
        .any(|(a, f)| *a == LogicalAttribute::Dimmer && !*f);
    let has_position = raw_attrs
        .iter()
        .any(|(a, _)| *a == LogicalAttribute::PositionXYZ);
    let treat_first_as_dimmer = !has_dimmer && !has_position && footprint == 1;

    // Detect CMY: has Cyan, Magenta, or Yellow without primary additive RGB
    let has_cmy_channels = raw_attrs.iter().any(|(a, _)| {
        matches!(
            a,
            LogicalAttribute::Cyan | LogicalAttribute::Magenta | LogicalAttribute::Yellow
        )
    });
    let has_rgb_channels = raw_attrs.iter().any(|(a, _)| {
        matches!(
            a,
            LogicalAttribute::Red | LogicalAttribute::Green | LogicalAttribute::Blue
        )
    });
    let is_cmy = has_cmy_channels && !has_rgb_channels;

    let mut color_summary = ColorChannelSummary::default();
    for (attr, _) in &raw_attrs {
        match attr {
            LogicalAttribute::Red
            | LogicalAttribute::Green
            | LogicalAttribute::Blue
            | LogicalAttribute::White
            | LogicalAttribute::Amber
            | LogicalAttribute::Cyan
            | LogicalAttribute::Magenta
            | LogicalAttribute::Yellow
            | LogicalAttribute::UV => color_summary.has_mix = true,
            LogicalAttribute::ColorWheel => color_summary.has_wheel = true,
            LogicalAttribute::ColorMacro => color_summary.has_macro = true,
            LogicalAttribute::ColorTemperature => color_summary.has_temp = true,
            _ => {}
        }
    }

    // 4. Detect pixels (cells) across channels
    let primary_attr = if is_cmy {
        LogicalAttribute::Cyan
    } else {
        LogicalAttribute::Red
    };

    let primary_indices: Vec<usize> = channels
        .iter()
        .enumerate()
        .filter_map(|(idx, ch)| {
            if ch.attribute == primary_attr {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    let coarse_dimmer_indices: Vec<usize> = channels
        .iter()
        .enumerate()
        .filter_map(|(idx, ch)| {
            if ch.attribute == LogicalAttribute::Dimmer {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    let mut pixels: Vec<CompiledFixturePixel> = Vec::new();

    if primary_indices.len() > 1 {
        // Multi-cell color fixture (e.g. RGB/RGBW/CMY pixel bars, washes)
        let num_pixels = primary_indices.len();
        let has_per_pixel_dimmer = coarse_dimmer_indices.len() == num_pixels;

        for (p_idx, &start_ch) in primary_indices.iter().enumerate() {
            let next_start = if p_idx + 1 < num_pixels {
                primary_indices[p_idx + 1]
            } else {
                channels.len()
            };

            let mut cell_channels = Vec::new();

            if has_per_pixel_dimmer {
                if let Some(&dim_idx) = coarse_dimmer_indices.get(p_idx) {
                    if dim_idx < next_start {
                        cell_channels.push(dim_idx);
                        channels[dim_idx].pixel_index = Some(p_idx);
                    }
                }
            }

            for (offset_idx, ch) in channels[start_ch..next_start].iter_mut().enumerate() {
                let ch_idx = start_ch + offset_idx;
                let attr = ch.attribute;
                if matches!(
                    attr,
                    LogicalAttribute::Red
                        | LogicalAttribute::Green
                        | LogicalAttribute::Blue
                        | LogicalAttribute::White
                        | LogicalAttribute::Amber
                        | LogicalAttribute::Lime
                        | LogicalAttribute::Cyan
                        | LogicalAttribute::Magenta
                        | LogicalAttribute::Yellow
                        | LogicalAttribute::UV
                ) {
                    cell_channels.push(ch_idx);
                    ch.pixel_index = Some(p_idx);
                }
            }

            pixels.push(CompiledFixturePixel {
                index: p_idx,
                channel_indices: cell_channels,
            });
        }
    } else if primary_indices.is_empty() && coarse_dimmer_indices.len() > 1 {
        // Multi-cell dimmer fixture (e.g. 4-cell or 8-cell blinder/sunstrip)
        for (p_idx, &dim_idx) in coarse_dimmer_indices.iter().enumerate() {
            channels[dim_idx].pixel_index = Some(p_idx);
            pixels.push(CompiledFixturePixel {
                index: p_idx,
                channel_indices: vec![dim_idx],
            });
        }
    }

    if pixels.is_empty() {
        let single_channels: Vec<usize> = channels
            .iter()
            .enumerate()
            .filter_map(|(idx, ch)| {
                if matches!(
                    ch.attribute,
                    LogicalAttribute::Red
                        | LogicalAttribute::Green
                        | LogicalAttribute::Blue
                        | LogicalAttribute::White
                        | LogicalAttribute::Amber
                        | LogicalAttribute::Lime
                        | LogicalAttribute::Cyan
                        | LogicalAttribute::Magenta
                        | LogicalAttribute::Yellow
                        | LogicalAttribute::UV
                        | LogicalAttribute::Dimmer
                ) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        pixels.push(CompiledFixturePixel {
            index: 0,
            channel_indices: single_channels,
        });
    }

    let pixel_count = pixels.len();

    let channel_capabilities = capabilities
        .map(|c| c.to_vec())
        .unwrap_or_else(|| vec![Vec::new(); footprint]);

    CompiledFixtureProfile {
        footprint,
        channels,
        default_dmx,
        has_dimmer,
        treat_first_as_dimmer,
        is_cmy,
        pan_bounds,
        tilt_bounds,
        color_summary,
        channel_capabilities,
        pixels,
        pixel_count,
    }
}

/// Helper to compile an `EmbeddedGdtfMode` with optional capability data.
pub fn compile_from_embedded_mode(
    mode: &EmbeddedGdtfMode,
    capabilities: Option<&[Vec<serde_json::Value>]>,
) -> CompiledFixtureProfile {
    compile_fixture_profile(
        &mode.channels,
        mode.channel_defaults.as_deref(),
        mode.shutter_open_values.as_ref(),
        mode.pan_bounds,
        mode.tilt_bounds,
        capabilities,
    )
}

/// Helper to extract mode information from a `PatchedFixture` and compile it.
pub fn compile_from_patched_fixture(fixture: &PatchedFixture) -> CompiledFixtureProfile {
    let mut channel_names = Vec::new();
    let mut capabilities = Vec::new();

    if let Some(channels_arr) = fixture.mode.get("channels").and_then(|c| c.as_array()) {
        for ch_val in channels_arr {
            let mut name = String::new();
            let mut caps = Vec::new();

            if let Some(s) = ch_val.as_str() {
                name = s.to_string();
            } else if let Some(obj) = ch_val.as_object() {
                if let Some(n) = obj.get("name").and_then(|v| v.as_str()) {
                    name = n.to_string();
                } else if let Some(c) = obj.get("channel").and_then(|v| v.as_str()) {
                    name = c.to_string();
                }
                if let Some(arr) = obj.get("capabilities").and_then(|v| v.as_array()) {
                    caps = arr.clone();
                } else if let Some(cap) = obj.get("capability") {
                    caps = vec![cap.clone()];
                }
            }
            channel_names.push(name);
            capabilities.push(caps);
        }
    }

    let defaults_vec: Option<Vec<u8>> = fixture
        .mode
        .get("channelDefaults")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect());

    let shutter_map: Option<HashMap<usize, u8>> = fixture
        .mode
        .get("shutterOpenValues")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| {
                    k.parse::<usize>()
                        .ok()
                        .map(|idx| (idx, v.as_u64().unwrap_or(255) as u8))
                })
                .collect()
        });

    let pan_bounds: Option<[i32; 2]> = fixture
        .mode
        .get("panBounds")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 2 {
                Some([arr[0].as_i64()? as i32, arr[1].as_i64()? as i32])
            } else {
                None
            }
        });

    let tilt_bounds: Option<[i32; 2]> = fixture
        .mode
        .get("tiltBounds")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 2 {
                Some([arr[0].as_i64()? as i32, arr[1].as_i64()? as i32])
            } else {
                None
            }
        });

    compile_fixture_profile(
        &channel_names,
        defaults_vec.as_deref(),
        shutter_map.as_ref(),
        pan_bounds,
        tilt_bounds,
        Some(&capabilities),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_moving_head_16bit() {
        let channel_names = vec![
            "Pan".to_string(),
            "Pan Fine".to_string(),
            "Tilt".to_string(),
            "Tilt Fine".to_string(),
            "Shutter".to_string(),
            "Dimmer".to_string(),
            "Dimmer Fine".to_string(),
            "Red".to_string(),
            "Green".to_string(),
            "Blue".to_string(),
            "White".to_string(),
        ];
        let mut shutter_map = HashMap::new();
        shutter_map.insert(4, 250);

        let profile = compile_fixture_profile(
            &channel_names,
            None,
            Some(&shutter_map),
            Some([-270, 270]),
            Some([-110, 110]),
            None,
        );

        assert_eq!(profile.footprint, 11);
        assert!(profile.has_dimmer);
        assert!(!profile.is_cmy);
        assert_eq!(profile.pan_bounds, Some([-270, 270]));
        assert_eq!(profile.tilt_bounds, Some([-110, 110]));

        // Check defaults: Pan/Tilt coarse at 128, fine at 0
        assert_eq!(profile.default_dmx[0], 128);
        assert_eq!(profile.default_dmx[1], 0);
        assert_eq!(profile.default_dmx[2], 128);
        assert_eq!(profile.default_dmx[3], 0);

        // Check 16-bit pairings
        let pan_ch = profile
            .channels
            .iter()
            .find(|c| c.attribute == LogicalAttribute::Pan)
            .unwrap();
        assert_eq!(
            pan_ch.resolution,
            ChannelResolution::Bits16 { coarse: 0, fine: 1 }
        );

        let tilt_ch = profile
            .channels
            .iter()
            .find(|c| c.attribute == LogicalAttribute::Tilt)
            .unwrap();
        assert_eq!(
            tilt_ch.resolution,
            ChannelResolution::Bits16 { coarse: 2, fine: 3 }
        );

        let dimmer_ch = profile
            .channels
            .iter()
            .find(|c| c.attribute == LogicalAttribute::Dimmer)
            .unwrap();
        assert_eq!(
            dimmer_ch.resolution,
            ChannelResolution::Bits16 { coarse: 5, fine: 6 }
        );

        let shutter_ch = profile
            .channels
            .iter()
            .find(|c| c.attribute == LogicalAttribute::Shutter)
            .unwrap();
        assert_eq!(shutter_ch.shutter_open_dmx, Some(250));
    }

    #[test]
    fn test_compile_cmy_fixture() {
        let channel_names = vec![
            "Dimmer".to_string(),
            "Cyan".to_string(),
            "Magenta".to_string(),
            "Yellow".to_string(),
        ];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);
        assert!(profile.has_dimmer);
        assert!(profile.is_cmy);
        assert!(profile.color_summary.has_mix);
    }

    #[test]
    fn test_compile_rgb_fixture_without_dimmer() {
        let channel_names = vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);
        assert!(!profile.has_dimmer);
        assert!(!profile.is_cmy);
        assert!(profile.color_summary.has_mix);
    }

    #[test]
    fn test_compile_batten_shutter_heuristic() {
        let channel_names = vec![
            "Tilt".to_string(),
            "Tilt Fine".to_string(),
            "NoFeature".to_string(),
            "Dimmer".to_string(),
            "Red".to_string(),
            "Green".to_string(),
            "Blue".to_string(),
            "White".to_string(),
        ];
        let profile = compile_fixture_profile(&channel_names, None, None, None, None, None);
        let shutter_ch = profile
            .channels
            .iter()
            .find(|c| c.attribute == LogicalAttribute::Shutter)
            .unwrap();
        assert_eq!(
            shutter_ch.resolution,
            ChannelResolution::Bits8 { offset: 2 }
        );
        assert_eq!(shutter_ch.shutter_open_dmx, Some(255));
    }
}
