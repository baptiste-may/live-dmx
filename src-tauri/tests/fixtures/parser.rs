use live_dmx_lib::fixtures::{
    get_generic_fixtures, map_gdtf_attribute_to_name, parse_gdtf_archive, parse_gdtf_xml,
};
use std::io::Write;
use zip::write::SimpleFileOptions;

#[test]
fn test_map_gdtf_attribute_to_name() {
    assert_eq!(map_gdtf_attribute_to_name("Dimmer", false), "Dimmer");
    assert_eq!(map_gdtf_attribute_to_name("ColorAdd_R", false), "Red");
    assert_eq!(map_gdtf_attribute_to_name("ColorAdd_G", false), "Green");
    assert_eq!(map_gdtf_attribute_to_name("ColorAdd_B", false), "Blue");
    assert_eq!(map_gdtf_attribute_to_name("ColorAdd_W", false), "White");
    assert_eq!(map_gdtf_attribute_to_name("ColorAdd_A", false), "Amber");
    assert_eq!(map_gdtf_attribute_to_name("ColorAdd_UV", false), "UV");
    assert_eq!(map_gdtf_attribute_to_name("Pan", false), "Pan");
    assert_eq!(map_gdtf_attribute_to_name("Pan", true), "Pan Fine");
    assert_eq!(map_gdtf_attribute_to_name("Tilt", false), "Tilt");
    assert_eq!(map_gdtf_attribute_to_name("Tilt", true), "Tilt Fine");
    assert_eq!(map_gdtf_attribute_to_name("Shutter1", false), "Strobe");
}

#[test]
fn test_parse_gdtf_xml_success() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GDTF DataVersion="1.2">
    <FixtureType Name="Robin Pointe" ShortName="Pointe" Manufacturer="Robe Lighting" Description="Spot and Beam" FixtureTypeID="uuid-1234">
        <DMXModes>
            <DMXMode Name="Standard 24ch" ShortName="Std">
                <DMXChannels>
                    <DMXChannel Offset="1,2">
                        <LogicalChannel Attribute="Pan"/>
                    </DMXChannel>
                    <DMXChannel Offset="3,4">
                        <LogicalChannel Attribute="Tilt"/>
                    </DMXChannel>
                    <DMXChannel Offset="5">
                        <LogicalChannel Attribute="ColorAdd_R"/>
                    </DMXChannel>
                    <DMXChannel Offset="6">
                        <LogicalChannel Attribute="ColorAdd_G"/>
                    </DMXChannel>
                    <DMXChannel Offset="7">
                        <LogicalChannel Attribute="ColorAdd_B"/>
                    </DMXChannel>
                    <DMXChannel Offset="8">
                        <LogicalChannel Attribute="Dimmer"/>
                    </DMXChannel>
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#;

    let parsed = parse_gdtf_xml(xml).expect("Should parse valid GDTF XML");
    assert_eq!(parsed.name, "Robin Pointe");
    assert_eq!(parsed.short_name, Some("Pointe".to_string()));
    assert_eq!(parsed.manufacturer, "Robe Lighting");
    assert_eq!(parsed.description, Some("Spot and Beam".to_string()));
    assert_eq!(parsed.fixture_type_id, Some("uuid-1234".to_string()));
    assert_eq!(parsed.modes.len(), 1);

    let mode = &parsed.modes[0];
    assert_eq!(mode.name, "Standard 24ch");
    assert_eq!(mode.channel_count, 8);
    assert_eq!(
        mode.channels,
        vec![
            "Pan".to_string(),
            "Pan Fine".to_string(),
            "Tilt".to_string(),
            "Tilt Fine".to_string(),
            "Red".to_string(),
            "Green".to_string(),
            "Blue".to_string(),
            "Dimmer".to_string(),
        ]
    );
}

#[test]
fn test_parse_gdtf_archive_in_memory_zip() {
    let xml = r#"<GDTF DataVersion="1.2">
    <FixtureType Name="Test Fixture" Manufacturer="Test Manuf">
        <DMXModes>
            <DMXMode Name="Mode 1">
                <DMXChannels>
                    <DMXChannel Offset="1">
                        <LogicalChannel Attribute="Dimmer"/>
                    </DMXChannel>
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#;

    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip_writer = zip::ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();
        zip_writer.start_file("description.xml", options).unwrap();
        zip_writer.write_all(xml.as_bytes()).unwrap();
        zip_writer.finish().unwrap();
    }

    let parsed = parse_gdtf_archive(&buf).expect("Should parse GDTF zip archive");
    assert_eq!(parsed.name, "Test Fixture");
    assert_eq!(parsed.manufacturer, "Test Manuf");
    assert_eq!(parsed.modes.len(), 1);
    assert_eq!(parsed.modes[0].channels, vec!["Dimmer".to_string()]);
}

#[test]
fn test_generic_fixtures_available() {
    let generics = get_generic_fixtures();
    assert!(!generics.is_empty());

    let dimmer = generics.iter().find(|g| g.name == "Dimmer").unwrap();
    assert_eq!(dimmer.manufacturer, "Generic");
    assert_eq!(dimmer.modes[0].channel_count, 1);
    assert_eq!(dimmer.modes[0].channels, vec!["Dimmer".to_string()]);

    let rgb = generics.iter().find(|g| g.name == "RGB PAR").unwrap();
    assert_eq!(rgb.modes[0].channel_count, 3);
    assert_eq!(
        rgb.modes[0].channels,
        vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()]
    );
}
