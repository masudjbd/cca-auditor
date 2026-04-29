use git2::Repository;
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GuardrailResult {
    pub allowed: bool,
    pub findings_count: usize,
    pub high_severity: usize,
}

pub async fn check_org_allowlist(remote_url: &str) -> Result<bool> {
    // Check if remote URL belongs to allowed GitHub orgs
    let allowed_orgs = vec!["masudjbd", "fahiminfo"];

    for org in allowed_orgs {
        if remote_url.contains(&format!("github.com/{}", org)) {
            return Ok(true);
        }
    }

    Ok(false)
}

pub async fn guard_push(
    repo_path: impl AsRef<Path>,
    remote: &str,
    _refspec: &str,
    _findings: Vec<crate::scanner::Finding>,
) -> Result<GuardrailResult> {
    let repo = Repository::open(repo_path)?;
    let remote_obj = repo.find_remote(remote)?;
    let remote_url = remote_obj.url().unwrap_or("unknown");

    // Check org allowlist
    if !check_org_allowlist(remote_url).await? {
        return Ok(GuardrailResult {
            allowed: false,
            findings_count: 0,
            high_severity: 0,
        });
    }

    // TODO: scan staged diffs with gitleaks
    // TODO: if high severity findings, block push
    // TODO: if medium/low, allow override

    Ok(GuardrailResult {
        allowed: true,
        findings_count: 0,
        high_severity: 0,
    })
}
