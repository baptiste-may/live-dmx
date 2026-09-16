pub mod fixtures;
pub mod output;
pub mod show;

use fixtures::{
    check_fixtures_installed, gdtf_fetch_mode, gdtf_get_credentials_status, gdtf_import_file,
    gdtf_logout, gdtf_search, gdtf_set_credentials, gdtf_sync_catalog, reload_fixtures,
};
use output::{
    get_all_dmx_data, get_dmx_data, get_output_interfaces, live_reset_all_buttons,
    live_reset_group_color_effect, live_reset_group_effect, live_set_all_groups_color,
    live_set_button, live_set_color_effect_bpm, live_set_color_fade_order,
    live_set_color_fade_speed, live_set_effect_bpm, live_set_flash, live_set_group_color,
    live_set_group_color_effect, live_set_group_color_effect_width, live_set_group_effect,
    live_set_group_effect_width, live_set_group_master, live_set_group_secondary_color,
    live_set_random_strobe_rate, live_set_simple_flash_fade, live_set_strobe_rate,
    live_set_white_flash_fade, scan_output_interfaces, set_dmx_channel, set_dmx_data,
};
use show::{
    get_pending_open_path, get_recent_shows, handle_open_request, remove_recent_show,
    save_recent_show, show_configure_get_group, show_configure_get_overview, show_create,
    show_get_data, show_group_create, show_group_remove, show_group_rename,
    show_group_update_settings, show_identity_update, show_import_mvr, show_load,
    show_output_add_universe, show_output_get_universes, show_output_remove_universe,
    show_output_set_enabled, show_output_set_target, show_patch_batch, show_patch_fixture,
    show_patch_remove_fixtures, show_patch_toggle_group, show_patch_update_address,
    show_patch_update_name, show_patch_update_universe, show_redo,
    show_register_fixture_definition, show_save, show_undo, show_unload, AppState,
    PendingOpenStore,
};
use tauri::Manager;
#[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
use tauri::RunEvent;
use tauri_plugin_deep_link::DeepLinkExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
            let resolved: Vec<String> = argv
                .into_iter()
                .map(|arg| {
                    let path = std::path::Path::new(&arg);
                    if url::Url::parse(&arg).is_err() && path.is_relative() {
                        std::path::Path::new(&cwd)
                            .join(path)
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        arg
                    }
                })
                .collect();
            handle_open_request(app, &resolved);
        }));
    }

    builder = builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("live-dmx".into()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .max_file_size(1_000_000)
                .build(),
        )
        .plugin(tauri_plugin_deep_link::init())
        .manage(AppState::default())
        .manage(PendingOpenStore::default())
        .invoke_handler(tauri::generate_handler![
            check_fixtures_installed,
            gdtf_search,
            gdtf_set_credentials,
            gdtf_get_credentials_status,
            gdtf_logout,
            gdtf_sync_catalog,
            gdtf_fetch_mode,
            gdtf_import_file,
            reload_fixtures,
            show_register_fixture_definition,
            get_recent_shows,
            save_recent_show,
            remove_recent_show,
            show_get_data,
            show_create,
            show_load,
            show_import_mvr,
            show_save,
            show_unload,
            show_undo,
            show_redo,
            show_identity_update,
            show_configure_get_overview,
            show_configure_get_group,
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
            show_group_update_settings,
            show_output_get_universes,
            show_output_set_target,
            show_output_set_enabled,
            show_output_add_universe,
            show_output_remove_universe,
            get_pending_open_path,
            get_output_interfaces,
            scan_output_interfaces,
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
            live_set_group_effect_width,
            live_set_group_color_effect_width,
            live_reset_group_effect,
            live_reset_group_color_effect,
            live_reset_all_buttons,
            set_dmx_channel,
            set_dmx_data,
            get_dmx_data,
            get_all_dmx_data,
        ])
        .setup(|app| {
            log::info!(target: "live-dmx::main", "starting live-dmx");

            #[cfg(desktop)]
            {
                let app_handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    let urls: Vec<String> =
                        event.urls().into_iter().map(|u| u.to_string()).collect();
                    handle_open_request(&app_handle, &urls);
                });
            }

            #[cfg(desktop)]
            if let Ok(Some(urls)) = app.deep_link().get_current() {
                let urls: Vec<String> = urls.into_iter().map(|u| u.to_string()).collect();
                handle_open_request(app.handle(), &urls);
            }

            #[cfg(any(windows, target_os = "linux"))]
            {
                let args: Vec<String> = std::env::args().skip(1).collect();
                if !args.is_empty() {
                    handle_open_request(app.handle(), &args);
                }
            }

            Ok(())
        });

    builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
            if let RunEvent::Opened { urls } = event {
                let urls: Vec<String> = urls.into_iter().map(|u| u.to_string()).collect();
                handle_open_request(app, &urls);
            }
            #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
            let _ = (app, event);
        });
}
