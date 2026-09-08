use live_dmx_lib::fixtures::{GdtfCatalogItem, GdtfCatalogManager, GdtfCatalogMode};
use tempfile::tempdir;

#[test]
fn test_catalog_manager_initial_search_contains_generics() {
    let tmp = tempdir().unwrap();
    let manager = GdtfCatalogManager::new(tmp.path());

    let results = manager.search(None);
    assert!(!results.is_empty());

    let dimmer = results.iter().find(|i| i.fixture == "Dimmer");
    assert!(dimmer.is_some());
    let dimmer_item = dimmer.unwrap();
    assert_eq!(dimmer_item.manufacturer, "Generic");
    assert!(dimmer_item.is_installed);
    assert!(dimmer_item.is_generic);
}

#[test]
fn test_catalog_manager_search_filtering() {
    let tmp = tempdir().unwrap();
    let manager = GdtfCatalogManager::new(tmp.path());

    let rgb_results = manager.search(Some("RGB PAR"));
    assert!(!rgb_results.is_empty());
    assert!(rgb_results.iter().any(|i| i.fixture == "RGB PAR"));

    let non_existent = manager.search(Some("NonExistentFixture12345XYZ"));
    assert!(non_existent.is_empty());
}

#[test]
fn test_catalog_manager_save_and_retrieve_online_catalog() {
    let tmp = tempdir().unwrap();
    let manager = GdtfCatalogManager::new(tmp.path());

    let test_item = GdtfCatalogItem {
        rid: Some(9999),
        fixture: "Mega Wash".to_string(),
        manufacturer: "SuperLights".to_string(),
        revision: Some("2.0".to_string()),
        uploader: Some("Manuf.".to_string()),
        rating: Some(4.8),
        modes: vec![GdtfCatalogMode {
            name: "Extended 28ch".to_string(),
            dmxfootprint: 28,
            channels: None,
        }],
        uuid: Some("uuid-mega-wash".to_string()),
        filesize: Some(1024),
        is_installed: false,
        is_generic: false,
    };

    manager
        .save_online_catalog(&[test_item.clone()])
        .expect("Should save catalog cache");

    let results = manager.search(Some("Mega Wash"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].fixture, "Mega Wash");
    assert_eq!(results[0].manufacturer, "SuperLights");
    assert_eq!(results[0].rid, Some(9999));
}

#[test]
fn test_gdtf_share_client_default_and_logout() {
    use live_dmx_lib::fixtures::GdtfShareClient;

    let client = GdtfShareClient::new();
    assert!(!client.is_logged_in());

    client.logout();
    assert!(!client.is_logged_in());
}

#[test]
fn test_catalog_manager_smart_deduplication_prefers_manuf_and_highest_rating() {
    let tmp = tempdir().unwrap();
    let manager = GdtfCatalogManager::new(tmp.path());

    let rev1 = GdtfCatalogItem {
        rid: Some(101),
        fixture: "Sharpy Plus".to_string(),
        manufacturer: "Clay Paky".to_string(),
        revision: Some("v1.0".to_string()),
        uploader: Some("User".to_string()),
        rating: Some(3.0),
        modes: vec![GdtfCatalogMode {
            name: "Standard".to_string(),
            dmxfootprint: 31,
            channels: None,
        }],
        uuid: None,
        filesize: None,
        is_installed: false,
        is_generic: false,
    };

    let rev2 = GdtfCatalogItem {
        rid: Some(102),
        fixture: "Sharpy Plus".to_string(),
        manufacturer: "Clay Paky".to_string(),
        revision: Some("Official Fw 2.5".to_string()),
        uploader: Some("Manuf.".to_string()),
        rating: Some(4.9),
        modes: vec![GdtfCatalogMode {
            name: "Standard".to_string(),
            dmxfootprint: 31,
            channels: None,
        }],
        uuid: None,
        filesize: None,
        is_installed: false,
        is_generic: false,
    };

    manager
        .save_online_catalog(&[rev1, rev2])
        .expect("Should save catalog");

    let results = manager.search(Some("Sharpy Plus"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].fixture, "Sharpy Plus");
    assert_eq!(results[0].rid, Some(102));
    assert_eq!(results[0].uploader.as_deref(), Some("Manuf."));
    assert_eq!(results[0].revision.as_deref(), Some("Official Fw 2.5"));
}

#[test]
fn test_catalog_manager_deduplicates_generic_against_online() {
    let tmp = tempdir().unwrap();
    let manager = GdtfCatalogManager::new(tmp.path());

    let online_dimmer = GdtfCatalogItem {
        rid: Some(5001),
        fixture: "Dimmer".to_string(),
        manufacturer: "Generic".to_string(),
        revision: Some("2.0".to_string()),
        uploader: Some("User".to_string()),
        rating: Some(4.0),
        modes: vec![GdtfCatalogMode {
            name: "1 Channel".to_string(),
            dmxfootprint: 1,
            channels: Some(vec!["Dimmer".to_string()]),
        }],
        uuid: None,
        filesize: None,
        is_installed: false,
        is_generic: false,
    };

    manager
        .save_online_catalog(&[online_dimmer])
        .expect("Should save catalog");

    let results = manager.search(Some("Dimmer"));
    let dimmer_matches: Vec<_> = results.iter().filter(|i| i.fixture == "Dimmer").collect();
    assert_eq!(dimmer_matches.len(), 1);
    assert_eq!(dimmer_matches[0].manufacturer, "Generic");
    assert!(dimmer_matches[0].is_generic);
    assert!(dimmer_matches[0].is_installed);
}

#[test]
fn test_gdtf_auth_storage_roundtrip_and_delete() {
    use live_dmx_lib::fixtures::{
        delete_stored_credentials, load_stored_credentials, save_stored_credentials,
    };

    let tmp = tempdir().unwrap();
    let app_data = tmp.path();

    // 1. Initial state: no credentials
    let initial = load_stored_credentials(app_data).expect("Should load without error");
    assert_eq!(initial, None);

    // 2. Save credentials
    save_stored_credentials(app_data, "lighting_tech", "super_secret")
        .expect("Should save credentials");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file_path = live_dmx_lib::fixtures::get_auth_file_path(app_data);
        let metadata = std::fs::metadata(&file_path).expect("File metadata should exist");
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }

    // 3. Load credentials back
    let loaded = load_stored_credentials(app_data)
        .expect("Should load credentials")
        .expect("Credentials should exist");
    assert_eq!(loaded.0, "lighting_tech");
    assert_eq!(loaded.1, "super_secret");

    // 4. Delete credentials
    delete_stored_credentials(app_data).expect("Should delete credentials");
    let after_delete = load_stored_credentials(app_data).expect("Should load after delete");
    assert_eq!(after_delete, None);
}

#[test]
fn test_gdtf_auth_corrupted_file_handling() {
    use live_dmx_lib::fixtures::{get_auth_file_path, load_stored_credentials};

    let tmp = tempdir().unwrap();
    let app_data = tmp.path();
    let file_path = get_auth_file_path(app_data);

    // Write corrupt JSON
    std::fs::write(&file_path, "{ broken json ...").unwrap();
    assert!(file_path.exists());

    // Loading should gracefully handle corruption by removing file and returning None
    let loaded = load_stored_credentials(app_data).expect("Should not fail on corrupt file");
    assert_eq!(loaded, None);
    assert!(!file_path.exists());
}

#[test]
fn test_catalog_manager_preserves_channels_when_merging_online_item() {
    let tmp = tempdir().unwrap();
    let manager = live_dmx_lib::fixtures::GdtfCatalogManager::new(tmp.path());

    // 1. Install a local GDTF fixture
    let xml = r#"<GDTF DataVersion="1.2">
    <FixtureType Name="Sharpy" Manufacturer="Clay Paky">
        <DMXModes>
            <DMXMode Name="Standard">
                <DMXChannels>
                    <DMXChannel Offset="1"><LogicalChannel Attribute="Color1"/></DMXChannel>
                    <DMXChannel Offset="2"><LogicalChannel Attribute="Strobe"/></DMXChannel>
                    <DMXChannel Offset="3"><LogicalChannel Attribute="Dimmer"/></DMXChannel>
                </DMXChannels>
            </DMXMode>
        </DMXModes>
    </FixtureType>
</GDTF>"#;

    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip_writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default();
        zip_writer.start_file("description.xml", options).unwrap();
        std::io::Write::write_all(&mut zip_writer, xml.as_bytes()).unwrap();
        zip_writer.finish().unwrap();
    }

    manager
        .save_downloaded_gdtf("Clay Paky", "Sharpy", &buf)
        .expect("Should save GDTF fixture");

    // 2. Online catalog entry with Manufacturer uploader, but channels: None
    let online_item = GdtfCatalogItem {
        rid: Some(75541),
        fixture: "Sharpy".to_string(),
        manufacturer: "Clay Paky".to_string(),
        revision: Some("Official Fw 2.0".to_string()),
        uploader: Some("Manuf.".to_string()),
        rating: Some(5.0),
        modes: vec![GdtfCatalogMode {
            name: "Standard".to_string(),
            dmxfootprint: 3,
            channels: None,
        }],
        uuid: None,
        filesize: Some(1024),
        is_installed: false,
        is_generic: false,
    };

    manager
        .save_online_catalog(&[online_item])
        .expect("Should save online catalog");

    // 3. Search for the fixture
    let results = manager.search(Some("sharpy"));
    assert_eq!(results.len(), 1);

    let match_item = &results[0];
    assert_eq!(match_item.fixture, "Sharpy");
    assert_eq!(match_item.manufacturer, "Clay Paky");
    assert!(match_item.is_installed);
    assert_eq!(match_item.uploader.as_deref(), Some("Manuf."));
    assert_eq!(match_item.rid, Some(75541));

    // CRITICAL: Ensure parsed channels were preserved!
    assert_eq!(match_item.modes.len(), 1);
    let mode = &match_item.modes[0];
    assert_eq!(mode.name, "Standard");
    assert_eq!(
        mode.channels,
        Some(vec![
            "Color1".to_string(),
            "Strobe".to_string(),
            "Dimmer".to_string(),
        ])
    );
}

#[test]
fn test_catalog_manager_preview_cache_and_persist() {
    let tmp = tempdir().unwrap();
    let manager = GdtfCatalogManager::new(tmp.path());

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GDTF DataVersion="1.2">
    <FixtureType Name="PreviewSpot" Manufacturer="PreviewManuf">
        <DMXModes>
            <DMXMode Name="Extended">
                <DMXChannels>
                    <DMXChannel Offset="1">
                        <LogicalChannel Attribute="Pan"/>
                    </DMXChannel>
                    <DMXChannel Offset="2">
                        <LogicalChannel Attribute="Tilt"/>
                    </DMXChannel>
                    <DMXChannel Offset="3">
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
        let options = zip::write::SimpleFileOptions::default();
        zip_writer.start_file("description.xml", options).unwrap();
        std::io::Write::write_all(&mut zip_writer, xml.as_bytes()).unwrap();
        zip_writer.finish().unwrap();
    }

    // 1. Save to preview cache
    let preview_parsed = manager
        .save_preview_gdtf("PreviewManuf", "PreviewSpot", &buf)
        .expect("Should save to preview cache");
    assert_eq!(preview_parsed.name, "PreviewSpot");
    assert_eq!(preview_parsed.modes.len(), 1);

    // 2. Must be accessible via get_preview_fixture
    let in_preview = manager.get_preview_fixture("PreviewManuf", "PreviewSpot");
    assert!(in_preview.is_some());
    assert_eq!(in_preview.unwrap().modes[0].name, "Extended");

    // 3. Must NOT be saved to local fixtures on disk yet
    let local = manager.get_local_fixture("PreviewManuf", "PreviewSpot");
    assert!(local.is_none());

    let fixture_file = manager
        .fixtures_dir()
        .join("PreviewManuf")
        .join("PreviewSpot.gdtf");
    assert!(!fixture_file.exists(), "File should not exist on disk yet");

    // 4. Persist to persistent storage
    let persisted = manager
        .persist_preview_fixture("PreviewManuf", "PreviewSpot")
        .expect("Should persist preview fixture");
    assert!(persisted.is_some());

    // 5. Must now be available as local fixture and written to disk
    let now_local = manager.get_local_fixture("PreviewManuf", "PreviewSpot");
    assert!(now_local.is_some());
    assert!(
        fixture_file.exists(),
        "File must exist on disk after persist"
    );

    // 6. Preview cache should be cleaned up
    let post_preview = manager.get_preview_fixture("PreviewManuf", "PreviewSpot");
    assert!(post_preview.is_none());
}
