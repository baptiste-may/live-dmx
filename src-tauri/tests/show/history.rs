use live_dmx_lib::show::{
    PatchGroupsState, ShowHistory, ShowIdentityState, ShowOutputState, ShowPatchState,
    ShowSnapshot, MAX_UNDO_STACK_SIZE,
};

fn mock_snapshot(name: &str) -> ShowSnapshot {
    ShowSnapshot {
        identity: ShowIdentityState {
            display_name: name.to_string(),
        },
        patch: ShowPatchState::default(),
        groups: PatchGroupsState::default(),
        output: ShowOutputState::default(),
        fixture_definitions: std::collections::HashMap::new(),
    }
}

#[test]
fn test_history_default_state() {
    let history = ShowHistory::default();
    assert!(history.undo_stack.is_empty());
    assert!(history.redo_stack.is_empty());
    assert_eq!(history.saved_snapshot, None);
    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert!(history.is_dirty(&mock_snapshot("Any")));
}

#[test]
fn test_reset_with_saved_and_reset_empty() {
    let mut history = ShowHistory::default();
    let s0 = mock_snapshot("Saved Show");

    history.push_mutation(mock_snapshot("Old 1"));
    history.redo_stack.push(mock_snapshot("Old Redo"));

    history.reset_with_saved(s0.clone());
    assert!(history.undo_stack.is_empty());
    assert!(history.redo_stack.is_empty());
    assert_eq!(history.saved_snapshot, Some(s0.clone()));
    assert!(!history.is_dirty(&s0));
    assert!(history.is_dirty(&mock_snapshot("Modified")));

    history.reset_empty();
    assert!(history.undo_stack.is_empty());
    assert!(history.redo_stack.is_empty());
    assert_eq!(history.saved_snapshot, None);
    assert!(history.is_dirty(&s0));
}

#[test]
fn test_show_history_undo_redo_and_dirty_state() {
    let mut history = ShowHistory::default();
    let initial_snapshot = mock_snapshot("Show 1");

    history.reset_with_saved(initial_snapshot.clone());
    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert!(!history.is_dirty(&initial_snapshot));

    let edited_snapshot = mock_snapshot("Show 1 Edited");

    history.push_mutation(initial_snapshot.clone());
    assert!(history.can_undo());
    assert!(!history.can_redo());
    assert!(history.is_dirty(&edited_snapshot));

    // Perform undo
    let prev = history.undo_stack.pop().unwrap();
    history.redo_stack.push(edited_snapshot.clone());
    assert_eq!(prev, initial_snapshot);
    assert!(!history.can_undo());
    assert!(history.can_redo());
    assert!(!history.is_dirty(&prev)); // back to initial -> not dirty!

    // Perform redo
    let next = history.redo_stack.pop().unwrap();
    history.undo_stack.push(prev);
    assert_eq!(next, edited_snapshot);
    assert!(history.can_undo());
    assert!(!history.can_redo());
    assert!(history.is_dirty(&next));
}

#[test]
fn test_new_mutation_clears_redo_stack() {
    let mut history = ShowHistory::default();
    let s0 = mock_snapshot("S0");
    let s1 = mock_snapshot("S1");
    let s2 = mock_snapshot("S2");

    history.reset_with_saved(s0.clone());
    history.push_mutation(s0.clone());
    assert!(history.can_undo());

    // Undo to S0
    let prev = history.undo_stack.pop().unwrap();
    history.redo_stack.push(s1.clone());
    assert!(history.can_redo());

    // Push new branch mutation S2
    history.push_mutation(prev);
    assert!(history.can_undo());
    assert!(!history.can_redo()); // Redo stack was cleared!
    assert!(history.is_dirty(&s2));
}

#[test]
fn test_mark_saved_updates_baseline() {
    let mut history = ShowHistory::default();
    let s0 = mock_snapshot("S0");
    let s1 = mock_snapshot("S1");

    history.reset_with_saved(s0.clone());
    history.push_mutation(s0.clone());
    assert!(history.is_dirty(&s1));

    history.mark_saved(s1.clone());
    assert!(!history.is_dirty(&s1));
    assert!(history.is_dirty(&s0));
}

#[test]
fn test_undo_stack_capacity_limit() {
    let mut history = ShowHistory::default();
    let s = mock_snapshot("S");

    for _ in 0..150 {
        history.push_mutation(s.clone());
    }

    assert_eq!(history.undo_stack.len(), MAX_UNDO_STACK_SIZE);
}
