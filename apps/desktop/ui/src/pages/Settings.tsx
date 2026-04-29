import { useState } from 'react'

export default function Settings() {
  const [watchPaths, setWatchPaths] = useState<string[]>([
    '~',
    '/tmp',
  ])
  const [newPath, setNewPath] = useState('')
  const [enabledTools, setEnabledTools] = useState<Set<string>>(
    new Set([
      'cursor',
      'claude-code',
      'windsurf',
      'ollama',
      'lm-studio',
      'aider',
      'cline',
      'continue',
    ])
  )
  const [encryption, setEncryption] = useState(false)

  const tools = [
    'cursor',
    'claude-code',
    'windsurf',
    'ollama',
    'lm-studio',
    'aider',
    'cline',
    'continue',
    'copilot-chat',
    'tabnine',
    'supermaven',
  ]

  const addPath = () => {
    if (newPath && !watchPaths.includes(newPath)) {
      setWatchPaths([...watchPaths, newPath])
      setNewPath('')
    }
  }

  const removePath = (path: string) => {
    setWatchPaths(watchPaths.filter((p) => p !== path))
  }

  const toggleTool = (tool: string) => {
    const newSet = new Set(enabledTools)
    if (newSet.has(tool)) {
      newSet.delete(tool)
    } else {
      newSet.add(tool)
    }
    setEnabledTools(newSet)
  }

  return (
    <div className="p-8 max-w-2xl">
      <h2 className="text-3xl font-bold text-gray-900">Settings</h2>
      <p className="text-gray-600 mt-2">Configure watch paths, tool fingerprints, encryption</p>

      <div className="mt-8 space-y-8">
        {/* Watch Paths */}
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Watch Paths</h3>
          <p className="text-sm text-gray-600 mb-4">
            Directories to monitor for file system activity
          </p>

          <div className="space-y-2 mb-4">
            {watchPaths.map((path) => (
              <div
                key={path}
                className="flex items-center justify-between bg-gray-50 p-3 rounded"
              >
                <code className="text-sm text-gray-700">{path}</code>
                <button
                  onClick={() => removePath(path)}
                  className="text-sm text-red-600 hover:text-red-700 font-medium"
                >
                  Remove
                </button>
              </div>
            ))}
          </div>

          <div className="flex gap-2">
            <input
              type="text"
              value={newPath}
              onChange={(e) => setNewPath(e.target.value)}
              placeholder="/path/to/monitor"
              className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              onKeyPress={(e) => e.key === 'Enter' && addPath()}
            />
            <button
              onClick={addPath}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium"
            >
              Add
            </button>
          </div>
        </div>

        {/* Tool Fingerprints */}
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Tool Detection</h3>
          <p className="text-sm text-gray-600 mb-4">
            Enable/disable detection for specific AI tools
          </p>

          <div className="space-y-2">
            {tools.map((tool) => (
              <label
                key={tool}
                className="flex items-center gap-3 p-3 rounded hover:bg-gray-50 cursor-pointer"
              >
                <input
                  type="checkbox"
                  checked={enabledTools.has(tool)}
                  onChange={() => toggleTool(tool)}
                  className="w-4 h-4 rounded border-gray-300"
                />
                <span className="text-sm font-medium text-gray-900 capitalize">
                  {tool.replace(/-/g, ' ')}
                </span>
              </label>
            ))}
          </div>
        </div>

        {/* Encryption */}
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Security</h3>

          <label className="flex items-center gap-3 p-3 rounded hover:bg-gray-50 cursor-pointer">
            <input
              type="checkbox"
              checked={encryption}
              onChange={(e) => setEncryption(e.target.checked)}
              className="w-4 h-4 rounded border-gray-300"
            />
            <div>
              <p className="text-sm font-medium text-gray-900">
                Encrypt database
              </p>
              <p className="text-xs text-gray-600 mt-1">
                Encrypt audit database with SQLCipher (requires password at startup)
              </p>
            </div>
          </label>
        </div>

        {/* Save Button */}
        <button className="w-full px-6 py-3 bg-green-600 text-white rounded-lg font-medium hover:bg-green-700 transition-colors">
          Save Settings
        </button>
      </div>
    </div>
  )
}
