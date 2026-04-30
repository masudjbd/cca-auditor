use auditor_core::events::{AuditEvent, EventKind};
use auditor_core::tool::{Confidence, ToolId};
use auditor_db::DbPool;
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo, TcpState,
};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use anyhow::Result;

/// DNS cache: addr → (hostname, expiry)
struct DnsCache {
    entries: HashMap<String, (Option<String>, Instant)>,
    ttl: Duration,
}

impl DnsCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Duration::from_secs(300), // 5 min cache
        }
    }

    fn get_or_resolve(&mut self, addr: &str) -> Option<String> {
        let now = Instant::now();
        if let Some((host, expiry)) = self.entries.get(addr) {
            if now < *expiry {
                return host.clone();
            }
        }

        let resolved = if let Ok(ip) = IpAddr::from_str(addr) {
            dns_lookup::lookup_addr(&ip).ok()
        } else {
            None
        };

        let host = resolved.filter(|h| !h.is_empty() && h != addr);
        self.entries.insert(addr.to_string(), (host.clone(), now + self.ttl));
        host
    }
}

/// Cache key for connection deduplication: (pid, remote_addr, remote_port)
type ConnKey = (u32, String, u16);

pub async fn start_monitor(
    db: Arc<DbPool>,
    active_sessions: Arc<RwLock<HashMap<u32, auditor_core::session::AuditSession>>>,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(5));
    let mut seen_connections: HashSet<ConnKey> = HashSet::new();
    let mut dns_cache = DnsCache::new();

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("network monitor shutting down");
                break;
            }
            _ = ticker.tick() => {
                if let Err(e) = poll_sockets(&db, &active_sessions, &mut seen_connections, &mut dns_cache).await {
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
    dns_cache: &mut DnsCache,
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
                    let hostname = dns_cache.get_or_resolve(&addr);
                    let display_addr = if let Some(host) = &hostname {
                        format!("{} ({})", host, addr)
                    } else {
                        addr.clone()
                    };

                    let event = AuditEvent {
                        id: 0,
                        session_id: *session_id,
                        tool_id: tool_id.clone(),
                        kind: EventKind::NetConnect {
                            addr: display_addr,
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
                            tool_id.0,
                            hostname.as_deref().unwrap_or(&addr),
                            port,
                            proto
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
