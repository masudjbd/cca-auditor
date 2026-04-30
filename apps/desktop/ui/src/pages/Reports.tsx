import { useEffect, useMemo, useState } from 'react'
import { save } from '@tauri-apps/plugin-dialog'
import { useAuditStore } from '../store/auditStore'
import {
  generateReport,
  getLiveSessions,
  saveReportToFile,
  openPathInFinder,
} from '../lib/tauri'

export default function Reports() {
  const { sessions } = useAuditStore()
  const [selectedSessions, setSelectedSessions] = useState<string[]>([])
  const [format, setFormat] = useState<'html' | 'pdf' | 'markdown' | 'json'>('html')
  const [loading, setLoading] = useState(false)
  const [filterTool, setFilterTool] = useState<string>('all')
  const [lastSavedPath, setLastSavedPath] = useState<string | null>(null)

  useEffect(() => {
    getLiveSessions().then((live) =>
      useAuditStore.setState({ sessions: live })
    )
  }, [])

  const toolList = useMemo(() => {
    const set = new Set(sessions.map((s) => s.tool_id))
    return Array.from(set).sort()
  }, [sessions])

  const filteredSessions = useMemo(
    () =>
      sessions.filter(
        (s) => filterTool === 'all' || s.tool_id === filterTool
      ),
    [sessions, filterTool]
  )

  const toggleSession = (sessionId: string) => {
    setSelectedSessions((prev) =>
      prev.includes(sessionId)
        ? prev.filter((id) => id !== sessionId)
        : [...prev, sessionId]
    )
  }

  const selectAll = () => {
    setSelectedSessions(filteredSessions.map((s) => s.id))
  }

  const clearSelection = () => {
    setSelectedSessions([])
  }

  const handleGenerateReport = async () => {
    if (selectedSessions.length === 0) {
      alert('Please select at least one session')
      return
    }

    setLoading(true)
    setLastSavedPath(null)
    try {
      const content = await generateReport({
        session_ids: selectedSessions,
        format,
      })

      const fileExtensions: Record<string, string> = {
        html: 'html',
        pdf: 'html', // PDF generation falls back to HTML for now
        markdown: 'md',
        json: 'json',
      }
      const ext = fileExtensions[format]
      const defaultName = `cca-audit-${new Date().toISOString().slice(0, 10)}.${ext}`

      // Open native save dialog
      const filePath = await save({
        defaultPath: defaultName,
        filters: [
          {
            name: format.toUpperCase(),
            extensions: [ext],
          },
        ],
      })

      if (!filePath) {
        // User cancelled
        return
      }

      await saveReportToFile(filePath, content)
      setLastSavedPath(filePath)
    } catch (error) {
      console.error('Failed to generate report:', error)
      alert(`Failed to generate report: ${error}`)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-gray-900">Generate Report</h2>
      <p className="text-gray-600 mt-2">
        Export audit sessions as HTML, Markdown, or JSON
      </p>

      <div className="mt-8 grid grid-cols-3 gap-8">
        {/* Sessions */}
        <div className="col-span-2">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900">
              Select Sessions ({selectedSessions.length}/{filteredSessions.length})
            </h3>
            <div className="flex gap-2">
              <select
                value={filterTool}
                onChange={(e) => setFilterTool(e.target.value)}
                className="px-3 py-1.5 border border-gray-300 rounded text-sm bg-white"
              >
                <option value="all">All tools</option>
                {toolList.map((t) => (
                  <option key={t} value={t}>{t}</option>
                ))}
              </select>
              <button
                onClick={selectAll}
                className="text-sm px-3 py-1.5 border border-gray-300 rounded hover:bg-gray-50"
              >
                Select All
              </button>
              <button
                onClick={clearSelection}
                className="text-sm px-3 py-1.5 border border-gray-300 rounded hover:bg-gray-50"
              >
                Clear
              </button>
            </div>
          </div>

          {filteredSessions.length === 0 ? (
            <div className="bg-white rounded-lg shadow p-8 text-center text-gray-500">
              No sessions available
            </div>
          ) : (
            <div className="bg-white rounded-lg shadow max-h-[500px] overflow-y-auto">
              <div className="divide-y">
                {filteredSessions.map((session) => (
                  <label
                    key={session.id}
                    className="flex items-center gap-3 p-3 cursor-pointer hover:bg-gray-50"
                  >
                    <input
                      type="checkbox"
                      checked={selectedSessions.includes(session.id)}
                      onChange={() => toggleSession(session.id)}
                      className="w-4 h-4 rounded border-gray-300"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between">
                        <p className="font-medium text-gray-900">{session.tool_id}</p>
                        <span
                          className={`px-2 py-0.5 rounded text-xs ${
                            !session.ended_at
                              ? 'bg-green-100 text-green-700'
                              : 'bg-gray-100 text-gray-700'
                          }`}
                        >
                          {!session.ended_at ? 'Active' : 'Closed'}
                        </span>
                      </div>
                      <p className="text-sm text-gray-500 truncate">
                        PID {session.pid} • {new Date(session.started_at).toLocaleString()}
                      </p>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Format + Action */}
        <div>
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Format</h3>
          <div className="space-y-2">
            {([
              { fmt: 'html', label: 'HTML', desc: 'Styled web report (recommended)' },
              { fmt: 'markdown', label: 'Markdown', desc: 'Plain text for sharing' },
              { fmt: 'json', label: 'JSON', desc: 'Raw data export' },
              { fmt: 'pdf', label: 'PDF (HTML)', desc: 'Print-ready HTML' },
            ] as const).map((opt) => (
              <label
                key={opt.fmt}
                className={`flex items-start gap-3 p-3 border rounded-lg cursor-pointer transition-colors ${
                  format === opt.fmt
                    ? 'border-blue-500 bg-blue-50'
                    : 'border-gray-200 hover:bg-gray-50'
                }`}
              >
                <input
                  type="radio"
                  name="format"
                  value={opt.fmt}
                  checked={format === opt.fmt}
                  onChange={(e) => setFormat(e.target.value as typeof format)}
                  className="w-4 h-4 mt-0.5 rounded-full border-gray-300"
                />
                <div>
                  <p className="font-medium text-gray-900">{opt.label}</p>
                  <p className="text-xs text-gray-500 mt-0.5">{opt.desc}</p>
                </div>
              </label>
            ))}
          </div>

          <button
            onClick={handleGenerateReport}
            disabled={loading || selectedSessions.length === 0}
            className="mt-6 w-full bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {loading ? 'Generating...' : `Generate (${selectedSessions.length})`}
          </button>

          {lastSavedPath && (
            <div className="mt-4 p-3 bg-green-50 border border-green-200 rounded-lg">
              <p className="text-sm font-medium text-green-900">✓ Saved!</p>
              <p className="text-xs text-green-700 mt-1 break-all">{lastSavedPath}</p>
              <button
                onClick={() => openPathInFinder(lastSavedPath)}
                className="mt-2 text-sm text-green-700 hover:text-green-800 underline"
              >
                Show in Finder
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
