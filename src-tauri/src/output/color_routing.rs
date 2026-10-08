use serde::{Deserialize, Serialize};

use crate::output::types::ChannelType;
use crate::show::ColorPreference;

/// Precomputed color-channel capability summary for a fixture, derived from its
/// stable channel list once instead of scanned on every frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorChannelSummary {
    pub has_mix: bool,
    pub has_wheel: bool,
    pub has_macro: bool,
    pub has_temp: bool,
}

impl ColorChannelSummary {
    pub fn from_channels(channels: &[ChannelType]) -> Self {
        let mut summary = ColorChannelSummary::default();
        for c in channels {
            match c {
                ChannelType::Red
                | ChannelType::Green
                | ChannelType::Blue
                | ChannelType::White
                | ChannelType::Amber
                | ChannelType::Cyan
                | ChannelType::Magenta
                | ChannelType::Yellow
                | ChannelType::UV => summary.has_mix = true,
                ChannelType::ColorWheel => summary.has_wheel = true,
                ChannelType::ColorMacro => summary.has_macro = true,
                ChannelType::ColorTemperature => summary.has_temp = true,
                _ => {}
            }
        }
        summary
    }
}

/// Resolves the color routing mode that should actually be applied to a
/// fixture, based on its configured preference and the channel classes it
/// actually exposes. When a fixture lacks the channels required by the
/// configured preference it falls back to whatever color channel class is
/// available so output is never silently dropped.
pub fn effective_color_preference(
    preference: ColorPreference,
    summary: ColorChannelSummary,
) -> ColorPreference {
    let ColorChannelSummary {
        has_mix,
        has_wheel,
        has_macro,
        has_temp,
    } = summary;

    match preference {
        ColorPreference::ColorMix => {
            if has_mix {
                ColorPreference::ColorMix
            } else if has_wheel {
                ColorPreference::ColorWheel
            } else if has_macro {
                ColorPreference::ColorPresets
            } else if has_temp {
                ColorPreference::Cct
            } else {
                ColorPreference::ColorMix
            }
        }
        ColorPreference::ColorWheel => {
            if has_wheel {
                ColorPreference::ColorWheel
            } else if has_mix {
                ColorPreference::ColorMix
            } else if has_macro {
                ColorPreference::ColorPresets
            } else if has_temp {
                ColorPreference::Cct
            } else {
                ColorPreference::ColorWheel
            }
        }
        ColorPreference::ColorPresets => {
            if has_macro {
                ColorPreference::ColorPresets
            } else if has_mix {
                ColorPreference::ColorMix
            } else if has_wheel {
                ColorPreference::ColorWheel
            } else if has_temp {
                ColorPreference::Cct
            } else {
                ColorPreference::ColorPresets
            }
        }
        ColorPreference::Cct => {
            if has_temp {
                ColorPreference::Cct
            } else if has_mix {
                ColorPreference::ColorMix
            } else if has_wheel {
                ColorPreference::ColorWheel
            } else if has_macro {
                ColorPreference::ColorPresets
            } else {
                ColorPreference::Cct
            }
        }
    }
}

/// Fallback palette used when a wheel/preset channel has no capability data.
const FALLBACK_SLOTS: [(u8, u8, u8); 8] = [
    (255, 255, 255), // white
    (255, 0, 0),     // red
    (255, 127, 0),   // orange
    (255, 255, 0),   // yellow
    (0, 255, 0),     // green
    (0, 255, 255),   // cyan
    (0, 0, 255),     // blue
    (255, 0, 255),   // magenta
];

fn squared_distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let dr = a.0 as f64 - b.0 as f64;
    let dg = a.1 as f64 - b.1 as f64;
    let db = a.2 as f64 - b.2 as f64;
    dr * dr + dg * dg + db * db
}

/// Picks a fixed slot position (midpoint of an even 8-slot spread) matching
/// the requested color, used when a fixture exposes no capability data.
fn fallback_slot_position(r: u8, g: u8, b: u8) -> u8 {
    let mut best_idx = 0;
    let mut best_dist = f64::MAX;
    for (i, &slot) in FALLBACK_SLOTS.iter().enumerate() {
        let dist = squared_distance((r, g, b), slot);
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }
    (((best_idx as f32 + 0.5) / FALLBACK_SLOTS.len() as f32) * 255.0).round() as u8
}

fn capability_dmx_value(cap: &serde_json::Value) -> Option<u8> {
    let from = cap.get("dmxFrom").and_then(|v| v.as_u64())?;
    let to = cap.get("dmxTo").and_then(|v| v.as_u64())?;
    Some((((from + to) / 2).clamp(0, 255)) as u8)
}

fn capability_colors(cap: &serde_json::Value) -> Vec<(u8, u8, u8)> {
    let mut out = Vec::new();
    if let Some(color) = cap.get("color").and_then(|c| c.as_str()) {
        if let Some(rgb) = parse_hex_color(color) {
            out.push(rgb);
        }
    }
    if let Some(colors) = cap.get("colors").and_then(|c| c.as_array()) {
        for c in colors {
            if let Some(hex) = c.as_str() {
                if let Some(rgb) = parse_hex_color(hex) {
                    out.push(rgb);
                }
            }
        }
    }
    out
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Finds the closest matching position for a wheel or preset channel given the
/// requested color. Uses capability color slots when available, otherwise the
/// fixed 8-slot fallback.
pub fn closest_color_position(r: u8, g: u8, b: u8, capabilities: &[serde_json::Value]) -> u8 {
    if capabilities.is_empty() {
        return fallback_slot_position(r, g, b);
    }

    let mut best_position: Option<u8> = None;
    let mut best_dist = f64::MAX;
    for cap in capabilities {
        let colors = capability_colors(cap);
        if colors.is_empty() {
            continue;
        }
        let Some(position) = capability_dmx_value(cap) else {
            continue;
        };
        for color in colors {
            let dist = squared_distance((r, g, b), color);
            if dist < best_dist {
                best_dist = dist;
                best_position = Some(position);
            }
        }
    }

    best_position.unwrap_or_else(|| fallback_slot_position(r, g, b))
}

fn linearize(c: u8) -> f64 {
    let v = c as f64 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Approximates the correlated color temperature (Kelvin) of an sRGB color
/// using the McCamy cubic formula. Clamped to the 2000K-10000K range typically
/// addressable by stage fixtures.
pub fn rgb_to_color_temperature(r: u8, g: u8, b: u8) -> f64 {
    let r_lin = linearize(r);
    let g_lin = linearize(g);
    let b_lin = linearize(b);

    let x = 0.4124564 * r_lin + 0.3575761 * g_lin + 0.1804375 * b_lin;
    let y = 0.2126729 * r_lin + 0.7151522 * g_lin + 0.0721750 * b_lin;
    let z = 0.0193339 * r_lin + 0.1191920 * g_lin + 0.9503041 * b_lin;

    let sum = x + y + z;
    if sum <= 0.0 {
        return 3200.0;
    }

    let cx = x / sum;
    let cy = y / sum;
    let n = (cx - 0.3320) / (0.1858 - cy);
    let cct = 449.0 * n.powi(3) + 3525.0 * n.powi(2) + 6823.3 * n + 5520.33;
    cct.clamp(2000.0, 10000.0)
}

/// Maps a color temperature to a 0-255 DMX byte over the 2000K-10000K range.
pub fn map_temperature_to_dmx(kelvin: f64) -> u8 {
    let t = kelvin.clamp(2000.0, 10000.0);
    (((t - 2000.0) / (10000.0 - 2000.0)) * 255.0).round() as u8
}
