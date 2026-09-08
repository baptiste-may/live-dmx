use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalInterface {
    pub id: String,
    pub name: String,
    pub device: String,
    pub port: String,
    pub status: String,
    pub fps: u32,
}
