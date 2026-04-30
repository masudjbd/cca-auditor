use auditor_core::config::ToolFingerprint;
use auditor_db::DbPool;
use auditor_fs::SensitivePathConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

use crate::broadcast::{EventSender, MonitorEvent};
use crate::state::{create_state, ActiveSessions};

pub async fn run_supervisor(
    db: Arc<DbPool>,
    fingerprints: Vec<ToolFingerprint>,
    watch_paths: Vec<PathBuf>,
    sensitive_patterns: Vec<SensitivePathConfig>,
    events: EventSender,
) -> Result<()> {
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    let active_sessions: ActiveSessions = create_state();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_clone.cancel();
    });

    // Alert mpsc → broadcast forwarding
    let (alert_tx, mut alert_rx) = mpsc::unbounded_channel();
    let events_for_alerts = events.clone();
    tokio::spawn(async move {
        while let Some((id, kind, severity, detail)) = alert_rx.recv().await {
            let _ = events_for_alerts.send(MonitorEvent::AlertRaised {
                id,
                kind,
                severity,
                detail,
            });
        }
    });

    let mut tasks = JoinSet::new();

    // Resource monitor
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let sessions_clone = active_sessions.clone();
    let events_clone = events.clone();
    tasks.spawn(async move {
        super::resource_monitor::start_monitor(
            db_clone,
            sessions_clone,
            events_clone,
            shutdown_clone,
        ).await
    });

    // Process monitor
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let fingerprints_clone = fingerprints.clone();
    let sessions_clone = active_sessions.clone();
    let events_clone = events.clone();
    tasks.spawn(async move {
        super::process_monitor::start_monitor(
            db_clone,
            fingerprints_clone,
            sessions_clone,
            events_clone,
            shutdown_clone,
        ).await
    });

    // FS monitor with alert forwarding
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let watch_paths_clone = watch_paths.clone();
    let sensitive_clone = sensitive_patterns.clone();
    let alert_tx_clone = alert_tx.clone();
    tasks.spawn(async move {
        auditor_fs::start_watcher(
            db_clone,
            watch_paths_clone,
            sensitive_clone,
            Some(alert_tx_clone),
            shutdown_clone,
        ).await
    });

    // Network monitor (5s polling, only AI tool PIDs)
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let sessions_clone = active_sessions.clone();
    tasks.spawn(async move {
        auditor_net::start_monitor(
            db_clone,
            sessions_clone,
            shutdown_clone,
        ).await
    });

    drop(alert_tx); // Allow forwarder to close when all monitors are done

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
