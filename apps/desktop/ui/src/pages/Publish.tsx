import { useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import {
  pushWithGuardrail,
  executePush,
  GuardrailResult,
} from '../lib/tauri'

export default function Publish() {
  const [repoPath, setRepoPath] = useState<string>('')
  const [remote, setRemote] = useState('origin')
  const [refspec, setRefspec] = useState('HEAD:main')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<GuardrailResult | null>(null)
  const [overriddenRules, setOverriddenRules] = useState<Set<string>>(new Set())
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  const browseForRepo = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select git repository',
      })
      if (selected && typeof selected === 'string') {
        setRepoPath(selected)
      }
    } catch (e) {
      console.error(e)
    }
  }

  const handleScan = async () => {
    setError(null)
    setSuccess(null)
    setLoading(true)
    try {
      const scanResult = await pushWithGuardrail(remote, refspec, repoPath || undefined)
      setResult(scanResult)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  const toggleOverride = (ruleId: string) => {
    const newSet = new Set(overriddenRules)
    if (newSet.has(ruleId)) newSet.delete(ruleId)
    else newSet.add(ruleId)
    setOverriddenRules(newSet)
  }

  const canPush =
    result &&
    (result.allowed ||
      (result.findings &&
        result.findings.every(
          (f) => f.severity !== 'high' && overriddenRules.has(f.rule_id)
        )))

  const handleConfirmPush = async () => {
    setError(null)
    setLoading(true)
    try {
      await executePush(remote, refspec, repoPath || undefined)
      setSuccess(`Pushed ${refspec} to ${remote}`)
      setResult(null)
      setOverriddenRules(new Set())
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  const reset = () => {
    setResult(null)
    setOverriddenRules(new Set())
    setError(null)
    setSuccess(null)
  }

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-gray-900">Publish with Guardrail</h2>
      <p className="text-gray-600 mt-2">
        Scan staged changes for secrets before pushing — only allows{' '}
        <code className="text-sm bg-gray-100 px-1 rounded">masudjbd/*</code> and{' '}
        <code className="text-sm bg-gray-100 px-1 rounded">fahiminfo/*</code>
      </p>

      <div className="mt-8 bg-white rounded-lg shadow p-6">
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Repository (defaults to current directory)
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={repoPath}
                onChange={(e) => setRepoPath(e.target.value)}
                disabled={result !== null}
                placeholder="/path/to/repo (or leave empty for cwd)"
                className="flex-1 px-4 py-2 border rounded-lg disabled:bg-gray-100 font-mono text-sm"
              />
              <button
                onClick={browseForRepo}
                disabled={result !== null}
                className="px-4 py-2 border border-gray-300 rounded-lg bg-gray-50 hover:bg-gray-100 disabled:opacity-50"
              >
                📁 Browse
              </button>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Remote
              </label>
              <input
                type="text"
                value={remote}
                onChange={(e) => setRemote(e.target.value)}
                disabled={result !== null}
                placeholder="origin"
                className="w-full px-4 py-2 border rounded-lg disabled:bg-gray-100"
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
                placeholder="HEAD:main"
                className="w-full px-4 py-2 border rounded-lg disabled:bg-gray-100"
              />
            </div>
          </div>
        </div>

        {result === null && (
          <button
            onClick={handleScan}
            disabled={loading}
            className="mt-6 w-full bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Scanning…' : 'Scan for Secrets'}
          </button>
        )}
      </div>

      {error && (
        <div className="mt-6 bg-red-50 border border-red-200 rounded-lg p-4">
          <p className="text-sm font-medium text-red-900">Error</p>
          <p className="text-sm text-red-700 mt-1 break-all">{error}</p>
        </div>
      )}

      {success && (
        <div className="mt-6 bg-green-50 border border-green-200 rounded-lg p-4">
          <p className="text-lg font-semibold text-green-900">✓ {success}</p>
          <button
            onClick={reset}
            className="mt-2 text-sm text-green-700 hover:text-green-800 underline"
          >
            Scan again
          </button>
        </div>
      )}

      {result && (
        <div className="mt-8">
          {result.allowed && !result.findings ? (
            <div className="bg-green-50 border border-green-200 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-green-900">
                ✓ No secrets detected
              </h3>
              <p className="text-green-700 mt-2">
                Staged changes are clean. Safe to publish.
              </p>
              <div className="flex gap-3 mt-4">
                <button
                  onClick={handleConfirmPush}
                  disabled={loading}
                  className="bg-green-600 text-white px-6 py-2 rounded-lg font-medium hover:bg-green-700 disabled:opacity-50"
                >
                  {loading ? 'Pushing…' : 'Confirm Push'}
                </button>
                <button
                  onClick={reset}
                  className="px-6 py-2 text-gray-600 hover:bg-gray-100 rounded-lg"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <div className="bg-red-50 border border-red-200 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-red-900">
                ⚠ Secrets detected
              </h3>
              <div className="mt-4 space-y-3">
                {result.findings?.map((f, idx) => (
                  <div
                    key={`${f.rule_id}-${idx}`}
                    className={`p-4 rounded border ${
                      f.severity === 'high'
                        ? 'bg-red-100 border-red-300'
                        : f.severity === 'medium'
                          ? 'bg-yellow-100 border-yellow-300'
                          : 'bg-blue-100 border-blue-300'
                    }`}
                  >
                    <div className="flex items-start gap-4">
                      <div className="flex-1">
                        <div className="flex items-center gap-2">
                          <span
                            className={`px-2 py-0.5 rounded text-xs font-bold ${
                              f.severity === 'high'
                                ? 'bg-red-600 text-white'
                                : f.severity === 'medium'
                                  ? 'bg-yellow-600 text-white'
                                  : 'bg-blue-600 text-white'
                            }`}
                          >
                            {f.severity.toUpperCase()}
                          </span>
                          <p className="font-medium text-gray-900">{f.rule_id}</p>
                        </div>
                        <p className="text-sm text-gray-600 mt-1 font-mono">
                          {f.file}:{f.line}
                        </p>
                        <p className="text-xs text-gray-500 mt-2 font-mono">
                          {f.redacted_value}
                        </p>
                      </div>
                      {f.severity !== 'high' && (
                        <label className="flex items-center gap-2 cursor-pointer flex-shrink-0">
                          <input
                            type="checkbox"
                            checked={overriddenRules.has(f.rule_id)}
                            onChange={() => toggleOverride(f.rule_id)}
                            className="w-4 h-4 rounded"
                          />
                          <span className="text-sm text-gray-700">Override</span>
                        </label>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              {result.findings?.some((f) => f.severity === 'high') ? (
                <div className="mt-4 p-3 bg-red-100 border border-red-300 rounded">
                  <p className="text-sm font-medium text-red-900">
                    ⛔ High-severity secrets cannot be overridden.
                  </p>
                  <p className="text-xs text-red-700 mt-1">
                    Remove these from your staged diff (e.g.,{' '}
                    <code>git restore --staged file</code>), rotate any leaked
                    credentials, then scan again.
                  </p>
                </div>
              ) : (
                <button
                  onClick={handleConfirmPush}
                  disabled={loading || !canPush}
                  className="mt-6 w-full bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {loading
                    ? 'Pushing…'
                    : `Push with ${overriddenRules.size} Override${overriddenRules.size === 1 ? '' : 's'}`}
                </button>
              )}

              <button
                onClick={reset}
                className="mt-3 w-full text-gray-600 px-6 py-2 rounded-lg hover:bg-gray-100"
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
