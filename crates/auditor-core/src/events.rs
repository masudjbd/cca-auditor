use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::tool::{Confidence, ToolId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Auto-increment integer assigned by SQLite. 0 for unsaved events.
    pub id: i64,
    pub session_id: Uuid,
    pub tool_id: ToolId,
    pub kind: EventKind,
    pub confidence: Confidence,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    FsRead { path: String },
    FsWrite { path: String },
    FsDelete { path: String },
    NetConnect { addr: String, port: u16, proto: String },
    ProcessSpawn { exe: String, args: Vec<String> },
    LocalArtifact { path: String, artifact_type: String },
}

impl AuditEvent {
    pub fn new(
        session_id: Uuid,
        tool_id: ToolId,
        kind: EventKind,
        confidence: Confidence,
    ) -> Self {
        Self {
            id: 0,
            session_id,
            tool_id,
            kind,
            confidence,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}
