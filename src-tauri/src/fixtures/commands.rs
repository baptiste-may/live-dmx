use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, RwLock};
use tauri::{AppHandle, Manager};

use super::gdtf_share::{
    delete_stored_credentials, load_stored_credentials, save_stored_credentials, GdtfCatalogItem,
    GdtfCatalogManager, GdtfCatalogMode, GdtfShareClient,
};
use super::parser::ParsedGdtfMode;

static GDTF_CATALOG: OnceLock<RwLock<GdtfCatalogManager>> = OnceLock::new();
static GDTF_CLIENT: OnceLock<GdtfShareClient> = OnceLock::new();
static GDTF_CREDS: Mutex<Option<(String, String)>> = Mutex::new(None);

fn get_app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {:?}", e))
}

fn ensure_logged_in(app: &AppHandle, client: &GdtfShareClient) -> bool {
    if client.is_logged_in() {
        return true;
    }

    if let Ok(app_data) = get_app_data_dir(app) {
        let mut creds_lock = match GDTF_CREDS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if creds_lock.is_none() {
            if let Ok(Some(loaded)) = load_stored_credentials(&app_data) {
                *creds_lock = Some(loaded);
            }
        }
    }

    let creds_opt = {
        let creds_lock = match GDTF_CREDS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        creds_lock.clone()
    };

    if let Some((ref u, ref p)) = creds_opt {
        if client.login(u, p).is_ok() {
            return true;
        }
    }

    client.is_logged_in()
}

fn get_catalog_manager(app: &AppHandle) -> Result<&'static RwLock<GdtfCatalogManager>, String> {
    if let Some(mgr) = GDTF_CATALOG.get() {
        return Ok(mgr);
    }

    let app_data = get_app_data_dir(app)?;
    let manager = GdtfCatalogManager::new(&app_data);
    let _ = GDTF_CATALOG.set(RwLock::new(manager));

    GDTF_CATALOG
        .get()
        .ok_or_else(|| "Failed to initialize GDTF catalog manager".to_string())
}

fn get_client() -> &'static GdtfShareClient {
    GDTF_CLIENT.get_or_init(GdtfShareClient::new)
}

#[tauri::command]
pub async fn check_fixtures_installed(_app: AppHandle) -> Result<bool, String> {
    // Built-in generic fixtures are always available immediately
    Ok(true)
}

#[tauri::command]
pub async fn gdtf_search(
    app: AppHandle,
    query: Option<String>,
) -> Result<Vec<GdtfCatalogItem>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let manager_lock = get_catalog_manager(&app)?;
        let manager = manager_lock.read().map_err(|e| e.to_string())?;
        Ok(manager.search(query.as_deref()))
    })
    .await
    .map_err(|e| format!("Search task failed: {:?}", e))?
}

#[tauri::command]
pub async fn gdtf_set_credentials(
    app: AppHandle,
    user: String,
    pass: String,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let client = get_client();
        client.login(&user, &pass)?;

        let app_data = get_app_data_dir(&app)?;
        save_stored_credentials(&app_data, &user, &pass)?;

        let mut creds = match GDTF_CREDS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *creds = Some((user, pass));

        Ok(true)
    })
    .await
    .map_err(|e| format!("Login task failed: {:?}", e))?
}

#[tauri::command]
pub async fn gdtf_get_credentials_status(app: AppHandle) -> Result<bool, String> {
    let client = get_client();
    if client.is_logged_in() {
        return Ok(true);
    }

    let app_data = get_app_data_dir(&app)?;
    let creds_opt = {
        let mut creds_lock = match GDTF_CREDS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if creds_lock.is_none() {
            if let Ok(Some(loaded)) = load_stored_credentials(&app_data) {
                *creds_lock = Some(loaded);
            }
        }
        creds_lock.clone()
    };

    if let Some((u, p)) = creds_opt {
        tauri::async_runtime::spawn_blocking(move || {
            let client = get_client();
            client.login(&u, &p).is_ok()
        })
        .await
        .map_err(|e| format!("Credential check task failed: {:?}", e))
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn gdtf_logout(app: AppHandle) -> Result<bool, String> {
    let app_data = get_app_data_dir(&app)?;
    let _ = delete_stored_credentials(&app_data);

    let mut creds = match GDTF_CREDS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *creds = None;
    get_client().logout();
    Ok(true)
}

#[tauri::command]
pub async fn gdtf_sync_catalog(app: AppHandle) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let client = get_client();
        if !ensure_logged_in(&app, client) {
            return Err(
                "GDTF-Share authentication required. Please connect your free GDTF-Share account."
                    .to_string(),
            );
        }

        let items = client.fetch_catalog_list()?;
        let count = items.len();

        let manager_lock = get_catalog_manager(&app)?;
        let manager = manager_lock.write().map_err(|e| e.to_string())?;
        let _ = manager.load_local_fixtures();
        manager.save_online_catalog(&items)?;

        Ok(count)
    })
    .await
    .map_err(|e| format!("Catalog sync task failed: {:?}", e))?
}

#[tauri::command]
pub async fn gdtf_fetch_mode(
    app: AppHandle,
    rid: Option<u32>,
    manufacturer: String,
    fixture: String,
    mode_name: String,
    persist: Option<bool>,
) -> Result<ParsedGdtfMode, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let manager_lock = get_catalog_manager(&app)?;
        let trimmed_mode = mode_name.trim();
        let should_persist = persist.unwrap_or(true);

        // 1. Check if already available locally or generic
        {
            let manager = manager_lock.read().map_err(|e| e.to_string())?;
            if let Some(parsed) = manager.get_local_fixture(&manufacturer, &fixture) {
                if let Some(m) = parsed
                    .modes
                    .into_iter()
                    .find(|m| m.name.trim().eq_ignore_ascii_case(trimmed_mode))
                {
                    return Ok(m);
                }
            }
        }

        // 2. Check preview cache
        if should_persist {
            let manager = manager_lock.write().map_err(|e| e.to_string())?;
            if let Ok(Some(parsed)) = manager.persist_preview_fixture(&manufacturer, &fixture) {
                if let Some(m) = parsed
                    .modes
                    .into_iter()
                    .find(|m| m.name.trim().eq_ignore_ascii_case(trimmed_mode))
                {
                    return Ok(m);
                }
            }
        } else {
            let manager = manager_lock.read().map_err(|e| e.to_string())?;
            if let Some(parsed) = manager.get_preview_fixture(&manufacturer, &fixture) {
                if let Some(m) = parsed
                    .modes
                    .into_iter()
                    .find(|m| m.name.trim().eq_ignore_ascii_case(trimmed_mode))
                {
                    return Ok(m);
                }
            }
        }

        // 3. If not in local or preview cache and RID is provided, download from GDTF-Share
        let rid_val = rid.ok_or_else(|| {
            format!(
                "Fixture '{}' by '{}' is not installed locally and has no GDTF-Share RID",
                fixture, manufacturer
            )
        })?;

        let client = get_client();
        if !ensure_logged_in(&app, client) {
            return Err(
                "GDTF-Share authentication required. Please connect your free GDTF-Share account."
                    .to_string(),
            );
        }

        let bytes = client.download_gdtf_file(rid_val)?;

        let manager = manager_lock.write().map_err(|e| e.to_string())?;
        let parsed = if should_persist {
            manager.save_downloaded_gdtf(&manufacturer, &fixture, &bytes)?
        } else {
            manager.save_preview_gdtf(&manufacturer, &fixture, &bytes)?
        };

        if let Some(m) = parsed
            .modes
            .into_iter()
            .find(|m| m.name.trim().eq_ignore_ascii_case(trimmed_mode))
        {
            Ok(m)
        } else {
            Err(format!(
                "Mode '{}' not found in downloaded GDTF fixture '{}'",
                mode_name, fixture
            ))
        }
    })
    .await
    .map_err(|e| format!("Fetch mode task failed: {:?}", e))?
}

#[tauri::command]
pub async fn gdtf_import_file(app: AppHandle, path: String) -> Result<GdtfCatalogItem, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bytes =
            fs::read(&path).map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;
        let manager_lock = get_catalog_manager(&app)?;
        let manager = manager_lock.write().map_err(|e| e.to_string())?;

        let parsed = super::parser::parse_gdtf_archive(&bytes)?;
        let saved = manager.save_downloaded_gdtf(&parsed.manufacturer, &parsed.name, &bytes)?;

        Ok(GdtfCatalogItem {
            rid: None,
            fixture: saved.name,
            manufacturer: saved.manufacturer,
            revision: None,
            uploader: Some("Imported".to_string()),
            rating: Some(5.0),
            modes: saved
                .modes
                .into_iter()
                .map(|m| GdtfCatalogMode {
                    name: m.name,
                    dmxfootprint: m.channel_count as u32,
                    channels: Some(m.channels),
                })
                .collect(),
            uuid: saved.fixture_type_id,
            filesize: Some(bytes.len() as u64),
            is_installed: true,
            is_generic: false,
        })
    })
    .await
    .map_err(|e| format!("Import task failed: {:?}", e))?
}

#[tauri::command]
pub async fn reload_fixtures(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let manager_lock = get_catalog_manager(&app)?;
        let manager = manager_lock.write().map_err(|e| e.to_string())?;
        manager.load_local_fixtures()
    })
    .await
    .map_err(|e| format!("Reload task failed: {:?}", e))?
}
