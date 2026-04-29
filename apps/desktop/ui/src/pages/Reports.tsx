import { useState } from 'react'
import { useAuditStore } from '../store/auditStore'
import { generateReport } from '../lib/tauri'

export default function Reports() {
  const { sessions } = useAuditStore()
  const [selectedSessions, setSelectedSessions] = useState<string[]>([])
  const [format, setFormat] = useState<'html' | 'pdf' | 'markdown' | 'json'>('html')
  const [loading, setLoading] = useState(false)

  const toggleSession = (sessionId: string) => {
    setSelectedSessions((prev) =>
      prev.includes(sessionId)
        ? prev.filter((id) => id !== sessionId)
        : [...prev, sessionId]
    )
  }

  const handleGenerateReport = async () => {
    if (selectedSessions.length === 0) {
      alert('Please select at least one session')
      return
    }

    setLoading(true)
    try {
      const result = await generateReport({
        session_ids: selectedSessions,
        format,
      })

      const fileExtensions: Record<string, string> = {
        html: 'html',
        pdf: 'pdf',
        markdown: 'md',
        json: 'json',
      }
      const ext = fileExtensions[format]
      const filename = `cca-audit-report-${Date.now()}.${ext}`

      const element = document.createElement('a')
      element.setAttribute('href', `data:text/plain;charset=utf-8,${encodeURIComponent(result)}`)
      element.setAttribute('download', filename)
      element.style.display = 'none'
      document.body.appendChild(element)
      element.click()
      document.body.removeChild(element)
    } catch (error) {
      console.error('Failed to generate report:', error)
      alert('Failed to generate report')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-gray-900">Generate Report</h2>
      <p className="text-gray-600 mt-2">Export audit session as HTML/PDF/Markdown/JSON</p>

      <div className="mt-8 grid grid-cols-2 gap-8">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Select Sessions</h3>
          {sessions.length === 0 ? (
            <p className="text-gray-500">No sessions available</p>
          ) : (
            <div className="space-y-2">
              {sessions.map((session) => (
                <label
                  key={session.id}
                  className="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-gray-50"
                >
                  <input
                    type="checkbox"
                    checked={selectedSessions.includes(session.id)}
                    onChange={() => toggleSession(session.id)}
                    className="w-4 h-4 rounded border-gray-300"
                  />
                  <div className="flex-1">
                    <p className="font-medium text-gray-900">{session.tool_id}</p>
                    <p className="text-sm text-gray-500">
                      PID {session.pid} — {new Date(session.started_at).toLocaleString()}
                    </p>
                    <p className="text-xs text-gray-400 mt-1">
                      Confidence: {session.confidence}
                    </p>
                  </div>
                </label>
              ))}
            </div>
          )}
        </div>

        <div>
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Format</h3>
          <div className="space-y-3">
            {(['html', 'pdf', 'markdown', 'json'] as const).map((fmt) => (
              <label
                key={fmt}
                className="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-gray-50"
              >
                <input
                  type="radio"
                  name="format"
                  value={fmt}
                  checked={format === fmt}
                  onChange={(e) => setFormat(e.target.value as typeof format)}
                  className="w-4 h-4 rounded-full border-gray-300"
                />
                <div>
                  <p className="font-medium text-gray-900 capitalize">{fmt}</p>
                  <p className="text-sm text-gray-500">
                    {fmt === 'html' && 'Browser-friendly web format'}
                    {fmt === 'pdf' && 'Printable document'}
                    {fmt === 'markdown' && 'Text format for sharing'}
                    {fmt === 'json' && 'Raw data export'}
                  </p>
                </div>
              </label>
            ))}
          </div>

          <button
            onClick={handleGenerateReport}
            disabled={loading || selectedSessions.length === 0}
            className="mt-8 w-full bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {loading ? 'Generating...' : 'Generate Report'}
          </button>
        </div>
      </div>
    </div>
  )
}
