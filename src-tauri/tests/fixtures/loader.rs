use live_dmx_lib::fixtures::{
    filter_and_sort_fixtures, Fixture, Fixtures, Manufacturer, RawFixture,
};
use std::collections::BTreeMap;

fn create_mock_fixtures() -> Fixtures {
    let mut fixtures_chauvet = BTreeMap::new();
    fixtures_chauvet.insert(
        "slimpar-64".to_string(),
        Fixture {
            name: "SlimPAR 64".to_string(),
            short_name: None,
            categories: vec!["Color Changer".to_string()],
            meta: None,
            links: None,
            physical: None,
            matrix: None,
            wheels: None,
            available_channels: None,
            template_channels: None,
            modes: vec![],
            fixture_key: "slimpar-64".to_string(),
            manufacturer_key: "chauvet".to_string(),
            ofl_url: None,
            comment: None,
            help_wanted: None,
            rdm: None,
        },
    );
    fixtures_chauvet.insert(
        "gigbar-2".to_string(),
        Fixture {
            name: "GigBAR 2".to_string(),
            short_name: None,
            categories: vec!["Multi FX".to_string()],
            meta: None,
            links: None,
            physical: None,
            matrix: None,
            wheels: None,
            available_channels: None,
            template_channels: None,
            modes: vec![],
            fixture_key: "gigbar-2".to_string(),
            manufacturer_key: "chauvet".to_string(),
            ofl_url: None,
            comment: None,
            help_wanted: None,
            rdm: None,
        },
    );

    let mut fixtures_adj = BTreeMap::new();
    fixtures_adj.insert(
        "mega-par-profile".to_string(),
        Fixture {
            name: "Mega Par Profile".to_string(),
            short_name: None,
            categories: vec!["Color Changer".to_string()],
            meta: None,
            links: None,
            physical: None,
            matrix: None,
            wheels: None,
            available_channels: None,
            template_channels: None,
            modes: vec![],
            fixture_key: "mega-par-profile".to_string(),
            manufacturer_key: "adj".to_string(),
            ofl_url: None,
            comment: None,
            help_wanted: None,
            rdm: None,
        },
    );

    let mut manufacturers = BTreeMap::new();
    manufacturers.insert(
        "chauvet".to_string(),
        Manufacturer {
            name: "Chauvet DJ".to_string(),
            website: None,
            comment: None,
            rdm_id: None,
            fixtures: fixtures_chauvet,
        },
    );
    manufacturers.insert(
        "adj".to_string(),
        Manufacturer {
            name: "American DJ".to_string(),
            website: None,
            comment: None,
            rdm_id: None,
            fixtures: fixtures_adj,
        },
    );

    manufacturers
}

#[test]
fn test_filter_and_sort_fixtures_empty_collection() {
    let empty: Fixtures = BTreeMap::new();
    let result = filter_and_sort_fixtures(&empty, Some("test"));
    assert!(result.is_empty());

    let result_none = filter_and_sort_fixtures(&empty, None);
    assert!(result_none.is_empty());
}

#[test]
fn test_filter_and_sort_fixtures_none_and_empty_query() {
    let data = create_mock_fixtures();

    let all_none = filter_and_sort_fixtures(&data, None);
    assert_eq!(all_none.len(), 2);
    let keys: Vec<_> = all_none.keys().cloned().collect();
    assert_eq!(keys, vec!["adj", "chauvet"]);

    let all_empty = filter_and_sort_fixtures(&data, Some(""));
    assert_eq!(all_empty.len(), 2);

    let all_spaces = filter_and_sort_fixtures(&data, Some("   "));
    assert_eq!(all_spaces.len(), 2);
}

#[test]
fn test_filter_and_sort_fixtures_by_fixture_name_and_key() {
    let data = create_mock_fixtures();

    // Query by fixture name
    let filtered = filter_and_sort_fixtures(&data, Some("GigBAR"));
    assert_eq!(filtered.len(), 1);
    assert!(filtered.contains_key("chauvet"));
    assert_eq!(filtered["chauvet"].fixtures.len(), 1);
    assert!(filtered["chauvet"].fixtures.contains_key("gigbar-2"));

    // Query by fixture key
    let filtered_key = filter_and_sort_fixtures(&data, Some("mega-par"));
    assert_eq!(filtered_key.len(), 1);
    assert!(filtered_key.contains_key("adj"));
    assert_eq!(filtered_key["adj"].fixtures.len(), 1);
}

#[test]
fn test_filter_and_sort_fixtures_by_manufacturer() {
    let data = create_mock_fixtures();

    // By manufacturer name
    let filtered_name = filter_and_sort_fixtures(&data, Some("American"));
    assert_eq!(filtered_name.len(), 1);
    assert!(filtered_name.contains_key("adj"));

    // By manufacturer key
    let filtered_key = filter_and_sort_fixtures(&data, Some("chauvet"));
    assert_eq!(filtered_key.len(), 1);
    assert!(filtered_key.contains_key("chauvet"));
    assert_eq!(filtered_key["chauvet"].fixtures.len(), 2);
}

#[test]
fn test_filter_and_sort_fixtures_no_match() {
    let data = create_mock_fixtures();

    let filtered = filter_and_sort_fixtures(&data, Some("nonexistent fixture 9999"));
    assert!(filtered.is_empty());
}

#[test]
fn test_filter_and_sort_fixtures_multi_token_and_category() {
    let data = create_mock_fixtures();

    // Multi-token: manufacturer + fixture name keyword
    let multi = filter_and_sort_fixtures(&data, Some("chauvet slimpar"));
    assert_eq!(multi.len(), 1);
    assert!(multi.contains_key("chauvet"));
    assert_eq!(multi["chauvet"].fixtures.len(), 1);
    assert!(multi["chauvet"].fixtures.contains_key("slimpar-64"));

    // Multi-token: category + manufacturer keyword
    let cat_man = filter_and_sort_fixtures(&data, Some("American Color"));
    assert_eq!(cat_man.len(), 1);
    assert!(cat_man.contains_key("adj"));
    assert_eq!(cat_man["adj"].fixtures.len(), 1);
    assert!(cat_man["adj"].fixtures.contains_key("mega-par-profile"));
}

#[test]
fn test_deserialize_all_fixtures() {
    let fixtures_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures");
    if !fixtures_dir.exists() {
        return;
    }

    let mut parsed_count = 0;
    let mut errors = Vec::new();

    fn walk_dir(dir: &std::path::Path, parsed_count: &mut usize, errors: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, parsed_count, errors);
                } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if file_name == "manufacturers.json" || file_name == "index.json" {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        match serde_json::from_str::<RawFixture>(&content) {
                            Ok(_) => *parsed_count += 1,
                            Err(e) => errors.push(format!("{}: {}", path.display(), e)),
                        }
                    }
                }
            }
        }
    }

    walk_dir(&fixtures_dir, &mut parsed_count, &mut errors);
    if !errors.is_empty() {
        panic!(
            "Failed to parse {} fixtures:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
    assert!(parsed_count > 0, "Should have parsed at least 1 fixture");
}
