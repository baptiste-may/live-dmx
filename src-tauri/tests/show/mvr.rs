use std::io::Write;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use live_dmx_lib::show::mvr::{
    allocate_fixture_ids, disambiguate_fixture_name, import_mvr_from_bytes, is_default_layer,
    parse_dmx_address, parse_mvr_xml, sanitize_group_name, MvrFixture,
};
use live_dmx_lib::show::types::ShowStatus;

#[test]
fn test_parse_dmx_address() {
    assert_eq!(parse_dmx_address("1.1"), (1, 1));
    assert_eq!(parse_dmx_address("2.100"), (2, 100));
    assert_eq!(parse_dmx_address("3/25"), (3, 25));
    assert_eq!(parse_dmx_address("4,50"), (4, 50));
    assert_eq!(parse_dmx_address("1"), (1, 1));
    assert_eq!(parse_dmx_address("512"), (1, 512));
    assert_eq!(parse_dmx_address("513"), (2, 1));
    assert_eq!(parse_dmx_address("1024"), (2, 512));
    assert_eq!(parse_dmx_address("1025"), (3, 1));
    assert_eq!(parse_dmx_address("0"), (1, 1));
    assert_eq!(parse_dmx_address(""), (1, 1));
    assert_eq!(parse_dmx_address("invalid"), (1, 1));
}

#[test]
fn test_parse_mvr_xml() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GeneralSceneDescription verMajor="1" verMinor="6" provider="Vectorworks" providerVersion="2024">
  <Scene>
    <Layers>
      <Layer name="Truss Front" uuid="layer-1">
        <ChildList>
          <Fixture name="Spot 1" uuid="fix-1">
            <GDTFSpec>Robe_Robin_Pointe.gdtf</GDTFSpec>
            <GDTFMode>Mode 1</GDTFMode>
            <FixtureID>101</FixtureID>
            <Addresses>
              <Address break="0">1.1</Address>
            </Addresses>
          </Fixture>
          <Fixture name="Spot 2" uuid="fix-2">
            <GDTFSpec>Robe_Robin_Pointe.gdtf</GDTFSpec>
            <GDTFMode>Mode 1</GDTFMode>
            <FixtureID>102</FixtureID>
            <Addresses>
              <Address break="0">1.25</Address>
            </Addresses>
          </Fixture>
        </ChildList>
      </Layer>
      <Layer name="Truss Back" uuid="layer-2">
        <ChildList>
          <Fixture name="Wash 1" uuid="fix-3">
            <GDTFSpec>Generic@RGBW.gdtf</GDTFSpec>
            <GDTFMode>Standard</GDTFMode>
            <FixtureID>201</FixtureID>
            <Addresses>
              <Address break="0">2.1</Address>
            </Addresses>
          </Fixture>
        </ChildList>
      </Layer>
    </Layers>
  </Scene>
</GeneralSceneDescription>"#;

    let scene = parse_mvr_xml(xml).expect("Should parse valid MVR XML");
    assert_eq!(scene.layers.len(), 2);
    assert_eq!(scene.layers[0].name, "Truss Front");
    assert_eq!(scene.layers[0].fixtures.len(), 2);
    assert_eq!(scene.layers[0].fixtures[0].name, "Spot 1");
    assert_eq!(
        scene.layers[0].fixtures[0].gdtf_spec.as_deref(),
        Some("Robe_Robin_Pointe.gdtf")
    );
    assert_eq!(
        scene.layers[0].fixtures[0].gdtf_mode.as_deref(),
        Some("Mode 1")
    );
    assert_eq!(
        scene.layers[0].fixtures[0].fixture_id.as_deref(),
        Some("101")
    );
    assert_eq!(scene.layers[0].fixtures[0].addresses.len(), 1);
    assert_eq!(scene.layers[0].fixtures[0].addresses[0].universe, 1);
    assert_eq!(scene.layers[0].fixtures[0].addresses[0].address, 1);

    assert_eq!(scene.layers[1].name, "Truss Back");
    assert_eq!(scene.layers[1].fixtures.len(), 1);
    assert_eq!(scene.layers[1].fixtures[0].addresses[0].universe, 2);
    assert_eq!(scene.layers[1].fixtures[0].addresses[0].address, 1);
}

fn create_test_gdtf_bytes(fixture_name: &str, manufacturer: &str, channels: &[&str]) -> Vec<u8> {
    let mut channels_xml = String::new();
    for (i, ch) in channels.iter().enumerate() {
        channels_xml.push_str(&format!(
            r#"<DMXChannel Offset="{}"><LogicalChannel Attribute="{}"/></DMXChannel>"#,
            i + 1,
            ch
        ));
    }

    let gdtf_xml = format!(
        r#"<GDTF DataVersion="1.2">
    <FixtureType Name="{fixture_name}" Manufacturer="{manufacturer}">
        <DMXModes>
            <DMXMode Name="Standard">
                <DMXChannels>
                    {channels_xml}
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#
    );

    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();
        zip.start_file("description.xml", options).unwrap();
        zip.write_all(gdtf_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn test_import_mvr_from_bytes_with_bundled_gdtf() {
    let gdtf_bytes = create_test_gdtf_bytes(
        "Robin Pointe",
        "Robe Lighting",
        &[
            "Pan",
            "Tilt",
            "ColorAdd_R",
            "ColorAdd_G",
            "ColorAdd_B",
            "Dimmer",
        ],
    );

    let scene_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GeneralSceneDescription verMajor="1" verMinor="6" provider="Vectorworks" providerVersion="2024">
  <Scene>
    <Layers>
      <Layer name="Main Stage" uuid="layer-main">
        <ChildList>
          <Fixture name="Pointe 1" uuid="f-1">
            <GDTFSpec>Robe_Robin_Pointe.gdtf</GDTFSpec>
            <GDTFMode>Standard</GDTFMode>
            <FixtureID>101</FixtureID>
            <Addresses>
              <Address break="0">1.1</Address>
            </Addresses>
          </Fixture>
          <Fixture name="Pointe 2" uuid="f-2">
            <GDTFSpec>Robe_Robin_Pointe.gdtf</GDTFSpec>
            <GDTFMode>Standard</GDTFMode>
            <FixtureID>102</FixtureID>
            <Addresses>
              <Address break="0">1.7</Address>
            </Addresses>
          </Fixture>
        </ChildList>
      </Layer>
      <Layer name="Floor Lights" uuid="layer-floor">
        <ChildList>
          <Fixture name="Pointe 3" uuid="f-3">
            <GDTFSpec>Robe_Robin_Pointe.gdtf</GDTFSpec>
            <GDTFMode>Standard</GDTFMode>
            <FixtureID>201</FixtureID>
            <Addresses>
              <Address break="0">2.1</Address>
            </Addresses>
          </Fixture>
        </ChildList>
      </Layer>
    </Layers>
  </Scene>
</GeneralSceneDescription>"#;

    let mut mvr_buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut mvr_buf);
        let mut zip = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();

        zip.start_file("GeneralSceneDescription.xml", options)
            .unwrap();
        zip.write_all(scene_xml.as_bytes()).unwrap();

        zip.start_file("Robe_Robin_Pointe.gdtf", options).unwrap();
        zip.write_all(&gdtf_bytes).unwrap();

        zip.finish().unwrap();
    }

    let show = import_mvr_from_bytes(&mvr_buf, "My Arena Rig").expect("Should import valid MVR");

    assert_eq!(show.status, ShowStatus::Loaded);
    assert_eq!(show.identity.display_name, "My Arena Rig");
    assert!(show.file.is_dirty);
    assert!(show.file.path.is_none());

    // Check groups: "All", "Main Stage", "Floor Lights"
    assert_eq!(show.groups.groups.len(), 3);
    assert_eq!(show.groups.groups[0].name, "All");
    assert_eq!(show.groups.groups[1].name, "Main Stage");
    assert_eq!(show.groups.groups[2].name, "Floor Lights");

    // Check fixtures
    assert_eq!(show.patch.fixtures.len(), 3);

    let f1 = &show.patch.fixtures[0];
    assert_eq!(f1.name, "Pointe 1");
    assert_eq!(f1.universe, 1);
    assert_eq!(f1.address, 1);
    assert_eq!(f1.groups, vec![1]);
    assert_eq!(f1.fixture_type.get("name").unwrap(), "Robin Pointe");
    let f1_channels = f1.mode.get("channels").unwrap().as_array().unwrap();
    assert_eq!(f1_channels.len(), 6);
    assert_eq!(f1_channels[0], "Pan");
    assert_eq!(f1_channels[5], "Dimmer");

    let f2 = &show.patch.fixtures[1];
    assert_eq!(f2.name, "Pointe 2");
    assert_eq!(f2.universe, 1);
    assert_eq!(f2.address, 7);
    assert_eq!(f2.groups, vec![1]);

    let f3 = &show.patch.fixtures[2];
    assert_eq!(f3.name, "Pointe 3");
    assert_eq!(f3.universe, 2);
    assert_eq!(f3.address, 1);
    assert_eq!(f3.groups, vec![2]);

    // Check outputs: universes 1 and 2
    assert_eq!(show.output.universes.len(), 2);
    assert_eq!(show.output.universes[0].universe, 1);
    assert_eq!(show.output.universes[1].universe, 2);

    // Check fixture definitions embedded
    assert!(!show.fixture_definitions.is_empty());
    let def = show
        .fixture_definitions
        .get("robe lighting@robin pointe")
        .expect("Definition should be stored");
    assert_eq!(def.name, "Robin Pointe");
    assert_eq!(def.manufacturer, "Robe Lighting");
    assert_eq!(def.modes.len(), 1);
    assert_eq!(def.modes[0].channel_count, 6);
}

#[test]
fn test_import_mvr_with_generic_fallbacks() {
    let scene_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GeneralSceneDescription verMajor="1" verMinor="6" provider="MA3" providerVersion="1.9">
  <Scene>
    <Layers>
      <Layer name="Generic Rig">
        <ChildList>
          <Fixture name="Dimmer Rack 1">
            <GDTFSpec>Dimmer.gdtf</GDTFSpec>
            <Addresses>
              <Address>1.1</Address>
            </Addresses>
          </Fixture>
          <Fixture name="Color Wash 1">
            <GDTFSpec>RGBW PAR.gdtf</GDTFSpec>
            <Addresses>
              <Address>1.5</Address>
            </Addresses>
          </Fixture>
        </ChildList>
      </Layer>
    </Layers>
  </Scene>
</GeneralSceneDescription>"#;

    let mut mvr_buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut mvr_buf);
        let mut zip = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();

        zip.start_file("GeneralSceneDescription.xml", options)
            .unwrap();
        zip.write_all(scene_xml.as_bytes()).unwrap();

        zip.finish().unwrap();
    }

    let show = import_mvr_from_bytes(&mvr_buf, "Generic Show").expect("Should import generic show");
    assert_eq!(show.patch.fixtures.len(), 2);

    // Fixture 1 matched generic Dimmer
    let f1 = &show.patch.fixtures[0];
    assert_eq!(f1.name, "Dimmer Rack 1");
    assert_eq!(f1.fixture_type.get("name").unwrap(), "Dimmer");

    // Fixture 2 matched generic RGBW PAR
    let f2 = &show.patch.fixtures[1];
    assert_eq!(f2.name, "Color Wash 1");
    assert_eq!(f2.fixture_type.get("name").unwrap(), "RGBW PAR");
}

#[test]
fn test_import_mvr_missing_xml() {
    let mut mvr_buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut mvr_buf);
        let mut zip = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();
        zip.start_file("random.txt", options).unwrap();
        zip.write_all(b"not an mvr").unwrap();
        zip.finish().unwrap();
    }

    let result = import_mvr_from_bytes(&mvr_buf, "Invalid");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("GeneralSceneDescription.xml not found"));
}

#[test]
fn test_import_mvr_avoids_false_positive_uv_matching() {
    let scene_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GeneralSceneDescription verMajor="1" verMinor="6">
  <Scene>
    <Layers>
      <Layer name="Chauvet Layer">
        <ChildList>
          <Fixture name="Chauvet Spot">
            <GDTFSpec>Chauvet_Spot.gdtf</GDTFSpec>
            <Addresses>
              <Address>1.1</Address>
            </Addresses>
          </Fixture>
        </ChildList>
      </Layer>
    </Layers>
  </Scene>
</GeneralSceneDescription>"#;

    let mut mvr_buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut mvr_buf);
        let mut zip = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();

        zip.start_file("GeneralSceneDescription.xml", options)
            .unwrap();
        zip.write_all(scene_xml.as_bytes()).unwrap();

        zip.finish().unwrap();
    }

    let show = import_mvr_from_bytes(&mvr_buf, "Chauvet Test").expect("Should import");
    let f = &show.patch.fixtures[0];
    // "Chauvet Spot" contains "spot", so it should match "Moving Head Spot", NOT "RGBAW+UV PAR"
    assert_eq!(f.fixture_type.get("name").unwrap(), "Moving Head Spot");
}

#[test]
fn test_sanitize_group_name() {
    assert_eq!(sanitize_group_name(">> BANK <<"), "BANK");
    assert_eq!(sanitize_group_name("[Front Truss]"), "Front Truss");
    assert_eq!(sanitize_group_name("- Backlight -"), "Backlight");
    assert_eq!(sanitize_group_name("  Clean Group  "), "Clean Group");
    assert_eq!(sanitize_group_name(">>>"), "Group");
    assert_eq!(sanitize_group_name(""), "Group");
}

#[test]
fn test_is_default_layer() {
    assert!(is_default_layer("Calque par défaut"));
    assert!(is_default_layer("calque par defaut"));
    assert!(is_default_layer("Default"));
    assert!(is_default_layer("Default Layer"));
    assert!(is_default_layer("None"));
    assert!(is_default_layer("unassigned"));
    assert!(!is_default_layer("Front Truss"));
    assert!(!is_default_layer("Backlight"));
}

#[test]
fn test_allocate_fixture_ids() {
    // Unique FIDs
    let f1 = MvrFixture {
        fixture_id: Some("101".to_string()),
        ..Default::default()
    };
    let f2 = MvrFixture {
        fixture_id: Some("102".to_string()),
        ..Default::default()
    };
    assert_eq!(allocate_fixture_ids(&[f1, f2]), vec![101, 102]);

    // Colliding FIDs fallback
    let f1 = MvrFixture {
        fixture_id: Some("101".to_string()),
        ..Default::default()
    };
    let f2 = MvrFixture {
        fixture_id: Some("101".to_string()),
        ..Default::default()
    };
    let f3 = MvrFixture {
        fixture_id: None,
        ..Default::default()
    };
    assert_eq!(allocate_fixture_ids(&[f1, f2, f3]), vec![1, 2, 3]);

    // Mixed valid and missing FIDs
    let f1 = MvrFixture {
        fixture_id: Some("200".to_string()),
        ..Default::default()
    };
    let f2 = MvrFixture {
        fixture_id: None,
        ..Default::default()
    };
    assert_eq!(allocate_fixture_ids(&[f1, f2]), vec![200, 1]);
}

#[test]
fn test_disambiguate_fixture_name() {
    // Unique name is untouched
    assert_eq!(
        disambiguate_fixture_name("Spot 1", Some("101"), 101, 1),
        "Spot 1"
    );

    // Duplicated name appends FID
    assert_eq!(
        disambiguate_fixture_name("Robin GigaPointe", Some("1201"), 1201, 2),
        "Robin GigaPointe 1201"
    );

    // Duplicated name that already ends with FID is untouched
    assert_eq!(
        disambiguate_fixture_name("Robin GigaPointe 1201", Some("1201"), 1201, 2),
        "Robin GigaPointe 1201"
    );

    // Empty name falls back to Fixture {ID}
    assert_eq!(
        disambiguate_fixture_name("", Some("105"), 105, 1),
        "Fixture 105"
    );
    assert_eq!(disambiguate_fixture_name("", None, 3, 1), "Fixture 3");
}

#[test]
fn test_import_mvr_clean_transformations() {
    let scene_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GeneralSceneDescription verMajor="1" verMinor="6" provider="Vectorworks">
  <Scene>
    <Layers>
      <Layer name=">> SPOT <<" uuid="layer-front">
        <ChildList>
          <Fixture name="Spot" uuid="f-1">
            <GDTFSpec>Dimmer.gdtf</GDTFSpec>
            <FixtureID>101</FixtureID>
            <Addresses><Address>2.10</Address></Addresses>
          </Fixture>
          <Fixture name="Spot" uuid="f-2">
            <GDTFSpec>Dimmer.gdtf</GDTFSpec>
            <FixtureID>102</FixtureID>
            <Addresses><Address>1.1</Address></Addresses>
          </Fixture>
        </ChildList>
      </Layer>
      <Layer name="Architectural Empty Layer" uuid="layer-empty">
        <ChildList />
      </Layer>
      <Layer name="Calque par défaut" uuid="layer-default">
        <ChildList>
          <Fixture name="Spot" uuid="f-3">
            <GDTFSpec>Dimmer.gdtf</GDTFSpec>
            <FixtureID>103</FixtureID>
            <Addresses><Address>1.50</Address></Addresses>
          </Fixture>
          <Fixture name="Strobe" uuid="f-4">
            <GDTFSpec>Strobe 1ch.gdtf</GDTFSpec>
            <FixtureID>201</FixtureID>
            <Addresses><Address>1.25</Address></Addresses>
          </Fixture>
        </ChildList>
      </Layer>
    </Layers>
  </Scene>
</GeneralSceneDescription>"#;

    let mut mvr_buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut mvr_buf);
        let mut zip = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();

        zip.start_file("GeneralSceneDescription.xml", options)
            .unwrap();
        zip.write_all(scene_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let show = import_mvr_from_bytes(&mvr_buf, "Clean Test Show").expect("Should import");

    // 1. Group sanitization: ">> SPOT <<" -> "SPOT"
    // 2. Empty layer "Architectural Empty Layer" is pruned (0 fixtures)
    // 3. "Calque par défaut" is NOT added as a group
    // 4. Default layer "Spot" fixture is merged into existing "SPOT" group
    // 5. Default layer "Strobe" fixture creates a new model group "Strobe"
    assert_eq!(show.groups.groups.len(), 3);
    assert_eq!(show.groups.groups[0].name, "All");
    assert_eq!(show.groups.groups[1].name, "SPOT");
    assert_eq!(show.groups.groups[2].name, "Strobe");

    // Check patch sorting by Universe and Address:
    // f2: U=1, A=1
    // f4: U=1, A=25
    // f3: U=1, A=50
    // f1: U=2, A=10
    assert_eq!(show.patch.fixtures.len(), 4);
    assert_eq!(show.patch.fixtures[0].universe, 1);
    assert_eq!(show.patch.fixtures[0].address, 1);
    assert_eq!(show.patch.fixtures[0].id, 102);
    assert_eq!(show.patch.fixtures[0].name, "Spot 102");
    assert_eq!(show.patch.fixtures[0].groups, vec![1]);

    assert_eq!(show.patch.fixtures[1].universe, 1);
    assert_eq!(show.patch.fixtures[1].address, 25);
    assert_eq!(show.patch.fixtures[1].id, 201);
    assert_eq!(show.patch.fixtures[1].name, "Strobe"); // only 1 strobe, stays single name
    assert_eq!(show.patch.fixtures[1].groups, vec![2]); // new Strobe group
                                                        // "Strobe 1ch.gdtf" must resolve to the built-in generic Strobe, not a fallback dimmer.
    assert_eq!(
        show.patch.fixtures[1].fixture_type.get("name").unwrap(),
        "Strobe"
    );
    assert_eq!(
        show.patch.fixtures[1].mode.get("channels").unwrap(),
        &serde_json::json!(["Strobe"])
    );

    assert_eq!(show.patch.fixtures[2].universe, 1);
    assert_eq!(show.patch.fixtures[2].address, 50);
    assert_eq!(show.patch.fixtures[2].id, 103);
    assert_eq!(show.patch.fixtures[2].name, "Spot 103");
    assert_eq!(show.patch.fixtures[2].groups, vec![1]); // merged into FRONT TRUSS

    assert_eq!(show.patch.fixtures[3].universe, 2);
    assert_eq!(show.patch.fixtures[3].address, 10);
    assert_eq!(show.patch.fixtures[3].id, 101);
    assert_eq!(show.patch.fixtures[3].name, "Spot 101");
    assert_eq!(show.patch.fixtures[3].groups, vec![1]);
}

#[test]
fn test_import_real_downloaded_mvr_if_present() {
    let path = std::env::var("LIVE_DMX_MVR_FIXTURE")
        .unwrap_or_else(|_| "/home/baptiste/Downloads/PCT26_MVR.mvr".to_string());
    if std::path::Path::new(&path).exists() {
        let show =
            live_dmx_lib::show::mvr::import_mvr_file(&path, None).expect("Should import real MVR");
        assert_eq!(show.identity.display_name, "PCT26_MVR");
        assert_eq!(show.patch.fixtures.len(), 173);
        assert_eq!(show.groups.groups.len(), 14);
        assert_eq!(show.output.universes.len(), 10);
        assert_eq!(show.fixture_definitions.len(), 13);
        // Verify no default layer groups remain
        assert!(show
            .groups
            .groups
            .iter()
            .all(|g| !is_default_layer(&g.name)));
    }
}
