import { useEffect, useState, useMemo } from 'react'
import { useAuditStore } from '../store/auditStore'
import { useAuditStream } from '../hooks/useAuditStream'
import { getLiveSessions } from '../lib/tauri'

type SortField = 'tool_id' | 'started_at' | 'duration'
type SortOrder = 'asc' | 'desc'

export default function Sessions() {
  const { sessions } = useAuditStore()
  const [filterTool, setFilterTool] = useState('')
  const [sortField, setSortField] = useState<SortField>('started_at')
  const [sortOrder, setSortOrder] = useState<SortOrder>('desc')
  const [refreshing, setRefreshing] = useState(false)
  useAuditStream()

  const refresh = async () => {
    setRefreshing(true)
    try {
      const live = await getLiveSessions()
      useAuditStore.setState({ sessions: live })
    } finally {
      setRefreshing(false)
    }
  }

  useEffect(() => {
    refresh()
  }, [])

  const filteredAndSorted = useMemo(() => {
    let result = sessions
    if (filterTool) {
      result = result.filter((s) => s.tool_id.includes(filterTool))
    }

    return result.sort((a, b) => {
      let aVal: number | string
      let bVal: number | string

      if (sortField === 'tool_id') {
        aVal = a.tool_id
        bVal = b.tool_id
      } else if (sortField === 'started_at') {
        aVal = a.started_at
        bVal = b.started_at
      } else {
        const aDuration = (a.ended_at || Date.now()) - a.started_at
        const bDuration = (b.ended_at || Date.now()) - b.started_at
        aVal = aDuration
        bVal = bDuration
      }

      if (typeof aVal === 'string') {
        return sortOrder === 'asc'
          ? aVal.localeCompare(bVal as string)
          : (bVal as string).localeCompare(aVal)
      } else {
        return sortOrder === 'asc'
          ? (aVal as number) - (bVal as number)
          : (bVal as number) - (aVal as number)
      }
    })
  }, [sessions, filterTool, sortField, sortOrder])

  const toggleSort = (field: SortField) => {
    if (sortField === field) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')
    } else {
      setSortField(field)
      setSortOrder('desc')
    }
  }

  const formatDuration = (ms: number) => {
    const seconds = Math.floor(ms / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)

    if (hours > 0) return `${hours}h ${minutes % 60}m`
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`
    return `${seconds}s`
  }

  const getSortIcon = (field: SortField) => {
    if (sortField !== field) return '⇅'
    return sortOrder === 'asc' ? '↑' : '↓'
  }

  return (
    <div className="p-8">
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900">Sessions</h2>
          <p className="text-gray-600 mt-2">Historical audit sessions by tool</p>
        </div>
        <button
          onClick={refresh}
          disabled={refreshing}
          className="flex items-center gap-2 px-4 py-2 text-sm border border-gray-300 rounded-lg bg-white hover:bg-gray-50 disabled:opacity-50"
        >
          <span className={refreshing ? 'animate-spin inline-block' : ''}>↻</span>
          {refreshing ? 'Refreshing…' : 'Refresh'}
        </button>
      </div>

      {sessions.length === 0 ? (
        <div className="mt-8 bg-white rounded-lg shadow p-8 text-center">
          <p className="text-gray-500">No sessions yet</p>
        </div>
      ) : (
        <div className="mt-8 space-y-4">
          <input
            type="text"
            placeholder="Filter by tool..."
            value={filterTool}
            onChange={(e) => setFilterTool(e.target.value)}
            className="px-4 py-2 border border-gray-300 rounded-lg w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
          />

          <div className="bg-white rounded-lg shadow overflow-hidden">
            <table className="w-full">
              <thead className="bg-gray-50 border-b">
                <tr>
                  <th className="px-6 py-3 text-left">
                    <button
                      onClick={() => toggleSort('tool_id')}
                      className="flex items-center gap-2 font-medium text-gray-700 hover:text-gray-900"
                    >
                      Tool {getSortIcon('tool_id')}
                    </button>
                  </th>
                  <th className="px-6 py-3 text-left font-medium text-gray-700">
                    PID
                  </th>
                  <th className="px-6 py-3 text-left font-medium text-gray-700">
                    Confidence
                  </th>
                  <th className="px-6 py-3 text-left">
                    <button
                      onClick={() => toggleSort('started_at')}
                      className="flex items-center gap-2 font-medium text-gray-700 hover:text-gray-900"
                    >
                      Started {getSortIcon('started_at')}
                    </button>
                  </th>
                  <th className="px-6 py-3 text-left">
                    <button
                      onClick={() => toggleSort('duration')}
                      className="flex items-center gap-2 font-medium text-gray-700 hover:text-gray-900"
                    >
                      Duration {getSortIcon('duration')}
                    </button>
                  </th>
                  <th className="px-6 py-3 text-left font-medium text-gray-700">
                    Status
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {filteredAndSorted.map((session) => {
                  const duration =
                    (session.ended_at || Date.now()) - session.started_at
                  return (
                    <tr key={session.id} className="hover:bg-gray-50">
                      <td className="px-6 py-4">
                        <span className="inline-block px-2 py-1 bg-blue-100 text-blue-700 rounded text-sm font-medium">
                          {session.tool_id}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-gray-900">{session.pid}</td>
                      <td className="px-6 py-4">
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
                      <td className="px-6 py-4 text-sm text-gray-600">
                        {new Date(session.started_at).toLocaleString()}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-600">
                        {formatDuration(duration)}
                      </td>
                      <td className="px-6 py-4">
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
      )}
    </div>
  )
}
