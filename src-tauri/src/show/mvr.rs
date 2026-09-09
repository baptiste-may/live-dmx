use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use tauri::{AppHandle, Manager};

use crate::fixtures::generic::get_generic_fixtures;
use crate::fixtures::parser::{parse_gdtf_archive, ParsedGdtfFixture};
use crate::show::types::{
    EmbeddedGdtfDefinition, EmbeddedGdtfMode, FixtureGroup, PatchedFixture, ShowChangesState,
    ShowData, ShowIdentityState, ShowOutputState, ShowPatchState, ShowStatus, UniverseOutputConfig,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MvrAddress {
    pub break_id: u32,
    pub universe: u32,
    pub address: u32,
}

#[derive(Debug, Clone, Default)]
pub struct MvrFixture {
    pub name: String,
    pub uuid: Option<String>,
    pub fixture_id: Option<String>,
    pub fixture_id_numeric: Option<u32>,
    pub unit_number: Option<u32>,
    pub gdtf_spec: Option<String>,
    pub gdtf_mode: Option<String>,
    pub addresses: Vec<MvrAddress>,
}

#[derive(Debug, Clone, Default)]
pub struct MvrLayer {
    pub name: String,
    pub uuid: Option<String>,
    pub fixtures: Vec<MvrFixture>,
}

#[derive(Debug, Clone, Default)]
pub struct MvrScene {
    pub layers: Vec<MvrLayer>,
}

/// Parses DMX address string representation per DIN SPEC 15801.
/// Supported formats:
/// - "Universe.Address" (e.g. "1.1", "2.100")
/// - "Universe/Address" or "Universe,Address"
/// - Absolute integer address (e.g. 1 -> 1.1, 512 -> 1.512, 513 -> 2.1)
pub fn parse_dmx_address(text: &str) -> (u32, u32) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (1, 1);
    }

    if let Some((u_str, a_str)) = trimmed
        .split_once('.')
        .or_else(|| trimmed.split_once('/'))
        .or_else(|| trimmed.split_once(','))
    {
        let u = u_str.trim().parse::<u32>().unwrap_or(1).clamp(1, 512);
        let a = a_str.trim().parse::<u32>().unwrap_or(1).clamp(1, 512);
        return (u, a);
    }

    if let Ok(abs_addr) = trimmed.parse::<u32>() {
        if abs_addr == 0 {
            return (1, 1);
        }
        let u = ((abs_addr - 1) / 512) + 1;
        let a = ((abs_addr - 1) % 512) + 1;
        return (u.clamp(1, 512), a.clamp(1, 512));
    }

    (1, 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagState {
    None,
    GdtfSpec,
    GdtfMode,
    FixtureId,
    FixtureIdNumeric,
    UnitNumber,
    Address,
}

/// Parses GeneralSceneDescription.xml into an MvrScene structure.
pub fn parse_mvr_xml(xml: &str) -> Result<MvrScene, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut scene = MvrScene::default();
    let mut current_layer: Option<MvrLayer> = None;
    let mut fixture_stack: Vec<MvrFixture> = Vec::new();
    let mut root_fixtures: Vec<MvrFixture> = Vec::new();

    let mut current_tag = TagState::None;
    let mut current_address_break: u32 = 0;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"Layer" => {
                    let mut name = String::new();
                    let mut uuid = None;
                    for attr in e.attributes().flatten() {
                        let val = attr.unescape_value().unwrap_or_default().into_owned();
                        match attr.key.as_ref() {
                            b"name" => name = val,
                            b"uuid" => uuid = Some(val),
                            _ => {}
                        }
                    }
                    if name.is_empty() {
                        name = format!("Layer {}", scene.layers.len() + 1);
                    }
                    current_layer = Some(MvrLayer {
                        name,
                        uuid,
                        fixtures: Vec::new(),
                    });
                }
                b"Fixture" => {
                    let mut name = String::new();
                    let mut uuid = None;
                    for attr in e.attributes().flatten() {
                        let val = attr.unescape_value().unwrap_or_default().into_owned();
                        match attr.key.as_ref() {
                            b"name" => name = val,
                            b"uuid" => uuid = Some(val),
                            _ => {}
                        }
                    }
                    fixture_stack.push(MvrFixture {
                        name,
                        uuid,
                        fixture_id: None,
                        fixture_id_numeric: None,
                        unit_number: None,
                        gdtf_spec: None,
                        gdtf_mode: None,
                        addresses: Vec::new(),
                    });
                }
                b"GDTFSpec" => current_tag = TagState::GdtfSpec,
                b"GDTFMode" => current_tag = TagState::GdtfMode,
                b"FixtureID" => current_tag = TagState::FixtureId,
                b"FixtureIDNumeric" => current_tag = TagState::FixtureIdNumeric,
                b"UnitNumber" => current_tag = TagState::UnitNumber,
                b"Address" => {
                    current_tag = TagState::Address;
                    current_address_break = 0;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"break" {
                            let val = attr.unescape_value().unwrap_or_default();
                            current_address_break = val.parse::<u32>().unwrap_or(0);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref t)) => {
                let text = t.unescape().unwrap_or_default().trim().to_string();
                if let Some(fixture) = fixture_stack.last_mut() {
                    match current_tag {
                        TagState::GdtfSpec => {
                            if !text.is_empty() {
                                fixture.gdtf_spec = Some(text);
                            }
                        }
                        TagState::GdtfMode => {
                            if !text.is_empty() {
                                fixture.gdtf_mode = Some(text);
                            }
                        }
                        TagState::FixtureId => {
                            if !text.is_empty() {
                                fixture.fixture_id = Some(text);
                            }
                        }
                        TagState::FixtureIdNumeric => {
                            if let Ok(num) = text.parse::<u32>() {
                                fixture.fixture_id_numeric = Some(num);
                            }
                        }
                        TagState::UnitNumber => {
                            if let Ok(num) = text.parse::<u32>() {
                                fixture.unit_number = Some(num);
                            }
                        }
                        TagState::Address => {
                            if !text.is_empty() {
                                let (universe, address) = parse_dmx_address(&text);
                                fixture.addresses.push(MvrAddress {
                                    break_id: current_address_break,
                                    universe,
                                    address,
                                });
                            }
                        }
                        TagState::None => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"Layer" => {
                    if let Some(layer) = current_layer.take() {
                        scene.layers.push(layer);
                    }
                }
                b"Fixture" => {
                    if let Some(fixture) = fixture_stack.pop() {
                        if let Some(layer) = current_layer.as_mut() {
                            layer.fixtures.push(fixture);
                        } else {
                            root_fixtures.push(fixture);
                        }
                    }
                }
                b"GDTFSpec" | b"GDTFMode" | b"FixtureID" | b"FixtureIDNumeric" | b"UnitNumber"
                | b"Address" => {
                    current_tag = TagState::None;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "XML parse error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => {}
        }
        buf.clear();
    }

    if let Some(layer) = current_layer {
        scene.layers.push(layer);
    }

    if !root_fixtures.is_empty() {
        if let Some(first_layer) = scene.layers.first_mut() {
            first_layer.fixtures.extend(root_fixtures);
        } else {
            scene.layers.push(MvrLayer {
                name: "Default".to_string(),
                uuid: None,
                fixtures: root_fixtures,
            });
        }
    }

    Ok(scene)
}

fn find_matching_generic_fixture(query: &str) -> Option<ParsedGdtfFixture> {
    let lower = query.to_lowercase();
    let generics = get_generic_fixtures();

    if lower.contains("dimmer") || lower.contains("intensity") {
        return generics
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case("Dimmer"));
    }
    if lower.contains("rgbw") {
        return generics
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case("RGBW PAR"));
    }
    let has_uv_token = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|token| token == "uv");
    if lower.contains("rgbaw") || has_uv_token {
        return generics
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case("RGBAW+UV PAR"));
    }
    if lower.contains("rgb") {
        return generics
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case("RGB PAR"));
    }
    if lower.contains("spot") {
        return generics
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case("Moving Head Spot"));
    }
    if lower.contains("wash") {
        return generics
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case("Moving Head Wash"));
    }
    if lower.contains("strobe") {
        return generics
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case("Strobe 1ch"));
    }

    None
}

fn fallback_fixture_definition(name: &str) -> ParsedGdtfFixture {
    ParsedGdtfFixture {
        name: name.to_string(),
        short_name: None,
        manufacturer: "Generic".to_string(),
        description: Some("Imported from MVR".to_string()),
        fixture_type_id: None,
        categories: Some(vec!["Dimmer".to_string()]),
        modes: vec![crate::fixtures::parser::ParsedGdtfMode {
            name: "1 Channel".to_string(),
            short_name: Some("1ch".to_string()),
            channel_count: 1,
            channels: vec!["Dimmer".to_string()],
        }],
    }
}

fn resolve_fixture_gdtf_and_mode(
    mvr_fixture: &MvrFixture,
    gdtf_files: &HashMap<String, ParsedGdtfFixture>,
) -> (ParsedGdtfFixture, crate::fixtures::parser::ParsedGdtfMode) {
    let gdtf_def = if let Some(ref spec) = mvr_fixture.gdtf_spec {
        let spec_lower = spec.to_lowercase();
        let spec_base = Path::new(spec)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(spec)
            .to_lowercase();
        let spec_stem = Path::new(&spec_base)
            .file_stem()
            .and_then(|f| f.to_str())
            .unwrap_or(&spec_base)
            .to_lowercase();

        if let Some(found) = gdtf_files
            .get(&spec_lower)
            .or_else(|| gdtf_files.get(&spec_base))
            .or_else(|| gdtf_files.get(&spec_stem))
        {
            found.clone()
        } else if let Some(generic) = find_matching_generic_fixture(spec) {
            generic
        } else if let Some(generic_by_name) = find_matching_generic_fixture(&mvr_fixture.name) {
            generic_by_name
        } else {
            fallback_fixture_definition(&spec_stem)
        }
    } else if let Some(generic_by_name) = find_matching_generic_fixture(&mvr_fixture.name) {
        generic_by_name
    } else if !mvr_fixture.name.is_empty() {
        fallback_fixture_definition(&mvr_fixture.name)
    } else {
        fallback_fixture_definition("Fixture")
    };

    let mode =
        if let Some(ref mode_name) = mvr_fixture.gdtf_mode {
            let trimmed = mode_name.trim();
            gdtf_def
                .modes
                .iter()
                .find(|m| {
                    m.name.trim().eq_ignore_ascii_case(trimmed)
                        || m.short_name
                            .as_deref()
                            .unwrap_or("")
                            .trim()
                            .eq_ignore_ascii_case(trimmed)
                })
                .cloned()
                .unwrap_or_else(|| {
                    gdtf_def.modes.first().cloned().unwrap_or_else(|| {
                        crate::fixtures::parser::ParsedGdtfMode {
                            name: trimmed.to_string(),
                            short_name: None,
                            channel_count: 1,
                            channels: vec!["Dimmer".to_string()],
                        }
                    })
                })
        } else {
            gdtf_def.modes.first().cloned().unwrap_or_else(|| {
                crate::fixtures::parser::ParsedGdtfMode {
                    name: "1 Channel".to_string(),
                    short_name: None,
                    channel_count: 1,
                    channels: vec!["Dimmer".to_string()],
                }
            })
        };

    (gdtf_def, mode)
}

fn find_first_free_span(
    occupied_ranges: &HashMap<u32, Vec<(u32, u32)>>,
    channel_count: u32,
) -> (u32, u32) {
    let count = channel_count.clamp(1, 512);
    for u in 1..=512 {
        let mut ranges = occupied_ranges.get(&u).cloned().unwrap_or_default();
        ranges.sort_unstable_by_key(|r| r.0);

        let mut candidate_addr = 1;
        for (start, end) in &ranges {
            if candidate_addr + count - 1 < *start {
                break;
            }
            if candidate_addr <= *end {
                candidate_addr = end + 1;
            }
        }

        if candidate_addr + count - 1 <= 512 {
            return (u, candidate_addr);
        }
    }
    (1, 1)
}

/// Sanitizes a group name by trimming whitespace and removing decorative characters
/// often found in CAD exports (e.g., ">> BANK <<" -> "BANK", "[Front Truss]" -> "Front Truss").
pub fn sanitize_group_name(name: &str) -> String {
    let chars = [
        '>', '<', '[', ']', '*', '~', '-', '_', ' ', '\t', '\r', '\n',
    ];
    let trimmed = name.trim();
    let mut clean = trimmed;
    while let Some(stripped) = clean.strip_prefix(&chars[..]) {
        clean = stripped;
    }
    while let Some(stripped) = clean.strip_suffix(&chars[..]) {
        clean = stripped;
    }
    let clean = clean.trim();
    if clean.is_empty() {
        "Group".to_string()
    } else {
        clean.to_string()
    }
}

/// Checks if a layer name corresponds to an unorganized default/catch-all CAD layer.
pub fn is_default_layer(name: &str) -> bool {
    let lower = name.trim().to_lowercase();
    matches!(
        lower.as_str(),
        "calque par défaut"
            | "calque par defaut"
            | "calque de dessin-1"
            | "calque-1"
            | "design layer-1"
            | "design layer 1"
            | "default"
            | "default layer"
            | "none"
            | "unassigned"
            | ""
    )
}

fn is_model_group_match(group_name: &str, model_name: &str) -> bool {
    let g = group_name.trim().to_lowercase();
    let m = model_name.trim().to_lowercase();
    if g.is_empty() || m.is_empty() {
        return false;
    }
    if g == m {
        return true;
    }
    if m.starts_with(&g) || m.ends_with(&g) || g.starts_with(&m) || g.ends_with(&m) {
        return true;
    }
    let g_words: HashSet<&str> = g.split_whitespace().collect();
    let m_words: HashSet<&str> = m.split_whitespace().collect();
    !g_words.is_disjoint(&m_words) && g_words.iter().any(|w| w.len() >= 3 && m_words.contains(w))
}

fn find_matching_group_id(groups: &[FixtureGroup], model_name: &str) -> Option<u32> {
    let model_lower = model_name.trim().to_lowercase();
    if model_lower.is_empty() {
        return None;
    }

    // 1. Exact match (case-insensitive)
    for g in groups {
        if g.id == 0 {
            continue;
        }
        if g.name.trim().eq_ignore_ascii_case(&model_lower) {
            return Some(g.id);
        }
    }

    // 2. Fuzzy / prefix / word-level match
    for g in groups {
        if g.id == 0 {
            continue;
        }
        if is_model_group_match(&g.name, &model_lower) {
            return Some(g.id);
        }
    }

    None
}

fn extract_model_name(fixture: &MvrFixture) -> &str {
    let name = fixture.name.trim();
    if !name.is_empty() {
        name
    } else if let Some(ref spec) = fixture.gdtf_spec {
        spec.split('@').next_back().unwrap_or(spec).trim()
    } else {
        "Fixture"
    }
}

/// Allocates fixture IDs preserving plot FixtureIDs where valid, positive, and unique,
/// falling back to sequential allocation for missing or conflicting IDs.
pub fn allocate_fixture_ids(fixtures: &[MvrFixture]) -> Vec<u32> {
    let candidate_ids: Vec<Option<u32>> = fixtures
        .iter()
        .map(|f| {
            f.fixture_id_numeric
                .or_else(|| {
                    f.fixture_id
                        .as_deref()
                        .and_then(|s| s.trim().parse::<u32>().ok())
                })
                .filter(|&id| id > 0)
        })
        .collect();

    let mut id_counts: HashMap<u32, usize> = HashMap::new();
    for &cand in candidate_ids.iter().flatten() {
        *id_counts.entry(cand).or_default() += 1;
    }

    let mut used_ids: HashSet<u32> = HashSet::new();
    let mut allocated_ids: Vec<Option<u32>> = vec![None; fixtures.len()];

    // Pass 1: Assign unique candidate IDs
    for (i, &cand_opt) in candidate_ids.iter().enumerate() {
        if let Some(cand) = cand_opt {
            if id_counts.get(&cand) == Some(&1) {
                allocated_ids[i] = Some(cand);
                used_ids.insert(cand);
            }
        }
    }

    // Pass 2: Assign next available sequential IDs for unassigned or colliding fixtures
    let mut next_id: u32 = 1;
    for slot in allocated_ids.iter_mut() {
        if slot.is_none() {
            while used_ids.contains(&next_id) {
                next_id += 1;
            }
            *slot = Some(next_id);
            used_ids.insert(next_id);
            next_id += 1;
        }
    }

    allocated_ids
        .into_iter()
        .map(|opt| opt.unwrap_or(1))
        .collect()
}

/// Disambiguates duplicate fixture names by appending the FixtureID or allocated ID
/// if the name is not already unique across the show and does not end with its ID.
pub fn disambiguate_fixture_name(
    raw_name: &str,
    fid: Option<&str>,
    allocated_id: u32,
    raw_name_count: usize,
) -> String {
    let trimmed = raw_name.trim();
    let id_str = fid
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| allocated_id.to_string());

    if trimmed.is_empty() {
        return format!("Fixture {}", id_str);
    }

    if raw_name_count <= 1 {
        return trimmed.to_string();
    }

    let id_suffix = format!(" {}", id_str);
    if trimmed.ends_with(&id_suffix) {
        trimmed.to_string()
    } else {
        format!("{} {}", trimmed, id_str)
    }
}

/// Parses an in-memory MVR ZIP archive and builds a ShowData structure.
pub fn import_mvr_from_bytes(data: &[u8], show_name: &str) -> Result<ShowData, String> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to read MVR zip archive: {}", e))?;

    let mut scene_xml: Option<String> = None;
    let mut gdtf_files: HashMap<String, ParsedGdtfFixture> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry #{}: {}", i, e))?;

        let entry_name = file.name().to_string();
        let lower_name = entry_name.to_lowercase();

        if lower_name == "generalscenedescription.xml"
            || lower_name.ends_with("/generalscenedescription.xml")
            || (scene_xml.is_none() && lower_name.ends_with("description.xml"))
        {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read {}: {}", entry_name, e))?;
            scene_xml = Some(content);
        } else if lower_name.ends_with(".gdtf") {
            let mut gdtf_bytes = Vec::new();
            if file.read_to_end(&mut gdtf_bytes).is_ok() {
                if let Ok(parsed) = parse_gdtf_archive(&gdtf_bytes) {
                    let base_name = Path::new(&entry_name)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(&entry_name)
                        .to_lowercase();

                    let stem = Path::new(&base_name)
                        .file_stem()
                        .and_then(|f| f.to_str())
                        .unwrap_or(&base_name)
                        .to_lowercase();

                    let key_mfg_name = format!(
                        "{}@{}",
                        parsed.manufacturer.to_lowercase(),
                        parsed.name.to_lowercase()
                    );
                    let key_name_only = parsed.name.to_lowercase();

                    gdtf_files.insert(base_name, parsed.clone());
                    gdtf_files.insert(stem, parsed.clone());
                    gdtf_files.insert(key_mfg_name, parsed.clone());
                    gdtf_files.insert(key_name_only, parsed);
                }
            }
        }
    }

    let xml = scene_xml
        .ok_or_else(|| "GeneralSceneDescription.xml not found in MVR archive".to_string())?;

    let scene = parse_mvr_xml(&xml)?;

    let mut show_groups = vec![FixtureGroup {
        id: 0,
        name: "All".to_string(),
    }];
    let mut next_group_id: u32 = 1;

    // 1. Separate layers into non-default layers and default layer fixtures
    let mut default_layer_fixtures: Vec<MvrFixture> = Vec::new();
    let mut fixture_layer_pairs: Vec<(MvrFixture, Option<u32>)> = Vec::new();

    for layer in scene.layers {
        if layer.fixtures.is_empty() {
            continue;
        }

        let clean_layer_name = sanitize_group_name(&layer.name);
        if is_default_layer(&clean_layer_name) {
            default_layer_fixtures.extend(layer.fixtures);
        } else {
            let gid = next_group_id;
            next_group_id += 1;
            show_groups.push(FixtureGroup {
                id: gid,
                name: clean_layer_name,
            });
            for fix in layer.fixtures {
                fixture_layer_pairs.push((fix, Some(gid)));
            }
        }
    }

    // 2. Resolve default layer fixtures: merge with existing model group or create new model group
    for fix in default_layer_fixtures {
        let model_name = extract_model_name(&fix);
        let gid = if let Some(matched_id) = find_matching_group_id(&show_groups, model_name) {
            matched_id
        } else {
            let new_gid = next_group_id;
            next_group_id += 1;
            let clean_name = sanitize_group_name(model_name);
            show_groups.push(FixtureGroup {
                id: new_gid,
                name: clean_name,
            });
            new_gid
        };
        fixture_layer_pairs.push((fix, Some(gid)));
    }

    // 3. Count raw fixture names to detect duplicates
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for (f, _) in &fixture_layer_pairs {
        let trimmed = f.name.trim().to_string();
        if !trimmed.is_empty() {
            *name_counts.entry(trimmed).or_default() += 1;
        }
    }

    // 4. Allocate fixture IDs preserving plot FIDs where unique
    let just_fixtures: Vec<MvrFixture> =
        fixture_layer_pairs.iter().map(|(f, _)| f.clone()).collect();
    let allocated_ids = allocate_fixture_ids(&just_fixtures);

    // 5. Build patched fixtures
    let mut patched_fixtures = Vec::new();
    let mut fixture_definitions: HashMap<String, EmbeddedGdtfDefinition> = HashMap::new();
    let mut used_universes: HashSet<u32> = HashSet::new();
    used_universes.insert(1);

    // Pre-reserve explicit address spans across all universes
    let mut occupied_ranges: HashMap<u32, Vec<(u32, u32)>> = HashMap::new();
    for (fix, _) in &fixture_layer_pairs {
        if let Some(addr) = fix
            .addresses
            .iter()
            .find(|a| a.break_id == 0 || a.break_id == 1)
            .or_else(|| fix.addresses.first())
        {
            let u = addr.universe.clamp(1, 512);
            let a = addr.address.clamp(1, 512);
            let (_, mode) = resolve_fixture_gdtf_and_mode(fix, &gdtf_files);
            let channel_count = (mode.channel_count as u32).clamp(1, 512);
            let end = (a + channel_count - 1).min(512);
            occupied_ranges.entry(u).or_default().push((a, end));
        }
    }

    for ((mvr_fixture, assigned_group_id), allocated_id) in
        fixture_layer_pairs.into_iter().zip(allocated_ids)
    {
        let (gdtf_def, mode) = resolve_fixture_gdtf_and_mode(&mvr_fixture, &gdtf_files);

        // Register fixture definition into show definitions
        let def_key = format!(
            "{}@{}",
            gdtf_def.manufacturer.to_lowercase(),
            gdtf_def.name.to_lowercase()
        );
        if !fixture_definitions.contains_key(&def_key) {
            fixture_definitions.insert(
                def_key.clone(),
                EmbeddedGdtfDefinition {
                    name: gdtf_def.name.clone(),
                    short_name: gdtf_def.short_name.clone(),
                    manufacturer: gdtf_def.manufacturer.clone(),
                    description: gdtf_def.description.clone(),
                    fixture_type_id: gdtf_def.fixture_type_id.clone(),
                    rid: None,
                    modes: gdtf_def
                        .modes
                        .iter()
                        .map(|m| EmbeddedGdtfMode {
                            name: m.name.clone(),
                            short_name: m.short_name.clone(),
                            channel_count: m.channel_count,
                            channels: m.channels.clone(),
                        })
                        .collect(),
                },
            );
        }

        // Resolve universe and address
        let channel_count = (mode.channel_count as u32).clamp(1, 512);
        let (universe, address) = if let Some(addr) = mvr_fixture
            .addresses
            .iter()
            .find(|a| a.break_id == 0 || a.break_id == 1)
            .or_else(|| mvr_fixture.addresses.first())
        {
            (addr.universe.clamp(1, 512), addr.address.clamp(1, 512))
        } else {
            let (u, a) = find_first_free_span(&occupied_ranges, channel_count);
            let end = (a + channel_count - 1).min(512);
            occupied_ranges.entry(u).or_default().push((a, end));
            (u, a)
        };

        used_universes.insert(universe);

        // Disambiguate fixture name
        let raw_name = mvr_fixture.name.trim();
        let count = name_counts.get(raw_name).copied().unwrap_or(0);
        let name = disambiguate_fixture_name(
            raw_name,
            mvr_fixture.fixture_id.as_deref(),
            allocated_id,
            count,
        );

        let fixture_groups = if let Some(gid) = assigned_group_id {
            vec![gid]
        } else {
            vec![]
        };

        let fixture_type_val = serde_json::json!({
            "name": gdtf_def.name,
            "shortName": gdtf_def.short_name,
            "fixtureKey": gdtf_def.name.to_lowercase().replace(' ', "_"),
            "manufacturerKey": gdtf_def.manufacturer.to_lowercase().replace(' ', "_"),
        });

        let mode_val = serde_json::json!({
            "name": mode.name,
            "shortName": mode.short_name,
            "channels": mode.channels,
        });

        patched_fixtures.push(PatchedFixture {
            id: allocated_id,
            groups: fixture_groups,
            name,
            fixture_type: fixture_type_val,
            mode: mode_val,
            universe,
            address,
        });
    }

    // 6. Sort patched fixtures by Universe, Address, ID
    patched_fixtures.sort_by_key(|f| (f.universe, f.address, f.id));

    let mut sorted_universes: Vec<u32> = used_universes.into_iter().collect();
    sorted_universes.sort_unstable();

    let output_universes: Vec<UniverseOutputConfig> = sorted_universes
        .into_iter()
        .map(|u| UniverseOutputConfig {
            universe: u,
            output: "none".to_string(),
            enabled: true,
        })
        .collect();

    Ok(ShowData {
        status: ShowStatus::Loaded,
        identity: ShowIdentityState {
            display_name: if show_name.trim().is_empty() {
                "MVR Show".to_string()
            } else {
                show_name.trim().to_string()
            },
        },
        file: ShowChangesState {
            path: None,
            is_dirty: true,
            is_unsaved_changes_modal_open: false,
            can_undo: false,
            can_redo: false,
        },
        patch: ShowPatchState {
            fixtures: patched_fixtures,
        },
        groups: crate::show::types::PatchGroupsState {
            groups: show_groups,
        },
        output: ShowOutputState {
            universes: output_universes,
        },
        fixture_definitions,
    })
}

/// Imports an MVR file from the file system, optionally caching bundled GDTFs into the local app data.
pub fn import_mvr_file(path_str: &str, app: Option<&AppHandle>) -> Result<ShowData, String> {
    let path = Path::new(path_str);
    let bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read MVR file {:?}: {}", path_str, e))?;

    let show_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("MVR Show");

    // Optional: cache bundled GDTF files in user catalog if app is available
    if let Some(app_handle) = app {
        if let Ok(app_data) = app_handle.path().app_data_dir() {
            let cursor = Cursor::new(&bytes);
            if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
                let manager = crate::fixtures::gdtf_share::GdtfCatalogManager::new(&app_data);
                for i in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(i) {
                        let name = file.name().to_lowercase();
                        if name.ends_with(".gdtf") {
                            let mut gdtf_data = Vec::new();
                            if file.read_to_end(&mut gdtf_data).is_ok() {
                                if let Ok(parsed) = parse_gdtf_archive(&gdtf_data) {
                                    let _ = manager.save_downloaded_gdtf(
                                        &parsed.manufacturer,
                                        &parsed.name,
                                        &gdtf_data,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    import_mvr_from_bytes(&bytes, show_name)
}
