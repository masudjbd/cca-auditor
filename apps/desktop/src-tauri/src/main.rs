#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;
use auditor_db::DbPool;
use auditor_ipc::commands::*;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("CCAudit starting...");

    // Set up DB path: ~/.cca-audit/audit.db
    let home = dirs::home_dir().expect("could not find home directory");
    let cca_dir = home.join(".cca-audit");
    std::fs::create_dir_all(&cca_dir).expect("failed to create ~/.cca-audit");
    let db_path = cca_dir.join("audit.db");

    tracing::info!("opening database at {:?}", db_path);

    // Initialize DB pool
    let db_pool: Arc<DbPool> = Arc::new(
        auditor_db::create_pool(&db_path).expect("failed to create database pool")
    );

    // Load tool fingerprints
    let config_path = std::env::current_dir()
        .ok()
        .and_then(|p| {
            let candidates = [
                p.join("config/tools.toml"),
                p.join("../../config/tools.toml"),
                p.join("../../../config/tools.toml"),
            ];
            candidates.iter().find(|p| p.exists()).cloned()
        });

    let fingerprints = match config_path {
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

    // Build Tauri app
    let db_pool_for_monitors = db_pool.clone();
    let fingerprints_for_monitors = fingerprints.clone();

    tauri::Builder::default()
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
        .setup(move |_app| {
            tracing::info!("Tauri app initialized");

            // Spawn monitor supervisor using Tauri's async runtime
            let db_clone = db_pool_for_monitors.clone();
            let fps_clone = fingerprints_for_monitors.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = auditor_monitors::run_supervisor(
                    db_clone,
                    fps_clone,
                ).await {
                    tracing::error!("monitor supervisor failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
