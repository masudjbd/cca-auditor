use crate::DbPool;
use auditor_core::error::CcaError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbStats {
    pub total_sessions: i64,
    pub active_sessions: i64,
    pub total_events: i64,
    pub total_samples: i64,
    pub total_alerts: i64,
    pub undismissed_alerts: i64,
    pub db_size_bytes: i64,
    pub oldest_event_ts: Option<i64>,
    pub newest_event_ts: Option<i64>,
    pub events_by_kind: Vec<(String, i64)>,
}

pub fn get_db_stats(pool: &DbPool) -> auditor_core::error::Result<DbStats> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;

    let total_sessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap_or(0);

    let active_sessions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE ended_at IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let total_events: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
        .unwrap_or(0);

    let total_samples: i64 = conn
        .query_row("SELECT COUNT(*) FROM samples", [], |r| r.get(0))
        .unwrap_or(0);

    let total_alerts: i64 = conn
        .query_row("SELECT COUNT(*) FROM alerts", [], |r| r.get(0))
        .unwrap_or(0);

    let undismissed_alerts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM alerts WHERE dismissed = 0",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let db_size_bytes: i64 = conn
        .query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let oldest_event_ts: Option<i64> = conn
        .query_row("SELECT MIN(ts) FROM events", [], |r| r.get(0))
        .ok();

    let newest_event_ts: Option<i64> = conn
        .query_row("SELECT MAX(ts) FROM events", [], |r| r.get(0))
        .ok();

    // Events by kind
    let mut stmt = conn
        .prepare("SELECT kind, COUNT(*) FROM events GROUP BY kind ORDER BY COUNT(*) DESC")
        .map_err(|e| CcaError::Database(e.to_string()))?;
    let events_by_kind = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map_err(|e| CcaError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(DbStats {
        total_sessions,
        active_sessions,
        total_events,
        total_samples,
        total_alerts,
        undismissed_alerts,
        db_size_bytes,
        oldest_event_ts,
        newest_event_ts,
        events_by_kind,
    })
}

/// Wipe all audit data (preserves schema).
pub fn purge_all_data(pool: &DbPool) -> auditor_core::error::Result<()> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    conn.execute_batch(
        "DELETE FROM events;
         DELETE FROM samples;
         DELETE FROM samples_10s;
         DELETE FROM samples_1m;
         DELETE FROM alerts;
         DELETE FROM sessions;
         DELETE FROM processes;
         DELETE FROM reports;
         VACUUM;",
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;
    Ok(())
}
