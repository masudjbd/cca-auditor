use auditor_core::samples::ResourceSample;
use auditor_core::error::CcaError;
use crate::DbPool;
use time::OffsetDateTime;

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
