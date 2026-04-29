use crate::DbPool;
use auditor_core::error::CcaError;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: i64,
    pub kind: String,
    pub severity: String,
    pub detail: String,
    pub timestamp: i64,
    pub dismissed: bool,
}

pub fn insert_alert(
    pool: &DbPool,
    kind: &str,
    severity: &str,
    detail: &str,
) -> auditor_core::error::Result<i64> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let now = OffsetDateTime::now_utc().unix_timestamp();

    conn.execute(
        "INSERT INTO alerts (kind, severity, detail_json, ts, dismissed) VALUES (?1, ?2, ?3, ?4, 0)",
        rusqlite::params![kind, severity, detail, now],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(conn.last_insert_rowid())
}

pub fn get_alerts(
    pool: &DbPool,
    include_dismissed: bool,
) -> auditor_core::error::Result<Vec<Alert>> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;

    let sql = if include_dismissed {
        "SELECT id, kind, severity, detail_json, ts, dismissed FROM alerts ORDER BY ts DESC LIMIT 200"
    } else {
        "SELECT id, kind, severity, detail_json, ts, dismissed FROM alerts WHERE dismissed = 0 ORDER BY ts DESC LIMIT 200"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| CcaError::Database(e.to_string()))?;

    let alerts = stmt
        .query_map([], |row| {
            Ok(Alert {
                id: row.get(0)?,
                kind: row.get(1)?,
                severity: row.get(2)?,
                detail: row.get(3)?,
                timestamp: row.get(4)?,
                dismissed: row.get::<_, i32>(5)? != 0,
            })
        })
        .map_err(|e| CcaError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(alerts)
}

pub fn dismiss_alert(pool: &DbPool, id: i64) -> auditor_core::error::Result<()> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    conn.execute(
        "UPDATE alerts SET dismissed = 1 WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;
    Ok(())
}
