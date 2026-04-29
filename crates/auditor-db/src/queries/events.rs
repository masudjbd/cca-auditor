use auditor_core::events::AuditEvent;
use uuid::Uuid;

pub fn insert_event(_event: &AuditEvent) -> crate::error::Result<()> {
    Ok(())
}

pub fn get_events(_session_id: Uuid, _limit: u32) -> crate::error::Result<Vec<AuditEvent>> {
    Ok(vec![])
}
