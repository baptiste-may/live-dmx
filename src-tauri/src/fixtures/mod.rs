//! Fixture definitions: built-in generics, the GDTF parser, the GDTF-Share
//! catalog/client/credentials vault and the Tauri commands exposing them.
//!
//! See `docs/contributing/fixtures.md` for the data model and data flows.

/// Tauri commands for searching, fetching and importing fixtures.
pub mod commands;
/// GDTF-Share HTTP client, local catalog cache and credentials vault.
pub mod gdtf_share;
/// Built-in generic fixtures available offline.
pub mod generic;
/// GDTF (DIN SPEC 15800) archive parser.
pub mod parser;

pub use commands::*;
pub use gdtf_share::*;
pub use generic::*;
pub use parser::*;
