use tauri::State;

use crate::show::AppState;

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

#[tauri::command]
pub fn live_reset_group_color_effect(
    state: State<'_, AppState>,
    group_id: u32,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.reset_group_color_effect(group_id);
    Ok(())
}

#[tauri::command]
pub fn live_set_color_fade_speed(state: State<'_, AppState>, speed: f32) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_color_fade_speed(speed);
    Ok(())
}

#[tauri::command]
pub fn live_set_color_fade_order(
    state: State<'_, AppState>,
    order: Option<String>,
) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_color_fade_order(order);
    Ok(())
}

#[tauri::command]
pub fn live_set_strobe_rate(state: State<'_, AppState>, rate: u8) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.strobe_rate = rate;
    Ok(())
}

#[tauri::command]
pub fn live_set_random_strobe_rate(state: State<'_, AppState>, rate: u8) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.random_strobe_rate = rate;
    Ok(())
}

#[tauri::command]
pub fn live_set_simple_flash_fade(state: State<'_, AppState>, fade: f32) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_simple_flash_fade(fade);
    Ok(())
}

#[tauri::command]
pub fn live_set_white_flash_fade(state: State<'_, AppState>, fade: f32) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_white_flash_fade(fade);
    Ok(())
}

#[tauri::command]
pub fn live_set_effect_bpm(state: State<'_, AppState>, bpm: u16) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_effect_bpm(bpm);
    Ok(())
}

#[tauri::command]
pub fn live_set_color_effect_bpm(state: State<'_, AppState>, bpm: u16) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.set_color_effect_bpm(bpm);
    Ok(())
}

#[tauri::command]
pub fn live_reset_group_effect(state: State<'_, AppState>, group_id: u32) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.reset_group_effect(group_id);
    Ok(())
}

#[tauri::command]
pub fn live_reset_all_buttons(state: State<'_, AppState>) -> Result<(), String> {
    let mut live = state.output.engine.lock().map_err(|e| e.to_string())?;
    live.reset_all();
    Ok(())
}

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
