use crate::output::types::ChannelType;
use crate::show::PatchedFixture;

pub fn get_fixture_channel_name(fixture: &PatchedFixture, ch_index: usize) -> Option<String> {
    let channels = fixture.mode.get("channels")?.as_array()?;
    let channel_val = channels.get(ch_index)?;
    if let Some(s) = channel_val.as_str() {
        return Some(s.to_string());
    }
    if let Some(obj) = channel_val.as_object() {
        if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
            return Some(name.to_string());
        }
        if let Some(ch) = obj.get("channel").and_then(|c| c.as_str()) {
            return Some(ch.to_string());
        }
    }
    None
}

pub fn classify_channel(name: &str) -> ChannelType {
    let lower = name.to_lowercase();
    let is_fine = lower.contains("fine")
        || lower.contains("lsb")
        || lower.contains("16 bit")
        || lower.contains("16bit")
        || lower.contains("16-bit");

    let is_curve_or_mode = lower.contains("curve")
        || lower.contains("mode")
        || lower.contains("speed")
        || lower.contains("time");

    if lower.contains("pan") {
        if is_fine {
            ChannelType::PanFine
        } else {
            ChannelType::Pan
        }
    } else if lower.contains("tilt") {
        if is_fine {
            ChannelType::TiltFine
        } else {
            ChannelType::Tilt
        }
    } else if lower.starts_with("xyz_") || lower.contains("xyz") {
        ChannelType::PositionXYZ
    } else if is_fine {
        ChannelType::Other
    } else if !is_curve_or_mode
        && lower.contains("beam")
        && (lower.contains("dimmer") || lower.contains("intensity"))
    {
        ChannelType::BeamDimmer
    } else if lower.contains("beam") && lower.contains("white") {
        ChannelType::BeamWhite
    } else if !is_curve_or_mode && (lower.contains("dimmer") || lower.contains("intensity")) {
        ChannelType::Dimmer
    } else if lower == "lime"
        || lower.contains("green-yellow")
        || lower.contains("green yellow")
        || lower.contains("coloradd_gy")
    {
        ChannelType::Lime
    } else if lower.contains("red") || lower == "r" || lower.contains("coloradd_r") {
        ChannelType::Red
    } else if lower.contains("green") || lower == "g" || lower.contains("coloradd_g") {
        ChannelType::Green
    } else if lower.contains("blue") || lower == "b" || lower.contains("coloradd_b") {
        ChannelType::Blue
    } else if lower.contains("white") || lower == "w" || lower.contains("coloradd_w") {
        ChannelType::White
    } else if lower.contains("amber") || lower == "a" || lower.contains("coloradd_a") {
        ChannelType::Amber
    } else if lower.contains("cyan") || lower == "c" || lower.contains("colorsub_c") {
        ChannelType::Cyan
    } else if lower.contains("magenta") || lower == "m" || lower.contains("colorsub_m") {
        ChannelType::Magenta
    } else if lower.contains("yellow") || lower == "y" || lower.contains("colorsub_y") {
        ChannelType::Yellow
    } else if lower.contains("uv") || lower.contains("ultraviolet") || lower.contains("coloradd_uv")
    {
        ChannelType::UV
    } else if lower.contains("strobe") || lower.contains("shutter") {
        ChannelType::Strobe
    } else if lower.contains("wheel") {
        ChannelType::ColorWheel
    } else if lower.contains("macro")
        || lower.contains("preset")
        || lower.contains("colors")
        || lower.contains("color slot")
    {
        ChannelType::ColorMacro
    } else if lower.contains("cct")
        || lower.contains("cto")
        || lower.contains("ctb")
        || lower.contains("ctc")
        || lower.contains("color temperature")
    {
        ChannelType::ColorTemperature
    } else {
        ChannelType::Other
    }
}

pub fn get_fixture_channel_default(fixture: &PatchedFixture, ch_index: usize) -> Option<u8> {
    if let Some(arr) = fixture
        .mode
        .get("channelDefaults")
        .and_then(|v| v.as_array())
    {
        if let Some(val) = arr.get(ch_index).and_then(|v| v.as_u64()) {
            return Some(val as u8);
        }
    }
    let channels = fixture.mode.get("channels")?.as_array()?;
    let channel_val = channels.get(ch_index)?;
    if let Some(obj) = channel_val.as_object() {
        if let Some(d) = obj.get("default").and_then(|v| v.as_u64()) {
            return Some(d as u8);
        }
    }
    None
}

pub fn get_fixture_shutter_open_value(fixture: &PatchedFixture, ch_index: usize) -> u8 {
    if let Some(obj) = fixture
        .mode
        .get("shutterOpenValues")
        .and_then(|v| v.as_object())
    {
        if let Some(val) = obj.get(&ch_index.to_string()).and_then(|v| v.as_u64()) {
            return val as u8;
        }
    }
    if let Some(d) = get_fixture_channel_default(fixture, ch_index) {
        if d > 0 {
            return d;
        }
    }
    255
}

/// Returns the capabilities array of a channel when it is stored as a
/// `NamedChannel` object ({ name, channel?, capability?, capabilities? }).
/// Capability data is currently only preserved when patched via the full GDTF
/// pipeline; the rest of the time this returns an empty slice.
pub fn get_fixture_channel_capabilities(
    fixture: &PatchedFixture,
    ch_index: usize,
) -> Vec<serde_json::Value> {
    let Some(channels) = fixture.mode.get("channels").and_then(|c| c.as_array()) else {
        return Vec::new();
    };
    let Some(channel_val) = channels.get(ch_index) else {
        return Vec::new();
    };
    let Some(obj) = channel_val.as_object() else {
        return Vec::new();
    };
    if let Some(caps) = obj.get("capabilities") {
        if let Some(arr) = caps.as_array() {
            return arr.clone();
        }
    }
    if let Some(cap) = obj.get("capability") {
        return vec![cap.clone()];
    }
    Vec::new()
}

pub fn extract_fixture_channel_count(fixture: &PatchedFixture) -> usize {
    if let Some(channels) = fixture.mode.get("channels").and_then(|c| c.as_array()) {
        return channels.len().max(1);
    }
    1
}
