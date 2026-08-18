use live_dmx_lib::show::ShowIdentityState;

#[test]
fn test_show_identity_state_modifications() {
    let mut identity = ShowIdentityState::default();
    assert_eq!(identity.display_name, "My show");

    identity.display_name = "Live Tour 2026 🎵".to_string();
    assert_eq!(identity.display_name, "Live Tour 2026 🎵");

    let json = serde_json::to_string(&identity).unwrap();
    let deserialized: ShowIdentityState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.display_name, "Live Tour 2026 🎵");
}

#[test]
fn test_show_identity_empty_name() {
    let identity = ShowIdentityState {
        display_name: String::new(),
    };
    let json = serde_json::to_string(&identity).unwrap();
    let deserialized: ShowIdentityState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.display_name, "");
}
