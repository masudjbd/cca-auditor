# Contributing to CCAudit

Thanks for your interest in CCAudit. We welcome contributions: bug reports, feature requests, PRs, documentation, and tool fingerprints.

## Code of conduct

Be respectful, inclusive, and assume good faith. Harassment, discrimination, and bad-faith arguments have no place here.

## Reporting bugs

1. Check [existing issues](https://github.com/masudjbd/cca-auditor/issues) to see if it's already reported.
2. Open a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs. actual behavior
   - macOS/Windows/Linux, app version, AI tool(s) involved
3. **Security bugs**: Do NOT open a public issue. Use [GitHub Security Advisories](https://github.com/masudjbd/cca-auditor/security/advisories/new).

## Feature requests

1. Check [existing issues](https://github.com/masudjbd/cca-auditor/discussions) and discussions.
2. Open an issue with clear description and use case.
3. We prioritize requests that align with the core mission: audit trail + resource monitoring + secret safety.

## Setup for local development

### Prerequisites

- Rust 1.70+ ([rustup](https://rustup.rs))
- Node 18+ ([nodejs.org](https://nodejs.org))
- pnpm: `npm install -g pnpm`
- Tauri CLI: `cargo install tauri-cli`
- macOS: Xcode Command Line Tools (`xcode-select --install`)
- Linux: libssl-dev, libgtk-3-dev, libayatana-appindicator3-dev (Debian/Ubuntu; check [Tauri docs](https://tauri.app/v1/guides/getting-started/prerequisites))

### Clone and install

```bash
git clone https://github.com/masudjbd/cca-auditor.git
cd cca-auditor
pnpm install
cargo fetch
```

### Run dev mode

```bash
cargo tauri dev
```

This launches the Tauri app in dev mode. File changes in `apps/desktop/ui/` reload the frontend instantly (Vite HMR); Rust changes require manual restart.

### Run tests

```bash
cargo test --workspace
```

### Code checks

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
gitleaks detect --source . --no-banner --redact
```

Before submitting a PR, ensure all three pass.

## Crate guide

### Core domain types

**`crates/auditor-core`**: types and enums used across all crates.

- `events.rs`: `AuditEvent` (FsWrite, NetConn, ProcessSpawn), `EventKind`, `Confidence`
- `samples.rs`: `ResourceSample` (CPU%, RSS, GPU memory, timestamp)
- `session.rs`: `AuditSession` (id, tool_id, pid, timestamps)
- `config.rs`: `AppConfig`, `ToolFingerprint`
- `error.rs`: `CcaError` (app-wide error enum)
- `tool.rs`: `ToolId`, `ToolKind`

All types must implement `Serialize`, `Deserialize`, `Clone`, `Debug`.

### Database layer

**`crates/auditor-db`**: SQLite persistence.

- `pool.rs`: `DbPool` (r2d2 wrapper)
- `queries/`: query functions per domain (events, samples, sessions)
- `downsample.rs`: LTTB (Largest-Triangle-Three-Buckets) downsampling for chart UI
- `migrations/`: SQL migration files (refinery)

Add a new query: create `src/queries/your_entity.rs`, impl functions, and export from `lib.rs`.

### Process detection

**`crates/auditor-detect`**: classify running processes as AI tools.

- `lib.rs`: `pub fn classify(proc, fingerprints) -> Option<ToolId>`
- Reads `config/tools.toml` at runtime; no recompile needed to add a new tool

### Monitors

**`crates/auditor-monitors`**: main supervisor spawning all background tasks.

- `lib.rs`: `pub async fn run_supervisor(db, broadcast_tx, shutdown_token)`
- Owns: `resource_monitor`, `process_monitor`, `fs_monitor`, `net_monitor` subtasks
- All communicate via `tokio::sync::broadcast` channel

### File system watcher

**`crates/auditor-fs`**: tracks file-system changes.

- Uses `notify` crate (cross-platform)
- Attributes events to active tools via session-overlap heuristic
- Emits `(path, kind, tool_id, confidence)` tuples

### Network monitor

**`crates/auditor-net`**: tracks network connections.

- Uses `netstat2` for socket enumeration
- Resolves IPs to hostnames async (hickory-resolver)
- Polls every 5 seconds

### Reports

**`crates/auditor-report`**: generate audit reports.

- Templates in `templates/*.tera` (Tera templating engine)
- `pub fn generate(session_ids, format, db) -> Result<Vec<u8>>`
- Formats: HTML, PDF (printpdf), Markdown, JSON

### Secret guardrail

**`crates/auditor-guardrail`**: gitleaks integration.

- `pub async fn scan_staged(repo_path) -> Result<Vec<Finding>>`
- `pub async fn push_with_guardrail(remote, refspec, allowlist) -> Result<PushResult>`
- SHA-256 pins in `src/pins.rs` (per OS/arch)
- High-severity findings block; medium/low can be overridden

### IPC & Tauri

**`crates/auditor-ipc`**: Tauri command handlers.

- `#[tauri::command]` functions (RPC entry points)
- Event broadcasts to frontend
- DB queries dispatched here

**`apps/desktop/src-tauri`**: Tauri shell.

- `src/main.rs`: app bootstrap, DB init, monitor spawn, tray icon setup
- `tauri.conf.json`: app config (ID, window, capabilities, updater)

### Frontend

**`apps/desktop/ui`**: React + TypeScript + Vite.

- `src/routes/`: pages (Dashboard, Live, Reports, Publish, etc.)
- `src/hooks/useAuditStream.ts`: Tauri event listener
- `src/store/`: Zustand state management
- `src/components/`: reusable UI (Charts, Tables, Alerts)

## Adding support for a new AI tool

### Step 1: Determine fingerprints

Find the process name(s) and command-line patterns that uniquely identify the tool. Examples:

**Cursor**: binary name `Cursor` or `cursor`, exe path contains `/Cursor.app/`
**Claude Code**: binary name `claude`, command contains `@anthropic-ai/claude-code`
**Ollama**: binary name `ollama`, listens on localhost:11434

### Step 2: Edit `config/tools.toml`

Add a `[[tool]]` stanza:

```toml
[[tool]]
id = "my-tool"
kind = "editor"  # or "cli-agent", "server", "browser-extension"
display_name = "My Tool"
exe_name = ["mytool", "mytool-bin"]
exe_path_contains = ["/opt/mytool", "/Applications/MyTool.app"]
cmd_contains = ["--my-tool-flag"]
include_descendants = true
```

Fields:
- `id`: unique identifier (alphanumeric + hyphens)
- `kind`: broad category
- `display_name`: human-readable name for UI
- `exe_name`: list of binary names to match
- `exe_path_contains`: list of path substrings to match
- `cmd_contains`: list of command-line substrings to match
- `include_descendants`: if true, child processes inherit the tool classification

### Step 3: (Optional) Monitor local artifacts

If your tool writes local artifacts (logs, chat history, session files), add paths to `auditor-detect` in `src/lib.rs`:

```rust
const CLAUDE_CODE_ARTIFACTS: &[&str] = &[
    "~/.claude/projects/",
    "~/.claude/todos/",
];
```

This lets CCAudit infer the tool was active even if the process isn't currently running (e.g., CLI tools that exit after each command).

### Step 4: Submit a PR

1. Fork the repo
2. Create a branch: `git checkout -b add-mytool-support`
3. Commit your changes: `git add config/tools.toml && git commit -m "Add MyTool fingerprints"`
4. Push and open a PR with:
   - Tool name and link
   - Why this tool matters for auditing (e.g., "popular coding assistant, 50K users")
   - Verification steps (e.g., "run MyTool on a test project, confirm detection in dashboard")

We'll review and merge within 48 hours (or request clarifications).

## Submitting a PR

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make changes, add tests if applicable
3. Run checks: `cargo fmt`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
4. Commit with clear message: "Implement X, fixes #123" or "Add support for Y tool"
5. Push and open a PR
6. Respond to feedback and CI results
7. Once approved, squash if helpful and merge

### PR checklist

- [ ] Cargo checks pass (`cargo check --workspace`)
- [ ] Tests pass (`cargo test --workspace`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Clippy clean (`cargo clippy --workspace -- -D warnings`)
- [ ] New feature has test coverage (if applicable)
- [ ] README updated (if new feature or tool)
- [ ] Changelog entry added (see `CHANGELOG.md`)
- [ ] No secrets committed (gitleaks passes)

## Questions?

Open a GitHub issue or discussion, or email maintainer. We're happy to help onboard contributors.

---

Thank you for contributing to CCAudit!
