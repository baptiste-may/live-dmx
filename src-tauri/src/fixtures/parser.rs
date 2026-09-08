use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedGdtfMode {
    pub name: String,
    pub short_name: Option<String>,
    pub channel_count: usize,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedGdtfFixture {
    pub name: String,
    pub short_name: Option<String>,
    pub manufacturer: String,
    pub description: Option<String>,
    pub fixture_type_id: Option<String>,
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

    let mut modes = Vec::new();
    let mut current_mode_name: Option<String> = None;
    let mut current_mode_short_name: Option<String> = None;
    let mut current_channels_map: BTreeMap<usize, String> = BTreeMap::new();

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
                    );
                }
                b"DMXMode" => {
                    in_dmx_mode = true;
                    current_channels_map.clear();
                    current_mode_name = None;
                    current_mode_short_name = None;

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
                    let mut attribute_name = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Attribute" {
                            attribute_name = attr.unescape_value().unwrap_or_default().into_owned();
                        }
                    }

                    if !attribute_name.is_empty() {
                        for (idx, &offset) in current_channel_offsets.iter().enumerate() {
                            let is_fine = idx > 0 || attribute_name.ends_with("Fine");
                            let channel_name = map_gdtf_attribute_to_name(&attribute_name, is_fine);
                            current_channels_map.insert(offset, channel_name);
                        }
                    }
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
                    );
                }
                b"LogicalChannel" if in_dmx_channel && !current_channel_offsets.is_empty() => {
                    let mut attribute_name = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Attribute" {
                            attribute_name = attr.unescape_value().unwrap_or_default().into_owned();
                        }
                    }

                    if !attribute_name.is_empty() {
                        for (idx, &offset) in current_channel_offsets.iter().enumerate() {
                            let is_fine = idx > 0 || attribute_name.ends_with("Fine");
                            let channel_name = map_gdtf_attribute_to_name(&attribute_name, is_fine);
                            current_channels_map.insert(offset, channel_name);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"DMXChannel" => {
                    in_dmx_channel = false;
                    current_channel_offsets.clear();
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

    Ok(ParsedGdtfFixture {
        name: fixture_name,
        short_name: fixture_short_name,
        manufacturer,
        description,
        fixture_type_id,
        modes,
    })
}

fn parse_fixture_type_attributes(
    e: &quick_xml::events::BytesStart,
    fixture_name: &mut String,
    fixture_short_name: &mut Option<String>,
    manufacturer: &mut String,
    description: &mut Option<String>,
    fixture_type_id: &mut Option<String>,
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
            _ => {}
        }
    }
}
