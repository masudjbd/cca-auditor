use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub db_path: String,
    pub watch_paths: Vec<String>,
    pub tools: Vec<ToolFingerprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFingerprint {
    pub id: String,
    pub kind: String,
    pub display_name: String,
    pub exe_name: Vec<String>,
    pub exe_path_contains: Vec<String>,
    pub cmd_contains: Vec<String>,
    pub include_descendants: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivePath {
    pub pattern: String,
    pub severity: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub config: AppConfig,
    pub sensitive_paths: Vec<SensitivePath>,
}
