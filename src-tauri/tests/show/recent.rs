use live_dmx_lib::show::{read_recent_shows_from_path, write_recent_shows_to_path, RecentShow};

#[test]
fn test_recent_shows_io_and_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("recent_shows.json");

    // Non-existent file
    assert_eq!(read_recent_shows_from_path(&file_path).unwrap().len(), 0);

    let shows = vec![
        RecentShow {
            display_name: "Concert 2026".to_string(),
            path: "/shows/concert.ldmx".to_string(),
            updated_at: "12345678".to_string(),
        },
        RecentShow {
            display_name: "Festival 2026".to_string(),
            path: "/shows/festival.ldmx".to_string(),
            updated_at: "87654321".to_string(),
        },
    ];

    write_recent_shows_to_path(&file_path, &shows).unwrap();
    let loaded = read_recent_shows_from_path(&file_path).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].display_name, "Concert 2026");
    assert_eq!(loaded[0].path, "/shows/concert.ldmx");
    assert_eq!(loaded[1].display_name, "Festival 2026");
    assert_eq!(loaded[1].path, "/shows/festival.ldmx");
}

#[test]
fn test_recent_shows_corrupt_file_handling() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("corrupt_recent_shows.json");

    std::fs::write(&file_path, "not a valid json list").unwrap();
    let loaded = read_recent_shows_from_path(&file_path);
    assert!(loaded.is_err());
}

#[test]
fn test_recent_shows_list_deduplication_and_truncation() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("recent_shows.json");

    let mut shows = Vec::new();
    for i in 0..25 {
        shows.push(RecentShow {
            display_name: format!("Show {}", i),
            path: format!("/shows/show_{}.ldmx", i),
            updated_at: format!("{}", 1000 + i),
        });
    }

    // Insert existing show at index 0 and truncate to 20
    let duplicate_path = "/shows/show_10.ldmx";
    shows.retain(|s| s.path != duplicate_path);
    shows.insert(
        0,
        RecentShow {
            display_name: "Show 10 Reopened".to_string(),
            path: duplicate_path.to_string(),
            updated_at: "99999".to_string(),
        },
    );
    if shows.len() > 20 {
        shows.truncate(20);
    }

    assert_eq!(shows.len(), 20);
    assert_eq!(shows[0].path, duplicate_path);
    assert_eq!(shows[0].display_name, "Show 10 Reopened");

    write_recent_shows_to_path(&file_path, &shows).unwrap();
    let loaded = read_recent_shows_from_path(&file_path).unwrap();
    assert_eq!(loaded.len(), 20);
    assert_eq!(loaded[0].path, duplicate_path);
}

#[test]
fn test_recent_shows_removal() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("recent_shows.json");

    let mut shows = vec![
        RecentShow {
            display_name: "Show 1".to_string(),
            path: "/shows/1.ldmx".to_string(),
            updated_at: "100".to_string(),
        },
        RecentShow {
            display_name: "Show 2".to_string(),
            path: "/shows/2.ldmx".to_string(),
            updated_at: "200".to_string(),
        },
    ];

    shows.retain(|s| s.path != "/shows/1.ldmx");
    assert_eq!(shows.len(), 1);
    assert_eq!(shows[0].display_name, "Show 2");

    write_recent_shows_to_path(&file_path, &shows).unwrap();
    let loaded = read_recent_shows_from_path(&file_path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].path, "/shows/2.ldmx");
}
