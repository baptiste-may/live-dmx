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
use tauri_plugin_log::fern;

const ANSI_RESET: &str = "\x1b[0m";

fn ansi(bold: bool, color: fern::colors::Color) -> String {
    if bold {
        format!("\x1b[1;{}m", color.to_fg_str())
    } else {
        format!("\x1b[{}m", color.to_fg_str())
    }
}

fn level_style(level: log::Level) -> (bool, fern::colors::Color) {
    match level {
        log::Level::Error => (true, fern::colors::Color::BrightRed),
        log::Level::Warn => (true, fern::colors::Color::BrightYellow),
        log::Level::Info => (true, fern::colors::Color::BrightGreen),
        log::Level::Debug => (false, fern::colors::Color::Blue),
        log::Level::Trace => (false, fern::colors::Color::BrightBlack),
    }
}

fn message_style(level: log::Level) -> Option<fern::colors::Color> {
    match level {
        log::Level::Error => Some(fern::colors::Color::BrightRed),
        log::Level::Warn => Some(fern::colors::Color::BrightYellow),
        _ => None,
    }
}

fn render_record(
    timestamp: &str,
    target: &str,
    level: log::Level,
    message: &str,
    colored: bool,
) -> String {
    if !colored {
        return format!("{timestamp}[{target}][{level}] {message}");
    }
    let dim = ansi(false, fern::colors::Color::BrightBlack);
    let (bold, color) = level_style(level);
    let label = format!("{}{level}{ANSI_RESET}", ansi(bold, color));
    let line =
        format!("{dim}{timestamp}{ANSI_RESET}[{dim}{target}{ANSI_RESET}][{label}] {message}");
    match message_style(level) {
        Some(color) => format!("{}{line}{ANSI_RESET}", ansi(false, color)),
        None => line,
    }
}

fn log_formatter(
    colored: bool,
) -> impl Fn(fern::FormatCallback, &std::fmt::Arguments, &log::Record) + Send + Sync + 'static {
    move |out, message, record| {
        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::macros::format_description!(
                "[[[year]-[month]-[day]][[[hour]:[minute]:[second]]"
            ))
            .unwrap_or_default();
        let line = render_record(
            &timestamp,
            record.target(),
            record.level(),
            &message.to_string(),
            colored,
        );
        out.finish(format_args!("{line}"));
    }
}

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
        builder = builder.plugin(tauri_plugin_process::init());
    }

    builder = builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout)
                        .format(log_formatter(true)),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("live-dmx".into()),
                    })
                    .format(log_formatter(false)),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview)
                        .format(log_formatter(false)),
                ])
                .clear_format()
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
        .plugin(tauri_plugin_window_state::Builder::default().build())
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
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn record(level: log::Level, message: &str) -> String {
        let timestamp = "[2026-09-16][19:50:02]";
        render_record(timestamp, "live-dmx::test", level, message, true)
    }

    #[test]
    fn plain_output_has_no_ansi_escape() {
        let line = render_record("[ts]", "live-dmx::test", log::Level::Error, "boom", false);
        assert!(!line.contains("\x1b["));
        assert_eq!(line, "[ts][live-dmx::test][ERROR] boom");
    }

    #[test]
    fn colored_output_wraps_level_in_ansi_escape() {
        assert!(record(log::Level::Error, "boom").contains("\x1b["));
        assert!(record(log::Level::Info, "hi").contains("\x1b["));
    }

    #[test]
    fn error_level_is_bold_bright_red() {
        let line = record(log::Level::Error, "boom");
        assert!(line.contains("\x1b[1;91mERROR\x1b[0m"));
        assert!(line.ends_with("\x1b[0m"));
    }

    #[test]
    fn warn_level_is_bold_bright_yellow() {
        assert!(record(log::Level::Warn, "careful").contains("\x1b[1;93mWARN\x1b[0m"));
    }

    #[test]
    fn info_level_is_bold_bright_green() {
        assert!(record(log::Level::Info, "hello").contains("\x1b[1;92mINFO\x1b[0m"));
    }

    #[test]
    fn debug_and_trace_levels_are_not_bold() {
        assert!(record(log::Level::Debug, "dbg").contains("\x1b[34mDEBUG\x1b[0m"));
        assert!(record(log::Level::Trace, "trc").contains("\x1b[90mTRACE\x1b[0m"));
    }

    #[test]
    fn error_and_warn_messages_are_tinted() {
        let error_line = record(log::Level::Error, "boom");
        assert!(error_line.contains("\x1b[91m"));
        assert!(error_line.ends_with("boom\x1b[0m"));
        let warn_line = record(log::Level::Warn, "careful");
        assert!(warn_line.contains("\x1b[93m"));
        assert!(warn_line.ends_with("careful\x1b[0m"));
    }
}
