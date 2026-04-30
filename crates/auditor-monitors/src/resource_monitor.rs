use auditor_core::samples::ResourceSample;
use auditor_db::DbPool;
use sysinfo::{Pid, System};
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use anyhow::Result;

use crate::broadcast::{EventSender, MonitorEvent};
use crate::state::ActiveSessions;

pub type SampleBuffer = VecDeque<ResourceSample>;

pub async fn start_monitor(
    db: Arc<DbPool>,
    active_sessions: ActiveSessions,
    events: EventSender,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut system = System::new_all();
    let mut ticker = interval(Duration::from_millis(1000));

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("resource monitor shutting down");
                break;
            }
            _ = ticker.tick() => {
                let tracked_pids: Vec<u32> = {
                    let sessions = active_sessions.read().await;
                    sessions.keys().copied().collect()
                };

                if tracked_pids.is_empty() {
                    continue;
                }

                system.refresh_processes();

                for pid_u32 in tracked_pids {
                    let pid = Pid::from_u32(pid_u32);
                    if let Some(process) = system.process(pid) {
                        let sample = ResourceSample::new(
                            pid_u32,
                            process.cpu_usage() as f64,
                            process.memory() as u64 * 1024,
                            None,
                        );

                        if let Err(e) = auditor_db::queries::samples::insert_sample(&db, &sample) {
                            tracing::warn!("failed to insert sample: {}", e);
                        }

                        // Broadcast sample to frontend (non-blocking)
                        let _ = events.send(MonitorEvent::ResourceSample {
                            sample: sample.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}
