//! Data model of a physical DMX interface exposed to the frontend.

use serde::{Deserialize, Serialize};

/// A physical (USB serial) DMX interface, serialized to the frontend in camelCase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalInterface {
    /// Stable identifier of the form `physical-{n}` (1-based position in the scan),
    /// used as a universe output target.
    pub id: String,
    /// Display name of the form `Physical {n}`.
    pub name: String,
    /// Human-readable device label (USB product, manufacturer or a generic fallback).
    pub device: String,
    /// OS serial port path, e.g. `/dev/ttyUSB0`, `COM3` or `/dev/cu.usbmodem101`.
    pub port: String,
    /// Connection status shown in the UI; discovery always reports `connected`.
    pub status: String,
    /// Nominal output refresh rate in frames per second.
    pub fps: u32,
}
