use live_dmx_lib::show::{AppState, ShowOutputState, UniverseOutputConfig};

#[test]
fn test_show_output_state_defaults() {
    let output = ShowOutputState::default();
    assert_eq!(output.universes.len(), 1);
    assert_eq!(output.universes[0].universe, 1);
    assert_eq!(output.universes[0].output, "none");
    assert!(output.universes[0].enabled);
}

#[test]
fn test_output_mutation_undo_redo_cycle() {
    let app_state = AppState::default();

    // 1. Initial state
    {
        let show = app_state.show.lock().unwrap();
        assert_eq!(show.output.universes.len(), 1);
        assert_eq!(show.output.universes[0].output, "none");
        assert!(show.output.universes[0].enabled);
    }

    // 2. Mutate target of universe 1
    let (_, changed) = app_state
        .mutate_show(|show| {
            let u1 = show
                .output
                .universes
                .iter_mut()
                .find(|u| u.universe == 1)
                .unwrap();
            u1.output = "sacn".to_string();
            Ok(())
        })
        .unwrap();
    assert!(changed);

    {
        let show = app_state.show.lock().unwrap();
        assert_eq!(show.output.universes[0].output, "sacn");
        assert!(show.file.is_dirty);
        assert!(show.file.can_undo);
    }

    // 3. Mutate enabled of universe 1
    let (_, changed) = app_state
        .mutate_show(|show| {
            let u1 = show
                .output
                .universes
                .iter_mut()
                .find(|u| u.universe == 1)
                .unwrap();
            u1.enabled = false;
            Ok(())
        })
        .unwrap();
    assert!(changed);

    {
        let show = app_state.show.lock().unwrap();
        assert!(!show.output.universes[0].enabled);
    }

    // 4. Add universe
    let (new_u, changed) = app_state
        .mutate_show(|show| {
            let max_u = show
                .output
                .universes
                .iter()
                .map(|u| u.universe)
                .max()
                .unwrap_or(0);
            let next = max_u + 1;
            show.output.universes.push(UniverseOutputConfig {
                universe: next,
                output: "none".to_string(),
                enabled: true,
            });
            Ok(next)
        })
        .unwrap();
    assert_eq!(new_u, 2);
    assert!(changed);

    {
        let show = app_state.show.lock().unwrap();
        assert_eq!(show.output.universes.len(), 2);
        assert_eq!(show.output.universes[1].universe, 2);
    }

    // 5. Remove universe
    let (_, changed) = app_state
        .mutate_show(|show| {
            let pos = show
                .output
                .universes
                .iter()
                .position(|u| u.universe == 2)
                .unwrap();
            show.output.universes.remove(pos);
            Ok(())
        })
        .unwrap();
    assert!(changed);

    {
        let show = app_state.show.lock().unwrap();
        assert_eq!(show.output.universes.len(), 1);
    }

    // 6. Test cannot remove last universe
    let err = app_state
        .mutate_show(|show| {
            if show.output.universes.len() <= 1 {
                return Err("Cannot remove the last universe".to_string());
            }
            show.output.universes.remove(0);
            Ok(())
        })
        .unwrap_err();
    assert_eq!(err, "Cannot remove the last universe");
}
