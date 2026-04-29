use auditor_core::samples::ResourceSample;
use auditor_db::DbPool;
use sysinfo::System;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

pub type SampleBuffer = VecDeque<ResourceSample>;

pub async fn start_monitor(
    db: Arc<DbPool>,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut system = System::new_all();
    let mut ticker = interval(Duration::from_millis(1000)); // 1 Hz base

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("resource monitor shutting down");
                break;
            }
            _ = ticker.tick() => {
                system.refresh_processes();

                for (pid, process) in system.processes() {
                    let sample = ResourceSample::new(
                        pid.as_u32(),
                        process.cpu_usage() as f64,
                        process.memory() as u64 * 1024, // sysinfo returns KB
                        None, // GPU not available without elevation
                    );

                    if let Err(e) = auditor_db::queries::samples::insert_sample(&db, &sample) {
                        tracing::warn!("failed to insert sample: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
