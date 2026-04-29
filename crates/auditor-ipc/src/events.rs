use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum AuditMessage {
    SessionOpened { tool_id: String, pid: u32 },
    SessionClosed { tool_id: String, pid: u32 },
    ResourceSample { pid: u32, cpu_pct: f64, rss_bytes: u64 },
    AuditEvent { session_id: String, kind: String, details: String },
    AlertRaised { severity: String, message: String },
}

pub fn emit_event(_event: AuditMessage) -> Result<(), String> {
    // TODO: emit via tauri app handle
    Ok(())
}
