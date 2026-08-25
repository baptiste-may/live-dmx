use std::sync::Mutex;

use super::loader::{
    check_fixtures_installed_sync, download_fixtures_sync, filter_and_sort_fixtures,
    load_fixtures_sync, CACHED_FIXTURES,
};
use super::types::Fixtures;

static FIXTURE_LOAD_LOCK: Mutex<()> = Mutex::new(());

#[tauri::command]
pub async fn check_fixtures_installed(app: tauri::AppHandle) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || check_fixtures_installed_sync(&app))
        .await
        .map_err(|e| format!("Failed to run background task: {:?}", e))?
}

#[tauri::command]
pub async fn download_fixtures(app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || download_fixtures_sync(&app))
        .await
        .map_err(|e| format!("Failed to run background task: {:?}", e))?
}

#[tauri::command]
pub async fn get_fixtures(
    app: tauri::AppHandle,
    query: Option<String>,
) -> Result<Fixtures, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if let Ok(guard) = CACHED_FIXTURES.read() {
            if let Some(cached) = guard.as_ref() {
                return Ok(filter_and_sort_fixtures(cached, query.as_deref()));
            }
        }

        let _guard = FIXTURE_LOAD_LOCK.lock().map_err(|e| e.to_string())?;

        if let Ok(guard) = CACHED_FIXTURES.read() {
            if let Some(cached) = guard.as_ref() {
                return Ok(filter_and_sort_fixtures(cached, query.as_deref()));
            }
        }

        let loaded = load_fixtures_sync(&app)?;
        let mut write_guard = CACHED_FIXTURES.write().map_err(|e| e.to_string())?;
        *write_guard = Some(loaded);

        let fixtures = write_guard.as_ref().unwrap();
        Ok(filter_and_sort_fixtures(fixtures, query.as_deref()))
    })
    .await
    .map_err(|e| format!("Failed to run background task: {:?}", e))?
}
