# Security Model

## Design Principles

1. **Local-only**: No network egress except user-initiated (push to GitHub)
2. **Least privilege**: Minimal Tauri capabilities; FS/shell access restricted
3. **Transparent**: All audit events stored locally; user owns the data
4. **Ephemeral secrets**: Gitleaks findings redacted; no secret values logged

## Tauri Capabilities

### Enabled

```json
{
  "tauri": {
    "allowlist": {
      "shell": {
        "open": true,
        "execute": false,
        "sidecar": false,
        "scope": ["https://github.com/masudjbd/cca-auditor"]
      },
      "fs": {
        "readDir": false,
        "readFile": false,
        "writeFile": false,
        "createDir": false,
        "removeDir": false,
        "removeFile": false,
        "renameFile": false,
        "copyFile": false
      }
    }
  }
}
```

**Rationale**:
- `shell:open` with scoped URL allowlist: only open GitHub org repo, no arbitrary URLs
- All FS operations disabled: IPC handlers query DB directly instead of exposing file system
- No `sidecar` or `execute`: prevents arbitrary subprocess spawning from frontend

### Disabled

- `fs:*` — Database queries replace direct file access
- `shell:execute` — No arbitrary command execution
- `clipboard` — No clipboard write
- `notification` — System tray updates via C API, not Tauri
- `window` — Single fixed-size window; no child windows
- `app:prevent_close` — Allow clean shutdown

## IPC Surface

### Auditor-IPC Commands (Tauri Handlers)

All commands accept JSON-serialized parameters and return JSON-serialized results.

```rust
#[tauri::command]
async fn get_live_sessions() -> Result<Vec<AuditSession>>
// Queries DB: SELECT * FROM sessions WHERE ended_at IS NULL

#[tauri::command]
async fn get_events(session_id: String, limit: u32) -> Result<Vec<AuditEvent>>
// Queries DB with session_id filter; max limit enforced (1000)

#[tauri::command]
async fn get_samples(pid: u32, from: i64, to: i64) -> Result<Vec<ResourceSample>>
// Queries DB; timestamp range must be within last 7 days (prevents large queries)

#[tauri::command]
async fn get_alerts(dismissed: bool) -> Result<Vec<Alert>>
// Queries DB; no sensitive data in alert detail (path only, not full secret)

#[tauri::command]
async fn dismiss_alert(id: i64) -> Result<()>
// Updates DB; idempotent

#[tauri::command]
async fn generate_report(session_ids: Vec<String>, format: String) -> Result<Vec<u8>>
// Validates format (html|pdf|md|json); generates in-memory, no tmp files

#[tauri::command]
async fn push_with_guardrail(remote: String, refspec: String) -> Result<GuardrailResult>
// Validates remote URL; calls auditor-guardrail; returns findings only
```

**Input Validation**:
- Session IDs: UUID format, must exist in DB
- Limits: enforced (max 1000 events, max 7 days sample range)
- Format: whitelist (html|pdf|md|json)
- Remote: allowlist (masudjbd/fahiminfo orgs only)

**Output Sanitization**:
- All serde::Serialize on domain types
- Alert details: redacted (path only, no secret values)
- Guardrail findings: first 4 + last 4 chars of secret

## Database Security

### Location

- **macOS/Linux**: `~/.cca-audit/audit.db` (user-owned, 0600 permissions)
- **Windows**: `%APPDATA%\masudjbd\cca-audit\audit.db`

### Encryption (Optional)

SQLCipher optional feature:
```bash
cargo build --features sqlcipher
```

If enabled:
- Database locked with user-supplied passphrase at startup
- Passphrase NOT saved; required on every app restart
- 256-bit AES encryption per-page

### Write Safety

- WAL mode: concurrent reads don't block writes
- NORMAL synchronous: flushed to disk every ~500 ms (acceptable for audit trail)
- PRAGMA journal_mode=WAL
- PRAGMA synchronous=NORMAL

### Query Isolation

- Read-only replica connection for IPC queries (separate from write pool)
- Write pool (max 5 connections) for monitors only
- No cross-user access (single-user desktop app)

## Secret Handling

### Gitleaks Integration

1. **Binary supply chain**:
   - SHA-256 pinned in `src/pins.rs`
   - Downloaded on-demand from GitHub Releases
   - Verified against pin before execution

2. **Execution**:
   - Runs in subprocess with `--staged` flag (staged changes only)
   - Output redirected to JSON; no console output
   - Timeout: 30 seconds per push

3. **Findings redaction**:
   - Secret values shown as: `pass••••••••word` (first 4 + last 4 chars)
   - File path shown (e.g., `.env:42`)
   - Rule ID shown (e.g., `github-token`, `aws-key`)
   - No full secret value logged anywhere

4. **User override**:
   - Medium/low severity: user can dismiss (stored in DB for audit)
   - High severity: hard block (no override)

### Credential Leaks

- **SSH keys**: Detected by gitleaks; path patterns in sensitive-paths.toml
- **API tokens**: Detected by gitleaks (GitHub, AWS, Stripe patterns)
- **.env files**: Detected by gitleaks + path pattern
- **Cloud credentials**: AWS/GCP/Azure patterns in gitleaks

CCAudit does NOT:
- Transmit findings anywhere
- Phone home
- Store findings persistently (not in DB)
- Auto-push or auto-remediate

## Network Safety

CCAudit makes network calls ONLY:
1. **On-demand push**: User initiates via Publish UI → git push to GitHub
2. **Gitleaks binary download**: Only on first push, SHA-256 verified
3. **Optional**: Tauri updater poll (disabled by default; can be enabled in tauri.conf.json)

CCAudit does NOT:
- Collect analytics
- Phone home to masudjbd.com or anywhere
- Send event logs anywhere
- Poll for tool updates
- Connect to any service except GitHub (on user request)

## File Access

CCAudit monitors but does NOT modify:
- Home directory files (via auditor-fs)
- Project repositories (via auditor-fs)
- Temporary directories (via auditor-fs)

CCAudit writes ONLY:
- SQLite database (`~/.cca-audit/audit.db`)
- Downloaded gitleaks binary (`~/.cca-audit/tools/gitleaks`)
- Generated reports (user-initiated download)

CCAudit reads ONLY:
- config/tools.toml (tool fingerprints)
- config/sensitive-paths.toml (alert patterns)
- tauri.conf.json (app configuration)
- /proc/* (Linux process monitoring only)
- /sys/* (Linux system monitoring only)

## Threat Model Assumptions

### In Scope

- Protect against accidental secret pushes (gitleaks guardrail)
- Protect against tool misattribution (confidence tagging)
- Protect database from unauthorized access (file permissions, optional encryption)
- Audit trail integrity (SQLite WAL, checksums)

### Out of Scope

- **Local attacker with filesystem access**: If attacker can read `~/.cca-audit/`, they own the audit trail
- **Insider threat (user)**: User can disable CCAudit or edit config; audit trail reflects truth at time
- **Memory forensics**: Running app can be inspected; secrets in memory during push
- **Tauri framework vulnerabilities**: Assume Tauri is secure; report issues upstream
- **Gitleaks FP rate**: Gitleaks may have false positives; user responsible for override decisions

## Compliance Notes

- **GDPR**: No personal data collected; audit trail is fact-based (who ran what)
- **HIPAA**: Not a medical device; if used in clinical setting, user responsible for compliance
- **SOC 2**: No data sharing; local audit trail suitable for compliance audit
- **PCI DSS**: Detects secret leaks; not a replacement for SIEM or WAF

## Audit Trail Retention

- Default: unlimited (until user manually purges)
- Recommended: monthly archive + deletion (via scripts in CONTRIBUTING.md)
- Note: SQLite WAL retains ~1 MB recent writes; deleting old events frees space

## Security Hardening Checklist

- [ ] App signed + notarized (macOS)
- [ ] App signed (Windows, optional)
- [ ] SHA-256 checksums for releases
- [ ] gitleaks binary SHA-256 pinned
- [ ] Gitleaks allowlist rules curated (no FP-prone rules)
- [ ] Tauri capabilities reviewed quarterly
- [ ] Dependency audit: `cargo audit`, `npm audit`
