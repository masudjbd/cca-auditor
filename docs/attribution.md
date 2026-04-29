# Event Attribution & Confidence Model

## Overview

CCAudit tracks which AI tool performed an action (file edit, network connection, subprocess spawn). Attribution is not trivial because:

1. Multiple tools may be active simultaneously
2. File system events don't include PID metadata on all OSes
3. Network events require socket→PID mapping (varies by OS)
4. Child processes inherit parent's descriptor tables

We use a **confidence tagging** system to quantify attribution certainty per event.

## Confidence Levels

### High
Exactly one tool is active during the event window, AND:
- For **process events**: the PID belongs directly to a known tool process
- For **FS events**: the tool was the only active process during the last 2 seconds
- For **network events**: the socket belongs to a tool process

**Reliability**: Strong. Safe for automated reporting and alerts.

**Example**:
```
14:23:00  Cursor spawns (PID 1234, high confidence)
14:23:01  File edit ~/projects/my_repo/main.py (high)
          → Only Cursor active in [23:00, 23:01], so Cursor did the edit
14:23:02  Cursor exits
```

### Ambiguous
Multiple tools active during the event window, OR:
- PID cannot be resolved (e.g., on Windows for network events)
- File system event occurs but no active process (unlikely, but possible on BSD)
- Session-overlap: two tools active for >500ms during the event window

**Reliability**: Weak. Suitable for audit trail but not for rule enforcement without user review.

**Example**:
```
14:23:00  Cursor spawns (PID 1234)
14:23:05  Claude Code spawns (PID 5678)
14:23:10  File edit ~/projects/my_repo/main.py (ambiguous)
          → Two tools active: could be either
14:23:15  Cursor exits
14:23:16  Claude Code exits
```

### Verified
High confidence + secondary confirmation:
- File access matches tool's known artifact paths (e.g., `~/.claude/projects/`, `state.vscdb` for Cursor)
- Event includes PID + matched process classifier + semantic path patterns

**Reliability**: Very strong. Reserved for high-confidence attribution + local artifact detection.

**Example**:
```
14:23:00  Claude Code spawns (PID 5678)
14:23:05  File write ~/.claude/projects/my-project/memory/project-context.md (verified)
          → PID 5678 is Claude Code + path is Claude Code artifact store
```

## Algorithm

### For File System Events

1. Event arrives with `path` and `timestamp` (within ~100 ms of actual write on most OSes)
2. Determine **active session window**: [timestamp - 2s, timestamp]
3. Query active sessions in window:
   - Count sessions with `started_at <= timestamp` and `ended_at IS NULL or ended_at >= timestamp - 2s`
4. **Assign confidence**:
   - 0 active sessions: `Ambiguous` (file edit but no tool detected—possible delay in process detection)
   - 1 active session: `High`
   - 2+ active sessions: `Ambiguous`
5. **Check artifact paths**:
   - If path matches known artifact path for the active tool: upgrade to `Verified`
   - Example: Claude Code checks `path.contains('/.claude/projects/')`

### For Network Events

1. Event arrives with `dest_ip`, `dest_port`, `protocol`, `timestamp`
2. Query open sockets at timestamp:
   - Filter sockets by dest_ip:dest_port, group by PID
3. For each matching PID:
   - Check if PID belongs to an active session
   - If exactly one session: `High` confidence
   - If multiple: `Ambiguous`
   - If no match: `Ambiguous`

### For Process Events

1. New process detected with `exe_path`, `cmdline`, `ppid` (parent PID)
2. **Classify via auditor-detect**:
   - Match exe_name against known tools (Cursor, Claude Code, etc.)
   - If match found: `High` confidence
   - If ambiguous (e.g., a generic Python process): `Ambiguous`
3. **Inherit from parent**:
   - If ppid matches an active session: inherit that session
   - Otherwise: create new session with `Ambiguous` confidence

## Known Limitations

### macOS

- **File system events**: FSEventStream API does not include PID; attribution relies on session-overlap window. Accuracy ~95% if tools have distinct activity patterns.
- **Network events**: `libproc` socket enumeration is reliable but has ~500 ms latency at high connection frequency.
- **GPU memory**: IOReport gives system-level only; per-process GPU is not exposed without privileged APIs.

**Mitigation**: Confidence tagging. High-confidence events > 80% are safe for automated alerts.

### Linux

- **File system events**: inotify is reliable; PID unavailable without elevated APIs. Session-overlap window helps.
- **Network events**: `/proc/net/tcp` enumeration is reliable but syscall-heavy. Polling at 5 s intervals reduces overhead.
- **Deep audit mode**: Optional feature behind `deep-audit` flag; uses fanotify (requires CAP_SYS_ADMIN at runtime) to include PID in events. This upgrades FS attribution to `High` for nearly all events.

**Mitigation**: Configure watch paths conservatively to reduce event volume.

### Windows

- **File system events**: ReadDirectoryChangesW is reliable; PID not available. Session-overlap essential.
- **Network events**: GetExtendedTcpTable requires elevated privileges on some Windows versions. Graceful fallback to `Ambiguous`.

**Mitigation**: Recommend running CCAudit as Admin for full coverage.

## Practical Guidance

### For Compliance Reporting
- Use events with `confidence >= High` only.
- Manual review for `Ambiguous` events (require user judgment).
- Filter by `tool_id` to scope audit trail to a specific tool.

### For Automated Alerts
- Trigger on `High` + `Verified` only.
- Example: SSH key access by non-approved tool → block if confidence is High.

### For Debugging
- Include all confidence levels; show alongside each event in Live Stream.
- Use Publish (guardrail) to block accidental secrets; confidence is advisory, not blocking.

## Future Improvements

1. **Heuristic confidence scoring**: Instead of discrete levels, compute confidence % based on:
   - Session-overlap duration ratio
   - Artifact path match score
   - PID → tool classifier score

2. **Event aggregation**: Group rapid file edits into "edit sessions" to reduce noise and improve attribution.

3. **Tool context hints**: Allow users to hint which tool they expect to be active (e.g., "I'm using Cursor now"), update confidence retroactively.
