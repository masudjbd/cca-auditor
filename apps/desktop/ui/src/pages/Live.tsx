import { useEffect, useRef } from 'react'
import { useAuditStore, AuditEvent } from '../store/auditStore'
import { useAuditStream } from '../hooks/useAuditStream'
import { getEvents } from '../lib/tauri'

export default function Live() {
  const { events, sessions } = useAuditStore()
  const scrollRef = useRef<HTMLDivElement>(null)
  useAuditStream()

  useEffect(() => {
    const loadInitial = async () => {
      const firstSession = sessions[0]
      if (firstSession) {
        const sessionEvents = await getEvents(firstSession.id, 1000)
        useAuditStore.setState({ events: sessionEvents })
      }
    }
    if (sessions.length === 0) {
      loadInitial()
    }
  }, [sessions])

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [events])

  const getEventIcon = (kind: AuditEvent['kind']) => {
    switch (kind) {
      case 'FsRead':
        return '📖'
      case 'FsWrite':
        return '✏️'
      case 'FsDelete':
        return '🗑️'
      case 'NetConnect':
        return '🌐'
      case 'ProcessSpawn':
        return '⚙️'
      case 'LocalArtifact':
        return '📦'
    }
  }

  const getKindLabel = (kind: AuditEvent['kind']) => {
    return kind.replace(/([A-Z])/g, ' $1').trim()
  }

  return (
    <div className="p-8 flex flex-col h-full">
      <h2 className="text-3xl font-bold text-gray-900">Live Audit Stream</h2>
      <p className="text-gray-600 mt-2">Real-time events: file access, network connections, subprocess execution</p>

      <div className="mt-8 flex-1 bg-white rounded-lg shadow overflow-hidden flex flex-col">
        <div
          ref={scrollRef}
          className="flex-1 overflow-y-auto font-mono text-sm"
        >
          {events.length === 0 ? (
            <div className="flex items-center justify-center h-64 text-gray-500">
              No events yet
            </div>
          ) : (
            <table className="w-full">
              <thead className="bg-gray-50 border-b sticky top-0">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Time</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Tool</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Type</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Details</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Confidence</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {events.map((event) => (
                  <tr
                    key={event.id}
                    className="hover:bg-gray-50 transition-colors"
                  >
                    <td className="px-6 py-3 whitespace-nowrap text-xs text-gray-500">
                      {new Date(event.timestamp).toLocaleTimeString()}
                    </td>
                    <td className="px-6 py-3 whitespace-nowrap">
                      <span className="inline-block px-2 py-1 bg-blue-100 text-blue-700 rounded text-xs">
                        {event.tool_id}
                      </span>
                    </td>
                    <td className="px-6 py-3 whitespace-nowrap">
                      <span className="text-base">
                        {getEventIcon(event.kind)} {getKindLabel(event.kind)}
                      </span>
                    </td>
                    <td className="px-6 py-3 text-gray-700 max-w-md truncate">
                      {event.path || event.dest_addr || event.kind}
                    </td>
                    <td className="px-6 py-3 whitespace-nowrap text-xs">
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
