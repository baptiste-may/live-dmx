//! Small color-conversion helpers shared by the live engine.
//!
//! See `docs/contributing/rendering.md` for how colors flow through the pipeline.

/// Resolves a color name or `#RRGGBB` hex string to an RGB triple (0-255 each).
///
/// Input is trimmed and names are case-insensitive. Unknown names and malformed
/// hex strings fall back to white so a bad value never blacks out a fixture.
pub fn resolve_color_rgb(color_name_or_hex: &str) -> (u8, u8, u8) {
    let s = color_name_or_hex.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return (r, g, b);
            }
        }
    }
    match s.to_lowercase().as_str() {
        "white" => (255, 255, 255),
        "ctb" => (198, 230, 255),
        "cto" => (255, 199, 138),
        "red" => (255, 0, 0),
        "orange" => (255, 127, 0),
        "yellow" => (255, 255, 0),
        "lime" => (127, 255, 0),
        "green" => (0, 255, 0),
        "mint" => (0, 255, 127),
        "cyan" => (0, 255, 255),
        "azure" => (0, 127, 255),
        "blue" => (0, 0, 255),
        "violet" => (127, 0, 255),
        "magenta" => (255, 0, 255),
        "pink" => (255, 0, 127),
        _ => (255, 255, 255),
    }
}

/// Converts HSV to RGB, returning each component on a 0.0-255.0 scale.
///
/// `h` is a hue in turns (1.0 = full circle) and wraps, so negative or
/// greater-than-1 values are valid. `s` and `v` are clamped to 0.0-1.0. The
/// result is not rounded so callers can keep float precision until encoding.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    if s <= 0.0 {
        let val = v * 255.0;
        return (val, val, val);
    }
    let h_scaled = (h.fract() + 1.0).fract() * 6.0;
    let i = h_scaled.floor() as i32;
    let f = h_scaled - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    (r * 255.0, g * 255.0, b * 255.0)
}
