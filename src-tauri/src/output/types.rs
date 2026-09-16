use serde::{Deserialize, Serialize};

fn default_width() -> u8 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupEffectState {
    pub effect: Option<String>,
    pub alignment: Option<String>,
    pub bounce: bool,
    #[serde(default = "default_width")]
    pub width: u8,
}

impl Default for GroupEffectState {
    fn default() -> Self {
        Self {
            effect: None,
            alignment: None,
            bounce: false,
            width: 100,
        }
    }
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
    ColorWheel,
    ColorMacro,
    ColorTemperature,
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
