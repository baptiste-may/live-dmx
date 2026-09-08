use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read;
use std::sync::RwLock;
use tauri::{Emitter, Manager};

use super::types::{
    Fixture, FixtureDownloadProgress, FixtureLoadProgress, Fixtures, Manufacturer,
    ManufacturerParsed, RawFixture,
};

pub static CACHED_FIXTURES: RwLock<Option<Fixtures>> = RwLock::new(None);

pub fn check_fixtures_installed_sync(app: &tauri::AppHandle) -> Result<bool, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {:?}", e))?;
    let manufacturers_path = app_data.join("fixtures").join("manufacturers.json");
    Ok(manufacturers_path.exists())
}

pub fn download_and_extract_ofl_fixtures(
    app: Option<&tauri::AppHandle>,
    target_dir: &std::path::Path,
) -> Result<(), String> {
    const FIXTURES_URL: &str = "https://open-fixture-library.org/download.ofl";
    const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;

    if let Some(app) = app {
        let _ = app.emit(
            "fixture-download-progress",
            FixtureDownloadProgress {
                percentage: 0,
                status: "Connecting to Open Fixture Library...".to_string(),
            },
        );
    }

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout_read(std::time::Duration::from_secs(120))
        .build();

    let response = agent
        .get(FIXTURES_URL)
        .call()
        .map_err(|e| format!("Failed to download fixtures from OFL: {}", e))?;

    let content_length: Option<u64> = response
        .header("Content-Length")
        .and_then(|v| v.parse().ok());

    let mut reader = Read::take(response.into_reader(), MAX_ARCHIVE_BYTES);
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 64 * 1024];
    let mut downloaded_bytes = 0u64;

    loop {
        let bytes_read = reader
            .read(&mut chunk)
            .map_err(|e| format!("Failed to read downloaded fixtures zip: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
        downloaded_bytes += bytes_read as u64;

        if let Some(app) = app {
            let pct = if let Some(total) = content_length {
                if total > 0 {
                    ((downloaded_bytes as f64 / total as f64) * 50.0).round() as u8
                } else {
                    25
                }
            } else {
                25
            };
            let _ = app.emit(
                "fixture-download-progress",
                FixtureDownloadProgress {
                    percentage: pct.min(50),
                    status: "Downloading fixtures...".to_string(),
                },
            );
        }
    }

    if let Some(app) = app {
        let _ = app.emit(
            "fixture-download-progress",
            FixtureDownloadProgress {
                percentage: 50,
                status: "Extracting fixtures...".to_string(),
            },
        );
    }

    let cursor = std::io::Cursor::new(buffer);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to parse fixtures zip archive: {}", e))?;

    if !target_dir.exists() {
        fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create fixtures directory: {}", e))?;
    }

    const MAX_EXTRACTED_BYTES: u64 = 512 * 1024 * 1024;
    let mut total_extracted_bytes = 0u64;

    let total_files = archive.len();
    for i in 0..total_files {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read entry from zip: {}", e))?;

        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory {:?}: {}", outpath, e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create directory {:?}: {}", p, e))?;
                }
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file {:?}: {}", outpath, e))?;

            let remaining_budget = MAX_EXTRACTED_BYTES.saturating_sub(total_extracted_bytes);
            let mut entry_reader = Read::take(&mut file, remaining_budget + 1);
            let copied = std::io::copy(&mut entry_reader, &mut outfile)
                .map_err(|e| format!("Failed to write file {:?}: {}", outpath, e))?;

            if copied > remaining_budget {
                return Err(
                    "Extracted fixtures archive exceeded maximum allowed size limit".to_string(),
                );
            }
            total_extracted_bytes += copied;
        }

        if let Some(app) = app {
            let extract_pct = if total_files > 0 {
                ((i + 1) as f64 / total_files as f64 * 50.0).round() as u8
            } else {
                50
            };
            let _ = app.emit(
                "fixture-download-progress",
                FixtureDownloadProgress {
                    percentage: 50 + extract_pct,
                    status: "Extracting fixtures...".to_string(),
                },
            );
        }
    }

    if let Some(app) = app {
        let _ = app.emit(
            "fixture-download-progress",
            FixtureDownloadProgress {
                percentage: 100,
                status: "Download complete".to_string(),
            },
        );
    }

    Ok(())
}

static FIXTURE_DOWNLOAD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn download_fixtures_sync(app: &tauri::AppHandle) -> Result<(), String> {
    let _download_guard = FIXTURE_DOWNLOAD_LOCK.lock().map_err(|e| e.to_string())?;

    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {:?}", e))?;
    let app_data_fixtures = app_data.join("fixtures");
    let staging_dir = app_data.join(format!("fixtures_staging_{}", std::process::id()));

    if staging_dir.exists() {
        let _ = fs::remove_dir_all(&staging_dir);
    }

    let download_result = download_and_extract_ofl_fixtures(Some(app), &staging_dir);

    if let Err(e) = download_result {
        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }
        return Err(e);
    }

    if !staging_dir.join("manufacturers.json").exists() {
        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }
        return Err(
            "Extracted fixtures archive is invalid: manufacturers.json not found".to_string(),
        );
    }

    let backup_dir = app_data.join(format!("fixtures_backup_{}", std::process::id()));
    if backup_dir.exists() {
        let _ = fs::remove_dir_all(&backup_dir);
    }

    if app_data_fixtures.exists() {
        fs::rename(&app_data_fixtures, &backup_dir)
            .map_err(|e| format!("Failed to move old fixtures directory: {}", e))?;
    }

    if let Err(e) = fs::rename(&staging_dir, &app_data_fixtures) {
        if backup_dir.exists() {
            let _ = fs::rename(&backup_dir, &app_data_fixtures);
        }
        return Err(format!("Failed to activate new fixtures directory: {}", e));
    }

    if backup_dir.exists() {
        let _ = fs::remove_dir_all(&backup_dir);
    }

    if let Ok(mut write_guard) = CACHED_FIXTURES.write() {
        *write_guard = None;
    }

    Ok(())
}

pub fn resolve_fixtures_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {:?}", e))?;
    let app_data_fixtures = app_data.join("fixtures");

    if app_data_fixtures.join("manufacturers.json").exists() {
        return Ok(app_data_fixtures);
    }

    Err("Fixtures directory does not exist or manufacturers.json is missing".to_string())
}

pub fn load_fixtures_sync(app: &tauri::AppHandle) -> Result<Fixtures, String> {
    let fixtures_dir = resolve_fixtures_dir(app)?;

    let manufacturers_file_path = fixtures_dir.join("manufacturers.json");
    let manufacturers_data = fs::read_to_string(&manufacturers_file_path).map_err(|e| {
        format!(
            "Failed to read manufacturers file at {:?}: {}",
            manufacturers_file_path, e
        )
    })?;
    let parsed_manufacturers: HashMap<String, Value> = serde_json::from_str(&manufacturers_data)
        .map_err(|e| format!("Failed to parse manufacturers data: {}", e))?;
    let mut res: Fixtures = BTreeMap::new();

    let total_manufacturers = parsed_manufacturers.len();
    let mut current_idx = 0;

    for (manufacturer_id, manufacturer_val) in &parsed_manufacturers {
        current_idx += 1;
        if manufacturer_id.starts_with('$') {
            continue;
        }

        let manufacturer: ManufacturerParsed =
            match serde_json::from_value(manufacturer_val.clone()) {
                Ok(m) => m,
                Err(_) => continue,
            };

        let percentage = if total_manufacturers > 0 {
            ((current_idx as f64 / total_manufacturers as f64) * 100.0).round() as u8
        } else {
            100
        };

        let _ = app.emit(
            "fixture-load-progress",
            FixtureLoadProgress {
                current: current_idx,
                total: total_manufacturers,
                percentage,
                manufacturer: manufacturer.name.clone(),
            },
        );
        let manufacturer_dir = fixtures_dir.join(manufacturer_id);
        if !manufacturer_dir.exists() {
            continue;
        }

        let mut fixtures = BTreeMap::new();

        if let Ok(entries) = fs::read_dir(&manufacturer_dir) {
            for entry in entries.flatten() {
                let fixture_path = entry.path();

                if !fixture_path.is_file() {
                    continue;
                }

                if fixture_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }

                let file_stem = fixture_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();

                if file_stem.is_empty() {
                    continue;
                }

                if let Ok(fixture_data) = fs::read_to_string(&fixture_path) {
                    match serde_json::from_str::<RawFixture>(&fixture_data) {
                        Ok(raw) => {
                            let fixture_key = raw.fixture_key.unwrap_or(file_stem);
                            let manufacturer_key = raw
                                .manufacturer_key
                                .unwrap_or_else(|| manufacturer_id.clone());

                            let fixture = Fixture {
                                name: raw.name,
                                short_name: raw.short_name,
                                categories: raw.categories,
                                meta: raw.meta,
                                links: raw.links,
                                physical: raw.physical,
                                matrix: raw.matrix,
                                wheels: raw.wheels,
                                available_channels: raw.available_channels,
                                template_channels: raw.template_channels,
                                modes: raw.modes,
                                fixture_key: fixture_key.clone(),
                                manufacturer_key,
                                ofl_url: raw.ofl_url,
                                comment: raw.comment,
                                help_wanted: raw.help_wanted,
                                rdm: raw.rdm,
                            };
                            fixtures.insert(fixture_key, fixture);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse fixture at {:?}: {}", fixture_path, e);
                        }
                    }
                }
            }
        }

        res.insert(
            manufacturer_id.clone(),
            Manufacturer {
                name: manufacturer.name.clone(),
                website: manufacturer.website.clone(),
                comment: manufacturer.comment.clone(),
                rdm_id: manufacturer.rdm_id,
                fixtures,
            },
        );
    }

    Ok(res)
}

pub fn filter_and_sort_fixtures(fixtures: &Fixtures, query: Option<&str>) -> Fixtures {
    let q = match query {
        Some(s) => s.trim().to_lowercase(),
        None => String::new(),
    };

    if q.is_empty() {
        return fixtures.clone();
    }

    let tokens: Vec<&str> = q.split_whitespace().collect();
    if tokens.is_empty() {
        return fixtures.clone();
    }

    let mut filtered = BTreeMap::new();

    for (man_id, manufacturer) in fixtures {
        let man_name_lower = manufacturer.name.to_lowercase();
        let man_id_lower = man_id.to_lowercase();

        let matching_fixtures: BTreeMap<String, Fixture> = manufacturer
            .fixtures
            .iter()
            .filter(|(fix_id, fixture)| {
                let fix_name_lower = fixture.name.to_lowercase();
                let fix_id_lower = fix_id.to_lowercase();
                let short_name_lower = fixture
                    .short_name
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase();

                tokens.iter().all(|token| {
                    man_name_lower.contains(token)
                        || man_id_lower.contains(token)
                        || fix_name_lower.contains(token)
                        || fix_id_lower.contains(token)
                        || short_name_lower.contains(token)
                        || fixture
                            .categories
                            .iter()
                            .any(|cat| cat.to_lowercase().contains(token))
                        || fixture
                            .modes
                            .iter()
                            .any(|m| m.name.to_lowercase().contains(token))
                })
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if !matching_fixtures.is_empty() {
            filtered.insert(
                man_id.clone(),
                Manufacturer {
                    name: manufacturer.name.clone(),
                    website: manufacturer.website.clone(),
                    comment: manufacturer.comment.clone(),
                    rdm_id: manufacturer.rdm_id,
                    fixtures: matching_fixtures,
                },
            );
        }
    }

    filtered
}
