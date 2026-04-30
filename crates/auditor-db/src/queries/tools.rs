use crate::DbPool;
use auditor_core::config::ToolFingerprint;
use auditor_core::error::CcaError;

pub fn seed_tools(pool: &DbPool, fingerprints: &[ToolFingerprint]) -> auditor_core::error::Result<()> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;

    for fp in fingerprints {
        let config_json = serde_json::to_string(fp)
            .unwrap_or_else(|_| "{}".to_string());
        conn.execute(
            "INSERT OR IGNORE INTO tools (id, kind, display_name, config_json) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                fp.id,
                fp.kind,
                fp.display_name,
                config_json,
            ],
        )
        .map_err(|e| CcaError::Database(e.to_string()))?;
    }

    // Also seed an "unknown" tool for ambiguous classifications
    conn.execute(
        "INSERT OR IGNORE INTO tools (id, kind, display_name, config_json) VALUES ('unknown', 'unknown', 'Unknown', NULL)",
        [],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(())
}
