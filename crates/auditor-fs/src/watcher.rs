use auditor_core::events::{AuditEvent, EventKind};
use auditor_core::tool::Confidence;
use auditor_db::DbPool;
use notify::{Watcher, RecursiveMode};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

pub async fn start_watcher(
    _db: Arc<DbPool>,
    shutdown: CancellationToken,
) -> Result<()> {
    let (_tx, _rx): (tokio::sync::mpsc::UnboundedSender<()>, _) = tokio::sync::mpsc::unbounded_channel();

    // Watch home directory and common project paths
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let _watch_paths = vec![
        home.clone(),
        home.join("projects"),
        home.join("code"),
        home.join("workspace"),
    ];

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("FS watcher shutting down");
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                // TODO: implement notify watcher with proper event handling
                // TODO: attribute to active tool session
                tracing::debug!("FS monitor tick");
            }
        }
    }

    Ok(())
}
