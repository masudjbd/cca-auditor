import { useState, useEffect, useMemo } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import {
  TOOL_PATH_SUGGESTIONS,
  COMMON_WORKSPACE_PATHS,
  getSuggestedPathsForTools,
  loadSettings,
  saveSettings,
  PersistedSettings,
} from '../lib/toolPaths'
import {
  saveSettingsToBackend,
  loadSettingsFromBackend,
  getDbStats,
  purgeAllData,
  DbStats,
} from '../lib/tauri'

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

function formatTime(ts: number | null): string {
  if (!ts) return '—'
  return new Date(ts * 1000).toLocaleString()
}

export default function Settings() {
  const [watchPaths, setWatchPaths] = useState<string[]>([])
  const [newPath, setNewPath] = useState('')
  const [enabledTools, setEnabledTools] = useState<Set<string>>(new Set())
  const [encryption, setEncryption] = useState(false)
  const [saved, setSaved] = useState(false)
  const [dbStats, setDbStats] = useState<DbStats | null>(null)

  const tools = Object.keys(TOOL_PATH_SUGGESTIONS)

  const loadStats = async () => {
    const stats = await getDbStats()
    setDbStats(stats)
  }

  useEffect(() => {
    loadStats()
    const interval = setInterval(loadStats, 5000) // refresh every 5s
    return () => clearInterval(interval)
  }, [])

  const handlePurge = async () => {
    if (
      !confirm(
        'This will permanently delete ALL audit data (sessions, events, samples, alerts).\n\nThis cannot be undone. Continue?'
      )
    )
      return
    try {
      await purgeAllData()
      await loadStats()
      alert('All audit data has been deleted.')
    } catch (e) {
      alert(`Failed to purge: ${e}`)
    }
  }

  // Load persisted settings on mount (try backend first, fall back to localStorage)
  useEffect(() => {
    const initSettings = async () => {
      const backendSettings = await loadSettingsFromBackend()
      if (backendSettings) {
        setWatchPaths(backendSettings.watch_paths)
        setEnabledTools(new Set(backendSettings.enabled_tools))
        setEncryption(backendSettings.encryption)
      } else {
        const localSettings = loadSettings()
        setWatchPaths(localSettings.watchPaths)
        setEnabledTools(new Set(localSettings.enabledTools))
        setEncryption(localSettings.encryption)
      }
    }
    initSettings()
  }, [])

  // Compute intelligent path suggestions based on enabled tools
  const suggestions = useMemo(() => {
    return getSuggestedPathsForTools(Array.from(enabledTools))
  }, [enabledTools])

  // Find paths that are suggested but not yet added
  const recommendedPaths = useMemo(() => {
    const allSuggested = suggestions.flatMap((s) => s.paths)
    return allSuggested.filter((p) => !watchPaths.includes(p))
  }, [suggestions, watchPaths])

  const addPath = (path: string) => {
    if (path && !watchPaths.includes(path)) {
      setWatchPaths([...watchPaths, path])
      setNewPath('')
    }
  }

  const removePath = (path: string) => {
    setWatchPaths(watchPaths.filter((p) => p !== path))
  }

  const browseForPath = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select directory to monitor',
      })
      if (selected && typeof selected === 'string' && !watchPaths.includes(selected)) {
        setWatchPaths([...watchPaths, selected])
      }
    } catch (error) {
      console.error('Failed to open directory picker:', error)
      alert('Browse not available. Please type the path manually.')
    }
  }

  const addAllRecommended = () => {
    const newPaths = [...watchPaths]
    for (const path of recommendedPaths) {
      if (!newPaths.includes(path)) {
        newPaths.push(path)
      }
    }
    setWatchPaths(newPaths)
  }

  const toggleTool = (tool: string) => {
    const newSet = new Set(enabledTools)
    if (newSet.has(tool)) {
      newSet.delete(tool)
    } else {
      newSet.add(tool)
    }
    setEnabledTools(newSet)
  }

  const handleSave = async () => {
    const settings: PersistedSettings = {
      watchPaths,
      enabledTools: Array.from(enabledTools),
      encryption,
    }
    // Save to localStorage (always works)
    saveSettings(settings)

    // Also save to backend (best effort)
    try {
      await saveSettingsToBackend({
        watch_paths: watchPaths,
        enabled_tools: Array.from(enabledTools),
        encryption,
      })
    } catch {
      // localStorage already saved, OK to proceed
    }

    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  return (
    <div className="p-8 max-w-3xl">
      <h2 className="text-3xl font-bold text-gray-900">Settings</h2>
      <p className="text-gray-600 mt-2">Configure watch paths, tool fingerprints, encryption</p>

      <div className="mt-8 space-y-8">
        {/* Watch Paths */}
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Watch Paths</h3>
          <p className="text-sm text-gray-600 mb-4">
            Directories to monitor for file system activity
          </p>

          {/* Current paths */}
          <div className="space-y-2 mb-4">
            {watchPaths.length === 0 ? (
              <p className="text-sm text-gray-400 italic py-3">No paths configured</p>
            ) : (
              watchPaths.map((path) => (
                <div
                  key={path}
                  className="flex items-center justify-between bg-gray-50 p-3 rounded border border-gray-200"
                >
                  <code className="text-sm text-gray-700 break-all">{path}</code>
                  <button
                    onClick={() => removePath(path)}
                    className="ml-3 text-sm text-red-600 hover:text-red-700 font-medium flex-shrink-0"
                  >
                    Remove
                  </button>
                </div>
              ))
            )}
          </div>

          {/* Add new path */}
          <div className="flex gap-2 mb-4">
            <input
              type="text"
              value={newPath}
              onChange={(e) => setNewPath(e.target.value)}
              placeholder="/path/to/monitor"
              className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              onKeyPress={(e) => e.key === 'Enter' && addPath(newPath)}
            />
            <button
              onClick={() => addPath(newPath)}
              disabled={!newPath}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Add
            </button>
            <button
              onClick={browseForPath}
              className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 font-medium border border-gray-300"
              title="Browse for directory"
            >
              📁 Browse
            </button>
          </div>

          {/* Common workspace paths */}
          <div className="mt-6 border-t pt-4">
            <div className="flex items-center justify-between mb-3">
              <h4 className="text-sm font-semibold text-gray-700">
                {COMMON_WORKSPACE_PATHS.tool_name}
              </h4>
            </div>
            <div className="flex flex-wrap gap-2">
              {COMMON_WORKSPACE_PATHS.paths.map((path) => {
                const isAdded = watchPaths.includes(path)
                return (
                  <button
                    key={path}
                    onClick={() => addPath(path)}
                    disabled={isAdded}
                    className={`text-xs px-3 py-1.5 rounded border ${
                      isAdded
                        ? 'bg-green-50 text-green-700 border-green-200 cursor-default'
                        : 'bg-white text-gray-700 border-gray-300 hover:bg-blue-50 hover:border-blue-300'
                    }`}
                  >
                    {isAdded ? '✓ ' : '+ '}
                    <code className="font-mono">{path}</code>
                  </button>
                )
              })}
            </div>
          </div>

          {/* Tool-specific recommendations */}
          {suggestions.length > 0 && (
            <div className="mt-6 border-t pt-4">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-sm font-semibold text-gray-700">
                  Recommended for Enabled Tools
                </h4>
                {recommendedPaths.length > 0 && (
                  <button
                    onClick={addAllRecommended}
                    className="text-xs px-3 py-1 bg-blue-600 text-white rounded font-medium hover:bg-blue-700"
                  >
                    Add All ({recommendedPaths.length})
                  </button>
                )}
              </div>
              <div className="space-y-3">
                {suggestions.map((suggestion) => (
                  <div key={suggestion.tool_id} className="bg-gray-50 rounded p-3">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium text-gray-900">
                        {suggestion.tool_name}
                      </span>
                      <span className="text-xs text-gray-500">{suggestion.description}</span>
                    </div>
                    <div className="flex flex-wrap gap-1.5">
                      {suggestion.paths.map((path) => {
                        const isAdded = watchPaths.includes(path)
                        return (
                          <button
                            key={path}
                            onClick={() => addPath(path)}
                            disabled={isAdded}
                            className={`text-xs px-2 py-1 rounded border ${
                              isAdded
                                ? 'bg-green-50 text-green-700 border-green-200 cursor-default'
                                : 'bg-white text-gray-700 border-gray-300 hover:bg-blue-50 hover:border-blue-300'
                            }`}
                          >
                            {isAdded ? '✓ ' : '+ '}
                            <code className="font-mono">{path}</code>
                          </button>
                        )
                      })}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Tool Fingerprints */}
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Tool Detection</h3>
          <p className="text-sm text-gray-600 mb-4">
            Enable/disable detection for specific AI tools (affects path suggestions)
          </p>

          <div className="grid grid-cols-2 gap-2">
            {tools.map((tool) => (
              <label
                key={tool}
                className="flex items-center gap-3 p-3 rounded border border-gray-200 hover:bg-gray-50 cursor-pointer"
              >
                <input
                  type="checkbox"
                  checked={enabledTools.has(tool)}
                  onChange={() => toggleTool(tool)}
                  className="w-4 h-4 rounded border-gray-300"
                />
                <span className="text-sm font-medium text-gray-900">
                  {TOOL_PATH_SUGGESTIONS[tool]?.tool_name || tool}
                </span>
              </label>
            ))}
          </div>
        </div>

        {/* Encryption */}
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Security</h3>

          <label className="flex items-center gap-3 p-3 rounded hover:bg-gray-50 cursor-pointer">
            <input
              type="checkbox"
              checked={encryption}
              onChange={(e) => setEncryption(e.target.checked)}
              className="w-4 h-4 rounded border-gray-300"
            />
            <div>
              <p className="text-sm font-medium text-gray-900">Encrypt database</p>
              <p className="text-xs text-gray-600 mt-1">
                Encrypt audit database with SQLCipher (requires password at startup)
              </p>
            </div>
          </label>
        </div>

        {/* Save Button */}
        <div className="flex items-center gap-3">
          <button
            onClick={handleSave}
            className="flex-1 px-6 py-3 bg-green-600 text-white rounded-lg font-medium hover:bg-green-700 transition-colors"
          >
            Save Settings
          </button>
          {saved && (
            <span className="text-sm text-green-600 font-medium">✓ Saved</span>
          )}
        </div>

        {/* Database Stats */}
        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900">Database</h3>
            <button
              onClick={loadStats}
              className="text-sm text-blue-600 hover:text-blue-700"
            >
              Refresh
            </button>
          </div>

          {dbStats ? (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="bg-gray-50 rounded p-3">
                  <p className="text-xs text-gray-500 uppercase">Database Size</p>
                  <p className="text-xl font-bold text-gray-900 mt-1">
                    {formatBytes(dbStats.db_size_bytes)}
                  </p>
                </div>
                <div className="bg-gray-50 rounded p-3">
                  <p className="text-xs text-gray-500 uppercase">Total Sessions</p>
                  <p className="text-xl font-bold text-gray-900 mt-1">
                    {dbStats.total_sessions}
                  </p>
                  <p className="text-xs text-gray-500 mt-0.5">
                    {dbStats.active_sessions} active
                  </p>
                </div>
                <div className="bg-gray-50 rounded p-3">
                  <p className="text-xs text-gray-500 uppercase">Total Events</p>
                  <p className="text-xl font-bold text-gray-900 mt-1">
                    {dbStats.total_events.toLocaleString()}
                  </p>
                </div>
                <div className="bg-gray-50 rounded p-3">
                  <p className="text-xs text-gray-500 uppercase">Resource Samples</p>
                  <p className="text-xl font-bold text-gray-900 mt-1">
                    {dbStats.total_samples.toLocaleString()}
                  </p>
                </div>
                <div className="bg-gray-50 rounded p-3">
                  <p className="text-xs text-gray-500 uppercase">Total Alerts</p>
                  <p className="text-xl font-bold text-gray-900 mt-1">
                    {dbStats.total_alerts}
                  </p>
                  <p className="text-xs text-gray-500 mt-0.5">
                    {dbStats.undismissed_alerts} active
                  </p>
                </div>
                <div className="bg-gray-50 rounded p-3">
                  <p className="text-xs text-gray-500 uppercase">Date Range</p>
                  <p className="text-sm text-gray-900 mt-1">
                    {formatTime(dbStats.oldest_event_ts)}
                  </p>
                  <p className="text-xs text-gray-500">to</p>
                  <p className="text-sm text-gray-900">
                    {formatTime(dbStats.newest_event_ts)}
                  </p>
                </div>
              </div>

              {dbStats.events_by_kind.length > 0 && (
                <div>
                  <p className="text-sm font-medium text-gray-700 mb-2">
                    Events by Kind
                  </p>
                  <div className="space-y-1">
                    {dbStats.events_by_kind.map(([kind, count]) => {
                      const total = dbStats.events_by_kind.reduce(
                        (s, [, c]) => s + c,
                        0
                      )
                      const pct = total > 0 ? (count / total) * 100 : 0
                      return (
                        <div key={kind} className="flex items-center gap-3">
                          <div className="w-24 text-xs text-gray-600">{kind}</div>
                          <div className="flex-1 h-5 bg-gray-100 rounded overflow-hidden relative">
                            <div
                              className="h-full bg-blue-500"
                              style={{ width: `${pct}%` }}
                            />
                            <span className="absolute inset-0 flex items-center pl-2 text-xs text-gray-700">
                              {count.toLocaleString()} ({pct.toFixed(1)}%)
                            </span>
                          </div>
                        </div>
                      )
                    })}
                  </div>
                </div>
              )}

              <div className="pt-4 border-t border-gray-200">
                <p className="text-xs text-gray-500 mb-2">
                  Database location: <code className="text-gray-700">~/.cca-audit/audit.db</code>
                </p>
                <button
                  onClick={handlePurge}
                  className="text-sm text-red-600 hover:text-red-700 font-medium"
                >
                  Delete All Audit Data...
                </button>
              </div>
            </div>
          ) : (
            <p className="text-sm text-gray-400">Loading database stats…</p>
          )}
        </div>
      </div>
    </div>
  )
}
