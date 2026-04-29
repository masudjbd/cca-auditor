use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSample {
    pub pid: u32,
    pub cpu_pct: f64,
    pub rss_bytes: u64,
    pub gpu_mem_bytes: Option<u64>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

impl ResourceSample {
    pub fn new(
        pid: u32,
        cpu_pct: f64,
        rss_bytes: u64,
        gpu_mem_bytes: Option<u64>,
    ) -> Self {
        Self {
            pid,
            cpu_pct,
            rss_bytes,
            gpu_mem_bytes,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}
