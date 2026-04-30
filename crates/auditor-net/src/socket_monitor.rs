use auditor_core::events::{AuditEvent, EventKind};
use auditor_core::tool::{Confidence, ToolId};
use auditor_db::DbPool;
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo, TcpState,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use anyhow::Result;

/// Cache key for connection deduplication: (pid, remote_addr, remote_port)
type ConnKey = (u32, String, u16);

pub async fn start_monitor(
    db: Arc<DbPool>,
    active_sessions: Arc<RwLock<HashMap<u32, auditor_core::session::AuditSession>>>,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(5));
    let mut seen_connections: HashSet<ConnKey> = HashSet::new();

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("network monitor shutting down");
                break;
            }
            _ = ticker.tick() => {
                if let Err(e) = poll_sockets(&db, &active_sessions, &mut seen_connections).await {
                    tracing::warn!("socket polling error: {}", e);
                }
            }
        }
    }

    Ok(())
}

async fn poll_sockets(
    db: &Arc<DbPool>,
    active_sessions: &Arc<RwLock<HashMap<u32, auditor_core::session::AuditSession>>>,
    seen_connections: &mut HashSet<ConnKey>,
) -> Result<()> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

    let sockets = get_sockets_info(af_flags, proto_flags)?;

    // Get the set of tracked PIDs
    let tracked: HashMap<u32, (Uuid, ToolId)> = {
        let sessions = active_sessions.read().await;
        sessions
            .iter()
            .map(|(pid, s)| (*pid, (s.id, s.tool_id.clone())))
            .collect()
    };

    if tracked.is_empty() {
        return Ok(());
    }

    let mut current_keys: HashSet<ConnKey> = HashSet::new();

    for socket in sockets {
        let SocketInfo {
            protocol_socket_info,
            associated_pids,
            ..
        } = socket;

        // Only attribute to tracked AI tools
        let attributing_pid = associated_pids.iter().find(|p| tracked.contains_key(p));

        if let Some(&pid) = attributing_pid {
            if let Some((session_id, tool_id)) = tracked.get(&pid) {
                let (addr, port, proto, is_established) = match &protocol_socket_info {
                    ProtocolSocketInfo::Tcp(tcp) => {
                        let established = matches!(tcp.state, TcpState::Established);
                        (
                            tcp.remote_addr.to_string(),
                            tcp.remote_port,
                            "tcp",
                            established,
                        )
                    }
                    ProtocolSocketInfo::Udp(udp) => {
                        // UDP has no concept of established; report local→remote when present
                        (udp.local_addr.to_string(), udp.local_port, "udp", true)
                    }
                };

                // Skip loopback to reduce noise
                if addr.starts_with("127.") || addr == "::1" || addr == "0.0.0.0" || addr == "::" {
                    continue;
                }

                if !is_established {
                    continue;
                }

                let key: ConnKey = (pid, addr.clone(), port);
                current_keys.insert(key.clone());

                if !seen_connections.contains(&key) {
                    let event = AuditEvent {
                        id: Uuid::new_v4(),
                        session_id: *session_id,
                        tool_id: tool_id.clone(),
                        kind: EventKind::NetConnect {
                            addr: addr.clone(),
                            port,
                            proto: proto.to_string(),
                        },
                        confidence: Confidence::High,
                        timestamp: OffsetDateTime::now_utc(),
                    };

                    if let Err(e) = auditor_db::queries::events::insert_event(db, &event) {
                        tracing::warn!("failed to insert net event: {}", e);
                    } else {
                        tracing::info!(
                            "net: {} → {}:{} ({})",
                            tool_id.0, addr, port, proto
                        );
                    }
                }
            }
        }
    }

    // Forget connections that are no longer active
    seen_connections.retain(|k| current_keys.contains(k));
    // Add new ones
    for k in current_keys {
        seen_connections.insert(k);
    }

    Ok(())
}
