use auditor_core::config::ToolFingerprint;
use auditor_db::DbPool;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

use crate::state::{create_state, ActiveSessions};

pub async fn run_supervisor(
    db: Arc<DbPool>,
    fingerprints: Vec<ToolFingerprint>,
) -> Result<()> {
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // Shared state across all monitors
    let active_sessions: ActiveSessions = create_state();

    // Handle Ctrl+C to trigger graceful shutdown
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_clone.cancel();
    });

    let mut tasks = JoinSet::new();

    // Spawn resource monitor (1 Hz sampling, only AI tool processes)
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let sessions_clone = active_sessions.clone();
    tasks.spawn(async move {
        super::resource_monitor::start_monitor(db_clone, sessions_clone, shutdown_clone).await
    });

    // Spawn process monitor (2 s polling)
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

    // Wait for all tasks to complete
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
