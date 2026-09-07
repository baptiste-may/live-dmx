pub mod fixtures;
pub mod output;
pub mod show;

use fixtures::{check_fixtures_installed, download_fixtures, get_fixtures};
use output::{
    live_reset_all_buttons, live_reset_group_color_effect, live_reset_group_effect,
    live_set_all_groups_color, live_set_button, live_set_color_effect_bpm,
    live_set_color_fade_order, live_set_color_fade_speed, live_set_effect_bpm, live_set_flash,
    live_set_group_color, live_set_group_color_effect, live_set_group_effect,
    live_set_group_master, live_set_group_secondary_color, live_set_random_strobe_rate,
    live_set_simple_flash_fade, live_set_strobe_rate, live_set_white_flash_fade, set_dmx_channel,
    set_dmx_data,
};
use show::{
    get_recent_shows, remove_recent_show, save_recent_show, show_create, show_get_data,
    show_group_create, show_group_remove, show_group_rename, show_identity_update, show_load,
    show_patch_batch, show_patch_fixture, show_patch_remove_fixtures, show_patch_toggle_group,
    show_patch_update_address, show_patch_update_name, show_patch_update_universe, show_redo,
    show_save, show_undo, show_unload, AppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|_app| Ok(()))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_fixtures_installed,
            download_fixtures,
            get_fixtures,
            get_recent_shows,
            save_recent_show,
            remove_recent_show,
            show_get_data,
            show_create,
            show_load,
            show_save,
            show_unload,
            show_undo,
            show_redo,
            show_identity_update,
            show_patch_fixture,
            show_patch_batch,
            show_patch_remove_fixtures,
            show_patch_update_name,
            show_patch_update_address,
            show_patch_update_universe,
            show_patch_toggle_group,
            show_group_create,
            show_group_remove,
            show_group_rename,
            live_set_flash,
            live_set_button,
            live_set_group_effect,
            live_set_group_color_effect,
            live_set_group_master,
            live_set_group_color,
            live_set_group_secondary_color,
            live_set_all_groups_color,
            live_set_color_fade_speed,
            live_set_color_fade_order,
            live_set_strobe_rate,
            live_set_random_strobe_rate,
            live_set_simple_flash_fade,
            live_set_white_flash_fade,
            live_set_effect_bpm,
            live_set_color_effect_bpm,
            live_reset_group_effect,
            live_reset_group_color_effect,
            live_reset_all_buttons,
            set_dmx_channel,
            set_dmx_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
