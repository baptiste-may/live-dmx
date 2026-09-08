use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::types::GdtfShareCredentials;

pub fn get_auth_file_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("gdtf_auth.json")
}

pub fn load_stored_credentials(app_data_dir: &Path) -> Result<Option<(String, String)>, String> {
    let path = get_auth_file_path(app_data_dir);
    if !path.exists() {
        return Ok(None);
    }

    let data = match fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) => return Err(format!("Failed to read GDTF auth file: {}", e)),
    };

    match serde_json::from_str::<GdtfShareCredentials>(&data) {
        Ok(creds) => Ok(Some((creds.user, creds.password))),
        Err(_) => {
            // Corrupted or invalid format, remove to avoid repeating errors
            let _ = fs::remove_file(&path);
            Ok(None)
        }
    }
}

pub fn save_stored_credentials(app_data_dir: &Path, user: &str, pass: &str) -> Result<(), String> {
    if !app_data_dir.exists() {
        fs::create_dir_all(app_data_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    let path = get_auth_file_path(app_data_dir);
    let creds = GdtfShareCredentials {
        user: user.to_string(),
        password: pass.to_string(),
    };

    let json_str = serde_json::to_string_pretty(&creds)
        .map_err(|e| format!("Failed to serialize GDTF credentials: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .map_err(|e| {
                format!(
                    "Failed to create GDTF auth file with restricted permissions: {}",
                    e
                )
            })?;
        file.write_all(json_str.as_bytes())
            .map_err(|e| format!("Failed to write GDTF credentials: {}", e))?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&path, json_str)
            .map_err(|e| format!("Failed to save GDTF credentials: {}", e))?;
    }

    Ok(())
}

pub fn delete_stored_credentials(app_data_dir: &Path) -> Result<(), String> {
    let path = get_auth_file_path(app_data_dir);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete GDTF auth file: {}", e))?;
    }
    Ok(())
}
