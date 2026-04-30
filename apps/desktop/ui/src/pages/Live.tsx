import { useEffect, useMemo, useRef, useState } from 'react'
import { useAuditStore, AuditEvent } from '../store/auditStore'
import { useAuditStream } from '../hooks/useAuditStream'
import { getEvents, getLiveSessions } from '../lib/tauri'

export default function Live() {
  const { events, sessions } = useAuditStore()
  const scrollRef = useRef<HTMLDivElement>(null)
  const [filterTool, setFilterTool] = useState<string>('all')
  const [filterKind, setFilterKind] = useState<string>('all')
  const [autoScroll, setAutoScroll] = useState(true)
  useAuditStream()

  // Initial load: fetch all events for all active sessions
  useEffect(() => {
    const loadAll = async () => {
      const live = await getLiveSessions()
      useAuditStore.setState({ sessions: live })
      const allEvents: AuditEvent[] = []
      for (const session of live.slice(0, 10)) {
        const sessionEvents = await getEvents(session.id, 200)
        allEvents.push(...sessionEvents)
      }
      // Sort newest first
      allEvents.sort((a, b) => b.timestamp - a.timestamp)
      useAuditStore.setState({ events: allEvents.slice(0, 1000) })
    }
    loadAll()
  }, [])

  // Auto-scroll to bottom on new events
  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [events, autoScroll])

  // Detect manual scroll up to disable auto-scroll
  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const el = e.currentTarget
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 50
    if (atBottom !== autoScroll) {
      setAutoScroll(atBottom)
    }
  }

  const uniqueTools = useMemo(() => {
    const set = new Set(events.map((e) => e.tool_id))
    sessions.forEach((s) => set.add(s.tool_id))
    return Array.from(set).sort()
  }, [events, sessions])

  const filteredEvents = useMemo(() => {
    return events.filter((e) => {
      if (filterTool !== 'all' && e.tool_id !== filterTool) return false
      if (filterKind !== 'all' && e.kind !== filterKind) return false
      return true
    })
  }, [events, filterTool, filterKind])

  const getEventIcon = (kind: AuditEvent['kind']) => {
    switch (kind) {
      case 'FsRead': return '📖'
      case 'FsWrite': return '✏️'
      case 'FsDelete': return '🗑️'
      case 'NetConnect': return '🌐'
      case 'ProcessSpawn': return '⚙️'
      case 'LocalArtifact': return '📦'
    }
  }

  const getKindLabel = (kind: AuditEvent['kind']) =>
    kind.replace(/([A-Z])/g, ' $1').trim()

  return (
    <div className="p-8 flex flex-col h-full">
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900">Live Audit Stream</h2>
          <p className="text-gray-600 mt-2">
            Real-time events: file access, network connections, subprocess execution
          </p>
        </div>
        <div className="flex items-center gap-2 mt-2">
          <span
            className={`inline-block w-2 h-2 rounded-full ${
              autoScroll ? 'bg-green-500 animate-pulse' : 'bg-gray-400'
            }`}
          />
          <span className="text-sm text-gray-600">
            {autoScroll ? 'Live' : 'Paused (scroll to bottom to resume)'}
          </span>
        </div>
      </div>

      {/* Filters */}
      <div className="mt-6 flex items-center gap-3">
        <select
          value={filterTool}
          onChange={(e) => setFilterTool(e.target.value)}
          className="px-3 py-2 border border-gray-300 rounded text-sm bg-white"
        >
          <option value="all">All tools ({events.length})</option>
          {uniqueTools.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>
        <select
          value={filterKind}
          onChange={(e) => setFilterKind(e.target.value)}
          className="px-3 py-2 border border-gray-300 rounded text-sm bg-white"
        >
          <option value="all">All event types</option>
          <option value="FsRead">📖 File Read</option>
          <option value="FsWrite">✏️ File Write</option>
          <option value="FsDelete">🗑️ File Delete</option>
          <option value="NetConnect">🌐 Network Connect</option>
          <option value="ProcessSpawn">⚙️ Process Spawn</option>
          <option value="LocalArtifact">📦 Local Artifact</option>
        </select>
        <span className="text-sm text-gray-500 ml-2">
          Showing {filteredEvents.length} of {events.length}
        </span>
        {(filterTool !== 'all' || filterKind !== 'all') && (
          <button
            onClick={() => {
              setFilterTool('all')
              setFilterKind('all')
            }}
            className="text-sm text-blue-600 hover:text-blue-700 underline"
          >
            Clear filters
          </button>
        )}
      </div>

      {/* Stream */}
      <div className="mt-6 flex-1 bg-white rounded-lg shadow overflow-hidden flex flex-col min-h-0">
        <div
          ref={scrollRef}
          onScroll={handleScroll}
          className="flex-1 overflow-y-auto font-mono text-sm"
        >
          {filteredEvents.length === 0 ? (
            <div className="flex items-center justify-center h-64 text-gray-500 flex-col gap-2">
              <span className="text-4xl">📡</span>
              <p>{events.length === 0 ? 'Waiting for events...' : 'No events match filters'}</p>
            </div>
          ) : (
            <table className="w-full">
              <thead className="bg-gray-50 border-b sticky top-0 z-10">
                <tr>
                  <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                    Time
                  </th>
                  <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                    Tool
                  </th>
                  <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                    Type
                  </th>
                  <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                    Details
                  </th>
                  <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                    Confidence
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {filteredEvents.map((event) => (
                  <tr key={event.id} className="hover:bg-blue-50 transition-colors">
                    <td className="px-4 py-2 whitespace-nowrap text-xs text-gray-500">
                      {new Date(event.timestamp).toLocaleTimeString()}
                    </td>
                    <td className="px-4 py-2 whitespace-nowrap">
                      <span className="inline-block px-2 py-1 bg-blue-100 text-blue-700 rounded text-xs">
                        {event.tool_id}
                      </span>
                    </td>
                    <td className="px-4 py-2 whitespace-nowrap">
                      <span className="text-sm">
                        {getEventIcon(event.kind)} {getKindLabel(event.kind)}
                      </span>
                    </td>
                    <td className="px-4 py-2 text-gray-700 max-w-md truncate">
                      {event.path ||
                        (event.dest_addr ? `${event.dest_addr}:${event.dest_port}` : event.kind)}
                    </td>
                    <td className="px-4 py-2 whitespace-nowrap text-xs">
                      <span
                        className={`px-2 py-1 rounded ${
                          event.confidence === 'High'
                            ? 'bg-green-100 text-green-700'
                            : event.confidence === 'Ambiguous'
                              ? 'bg-yellow-100 text-yellow-700'
                              : 'bg-blue-100 text-blue-700'
                        }`}
                      >
                        {event.confidence}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  )
}
