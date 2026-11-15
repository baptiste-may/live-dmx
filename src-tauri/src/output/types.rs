//! Small shared types of the live output: per-group effect state, the Configure channel
//! classification and the colour/flash fade transitions kept by `LiveEngine`.
//!
//! See `docs/contributing/rendering.md` and `docs/contributing/dmx-output.md`.

use serde::{Deserialize, Serialize};

/// Serde default for [`GroupEffectState::width`]: 100 %, i.e. the whole cycle is active.
fn default_width() -> u8 {
    100
}

/// Effect settings applied to one group (or to the global group `0`).
///
/// Used for both the intensity effect and the colour effect of a group. The default value
/// means "no effect": nothing runs and `width` is 100 %. Serialized in camelCase, with
/// `width` defaulting to 100 when absent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupEffectState {
    /// Effect name, or `None` when no effect is active.
    pub effect: Option<String>,
    /// Alignment of the effect phase across the group's fixtures or pixels (for example
    /// `"rng"` for a shuffled order), or `None` for the default alignment.
    pub alignment: Option<String>,
    /// Whether every odd cycle plays in reverse, so the effect bounces back and forth.
    pub bounce: bool,
    /// Active part of each effect cycle in percent (0-100); the rest of the cycle is off.
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

/// Channel classification used by the Configure capabilities/options UI.
///
/// Unlike `LogicalAttribute` (DMX encoding, in `profile.rs`) this keeps the fine channels of
/// pan and tilt as separate variants. It is produced by `classify_channel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    /// Master intensity.
    Dimmer,
    /// Intensity of a separate beam/strobe section of a hybrid fixture.
    BeamDimmer,
    /// White emitter of a beam section.
    BeamWhite,
    /// Pan, coarse byte.
    Pan,
    /// Pan, fine (LSB) byte.
    PanFine,
    /// Tilt, coarse byte.
    Tilt,
    /// Tilt, fine (LSB) byte.
    TiltFine,
    /// Position along an XYZ axis.
    PositionXYZ,
    /// Red emitter.
    Red,
    /// Green emitter.
    Green,
    /// Blue emitter.
    Blue,
    /// White emitter.
    White,
    /// Amber emitter.
    Amber,
    /// Lime (green-yellow) emitter.
    Lime,
    /// Cyan colour-mixing channel.
    Cyan,
    /// Magenta colour-mixing channel.
    Magenta,
    /// Yellow colour-mixing channel.
    Yellow,
    /// Ultraviolet emitter.
    UV,
    /// Strobe or shutter.
    Strobe,
    /// Colour wheel position.
    ColorWheel,
    /// Colour macro or preset selector.
    ColorMacro,
    /// Colour temperature (CCT/CTO/CTB) control.
    ColorTemperature,
    /// Anything not recognised above, including unpaired fine channels.
    Other,
}

/// In-flight colour fade of one fixture towards a new group colour.
///
/// The engine interpolates linearly between `start_rgb` and `target_rgb` once `delay_secs`
/// have elapsed since `start_time`, over `duration_secs`; it drops the transition when
/// finished. The per-fixture delay staggers the fade according to `color_fade_order`.
#[derive(Debug, Clone)]
pub struct FixtureColorTransition {
    /// RGB at the start of the fade, each component 0.0-255.0.
    pub start_rgb: (f32, f32, f32),
    /// RGB the fade ends on, each component 0.0-255.0.
    pub target_rgb: (f32, f32, f32),
    /// Instant at which the transition was created.
    pub start_time: std::time::Instant,
    /// Wait before the fade starts, in seconds.
    pub delay_secs: f32,
    /// Length of the fade itself (excluding the delay), in seconds. A value of 0 or less
    /// jumps straight to the target.
    pub duration_secs: f32,
}

/// Release fade of a flash button after it is let go.
///
/// The flash factor decreases linearly from 1.0 to 0.0 over `duration_secs` from `start_time`.
#[derive(Debug, Clone)]
pub struct FlashFadeTransition {
    /// Instant at which the flash was released.
    pub start_time: std::time::Instant,
    /// Fade length in seconds (always positive when a transition is stored).
    pub duration_secs: f32,
}
