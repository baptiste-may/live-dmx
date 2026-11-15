//! Tauri commands for the fixture catalog: search, GDTF-Share login and sync,
//! mode fetching and file import.
//!
//! They operate on process-wide statics (catalog manager, client, credential
//! cache), never on `AppState`, so they emit no events and create no undo
//! entries. See `docs/contributing/fixtures.md`.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, RwLock};
use tauri::{AppHandle, Manager};

use super::gdtf_share::{
    delete_stored_credentials, load_stored_credentials, save_stored_credentials, GdtfCatalogItem,
    GdtfCatalogManager, GdtfCatalogMode, GdtfShareClient,
};
use super::parser::ParsedGdtfMode;

/// Process-wide catalog manager, created lazily on first use.
static GDTF_CATALOG: OnceLock<RwLock<GdtfCatalogManager>> = OnceLock::new();
/// Process-wide GDTF-Share HTTP client.
static GDTF_CLIENT: OnceLock<GdtfShareClient> = OnceLock::new();
/// In-memory cache of the `(user, password)` credentials loaded from the vault
/// or entered by the user; `None` until loaded or after logout.
static GDTF_CREDS: Mutex<Option<(String, String)>> = Mutex::new(None);

/// Resolves the Tauri app data directory (catalog index, fixtures, vault).
fn get_app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {:?}", e))
}

/// Resolves the Tauri app local data directory (fallback vault key file).
fn get_app_local_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {:?}", e))
}

/// Makes sure the client has a GDTF-Share session, logging in with the cached or
/// vaulted credentials if needed.
///
/// Returns `false` when no valid session could be established. Recovers from a
/// poisoned credentials mutex.
fn ensure_logged_in(app: &AppHandle, client: &GdtfShareClient) -> bool {
    if client.is_logged_in() {
        return true;
    }

    if let (Ok(app_data), Ok(local_dir)) = (get_app_data_dir(app), get_app_local_data_dir(app)) {
        let mut creds_lock = match GDTF_CREDS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if creds_lock.is_none() {
            if let Ok(Some(loaded)) = load_stored_credentials(&app_data, &local_dir) {
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

/// Returns the shared catalog manager, creating it (and loading the local cache)
/// on first call.
///
/// Errors when the app data dir cannot be resolved.
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

/// Returns the shared GDTF-Share client, creating it on first call.
fn get_client() -> &'static GdtfShareClient {
    GDTF_CLIENT.get_or_init(GdtfShareClient::new)
}

/// Tauri command: reports whether fixtures are available.
///
/// Always `true` because the built-in generic fixtures need no download. Does
/// not touch `AppState` or emit events.
///
/// # Errors
///
/// Never fails; the `Result` is part of the command signature.
#[tauri::command]
pub async fn check_fixtures_installed(_app: AppHandle) -> Result<bool, String> {
    // Built-in generic fixtures are always available immediately
    Ok(true)
}

/// Tauri command: searches the fixture catalog (generics, local files, online index).
///
/// A `None`/empty query lists only generics and installed fixtures. Read-only:
/// does not touch `AppState`, emit events or create undo entries; runs on a
/// blocking thread.
///
/// # Errors
///
/// Returns an error when the catalog cannot be initialised, its lock is
/// poisoned, or the blocking task fails.
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

/// Tauri command: logs in to GDTF-Share and saves the credentials.
///
/// The credentials are validated with a real login first, then stored in the
/// encrypted vault and cached in memory. Does not touch `AppState`, emit events
/// or create undo entries.
///
/// # Errors
///
/// Returns an error when the login fails, the data dirs cannot be resolved, the
/// vault write fails, or the blocking task fails.
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
        let local_dir = get_app_local_data_dir(&app)?;
        save_stored_credentials(&app_data, &local_dir, &user, &pass)?;

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

/// Tauri command: reports whether GDTF-Share access is available.
///
/// Returns `true` if already logged in; otherwise loads vaulted credentials (if
/// any) and tries to log in with them. `false` means no stored credentials or a
/// failed login.
///
/// # Errors
///
/// Returns an error when the data dirs cannot be resolved or the blocking task
/// fails. Vault read errors are ignored and yield `false`.
#[tauri::command]
pub async fn gdtf_get_credentials_status(app: AppHandle) -> Result<bool, String> {
    let client = get_client();
    if client.is_logged_in() {
        return Ok(true);
    }

    let app_data = get_app_data_dir(&app)?;
    let local_dir = get_app_local_data_dir(&app)?;
    let creds_opt = {
        let mut creds_lock = match GDTF_CREDS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if creds_lock.is_none() {
            if let Ok(Some(loaded)) = load_stored_credentials(&app_data, &local_dir) {
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

/// Tauri command: forgets the GDTF-Share account.
///
/// Deletes the vaulted credentials, clears the in-memory cache and logs the
/// client out. Does not touch `AppState` or emit events.
///
/// # Errors
///
/// Returns an error when the data dirs cannot be resolved or the vault deletion
/// fails.
#[tauri::command]
pub async fn gdtf_logout(app: AppHandle) -> Result<bool, String> {
    let app_data = get_app_data_dir(&app)?;
    let local_dir = get_app_local_data_dir(&app)?;
    delete_stored_credentials(&app_data, &local_dir)?;

    let mut creds = match GDTF_CREDS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *creds = None;
    get_client().logout();
    Ok(true)
}

/// Tauri command: refreshes the online catalog index from GDTF-Share.
///
/// Requires a session (re-using vaulted credentials if needed). Rescans local
/// fixtures and persists the downloaded index to disk. Returns the number of
/// online entries fetched. Does not touch `AppState` or emit events.
///
/// # Errors
///
/// Returns an error when authentication is unavailable, the list cannot be
/// fetched, the catalog lock is poisoned, the index cannot be saved, or the
/// blocking task fails.
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

/// Tauri command: returns the parsed DMX mode of a fixture, downloading it if needed.
///
/// Lookup order: installed fixture (including generics), then the in-memory
/// preview cache, then a GDTF-Share download by `rid` (requires a session).
/// Mode names are matched trimmed and case-insensitively. With `persist`
/// (default `true`) the archive is saved to the local fixture cache; otherwise
/// it is only kept in the preview cache. Does not touch `AppState`, emit events
/// or create undo entries.
///
/// # Errors
///
/// Returns an error when the fixture is not local and has no `rid`, when
/// authentication or the download fails, when the file cannot be parsed or
/// saved, when the mode does not exist in a downloaded fixture, when a catalog
/// lock is poisoned, or when the blocking task fails.
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

/// Tauri command: imports a `.gdtf` file from disk into the local fixture cache.
///
/// The returned item is marked installed, uploaded by `"Imported"` and has no
/// `rid`. Does not touch `AppState`, emit events or create undo entries.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed, cannot be saved in
/// the cache, the catalog lock is poisoned, or the blocking task fails.
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
                    pan_bounds: m.pan_bounds,
                    tilt_bounds: m.tilt_bounds,
                })
                .collect(),
            uuid: saved.fixture_type_id.clone(),
            filesize: Some(bytes.len() as u64),
            is_installed: true,
            is_generic: false,
            categories: saved.categories.clone(),
        })
    })
    .await
    .map_err(|e| format!("Import task failed: {:?}", e))?
}

/// Tauri command: rescans the local fixture cache from disk.
///
/// Does not touch `AppState` or emit events.
///
/// # Errors
///
/// Returns an error when the catalog cannot be initialised, its lock is
/// poisoned, the scan fails, or the blocking task fails.
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
