use auditor_db::DbPool;
use notify::{Config, Event, EventKind as NotifyEventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

/// Notification sent to the supervisor when an alert is fired.
/// Tuple: (alert_id, kind, severity, detail_json)
pub type AlertNotification = (i64, String, String, String);
pub type AlertSender = mpsc::UnboundedSender<AlertNotification>;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SensitivePathConfig {
    pub pattern: String,
    pub severity: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SensitivePathsFile {
    #[serde(rename = "path")]
    pub paths: Vec<SensitivePathConfig>,
}

pub fn load_sensitive_paths(toml_path: &Path) -> Result<Vec<SensitivePathConfig>> {
    let content = std::fs::read_to_string(toml_path)?;
    let parsed: SensitivePathsFile = toml::from_str(&content)?;
    Ok(parsed.paths)
}

pub async fn start_watcher(
    db: Arc<DbPool>,
    watch_paths: Arc<tokio::sync::RwLock<Vec<PathBuf>>>,
    sensitive_patterns: Vec<SensitivePathConfig>,
    alert_sender: Option<AlertSender>,
    reload_signal: Arc<tokio::sync::Notify>,
    shutdown: CancellationToken,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let mut current_watcher = build_watcher(&watch_paths, &tx).await?;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("FS watcher shutting down");
                break;
            }
            _ = reload_signal.notified() => {
                tracing::info!("FS watcher: reload signal received, rebuilding");
                drop(current_watcher);
                current_watcher = build_watcher(&watch_paths, &tx).await?;
            }
            Some(event) = rx.recv() => {
                handle_event(&db, &event, &sensitive_patterns, alert_sender.as_ref()).await;
            }
        }
    }

    Ok(())
}

async fn build_watcher(
    watch_paths: &Arc<tokio::sync::RwLock<Vec<PathBuf>>>,
    tx: &mpsc::UnboundedSender<Event>,
) -> Result<RecommendedWatcher> {
    let watcher_tx = tx.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = watcher_tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;

    let paths = watch_paths.read().await;
    let mut watched_count = 0;
    for path in paths.iter() {
        if path.exists() {
            match watcher.watch(path, RecursiveMode::Recursive) {
                Ok(()) => {
                    tracing::info!("watching {:?}", path);
                    watched_count += 1;
                }
                Err(e) => {
                    tracing::warn!("failed to watch {:?}: {}", path, e);
                }
            }
        } else {
            tracing::debug!("watch path does not exist: {:?}", path);
        }
    }

    if watched_count == 0 && !paths.is_empty() {
        tracing::warn!("no paths could be watched (configured: {})", paths.len());
    } else if watched_count > 0 {
        tracing::info!("FS watcher active on {} path(s)", watched_count);
    }

    Ok(watcher)
}

async fn handle_event(
    db: &Arc<DbPool>,
    event: &Event,
    sensitive_patterns: &[SensitivePathConfig],
    alert_sender: Option<&AlertSender>,
) {
    // Only process write/create/modify events
    let is_write = matches!(
        event.kind,
        NotifyEventKind::Modify(_) | NotifyEventKind::Create(_)
    );

    if !is_write {
        return;
    }

    for path in &event.paths {
        // Skip noisy paths
        let path_str = path.to_string_lossy();
        if path_str.contains("/.git/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/target/")
            || path_str.contains("/.DS_Store")
        {
            continue;
        }

        // Check against sensitive patterns
        if let Some(matched) = match_sensitive(path, sensitive_patterns) {
            let detail = format!(
                "{{\"path\":\"{}\",\"reason\":\"{}\"}}",
                path_str.replace('"', "\\\""),
                matched.reason.replace('"', "\\\"")
            );

            match auditor_db::queries::alerts::insert_alert(
                db,
                "sensitive_path_access",
                &matched.severity,
                &detail,
            ) {
                Err(e) => tracing::warn!("failed to insert alert: {}", e),
                Ok(alert_id) => {
                    tracing::warn!(
                        "[ALERT] sensitive path accessed: {} (reason: {})",
                        path_str,
                        matched.reason
                    );
                    if let Some(sender) = alert_sender {
                        let _ = sender.send((
                            alert_id,
                            "sensitive_path_access".to_string(),
                            matched.severity.clone(),
                            detail,
                        ));
                    }
                }
            }
        }
    }
}

fn match_sensitive<'a>(
    path: &Path,
    patterns: &'a [SensitivePathConfig],
) -> Option<&'a SensitivePathConfig> {
    let path_str = path.to_string_lossy();
    let home = dirs::home_dir().map(|h| h.to_string_lossy().to_string()).unwrap_or_default();

    for pattern_cfg in patterns {
        let mut pattern = pattern_cfg.pattern.clone();
        // Expand ~/ to home directory
        if pattern.starts_with("~/") {
            pattern = pattern.replacen("~", &home, 1);
        }

        // Convert glob to simple substring match for ** and prefix match for /*
        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            if path_str.starts_with(prefix) {
                return Some(pattern_cfg);
            }
        } else if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            if path_str.ends_with(suffix) {
                return Some(pattern_cfg);
            }
        } else if path_str.contains(&pattern) {
            return Some(pattern_cfg);
        }
    }

    None
}
