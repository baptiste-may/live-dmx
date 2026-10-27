use live_dmx_lib::show::{
    apply_availability, insert_recent_show, limit_recent_shows, read_recent_shows_from_path,
    remove_recent_show_by_path, remove_unavailable_recent_shows, write_recent_shows_to_path,
    RecentShow, MAX_RECENT_SHOWS,
};

fn recent(display_name: &str, path: &str, updated_at: &str) -> RecentShow {
    RecentShow {
        display_name: display_name.to_string(),
        path: path.to_string(),
        updated_at: updated_at.to_string(),
        available: false,
    }
}

#[test]
fn test_recent_shows_io_and_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("recent_shows.json");

    // Non-existent file
    assert_eq!(read_recent_shows_from_path(&file_path).unwrap().len(), 0);

    let shows = vec![
        recent("Concert 2026", "/shows/concert.ldmx", "12345678"),
        recent("Festival 2026", "/shows/festival.ldmx", "87654321"),
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
fn test_recent_shows_deduplication_moves_to_top() {
    let mut shows = Vec::new();

    insert_recent_show(&mut shows, recent("Show 1", "/shows/1.ldmx", "100"));
    insert_recent_show(&mut shows, recent("Show 2", "/shows/2.ldmx", "200"));
    insert_recent_show(
        &mut shows,
        recent("Show 1 reopened", "/shows/1.ldmx", "300"),
    );

    assert_eq!(shows.len(), 2);
    assert_eq!(shows[0].path, "/shows/1.ldmx");
    assert_eq!(shows[0].display_name, "Show 1 reopened");
    assert_eq!(shows[0].updated_at, "300");
    assert_eq!(shows[1].path, "/shows/2.ldmx");
}

#[test]
fn test_recent_shows_cap_at_five() {
    let mut shows = Vec::new();

    for i in 0..6 {
        insert_recent_show(
            &mut shows,
            recent(
                &format!("Show {}", i),
                &format!("/shows/{}.ldmx", i),
                &i.to_string(),
            ),
        );
    }

    assert_eq!(shows.len(), MAX_RECENT_SHOWS);
    assert_eq!(shows[0].path, "/shows/5.ldmx");
    assert_eq!(shows[4].path, "/shows/1.ldmx");
    assert!(shows.iter().all(|show| show.path != "/shows/0.ldmx"));
}

#[test]
fn test_recent_shows_legacy_read_limits_without_rewriting() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("recent_shows.json");

    let mut shows = Vec::new();
    for i in 0..6 {
        shows.push(recent(
            &format!("Show {}", i),
            &format!("/shows/{}.ldmx", i),
            &i.to_string(),
        ));
    }
    write_recent_shows_to_path(&file_path, &shows).unwrap();

    let limited = limit_recent_shows(read_recent_shows_from_path(&file_path).unwrap());
    assert_eq!(limited.len(), MAX_RECENT_SHOWS);
    assert_eq!(limited[0].path, "/shows/0.ldmx");
    assert_eq!(limited[4].path, "/shows/4.ldmx");

    let stored: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(&file_path).unwrap()).unwrap();
    assert_eq!(stored.len(), 6, "reading must not rewrite the legacy file");
}

#[test]
fn test_recent_shows_availability() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join("exists.ldmx");
    std::fs::write(&existing, "{}").unwrap();
    let folder = dir.path().join("folder.ldmx");
    std::fs::create_dir(&folder).unwrap();

    let mut shows = vec![
        recent("Exists", existing.to_str().unwrap(), "1"),
        recent(
            "Missing",
            dir.path().join("missing.ldmx").to_str().unwrap(),
            "2",
        ),
        recent("Folder", folder.to_str().unwrap(), "3"),
    ];

    apply_availability(&mut shows);

    assert!(shows[0].available);
    assert!(!shows[1].available);
    assert!(!shows[2].available, "a directory is not a show file");
}

#[test]
fn test_recent_shows_clean_removes_unavailable_entries() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join("exists.ldmx");
    std::fs::write(&existing, "{}").unwrap();
    let folder = dir.path().join("folder.ldmx");
    std::fs::create_dir(&folder).unwrap();

    let mut shows = vec![
        recent("Exists", existing.to_str().unwrap(), "1"),
        recent(
            "Missing",
            dir.path().join("missing.ldmx").to_str().unwrap(),
            "2",
        ),
        recent("Folder", folder.to_str().unwrap(), "3"),
    ];

    remove_unavailable_recent_shows(&mut shows);

    assert_eq!(shows.len(), 1);
    assert_eq!(shows[0].display_name, "Exists");
    assert!(shows[0].available);

    let file_path = dir.path().join("recent_shows.json");
    write_recent_shows_to_path(&file_path, &shows).unwrap();
    let loaded = read_recent_shows_from_path(&file_path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].display_name, "Exists");
}

#[test]
fn test_recent_shows_removal() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("recent_shows.json");

    let mut shows = vec![
        recent("Show 1", "/shows/1.ldmx", "100"),
        recent("Show 2", "/shows/2.ldmx", "200"),
    ];

    remove_recent_show_by_path(&mut shows, "/shows/1.ldmx");
    assert_eq!(shows.len(), 1);
    assert_eq!(shows[0].display_name, "Show 2");

    remove_recent_show_by_path(&mut shows, "/shows/unknown.ldmx");
    assert_eq!(shows.len(), 1, "unknown paths are a no-op");

    write_recent_shows_to_path(&file_path, &shows).unwrap();
    let loaded = read_recent_shows_from_path(&file_path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].path, "/shows/2.ldmx");
}

#[test]
fn test_recent_shows_removal_enforces_cap() {
    let mut shows: Vec<RecentShow> = (0..7)
        .map(|i| {
            recent(
                &format!("Show {}", i),
                &format!("/shows/{}.ldmx", i),
                &i.to_string(),
            )
        })
        .collect();

    remove_recent_show_by_path(&mut shows, "/shows/2.ldmx");

    assert_eq!(shows.len(), MAX_RECENT_SHOWS);
    assert_eq!(shows[0].path, "/shows/0.ldmx");
    assert_eq!(shows[4].path, "/shows/5.ldmx");
}

#[test]
fn test_recent_shows_serialization_skips_availability() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("recent_shows.json");

    let mut shows = vec![recent("Show", "/shows/show.ldmx", "123")];
    shows[0].available = true;

    write_recent_shows_to_path(&file_path, &shows).unwrap();

    let raw = std::fs::read_to_string(&file_path).unwrap();
    assert!(!raw.contains("available"));

    let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let object = value.as_array().unwrap()[0].as_object().unwrap();
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["displayName", "path", "updatedAt"]);

    let loaded = read_recent_shows_from_path(&file_path).unwrap();
    assert!(
        !loaded[0].available,
        "availability is recomputed, not stored"
    );
}

#[test]
fn test_recent_shows_serialization_includes_availability_for_frontend() {
    let mut show = recent("Show", "/shows/show.ldmx", "123");
    show.available = true;

    let value = serde_json::to_value(&show).unwrap();

    assert_eq!(value["displayName"], "Show");
    assert_eq!(value["path"], "/shows/show.ldmx");
    assert_eq!(value["updatedAt"], "123");
    assert_eq!(value["available"], true);
}
