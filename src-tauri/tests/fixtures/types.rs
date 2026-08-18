use live_dmx_lib::fixtures::{
    Capability, ChannelRef, Fixture, FixtureLinks, FixtureMeta, FixturePhysical, FixtureWheel,
    GoboReference, ManufacturerParsed, Mode, NumberOrString, RawFixture, StringOrBool,
    StringOrVecString,
};

#[test]
fn test_number_or_string_enum() {
    let num_json = "42.5";
    let parsed_num: NumberOrString = serde_json::from_str(num_json).unwrap();
    assert_eq!(parsed_num, NumberOrString::Number(42.5));

    let str_json = "\"fast\"";
    let parsed_str: NumberOrString = serde_json::from_str(str_json).unwrap();
    assert_eq!(parsed_str, NumberOrString::String("fast".to_string()));

    let serialized = serde_json::to_string(&parsed_num).unwrap();
    assert_eq!(serialized, "42.5");
}

#[test]
fn test_string_or_bool_enum() {
    let bool_json = "true";
    let parsed_bool: StringOrBool = serde_json::from_str(bool_json).unwrap();
    assert_eq!(parsed_bool, StringOrBool::Bool(true));

    let str_json = "\"Help needed for modes\"";
    let parsed_str: StringOrBool = serde_json::from_str(str_json).unwrap();
    assert_eq!(
        parsed_str,
        StringOrBool::String("Help needed for modes".to_string())
    );
}

#[test]
fn test_string_or_vec_string_enum() {
    let single_json = "\"pan\"";
    let parsed_single: StringOrVecString = serde_json::from_str(single_json).unwrap();
    assert_eq!(parsed_single, StringOrVecString::String("pan".to_string()));

    let vec_json = "[\"pan\", \"tilt\"]";
    let parsed_vec: StringOrVecString = serde_json::from_str(vec_json).unwrap();
    assert_eq!(
        parsed_vec,
        StringOrVecString::Vec(vec!["pan".to_string(), "tilt".to_string()])
    );
}

#[test]
fn test_fixture_meta_and_import_plugin() {
    let json_data = r#"{
        "authors": ["Alice", "Bob"],
        "createDate": "2026-01-01",
        "lastModifyDate": "2026-02-01",
        "importPlugin": {
            "plugin": "qlcplus",
            "date": "2026-01-01",
            "comment": "Imported from QLC+"
        }
    }"#;

    let meta: FixtureMeta = serde_json::from_str(json_data).unwrap();
    assert_eq!(meta.authors, vec!["Alice", "Bob"]);
    assert_eq!(meta.create_date, "2026-01-01");
    assert_eq!(meta.last_modify_date, "2026-02-01");
    assert!(meta.import_plugin.is_some());
    let plugin = meta.import_plugin.unwrap();
    assert_eq!(plugin.plugin, "qlcplus");
    assert_eq!(plugin.comment.as_deref(), Some("Imported from QLC+"));
}

#[test]
fn test_fixture_links() {
    let json_data = r#"{
        "manual": ["https://example.com/manual.pdf"],
        "productPage": ["https://example.com/product"],
        "video": ["https://youtube.com/watch?v=123"],
        "forum": ["https://forum.example.com/topic/1"]
    }"#;

    let links: FixtureLinks = serde_json::from_str(json_data).unwrap();
    assert_eq!(
        links.manual,
        Some(vec!["https://example.com/manual.pdf".to_string()])
    );
    assert_eq!(
        links.product_page,
        Some(vec!["https://example.com/product".to_string()])
    );
    assert_eq!(
        links.video,
        Some(vec!["https://youtube.com/watch?v=123".to_string()])
    );
    assert_eq!(
        links.forum,
        Some(vec!["https://forum.example.com/topic/1".to_string()])
    );
}

#[test]
fn test_fixture_physical_properties() {
    let json_data = r#"{
        "dimensions": [300.0, 200.0, 150.0],
        "weight": 4.5,
        "power": 120.0,
        "DMXconnector": "3-pin",
        "bulb": {
            "type": "LED",
            "colorTemperature": 6500.0,
            "lumens": 3000.0
        },
        "lens": {
            "name": "Fresnel",
            "degreesMinMax": [15.0, 45.0]
        },
        "matrixPixels": {
            "dimensions": [10.0, 10.0, 5.0],
            "spacing": [2.0, 2.0, 0.0]
        }
    }"#;

    let phys: FixturePhysical = serde_json::from_str(json_data).unwrap();
    assert_eq!(phys.dimensions, Some((300.0, 200.0, 150.0)));
    assert_eq!(phys.weight, Some(4.5));
    assert_eq!(phys.power, Some(120.0));
    assert_eq!(phys.dmx_connector.as_deref(), Some("3-pin"));

    let bulb = phys.bulb.unwrap();
    assert_eq!(bulb.r#type.as_deref(), Some("LED"));
    assert_eq!(bulb.color_temperature, Some(6500.0));

    let lens = phys.lens.unwrap();
    assert_eq!(lens.name.as_deref(), Some("Fresnel"));
    assert_eq!(lens.degrees_min_max, Some((15.0, 45.0)));

    let matrix_pixels = phys.matrix_pixels.unwrap();
    assert_eq!(matrix_pixels.dimensions, Some((10.0, 10.0, 5.0)));
}

#[test]
fn test_fixture_wheel_and_gobo() {
    let json_data = r##"{
        "direction": "CW",
        "slots": [
            {
                "type": "Color",
                "name": "Red",
                "colors": ["#FF0000"]
            },
            {
                "type": "Gobo",
                "name": "Stars",
                "gobo": {
                    "name": "Dot Stars",
                    "resource": "gobo-stars.png"
                }
            }
        ]
    }"##;

    let wheel: FixtureWheel = serde_json::from_str(json_data).unwrap();
    assert_eq!(wheel.direction.as_deref(), Some("CW"));
    let slots = wheel.slots.unwrap();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].r#type, "Color");
    assert_eq!(slots[0].colors, Some(vec!["#FF0000".to_string()]));
    assert_eq!(slots[1].r#type, "Gobo");
    match &slots[1].gobo {
        Some(GoboReference::Slot(slot)) => {
            assert_eq!(slot.name.as_deref(), Some("Dot Stars"));
            assert_eq!(slot.resource.as_deref(), Some("gobo-stars.png"));
        }
        _ => panic!("Expected GoboReference::Slot"),
    }
}

#[test]
fn test_capability_deserialization() {
    let json_data = r#"{
        "dmxRange": [0, 255],
        "type": "Shutter",
        "shutterEffect": "Strobe",
        "soundControlled": true,
        "speed": "fast",
        "comment": "Strobe effect"
    }"#;

    let cap: Capability = serde_json::from_str(json_data).unwrap();
    assert_eq!(cap.dmx_range, Some((0, 255)));
    assert_eq!(cap.r#type, "Shutter");
    assert_eq!(cap.shutter_effect.as_deref(), Some("Strobe"));
    assert_eq!(cap.sound_controlled, Some(true));
    assert_eq!(cap.speed, Some(NumberOrString::String("fast".to_string())));
    assert_eq!(cap.comment.as_deref(), Some("Strobe effect"));
}

#[test]
fn test_channel_and_mode_deserialization() {
    let json_data = r#"{
        "name": "Standard Mode",
        "shortName": "std",
        "channels": [
            "dimmer",
            null,
            {
                "name": "Fine Pan",
                "dmxValueResolution": "16bit",
                "defaultValue": 128
            }
        ],
        "rdmPersonalityIndex": 1
    }"#;

    let mode: Mode = serde_json::from_str(json_data).unwrap();
    assert_eq!(mode.name, "Standard Mode");
    assert_eq!(mode.short_name.as_deref(), Some("std"));
    assert_eq!(mode.channels.len(), 3);
    assert_eq!(
        mode.channels[0],
        Some(ChannelRef::Key("dimmer".to_string()))
    );
    assert_eq!(mode.channels[1], None);
    match &mode.channels[2] {
        Some(ChannelRef::Object(named)) => {
            assert_eq!(named.name.as_deref(), Some("Fine Pan"));
            assert_eq!(named.default_value, Some(NumberOrString::Number(128.0)));
        }
        _ => panic!("Expected inline NamedChannel"),
    }
    assert_eq!(mode.rdm_personality_index, Some(1));
}

#[test]
fn test_raw_fixture_deserialization() {
    let json_data = r#"{
        "name": "SlimPAR Pro RGBA",
        "categories": ["Color Changer"],
        "modes": [
            {
                "name": "4-channel",
                "channels": ["Red", "Green", "Blue", "Amber"]
            }
        ]
    }"#;

    let raw: RawFixture = serde_json::from_str(json_data).expect("Should deserialize");
    assert_eq!(raw.name, "SlimPAR Pro RGBA");
    assert_eq!(raw.categories, vec!["Color Changer"]);
    assert_eq!(raw.modes.len(), 1);
    assert_eq!(raw.modes[0].name, "4-channel");

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
        fixture_key: raw
            .fixture_key
            .unwrap_or_else(|| "slimpar-pro-rgba".to_string()),
        manufacturer_key: raw
            .manufacturer_key
            .unwrap_or_else(|| "chauvet-dj".to_string()),
        ofl_url: raw.ofl_url,
        comment: raw.comment,
        help_wanted: raw.help_wanted,
        rdm: raw.rdm,
    };

    assert_eq!(fixture.fixture_key, "slimpar-pro-rgba");
    assert_eq!(fixture.manufacturer_key, "chauvet-dj");
}

#[test]
fn test_manufacturer_parsed() {
    let json_data = r#"{
        "name": "Chauvet DJ",
        "website": "https://www.chauvetdj.com",
        "comment": "Leading manufacturer",
        "rdmId": 16968
    }"#;

    let parsed: ManufacturerParsed = serde_json::from_str(json_data).unwrap();
    assert_eq!(parsed.name, "Chauvet DJ");
    assert_eq!(parsed.website.as_deref(), Some("https://www.chauvetdj.com"));
    assert_eq!(parsed.comment.as_deref(), Some("Leading manufacturer"));
    assert_eq!(parsed.rdm_id, Some(16968));
}
