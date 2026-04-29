#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("CCAudit starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
        ])
        .setup(|_app| {
            // TODO: Initialize database pool
            // TODO: Load tool fingerprints from config/tools.toml
            // TODO: Spawn monitor supervisor in background
            // TODO: Set up event broadcasting from monitors to frontend
            // TODO: Setup system tray with icon
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
