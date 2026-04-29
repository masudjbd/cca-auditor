import { useEffect } from 'react'
import { LineChart, Line, ResponsiveContainer } from 'recharts'
import { useAuditStore } from '../store/auditStore'
import { useAuditStream } from '../hooks/useAuditStream'
import { getLiveSessions, getAlerts } from '../lib/tauri'

export default function Dashboard() {
  const { sessions, samples, alerts } = useAuditStore()
  useAuditStream()

  useEffect(() => {
    const loadInitial = async () => {
      const liveSessions = await getLiveSessions()
      const liveAlerts = await getAlerts(false)
      useAuditStore.setState({
        sessions: liveSessions,
        alerts: liveAlerts,
      })
    }
    loadInitial()
  }, [])

  const activeSessions = sessions.filter((s) => !s.ended_at)
  const toolsSet = new Set(activeSessions.map((s) => s.tool_id))
  const activeToolCount = toolsSet.size

  const allSamples = Array.from(samples.values()).flat()
  const avgCpu = allSamples.length
    ? (allSamples.reduce((sum, s) => sum + s.cpu_pct, 0) / allSamples.length).toFixed(1)
    : 0

  const avgMem = allSamples.length
    ? (
        allSamples.reduce((sum, s) => sum + s.rss_bytes, 0) / allSamples.length /
        1024 /
        1024
      ).toFixed(0)
    : 0

  const recentAlerts = alerts.filter((a) => !a.dismissed).slice(0, 5)

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-gray-900">Dashboard</h2>
      <p className="text-gray-600 mt-2">Active tools, CPU/memory usage, recent alerts</p>

      <div className="mt-8 grid grid-cols-3 gap-6">
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Active Tools</h3>
          <p className="text-3xl font-bold text-gray-900 mt-2">{activeToolCount}</p>
          <p className="text-xs text-gray-400 mt-2">{activeSessions.length} sessions</p>
        </div>
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">CPU Usage</h3>
          <p className="text-3xl font-bold text-gray-900 mt-2">{avgCpu}%</p>
          <div className="mt-4 h-12">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={allSamples.slice(-60)}>
                <Line
                  type="monotone"
                  dataKey="cpu_pct"
                  stroke="#3b82f6"
                  dot={false}
                  isAnimationActive={false}
                  strokeWidth={1}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Memory Usage</h3>
          <p className="text-3xl font-bold text-gray-900 mt-2">{avgMem} MB</p>
          <div className="mt-4 h-12">
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
                  strokeWidth={1}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>

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
                  <p className="text-sm text-gray-600 mt-1">{alert.detail}</p>
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
