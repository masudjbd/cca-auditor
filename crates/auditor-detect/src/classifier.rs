use auditor_core::config::ToolFingerprint;
use auditor_core::tool::ToolId;
use sysinfo::Process;

pub fn classify(proc: &Process, fingerprints: &[ToolFingerprint]) -> Option<ToolId> {
    let exe_name = proc.name();
    let exe_path = proc.exe().map(|p| p.to_string_lossy().to_string());
    let cmdline = proc.cmd().join(" ");

    for fp in fingerprints {
        // Match exe_name
        if fp.exe_name.iter().any(|n| n == exe_name) {
            return Some(ToolId::new(fp.id.clone()));
        }

        // Match exe_path_contains
        if let Some(ref path) = exe_path {
            if fp.exe_path_contains
                .iter()
                .any(|pattern| path.contains(pattern))
            {
                return Some(ToolId::new(fp.id.clone()));
            }
        }

        // Match cmd_contains
        if fp.cmd_contains.iter().any(|pattern| cmdline.contains(pattern)) {
            return Some(ToolId::new(fp.id.clone()));
        }
    }

    None
}

pub fn load_fingerprints(toml_path: &str) -> anyhow::Result<Vec<ToolFingerprint>> {
    let content = std::fs::read_to_string(toml_path)?;
    let table: toml::Table = toml::from_str(&content)?;

    let mut fingerprints = Vec::new();

    if let Some(tools) = table.get("tool").and_then(|v| v.as_array()) {
        for tool_table in tools {
            if let Some(tool) = tool_table.as_table() {
                let id = tool
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let kind = tool
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let display_name = tool
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let exe_name = tool
                    .get("exe_name")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                let exe_path_contains = tool
                    .get("exe_path_contains")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                let cmd_contains = tool
                    .get("cmd_contains")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                let include_descendants = tool
                    .get("include_descendants")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                fingerprints.push(ToolFingerprint {
                    id,
                    kind,
                    display_name,
                    exe_name,
                    exe_path_contains,
                    cmd_contains,
                    include_descendants,
                });
            }
        }
    }

    Ok(fingerprints)
}
