-- CCAudit core schema

CREATE TABLE IF NOT EXISTS tools (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  display_name TEXT NOT NULL,
  config_json TEXT
);

CREATE TABLE IF NOT EXISTS processes (
  pid INTEGER NOT NULL,
  tool_id TEXT,
  exe_path TEXT,
  cmdline TEXT,
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  PRIMARY KEY (pid, started_at),
  FOREIGN KEY (tool_id) REFERENCES tools(id)
);

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  tool_id TEXT NOT NULL,
  pid INTEGER NOT NULL,
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  confidence TEXT NOT NULL,
  FOREIGN KEY (tool_id) REFERENCES tools(id)
);

CREATE TABLE IF NOT EXISTS events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  path TEXT,
  dest_addr TEXT,
  dest_port INTEGER,
  dest_hostname TEXT,
  process_args TEXT,
  confidence TEXT NOT NULL,
  ts INTEGER NOT NULL,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS samples (
  pid INTEGER NOT NULL,
  cpu_pct REAL NOT NULL,
  rss_bytes INTEGER NOT NULL,
  gpu_mem_bytes INTEGER,
  ts INTEGER NOT NULL,
  PRIMARY KEY (pid, ts)
);

CREATE TABLE IF NOT EXISTS samples_10s (
  pid INTEGER NOT NULL,
  cpu_avg REAL NOT NULL,
  rss_avg INTEGER NOT NULL,
  ts INTEGER NOT NULL,
  PRIMARY KEY (pid, ts)
);

CREATE TABLE IF NOT EXISTS samples_1m (
  pid INTEGER NOT NULL,
  cpu_avg REAL NOT NULL,
  rss_avg INTEGER NOT NULL,
  ts INTEGER NOT NULL,
  PRIMARY KEY (pid, ts)
);

CREATE TABLE IF NOT EXISTS alerts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  kind TEXT NOT NULL,
  severity TEXT NOT NULL,
  detail_json TEXT,
  ts INTEGER NOT NULL,
  dismissed INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS config (
  key TEXT PRIMARY KEY,
  value TEXT
);

CREATE TABLE IF NOT EXISTS reports (
  id TEXT PRIMARY KEY,
  session_ids TEXT NOT NULL,
  format TEXT NOT NULL,
  path TEXT,
  created_at INTEGER NOT NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_events_session_id ON events(session_id);
CREATE INDEX IF NOT EXISTS idx_events_ts ON events(ts);
CREATE INDEX IF NOT EXISTS idx_samples_pid_ts ON samples(pid, ts);
CREATE INDEX IF NOT EXISTS idx_sessions_tool_id ON sessions(tool_id);
CREATE INDEX IF NOT EXISTS idx_processes_pid ON processes(pid);
CREATE INDEX IF NOT EXISTS idx_alerts_ts ON alerts(ts);

-- WAL mode and pragmas
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-64000;
PRAGMA temp_store=MEMORY;
