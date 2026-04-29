use auditor_core::config::ToolFingerprint;
use auditor_core::session::AuditSession;
use auditor_core::tool::Confidence;
use auditor_detect::classify;
use auditor_db::DbPool;
use sysinfo::System;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

pub async fn start_monitor(
    db: Arc<DbPool>,
    fingerprints: Vec<ToolFingerprint>,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut system = System::new_all();
    let mut ticker = interval(Duration::from_secs(2));
    let mut active_pids: HashMap<u32, AuditSession> = HashMap::new();

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("process monitor shutting down");
                break;
            }
            _ = ticker.tick() => {
                system.refresh_processes();

                let mut current_pids = std::collections::HashSet::new();

                for (pid, process) in system.processes() {
                    let pid_u32 = pid.as_u32();
                    current_pids.insert(pid_u32);

                    if !active_pids.contains_key(&pid_u32) {
                        // New process detected
                        if let Some(tool_id) = classify(process, &fingerprints) {
                            let session = AuditSession::new(
                                tool_id,
                                pid_u32,
                                Confidence::High,
                            );

                            if let Err(e) = auditor_db::queries::sessions::insert_session(&db, &session) {
                                tracing::warn!("failed to insert session: {}", e);
                            } else {
                                tracing::info!("detected tool: {} (pid {})", session.tool_id.0, pid_u32);
                                active_pids.insert(pid_u32, session);
                            }
                        }
                    }
                }

                // Clean up exited processes
                for (pid, _) in active_pids.iter() {
                    if !current_pids.contains(pid) {
                        tracing::info!("process exited: pid {}", pid);
                    }
                }
                active_pids.retain(|pid, _| current_pids.contains(pid));
            }
        }
    }

    Ok(())
}
