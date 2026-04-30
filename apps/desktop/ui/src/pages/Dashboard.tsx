import { useEffect, useMemo } from 'react'
import { LineChart, Line, ResponsiveContainer } from 'recharts'
import { useAuditStore } from '../store/auditStore'
import { useAuditStream } from '../hooks/useAuditStream'
import { getLiveSessions, getAlerts } from '../lib/tauri'

const TOOL_DISPLAY: Record<string, { label: string; color: string; emoji: string }> = {
  cursor: { label: 'Cursor', color: 'bg-purple-50 border-purple-200 text-purple-700', emoji: '🖱️' },
  'claude-code': { label: 'Claude Code', color: 'bg-orange-50 border-orange-200 text-orange-700', emoji: '🤖' },
  'claude-desktop': { label: 'Claude Desktop', color: 'bg-orange-50 border-orange-200 text-orange-700', emoji: '💬' },
  windsurf: { label: 'Windsurf', color: 'bg-cyan-50 border-cyan-200 text-cyan-700', emoji: '🌊' },
  ollama: { label: 'Ollama', color: 'bg-gray-50 border-gray-300 text-gray-700', emoji: '🦙' },
  lmstudio: { label: 'LM Studio', color: 'bg-emerald-50 border-emerald-200 text-emerald-700', emoji: '🎬' },
  aider: { label: 'Aider', color: 'bg-rose-50 border-rose-200 text-rose-700', emoji: '✏️' },
  cline: { label: 'Cline', color: 'bg-indigo-50 border-indigo-200 text-indigo-700', emoji: '🔌' },
  continue: { label: 'Continue', color: 'bg-blue-50 border-blue-200 text-blue-700', emoji: '▶️' },
  'copilot-chat': { label: 'Copilot Chat', color: 'bg-slate-50 border-slate-200 text-slate-700', emoji: '👨‍✈️' },
  tabnine: { label: 'Tabnine', color: 'bg-amber-50 border-amber-200 text-amber-700', emoji: '⚡' },
  supermaven: { label: 'Supermaven', color: 'bg-pink-50 border-pink-200 text-pink-700', emoji: '🦅' },
}

export default function Dashboard() {
  const { sessions, samples, alerts } = useAuditStore()
  useAuditStream()

  useEffect(() => {
    const loadInitial = async () => {
      const liveSessions = await getLiveSessions()
      const liveAlerts = await getAlerts(false)
      useAuditStore.setState({ sessions: liveSessions, alerts: liveAlerts })
    }
    loadInitial()
    const interval = setInterval(loadInitial, 5000) // refresh every 5s
    return () => clearInterval(interval)
  }, [])

  const activeSessions = useMemo(
    () => sessions.filter((s) => !s.ended_at),
    [sessions]
  )

  const sessionsByTool = useMemo(() => {
    const map = new Map<string, typeof activeSessions>()
    for (const s of activeSessions) {
      const list = map.get(s.tool_id) ?? []
      list.push(s)
      map.set(s.tool_id, list)
    }
    return map
  }, [activeSessions])

  const allSamples = Array.from(samples.values()).flat()
  const avgCpu = allSamples.length
    ? (allSamples.reduce((sum, s) => sum + s.cpu_pct, 0) / allSamples.length).toFixed(1)
    : '0.0'

  const totalMem = allSamples.length
    ? (
        allSamples
          .slice(-Math.min(allSamples.length, sessionsByTool.size || 1))
          .reduce((sum, s) => sum + s.rss_bytes, 0) /
        1024 /
        1024
      ).toFixed(0)
    : '0'

  const recentAlerts = alerts.filter((a) => !a.dismissed).slice(0, 5)

  const samplesForPid = (pid: number) =>
    (samples.get(pid) ?? []).slice(-60)

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-gray-900">Dashboard</h2>
      <p className="text-gray-600 mt-2">
        Live monitoring of AI coding tools, file system, and network activity
      </p>

      {/* Top stats */}
      <div className="mt-8 grid grid-cols-4 gap-6">
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Active Tools</h3>
          <p className="text-3xl font-bold text-gray-900 mt-2">{sessionsByTool.size}</p>
          <p className="text-xs text-gray-400 mt-2">{activeSessions.length} sessions</p>
        </div>
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Avg CPU</h3>
          <p className="text-3xl font-bold text-gray-900 mt-2">{avgCpu}%</p>
          <div className="mt-2 h-8">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={allSamples.slice(-60)}>
                <Line
                  type="monotone"
                  dataKey="cpu_pct"
                  stroke="#3b82f6"
                  dot={false}
                  isAnimationActive={false}
                  strokeWidth={1.5}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Total Memory</h3>
          <p className="text-3xl font-bold text-gray-900 mt-2">{totalMem} MB</p>
          <div className="mt-2 h-8">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart
                data={allSamples
                  .slice(-60)
                  .map((s) => ({ ...s, rss_mb: s.rss_bytes / 1024 / 1024 }))}
              >
                <Line
                  type="monotone"
                  dataKey="rss_mb"
                  stroke="#10b981"
                  dot={false}
                  isAnimationActive={false}
                  strokeWidth={1.5}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Alerts</h3>
          <p className="text-3xl font-bold text-gray-900 mt-2">
            {recentAlerts.length}
          </p>
          <p className="text-xs text-gray-400 mt-2">
            {recentAlerts.filter((a) => a.severity === 'high').length} high severity
          </p>
        </div>
      </div>

      {/* Per-tool tiles */}
      <div className="mt-8">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          Active AI Tools
        </h3>
        {sessionsByTool.size === 0 ? (
          <div className="bg-white rounded-lg shadow p-12 text-center">
            <p className="text-gray-400 text-lg">No AI tools detected yet</p>
            <p className="text-sm text-gray-400 mt-2">
              Start Cursor, Claude Code, Ollama, or another supported tool to see activity here
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-3 gap-4">
            {Array.from(sessionsByTool.entries()).map(([toolId, toolSessions]) => {
              const display = TOOL_DISPLAY[toolId] || {
                label: toolId,
                color: 'bg-gray-50 border-gray-200 text-gray-700',
                emoji: '🔧',
              }
              const allSamplesForTool = toolSessions.flatMap((s) =>
                samplesForPid(s.pid)
              )
              const toolCpu = allSamplesForTool.length
                ? (
                    allSamplesForTool.reduce((sum, s) => sum + s.cpu_pct, 0) /
                    allSamplesForTool.length
                  ).toFixed(1)
                : '—'
              const toolMem = allSamplesForTool.length
                ? Math.round(
                    allSamplesForTool.reduce((sum, s) => sum + s.rss_bytes, 0) /
                      allSamplesForTool.length /
                      1024 /
                      1024
                  ).toString()
                : '—'

              return (
                <div
                  key={toolId}
                  className={`rounded-lg shadow border p-5 ${display.color}`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="text-2xl">{display.emoji}</span>
                      <span className="font-semibold text-gray-900">
                        {display.label}
                      </span>
                    </div>
                    <span className="text-xs px-2 py-0.5 bg-white rounded-full font-medium">
                      {toolSessions.length}{' '}
                      {toolSessions.length === 1 ? 'session' : 'sessions'}
                    </span>
                  </div>

                  <div className="mt-4 grid grid-cols-2 gap-2">
                    <div>
                      <p className="text-xs text-gray-600">CPU</p>
                      <p className="text-lg font-bold text-gray-900">{toolCpu}%</p>
                    </div>
                    <div>
                      <p className="text-xs text-gray-600">Memory</p>
                      <p className="text-lg font-bold text-gray-900">{toolMem} MB</p>
                    </div>
                  </div>

                  <div className="mt-3 h-10">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={allSamplesForTool}>
                        <Line
                          type="monotone"
                          dataKey="cpu_pct"
                          stroke="currentColor"
                          dot={false}
                          isAnimationActive={false}
                          strokeWidth={1.5}
                        />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>

                  <div className="mt-3 text-xs text-gray-600">
                    PIDs: {toolSessions.map((s) => s.pid).join(', ')}
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>

      {/* Recent alerts */}
      {recentAlerts.length > 0 && (
        <div className="mt-8 bg-white rounded-lg shadow">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">Recent Alerts</h3>
          </div>
          <div className="divide-y">
            {recentAlerts.map((alert) => (
              <div key={alert.id} className="px-6 py-4 flex items-start gap-4">
                <div
                  className={`w-2 h-2 rounded-full mt-1.5 flex-shrink-0 ${
                    alert.severity === 'high'
                      ? 'bg-red-500'
                      : alert.severity === 'medium'
                        ? 'bg-yellow-500'
                        : 'bg-blue-500'
                  }`}
                />
                <div className="flex-1">
                  <p className="text-sm font-medium text-gray-900">{alert.kind}</p>
                  <p className="text-sm text-gray-600 mt-1 break-all">
                    {alert.detail}
                  </p>
                  <p className="text-xs text-gray-400 mt-1">
                    {new Date(alert.timestamp).toLocaleString()}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
