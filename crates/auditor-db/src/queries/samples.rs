use auditor_core::samples::ResourceSample;
use auditor_core::error::CcaError;
use crate::DbPool;
use time::OffsetDateTime;

/// Roll up 1Hz samples → 10s averages. Should be called periodically (e.g., every 10s).
/// Aggregates samples older than 60s to avoid touching live data.
pub fn rollup_samples_10s(pool: &DbPool) -> auditor_core::error::Result<u64> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let cutoff = now - 60; // Don't roll up data newer than 60s

    // Aggregate per (pid, 10s bucket) where bucket = ts - (ts % 10)
    let inserted = conn.execute(
        "INSERT OR REPLACE INTO samples_10s (pid, cpu_avg, rss_avg, ts)
         SELECT pid,
                AVG(cpu_pct),
                CAST(AVG(rss_bytes) AS INTEGER),
                ts - (ts % 10)
         FROM samples
         WHERE ts <= ?1
         GROUP BY pid, ts - (ts % 10)",
        rusqlite::params![cutoff],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(inserted as u64)
}

/// Roll up 10s averages → 1min averages.
pub fn rollup_samples_1m(pool: &DbPool) -> auditor_core::error::Result<u64> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let cutoff = now - 600; // 10 min lookback

    let inserted = conn.execute(
        "INSERT OR REPLACE INTO samples_1m (pid, cpu_avg, rss_avg, ts)
         SELECT pid,
                AVG(cpu_avg),
                CAST(AVG(rss_avg) AS INTEGER),
                ts - (ts % 60)
         FROM samples_10s
         WHERE ts <= ?1
         GROUP BY pid, ts - (ts % 60)",
        rusqlite::params![cutoff],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(inserted as u64)
}

/// Delete raw samples older than threshold (default: 24 hours).
pub fn purge_old_samples(pool: &DbPool, max_age_seconds: i64) -> auditor_core::error::Result<u64> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let cutoff = now - max_age_seconds;

    let deleted = conn.execute(
        "DELETE FROM samples WHERE ts < ?1",
        rusqlite::params![cutoff],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(deleted as u64)
}

/// Delete 10s rollups older than threshold (default: 30 days).
pub fn purge_old_rollups_10s(pool: &DbPool, max_age_seconds: i64) -> auditor_core::error::Result<u64> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let cutoff = now - max_age_seconds;

    let deleted = conn.execute(
        "DELETE FROM samples_10s WHERE ts < ?1",
        rusqlite::params![cutoff],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(deleted as u64)
}

pub fn insert_sample(pool: &DbPool, sample: &ResourceSample) -> auditor_core::error::Result<()> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;

    conn.execute(
        "INSERT INTO samples (pid, cpu_pct, rss_bytes, gpu_mem_bytes, ts)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            sample.pid as i32,
            sample.cpu_pct,
            sample.rss_bytes as i64,
            sample.gpu_mem_bytes.map(|b| b as i64),
            sample.timestamp.unix_timestamp(),
        ],
    )
    .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(())
}

pub fn get_samples(
    pool: &DbPool,
    pid: u32,
    from: i64,
    to: i64,
) -> auditor_core::error::Result<Vec<ResourceSample>> {
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let mut stmt = conn
        .prepare("SELECT pid, cpu_pct, rss_bytes, gpu_mem_bytes, ts FROM samples WHERE pid = ?1 AND ts >= ?2 AND ts <= ?3 ORDER BY ts")
        .map_err(|e| CcaError::Database(e.to_string()))?;

    let samples = stmt
        .query_map(rusqlite::params![pid as i32, from, to], |row| {
            let pid: i32 = row.get(0)?;
            let cpu_pct: f64 = row.get(1)?;
            let rss_bytes: i64 = row.get(2)?;
            let gpu_mem_bytes: Option<i64> = row.get(3)?;
            let ts: i64 = row.get(4)?;

            Ok(ResourceSample {
                pid: pid as u32,
                cpu_pct,
                rss_bytes: rss_bytes as u64,
                gpu_mem_bytes: gpu_mem_bytes.map(|b| b as u64),
                timestamp: OffsetDateTime::from_unix_timestamp(ts)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc()),
            })
        })
        .map_err(|e| CcaError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(samples)
}
