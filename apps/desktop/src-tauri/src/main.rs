#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;
use std::sync::Arc;
use auditor_db::DbPool;
use auditor_ipc::commands::*;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

fn find_config_file(name: &str) -> Option<PathBuf> {
    std::env::current_dir().ok().and_then(|p| {
        let candidates = [
            p.join(format!("config/{}", name)),
            p.join(format!("../../config/{}", name)),
            p.join(format!("../../../config/{}", name)),
        ];
        candidates.iter().find(|p| p.exists()).cloned()
    })
}

fn load_user_settings() -> AppSettings {
    let path = dirs::home_dir()
        .map(|h| h.join(".cca-audit").join("settings.json"));

    if let Some(path) = path {
        if path.exists() {
            if let Ok(json) = std::fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&json) {
                    return settings;
                }
            }
        }
    }
    AppSettings::default()
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("CCAudit starting...");

    // Initialize DB
    let home = dirs::home_dir().expect("could not find home directory");
    let cca_dir = home.join(".cca-audit");
    std::fs::create_dir_all(&cca_dir).expect("failed to create ~/.cca-audit");
    let db_path = cca_dir.join("audit.db");

    tracing::info!("opening database at {:?}", db_path);
    let db_pool: Arc<DbPool> = Arc::new(
        auditor_db::create_pool(&db_path).expect("failed to create database pool")
    );

    // Load tool fingerprints
    let fingerprints = match find_config_file("tools.toml") {
        Some(p) => {
            tracing::info!("loading tool fingerprints from {:?}", p);
            let path_str = p.to_string_lossy();
            auditor_detect::load_fingerprints(&path_str).unwrap_or_else(|e| {
                tracing::warn!("failed to load fingerprints: {}", e);
                vec![]
            })
        }
        None => {
            tracing::warn!("config/tools.toml not found, using empty fingerprints");
            vec![]
        }
    };
    tracing::info!("loaded {} tool fingerprints", fingerprints.len());

    // Seed tools table to satisfy FOREIGN KEY constraints
    if let Err(e) = auditor_db::queries::tools::seed_tools(&db_pool, &fingerprints) {
        tracing::warn!("failed to seed tools table: {}", e);
    } else {
        tracing::info!("seeded tools table");
    }

    // Load sensitive paths config
    let sensitive_patterns = match find_config_file("sensitive-paths.toml") {
        Some(p) => {
            tracing::info!("loading sensitive paths from {:?}", p);
            auditor_fs::load_sensitive_paths(&p).unwrap_or_else(|e| {
                tracing::warn!("failed to load sensitive paths: {}", e);
                vec![]
            })
        }
        None => {
            tracing::warn!("config/sensitive-paths.toml not found");
            vec![]
        }
    };
    tracing::info!("loaded {} sensitive path patterns", sensitive_patterns.len());

    // Load user-configured watch paths from settings.json
    let user_settings = load_user_settings();
    let watch_paths: Vec<PathBuf> = user_settings
        .watch_paths
        .iter()
        .map(|p| {
            // Expand ~ to home directory
            if p.starts_with("~/") {
                home.join(&p[2..])
            } else {
                PathBuf::from(p)
            }
        })
        .collect();
    tracing::info!("watching {} paths", watch_paths.len());

    let db_pool_for_monitors = db_pool.clone();
    let fingerprints_for_monitors = fingerprints.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Focus existing window if user tries to launch a 2nd instance
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(db_pool)
        .invoke_handler(tauri::generate_handler![
            get_live_sessions,
            get_events,
            get_samples,
            get_alerts,
            dismiss_alert,
            generate_report,
            push_with_guardrail,
            save_settings,
            load_settings,
        ])
        .setup(move |app| {
            tracing::info!("Tauri app initialized");

            // Create monitor event broadcast channel
            let (event_tx, mut event_rx) = auditor_monitors::create_channel();

            // Bridge: forward broadcast events to Tauri emit
            let app_handle_for_bridge = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(event) = event_rx.recv().await {
                    use auditor_monitors::MonitorEvent;
                    let result = match &event {
                        MonitorEvent::SessionOpened { session } => {
                            app_handle_for_bridge.emit("session-opened", serde_json::json!({
                                "session": session
                            }))
                        }
                        MonitorEvent::SessionClosed { session_id } => {
                            app_handle_for_bridge.emit("session-closed", serde_json::json!({
                                "session_id": session_id
                            }))
                        }
                        MonitorEvent::ResourceSample { sample } => {
                            app_handle_for_bridge.emit("resource-sample", serde_json::json!({
                                "sample": sample
                            }))
                        }
                        MonitorEvent::AlertRaised { id, kind, severity, detail } => {
                            app_handle_for_bridge.emit("alert-raised", serde_json::json!({
                                "alert": {
                                    "id": id,
                                    "kind": kind,
                                    "severity": severity,
                                    "detail": detail,
                                    "timestamp": time::OffsetDateTime::now_utc().unix_timestamp() * 1000,
                                    "dismissed": false,
                                }
                            }))
                        }
                    };
                    if let Err(e) = result {
                        tracing::warn!("failed to emit Tauri event: {}", e);
                    }
                }
            });

            // System tray menu
            let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit CCAudit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("CCAudit — AI Tool Auditor")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Spawn monitor supervisor with broadcast channel
            let db_clone = db_pool_for_monitors.clone();
            let fps_clone = fingerprints_for_monitors.clone();
            let watch_paths_clone = watch_paths.clone();
            let sensitive_clone = sensitive_patterns.clone();
            let event_tx_clone = event_tx.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = auditor_monitors::run_supervisor(
                    db_clone,
                    fps_clone,
                    watch_paths_clone,
                    sensitive_clone,
                    event_tx_clone,
                ).await {
                    tracing::error!("monitor supervisor failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
