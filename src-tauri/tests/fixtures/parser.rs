use live_dmx_lib::fixtures::{
    get_generic_fixtures, map_gdtf_attribute_to_name, parse_gdtf_archive, parse_gdtf_xml,
    parse_physical_bounds,
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
    <FixtureType Name="Robin Pointe" ShortName="Pointe" Manufacturer="Robe Lighting" Description="Spot and Beam" FixtureTypeID="uuid-1234" Categories="Moving Head, Spot">
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
    assert_eq!(
        parsed.categories,
        Some(vec!["Moving Head".to_string(), "Spot".to_string()])
    );
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
fn test_parse_gdtf_xml_infers_categories_when_attribute_missing() {
    let xml = r#"<GDTF DataVersion="1.2">
    <FixtureType Name="Sharpy" Manufacturer="Clay Paky">
        <DMXModes>
            <DMXMode Name="Standard">
                <DMXChannels>
                    <DMXChannel Offset="1,2"><LogicalChannel Attribute="Pan"/></DMXChannel>
                    <DMXChannel Offset="3,4"><LogicalChannel Attribute="Tilt"/></DMXChannel>
                    <DMXChannel Offset="5"><LogicalChannel Attribute="ColorAdd_R"/></DMXChannel>
                    <DMXChannel Offset="6"><LogicalChannel Attribute="Dimmer"/></DMXChannel>
                    <DMXChannel Offset="7"><LogicalChannel Attribute="Shutter1Strobe"/></DMXChannel>
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#;

    let parsed = parse_gdtf_xml(xml).expect("Should parse GDTF XML");
    assert_eq!(
        parsed.categories,
        Some(vec![
            "Moving Head".to_string(),
            "Color Changer".to_string(),
            "Strobe".to_string(),
            "Dimmer".to_string(),
        ])
    );
}

#[test]
fn test_parse_gdtf_xml_keeps_explicit_categories_over_inference() {
    let xml = r#"<GDTF DataVersion="1.2">
    <FixtureType Name="Sharpy" Manufacturer="Clay Paky" Categories="Moving Head, Spot">
        <DMXModes>
            <DMXMode Name="Standard">
                <DMXChannels>
                    <DMXChannel Offset="1"><LogicalChannel Attribute="Dimmer"/></DMXChannel>
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#;

    let parsed = parse_gdtf_xml(xml).expect("Should parse GDTF XML");
    assert_eq!(
        parsed.categories,
        Some(vec!["Moving Head".to_string(), "Spot".to_string()])
    );
}

#[test]
fn test_parse_gdtf_xml_unknown_fixture_stays_without_categories() {
    let xml = r#"<GDTF DataVersion="1.2">
    <FixtureType Name="Mystery Device" Manufacturer="Acme">
        <DMXModes>
            <DMXMode Name="1ch">
                <DMXChannels>
                    <DMXChannel Offset="1"><LogicalChannel Attribute="NoFeature"/></DMXChannel>
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#;

    let parsed = parse_gdtf_xml(xml).expect("Should parse GDTF XML");
    assert_eq!(parsed.categories, None);
}

#[test]
fn test_generic_fixtures_available() {
    let generics = get_generic_fixtures();
    assert!(!generics.is_empty());

    let dimmer = generics.iter().find(|g| g.name == "Dimmer").unwrap();
    assert_eq!(dimmer.manufacturer, "Generic");
    assert_eq!(dimmer.categories, Some(vec!["Dimmer".to_string()]));
    assert_eq!(dimmer.modes[0].channel_count, 1);
    assert_eq!(dimmer.modes[0].channels, vec!["Dimmer".to_string()]);

    let rgb = generics.iter().find(|g| g.name == "RGB PAR").unwrap();
    assert_eq!(
        rgb.categories,
        Some(vec!["Color Changer".to_string(), "PAR".to_string()])
    );
    assert_eq!(rgb.modes[0].channel_count, 3);
    assert_eq!(
        rgb.modes[0].channels,
        vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()]
    );

    let mh_spot = generics
        .iter()
        .find(|g| g.name == "Moving Head Spot")
        .unwrap();
    assert_eq!(
        mh_spot.categories,
        Some(vec!["Moving Head".to_string(), "Spot".to_string()])
    );
    assert_eq!(mh_spot.modes[0].pan_bounds, Some([-270, 270]));
    assert_eq!(mh_spot.modes[0].tilt_bounds, Some([-135, 135]));

    let mh_wash = generics
        .iter()
        .find(|g| g.name == "Moving Head Wash")
        .unwrap();
    assert_eq!(mh_wash.modes[0].pan_bounds, Some([-270, 270]));
    assert_eq!(mh_wash.modes[0].tilt_bounds, Some([-135, 135]));
}

#[test]
fn test_parse_physical_bounds_centering_and_ranges() {
    // Zero-based 0..540 should center to -270..270
    assert_eq!(parse_physical_bounds("0", "540"), Some([-270, 270]));
    assert_eq!(parse_physical_bounds("0.0", "270.0"), Some([-135, 135]));
    assert_eq!(parse_physical_bounds("540", "0"), Some([-270, 270]));

    // Already signed -270..270
    assert_eq!(parse_physical_bounds("-270", "270"), Some([-270, 270]));
    assert_eq!(parse_physical_bounds("-135.0", "135.0"), Some([-135, 135]));
    assert_eq!(parse_physical_bounds("-110", "110"), Some([-110, 110]));
    assert_eq!(parse_physical_bounds("-30", "100"), Some([-30, 100]));

    // Invalid / identical
    assert_eq!(parse_physical_bounds("0", "0"), None);
    assert_eq!(parse_physical_bounds("invalid", "270"), None);
    assert_eq!(parse_physical_bounds("0", "0.2"), None);
}

#[test]
fn test_parse_gdtf_xml_with_pan_tilt_physical_bounds() {
    let xml = r#"<GDTF DataVersion="1.2">
    <FixtureType Name="Test Head" Manufacturer="TestCo">
        <DMXModes>
            <DMXMode Name="Standard 16ch">
                <DMXChannels>
                    <DMXChannel Offset="1,2">
                        <LogicalChannel Attribute="Pan">
                            <ChannelFunction Attribute="Pan" PhysicalFrom="-270.0" PhysicalTo="270.0"/>
                        </LogicalChannel>
                    </DMXChannel>
                    <DMXChannel Offset="3,4">
                        <LogicalChannel Attribute="Tilt">
                            <ChannelFunction Attribute="Tilt" PhysicalFrom="0.0" PhysicalTo="270.0"/>
                        </LogicalChannel>
                    </DMXChannel>
                    <DMXChannel Offset="5">
                        <LogicalChannel Attribute="Dimmer"/>
                    </DMXChannel>
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#;

    let parsed = parse_gdtf_xml(xml).expect("Should parse GDTF XML with bounds");
    assert_eq!(parsed.modes.len(), 1);
    let mode = &parsed.modes[0];
    assert_eq!(mode.pan_bounds, Some([-270, 270]));
    assert_eq!(mode.tilt_bounds, Some([-135, 135]));
}

#[test]
fn test_parse_gdtf_xml_with_logical_channel_bounds() {
    let xml = r#"<GDTF DataVersion="1.2">
    <FixtureType Name="Test Head 2" Manufacturer="TestCo">
        <DMXModes>
            <DMXMode Name="Basic">
                <DMXChannels>
                    <DMXChannel Offset="1">
                        <LogicalChannel Attribute="Pan" PhysicalFrom="-180" PhysicalTo="180"/>
                    </DMXChannel>
                    <DMXChannel Offset="2">
                        <LogicalChannel Attribute="Tilt" PhysicalFrom="-110" PhysicalTo="110"/>
                    </DMXChannel>
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#;

    let parsed = parse_gdtf_xml(xml).expect("Should parse GDTF XML with LogicalChannel bounds");
    assert_eq!(parsed.modes.len(), 1);
    let mode = &parsed.modes[0];
    assert_eq!(mode.pan_bounds, Some([-180, 180]));
    assert_eq!(mode.tilt_bounds, Some([-110, 110]));
}
