#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;
use std::sync::Arc;
use auditor_db::DbPool;
use auditor_ipc::commands::*;
use auditor_monitors::WatchPaths;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tokio::sync::{Notify, RwLock};
use std::collections::HashMap;

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

#[tauri::command]
async fn save_settings_with_reload(
    watch_paths_state: tauri::State<'_, WatchPaths>,
    fs_reload: tauri::State<'_, Arc<Notify>>,
    settings: auditor_ipc::commands::AppSettings,
) -> Result<(), String> {
    // 1. Save to disk
    auditor_ipc::commands::save_settings_impl(&settings)?;

    // 2. Update shared watch paths
    let home = dirs::home_dir().ok_or("could not find home directory")?;
    let new_paths: Vec<PathBuf> = settings
        .watch_paths
        .iter()
        .map(|p| expand_path(p, &home))
        .collect();

    {
        let mut paths = watch_paths_state.write().await;
        *paths = new_paths;
    }

    // 3. Signal FS watcher to rebuild
    fs_reload.notify_one();
    tracing::info!("settings saved + FS watcher reload signaled");

    Ok(())
}

fn expand_path(p: &str, home: &std::path::Path) -> PathBuf {
    if p.starts_with("~/") {
        home.join(&p[2..])
    } else if p == "~" {
        home.to_path_buf()
    } else {
        PathBuf::from(p)
    }
}

fn compute_tray_summary(pool: &Arc<DbPool>) -> String {
    use auditor_db::queries::sessions::get_sessions;

    let sessions = get_sessions(pool, 100).unwrap_or_default();
    let active: Vec<_> = sessions.iter().filter(|s| s.ended_at.is_none()).collect();

    if active.is_empty() {
        return "CCAudit — No tools active".to_string();
    }

    let mut tool_counts: HashMap<String, usize> = HashMap::new();
    for session in &active {
        *tool_counts.entry(session.tool_id.0.clone()).or_insert(0) += 1;
    }

    let tool_summary: Vec<String> = tool_counts
        .iter()
        .map(|(tool, count)| {
            if *count > 1 {
                format!("{} ({})", tool, count)
            } else {
                tool.clone()
            }
        })
        .collect();

    format!(
        "CCAudit — {} session{} • {}",
        active.len(),
        if active.len() == 1 { "" } else { "s" },
        tool_summary.join(", ")
    )
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
    let initial_watch_paths: Vec<PathBuf> = user_settings
        .watch_paths
        .iter()
        .map(|p| expand_path(p, &home))
        .collect();
    tracing::info!("watching {} paths", initial_watch_paths.len());

    // Shared, mutable watch paths (allows hot-reload from save_settings)
    let watch_paths: WatchPaths = Arc::new(RwLock::new(initial_watch_paths));
    let fs_reload_signal = Arc::new(Notify::new());

    let db_pool_for_monitors = db_pool.clone();
    let fingerprints_for_monitors = fingerprints.clone();
    let watch_paths_for_monitors = watch_paths.clone();
    let fs_reload_for_monitors = fs_reload_signal.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(db_pool)
        .manage(watch_paths.clone())
        .manage(fs_reload_signal.clone())
        .invoke_handler(tauri::generate_handler![
            get_live_sessions,
            get_events,
            get_samples,
            get_alerts,
            dismiss_alert,
            generate_report,
            push_with_guardrail,
            save_settings,
            save_settings_with_reload,
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

            let tray = TrayIconBuilder::with_id("main-tray")
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

            // Live tray tooltip updater (every 2s)
            let app_handle_for_tray = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(2));
                loop {
                    ticker.tick().await;
                    if let Some(tray) = app_handle_for_tray.tray_by_id("main-tray") {
                        let pool: tauri::State<Arc<DbPool>> = app_handle_for_tray.state();
                        let summary = compute_tray_summary(&pool);
                        let _ = tray.set_tooltip(Some(&summary));
                    }
                }
            });
            // Drop unused tray binding
            drop(tray);

            // Spawn monitor supervisor with broadcast channel
            let db_clone = db_pool_for_monitors.clone();
            let fps_clone = fingerprints_for_monitors.clone();
            let watch_paths_clone = watch_paths_for_monitors.clone();
            let sensitive_clone = sensitive_patterns.clone();
            let event_tx_clone = event_tx.clone();
            let fs_reload_clone = fs_reload_for_monitors.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = auditor_monitors::run_supervisor(
                    db_clone,
                    fps_clone,
                    watch_paths_clone,
                    sensitive_clone,
                    event_tx_clone,
                    fs_reload_clone,
                ).await {
                    tracing::error!("monitor supervisor failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
