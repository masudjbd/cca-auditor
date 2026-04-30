use auditor_core::config::ToolFingerprint;
use auditor_db::DbPool;
use auditor_fs::SensitivePathConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Notify, RwLock};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

use crate::broadcast::{EventSender, MonitorEvent};
use crate::state::{create_state, ActiveSessions};

pub type WatchPaths = Arc<RwLock<Vec<PathBuf>>>;

pub async fn run_supervisor(
    db: Arc<DbPool>,
    fingerprints: Vec<ToolFingerprint>,
    watch_paths: WatchPaths,
    sensitive_patterns: Vec<SensitivePathConfig>,
    events: EventSender,
    fs_reload: Arc<Notify>,
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

    // FS monitor with hot-reload + alert forwarding
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let watch_paths_clone = watch_paths.clone();
    let sensitive_clone = sensitive_patterns.clone();
    let alert_tx_clone = alert_tx.clone();
    let fs_reload_clone = fs_reload.clone();
    tasks.spawn(async move {
        auditor_fs::start_watcher(
            db_clone,
            watch_paths_clone,
            sensitive_clone,
            Some(alert_tx_clone),
            fs_reload_clone,
            shutdown_clone,
        ).await
    });

    // Network monitor
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

    // Rollup task
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    tasks.spawn(async move {
        run_rollup_task(db_clone, shutdown_clone).await
    });

    drop(alert_tx);

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

async fn run_rollup_task(db: Arc<DbPool>, shutdown: CancellationToken) -> Result<()> {
    use std::time::Duration;
    use tokio::time::interval;

    let mut ticker_10s = interval(Duration::from_secs(10));
    let mut ticker_1m = interval(Duration::from_secs(60));
    let mut ticker_purge = interval(Duration::from_secs(3600));

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("rollup task shutting down");
                break;
            }
            _ = ticker_10s.tick() => {
                match auditor_db::queries::samples::rollup_samples_10s(&db) {
                    Ok(n) if n > 0 => tracing::debug!("rolled up {} 10s samples", n),
                    Err(e) => tracing::warn!("10s rollup failed: {}", e),
                    _ => {}
                }
            }
            _ = ticker_1m.tick() => {
                match auditor_db::queries::samples::rollup_samples_1m(&db) {
                    Ok(n) if n > 0 => tracing::debug!("rolled up {} 1m samples", n),
                    Err(e) => tracing::warn!("1m rollup failed: {}", e),
                    _ => {}
                }
            }
            _ = ticker_purge.tick() => {
                match auditor_db::queries::samples::purge_old_samples(&db, 86400) {
                    Ok(n) if n > 0 => tracing::info!("purged {} old raw samples", n),
                    Err(e) => tracing::warn!("purge failed: {}", e),
                    _ => {}
                }
                match auditor_db::queries::samples::purge_old_rollups_10s(&db, 86400 * 30) {
                    Ok(n) if n > 0 => tracing::info!("purged {} old 10s rollups", n),
                    Err(e) => tracing::warn!("10s purge failed: {}", e),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
