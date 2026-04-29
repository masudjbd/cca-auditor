import { useState } from 'react'
import { pushWithGuardrail, GuardrailResult } from '../lib/tauri'

export default function Publish() {
  const [remote, setRemote] = useState('origin')
  const [refspec, setRefspec] = useState('HEAD:main')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<GuardrailResult | null>(null)
  const [overriddenRules, setOverriddenRules] = useState<Set<string>>(new Set())

  const handleScan = async () => {
    setLoading(true)
    try {
      const scanResult = await pushWithGuardrail(remote, refspec)
      setResult(scanResult)
    } catch (error) {
      console.error('Scan failed:', error)
      alert('Failed to scan for secrets')
    } finally {
      setLoading(false)
    }
  }

  const toggleOverride = (ruleId: string) => {
    const newSet = new Set(overriddenRules)
    if (newSet.has(ruleId)) {
      newSet.delete(ruleId)
    } else {
      newSet.add(ruleId)
    }
    setOverriddenRules(newSet)
  }

  const canPush = result && (
    result.allowed ||
    (result.findings && result.findings.every((f) => overriddenRules.has(f.rule_id)))
  )

  const handlePush = async () => {
    if (!canPush) {
      alert('Cannot push: high-severity secrets detected')
      return
    }
    setLoading(true)
    try {
      await pushWithGuardrail(remote, refspec)
      alert('Push successful!')
      setResult(null)
      setOverriddenRules(new Set())
    } catch (error) {
      console.error('Push failed:', error)
      alert('Push failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-gray-900">Publish with Guardrail</h2>
      <p className="text-gray-600 mt-2">Scan staged changes for secrets before push</p>

      <div className="mt-8 bg-white rounded-lg shadow p-6">
        <div className="grid grid-cols-2 gap-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Remote
            </label>
            <input
              type="text"
              value={remote}
              onChange={(e) => setRemote(e.target.value)}
              disabled={result !== null}
              className="w-full px-4 py-2 border rounded-lg disabled:bg-gray-100"
              placeholder="origin"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Refspec
            </label>
            <input
              type="text"
              value={refspec}
              onChange={(e) => setRefspec(e.target.value)}
              disabled={result !== null}
              className="w-full px-4 py-2 border rounded-lg disabled:bg-gray-100"
              placeholder="HEAD:main"
            />
          </div>
        </div>

        {result === null && (
          <button
            onClick={handleScan}
            disabled={loading}
            className="mt-6 w-full bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
          >
            {loading ? 'Scanning...' : 'Scan for Secrets'}
          </button>
        )}
      </div>

      {result && (
        <div className="mt-8">
          {result.allowed ? (
            <div className="bg-green-50 border border-green-200 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-green-900">✓ No secrets detected</h3>
              <p className="text-green-700 mt-2">Safe to push</p>
              <button
                onClick={handlePush}
                disabled={loading}
                className="mt-4 bg-green-600 text-white px-6 py-2 rounded-lg font-medium hover:bg-green-700 disabled:opacity-50 transition-colors"
              >
                {loading ? 'Pushing...' : 'Push Now'}
              </button>
            </div>
          ) : (
            <div className="bg-red-50 border border-red-200 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-red-900">⚠ Secrets detected</h3>
              <div className="mt-4 space-y-3">
                {result.findings?.map((finding) => (
                  <div
                    key={finding.rule_id}
                    className={`p-4 rounded border ${
                      finding.severity === 'high'
                        ? 'bg-red-100 border-red-300'
                        : finding.severity === 'medium'
                          ? 'bg-yellow-100 border-yellow-300'
                          : 'bg-blue-100 border-blue-300'
                    }`}
                  >
                    <div className="flex items-start gap-4">
                      <div className="flex-1">
                        <p className="font-medium text-gray-900">
                          {finding.rule_id}: {finding.rule_id}
                        </p>
                        <p className="text-sm text-gray-600 mt-1">
                          {finding.file}:{finding.line}
                        </p>
                        <p className="text-xs text-gray-500 mt-2 font-mono">
                          {finding.redacted_value}
                        </p>
                      </div>
                      {finding.severity !== 'high' && (
                        <label className="flex items-center gap-2 cursor-pointer">
                          <input
                            type="checkbox"
                            checked={overriddenRules.has(finding.rule_id)}
                            onChange={() => toggleOverride(finding.rule_id)}
                            className="w-4 h-4 rounded"
                          />
                          <span className="text-sm text-gray-600">Override</span>
                        </label>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              {result.findings?.some((f) => f.severity === 'high') ? (
                <p className="mt-4 text-sm text-red-700">
                  High-severity secrets cannot be overridden. Fix these before pushing.
                </p>
              ) : (
                <button
                  onClick={handlePush}
                  disabled={loading || !canPush}
                  className="mt-6 w-full bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
                >
                  {loading ? 'Pushing...' : 'Push with Overrides'}
                </button>
              )}

              <button
                onClick={() => setResult(null)}
                className="mt-4 w-full text-gray-600 px-6 py-2 rounded-lg font-medium hover:bg-gray-100 transition-colors"
              >
                Cancel
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
