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
    let raw = auditor_db::queries::samples::get_samples(&state, pid, from, to)
        .map_err(|e| e.to_string())?;

    // Downsample to ~200 points for chart rendering performance
    Ok(auditor_db::downsample::lttb_downsample(raw, 200))
}

pub use auditor_db::queries::alerts::Alert;

#[tauri::command]
pub fn get_alerts(
    state: tauri::State<'_, Arc<DbPool>>,
    dismissed: bool,
) -> Result<Vec<Alert>, String> {
    auditor_db::queries::alerts::get_alerts(&state, dismissed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dismiss_alert(
    state: tauri::State<'_, Arc<DbPool>>,
    id: i64,
) -> Result<(), String> {
    auditor_db::queries::alerts::dismiss_alert(&state, id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_report(
    state: tauri::State<'_, Arc<DbPool>>,
    session_ids: Vec<String>,
    format: String,
) -> Result<String, String> {
    let fmt = auditor_report::ReportFormat::from_str(&format)
        .ok_or_else(|| format!("invalid format: {}", format))?;

    let uuids: Result<Vec<Uuid>, _> = session_ids
        .iter()
        .map(|s| Uuid::parse_str(s))
        .collect();
    let uuids = uuids.map_err(|e| format!("invalid session_id: {}", e))?;

    let bytes = auditor_report::generate(state.inner().clone(), uuids, fmt)
        .await
        .map_err(|e| e.to_string())?;

    String::from_utf8(bytes).map_err(|e| format!("utf8 error: {}", e))
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

    // Scan staged changes in current working dir
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let scan_result = auditor_guardrail::scan_staged(&cwd).await;

    let findings = match scan_result {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("guardrail scan failed: {}", e);
            // If we can't scan (e.g., not a git repo), allow with warning
            return Ok(GuardrailResult {
                allowed: true,
                findings: None,
            });
        }
    };

    if findings.is_empty() {
        return Ok(GuardrailResult {
            allowed: true,
            findings: None,
        });
    }

    let has_high = findings.iter().any(|f| f.severity == "high");

    let mapped: Vec<GuardrailFinding> = findings
        .into_iter()
        .map(|f| GuardrailFinding {
            rule_id: f.rule_id,
            file: f.file,
            line: f.line as i32,
            severity: f.severity,
            redacted_value: f.secret_value,
        })
        .collect();

    Ok(GuardrailResult {
        allowed: !has_high,
        findings: Some(mapped),
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

/// Save settings to disk. Caller (main.rs) should follow up with reload signal
/// to FS watcher if watch_paths changed.
pub fn save_settings_impl(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path().ok_or("could not determine home directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    tracing::info!("saved settings to {:?}", path);
    Ok(())
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    save_settings_impl(&settings)
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
