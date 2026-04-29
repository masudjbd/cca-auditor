use auditor_core::config::ToolFingerprint;
use auditor_db::DbPool;
use auditor_fs::SensitivePathConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

use crate::state::{create_state, ActiveSessions};

pub async fn run_supervisor(
    db: Arc<DbPool>,
    fingerprints: Vec<ToolFingerprint>,
    watch_paths: Vec<PathBuf>,
    sensitive_patterns: Vec<SensitivePathConfig>,
) -> Result<()> {
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // Shared state across all monitors
    let active_sessions: ActiveSessions = create_state();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_clone.cancel();
    });

    let mut tasks = JoinSet::new();

    // Resource monitor (1 Hz, only AI tool processes)
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let sessions_clone = active_sessions.clone();
    tasks.spawn(async move {
        super::resource_monitor::start_monitor(db_clone, sessions_clone, shutdown_clone).await
    });

    // Process monitor (2s polling)
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let fingerprints_clone = fingerprints.clone();
    let sessions_clone = active_sessions.clone();
    tasks.spawn(async move {
        super::process_monitor::start_monitor(
            db_clone,
            fingerprints_clone,
            sessions_clone,
            shutdown_clone,
        ).await
    });

    // FS monitor with sensitive path detection
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let watch_paths_clone = watch_paths.clone();
    let sensitive_clone = sensitive_patterns.clone();
    tasks.spawn(async move {
        auditor_fs::start_watcher(
            db_clone,
            watch_paths_clone,
            sensitive_clone,
            shutdown_clone,
        ).await
    });

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::error!("monitor task failed: {}", e),
            Err(e) => tracing::error!("task join error: {}", e),
        }
    }

    tracing::info!("all monitors shut down");
    Ok(())
}
