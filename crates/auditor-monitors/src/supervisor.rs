use auditor_core::config::ToolFingerprint;
use auditor_db::DbPool;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

pub async fn run_supervisor(
    db: Arc<DbPool>,
    fingerprints: Vec<ToolFingerprint>,
) -> Result<()> {
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // Handle Ctrl+C to trigger graceful shutdown
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_clone.cancel();
    });

    let mut tasks = JoinSet::new();

    // Spawn resource monitor (1 Hz sampling)
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    tasks.spawn(async move {
        super::resource_monitor::start_monitor(db_clone, shutdown_clone).await
    });

    // Spawn process monitor (2 s polling)
    let db_clone = db.clone();
    let shutdown_clone = shutdown.clone();
    let fingerprints_clone = fingerprints.clone();
    tasks.spawn(async move {
        super::process_monitor::start_monitor(db_clone, fingerprints_clone, shutdown_clone).await
    });

    // Spawn FS monitor
    let _db_clone = db.clone();
    let _shutdown_clone = shutdown.clone();
    tasks.spawn(async move {
        // TODO: implement FS monitor
        Ok(())
    });

    // Spawn network monitor
    let _db_clone = db.clone();
    let _shutdown_clone = shutdown.clone();
    tasks.spawn(async move {
        // TODO: implement network monitor
        Ok(())
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
