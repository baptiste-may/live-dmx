//! Serializable types shared by the GDTF-Share client, the catalog and the
//! frontend fixture browser.
//!
//! See `docs/contributing/fixtures.md` (GDTF-Share catalog section).

use serde::{Deserialize, Serialize};

/// A DMX mode of a catalog fixture, as shown in the fixture browser.
///
/// Serialized in camelCase; `channels` and the bounds are omitted when unknown
/// (e.g. for online entries that were never downloaded).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GdtfCatalogMode {
    /// Mode name, unique within its fixture.
    pub name: String,
    /// Number of DMX channels the mode occupies (its footprint).
    pub dmxfootprint: u32,
    /// Human-readable channel names, index 0 = DMX offset 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    /// Pan range in degrees (e.g. `[-270, 270]`), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan_bounds: Option<[i32; 2]>,
    /// Tilt range in degrees (e.g. `[-135, 135]`), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tilt_bounds: Option<[i32; 2]>,
}

/// A fixture entry in the catalog: a built-in generic, a locally installed
/// GDTF file or an online GDTF-Share listing.
///
/// Serialized in camelCase for the frontend and for the persisted online index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GdtfCatalogItem {
    /// GDTF-Share revision id used to download the file; `None` for generic
    /// and locally imported fixtures.
    pub rid: Option<u32>,
    /// Fixture name.
    pub fixture: String,
    /// Manufacturer name (`"Generic"` for built-in generics).
    pub manufacturer: String,
    /// Revision label published on GDTF-Share.
    #[serde(default)]
    pub revision: Option<String>,
    /// Uploader name; `"Installed"` for local files and `"Imported"` for
    /// manually imported ones.
    #[serde(default)]
    pub uploader: Option<String>,
    /// Community rating as reported by GDTF-Share (used to rank duplicates).
    #[serde(default)]
    pub rating: Option<f32>,
    /// Available DMX modes.
    pub modes: Vec<GdtfCatalogMode>,
    /// GDTF fixture type UUID.
    #[serde(default)]
    pub uuid: Option<String>,
    /// Archive size in bytes.
    #[serde(default)]
    pub filesize: Option<u64>,
    /// Whether the `.gdtf` file is available locally (or built in).
    #[serde(default)]
    pub is_installed: bool,
    /// Whether this is a built-in generic fixture.
    #[serde(default)]
    pub is_generic: bool,
    /// DIN SPEC 15800 categories (e.g. `"Moving Head"`), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
}

/// GDTF-Share account credentials.
///
/// Serialized as JSON and stored only inside the encrypted vault (see
/// `auth.rs`); never log or persist elsewhere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdtfShareCredentials {
    /// GDTF-Share account user name.
    pub user: String,
    /// GDTF-Share account password.
    pub password: String,
}

/// Response body of the GDTF-Share login endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdtfLoginResponse {
    /// `true` when the login succeeded.
    pub result: bool,
    /// Server-provided error message when `result` is `false`.
    pub error: Option<String>,
    /// Optional informational message from the server.
    pub notice: Option<String>,
}
