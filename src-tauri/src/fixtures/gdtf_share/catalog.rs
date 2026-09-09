use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::fixtures::gdtf_share::types::{GdtfCatalogItem, GdtfCatalogMode};
use crate::fixtures::generic::get_generic_fixtures;
use crate::fixtures::parser::{parse_gdtf_archive, ParsedGdtfFixture};

const MAX_PREVIEW_ENTRIES: usize = 30;

type CachedPreviewEntry = (ParsedGdtfFixture, Arc<Vec<u8>>);

pub struct GdtfCatalogManager {
    fixtures_dir: PathBuf,
    catalog_cache_file: PathBuf,
    online_catalog: RwLock<Vec<GdtfCatalogItem>>,
    local_fixtures: RwLock<HashMap<String, ParsedGdtfFixture>>,
    preview_cache: RwLock<HashMap<String, CachedPreviewEntry>>,
}

impl GdtfCatalogManager {
    pub fn new(app_data_dir: &Path) -> Self {
        let fixtures_dir = app_data_dir.join("fixtures");
        let catalog_cache_file = app_data_dir.join("gdtf_catalog.json");

        let manager = Self {
            fixtures_dir,
            catalog_cache_file,
            online_catalog: RwLock::new(Vec::new()),
            local_fixtures: RwLock::new(HashMap::new()),
            preview_cache: RwLock::new(HashMap::new()),
        };

        let _ = manager.load_local_fixtures();
        let _ = manager.load_cached_online_catalog();

        manager
    }

    pub fn fixtures_dir(&self) -> &Path {
        &self.fixtures_dir
    }

    pub fn load_local_fixtures(&self) -> Result<(), String> {
        if !self.fixtures_dir.exists() {
            let _ = fs::create_dir_all(&self.fixtures_dir);
        }

        let mut local_map = HashMap::new();

        // Scan local directory for .gdtf files
        if let Ok(entries) = fs::read_dir(&self.fixtures_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.extension().and_then(|e| e.to_str()) == Some("gdtf") {
                                if let Ok(bytes) = fs::read(&sub_path) {
                                    if let Ok(parsed) = parse_gdtf_archive(&bytes) {
                                        let key = format!(
                                            "{}@{}",
                                            parsed.manufacturer.to_lowercase(),
                                            parsed.name.to_lowercase()
                                        );
                                        local_map.insert(key, parsed);
                                    }
                                }
                            }
                        }
                    }
                } else if path.extension().and_then(|e| e.to_str()) == Some("gdtf") {
                    if let Ok(bytes) = fs::read(&path) {
                        if let Ok(parsed) = parse_gdtf_archive(&bytes) {
                            let key = format!(
                                "{}@{}",
                                parsed.manufacturer.to_lowercase(),
                                parsed.name.to_lowercase()
                            );
                            local_map.insert(key, parsed);
                        }
                    }
                }
            }
        }

        if let Ok(mut lock) = self.local_fixtures.write() {
            *lock = local_map;
        }

        Ok(())
    }

    pub fn load_cached_online_catalog(&self) -> Result<(), String> {
        if self.catalog_cache_file.exists() {
            if let Ok(json_str) = fs::read_to_string(&self.catalog_cache_file) {
                if let Ok(items) = serde_json::from_str::<Vec<GdtfCatalogItem>>(&json_str) {
                    if let Ok(mut lock) = self.online_catalog.write() {
                        *lock = items;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_online_catalog(&self, items: &[GdtfCatalogItem]) -> Result<(), String> {
        let json_str = serde_json::to_string_pretty(items)
            .map_err(|e| format!("Failed to serialize catalog: {}", e))?;
        fs::write(&self.catalog_cache_file, json_str)
            .map_err(|e| format!("Failed to write catalog cache: {}", e))?;

        if let Ok(mut lock) = self.online_catalog.write() {
            *lock = items.to_vec();
        }

        Ok(())
    }

    pub fn save_downloaded_gdtf(
        &self,
        manufacturer: &str,
        fixture_name: &str,
        data: &[u8],
    ) -> Result<ParsedGdtfFixture, String> {
        let parsed = parse_gdtf_archive(data)?;

        let sanitized_man = sanitize_filename(manufacturer);
        let sanitized_man = if sanitized_man.is_empty() {
            "generic".to_string()
        } else {
            sanitized_man
        };

        let sanitized_fix = sanitize_filename(fixture_name);
        let stem = if sanitized_fix.is_empty() {
            "fixture".to_string()
        } else {
            sanitized_fix
        };

        let target_dir = self.fixtures_dir.join(&sanitized_man);
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create directory {:?}: {}", target_dir, e))?;

        let file_name = if let Some(ref tid) = parsed.fixture_type_id {
            let tid_clean = sanitize_filename(tid);
            let disc = tid_clean.chars().take(8).collect::<String>();
            if !disc.is_empty() {
                format!("{}_{}.gdtf", stem, disc)
            } else {
                format!("{}.gdtf", stem)
            }
        } else {
            format!("{}.gdtf", stem)
        };

        let file_path = target_dir.join(file_name);
        fs::write(&file_path, data)
            .map_err(|e| format!("Failed to write GDTF file {:?}: {}", file_path, e))?;

        let key = format!(
            "{}@{}",
            parsed.manufacturer.to_lowercase(),
            parsed.name.to_lowercase()
        );

        if let Ok(mut lock) = self.local_fixtures.write() {
            lock.insert(key.clone(), parsed.clone());
        }

        if let Ok(mut lock) = self.preview_cache.write() {
            lock.remove(&key);
            let requested_key = format!(
                "{}@{}",
                manufacturer.to_lowercase(),
                fixture_name.to_lowercase()
            );
            lock.remove(&requested_key);
        }

        Ok(parsed)
    }

    pub fn get_preview_fixture(
        &self,
        manufacturer: &str,
        fixture: &str,
    ) -> Option<ParsedGdtfFixture> {
        let key = format!(
            "{}@{}",
            manufacturer.trim().to_lowercase(),
            fixture.trim().to_lowercase()
        );
        if let Ok(lock) = self.preview_cache.read() {
            if let Some((parsed, _)) = lock.get(&key) {
                return Some(parsed.clone());
            }
            for (parsed, _) in lock.values() {
                if parsed.name.trim().eq_ignore_ascii_case(fixture.trim())
                    && (manufacturer.trim().is_empty()
                        || parsed
                            .manufacturer
                            .trim()
                            .eq_ignore_ascii_case(manufacturer.trim()))
                {
                    return Some(parsed.clone());
                }
            }
        }
        None
    }

    pub fn save_preview_gdtf(
        &self,
        manufacturer: &str,
        fixture_name: &str,
        data: &[u8],
    ) -> Result<ParsedGdtfFixture, String> {
        let parsed = parse_gdtf_archive(data)?;
        let key1 = format!(
            "{}@{}",
            manufacturer.trim().to_lowercase(),
            fixture_name.trim().to_lowercase()
        );
        let key2 = format!(
            "{}@{}",
            parsed.manufacturer.trim().to_lowercase(),
            parsed.name.trim().to_lowercase()
        );

        let shared_bytes = Arc::new(data.to_vec());

        if let Ok(mut lock) = self.preview_cache.write() {
            while lock.len() >= MAX_PREVIEW_ENTRIES {
                if let Some(old_key) = lock.keys().next().cloned() {
                    lock.remove(&old_key);
                } else {
                    break;
                }
            }
            lock.insert(key1, (parsed.clone(), Arc::clone(&shared_bytes)));
            lock.insert(key2, (parsed.clone(), shared_bytes));
        }

        Ok(parsed)
    }

    pub fn persist_preview_fixture(
        &self,
        manufacturer: &str,
        fixture_name: &str,
    ) -> Result<Option<ParsedGdtfFixture>, String> {
        let key = format!(
            "{}@{}",
            manufacturer.trim().to_lowercase(),
            fixture_name.trim().to_lowercase()
        );

        let cached_bytes = {
            if let Ok(mut lock) = self.preview_cache.write() {
                if let Some((_, bytes)) = lock.remove(&key) {
                    Some(bytes)
                } else {
                    let mut found_key = None;
                    for (k, (p, _)) in lock.iter() {
                        if p.name.trim().eq_ignore_ascii_case(fixture_name.trim())
                            && (manufacturer.trim().is_empty()
                                || p.manufacturer
                                    .trim()
                                    .eq_ignore_ascii_case(manufacturer.trim()))
                        {
                            found_key = Some(k.clone());
                            break;
                        }
                    }
                    if let Some(fk) = found_key {
                        lock.remove(&fk).map(|(_, bytes)| bytes)
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        };

        if let Some(bytes) = cached_bytes {
            let saved = self.save_downloaded_gdtf(manufacturer, fixture_name, &bytes)?;
            Ok(Some(saved))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: Option<&str>) -> Vec<GdtfCatalogItem> {
        let q = query.map(|s| s.trim().to_lowercase()).unwrap_or_default();
        let tokens: Vec<&str> = q.split_whitespace().collect();

        let local_snapshot: HashMap<String, ParsedGdtfFixture> = self
            .local_fixtures
            .read()
            .map(|lock| lock.clone())
            .unwrap_or_default();

        let mut results = Vec::new();
        let mut results_map: HashMap<String, usize> = HashMap::new();

        let mut add_or_update = |mut item: GdtfCatalogItem| {
            let key = format!(
                "{}@{}",
                item.manufacturer.trim().to_lowercase(),
                item.fixture.trim().to_lowercase()
            );

            if let Some(&idx) = results_map.get(&key) {
                let existing = &mut results[idx];
                let better = is_better_item(&item, existing);
                if better {
                    if existing.is_installed {
                        item.is_installed = true;
                    }
                    if item.categories.is_none() && existing.categories.is_some() {
                        item.categories = existing.categories.clone();
                    }
                    if item.modes.is_empty() && !existing.modes.is_empty() {
                        item.modes = existing.modes.clone();
                    } else {
                        // Merge channels from existing modes
                        for target in &mut item.modes {
                            if target.channels.is_none() {
                                if let Some(source) = existing.modes.iter().find(|s| {
                                    s.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    if source.channels.is_some() {
                                        target.channels = source.channels.clone();
                                    }
                                }
                            }
                        }
                    }

                    // If any mode in item still lacks channels, try populating from local_snapshot
                    if let Some(parsed) = local_snapshot.get(&key) {
                        item.is_installed = true;
                        for target in &mut item.modes {
                            if target.channels.is_none() {
                                if let Some(pm) = parsed.modes.iter().find(|pm| {
                                    pm.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    target.channels = Some(pm.channels.clone());
                                }
                            }
                        }
                    }
                    if item.categories.is_none() {
                        if let Some(parsed) = local_snapshot.get(&key) {
                            if let Some(cats) = &parsed.categories {
                                item.categories = Some(cats.clone());
                            }
                        }
                    }

                    // If existing had channels and item.modes still has no channels at all, keep existing modes
                    if item.modes.iter().all(|m| m.channels.is_none())
                        && existing.modes.iter().any(|m| m.channels.is_some())
                    {
                        item.modes = existing.modes.clone();
                    }

                    results[idx] = item;
                } else {
                    if item.is_installed {
                        existing.is_installed = true;
                    }
                    if existing.rid.is_none() && item.rid.is_some() {
                        existing.rid = item.rid;
                    }
                    if existing.revision.is_none() && item.revision.is_some() {
                        existing.revision = item.revision.clone();
                    }
                    if existing.uploader.as_deref() == Some("Installed") && item.uploader.is_some()
                    {
                        existing.uploader = item.uploader.clone();
                    }
                    if existing.rating.is_none() && item.rating.is_some() {
                        existing.rating = item.rating;
                    }
                    if existing.categories.is_none() && item.categories.is_some() {
                        existing.categories = item.categories.clone();
                    }
                    if existing.modes.is_empty() && !item.modes.is_empty() {
                        existing.modes = item.modes.clone();
                    } else {
                        // Merge channels from candidate into existing
                        for target in &mut existing.modes {
                            if target.channels.is_none() {
                                if let Some(source) = item.modes.iter().find(|s| {
                                    s.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    if source.channels.is_some() {
                                        target.channels = source.channels.clone();
                                    }
                                }
                            }
                        }
                    }

                    // If existing still has modes lacking channels, check local_snapshot
                    if let Some(parsed) = local_snapshot.get(&key) {
                        existing.is_installed = true;
                        for target in &mut existing.modes {
                            if target.channels.is_none() {
                                if let Some(pm) = parsed.modes.iter().find(|pm| {
                                    pm.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    target.channels = Some(pm.channels.clone());
                                }
                            }
                        }
                    }
                }
            } else {
                // First time seeing this fixture key: ensure local channels are attached if installed locally
                if let Some(parsed) = local_snapshot.get(&key) {
                    item.is_installed = true;
                    if item.categories.is_none() {
                        if let Some(cats) = &parsed.categories {
                            item.categories = Some(cats.clone());
                        }
                    }
                    if item.modes.is_empty() {
                        item.modes = parsed
                            .modes
                            .iter()
                            .map(|m| GdtfCatalogMode {
                                name: m.name.clone(),
                                dmxfootprint: m.channel_count as u32,
                                channels: Some(m.channels.clone()),
                            })
                            .collect();
                    } else {
                        for target in &mut item.modes {
                            if target.channels.is_none() {
                                if let Some(pm) = parsed.modes.iter().find(|pm| {
                                    pm.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    target.channels = Some(pm.channels.clone());
                                }
                            }
                        }
                    }
                }
                results_map.insert(key, results.len());
                results.push(item);
            }
            results.len()
        };

        // 1. Generic fixtures
        let generic_fixtures = get_generic_fixtures();
        for g in generic_fixtures {
            let fixture_lower = g.name.to_lowercase();
            let man_lower = g.manufacturer.to_lowercase();

            let matches = tokens.is_empty()
                || tokens.iter().all(|&tok| {
                    fixture_lower.contains(tok)
                        || man_lower.contains(tok)
                        || g.modes.iter().any(|m| m.name.to_lowercase().contains(tok))
                });

            if matches {
                add_or_update(GdtfCatalogItem {
                    rid: None,
                    fixture: g.name,
                    manufacturer: g.manufacturer,
                    revision: Some("1.0".to_string()),
                    uploader: Some("Built-in".to_string()),
                    rating: Some(5.0),
                    modes: g
                        .modes
                        .into_iter()
                        .map(|m| GdtfCatalogMode {
                            name: m.name,
                            dmxfootprint: m.channel_count as u32,
                            channels: Some(m.channels),
                        })
                        .collect(),
                    uuid: g.fixture_type_id,
                    filesize: None,
                    is_installed: true,
                    is_generic: true,
                    categories: g.categories,
                });
            }
        }

        // 2. Locally installed custom GDTF fixtures
        for parsed in local_snapshot.values() {
            let fixture_lower = parsed.name.to_lowercase();
            let man_lower = parsed.manufacturer.to_lowercase();

            let matches = tokens.is_empty()
                || tokens.iter().all(|&tok| {
                    fixture_lower.contains(tok)
                        || man_lower.contains(tok)
                        || parsed
                            .modes
                            .iter()
                            .any(|m| m.name.to_lowercase().contains(tok))
                });

            if matches {
                add_or_update(GdtfCatalogItem {
                    rid: None,
                    fixture: parsed.name.clone(),
                    manufacturer: parsed.manufacturer.clone(),
                    revision: None,
                    uploader: Some("Installed".to_string()),
                    rating: Some(5.0),
                    modes: parsed
                        .modes
                        .iter()
                        .map(|m| GdtfCatalogMode {
                            name: m.name.clone(),
                            dmxfootprint: m.channel_count as u32,
                            channels: Some(m.channels.clone()),
                        })
                        .collect(),
                    uuid: parsed.fixture_type_id.clone(),
                    filesize: None,
                    is_installed: true,
                    is_generic: false,
                    categories: parsed.categories.clone(),
                });
            }
        }

        // 3. Online GDTF-Share catalog items (if search query is present)
        if !tokens.is_empty() {
            if let Ok(online_lock) = self.online_catalog.read() {
                let mut online_added = 0usize;
                for item in online_lock.iter() {
                    let fix_lower = item.fixture.to_lowercase();
                    let man_lower = item.manufacturer.to_lowercase();

                    let matches = tokens.iter().all(|&tok| {
                        fix_lower.contains(tok)
                            || man_lower.contains(tok)
                            || item
                                .modes
                                .iter()
                                .any(|m| m.name.to_lowercase().contains(tok))
                    });

                    if matches {
                        let mut item_clone = item.clone();
                        // Check if already installed locally
                        let key = format!("{}@{}", man_lower, fix_lower);
                        item_clone.is_installed = local_snapshot.contains_key(&key);
                        if !item_clone.is_installed && item_clone.categories.is_none() {
                            item_clone.categories = infer_categories_from_name(&item_clone.fixture);
                        }
                        add_or_update(item_clone);
                        online_added += 1;
                        if online_added >= 100 {
                            break;
                        }
                    }
                }
            }
        }

        results
    }

    pub fn get_local_fixture(
        &self,
        manufacturer: &str,
        fixture: &str,
    ) -> Option<ParsedGdtfFixture> {
        let key = format!("{}@{}", manufacturer.to_lowercase(), fixture.to_lowercase());

        if let Ok(local_lock) = self.local_fixtures.read() {
            if let Some(f) = local_lock.get(&key) {
                return Some(f.clone());
            }
        }

        // Check generic
        get_generic_fixtures().into_iter().find(|g| {
            g.manufacturer.eq_ignore_ascii_case(manufacturer)
                && g.name.eq_ignore_ascii_case(fixture)
        })
    }
}

// Well-known moving-head fixture families whose model names carry no generic keyword
// (e.g. Clay Paky "Sharpy"). Used only as a fallback when categories are absent.
const FAMOUS_MOVING_HEAD_NAMES: &[&str] = &["sharpy"];

fn infer_categories_from_name(name: &str) -> Option<Vec<String>> {
    let lower = name.to_lowercase();
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let word = |needle: &str| tokens.contains(&needle);

    let mut categories = Vec::new();

    if lower.contains("strobe") {
        categories.push("Strobe".to_string());
    }
    if lower.contains("blinder") {
        categories.push("Blinder".to_string());
    }
    if lower.contains("laser") {
        categories.push("Laser".to_string());
    }
    if lower.contains("smoke") || lower.contains("hazer") || lower.contains("fog") {
        categories.push("Smoke".to_string());
    }
    if lower.contains("matrix") {
        categories.push("Matrix".to_string());
    }
    if lower.contains("dimmer") {
        categories.push("Dimmer".to_string());
    }
    if lower.contains("confetti") {
        categories.push("Confetti".to_string());
    }
    if lower.contains("geyser") {
        categories.push("Geyser".to_string());
    }
    if lower.contains("pyro") {
        categories.push("Pyro".to_string());
    }
    if lower.contains("water") {
        categories.push("Water".to_string());
    }
    if lower.contains("wind") {
        categories.push("Wind".to_string());
    }
    if word("scanner") {
        categories.push("Scanner".to_string());
    }
    if word("pixel") {
        categories.push("Pixel Bar".to_string());
    }
    if word("par") {
        categories.push("PAR".to_string());
    }
    if word("spot") {
        categories.push("Spot".to_string());
    }
    if word("wash") {
        categories.push("Wash".to_string());
    }
    if lower.contains("moving")
        || word("head")
        || word("beam")
        || FAMOUS_MOVING_HEAD_NAMES.iter().any(|f| tokens.contains(f))
    {
        categories.push("Moving Head".to_string());
    }

    if categories.is_empty() {
        None
    } else {
        Some(categories)
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn is_uploader_manuf(uploader: Option<&str>) -> bool {
    if let Some(u) = uploader {
        let lower = u.to_lowercase();
        lower.starts_with("manuf")
    } else {
        false
    }
}

fn is_better_item(candidate: &GdtfCatalogItem, existing: &GdtfCatalogItem) -> bool {
    // 1. Generic fixtures always stay generic
    if existing.is_generic && !candidate.is_generic {
        return false;
    }
    if candidate.is_generic && !existing.is_generic {
        return true;
    }

    // 2. Installed fixtures take priority over uninstalled
    if existing.is_installed && !candidate.is_installed {
        return false;
    }
    if candidate.is_installed && !existing.is_installed {
        return true;
    }

    // 3. Official manufacturer upload beats community upload
    let cand_manuf = is_uploader_manuf(candidate.uploader.as_deref());
    let exist_manuf = is_uploader_manuf(existing.uploader.as_deref());
    if cand_manuf && !exist_manuf {
        return true;
    }
    if !cand_manuf && exist_manuf {
        return false;
    }

    // 4. Higher rating
    let cand_rating = candidate.rating.unwrap_or(0.0);
    let exist_rating = existing.rating.unwrap_or(0.0);
    if (cand_rating - exist_rating).abs() > 0.05 {
        return cand_rating > exist_rating;
    }

    // 5. Higher RID (later revision on GDTF-Share)
    let cand_rid = candidate.rid.unwrap_or(0);
    let exist_rid = existing.rid.unwrap_or(0);
    cand_rid > exist_rid
}
