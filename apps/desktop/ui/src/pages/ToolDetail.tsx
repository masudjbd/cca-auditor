import { useEffect, useMemo, useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import {
  LineChart,
  Line,
  ResponsiveContainer,
  XAxis,
  YAxis,
  Tooltip,
  CartesianGrid,
} from 'recharts'
import { useAuditStore, AuditEvent, ResourceSample } from '../store/auditStore'
import { useAuditStream } from '../hooks/useAuditStream'
import { getEvents, getSamples, getLiveSessions } from '../lib/tauri'

export default function ToolDetail() {
  const { toolId } = useParams<{ toolId: string }>()
  const { sessions, samples } = useAuditStore()
  const [toolEvents, setToolEvents] = useState<AuditEvent[]>([])
  const [historicalSamples, setHistoricalSamples] = useState<ResourceSample[]>([])
  useAuditStream()

  const toolSessions = useMemo(
    () => sessions.filter((s) => s.tool_id === toolId),
    [sessions, toolId]
  )

  const activeSessions = toolSessions.filter((s) => !s.ended_at)

  // Load events for all sessions of this tool
  useEffect(() => {
    const loadData = async () => {
      const live = await getLiveSessions()
      useAuditStore.setState({ sessions: live })
      const matching = live.filter((s) => s.tool_id === toolId)
      const allEvents: AuditEvent[] = []
      for (const session of matching.slice(0, 20)) {
        const sessionEvents = await getEvents(session.id, 200)
        allEvents.push(...sessionEvents)
      }
      allEvents.sort((a, b) => b.timestamp - a.timestamp)
      setToolEvents(allEvents.slice(0, 500))

      // Fetch historical samples for active PIDs (last 1 hour)
      const now = Math.floor(Date.now() / 1000)
      const oneHourAgo = now - 3600
      const allHistoric: ResourceSample[] = []
      for (const session of matching.filter((s) => !s.ended_at).slice(0, 5)) {
        const histSamples = await getSamples(session.pid, oneHourAgo, now)
        allHistoric.push(...histSamples)
      }
      allHistoric.sort((a, b) => a.timestamp - b.timestamp)
      setHistoricalSamples(allHistoric)
    }
    loadData()
  }, [toolId])

  // Combine live samples with historical
  const allSamples = useMemo(() => {
    const liveSamples = activeSessions.flatMap((s) => samples.get(s.pid) ?? [])
    return [...historicalSamples, ...liveSamples]
      .sort((a, b) => a.timestamp - b.timestamp)
      .map((s) => ({
        ...s,
        rss_mb: s.rss_bytes / 1024 / 1024,
        time: new Date(s.timestamp).toLocaleTimeString(),
      }))
  }, [activeSessions, samples, historicalSamples])

  const totalEvents = toolEvents.length
  const eventsByKind = useMemo(() => {
    const map = new Map<string, number>()
    for (const e of toolEvents) {
      map.set(e.kind, (map.get(e.kind) ?? 0) + 1)
    }
    return map
  }, [toolEvents])

  if (toolSessions.length === 0) {
    return (
      <div className="p-8">
        <Link to="/" className="text-sm text-blue-600 hover:underline">
          ← Back to Dashboard
        </Link>
        <h2 className="text-3xl font-bold text-gray-900 mt-4">Tool: {toolId}</h2>
        <div className="mt-8 bg-white rounded-lg shadow p-12 text-center">
          <p className="text-gray-500">No sessions found for this tool</p>
        </div>
      </div>
    )
  }

  return (
    <div className="p-8">
      <Link to="/" className="text-sm text-blue-600 hover:underline">
        ← Back to Dashboard
      </Link>
      <div className="flex items-center gap-3 mt-4">
        <h2 className="text-3xl font-bold text-gray-900">{toolId}</h2>
        {activeSessions.length > 0 && (
          <span className="px-3 py-1 bg-green-100 text-green-700 rounded-full text-sm font-medium">
            ● Active
          </span>
        )}
      </div>

      {/* Stats */}
      <div className="mt-6 grid grid-cols-4 gap-4">
        <div className="bg-white rounded-lg shadow p-5">
          <p className="text-sm text-gray-500">Total Sessions</p>
          <p className="text-2xl font-bold text-gray-900">{toolSessions.length}</p>
          <p className="text-xs text-gray-400 mt-1">
            {activeSessions.length} active
          </p>
        </div>
        <div className="bg-white rounded-lg shadow p-5">
          <p className="text-sm text-gray-500">Total Events</p>
          <p className="text-2xl font-bold text-gray-900">{totalEvents}</p>
        </div>
        <div className="bg-white rounded-lg shadow p-5">
          <p className="text-sm text-gray-500">Avg CPU</p>
          <p className="text-2xl font-bold text-gray-900">
            {allSamples.length > 0
              ? (
                  allSamples.reduce((s, x) => s + x.cpu_pct, 0) / allSamples.length
                ).toFixed(1)
              : '—'}
            %
          </p>
        </div>
        <div className="bg-white rounded-lg shadow p-5">
          <p className="text-sm text-gray-500">Peak Memory</p>
          <p className="text-2xl font-bold text-gray-900">
            {allSamples.length > 0
              ? Math.round(
                  Math.max(...allSamples.map((s) => s.rss_bytes / 1024 / 1024))
                ).toString()
              : '—'}
            <span className="text-sm font-normal text-gray-500"> MB</span>
          </p>
        </div>
      </div>

      {/* CPU/Memory chart */}
      <div className="mt-8 bg-white rounded-lg shadow p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          Resource Usage (Last Hour)
        </h3>
        {allSamples.length === 0 ? (
          <p className="text-gray-400 py-12 text-center">
            No sample data yet. Wait a few seconds for live data.
          </p>
        ) : (
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={allSamples}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis
                  dataKey="time"
                  tick={{ fontSize: 11 }}
                  stroke="#9ca3af"
                  interval="preserveStartEnd"
                />
                <YAxis
                  yAxisId="cpu"
                  tick={{ fontSize: 11 }}
                  stroke="#3b82f6"
                  unit="%"
                />
                <YAxis
                  yAxisId="mem"
                  orientation="right"
                  tick={{ fontSize: 11 }}
                  stroke="#10b981"
                  unit=" MB"
                />
                <Tooltip />
                <Line
                  yAxisId="cpu"
                  type="monotone"
                  dataKey="cpu_pct"
                  stroke="#3b82f6"
                  dot={false}
                  isAnimationActive={false}
                  strokeWidth={2}
                  name="CPU"
                />
                <Line
                  yAxisId="mem"
                  type="monotone"
                  dataKey="rss_mb"
                  stroke="#10b981"
                  dot={false}
                  isAnimationActive={false}
                  strokeWidth={2}
                  name="Memory"
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        )}
      </div>

      {/* Event breakdown */}
      {eventsByKind.size > 0 && (
        <div className="mt-8 bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">
            Activity Breakdown
          </h3>
          <div className="grid grid-cols-3 gap-4">
            {Array.from(eventsByKind.entries()).map(([kind, count]) => (
              <div key={kind} className="bg-gray-50 rounded p-3">
                <p className="text-sm text-gray-600">{kind}</p>
                <p className="text-2xl font-bold text-gray-900">{count}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Sessions table */}
      <div className="mt-8 bg-white rounded-lg shadow overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">Sessions</h3>
        </div>
        <table className="w-full">
          <thead className="bg-gray-50 border-b">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                PID
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Started
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Duration
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Confidence
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Status
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {toolSessions.map((session) => {
              const duration = (session.ended_at ?? Date.now()) - session.started_at
              const seconds = Math.floor(duration / 1000)
              const minutes = Math.floor(seconds / 60)
              const formatted =
                minutes > 0
                  ? `${minutes}m ${seconds % 60}s`
                  : `${seconds}s`
              return (
                <tr key={session.id} className="hover:bg-gray-50">
                  <td className="px-6 py-3 font-mono text-sm">{session.pid}</td>
                  <td className="px-6 py-3 text-sm text-gray-600">
                    {new Date(session.started_at).toLocaleString()}
                  </td>
                  <td className="px-6 py-3 text-sm text-gray-600">{formatted}</td>
                  <td className="px-6 py-3">
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium ${
                        session.confidence === 'High'
                          ? 'bg-green-100 text-green-700'
                          : session.confidence === 'Ambiguous'
                            ? 'bg-yellow-100 text-yellow-700'
                            : 'bg-blue-100 text-blue-700'
                      }`}
                    >
                      {session.confidence}
                    </span>
                  </td>
                  <td className="px-6 py-3">
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium ${
                        session.ended_at
                          ? 'bg-gray-100 text-gray-700'
                          : 'bg-green-100 text-green-700'
                      }`}
                    >
                      {session.ended_at ? 'Closed' : 'Active'}
                    </span>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </div>
  )
}
