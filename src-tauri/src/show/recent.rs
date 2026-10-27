use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub const MAX_RECENT_SHOWS: usize = 5;

/// Wire shape returned to the frontend. `available` is computed on every read
/// and must never reach `recent_shows.json`, so storage uses `StoredRecentShow`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentShow {
    pub display_name: String,
    pub path: String,
    pub updated_at: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredRecentShow {
    display_name: String,
    path: String,
    updated_at: String,
}

impl From<&RecentShow> for StoredRecentShow {
    fn from(show: &RecentShow) -> Self {
        Self {
            display_name: show.display_name.clone(),
            path: show.path.clone(),
            updated_at: show.updated_at.clone(),
        }
    }
}

impl From<StoredRecentShow> for RecentShow {
    fn from(stored: StoredRecentShow) -> Self {
        Self {
            display_name: stored.display_name,
            path: stored.path,
            updated_at: stored.updated_at,
            available: false,
        }
    }
}

pub fn get_recent_shows_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {:?}", e))?;

    if !app_data.exists() {
        fs::create_dir_all(&app_data)
            .map_err(|e| format!("Failed to create app data dir: {:?}", e))?;
    }

    Ok(app_data.join("recent_shows.json"))
}

pub fn read_recent_shows_from_path(file_path: &PathBuf) -> Result<Vec<RecentShow>, String> {
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read recent shows file: {}", e))?;
    let stored = serde_json::from_str::<Vec<StoredRecentShow>>(&data)
        .map_err(|e| format!("Failed to parse recent shows file: {}", e))?;
    Ok(stored.into_iter().map(RecentShow::from).collect())
}

pub fn write_recent_shows_to_path(file_path: &PathBuf, shows: &[RecentShow]) -> Result<(), String> {
    let stored: Vec<StoredRecentShow> = shows.iter().map(StoredRecentShow::from).collect();
    let json_str = serde_json::to_string_pretty(&stored)
        .map_err(|e| format!("Failed to serialize recent shows: {}", e))?;
    fs::write(file_path, json_str)
        .map_err(|e| format!("Failed to write recent shows file: {}", e))?;
    Ok(())
}

/// Recomputes each entry's `available` flag from the filesystem.
pub fn apply_availability(shows: &mut [RecentShow]) {
    for show in shows {
        show.available = Path::new(&show.path).is_file();
    }
}

/// Inserts an entry at the top, deduplicating by path and enforcing the cap.
pub fn insert_recent_show(shows: &mut Vec<RecentShow>, entry: RecentShow) {
    shows.retain(|show| show.path != entry.path);
    shows.insert(0, entry);
    shows.truncate(MAX_RECENT_SHOWS);
}

/// Removes an entry by path and enforces the cap.
pub fn remove_recent_show_by_path(shows: &mut Vec<RecentShow>, path: &str) {
    shows.retain(|show| show.path != path);
    shows.truncate(MAX_RECENT_SHOWS);
}

/// Removes entries whose file no longer exists, recomputing availability.
pub fn remove_unavailable_recent_shows(shows: &mut Vec<RecentShow>) {
    shows.retain(|show| Path::new(&show.path).is_file());
    apply_availability(shows);
}

/// Limits a stored list to the newest entries without rewriting the file.
pub fn limit_recent_shows(shows: Vec<RecentShow>) -> Vec<RecentShow> {
    shows.into_iter().take(MAX_RECENT_SHOWS).collect()
}

#[tauri::command]
pub fn get_recent_shows(app: AppHandle) -> Result<Vec<RecentShow>, String> {
    let file_path = get_recent_shows_file_path(&app)?;
    let mut shows = limit_recent_shows(read_recent_shows_from_path(&file_path)?);
    apply_availability(&mut shows);
    Ok(shows)
}

#[tauri::command]
pub fn save_recent_show(
    app: AppHandle,
    display_name: String,
    path: String,
) -> Result<Vec<RecentShow>, String> {
    let file_path = get_recent_shows_file_path(&app)?;
    let mut shows = read_recent_shows_from_path(&file_path).unwrap_or_default();

    insert_recent_show(
        &mut shows,
        RecentShow {
            display_name,
            path,
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis().to_string())
                .unwrap_or_default(),
            available: false,
        },
    );

    write_recent_shows_to_path(&file_path, &shows)?;
    apply_availability(&mut shows);
    Ok(shows)
}

#[tauri::command]
pub fn remove_recent_show(app: AppHandle, path: String) -> Result<Vec<RecentShow>, String> {
    let file_path = get_recent_shows_file_path(&app)?;
    let mut shows = read_recent_shows_from_path(&file_path).unwrap_or_default();

    remove_recent_show_by_path(&mut shows, &path);
    write_recent_shows_to_path(&file_path, &shows)?;
    apply_availability(&mut shows);
    Ok(shows)
}

#[tauri::command]
pub fn clean_recent_shows(app: AppHandle) -> Result<Vec<RecentShow>, String> {
    let file_path = get_recent_shows_file_path(&app)?;
    let mut shows = read_recent_shows_from_path(&file_path).unwrap_or_default();

    remove_unavailable_recent_shows(&mut shows);
    shows.truncate(MAX_RECENT_SHOWS);
    write_recent_shows_to_path(&file_path, &shows)?;
    Ok(shows)
}
