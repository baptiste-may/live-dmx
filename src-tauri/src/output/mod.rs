//! Live DMX output: rendering, universe routing and transmission.
//!
//! [`engine`] holds the mutable live state (`LiveEngine`) and turns it plus the show into
//! one 512-byte frame per universe, using the pure rendering stack ([`profile`],
//! [`channels`], [`colors`], [`color_routing`], [`effects`], [`pipeline`]). [`runner`] drives
//! it from a background loop at about 44 Hz and hands each enabled universe to a network
//! sender ([`protocols`]) or a USB interface ([`interfaces`]). [`commands`] exposes the live
//! controls to the frontend and [`types`] holds the small shared state types.
//!
//! Everything is re-exported flat so callers use `crate::output::Item`.
//!
//! See `docs/contributing/dmx-output.md` and `docs/contributing/rendering.md`.

pub mod channels;
pub mod color_routing;
pub mod colors;
pub mod commands;
pub mod effects;
pub mod engine;
pub mod interfaces;
pub mod pipeline;
pub mod profile;
pub mod protocols;
pub mod runner;
pub mod types;

pub use channels::*;
pub use color_routing::*;
pub use colors::*;
pub use commands::*;
pub use effects::*;
pub use engine::*;
pub use interfaces::*;
pub use pipeline::*;
pub use profile::*;
pub use protocols::*;
pub use runner::*;
pub use types::*;
