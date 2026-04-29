use auditor_core::session::AuditSession;
use uuid::Uuid;

pub fn insert_session(_session: &AuditSession) -> crate::error::Result<()> {
    Ok(())
}

pub fn get_sessions(_limit: u32) -> crate::error::Result<Vec<AuditSession>> {
    Ok(vec![])
}

pub fn get_session(_id: Uuid) -> crate::error::Result<Option<AuditSession>> {
    Ok(None)
}
