use anyhow::{anyhow, Result};
use git2::{
    Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository,
};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GuardrailResult {
    pub allowed: bool,
    pub findings_count: usize,
    pub high_severity: usize,
    pub remote_url: Option<String>,
}

pub fn check_org_allowlist(remote_url: &str) -> bool {
    let allowed_orgs = ["masudjbd", "fahiminfo"];
    allowed_orgs
        .iter()
        .any(|org| remote_url.contains(&format!("github.com/{}", org))
            || remote_url.contains(&format!("github.com:{}", org)))
}

/// Get the remote URL for a configured remote name (e.g., "origin").
pub fn get_remote_url(repo_path: impl AsRef<Path>, remote: &str) -> Result<String> {
    let repo = Repository::open(repo_path)?;
    let remote_obj = repo.find_remote(remote)?;
    Ok(remote_obj.url().unwrap_or("unknown").to_string())
}

/// Actually execute a git push using libgit2. SSH auth via ssh-agent;
/// HTTPS auth via stored credentials (system keychain / .git-credentials).
pub fn execute_push(
    repo_path: impl AsRef<Path>,
    remote: &str,
    refspec: &str,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let mut remote_obj = repo.find_remote(remote)?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|url, username_from_url, allowed_types| {
        // SSH (key-based)
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            let username = username_from_url.unwrap_or("git");
            return Cred::ssh_key_from_agent(username);
        }
        // HTTPS (default credentials helper)
        if allowed_types.contains(git2::CredentialType::DEFAULT) {
            return Cred::default();
        }
        // Username/password (last resort, will fail without input)
        Err(git2::Error::from_str(&format!(
            "no available credentials for {}",
            url
        )))
    });

    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    remote_obj
        .push(&[refspec], Some(&mut push_opts))
        .map_err(|e| anyhow!("git push failed: {}", e))?;

    Ok(())
}

/// Fetch from a remote (non-mutating, used for status checks).
#[allow(dead_code)]
pub fn fetch_remote(repo_path: impl AsRef<Path>, remote: &str) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let mut remote_obj = repo.find_remote(remote)?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            let username = username_from_url.unwrap_or("git");
            return Cred::ssh_key_from_agent(username);
        }
        Cred::default()
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    remote_obj.fetch::<&str>(&[], Some(&mut fetch_opts), None)?;
    Ok(())
}

/// Full guarded push flow:
/// 1. Validate remote URL against org allowlist
/// 2. Scan staged diff for secrets
/// 3. If high-severity: block; otherwise return findings (caller decides override)
/// 4. If approved: execute push
pub async fn guard_push(
    repo_path: impl AsRef<Path>,
    remote: &str,
    refspec: &str,
) -> Result<(GuardrailResult, Vec<crate::scanner::Finding>)> {
    let repo_path = repo_path.as_ref();
    let remote_url = get_remote_url(repo_path, remote)?;

    if !check_org_allowlist(&remote_url) {
        return Ok((
            GuardrailResult {
                allowed: false,
                findings_count: 0,
                high_severity: 0,
                remote_url: Some(remote_url),
            },
            vec![],
        ));
    }

    let findings = crate::scanner::scan_staged(repo_path).await?;
    let high_severity = findings.iter().filter(|f| f.severity == "high").count();

    let result = GuardrailResult {
        allowed: high_severity == 0,
        findings_count: findings.len(),
        high_severity,
        remote_url: Some(remote_url),
    };

    // If clean, push immediately. Caller (IPC layer) is responsible for
    // calling execute_push() separately if user overrides medium/low findings.
    if result.allowed && findings.is_empty() {
        execute_push(repo_path, remote, refspec)?;
    }

    Ok((result, findings))
}
