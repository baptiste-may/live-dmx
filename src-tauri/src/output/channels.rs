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
    if lower.contains("dimmer") || lower.contains("intensity") {
        ChannelType::Dimmer
    } else if lower.contains("red") || lower == "r" {
        ChannelType::Red
    } else if lower.contains("green") || lower == "g" {
        ChannelType::Green
    } else if lower.contains("blue") || lower == "b" {
        ChannelType::Blue
    } else if lower.contains("white") || lower == "w" {
        ChannelType::White
    } else if lower.contains("amber") || lower == "a" {
        ChannelType::Amber
    } else if lower.contains("cyan") || lower == "c" {
        ChannelType::Cyan
    } else if lower.contains("magenta") || lower == "m" {
        ChannelType::Magenta
    } else if lower.contains("yellow") || lower == "y" {
        ChannelType::Yellow
    } else if lower.contains("uv") || lower.contains("ultraviolet") {
        ChannelType::UV
    } else if lower.contains("strobe") || lower.contains("shutter") {
        ChannelType::Strobe
    } else {
        ChannelType::Other
    }
}

pub fn extract_fixture_channel_count(fixture: &PatchedFixture) -> usize {
    if let Some(channels) = fixture.mode.get("channels").and_then(|c| c.as_array()) {
        return channels.len().max(1);
    }
    1
}
