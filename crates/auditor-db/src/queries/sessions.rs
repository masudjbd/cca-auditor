use auditor_core::session::AuditSession;
use auditor_core::tool::{Confidence, ToolId};
use auditor_core::error::CcaError;
use crate::DbPool;
use uuid::Uuid;
use time::OffsetDateTime;
use rusqlite::OptionalExtension;

pub fn insert_session(pool: &DbPool, session: &AuditSession) -> auditor_core::error::Result<()> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;

    conn.execute(
        "INSERT INTO sessions (id, tool_id, pid, started_at, ended_at, confidence)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            session.id.to_string(),
            session.tool_id.0,
            session.pid as i32,
            session.started_at.unix_timestamp(),
            session.ended_at.map(|t| t.unix_timestamp()),
            session.confidence.to_string(),
        ],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(())
}

pub fn get_sessions(pool: &DbPool, limit: u32) -> auditor_core::error::Result<Vec<AuditSession>> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, tool_id, pid, started_at, ended_at, confidence FROM sessions ORDER BY started_at DESC LIMIT ?1",
        )
        .map_err(|e| CcaError::Database(e.to_string()))?;

    let sessions = stmt
        .query_map(rusqlite::params![limit], |row| {
            let id_str: String = row.get(0)?;
            let tool_id_str: String = row.get(1)?;
            let pid: i32 = row.get(2)?;
            let started_at_ts: i64 = row.get(3)?;
            let ended_at_ts: Option<i64> = row.get(4)?;
            let confidence_str: String = row.get(5)?;

            let confidence = match confidence_str.as_str() {
                "High" => Confidence::High,
                "Ambiguous" => Confidence::Ambiguous,
                "Verified" => Confidence::Verified,
                _ => Confidence::Ambiguous,
            };

            Ok(AuditSession {
                id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
                tool_id: ToolId::new(tool_id_str),
                pid: pid as u32,
                confidence,
                started_at: OffsetDateTime::from_unix_timestamp(started_at_ts)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                ended_at: ended_at_ts.and_then(|ts| {
                    OffsetDateTime::from_unix_timestamp(ts).ok()
                }),
            })
        })
        .map_err(|e| CcaError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(sessions)
}

pub fn get_session(pool: &DbPool, id: Uuid) -> auditor_core::error::Result<Option<AuditSession>> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let mut stmt = conn
        .prepare("SELECT id, tool_id, pid, started_at, ended_at, confidence FROM sessions WHERE id = ?1")
        .map_err(|e| CcaError::Database(e.to_string()))?;

    let session = stmt
        .query_row(rusqlite::params![id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let tool_id_str: String = row.get(1)?;
            let pid: i32 = row.get(2)?;
            let started_at_ts: i64 = row.get(3)?;
            let ended_at_ts: Option<i64> = row.get(4)?;
            let confidence_str: String = row.get(5)?;

            let confidence = match confidence_str.as_str() {
                "High" => Confidence::High,
                "Ambiguous" => Confidence::Ambiguous,
                "Verified" => Confidence::Verified,
                _ => Confidence::Ambiguous,
            };

            Ok(AuditSession {
                id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
                tool_id: ToolId::new(tool_id_str),
                pid: pid as u32,
                confidence,
                started_at: OffsetDateTime::from_unix_timestamp(started_at_ts)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                ended_at: ended_at_ts.and_then(|ts| {
                    OffsetDateTime::from_unix_timestamp(ts).ok()
                }),
            })
        })
        .optional()
        .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(session)
}
