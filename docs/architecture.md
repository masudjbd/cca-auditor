# Architecture Overview

## Crate Dependency Graph

```
auditor-ipc (Tauri command handlers)
├── auditor-core (domain types)
├── auditor-db (SQLite queries)
├── auditor-monitors (process/resource/fs/net monitors)
├── auditor-report (report generation)
└── auditor-guardrail (secret scanning)

auditor-monitors
├── auditor-core
├── auditor-db
├── auditor-fs (file-system watching)
├── auditor-net (network polling)
└── auditor-detect (tool classification)

auditor-detect
├── auditor-core
└── sysinfo (process enumeration)

auditor-fs
└── notify crate (cross-platform FS events)

auditor-net
└── netstat2 (socket enumeration)

auditor-db
├── auditor-core
├── rusqlite (SQLite driver)
├── r2d2 (connection pooling)
└── refinery (migrations)

auditor-guardrail
├── auditor-core
├── git2 (repository operations)
└── gitleaks (secret scanner)

auditor-report
├── auditor-core
├── auditor-db (data queries)
├── tera (template rendering)
└── printpdf (PDF generation)
```

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                   Tauri Desktop App                     │
│                  (apps/desktop/src-tauri)               │
│  ┌────────────────────────────────────────────────────┐ │
│  │         auditor-ipc (IPC Handlers)                │ │
│  │  - get_live_sessions()                            │ │
│  │  - get_events(session_id, limit)                  │ │
│  │  - get_samples(pid, from, to)                     │ │
│  │  - generate_report(session_ids, format)           │ │
│  │  - push_with_guardrail(remote, refspec)           │ │
│  └─────────────┬──────────────────────────────────────┘ │
│                │                                        │
│  ┌─────────────▼──────────────────────────────────────┐ │
│  │    auditor-monitors (Supervisor)                  │ │
│  │                                                    │ │
│  │  ┌──────────────┐    ┌──────────────┐             │ │
│  │  │  resource    │    │   process    │             │ │
│  │  │  monitor     │    │   monitor    │             │ │
│  │  │  (1 Hz)      │    │   (2 s)      │             │ │
│  │  └──────┬───────┘    └──────┬───────┘             │ │
│  │         │                   │                     │ │
│  │  ┌──────▼───────┐    ┌──────▼────────┐            │ │
│  │  │  auditor-fs  │    │ auditor-detect│            │ │
│  │  │  (watches)   │    │  (classify)   │            │ │
│  │  └──────┬───────┘    └──────┬────────┘            │ │
│  │         │                   │                     │ │
│  │  ┌──────▼───────┐           │                     │ │
│  │  │ auditor-net  │           │                     │ │
│  │  │ (sockets, 5s)│           │                     │ │
│  │  └──────┬───────┘           │                     │ │
│  └─────────┼───────────────────┼────────────────────┘ │
│            │                   │                     │
│  ┌─────────▼───────────────────▼────────────────────┐ │
│  │         auditor-db (SQLite, WAL mode)            │ │
│  │  ┌─────────────────────────────────────────────┐ │ │
│  │  │  Tables:                                    │ │ │
│  │  │  • sessions (tool_id, pid, confidence)     │ │ │
│  │  │  • events (kind, path, dest_addr, ts)      │ │ │
│  │  │  • samples (pid, cpu_pct, rss_bytes, ts)   │ │ │
│  │  │  • alerts (kind, severity, ts)             │ │ │
│  │  │  • processes (pid, exe_path, cmdline)      │ │ │
│  │  │  • tools (id, kind, config)                │ │ │
│  │  │  • samples_10s (rolled-up 10s avg)         │ │ │
│  │  │  • samples_1m (rolled-up 1m avg)           │ │ │
│  │  │  • reports (session_ids, format, path)     │ │ │
│  │  └─────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────┘ │
│                                                        │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Broadcast channel (for live frontend updates)   │ │
│  │  • session-opened                               │ │
│  │  • session-closed                               │ │
│  │  • resource-sample (1 Hz → 4 Hz burst)          │ │
│  │  • audit-event (batched 500 ms)                 │ │
│  │  • alert-raised                                 │ │
│  └────────────────────────────────────────────────────┘ │
│                                                        │
└────────────────────────────────────────────────────────┘
                            ▲
                            │ Tauri IPC
                            │
┌───────────────────────────▼──────────────────────────┐
│           React Frontend (apps/desktop/ui)           │
│  ┌─────────────────────────────────────────────────┐ │
│  │  useAuditStream (Zustand + event listeners)    │ │
│  └─────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────┐ │
│  │  Pages:                                         │ │
│  │  • Dashboard (stats, sparklines, alerts)       │ │
│  │  • Live (event stream, auto-scroll)            │ │
│  │  • Sessions (history table, filters, sort)     │ │
│  │  • Alerts (severity badges, dismiss)           │ │
│  │  • Reports (select sessions, generate)         │ │
│  │  • Publish (guardrail scan, findings, push)    │ │
│  │  • Settings (paths, tools, encryption)         │ │
│  └─────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────┘
```

## Event Flow

1. **Process Detection** (process_monitor, 2 s poll)
   - Enumerate all running processes via sysinfo
   - Classify via auditor-detect (exe_name, exe_path, cmd_contains)
   - On first detection: create AuditSession, emit `session-opened`
   - On process exit: close AuditSession, emit `session-closed`

2. **Resource Sampling** (resource_monitor, 1 Hz → 4 Hz burst)
   - Poll CPU %, resident set, GPU memory for all PIDs
   - Insert ResourceSample into DB
   - When ring buffer reaches 600 samples: trigger rollup (10s avg, 1m avg)
   - Emit `resource-sample` to frontend (throttled)

3. **File System Events** (fs_monitor, via notify crate)
   - Watch home/projects/code/workspace directories
   - On file change: determine active tool (session-overlap attribution)
   - Create AuditEvent with kind=FsWrite/FsRead/FsDelete, confidence tag
   - Insert into DB, batch emit `audit-event` (500 ms window)

4. **Network Attribution** (net_monitor, 5 s poll)
   - Enumerate TCP/UDP sockets per PID
   - Resolve IP → hostname (60 s cache)
   - Create AuditEvent with kind=NetConnect, dest_addr, dest_port
   - Insert into DB, batch emit `audit-event`

5. **Alert Emission**
   - Sensitive path accessed (from config/sensitive-paths.toml) → alert
   - Tool fingerprint mismatch → alert
   - Insert into DB, emit `alert-raised`

6. **Report Generation** (on-demand)
   - Select sessions from history
   - Query events + samples from DB filtered by session_ids
   - Render via tera templates (HTML/Markdown)
   - Generate PDF via printpdf
   - Return file blob

7. **Secret Guardrail** (on push)
   - Validate remote URL against allowlist (masudjbd/fahiminfo orgs)
   - Download gitleaks binary (SHA-256 pinned)
   - Scan staged changes: `gitleaks detect --staged`
   - Parse findings, redact secrets (first 4 + last 4 chars)
   - Show in UI; hard-block high severity, allow override for medium/low
   - On approval: call `git push`

## Thread/Task Model

All monitors run on Tokio async tasks spawned by auditor-monitors supervisor:

- **DB access**: Single r2d2 connection pool (max 5 connections); write-heavy ops use WAL mode + NORMAL sync
- **Event channels**: tokio::sync::broadcast for IPC; tokio::sync::mpsc for FS/net channels
- **Shutdown**: CancellationToken from tokio-util; graceful drain on app close

## Deployment Targets

- **macOS (arm64 + x86_64)**: Universal binary via Tauri build, codesign + notarize
- **Windows (x86_64)**: MSI installer via Tauri build
- **Linux (x86_64)**: AppImage + .deb packages via Tauri build

Signed binaries + SHA-256 checksums published to GitHub Releases. Tauri updater polls `latest.json`.
