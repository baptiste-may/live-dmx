//! Local fixture cache, online GDTF-Share index and catalog search.
//!
//! See `docs/contributing/fixtures.md` (cache layout, preview cache and search).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::fixtures::gdtf_share::types::{GdtfCatalogItem, GdtfCatalogMode};
use crate::fixtures::generic::get_generic_fixtures;
use crate::fixtures::parser::{parse_gdtf_archive, ParsedGdtfFixture};

/// Maximum number of keys held in the in-memory preview cache.
const MAX_PREVIEW_ENTRIES: usize = 30;

/// A previewed fixture with its raw `.gdtf` bytes, shared between cache keys.
type CachedPreviewEntry = (ParsedGdtfFixture, Arc<Vec<u8>>);

/// Local fixture cache and online catalog index.
///
/// Created once per process (behind an `RwLock`, see `commands.rs`). Combines
/// built-in generics, `.gdtf` files installed on disk, the persisted GDTF-Share
/// index and a bounded in-memory preview cache. Cache keys are
/// `"<manufacturer>@<fixture>"` in lowercase. Each inner `RwLock` failure
/// (poisoning) is tolerated by skipping the update or returning empty data.
pub struct GdtfCatalogManager {
    /// Directory holding installed `.gdtf` archives (`<app_data_dir>/fixtures`).
    fixtures_dir: PathBuf,
    /// Persisted online index (`<app_data_dir>/gdtf_catalog.json`).
    catalog_cache_file: PathBuf,
    /// Online GDTF-Share listing, as last synced.
    online_catalog: RwLock<Vec<GdtfCatalogItem>>,
    /// Parsed installed fixtures, keyed by `"<manufacturer>@<fixture>"` (lowercase).
    local_fixtures: RwLock<HashMap<String, ParsedGdtfFixture>>,
    /// Downloaded-but-not-installed fixtures, memory only, bounded by
    /// `MAX_PREVIEW_ENTRIES`.
    preview_cache: RwLock<HashMap<String, CachedPreviewEntry>>,
}

impl GdtfCatalogManager {
    /// Creates a manager rooted at `app_data_dir` and loads the local fixtures
    /// and the persisted online index, ignoring read errors.
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

    /// Returns the directory where installed `.gdtf` archives are stored.
    pub fn fixtures_dir(&self) -> &Path {
        &self.fixtures_dir
    }

    /// Rescans the fixtures directory (root and one level of subdirectories) and
    /// replaces the installed fixture map.
    ///
    /// Unreadable or unparsable archives are skipped silently; a later file with
    /// the same key overrides an earlier one.
    ///
    /// # Errors
    ///
    /// Currently never fails; scan problems are ignored.
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

    /// Loads the persisted online index into memory, if it exists and is valid.
    ///
    /// # Errors
    ///
    /// Currently never fails; a missing or corrupt file leaves the index unchanged.
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

    /// Persists the online index to disk and replaces the in-memory copy.
    ///
    /// # Errors
    ///
    /// Returns an error when the index cannot be serialized or written.
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

    /// Parses a `.gdtf` archive and installs it in the local fixture cache.
    ///
    /// The file is written to `<manufacturer>/<fixture>[_<first 8 chars of the
    /// fixture type id>].gdtf` with sanitized names, so two fixtures resolving to
    /// the same path overwrite each other. Any matching preview entries are dropped.
    ///
    /// # Errors
    ///
    /// Returns an error when the archive cannot be parsed or the directory or
    /// file cannot be written.
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

    /// Looks up a previewed fixture without removing it.
    ///
    /// Tries the exact key first, then a case-insensitive name match (any
    /// manufacturer when `manufacturer` is empty).
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

    /// Parses a `.gdtf` archive and keeps it in the in-memory preview cache only.
    ///
    /// Stored under both the requested and the parsed manufacturer/name keys,
    /// sharing the bytes. When full, arbitrary entries are evicted (not LRU).
    ///
    /// # Errors
    ///
    /// Returns an error when the archive cannot be parsed.
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

    /// Promotes a previewed fixture to an installed one by writing its archive to disk.
    ///
    /// Returns `Ok(None)` when there is no matching preview entry. The entry is
    /// removed from the cache.
    ///
    /// # Errors
    ///
    /// Returns an error when installing the cached archive fails (see
    /// [`Self::save_downloaded_gdtf`]).
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

    /// Searches generics, installed fixtures and the online index.
    ///
    /// All whitespace-separated tokens (case-insensitive substring) must match
    /// the fixture name, manufacturer or a mode name. An empty query returns only
    /// generics and installed fixtures; online entries are added only for a
    /// non-empty query, capped at 100. Duplicates (same manufacturer and
    /// fixture) are merged, keeping the best item per `is_better_item` and
    /// completing missing fields from the other.
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

        let mut add_or_update =
            |mut item: GdtfCatalogItem| {
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
                                if let Some(source) = existing.modes.iter().find(|s| {
                                    s.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    if target.channels.is_none() {
                                        if let Some(ch) = &source.channels {
                                            target.channels = Some(ch.clone());
                                        }
                                    }
                                    if target.pan_bounds.is_none() {
                                        target.pan_bounds = source.pan_bounds;
                                    }
                                    if target.tilt_bounds.is_none() {
                                        target.tilt_bounds = source.tilt_bounds;
                                    }
                                }
                            }
                        }

                        // If any mode in item still lacks channels, try populating from local_snapshot
                        if let Some(parsed) = local_snapshot.get(&key) {
                            item.is_installed = true;
                            for target in &mut item.modes {
                                if let Some(pm) = parsed.modes.iter().find(|pm| {
                                    pm.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    if target.channels.is_none() {
                                        target.channels = Some(pm.channels.clone());
                                    }
                                    if target.pan_bounds.is_none() {
                                        target.pan_bounds = pm.pan_bounds;
                                    }
                                    if target.tilt_bounds.is_none() {
                                        target.tilt_bounds = pm.tilt_bounds;
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
                        if existing.uploader.as_deref() == Some("Installed")
                            && item.uploader.is_some()
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
                                        if target.pan_bounds.is_none()
                                            && source.pan_bounds.is_some()
                                        {
                                            target.pan_bounds = source.pan_bounds;
                                        }
                                        if target.tilt_bounds.is_none()
                                            && source.tilt_bounds.is_some()
                                        {
                                            target.tilt_bounds = source.tilt_bounds;
                                        }
                                    }
                                }
                            }
                        }

                        // If existing still has modes lacking channels or bounds, check local_snapshot
                        if let Some(parsed) = local_snapshot.get(&key) {
                            existing.is_installed = true;
                            for target in &mut existing.modes {
                                if let Some(pm) = parsed.modes.iter().find(|pm| {
                                    pm.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    if target.channels.is_none() {
                                        target.channels = Some(pm.channels.clone());
                                    }
                                    if target.pan_bounds.is_none() {
                                        target.pan_bounds = pm.pan_bounds;
                                    }
                                    if target.tilt_bounds.is_none() {
                                        target.tilt_bounds = pm.tilt_bounds;
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
                                    pan_bounds: m.pan_bounds,
                                    tilt_bounds: m.tilt_bounds,
                                })
                                .collect();
                        } else {
                            for target in &mut item.modes {
                                if let Some(pm) = parsed.modes.iter().find(|pm| {
                                    pm.name.trim().eq_ignore_ascii_case(target.name.trim())
                                }) {
                                    if target.channels.is_none() {
                                        target.channels = Some(pm.channels.clone());
                                    }
                                    if target.pan_bounds.is_none() {
                                        target.pan_bounds = pm.pan_bounds;
                                    }
                                    if target.tilt_bounds.is_none() {
                                        target.tilt_bounds = pm.tilt_bounds;
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
        for g in generic_fixtures.iter() {
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
                    fixture: g.name.clone(),
                    manufacturer: g.manufacturer.clone(),
                    revision: Some("1.0".to_string()),
                    uploader: Some("Built-in".to_string()),
                    rating: Some(5.0),
                    modes: g
                        .modes
                        .iter()
                        .cloned()
                        .map(|m| GdtfCatalogMode {
                            name: m.name,
                            dmxfootprint: m.channel_count as u32,
                            channels: Some(m.channels),
                            pan_bounds: m.pan_bounds,
                            tilt_bounds: m.tilt_bounds,
                        })
                        .collect(),
                    uuid: g.fixture_type_id.clone(),
                    filesize: None,
                    is_installed: true,
                    is_generic: true,
                    categories: g.categories.clone(),
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
                            pan_bounds: m.pan_bounds,
                            tilt_bounds: m.tilt_bounds,
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

    /// Returns an installed or built-in fixture by manufacturer and name.
    ///
    /// Installed fixtures take precedence over generics.
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
        get_generic_fixtures()
            .iter()
            .find(|g| {
                g.manufacturer.eq_ignore_ascii_case(manufacturer)
                    && g.name.eq_ignore_ascii_case(fixture)
            })
            .cloned()
    }
}

/// Well-known moving-head fixture families whose model names carry no generic keyword
/// (e.g. Clay Paky "Sharpy"). Used only as a fallback when categories are absent.
const FAMOUS_MOVING_HEAD_NAMES: &[&str] = &["sharpy"];

/// Guesses categories from a fixture name using conservative keywords.
///
/// Used for online, non-installed items that lack categories. Returns `None`
/// when no keyword matches.
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

/// Replaces every character that is not alphanumeric, `-` or `_` with `_`.
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

/// Returns whether the uploader name starts with `manuf` (official manufacturer upload).
fn is_uploader_manuf(uploader: Option<&str>) -> bool {
    if let Some(u) = uploader {
        let lower = u.to_lowercase();
        lower.starts_with("manuf")
    } else {
        false
    }
}

/// Returns whether `candidate` should replace `existing` for the same fixture.
///
/// Priority: generic, installed, manufacturer upload, rating (difference above
/// 0.05), then the larger `rid` (more recent revision).
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
