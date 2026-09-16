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

pub fn map_gdtf_attribute_to_name(attribute: &str, is_fine: bool) -> String {
    let (attr_name, has_fine_suffix) = if let Some(stripped) = attribute.strip_suffix("Fine") {
        (stripped, true)
    } else {
        (attribute, false)
    };

    let base = match attr_name {
        "ColorAdd_R" | "ColorAdd_RY" => "Red",
        "ColorAdd_G" | "ColorAdd_GY" => "Green",
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

fn handle_logical_channel(
    e: &quick_xml::events::BytesStart,
    current_channel_offsets: &[usize],
    fixture_attributes: &mut HashSet<String>,
    current_channels_map: &mut BTreeMap<usize, String>,
    current_mode_pan_bounds: &mut Option<[i32; 2]>,
    current_mode_tilt_bounds: &mut Option<[i32; 2]>,
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

        for (idx, &offset) in current_channel_offsets.iter().enumerate() {
            let is_fine = idx > 0 || attribute_name.ends_with("Fine");
            let channel_name = map_gdtf_attribute_to_name(&attribute_name, is_fine);
            current_channels_map.insert(offset, channel_name);
        }

        if let (Some(f), Some(t)) = (phys_from, phys_to) {
            if attribute_name == "Pan" && current_mode_pan_bounds.is_none() {
                *current_mode_pan_bounds = parse_physical_bounds(&f, &t);
            } else if attribute_name == "Tilt" && current_mode_tilt_bounds.is_none() {
                *current_mode_tilt_bounds = parse_physical_bounds(&f, &t);
            }
        }

        Some(attribute_name)
    } else {
        None
    }
}

fn handle_channel_function(
    e: &quick_xml::events::BytesStart,
    current_logical_attribute: Option<&str>,
    current_mode_pan_bounds: &mut Option<[i32; 2]>,
    current_mode_tilt_bounds: &mut Option<[i32; 2]>,
) {
    let mut func_attr = current_logical_attribute.map(str::to_string);
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
            if attr == "Pan" && current_mode_pan_bounds.is_none() {
                *current_mode_pan_bounds = parse_physical_bounds(&f, &t);
            } else if attr == "Tilt" && current_mode_tilt_bounds.is_none() {
                *current_mode_tilt_bounds = parse_physical_bounds(&f, &t);
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

    let mut modes = Vec::new();
    let mut current_mode_name: Option<String> = None;
    let mut current_mode_short_name: Option<String> = None;
    let mut current_channels_map: BTreeMap<usize, String> = BTreeMap::new();
    let mut current_mode_pan_bounds: Option<[i32; 2]> = None;
    let mut current_mode_tilt_bounds: Option<[i32; 2]> = None;
    let mut current_logical_attribute: Option<String> = None;

    let mut current_channel_offsets: Vec<usize> = Vec::new();
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
                b"DMXMode" => {
                    in_dmx_mode = true;
                    current_channels_map.clear();
                    current_mode_name = None;
                    current_mode_short_name = None;
                    current_mode_pan_bounds = None;
                    current_mode_tilt_bounds = None;
                    current_logical_attribute = None;

                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let val = attr.unescape_value().unwrap_or_default().into_owned();
                        match key {
                            b"Name" => current_mode_name = Some(val),
                            b"ShortName" if !val.is_empty() => {
                                current_mode_short_name = Some(val);
                            }
                            _ => {}
                        }
                    }
                }
                b"DMXChannel" => {
                    if in_dmx_mode {
                        in_dmx_channel = true;
                        current_channel_offsets.clear();
                        current_logical_attribute = None;

                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"Offset" {
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
                        }
                    }
                }
                b"LogicalChannel" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    current_logical_attribute = handle_logical_channel(
                        e,
                        &current_channel_offsets,
                        &mut fixture_attributes,
                        &mut current_channels_map,
                        &mut current_mode_pan_bounds,
                        &mut current_mode_tilt_bounds,
                    );
                }
                b"ChannelFunction" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    handle_channel_function(
                        e,
                        current_logical_attribute.as_deref(),
                        &mut current_mode_pan_bounds,
                        &mut current_mode_tilt_bounds,
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
                b"LogicalChannel" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    current_logical_attribute = handle_logical_channel(
                        e,
                        &current_channel_offsets,
                        &mut fixture_attributes,
                        &mut current_channels_map,
                        &mut current_mode_pan_bounds,
                        &mut current_mode_tilt_bounds,
                    );
                }
                b"ChannelFunction" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    handle_channel_function(
                        e,
                        current_logical_attribute.as_deref(),
                        &mut current_mode_pan_bounds,
                        &mut current_mode_tilt_bounds,
                    );
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"LogicalChannel" => {
                    current_logical_attribute = None;
                }
                b"DMXChannel" => {
                    in_dmx_channel = false;
                    current_channel_offsets.clear();
                    current_logical_attribute = None;
                }
                b"DMXMode" => {
                    in_dmx_mode = false;
                    if let Some(mode_name) = current_mode_name.take() {
                        let max_offset = current_channels_map.keys().copied().max().unwrap_or(0);
                        let mut channels = Vec::with_capacity(max_offset);
                        for i in 1..=max_offset {
                            if let Some(ch) = current_channels_map.get(&i) {
                                channels.push(ch.clone());
                            } else {
                                channels.push(format!("Channel {}", i));
                            }
                        }

                        modes.push(ParsedGdtfMode {
                            name: mode_name,
                            short_name: current_mode_short_name.take(),
                            channel_count: max_offset,
                            channels,
                            pan_bounds: current_mode_pan_bounds.take(),
                            tilt_bounds: current_mode_tilt_bounds.take(),
                        });
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
