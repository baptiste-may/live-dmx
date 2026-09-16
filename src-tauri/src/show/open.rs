use std::path::Path;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};
use url::Url;

pub struct PendingOpenStore {
    pub path: Mutex<Option<String>>,
}

impl Default for PendingOpenStore {
    fn default() -> Self {
        Self {
            path: Mutex::new(None),
        }
    }
}

fn is_ldmx_str(candidate: &str) -> bool {
    !candidate.starts_with('-') && candidate.to_ascii_lowercase().ends_with(".ldmx")
}

fn is_ldmx_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("ldmx"))
        .unwrap_or(false)
}

fn is_windows_drive_path(candidate: &str) -> bool {
    let bytes = candidate.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

/// Extracts a `.ldmx` show file path from a raw candidate: a plain filesystem
/// path, a `file://` URL or a `live-dmx://open?path=...` deep link.
/// Non-`.ldmx` candidates are rejected so random JSON/archive files are never
/// opened as shows.
pub fn extract_show_path(candidate: &str) -> Option<String> {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return None;
    }

    if !is_windows_drive_path(trimmed) {
        if let Ok(url) = Url::parse(trimmed) {
            return match url.scheme() {
                "file" => {
                    let path = url.to_file_path().ok()?;
                    is_ldmx_path(&path).then(|| path.to_string_lossy().into_owned())
                }
                "live-dmx" => {
                    let path = url
                        .query_pairs()
                        .find(|(key, _)| key == "path")
                        .map(|(_, value)| value.into_owned())?;
                    is_ldmx_str(&path).then_some(path)
                }
                _ => None,
            };
        }
    }

    is_ldmx_str(trimmed).then(|| trimmed.to_string())
}

/// Stores a pending open request (read once by the frontend at startup) and
/// forwards it to the running frontend through the `open-show-request` event.
pub fn handle_open_request(app: &AppHandle, candidates: &[String]) {
    let Some(path) = candidates.iter().find_map(|c| extract_show_path(c)) else {
        return;
    };

    log::info!(target: "live-dmx::open", "open show request: {path}");

    if let Some(store) = app.try_state::<PendingOpenStore>() {
        if let Ok(mut pending) = store.path.lock() {
            *pending = Some(path.clone());
        }
    }

    let _ = app.emit("open-show-request", path);
}

#[tauri::command]
pub fn get_pending_open_path(state: tauri::State<PendingOpenStore>) -> Option<String> {
    state
        .path
        .lock()
        .ok()
        .and_then(|mut pending| pending.take())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_absolute_path() {
        assert_eq!(
            extract_show_path("/home/user/My Show.ldmx"),
            Some("/home/user/My Show.ldmx".to_string())
        );
    }

    #[test]
    fn accepts_relative_path() {
        assert_eq!(
            extract_show_path("shows/my-show.ldmx"),
            Some("shows/my-show.ldmx".to_string())
        );
    }

    #[test]
    fn accepts_windows_drive_path() {
        assert_eq!(
            extract_show_path("C:\\Shows\\show.ldmx"),
            Some("C:\\Shows\\show.ldmx".to_string())
        );
        assert_eq!(
            extract_show_path("C:/Shows/show.ldmx"),
            Some("C:/Shows/show.ldmx".to_string())
        );
    }

    #[test]
    fn accepts_file_url() {
        assert_eq!(
            extract_show_path("file:///home/user/show.ldmx"),
            Some("/home/user/show.ldmx".to_string())
        );
    }

    #[test]
    fn accepts_file_url_with_encoded_space() {
        assert_eq!(
            extract_show_path("file:///home/user/my%20show.ldmx"),
            Some("/home/user/my show.ldmx".to_string())
        );
    }

    #[test]
    fn accepts_deep_link_with_encoded_path() {
        assert_eq!(
            extract_show_path("live-dmx://open?path=%2Fhome%2Fuser%2Fshow.ldmx"),
            Some("/home/user/show.ldmx".to_string())
        );
    }

    #[test]
    fn accepts_deep_link_with_inline_path() {
        assert_eq!(
            extract_show_path("live-dmx://open?path=shows/show.ldmx"),
            Some("shows/show.ldmx".to_string())
        );
    }

    #[test]
    fn rejects_non_ldmx_extension() {
        assert_eq!(extract_show_path("/home/user/show.json"), None);
        assert_eq!(extract_show_path("/home/user/show.txt"), None);
    }

    #[test]
    fn rejects_flag_like_argument() {
        assert_eq!(extract_show_path("-show.ldmx"), None);
    }

    #[test]
    fn rejects_deep_link_pointing_to_non_ldmx() {
        assert_eq!(
            extract_show_path("live-dmx://open?path=%2Fhome%2Fuser%2Fshow.json"),
            None
        );
    }

    #[test]
    fn rejects_file_url_pointing_to_non_ldmx() {
        assert_eq!(extract_show_path("file:///home/user/show.json"), None);
    }

    #[test]
    fn rejects_unrelated_scheme() {
        assert_eq!(extract_show_path("https://example.com/show.ldmx"), None);
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(extract_show_path("  "), None);
    }
}
