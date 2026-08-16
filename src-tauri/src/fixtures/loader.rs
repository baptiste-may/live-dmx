use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::sync::OnceLock;
use tauri::Manager;

use super::types::{
    Fixture, FixtureLoadProgress, Fixtures, Manufacturer, ManufacturerParsed, RawFixture,
};

pub static CACHED_FIXTURES: OnceLock<Fixtures> = OnceLock::new();

pub fn download_and_extract_ofl_fixtures(target_dir: &std::path::Path) -> Result<(), String> {
    const FIXTURES_URL: &str = "https://open-fixture-library.org/download.ofl";
    const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout_read(std::time::Duration::from_secs(120))
        .build();

    let response = agent
        .get(FIXTURES_URL)
        .call()
        .map_err(|e| format!("Failed to download fixtures from OFL: {}", e))?;

    let mut reader = std::io::Read::take(response.into_reader(), MAX_ARCHIVE_BYTES);
    let mut buffer = Vec::new();
    std::io::copy(&mut reader, &mut buffer)
        .map_err(|e| format!("Failed to read downloaded fixtures zip: {}", e))?;

    let cursor = std::io::Cursor::new(buffer);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to parse fixtures zip archive: {}", e))?;

    if !target_dir.exists() {
        fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create fixtures directory: {}", e))?;
    }

    archive
        .extract(target_dir)
        .map_err(|e| format!("Failed to extract fixtures zip: {}", e))?;

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

    download_and_extract_ofl_fixtures(&app_data_fixtures)?;
    Ok(app_data_fixtures)
}

pub fn load_fixtures_sync(app: &tauri::AppHandle) -> Result<Fixtures, String> {
    use tauri::Emitter;

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

    let mut filtered = BTreeMap::new();

    for (man_id, manufacturer) in fixtures {
        let man_name_lower = manufacturer.name.to_lowercase();
        let man_id_lower = man_id.to_lowercase();
        let man_match = man_name_lower.contains(&q) || man_id_lower.contains(&q);

        let matching_fixtures: BTreeMap<String, Fixture> = manufacturer
            .fixtures
            .iter()
            .filter(|(fix_id, fixture)| {
                man_match
                    || fixture.name.to_lowercase().contains(&q)
                    || fix_id.to_lowercase().contains(&q)
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
