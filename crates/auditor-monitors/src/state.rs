use auditor_core::session::AuditSession;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ActiveSessions = Arc<RwLock<HashMap<u32, AuditSession>>>;

pub fn create_state() -> ActiveSessions {
    Arc::new(RwLock::new(HashMap::new()))
}
