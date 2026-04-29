use auditor_core::session::AuditSession;
use auditor_core::events::AuditEvent;
use auditor_core::samples::ResourceSample;
use auditor_db::DbPool;
use std::sync::Arc;

#[tauri::command]
pub fn get_live_sessions() -> Result<Vec<AuditSession>, String> {
    // TODO: get db pool from state
    Ok(vec![])
}

#[tauri::command]
pub fn get_events(
    _session_id: String,
    _limit: u32,
) -> Result<Vec<AuditEvent>, String> {
    // TODO: get db pool from state
    Ok(vec![])
}

#[tauri::command]
pub fn get_samples(
    _pid: u32,
    _from: i64,
    _to: i64,
) -> Result<Vec<ResourceSample>, String> {
    // TODO: get db pool from state
    Ok(vec![])
}

#[tauri::command]
pub async fn get_alerts(_state: tauri::State<'_, Arc<DbPool>>) -> Result<Vec<String>, String> {
    // TODO: implement alert query
    Ok(vec![])
}

#[tauri::command]
pub async fn dismiss_alert(_id: i64) -> Result<(), String> {
    // TODO: implement alert dismissal
    Ok(())
}

#[tauri::command]
pub fn generate_report(
    _session_ids: Vec<String>,
    _format: String,
) -> Result<Vec<u8>, String> {
    // TODO: get db pool from state, generate report
    Ok(b"<html>Report placeholder</html>".to_vec())
}

#[tauri::command]
pub async fn push_with_guardrail(
    _remote: String,
    _refspec: String,
) -> Result<String, String> {
    // TODO: implement guardrail + push
    Ok("push allowed".to_string())
}
