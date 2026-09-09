use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GdtfCatalogMode {
    pub name: String,
    pub dmxfootprint: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GdtfCatalogItem {
    pub rid: Option<u32>,
    pub fixture: String,
    pub manufacturer: String,
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(default)]
    pub uploader: Option<String>,
    #[serde(default)]
    pub rating: Option<f32>,
    pub modes: Vec<GdtfCatalogMode>,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub filesize: Option<u64>,
    #[serde(default)]
    pub is_installed: bool,
    #[serde(default)]
    pub is_generic: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdtfShareCredentials {
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdtfLoginResponse {
    pub result: bool,
    pub error: Option<String>,
    pub notice: Option<String>,
}
