use auditor_core::config::ToolFingerprint;
use auditor_core::session::AuditSession;
use auditor_core::tool::Confidence;
use auditor_detect::classify;
use auditor_db::DbPool;
use sysinfo::System;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

use crate::broadcast::{EventSender, MonitorEvent};
use crate::state::ActiveSessions;

pub async fn start_monitor(
    db: Arc<DbPool>,
    fingerprints: Vec<ToolFingerprint>,
    active_sessions: ActiveSessions,
    events: EventSender,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut system = System::new_all();
    let mut ticker = interval(Duration::from_secs(2));

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("process monitor shutting down");
                break;
            }
            _ = ticker.tick() => {
                system.refresh_processes();

                let mut current_pids = std::collections::HashSet::new();
                let mut new_sessions: Vec<AuditSession> = Vec::new();

                {
                    let sessions_read = active_sessions.read().await;
                    for (pid, process) in system.processes() {
                        let pid_u32 = pid.as_u32();
                        current_pids.insert(pid_u32);

                        if !sessions_read.contains_key(&pid_u32) {
                            if let Some(tool_id) = classify(process, &fingerprints) {
                                let session = AuditSession::new(
                                    tool_id,
                                    pid_u32,
                                    Confidence::High,
                                );
                                new_sessions.push(session);
                            }
                        }
                    }
                }

                if !new_sessions.is_empty() {
                    let mut sessions_write = active_sessions.write().await;
                    for session in new_sessions {
                        if let Err(e) = auditor_db::queries::sessions::insert_session(&db, &session) {
                            tracing::warn!("failed to insert session: {}", e);
                        } else {
                            tracing::info!(
                                "detected tool: {} (pid {})",
                                session.tool_id.0,
                                session.pid
                            );
                            // Emit event before moving session
                            let _ = events.send(MonitorEvent::SessionOpened {
                                session: session.clone(),
                            });
                            sessions_write.insert(session.pid, session);
                        }
                    }
                }

                // Clean up exited processes
                {
                    let mut sessions_write = active_sessions.write().await;
                    let exited: Vec<u32> = sessions_write
                        .keys()
                        .copied()
                        .filter(|pid| !current_pids.contains(pid))
                        .collect();

                    for pid in exited {
                        if let Some(session) = sessions_write.remove(&pid) {
                            tracing::info!(
                                "process exited: {} (pid {})",
                                session.tool_id.0,
                                pid
                            );
                            let _ = events.send(MonitorEvent::SessionClosed {
                                session_id: session.id.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
