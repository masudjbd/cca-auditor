# Changelog

All notable changes to CCAudit are documented in this file. We follow [Semantic Versioning](https://semver.org).

## [Unreleased]

### Added

- Core project initialization: git repo, .gitignore, LICENSE, SECURITY.md, secret-safety baseline
- README: feature overview, installation, architecture guide
- CONTRIBUTING: dev setup, crate guide, tool fingerprint contribution workflow
- Cargo workspace: 9 core crates + Tauri desktop app scaffold
- Domain types (auditor-core): events, sessions, resource samples, errors, config
- Database layer (auditor-db): SQLite schema, migrations, query functions, LTTB downsampling
- Process detection (auditor-detect): tool classification, local artifact monitoring
- Monitor supervisor (auditor-monitors): resource, process, FS, and net monitor coordination
- File-system watcher (auditor-fs): cross-platform notify integration
- Network monitor (auditor-net): socket attribution, DNS resolution
- Report generation (auditor-report): HTML/PDF/Markdown/JSON exports
- Secret guardrail (auditor-guardrail): gitleaks integration, push-guard flow
- IPC layer (auditor-ipc): Tauri command handlers, event broadcast
- Tauri app shell: tray icon, system integration, app bootstrap
- React frontend: dashboard, live audit stream, reports, secret guardrail UI
- Config files: tools.toml (AI-tool fingerprints), sensitive-paths.toml
- Documentation: architecture, attribution, security model, threat model
- CI/CD: GitHub Actions for check/test/build/release

### Architecture

- **Tech stack**: Tauri 2 + Rust backend, React 18 + TypeScript + Vite + Tailwind + shadcn/ui frontend
- **Data storage**: SQLite (WAL mode, optional SQLCipher)
- **Monitoring**: 1 Hz resource sampling, FS events via FSEvents/inotify, network polling at 5 s
- **Attribution**: Session-overlap confidence model (High/Ambiguous), optional deep-audit mode
- **Security**: local-first, zero network egress, Tauri sandboxing, gitleaks pre-push scan

### Known limitations

- File-system attribution lossy without elevated APIs (ETW/fanotify); mitigated by confidence tagging
- macOS network attribution via libproc brittle at high connection frequency (poll at 5 s)
- GPU memory on Apple Silicon is shared RAM; per-process attribution not available
- Tool fingerprints will require community maintenance as tools rename binaries over time

## Versions

### Future milestones

- **v1.0** (Q3 2026): Full feature set, all platforms, packaging, auto-updater
- **v0.2** (Q2 2026): Beta: FS/net monitors, reports, secret guardrail
- **v0.1** (May 2026): Alpha: process detection, resource sampling, live tray, dashboard

---

**Changelog maintained by the CCAudit team.** See `git log` for detailed commit history.
