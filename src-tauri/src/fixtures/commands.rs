use std::sync::Mutex;

use super::loader::{filter_and_sort_fixtures, load_fixtures_sync, CACHED_FIXTURES};
use super::types::Fixtures;

static FIXTURE_LOAD_LOCK: Mutex<()> = Mutex::new(());

#[tauri::command]
pub async fn get_fixtures(
    app: tauri::AppHandle,
    query: Option<String>,
) -> Result<Fixtures, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(cached) = CACHED_FIXTURES.get() {
            return Ok(filter_and_sort_fixtures(cached, query.as_deref()));
        }

        let _guard = FIXTURE_LOAD_LOCK.lock().map_err(|e| e.to_string())?;
        let fixtures = if let Some(cached) = CACHED_FIXTURES.get() {
            cached
        } else {
            let loaded = load_fixtures_sync(&app)?;
            let _ = CACHED_FIXTURES.set(loaded);
            CACHED_FIXTURES.get().unwrap()
        };

        Ok(filter_and_sort_fixtures(fixtures, query.as_deref()))
    })
    .await
    .map_err(|e| format!("Failed to run background task: {:?}", e))?
}
