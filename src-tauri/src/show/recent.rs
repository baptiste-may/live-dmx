use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentShow {
    pub display_name: String,
    pub path: String,
    pub updated_at: String,
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
    serde_json::from_str::<Vec<RecentShow>>(&data)
        .map_err(|e| format!("Failed to parse recent shows file: {}", e))
}

pub fn write_recent_shows_to_path(file_path: &PathBuf, shows: &[RecentShow]) -> Result<(), String> {
    let json_str = serde_json::to_string_pretty(shows)
        .map_err(|e| format!("Failed to serialize recent shows: {}", e))?;
    fs::write(file_path, json_str)
        .map_err(|e| format!("Failed to write recent shows file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_recent_shows(app: AppHandle) -> Result<Vec<RecentShow>, String> {
    let file_path = get_recent_shows_file_path(&app)?;
    read_recent_shows_from_path(&file_path)
}

#[tauri::command]
pub fn save_recent_show(
    app: AppHandle,
    display_name: String,
    path: String,
) -> Result<Vec<RecentShow>, String> {
    let file_path = get_recent_shows_file_path(&app)?;
    let mut shows = read_recent_shows_from_path(&file_path).unwrap_or_default();

    shows.retain(|s| s.path != path);
    shows.insert(
        0,
        RecentShow {
            display_name,
            path,
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis().to_string())
                .unwrap_or_default(),
        },
    );

    if shows.len() > 20 {
        shows.truncate(20);
    }

    write_recent_shows_to_path(&file_path, &shows)?;
    Ok(shows)
}

#[tauri::command]
pub fn remove_recent_show(app: AppHandle, path: String) -> Result<Vec<RecentShow>, String> {
    let file_path = get_recent_shows_file_path(&app)?;
    let mut shows = read_recent_shows_from_path(&file_path).unwrap_or_default();

    shows.retain(|s| s.path != path);
    write_recent_shows_to_path(&file_path, &shows)?;
    Ok(shows)
}
