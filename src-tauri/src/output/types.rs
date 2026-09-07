use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupEffectState {
    pub effect: Option<String>,
    pub alignment: Option<String>,
    pub bounce: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Dimmer,
    Red,
    Green,
    Blue,
    White,
    Amber,
    Cyan,
    Magenta,
    Yellow,
    UV,
    Strobe,
    Other,
}

#[derive(Debug, Clone)]
pub struct FixtureColorTransition {
    pub start_rgb: (f32, f32, f32),
    pub target_rgb: (f32, f32, f32),
    pub start_time: std::time::Instant,
    pub delay_secs: f32,
    pub duration_secs: f32,
}

#[derive(Debug, Clone)]
pub struct FlashFadeTransition {
    pub start_time: std::time::Instant,
    pub duration_secs: f32,
}
