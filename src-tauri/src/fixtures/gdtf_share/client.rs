use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use ureq::Agent;

use super::types::{GdtfCatalogItem, GdtfCatalogMode, GdtfLoginResponse};

pub struct GdtfShareClient {
    agent: Arc<Mutex<Agent>>,
    is_authenticated: Arc<Mutex<bool>>,
}

impl Default for GdtfShareClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GdtfShareClient {
    pub fn new() -> Self {
        let agent = ureq::builder()
            .timeout_connect(Duration::from_secs(15))
            .timeout_read(Duration::from_secs(60))
            .build();

        Self {
            agent: Arc::new(Mutex::new(agent)),
            is_authenticated: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_logged_in(&self) -> bool {
        *self
            .is_authenticated
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    pub fn login(&self, user: &str, pass: &str) -> Result<(), String> {
        let agent = self.agent.lock().map_err(|e| e.to_string())?.clone();
        let payload = serde_json::json!({
            "user": user,
            "password": pass,
        });

        let response = agent
            .post("https://gdtf-share.com/apis/public/login.php")
            .set("Content-Type", "application/json")
            .send_json(payload)
            .map_err(|e| format!("Login request failed: {}", e))?;

        let login_res: GdtfLoginResponse = response
            .into_json()
            .map_err(|e| format!("Failed to parse login response: {}", e))?;

        if login_res.result {
            let mut auth = self
                .is_authenticated
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            *auth = true;
            Ok(())
        } else {
            Err(login_res
                .error
                .unwrap_or_else(|| "GDTF-Share login failed".to_string()))
        }
    }

    pub fn logout(&self) {
        let mut auth = self
            .is_authenticated
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *auth = false;
        let mut agent_lock = self.agent.lock().unwrap_or_else(|e| e.into_inner());
        *agent_lock = ureq::builder()
            .timeout_connect(Duration::from_secs(15))
            .timeout_read(Duration::from_secs(60))
            .build();
    }

    pub fn fetch_catalog_list(&self) -> Result<Vec<GdtfCatalogItem>, String> {
        let agent = self.agent.lock().map_err(|e| e.to_string())?.clone();
        let response = agent
            .get("https://gdtf-share.com/apis/public/getList.php")
            .call()
            .map_err(|e| format!("Failed to fetch GDTF-Share catalog: {}", e))?;

        let raw_val: Value = response
            .into_json()
            .map_err(|e| format!("Failed to parse catalog JSON: {}", e))?;

        let list_array = raw_val
            .get("list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "GDTF-Share getList response missing 'list' array".to_string())?;

        let mut items = Vec::with_capacity(list_array.len());
        for item in list_array {
            let rid = item.get("rid").and_then(|v| v.as_u64()).map(|v| v as u32);
            let fixture = item
                .get("fixture")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let manufacturer = item
                .get("manufacturer")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if fixture.is_empty() || manufacturer.is_empty() {
                continue;
            }

            let revision = item
                .get("revision")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let uploader = item
                .get("uploader")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let rating = item.get("rating").and_then(|v| {
                if let Some(n) = v.as_f64() {
                    Some(n as f32)
                } else if let Some(s) = v.as_str() {
                    s.parse::<f32>().ok()
                } else {
                    None
                }
            });
            let uuid = item
                .get("uuid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let filesize = item.get("filesize").and_then(|v| v.as_u64());

            let mut modes = Vec::new();
            if let Some(modes_arr) = item.get("modes").and_then(|v| v.as_array()) {
                for m in modes_arr {
                    if let Some(mode_obj) = m.as_object() {
                        let name = mode_obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Default")
                            .to_string();
                        let dmxfootprint = mode_obj
                            .get("dmxfootprint")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32)
                            .unwrap_or(1);
                        modes.push(GdtfCatalogMode {
                            name,
                            dmxfootprint,
                            channels: None,
                            pan_bounds: None,
                            tilt_bounds: None,
                        });
                    }
                }
            }

            items.push(GdtfCatalogItem {
                rid,
                fixture,
                manufacturer,
                revision,
                uploader,
                rating,
                modes,
                uuid,
                filesize,
                is_installed: false,
                is_generic: false,
                categories: None,
            });
        }

        Ok(items)
    }

    pub fn download_gdtf_file(&self, rid: u32) -> Result<Vec<u8>, String> {
        let agent = self.agent.lock().map_err(|e| e.to_string())?.clone();
        let url = format!(
            "https://gdtf-share.com/apis/public/downloadFile.php?rid={}",
            rid
        );

        let response = agent
            .get(&url)
            .call()
            .map_err(|e| format!("Failed to download GDTF file (rid {}): {}", rid, e))?;

        let mut bytes = Vec::new();
        let mut reader = response.into_reader();
        std::io::copy(&mut reader, &mut bytes)
            .map_err(|e| format!("Failed to read downloaded GDTF file: {}", e))?;

        Ok(bytes)
    }
}
