//! Show state, undo/redo history, patch, groups, output routing and file handling.
//!
//! The submodules are re-exported flat so the rest of the crate can use
//! `crate::show::*`. See `docs/contributing/architecture.md` for the state and
//! mutation model and `docs/contributing/show-openers.md` for how shows are opened.

pub mod commands;
pub mod configure;
pub mod history;
pub mod identity;
pub mod mvr;
pub mod open;
pub mod output;
pub mod patch;
pub mod recent;
pub mod state;
pub mod types;

pub use commands::*;
pub use configure::*;
pub use history::*;
pub use identity::*;
pub use mvr::*;
pub use open::*;
pub use output::*;
pub use patch::*;
pub use recent::*;
pub use state::*;
pub use types::*;
