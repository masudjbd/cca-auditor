use anyhow::Result;
use git2::{DiffOptions, Repository};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub rule_name: String,
    pub file: String,
    pub line: u32,
    pub secret_value: String, // redacted
    pub severity: String,
}

pub fn redact_secret(secret: &str) -> String {
    if secret.len() <= 8 {
        return "*".repeat(secret.len());
    }
    let first_4 = &secret[..4];
    let last_4 = &secret[secret.len() - 4..];
    format!("{}{}{}", first_4, "*".repeat(secret.len() - 8), last_4)
}

struct Pattern {
    rule_id: &'static str,
    rule_name: &'static str,
    severity: &'static str,
    regex: Regex,
}

static PATTERNS: Lazy<Vec<Pattern>> = Lazy::new(|| {
    vec![
        Pattern {
            rule_id: "github-pat",
            rule_name: "GitHub Personal Access Token",
            severity: "high",
            regex: Regex::new(r"\bghp_[A-Za-z0-9]{36}\b").unwrap(),
        },
        Pattern {
            rule_id: "github-fine-grained-pat",
            rule_name: "GitHub Fine-grained PAT",
            severity: "high",
            regex: Regex::new(r"\bgithub_pat_[A-Za-z0-9_]{82}\b").unwrap(),
        },
        Pattern {
            rule_id: "github-oauth",
            rule_name: "GitHub OAuth Token",
            severity: "high",
            regex: Regex::new(r"\bgho_[A-Za-z0-9]{36}\b").unwrap(),
        },
        Pattern {
            rule_id: "anthropic-api-key",
            rule_name: "Anthropic API Key",
            severity: "high",
            regex: Regex::new(r"\bsk-ant-[A-Za-z0-9_\-]{20,}\b").unwrap(),
        },
        Pattern {
            rule_id: "openai-api-key",
            rule_name: "OpenAI API Key",
            severity: "high",
            regex: Regex::new(r"\bsk-[A-Za-z0-9]{32,}\b").unwrap(),
        },
        Pattern {
            rule_id: "aws-access-key",
            rule_name: "AWS Access Key ID",
            severity: "high",
            regex: Regex::new(r"\b(?:AKIA|ASIA|AROA|AIPA|ANPA|ANVA|ABIA|ACCA)[0-9A-Z]{16}\b")
                .unwrap(),
        },
        Pattern {
            rule_id: "aws-secret-key",
            rule_name: "AWS Secret Access Key (heuristic)",
            severity: "high",
            regex: Regex::new(
                r#"(?i)aws.{0,20}?(?:secret|key).{0,5}?[=:]\s*['""]?([A-Za-z0-9/+=]{40})"#,
            )
            .unwrap(),
        },
        Pattern {
            rule_id: "google-api-key",
            rule_name: "Google API Key",
            severity: "high",
            regex: Regex::new(r"\bAIza[0-9A-Za-z_\-]{35}\b").unwrap(),
        },
        Pattern {
            rule_id: "stripe-secret",
            rule_name: "Stripe Secret Key",
            severity: "high",
            regex: Regex::new(r"\b(?:sk|rk)_(?:live|test)_[A-Za-z0-9]{24,}\b").unwrap(),
        },
        Pattern {
            rule_id: "slack-token",
            rule_name: "Slack Token",
            severity: "high",
            regex: Regex::new(r"\bxox[baprs]-[A-Za-z0-9\-]{10,}\b").unwrap(),
        },
        Pattern {
            rule_id: "private-key-pem",
            rule_name: "Private Key (PEM block)",
            severity: "high",
            regex: Regex::new(r"-----BEGIN (?:RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY-----")
                .unwrap(),
        },
        Pattern {
            rule_id: "jwt",
            rule_name: "JSON Web Token",
            severity: "medium",
            regex: Regex::new(r"\beyJ[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]+\b")
                .unwrap(),
        },
        Pattern {
            rule_id: "generic-api-key",
            rule_name: "Generic API key (heuristic)",
            severity: "medium",
            regex: Regex::new(
                r#"(?i)(?:api[_\-]?key|apikey|access[_\-]?token).{0,5}?[=:]\s*['"]([A-Za-z0-9_\-]{20,})['"]"#,
            )
            .unwrap(),
        },
        Pattern {
            rule_id: "generic-password",
            rule_name: "Generic password (heuristic)",
            severity: "low",
            regex: Regex::new(
                r#"(?i)(?:password|passwd|pwd).{0,5}?[=:]\s*['"]([^\s'"]{8,})['"]"#,
            )
            .unwrap(),
        },
    ]
});

fn scan_text(file: &str, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        for pattern in PATTERNS.iter() {
            if let Some(m) = pattern.regex.find(line) {
                findings.push(Finding {
                    rule_id: pattern.rule_id.to_string(),
                    rule_name: pattern.rule_name.to_string(),
                    file: file.to_string(),
                    line: (line_no + 1) as u32,
                    secret_value: redact_secret(m.as_str()),
                    severity: pattern.severity.to_string(),
                });
            }
        }
    }
    findings
}

pub async fn scan_staged(repo_path: impl AsRef<Path>) -> Result<Vec<Finding>> {
    let repo_path = repo_path.as_ref();
    let repo = Repository::open(repo_path)?;
    let head = repo.head().ok().and_then(|h| h.peel_to_tree().ok());

    let index = repo.index()?;
    let mut diff_opts = DiffOptions::new();
    let diff = repo.diff_tree_to_index(head.as_ref(), Some(&index), Some(&mut diff_opts))?;

    let mut findings = Vec::new();

    diff.foreach(
        &mut |_delta, _progress| true,
        None,
        None,
        Some(&mut |delta, _hunk, line| {
            // Only scan lines being added in the diff
            if line.origin() != '+' {
                return true;
            }
            let path = delta
                .new_file()
                .path()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            if let Ok(content_str) = std::str::from_utf8(line.content()) {
                let line_findings = scan_text(&path, content_str);
                findings.extend(line_findings);
            }
            true
        }),
    )?;

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_short() {
        assert_eq!(redact_secret("short"), "*****");
    }

    #[test]
    fn redact_long() {
        assert_eq!(redact_secret("ghp_1234567890abcdef"), "ghp_************cdef");
    }

    #[test]
    fn detects_github_pat() {
        // GitHub PAT has exactly 36 chars after ghp_
        let findings = scan_text(".env", "GITHUB_TOKEN=ghp_123456789012345678901234567890123456");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "github-pat");
    }

    #[test]
    fn detects_anthropic_key() {
        let findings = scan_text(
            ".env",
            "ANTHROPIC_API_KEY=sk-ant-api03-abc123def456ghi789",
        );
        assert!(findings.iter().any(|f| f.rule_id == "anthropic-api-key"));
    }
}
