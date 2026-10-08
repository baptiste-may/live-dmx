use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::{Cursor, Read};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedGdtfMode {
    pub name: String,
    pub short_name: Option<String>,
    pub channel_count: usize,
    pub channels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_defaults: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutter_open_values: Option<std::collections::HashMap<usize, u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan_bounds: Option<[i32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tilt_bounds: Option<[i32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedGdtfFixture {
    pub name: String,
    pub short_name: Option<String>,
    pub manufacturer: String,
    pub description: Option<String>,
    pub fixture_type_id: Option<String>,
    pub categories: Option<Vec<String>>,
    pub modes: Vec<ParsedGdtfMode>,
}

pub fn parse_gdtf_dmx_value(val_str: &str, byte_index: usize, total_bytes: usize) -> u8 {
    let mut parts = val_str.split('/');
    let Ok(num) = parts.next().unwrap_or("0").trim().parse::<u64>() else {
        return 0;
    };
    let src_bytes = parts
        .next()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&n| (1..=4).contains(&n))
        .unwrap_or(1);
    let dst_bytes = total_bytes.clamp(1, 4);
    let scaled = if dst_bytes >= src_bytes {
        num << (8 * (dst_bytes - src_bytes))
    } else {
        num >> (8 * (src_bytes - dst_bytes))
    };
    let idx = byte_index.min(dst_bytes - 1);
    ((scaled >> (8 * (dst_bytes - 1 - idx))) & 0xFF) as u8
}

pub fn map_gdtf_attribute_to_name(attribute: &str, is_fine: bool) -> String {
    let (attr_name, has_fine_suffix) = if let Some(stripped) = attribute.strip_suffix("Fine") {
        (stripped, true)
    } else {
        (attribute, false)
    };

    let base = match attr_name {
        "ColorAdd_R" | "ColorAdd_RY" => "Red",
        "ColorAdd_G" => "Green",
        "ColorAdd_GY" => "Lime",
        "ColorAdd_B" | "ColorAdd_BC" => "Blue",
        "ColorAdd_W" | "ColorAdd_WW" | "ColorAdd_CW" => "White",
        "ColorAdd_A" => "Amber",
        "ColorAdd_UV" => "UV",
        "ColorSub_C" => "Cyan",
        "ColorSub_M" => "Magenta",
        "ColorSub_Y" => "Yellow",
        "Shutter1" | "Shutter1Strobe" | "StrobeRate" | "Shutter" => "Strobe",
        other => other,
    };

    if is_fine || has_fine_suffix {
        format!("{} Fine", base)
    } else {
        base.to_string()
    }
}

pub fn parse_physical_bounds(from_str: &str, to_str: &str) -> Option<[i32; 2]> {
    let from: f64 = from_str.trim().parse().ok()?;
    let to: f64 = to_str.trim().parse().ok()?;
    if from == to || !from.is_finite() || !to.is_finite() {
        return None;
    }
    let min = from.min(to);
    let max = from.max(to);
    if min >= 0.0 && max > 0.0 {
        let span = ((max - min) / 2.0).round() as i32;
        if span <= 0 {
            return None;
        }
        Some([-span, span])
    } else {
        let min_i = min.round() as i32;
        let max_i = max.round() as i32;
        if min_i == max_i {
            None
        } else {
            Some([min_i, max_i])
        }
    }
}

#[derive(Default, Debug, Clone)]
struct ModeMovementBounds {
    pan: Option<[i32; 2]>,
    tilt: Option<[i32; 2]>,
}

#[allow(clippy::too_many_arguments)]
fn handle_logical_channel(
    e: &quick_xml::events::BytesStart,
    current_channel_offsets: &[usize],
    break_offsets: &[usize],
    current_channel_geometry: &str,
    fixture_attributes: &mut HashSet<String>,
    current_channels_map: &mut BTreeMap<usize, String>,
    geometry_nofeature_counts: &mut std::collections::HashMap<String, usize>,
    bounds: &mut ModeMovementBounds,
) -> Option<String> {
    let mut attribute_name = String::new();
    let mut phys_from = None;
    let mut phys_to = None;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"Attribute" => {
                attribute_name = attr.unescape_value().unwrap_or_default().into_owned();
            }
            b"PhysicalFrom" => {
                phys_from = Some(attr.unescape_value().unwrap_or_default().into_owned());
            }
            b"PhysicalTo" => {
                phys_to = Some(attr.unescape_value().unwrap_or_default().into_owned());
            }
            _ => {}
        }
    }

    if !attribute_name.is_empty() {
        fixture_attributes.insert(attribute_name.clone());

        let channel_name_prefix = if attribute_name == "NoFeature" {
            let count = geometry_nofeature_counts
                .entry(current_channel_geometry.to_string())
                .or_insert(0);
            let idx_in_geom = *count;
            *count += 1;

            if current_channel_geometry.contains("RGBW") {
                let colors = ["Red", "Green", "Blue", "White"];
                Some(colors[idx_in_geom % 4].to_string())
            } else if current_channel_geometry.contains("RGBA") {
                let colors = ["Red", "Green", "Blue", "Amber"];
                Some(colors[idx_in_geom % 4].to_string())
            } else if current_channel_geometry.contains("RGB") {
                let colors = ["Red", "Green", "Blue"];
                Some(colors[idx_in_geom % 3].to_string())
            } else {
                None
            }
        } else {
            None
        };

        for (idx, &offset) in current_channel_offsets.iter().enumerate() {
            let is_fine = idx > 0 || attribute_name.ends_with("Fine");
            let channel_name = if let Some(ref base) = channel_name_prefix {
                if is_fine {
                    format!("{} Fine", base)
                } else {
                    base.clone()
                }
            } else {
                let base_name = map_gdtf_attribute_to_name(&attribute_name, is_fine);
                let geom_lower = current_channel_geometry.to_lowercase();
                if geom_lower.contains("beam") && !geom_lower.contains("plate") {
                    if base_name == "Dimmer" {
                        "Dimmer Beam".to_string()
                    } else if base_name == "White" {
                        "White (Beam)".to_string()
                    } else {
                        base_name
                    }
                } else if geom_lower.contains("plate") && !geom_lower.contains("beam") {
                    if base_name == "Dimmer" {
                        "Dimmer Plate".to_string()
                    } else {
                        base_name
                    }
                } else {
                    base_name
                }
            };

            for &bo in break_offsets {
                let inst_offset = bo + offset - 1;
                current_channels_map.insert(inst_offset, channel_name.clone());
            }
        }

        if let (Some(f), Some(t)) = (phys_from, phys_to) {
            if attribute_name == "Pan" && bounds.pan.is_none() {
                bounds.pan = parse_physical_bounds(&f, &t);
            } else if attribute_name == "Tilt" && bounds.tilt.is_none() {
                bounds.tilt = parse_physical_bounds(&f, &t);
            }
        }

        Some(attribute_name)
    } else {
        None
    }
}

fn is_shutter_open_name(lname: &str) -> bool {
    let effects = ["pulse", "ramp", "strobe", "random", "effect", "close"];
    if effects.iter().any(|k| lname.contains(k)) {
        return false;
    }
    lname.contains("open")
        || lname.contains("no function")
        || lname == "on"
        || lname.contains("led on")
}

#[allow(clippy::too_many_arguments)]
fn handle_channel_function(
    e: &quick_xml::events::BytesStart,
    current_logical_attribute: Option<&str>,
    current_channel_offsets: &[usize],
    break_offsets: &[usize],
    current_channels_map: &mut BTreeMap<usize, String>,
    current_channel_defaults_map: &mut BTreeMap<usize, u8>,
    current_shutter_open_map: &mut std::collections::HashMap<usize, u8>,
    bounds: &mut ModeMovementBounds,
) -> Option<(String, bool)> {
    let mut func_attr = current_logical_attribute.map(str::to_string);
    let mut func_name = None;
    let mut default_str = None;
    let mut dmx_from_str = None;
    let mut phys_from = None;
    let mut phys_to = None;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"Attribute" => {
                let val = attr.unescape_value().unwrap_or_default().into_owned();
                if !val.is_empty() {
                    func_attr = Some(val);
                }
            }
            b"Name" => {
                let val = attr.unescape_value().unwrap_or_default().into_owned();
                if !val.is_empty() {
                    func_name = Some(val);
                }
            }
            b"Default" => {
                let val = attr.unescape_value().unwrap_or_default().into_owned();
                if !val.is_empty() {
                    default_str = Some(val);
                }
            }
            b"DMXFrom" => {
                let val = attr.unescape_value().unwrap_or_default().into_owned();
                if !val.is_empty() {
                    dmx_from_str = Some(val);
                }
            }
            b"PhysicalFrom" => {
                phys_from = Some(attr.unescape_value().unwrap_or_default().into_owned());
            }
            b"PhysicalTo" => {
                phys_to = Some(attr.unescape_value().unwrap_or_default().into_owned());
            }
            _ => {}
        }
    }

    if let (Some(f), Some(t)) = (phys_from, phys_to) {
        if let Some(ref attr) = func_attr {
            if attr == "Pan" && bounds.pan.is_none() {
                bounds.pan = parse_physical_bounds(&f, &t);
            } else if attr == "Tilt" && bounds.tilt.is_none() {
                bounds.tilt = parse_physical_bounds(&f, &t);
            }
        }
    }

    if let Some(ref name) = func_name {
        let lname = name.to_lowercase();
        if lname.contains("beam") && (lname.contains("dimmer") || lname.contains("intensity")) {
            for &bo in break_offsets {
                for (idx, &offset) in current_channel_offsets.iter().enumerate() {
                    let inst_offset = bo + offset - 1;
                    let ch_name = if idx > 0 {
                        "Dimmer Beam Fine".to_string()
                    } else {
                        "Dimmer Beam".to_string()
                    };
                    current_channels_map.insert(inst_offset, ch_name);
                }
            }
        } else if lname.contains("beam") && lname.contains("white") {
            for &bo in break_offsets {
                for (idx, &offset) in current_channel_offsets.iter().enumerate() {
                    let inst_offset = bo + offset - 1;
                    let ch_name = if idx > 0 {
                        "White (Beam) Fine".to_string()
                    } else {
                        "White (Beam)".to_string()
                    };
                    current_channels_map.insert(inst_offset, ch_name);
                }
            }
        } else if lname.contains("plate")
            && (lname.contains("dimmer") || lname.contains("intensity"))
        {
            for &bo in break_offsets {
                for (idx, &offset) in current_channel_offsets.iter().enumerate() {
                    let inst_offset = bo + offset - 1;
                    let ch_name = if idx > 0 {
                        "Dimmer Plate Fine".to_string()
                    } else {
                        "Dimmer Plate".to_string()
                    };
                    current_channels_map.insert(inst_offset, ch_name);
                }
            }
        }
    }

    if let Some(ref def_s) = default_str {
        for (idx, &offset) in current_channel_offsets.iter().enumerate() {
            let byte_val = parse_gdtf_dmx_value(def_s, idx, current_channel_offsets.len());
            for &bo in break_offsets {
                let inst_offset = bo + offset - 1;
                current_channel_defaults_map
                    .entry(inst_offset)
                    .or_insert(byte_val);
            }
        }
    }

    let is_shutter = func_attr
        .as_deref()
        .map(|a| {
            let l = a.to_lowercase();
            l.contains("shutter") || l.contains("strobe")
        })
        .unwrap_or(false)
        || current_logical_attribute
            .map(|a| {
                let l = a.to_lowercase();
                l.contains("shutter") || l.contains("strobe")
            })
            .unwrap_or(false);

    if is_shutter {
        if let Some(ref name) = func_name {
            let lname = name.to_lowercase();
            if is_shutter_open_name(&lname) {
                let open_val = default_str
                    .as_deref()
                    .or(dmx_from_str.as_deref())
                    .map(|s| parse_gdtf_dmx_value(s, 0, current_channel_offsets.len().max(1)))
                    .unwrap_or(255);
                for &bo in break_offsets {
                    if let Some(&first_off) = current_channel_offsets.first() {
                        let ch_idx = (bo + first_off - 1).saturating_sub(1);
                        current_shutter_open_map.insert(ch_idx, open_val);
                    }
                }
            }
        }
        if let Some(ref def_s) = default_str {
            let def_val = parse_gdtf_dmx_value(def_s, 0, current_channel_offsets.len().max(1));
            if def_val > 0 {
                for &bo in break_offsets {
                    if let Some(&first_off) = current_channel_offsets.first() {
                        let ch_idx = (bo + first_off - 1).saturating_sub(1);
                        current_shutter_open_map.entry(ch_idx).or_insert(def_val);
                    }
                }
            }
        }
    }

    func_attr.map(|a| (a, is_shutter))
}

fn handle_channel_set(
    e: &quick_xml::events::BytesStart,
    is_shutter: bool,
    current_channel_offsets: &[usize],
    break_offsets: &[usize],
    current_shutter_open_map: &mut std::collections::HashMap<usize, u8>,
) {
    if !is_shutter {
        return;
    }
    let mut name = String::new();
    let mut dmx_from = String::new();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"Name" => name = attr.unescape_value().unwrap_or_default().into_owned(),
            b"DMXFrom" => dmx_from = attr.unescape_value().unwrap_or_default().into_owned(),
            _ => {}
        }
    }
    let lname = name.to_lowercase();
    if is_shutter_open_name(&lname) {
        let open_val = parse_gdtf_dmx_value(&dmx_from, 0, current_channel_offsets.len().max(1));
        for &bo in break_offsets {
            if let Some(&first_off) = current_channel_offsets.first() {
                let ch_idx = (bo + first_off - 1).saturating_sub(1);
                current_shutter_open_map.insert(ch_idx, open_val);
            }
        }
    }
}

pub fn parse_gdtf_archive(data: &[u8]) -> Result<ParsedGdtfFixture, String> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to read GDTF zip archive: {}", e))?;

    let mut description_xml = None;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;
        if file.name() == "description.xml" || file.name().ends_with("/description.xml") {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read description.xml: {}", e))?;
            description_xml = Some(content);
            break;
        }
    }

    let xml =
        description_xml.ok_or_else(|| "description.xml not found in GDTF archive".to_string())?;
    parse_gdtf_xml(&xml)
}

pub fn parse_gdtf_xml(xml: &str) -> Result<ParsedGdtfFixture, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut fixture_name = String::new();
    let mut fixture_short_name = None;
    let mut manufacturer = String::new();
    let mut description = None;
    let mut fixture_type_id = None;
    let mut categories: Option<Vec<String>> = None;
    let mut fixture_attributes: HashSet<String> = HashSet::new();

    let mut geometry_references: std::collections::HashMap<
        (Option<String>, String, usize),
        Vec<usize>,
    > = std::collections::HashMap::new();
    let mut current_geometry_reference_target: Option<String> = None;
    let mut current_root_geometry: Option<String> = None;
    let mut in_geometries = false;
    let mut geometries_depth: usize = 0;
    let mut current_mode_geometry: Option<String> = None;

    let mut modes = Vec::new();
    let mut current_mode_name: Option<String> = None;
    let mut current_mode_short_name: Option<String> = None;
    let mut current_channels_map: BTreeMap<usize, String> = BTreeMap::new();
    let mut current_channel_defaults_map: BTreeMap<usize, u8> = BTreeMap::new();
    let mut current_shutter_open_map: std::collections::HashMap<usize, u8> =
        std::collections::HashMap::new();
    let mut current_movement_bounds = ModeMovementBounds::default();
    let mut geometry_nofeature_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut current_logical_attribute: Option<String> = None;
    let mut current_function_is_shutter = false;

    let mut current_channel_offsets: Vec<usize> = Vec::new();
    let mut current_channel_geometry = String::new();
    let mut current_break_offsets: Vec<usize> = vec![1];
    let mut in_dmx_mode = false;
    let mut in_dmx_channel = false;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"FixtureType" => {
                    parse_fixture_type_attributes(
                        e,
                        &mut fixture_name,
                        &mut fixture_short_name,
                        &mut manufacturer,
                        &mut description,
                        &mut fixture_type_id,
                        &mut categories,
                    );
                }
                b"Geometries" => {
                    in_geometries = true;
                    geometries_depth = 0;
                    current_root_geometry = None;
                }
                b"GeometryReference" => {
                    if in_geometries {
                        geometries_depth += 1;
                    }
                    let mut target_geom = None;
                    let mut dmx_offset_attr = None;
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"Geometry" => {
                                target_geom =
                                    Some(attr.unescape_value().unwrap_or_default().into_owned())
                            }
                            b"DMXOffset" => {
                                dmx_offset_attr = attr
                                    .unescape_value()
                                    .unwrap_or_default()
                                    .parse::<usize>()
                                    .ok()
                            }
                            _ => {}
                        }
                    }
                    if let (Some(ref target), Some(off)) = (&target_geom, dmx_offset_attr) {
                        if off > 0 {
                            geometry_references
                                .entry((current_root_geometry.clone(), target.clone(), 1))
                                .or_default()
                                .push(off);
                        }
                    }
                    current_geometry_reference_target = target_geom;
                }
                b"Break" => {
                    if in_geometries {
                        geometries_depth += 1;
                    }
                    if let Some(ref target) = current_geometry_reference_target {
                        let mut dmx_break = 1;
                        let mut dmx_offset = None;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"DMXBreak" => {
                                    if let Ok(b) =
                                        attr.unescape_value().unwrap_or_default().parse::<usize>()
                                    {
                                        if b > 0 {
                                            dmx_break = b;
                                        }
                                    }
                                }
                                b"DMXOffset" => {
                                    dmx_offset = attr
                                        .unescape_value()
                                        .unwrap_or_default()
                                        .parse::<usize>()
                                        .ok();
                                }
                                _ => {}
                            }
                        }
                        if let Some(off) = dmx_offset {
                            if off > 0 {
                                geometry_references
                                    .entry((
                                        current_root_geometry.clone(),
                                        target.clone(),
                                        dmx_break,
                                    ))
                                    .or_default()
                                    .push(off);
                            }
                        }
                    }
                }
                b"DMXMode" => {
                    in_dmx_mode = true;
                    current_channels_map.clear();
                    current_channel_defaults_map.clear();
                    current_shutter_open_map.clear();
                    geometry_nofeature_counts.clear();
                    current_mode_name = None;
                    current_mode_short_name = None;
                    current_mode_geometry = None;
                    current_movement_bounds = ModeMovementBounds::default();
                    current_logical_attribute = None;
                    current_function_is_shutter = false;

                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let val = attr.unescape_value().unwrap_or_default().into_owned();
                        match key {
                            b"Name" => current_mode_name = Some(val),
                            b"ShortName" if !val.is_empty() => {
                                current_mode_short_name = Some(val);
                            }
                            b"Geometry" if !val.is_empty() => {
                                current_mode_geometry = Some(val);
                            }
                            _ => {}
                        }
                    }
                }
                b"DMXChannel" => {
                    if in_dmx_mode {
                        in_dmx_channel = true;
                        current_channel_offsets.clear();
                        current_channel_geometry.clear();
                        current_break_offsets = vec![1];
                        current_logical_attribute = None;
                        current_function_is_shutter = false;

                        let mut channel_dmx_break = 1;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"DMXBreak" => {
                                    if let Ok(b) =
                                        attr.unescape_value().unwrap_or_default().parse::<usize>()
                                    {
                                        if b > 0 {
                                            channel_dmx_break = b;
                                        }
                                    }
                                }
                                b"Offset" => {
                                    let offset_str =
                                        attr.unescape_value().unwrap_or_default().into_owned();
                                    for part in offset_str.split(',') {
                                        let trimmed = part.trim();
                                        if let Ok(num) = trimmed.parse::<usize>() {
                                            if num > 0 {
                                                current_channel_offsets.push(num);
                                            }
                                        }
                                    }
                                }
                                b"Geometry" => {
                                    current_channel_geometry =
                                        attr.unescape_value().unwrap_or_default().into_owned();
                                }
                                _ => {}
                            }
                        }

                        let direct_key = (
                            current_mode_geometry.clone(),
                            current_channel_geometry.clone(),
                            channel_dmx_break,
                        );
                        if let Some(offsets) = geometry_references.get(&direct_key) {
                            if !offsets.is_empty() {
                                current_break_offsets = offsets.clone();
                            }
                        } else {
                            let fallback: Vec<usize> = geometry_references
                                .iter()
                                .filter(|((_, target, b), _)| {
                                    target == &current_channel_geometry && *b == channel_dmx_break
                                })
                                .flat_map(|(_, offs)| offs.clone())
                                .collect();
                            if !fallback.is_empty() {
                                current_break_offsets = fallback;
                            }
                        }
                    }
                }
                _ if in_geometries => {
                    geometries_depth += 1;
                    if geometries_depth == 1 {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"Name" {
                                current_root_geometry =
                                    Some(attr.unescape_value().unwrap_or_default().into_owned());
                            }
                        }
                    }
                }
                b"LogicalChannel" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    current_logical_attribute = handle_logical_channel(
                        e,
                        &current_channel_offsets,
                        &current_break_offsets,
                        &current_channel_geometry,
                        &mut fixture_attributes,
                        &mut current_channels_map,
                        &mut geometry_nofeature_counts,
                        &mut current_movement_bounds,
                    );
                }
                b"ChannelFunction" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    current_function_is_shutter = handle_channel_function(
                        e,
                        current_logical_attribute.as_deref(),
                        &current_channel_offsets,
                        &current_break_offsets,
                        &mut current_channels_map,
                        &mut current_channel_defaults_map,
                        &mut current_shutter_open_map,
                        &mut current_movement_bounds,
                    )
                    .map(|(_, is_sh)| is_sh)
                    .unwrap_or(false);
                }
                b"ChannelSet" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    handle_channel_set(
                        e,
                        current_function_is_shutter,
                        &current_channel_offsets,
                        &current_break_offsets,
                        &mut current_shutter_open_map,
                    );
                }
                _ => {}
            },
            Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"FixtureType" => {
                    parse_fixture_type_attributes(
                        e,
                        &mut fixture_name,
                        &mut fixture_short_name,
                        &mut manufacturer,
                        &mut description,
                        &mut fixture_type_id,
                        &mut categories,
                    );
                }
                b"GeometryReference" => {
                    let mut target_geom = None;
                    let mut dmx_break = 1;
                    let mut dmx_offset_attr = None;
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"Geometry" => {
                                target_geom =
                                    Some(attr.unescape_value().unwrap_or_default().into_owned())
                            }
                            b"DMXBreak" => {
                                if let Ok(b) =
                                    attr.unescape_value().unwrap_or_default().parse::<usize>()
                                {
                                    if b > 0 {
                                        dmx_break = b;
                                    }
                                }
                            }
                            b"DMXOffset" => {
                                dmx_offset_attr = attr
                                    .unescape_value()
                                    .unwrap_or_default()
                                    .parse::<usize>()
                                    .ok()
                            }
                            _ => {}
                        }
                    }
                    if let (Some(ref target), Some(off)) = (&target_geom, dmx_offset_attr) {
                        if off > 0 {
                            geometry_references
                                .entry((current_root_geometry.clone(), target.clone(), dmx_break))
                                .or_default()
                                .push(off);
                        }
                    }
                }
                b"Break" => {
                    if let Some(ref target) = current_geometry_reference_target {
                        let mut dmx_break = 1;
                        let mut dmx_offset = None;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"DMXBreak" => {
                                    if let Ok(b) =
                                        attr.unescape_value().unwrap_or_default().parse::<usize>()
                                    {
                                        if b > 0 {
                                            dmx_break = b;
                                        }
                                    }
                                }
                                b"DMXOffset" => {
                                    dmx_offset = attr
                                        .unescape_value()
                                        .unwrap_or_default()
                                        .parse::<usize>()
                                        .ok();
                                }
                                _ => {}
                            }
                        }
                        if let Some(off) = dmx_offset {
                            if off > 0 {
                                geometry_references
                                    .entry((
                                        current_root_geometry.clone(),
                                        target.clone(),
                                        dmx_break,
                                    ))
                                    .or_default()
                                    .push(off);
                            }
                        }
                    }
                }
                b"LogicalChannel" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    current_logical_attribute = handle_logical_channel(
                        e,
                        &current_channel_offsets,
                        &current_break_offsets,
                        &current_channel_geometry,
                        &mut fixture_attributes,
                        &mut current_channels_map,
                        &mut geometry_nofeature_counts,
                        &mut current_movement_bounds,
                    );
                }
                b"ChannelFunction" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    current_function_is_shutter = handle_channel_function(
                        e,
                        current_logical_attribute.as_deref(),
                        &current_channel_offsets,
                        &current_break_offsets,
                        &mut current_channels_map,
                        &mut current_channel_defaults_map,
                        &mut current_shutter_open_map,
                        &mut current_movement_bounds,
                    )
                    .map(|(_, is_sh)| is_sh)
                    .unwrap_or(false);
                }
                b"ChannelSet" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    handle_channel_set(
                        e,
                        current_function_is_shutter,
                        &current_channel_offsets,
                        &current_break_offsets,
                        &mut current_shutter_open_map,
                    );
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"Geometries" => {
                    in_geometries = false;
                    geometries_depth = 0;
                    current_root_geometry = None;
                }
                b"GeometryReference" => {
                    current_geometry_reference_target = None;
                    if in_geometries && geometries_depth > 0 {
                        geometries_depth -= 1;
                        if geometries_depth == 0 {
                            current_root_geometry = None;
                        }
                    }
                }
                b"Break" => {
                    if in_geometries && geometries_depth > 0 {
                        geometries_depth -= 1;
                        if geometries_depth == 0 {
                            current_root_geometry = None;
                        }
                    }
                }
                b"LogicalChannel" => {
                    current_logical_attribute = None;
                }
                b"ChannelFunction" => {
                    current_function_is_shutter = false;
                }
                b"DMXChannel" => {
                    in_dmx_channel = false;
                    current_channel_offsets.clear();
                    current_channel_geometry.clear();
                    current_break_offsets = vec![1];
                    current_logical_attribute = None;
                    current_function_is_shutter = false;
                }
                b"DMXMode" => {
                    in_dmx_mode = false;
                    current_mode_geometry = None;
                    if let Some(mode_name) = current_mode_name.take() {
                        let max_offset = current_channels_map.keys().copied().max().unwrap_or(0);
                        let mut channels = Vec::with_capacity(max_offset);
                        let mut channel_defaults = Vec::with_capacity(max_offset);
                        for i in 1..=max_offset {
                            let ch_name = if let Some(ch) = current_channels_map.get(&i) {
                                ch.clone()
                            } else {
                                format!("Channel {}", i)
                            };
                            let def = current_channel_defaults_map
                                .get(&i)
                                .copied()
                                .unwrap_or_else(|| {
                                    let lower = ch_name.to_lowercase();
                                    if lower.contains("pan") || lower.contains("tilt") {
                                        if !lower.contains("fine")
                                            && !lower.contains("lsb")
                                            && !lower.contains("16 bit")
                                            && !lower.contains("16bit")
                                            && !lower.contains("16-bit")
                                        {
                                            128
                                        } else {
                                            0
                                        }
                                    } else if lower.starts_with("xyz") || lower.contains("xyz") {
                                        255
                                    } else {
                                        0
                                    }
                                });
                            channels.push(ch_name);
                            channel_defaults.push(def);
                        }

                        let shutter_open_values = if current_shutter_open_map.is_empty() {
                            None
                        } else {
                            Some(current_shutter_open_map.clone())
                        };

                        modes.push(ParsedGdtfMode {
                            name: mode_name,
                            short_name: current_mode_short_name.take(),
                            channel_count: max_offset,
                            channels,
                            channel_defaults: Some(channel_defaults),
                            shutter_open_values,
                            pan_bounds: current_movement_bounds.pan.take(),
                            tilt_bounds: current_movement_bounds.tilt.take(),
                        });
                    }
                }
                _ if in_geometries && geometries_depth > 0 => {
                    geometries_depth -= 1;
                    if geometries_depth == 0 {
                        current_root_geometry = None;
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "Error parsing XML at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => {}
        }
        buf.clear();
    }

    if fixture_name.is_empty() {
        return Err("GDTF FixtureType has no Name attribute".to_string());
    }

    let categories = categories
        .filter(|cats| !cats.is_empty())
        .or_else(|| infer_categories_from_attributes(&fixture_attributes));

    Ok(ParsedGdtfFixture {
        name: fixture_name,
        short_name: fixture_short_name,
        manufacturer,
        description,
        fixture_type_id,
        categories,
        modes,
    })
}

fn infer_categories_from_attributes(attributes: &HashSet<String>) -> Option<Vec<String>> {
    let mut categories = Vec::new();

    let has = |needle: &str| attributes.iter().any(|a| a.contains(needle));
    let has_color = attributes.iter().any(|a| a.starts_with("Color"));
    let has_strobe = attributes
        .iter()
        .any(|a| a.starts_with("Shutter") || a == "StrobeRate" || a.starts_with("Strobe"));
    let has_dimmer = attributes.iter().any(|a| a.starts_with("Dim"));
    let has_laser = has("Laser");

    if has("Pan") && has("Tilt") {
        categories.push("Moving Head".to_string());
    }
    if has_color {
        categories.push("Color Changer".to_string());
    }
    if has_strobe {
        categories.push("Strobe".to_string());
    }
    if has_dimmer {
        categories.push("Dimmer".to_string());
    }
    if has_laser {
        categories.push("Laser".to_string());
    }

    if categories.is_empty() {
        None
    } else {
        Some(categories)
    }
}

fn parse_fixture_type_attributes(
    e: &quick_xml::events::BytesStart,
    fixture_name: &mut String,
    fixture_short_name: &mut Option<String>,
    manufacturer: &mut String,
    description: &mut Option<String>,
    fixture_type_id: &mut Option<String>,
    categories: &mut Option<Vec<String>>,
) {
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref();
        let val = attr.unescape_value().unwrap_or_default().into_owned();
        match key {
            b"Name" => *fixture_name = val,
            b"ShortName" if !val.is_empty() => {
                *fixture_short_name = Some(val);
            }
            b"Manufacturer" => *manufacturer = val,
            b"Description" if !val.is_empty() => {
                *description = Some(val);
            }
            b"FixtureTypeID" if !val.is_empty() => {
                *fixture_type_id = Some(val);
            }
            b"Categories" if !val.is_empty() => {
                *categories = Some(
                    val.split(',')
                        .map(|c| c.trim().to_string())
                        .filter(|c| !c.is_empty())
                        .collect(),
                );
            }
            _ => {}
        }
    }
}
