use auditor_core::events::{AuditEvent, EventKind};
use auditor_core::tool::Confidence;
use auditor_core::error::CcaError;
use crate::DbPool;
use uuid::Uuid;
use time::OffsetDateTime;

pub fn insert_event(pool: &DbPool, event: &AuditEvent) -> auditor_core::error::Result<()> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;

    let kind_str = match &event.kind {
        EventKind::FsRead { .. } => "fs_read",
        EventKind::FsWrite { .. } => "fs_write",
        EventKind::FsDelete { .. } => "fs_delete",
        EventKind::NetConnect { .. } => "net_connect",
        EventKind::ProcessSpawn { .. } => "process_spawn",
        EventKind::LocalArtifact { .. } => "local_artifact",
    };

    let (path, dest_addr, dest_port, dest_hostname): (Option<&str>, Option<&str>, Option<i32>, Option<&str>) = match &event.kind {
        EventKind::FsRead { path } | EventKind::FsWrite { path } | EventKind::FsDelete { path } => {
            (Some(path.as_str()), None, None, None)
        }
        EventKind::NetConnect { addr, port, .. } => {
            (None, Some(addr.as_str()), Some(*port as i32), None)
        }
        EventKind::ProcessSpawn { .. } => (None, None, None, None),
        EventKind::LocalArtifact { path, .. } => (Some(path.as_str()), None, None, None),
    };

    conn.execute(
        "INSERT INTO events (session_id, kind, path, dest_addr, dest_port, dest_hostname, confidence, ts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            event.session_id.to_string(),
            kind_str,
            path,
            dest_addr,
            dest_port,
            dest_hostname,
            event.confidence.to_string(),
            event.timestamp.unix_timestamp(),
        ],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(())
}

pub fn get_events(
    pool: &DbPool,
    session_id: Uuid,
    limit: u32,
) -> auditor_core::error::Result<Vec<AuditEvent>> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let mut stmt = conn
        .prepare("SELECT id, session_id, kind, path, dest_addr, dest_port, confidence, ts FROM events WHERE session_id = ?1 ORDER BY ts DESC LIMIT ?2")
        .map_err(|e| CcaError::Database(e.to_string()))?;

    let events = stmt
        .query_map(rusqlite::params![session_id.to_string(), limit], |row| {
            let _id: i64 = row.get(0)?;
            let session_id_str: String = row.get(1)?;
            let kind_str: String = row.get(2)?;
            let path: Option<String> = row.get(3)?;
            let dest_addr: Option<String> = row.get(4)?;
            let dest_port: Option<i32> = row.get(5)?;
            let confidence_str: String = row.get(6)?;
            let ts: i64 = row.get(7)?;

            let kind = match kind_str.as_str() {
                "fs_read" => EventKind::FsRead {
                    path: path.unwrap_or_default(),
                },
                "fs_write" => EventKind::FsWrite {
                    path: path.unwrap_or_default(),
                },
                "fs_delete" => EventKind::FsDelete {
                    path: path.unwrap_or_default(),
                },
                "net_connect" => EventKind::NetConnect {
                    addr: dest_addr.unwrap_or_default(),
                    port: dest_port.unwrap_or(0) as u16,
                    proto: "tcp".to_string(),
                },
                "process_spawn" => EventKind::ProcessSpawn {
                    exe: String::new(),
                    args: vec![],
                },
                "local_artifact" => EventKind::LocalArtifact {
                    path: path.unwrap_or_default(),
                    artifact_type: String::new(),
                },
                _ => return Err(rusqlite::Error::InvalidParameterName("unknown kind".into())),
            };

            let confidence = match confidence_str.as_str() {
                "High" => Confidence::High,
                "Ambiguous" => Confidence::Ambiguous,
                "Verified" => Confidence::Verified,
                _ => Confidence::Ambiguous,
            };

            let timestamp = OffsetDateTime::from_unix_timestamp(ts)
                .unwrap_or_else(|_| OffsetDateTime::now_utc());

            Ok(AuditEvent {
                id: Uuid::parse_str(&_id.to_string()).unwrap_or_else(|_| Uuid::new_v4()),
                session_id: Uuid::parse_str(&session_id_str).unwrap_or_else(|_| Uuid::new_v4()),
                tool_id: auditor_core::tool::ToolId::new("unknown"),
                kind,
                confidence,
                timestamp,
            })
        })
        .map_err(|e| CcaError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(events)
}
