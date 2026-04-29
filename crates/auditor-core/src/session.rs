use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::tool::{Confidence, ToolId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSession {
    pub id: Uuid,
    pub tool_id: ToolId,
    pub pid: u32,
    pub confidence: Confidence,
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub ended_at: Option<OffsetDateTime>,
}

impl AuditSession {
    pub fn new(tool_id: ToolId, pid: u32, confidence: Confidence) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_id,
            pid,
            confidence,
            started_at: OffsetDateTime::now_utc(),
            ended_at: None,
        }
    }

    pub fn close(&mut self) {
        self.ended_at = Some(OffsetDateTime::now_utc());
    }
}
