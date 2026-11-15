//! Tauri commands for live control and DMX readback.
//!
//! Handlers lock `AppState::output.engine` (a `LiveEngine`) and forward to it. This state is
//! live-only: it is not part of `ShowData`, so these commands never go through
//! `AppState::apply_mutation` (no undo/redo entry, nothing persisted in the `.ldmx` file) and
//! emit no `show-state-changed` event. The runner picks the changes up on its next 44 Hz
//! tick. Group id `0` is the global group that applies to every fixture. Universe numbers
//! are 1-based, as are DMX channels (1-512).
//!
//! See `docs/contributing/dmx-output.md`.

use tauri::State;

use crate::show::AppState;

/// Presses or releases a flash button of a group.
///
/// `flash_type` is one of `simple`, `white`, `master`, `strobe`, `randomStrobe` or `random_strobe`;
/// other values are stored but have no fade. Releasing `simple`, `white` or `master` starts a
/// fade-out when the matching flash fade time is above zero. Group `0` flashes every group.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_flash(
    state: State<'_, AppState>,
    group_id: u32,
    flash_type: String,
    active: bool,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_flash(group_id, flash_type, active);
    Ok(())
}

/// Sets the pressed state of a generic live button, identified by the frontend `button_id`.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_button(
    state: State<'_, AppState>,
    button_id: String,
    active: bool,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_button(button_id, active);
    Ok(())
}

/// Sets the intensity effect of a group.
///
/// `alignment == Some("rng")` reshuffles the random step order of the group. The effect width is
/// left unchanged.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_group_effect(
    state: State<'_, AppState>,
    group_id: u32,
    effect: Option<String>,
    alignment: Option<String>,
    bounce: bool,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_group_effect(group_id, effect, alignment, bounce);
    Ok(())
}

/// Sets the master level of a group.
///
/// `level` is a percentage, 100 meaning no limit (the default for a group without a master). Group
/// `0` is the global master.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_group_master(
    state: State<'_, AppState>,
    group_id: u32,
    level: u8,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_group_master(group_id, level);
    Ok(())
}

/// Sets the colour of a group and moves its fixtures to it, fading when the fade speed is above 0.
///
/// `color` is a colour name or hex value; `None` clears the group colour and targets white. The
/// patch is read from a clone of the show so no show lock is held while the engine is locked.
///
/// # Errors
///
/// Returns an error message if the show or engine lock is poisoned.
#[tauri::command]
pub fn live_set_group_color(
    state: State<'_, AppState>,
    group_id: u32,
    color: Option<String>,
) -> Result<(), String> {
    let show = state.show.lock().map_err(|e| e.to_string())?.clone();
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_group_color(group_id, color, Some(&show));
    Ok(())
}

/// Sets or clears (`None`) the secondary colour of a group.
///
/// Fixtures of a group without a secondary colour fall back to the one of group `0`.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_group_secondary_color(
    state: State<'_, AppState>,
    group_id: u32,
    color: Option<String>,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_group_secondary_color(group_id, color);
    Ok(())
}

/// Sets the same colour on every group and fixture.
///
/// `None` resets everything to white. Fades like [`live_set_group_color`] for group `0`.
///
/// # Errors
///
/// Returns an error message if the show or engine lock is poisoned.
#[tauri::command]
pub fn live_set_all_groups_color(
    state: State<'_, AppState>,
    color: Option<String>,
) -> Result<(), String> {
    let show = state.show.lock().map_err(|e| e.to_string())?.clone();
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_all_groups_color(color, Some(&show));
    Ok(())
}

/// Sets the colour effect of a group.
///
/// `alignment == Some("rng")` reshuffles the random colour step order of the group. The effect
/// width is left unchanged.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_group_color_effect(
    state: State<'_, AppState>,
    group_id: u32,
    effect: Option<String>,
    alignment: Option<String>,
    bounce: bool,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_group_color_effect(group_id, effect, alignment, bounce);
    Ok(())
}

/// Clears the colour effect of a group back to its default (no effect, width 100 %).
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_reset_group_color_effect(
    state: State<'_, AppState>,
    group_id: u32,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.reset_group_color_effect(group_id);
    Ok(())
}

/// Sets the duration of colour fades, in seconds.
///
/// Negative values are clamped to 0, which makes colour changes instant.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_color_fade_speed(state: State<'_, AppState>, speed: f32) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_color_fade_speed(speed);
    Ok(())
}

/// Sets the order in which the fixtures of a group start a colour fade.
///
/// `left`, `right`, `wing` or `center` stagger the fades across the fixtures; `None` fades them all
/// together over the full fade time. Only affects fades started afterwards.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_color_fade_order(
    state: State<'_, AppState>,
    order: Option<String>,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_color_fade_order(order);
    Ok(())
}

/// Sets the strobe rate, on a 0-100 scale.
///
/// The engine maps it to about 1-20 Hz; values above 100 are not clamped and extrapolate.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_strobe_rate(state: State<'_, AppState>, rate: u8) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.strobe_rate = rate;
    Ok(())
}

/// Sets the random strobe rate, on a 0-100 scale.
///
/// Higher values shorten the random slots and make flashes more likely.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_random_strobe_rate(state: State<'_, AppState>, rate: u8) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.random_strobe_rate = rate;
    Ok(())
}

/// Sets the release fade of the simple flash, in seconds.
///
/// Negative values are clamped to 0 (no fade).
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_simple_flash_fade(state: State<'_, AppState>, fade: f32) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_simple_flash_fade(fade);
    Ok(())
}

/// Sets the release fade of the white and master flashes, in seconds.
///
/// Negative values are clamped to 0 (no fade).
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_white_flash_fade(state: State<'_, AppState>, fade: f32) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_white_flash_fade(fade);
    Ok(())
}

/// Sets the tempo of intensity effects, in beats per minute.
///
/// A tempo of 0 makes tempo-based effects output nothing.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_effect_bpm(state: State<'_, AppState>, bpm: u16) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_effect_bpm(bpm);
    Ok(())
}

/// Sets the tempo of colour effects, in beats per minute.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_color_effect_bpm(state: State<'_, AppState>, bpm: u16) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_color_effect_bpm(bpm);
    Ok(())
}

/// Sets the width of the intensity effect of a group.
///
/// `width` is a percentage of the effect cycle that is active, clamped to 100.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_group_effect_width(
    state: State<'_, AppState>,
    group_id: u32,
    width: u8,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_group_effect_width(group_id, width);
    Ok(())
}

/// Sets the width of the colour effect of a group.
///
/// `width` is a percentage of the effect cycle that is active, clamped to 100.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_set_group_color_effect_width(
    state: State<'_, AppState>,
    group_id: u32,
    width: u8,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_group_color_effect_width(group_id, width);
    Ok(())
}

/// Clears the intensity effect of a group back to its default (no effect, width 100 %).
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_reset_group_effect(state: State<'_, AppState>, group_id: u32) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.reset_group_effect(group_id);
    Ok(())
}

/// Resets all live state to its defaults.
///
/// Clears flashes, buttons, effects, masters, colours (global colour back to white), fades, manual
/// DMX overrides and zeroes every universe buffer. Rates, fades and tempos return to their startup
/// values.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn live_reset_all_buttons(state: State<'_, AppState>) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.reset_all();
    Ok(())
}

/// Overrides one DMX channel with a fixed value.
///
/// `universe` and `channel` are 1-based (channel 1-512); `value` is 0-255. The override is stored
/// without validation and re-applied after every computed frame, so it wins over fixture output
/// until [`live_reset_all_buttons`]. Out-of-range pairs are silently ignored by the engine.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn set_dmx_channel(
    state: State<'_, AppState>,
    universe: u16,
    channel: u16,
    value: u8,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.manual_channels.insert((universe, channel), value);
    Ok(())
}

/// Overrides the first channels of a universe with `data`.
///
/// `data[0]` goes to channel 1; bytes beyond 512 are dropped. Each byte becomes a permanent
/// override like [`set_dmx_channel`]; channels beyond `data.len()` keep their existing overrides.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn set_dmx_data(
    state: State<'_, AppState>,
    universe: u16,
    data: Vec<u8>,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    for (idx, &val) in data.iter().take(512).enumerate() {
        live.manual_channels
            .insert((universe, (idx + 1) as u16), val);
    }
    Ok(())
}

/// Returns the last computed 512-byte frame of a universe.
///
/// A universe the engine has not computed yet reads as 512 zeros.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn get_dmx_data(state: State<'_, AppState>, universe: u16) -> Result<Vec<u8>, String> {
    let live = state.output.engine.lock().map_err(|e| e.to_string())?;
    Ok(live
        .universe_buffers
        .get(&universe)
        .cloned()
        .unwrap_or_else(|| vec![0u8; 512]))
}

/// Returns the last computed frame of every known universe, keyed by universe number.
///
/// # Errors
///
/// Returns an error message if the engine lock is poisoned.
#[tauri::command]
pub fn get_all_dmx_data(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<u16, Vec<u8>>, String> {
    let live = state.output.engine.lock().map_err(|e| e.to_string())?;
    Ok(live.universe_buffers.clone())
}
