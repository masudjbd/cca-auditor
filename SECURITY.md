# Security policy

CCAudit handles potentially sensitive data — every file path, network destination, and subprocess command line that AI coding tools generate on a developer's machine. We take security reports seriously.

## Reporting a vulnerability

**Do not open a public issue for security bugs.** Use one of:

1. **GitHub Security Advisory** (preferred): https://github.com/masudjbd/cca-auditor/security/advisories/new
2. **Email**: open a private GitHub Security Advisory and we will respond there.

Please include:

- A clear description of the issue and its impact.
- Steps to reproduce, or a minimal proof of concept.
- The version of CCAudit you tested against (`Settings → About`).
- Your operating system and version.

We aim to acknowledge reports within **72 hours**, triage within **7 days**, and ship a fix or mitigation for critical issues within **30 days**.

## Threat model (summary)

CCAudit's full threat model is documented in [`docs/threat-model.md`](docs/threat-model.md). High-level posture:

- **Local-first.** Audit data is stored in a SQLite database in the user's app-data directory. No network egress without an explicit user action (export, sync, opt-in update check).
- **Privileged helper isolation.** When deep-audit mode is enabled, a small per-OS helper runs with elevated privileges; the main UI runs unprivileged. The helper accepts only a narrow IPC protocol over a local Unix-domain socket / named pipe, authenticated by peer-credentials.
- **Tauri capabilities.** The frontend has the minimum capability set to make IPC calls; `shell:open` is restricted to a hard-coded allowlist; `fs` access is denied except via explicit IPC commands.
- **Supply chain.** External binaries (`gitleaks`, `trufflehog`) are SHA-256 pinned and signature-verified on first download. Versions are surfaced in `Settings → External tools`.
- **Secret handling.** The pre-push guardrail scans staged diffs before any push CCAudit performs on the user's behalf. Findings are shown with the secret value redacted (first/last 4 chars only). The app never stores Git credentials — `git2`/libgit2 calls the system credential helper.

## Out of scope

- Vulnerabilities requiring an attacker who already has root/admin and the ability to modify the privileged helper binary (such an attacker has full system access and CCAudit cannot defend against them).
- Issues in third-party dependencies (`gitleaks`, `trufflehog`, Tauri, etc.) — please report those upstream.
- Findings from automated scanners with no proof of impact.

## Supported versions

| Version | Supported |
|---|---|
| `1.x` | ✅ Active development. Security fixes shipped as patch releases. |
| `0.x` (pre-release) | ⚠️ Best-effort during early access; upgrade to 1.x when available. |

## Coordinated disclosure

We follow standard coordinated-disclosure practice:

1. Reporter and maintainers agree on a fix and disclosure date.
2. Patch released; advisory marked public on the disclosure date with credit to the reporter (unless they request anonymity).
3. CVE requested via GitHub Security Advisories where applicable.

Thank you for helping keep CCAudit and its users safe.
