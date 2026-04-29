use auditor_core::session::AuditSession;
use auditor_core::events::AuditEvent;
use auditor_core::samples::ResourceSample;
use auditor_db::DbPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub fn get_live_sessions(
    state: tauri::State<'_, Arc<DbPool>>,
) -> Result<Vec<AuditSession>, String> {
    auditor_db::queries::sessions::get_sessions(&state, 100)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_events(
    state: tauri::State<'_, Arc<DbPool>>,
    session_id: String,
    limit: u32,
) -> Result<Vec<AuditEvent>, String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("invalid session_id: {}", e))?;
    auditor_db::queries::events::get_events(&state, uuid, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_samples(
    state: tauri::State<'_, Arc<DbPool>>,
    pid: u32,
    from: i64,
    to: i64,
) -> Result<Vec<ResourceSample>, String> {
    auditor_db::queries::samples::get_samples(&state, pid, from, to)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: i64,
    pub kind: String,
    pub severity: String,
    pub detail: String,
    pub timestamp: i64,
    pub dismissed: bool,
}

#[tauri::command]
pub fn get_alerts(
    _state: tauri::State<'_, Arc<DbPool>>,
    _dismissed: bool,
) -> Result<Vec<Alert>, String> {
    // TODO: implement alert query when alerts query module is added
    Ok(vec![])
}

#[tauri::command]
pub fn dismiss_alert(_id: i64) -> Result<(), String> {
    // TODO: implement when alerts query module is added
    Ok(())
}

#[tauri::command]
pub fn generate_report(
    _state: tauri::State<'_, Arc<DbPool>>,
    session_ids: Vec<String>,
    format: String,
) -> Result<String, String> {
    // TODO: full implementation. For now, return JSON summary.
    let report = serde_json::json!({
        "session_ids": session_ids,
        "format": format,
        "generated_at": time::OffsetDateTime::now_utc().to_string(),
        "note": "Report generation will be fully implemented in next iteration"
    });
    Ok(report.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailFinding {
    pub rule_id: String,
    pub file: String,
    pub line: i32,
    pub severity: String,
    pub redacted_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailResult {
    pub allowed: bool,
    pub findings: Option<Vec<GuardrailFinding>>,
}

#[tauri::command]
pub async fn push_with_guardrail(
    remote: String,
    _refspec: String,
) -> Result<GuardrailResult, String> {
    // Org allowlist check
    let allowed_orgs = ["masudjbd", "fahiminfo"];
    let url_lower = remote.to_lowercase();
    let in_allowlist = allowed_orgs.iter().any(|org| url_lower.contains(org));

    if !in_allowlist {
        return Err(format!(
            "Remote '{}' not in allowlist. Only masudjbd/* and fahiminfo/* permitted.",
            remote
        ));
    }

    // TODO: actual gitleaks scan + push
    Ok(GuardrailResult {
        allowed: true,
        findings: None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub watch_paths: Vec<String>,
    pub enabled_tools: Vec<String>,
    pub encryption: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            watch_paths: vec![],
            enabled_tools: vec![
                "cursor".to_string(),
                "claude-code".to_string(),
                "windsurf".to_string(),
                "ollama".to_string(),
                "lmstudio".to_string(),
                "aider".to_string(),
                "cline".to_string(),
                "continue".to_string(),
            ],
            encryption: false,
        }
    }
}

fn settings_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".cca-audit").join("settings.json"))
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = settings_path().ok_or("could not determine home directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    tracing::info!("saved settings to {:?}", path);
    Ok(())
}

#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    let path = settings_path().ok_or("could not determine home directory")?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}
