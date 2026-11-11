//! Universe output routing commands of the show.
//!
//! The configuration lives in `ShowData.output.universes` and is read by the
//! live output loop on its next tick. See `docs/contributing/dmx-output.md`.

pub mod commands;

pub use commands::*;
