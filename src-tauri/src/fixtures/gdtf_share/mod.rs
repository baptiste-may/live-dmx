//! GDTF-Share integration: HTTP client, local fixture catalog and the
//! encrypted credentials vault.
//!
//! See `docs/contributing/fixtures.md` (GDTF-Share catalog section).

/// Encrypted storage of the GDTF-Share credentials.
pub mod auth;
/// Local fixture cache, online index and catalog search.
pub mod catalog;
/// HTTP client for gdtf-share.com.
pub mod client;
/// Serializable catalog and credential types.
pub mod types;

pub use auth::*;
pub use catalog::GdtfCatalogManager;
pub use client::GdtfShareClient;
pub use types::*;
