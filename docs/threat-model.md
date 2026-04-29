# Threat Model

## Asset Inventory

1. **Audit database** (`~/.cca-audit/audit.db`)
   - Contains: events, sessions, samples, alerts history
   - Value: high (compliance evidence, security audit trail)
   - Sensitivity: medium (no credentials, but shows tool activity patterns)

2. **Configuration files**
   - `config/tools.toml` (tool fingerprints)
   - `config/sensitive-paths.toml` (alert patterns)
   - Sensitivity: low (public schema)

3. **Generated reports**
   - HTML/PDF/Markdown exports
   - Value: medium (summary of activity; user-initiated download)
   - Sensitivity: medium (may contain PII in file paths)

4. **Gitleaks binary**
   - Downloaded once, cached at `~/.cca-audit/tools/gitleaks`
   - Value: functional (scanning tool)
   - Sensitivity: low (public binary)

## Threat Scenarios

### T1: Unauthorized Database Access

**Attack**: Local attacker reads `~/.cca-audit/audit.db` to extract activity history.

**Prerequisites**:
- Filesystem access to user's home directory
- No encryption enabled

**Impact**: High (activity patterns, timing, tools used exposed)

**Mitigation**:
- File permissions: `chmod 600 ~/.cca-audit/audit.db` (user-only)
- Optional: Enable SQLCipher encryption (requires passphrase on startup)

**Risk Level**: Medium (requires local access; typical in multi-user systems)

---

### T2: Database Tampering

**Attack**: Attacker modifies audit trail (delete events, change timestamps, insert false events).

**Prerequisites**:
- Filesystem write access
- OR Tauri process compromise

**Impact**: High (audit trail integrity compromised; evidence inadmissible)

**Mitigation**:
- SQLite PRAGMA integrity_check on app startup (detects corruption)
- Digital signature on exported reports (future: sign with user key)
- Append-only audit log export (future: immutable log to cloud storage)

**Risk Level**: Medium (requires sustained write access; detected on next integrity check)

---

### T3: Accidental Secret Push (Primary Use Case)

**Attack**: Developer stages a `.env` file with API key, pushes to GitHub.

**Prerequisites**:
- Secret in staged changeset
- User forgets to run gitleaks manually

**Impact**: Critical (credential compromise, billing, supply chain risk)

**Mitigation**: ✅ Implemented
- Guardrail scans staged changes before push
- Shows findings with redacted secret values
- Hard-blocks high-severity secrets
- User can override medium/low (tracked in audit log)

**Risk Level**: Low (primary threat CCAudit solves)

---

### T4: Guardrail Bypass

**Attack**: Attacker (with local access) disables guardrail or pushes secret to non-allowlist remote.

**Scenarios**:
1. Disable guardrail: Modify tauri.conf.json to remove command registration
2. Push to personal fork: Remote URL not in allowlist → guardrail blocks
3. Patch gitleaks output: Modify subprocess output before findings shown

**Prerequisites**:
- Write access to app code or config
- OR Tauri process compromise

**Impact**: Critical (guardrail ineffective)

**Mitigations**:
- Allowlist enforced in code (not config-driven)
- Gitleaks binary SHA-256 pinned in compiled binary
- Findings parsed in Rust (harder to MITM than JSON config)

**Risk Level**: Medium (requires code modification; visible in git diff)

---

### T5: Tool Misattribution

**Attack**: Attacker launches innocent-looking subprocess that mimics Cursor or Claude Code.

**Example**:
```bash
# Attacker creates fake Cursor
mkdir -p /tmp/Cursor.app/Contents/MacOS
cp /usr/bin/python /tmp/Cursor.app/Contents/MacOS/Cursor
# App detects it as real Cursor, attributes file edits to fake Cursor
```

**Prerequisites**:
- Shell access
- Knowledge of CCAudit's classifier heuristics

**Impact**: Medium (false attribution; audit trail misleading)

**Mitigations**:
- Confidence tagging: ambiguous processes marked as `Ambiguous` confidence
- Artifact path verification: upgrade to `Verified` only if artifact paths match
- Deep audit mode (Linux): PID→exe→hash verification (future enhancement)

**Risk Level**: Low (misattribution is tagged; reports filter by confidence)

---

### T6: Gitleaks Supply Chain Attack

**Attack**: Attacker compromises gitleaks GitHub release, replaces binary with malware.

**Prerequisites**:
- Attacker gains control of github.com/zricethezav/gitleaks
- OR Man-in-the-middle attack on GitHub release download

**Impact**: Critical (malware execution as app user)

**Mitigations**:
- SHA-256 pin in compiled binary (`src/pins.rs`)
- Signature verification (future: verify GPG signature on gitleaks release)
- Binary hosted on GitHub (trusted CDN)

**Risk Level**: Low (SHA-256 pin prevents binary swap; requires code modification to exploit)

---

### T7: Tauri Framework Vulnerability

**Attack**: Attacker exploits Tauri XSS, RCE, or privilege escalation.

**Examples**:
- Tauri IPC command injection (e.g., `get_events(; DROP TABLE events; --)`)
- Frontend XSS → RCE via Tauri bridge
- Privilege escalation in Tauri updater

**Prerequisites**:
- Vulnerability in Tauri codebase
- OR Vulnerability in CCAudit's command validation

**Impact**: Critical (app compromise)

**Mitigations**:
- Input validation on all IPC commands (UUID, limit, format whitelist)
- Parameterized queries (rusqlite prevents SQL injection)
- Content Security Policy (strict; no inline script)
- Keep Tauri updated to latest version

**Risk Level**: Low (Tauri is mature; regular updates reduce exposure)

---

### T8: Configuration Injection

**Attack**: Attacker modifies `config/tools.toml` to inject malicious regex in exe_path_contains.

**Scenario**:
```toml
[[tool]]
id = "malicious"
exe_path_contains = [".*"]  # Match everything
```

**Prerequisites**:
- Write access to config directory
- OR Attacker can force config reload

**Impact**: Medium (all tools misclassified; audit trail corrupted)

**Mitigations**:
- Config files shipped with app (not user-editable at runtime)
- Config changes require app restart
- Config schema validated on load (serde + custom validation)

**Risk Level**: Low (requires code modification; visible in git diff)

---

### T9: Memory Disclosure

**Attack**: Attacker inspects process memory to extract secrets during push.

**Scenario**:
```
User pushes with secret in staged changes
Gitleaks scans; secret value in subprocess stdout
Attacker attaches debugger; reads memory
```

**Prerequisites**:
- Local shell access (same user or root)
- Timing window during push

**Impact**: High (secret extracted)

**Mitigations**:
- None (inherent to subprocess model)
- Recommendation: Use push guardrail on secure machine (not shared)

**Risk Level**: Medium (requires active attack during push; secrets already staged before CCAudit runs)

---

### T10: Audit Trail Deletion

**Attack**: Attacker deletes `~/.cca-audit/audit.db` to erase activity history.

**Prerequisites**:
- Filesystem write access
- User doesn't have offsite backups

**Impact**: High (audit trail lost)

**Mitigations**:
- User responsibility: periodic backups
- Future: Immutable log export to cloud storage
- Future: Digital signatures on reports for non-repudiation

**Risk Level**: Medium (user responsible for backup strategy)

---

## Attack Surface Summary

| Component | Threat Level | Ease | Impact | Mitigation |
|-----------|--------------|------|--------|-----------|
| Database | Medium | Easy | High | File perms, encryption, backup |
| Config | Low | Medium | Medium | Version control, signed commits |
| Gitleaks binary | Low | Hard | Critical | SHA-256 pin, signature verify |
| Tauri IPC | Low | Hard | Critical | Input validation, parameterized queries |
| Frontend | Low | Medium | High | CSP, XSS prevention, input sanitization |
| Tool classifier | Low | Medium | Medium | Confidence tagging, artifact verification |
| Guardrail | Low | Hard | Critical | Allowlist in code, binary pin |

## Recommendations

### For Single-User System (Typical)
- ✅ File permissions sufficient (0600 on DB)
- ✅ Skip SQLCipher (adds startup latency)
- ✅ Recommended: Daily backup of `~/.cca-audit/` to cloud storage

### For Multi-User System
- ✅ Enable SQLCipher encryption
- ✅ Backup encrypted database
- ✅ Monitor `~/.cca-audit/` for unauthorized access

### For Organization Deployment
- ✅ MDM: Push config/tools.toml to all machines
- ✅ Audit: Collect reports via script; centralize to SIEM
- ✅ Sign: All executables and config; verify in app startup
- ✅ Logging: Forward to syslog for centralized audit trail

### For High-Security Environments (Finance, Healthcare)
- ✅ Immutable log storage (append-only, cloud-backed)
- ✅ Digital signatures on all reports
- ✅ Gitleaks rule customization (remove false-positive-prone rules)
- ✅ Regular security audit of CCAudit source code
- ⚠️ CCAudit is not a replacement for EDR, SIEM, or DLP solutions

## Out-of-Scope Threats

1. **Zero-day in Rust standard library**: Impossible to defend; mitigated by staying on latest version
2. **Hardware-level attacks** (Spectre, Meltdown, UEFI rootkit): Beyond app scope
3. **Physical tampering** (USB injection, DMA attack): Requires hardware security
4. **Nation-state adversaries**: Out of scope for open-source desktop app
5. **AI tool supply chain** (Cursor, Claude Code trojanized): Assume upstream is trustworthy; CCAudit only audits activity, not tool integrity

## Incident Response Playbook

### If Database Is Compromised

1. Stop app (`killall cca-audit`)
2. Rotate any exposed credentials (if evident from audit trail)
3. Backup DB: `cp ~/.cca-audit/audit.db ~/.cca-audit/audit.db.backup`
4. Review events for unauthorized activity: `SELECT * FROM events WHERE confidence = 'High' AND ts > X`
5. Enable encryption: rebuild with `--features sqlcipher`, re-import DB
6. Audit git log for any pushed secrets

### If Guardrail Is Bypassed

1. Check git log for unauthorized commits: `git log --oneline | head -20`
2. Run `git log -p --follow -S <secret_pattern>` to trace secret introduction
3. Rotate exposed secrets immediately
4. Review CCAudit config/source for tampering
5. Report to security team

### If App Is Compromised

1. Disconnect machine from network (if critical)
2. Backup audit database
3. Uninstall CCAudit
4. Review machine for other compromises (EDR scan)
5. Rebuild CCAudit from source + signed release

## Future Hardening

- [ ] Code signing (macOS, Windows)
- [ ] Gitleaks signature verification
- [ ] Immutable audit log (cloud-backed)
- [ ] Per-user encryption keys (HSM integration)
- [ ] Compliance report generation (SOC 2, HIPAA templates)
- [ ] Centralized audit aggregation
