use auditor_db::DbPool;
use std::time::Duration;
use std::sync::Arc;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

pub async fn start_monitor(
    _db: Arc<DbPool>,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("network monitor shutting down");
                break;
            }
            _ = ticker.tick() => {
                // TODO: use netstat2 to enumerate sockets per PID
                // netstat2::iterate_sockets_info(AddressFamily::Ipv4)?
                // For each socket: resolve IP -> hostname async
                // Emit NetConnect event if new connection detected

                tracing::debug!("polling network sockets");
            }
        }
    }

    Ok(())
}
