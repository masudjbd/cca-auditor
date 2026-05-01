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
pub use auditor_db::queries::stats::DbStats;

#[tauri::command]
pub fn get_db_stats(
    state: tauri::State<'_, Arc<DbPool>>,
) -> Result<DbStats, String> {
    auditor_db::queries::stats::get_db_stats(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn purge_all_data(
    state: tauri::State<'_, Arc<DbPool>>,
) -> Result<(), String> {
    auditor_db::queries::stats::purge_all_data(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_report_to_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content)
        .map_err(|e| format!("failed to write {}: {}", path, e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSensitivePath {
    pub pattern: String,
    pub severity: String,
    pub reason: String,
}

fn user_sensitive_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".cca-audit").join("sensitive-paths.json"))
}

#[tauri::command]
pub fn get_user_sensitive_paths() -> Result<Vec<UserSensitivePath>, String> {
    let path = match user_sensitive_path() {
        Some(p) => p,
        None => return Ok(vec![]),
    };
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_user_sensitive_paths(
    paths: Vec<UserSensitivePath>,
) -> Result<(), String> {
    let path = user_sensitive_path().ok_or("could not determine home directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&paths).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    tracing::info!("saved {} user sensitive path patterns", paths.len());
    Ok(())
}

#[tauri::command]
pub fn open_path_in_finder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

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

/// Phase 1: scan only. Returns findings without pushing.
/// Frontend calls this first, shows results, then calls execute_push if user approves.
#[tauri::command]
pub async fn push_with_guardrail(
    repo_path: Option<String>,
    remote: String,
    _refspec: String,
) -> Result<GuardrailResult, String> {
    let cwd = match repo_path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    // Resolve remote URL via git2 to get the actual URL (not just the alias)
    let remote_url = auditor_guardrail::push_guard::get_remote_url(&cwd, &remote)
        .unwrap_or_else(|_| remote.clone());

    if !auditor_guardrail::push_guard::check_org_allowlist(&remote_url) {
        return Err(format!(
            "Remote '{}' not in allowlist. Only masudjbd/* and fahiminfo/* permitted.",
            remote_url
        ));
    }

    // Scan staged changes
    let findings = auditor_guardrail::scan_staged(&cwd)
        .await
        .map_err(|e| format!("scan failed: {}. Run from a git repo.", e))?;

    let mapped: Vec<GuardrailFinding> = findings
        .iter()
        .map(|f| GuardrailFinding {
            rule_id: f.rule_id.clone(),
            file: f.file.clone(),
            line: f.line as i32,
            severity: f.severity.clone(),
            redacted_value: f.secret_value.clone(),
        })
        .collect();

    let has_high = findings.iter().any(|f| f.severity == "high");

    Ok(GuardrailResult {
        allowed: !has_high,
        findings: if mapped.is_empty() { None } else { Some(mapped) },
    })
}

/// Phase 2: actually execute push. Called after user approves (or if no findings).
#[tauri::command]
pub async fn execute_push(
    repo_path: Option<String>,
    remote: String,
    refspec: String,
) -> Result<(), String> {
    let cwd = match repo_path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    // Re-validate allowlist (defense in depth)
    let remote_url = auditor_guardrail::push_guard::get_remote_url(&cwd, &remote)
        .map_err(|e| e.to_string())?;
    if !auditor_guardrail::push_guard::check_org_allowlist(&remote_url) {
        return Err(format!("Remote '{}' not in allowlist.", remote_url));
    }

    auditor_guardrail::push_guard::execute_push(&cwd, &remote, &refspec)
        .map_err(|e| format!("push failed: {}", e))?;

    tracing::info!("push succeeded: {} {}", remote, refspec);
    Ok(())
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
