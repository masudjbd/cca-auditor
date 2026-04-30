export default function About() {
  const supportedTools = [
    { name: 'Cursor', description: 'AI code editor' },
    { name: 'Claude Code', description: 'CLI agent by Anthropic' },
    { name: 'Claude Desktop', description: 'Anthropic desktop app' },
    { name: 'Windsurf', description: 'Codeium IDE' },
    { name: 'Ollama', description: 'Local LLM server' },
    { name: 'LM Studio', description: 'Local model GUI' },
    { name: 'Aider', description: 'CLI pair programmer' },
    { name: 'Cline', description: 'VS Code extension (Roo)' },
    { name: 'Continue.dev', description: 'VS Code extension' },
    { name: 'GitHub Copilot Chat', description: 'VS Code extension' },
    { name: 'Tabnine', description: 'Code completion' },
    { name: 'Supermaven', description: 'Fast completion' },
  ]

  return (
    <div className="p-8 max-w-3xl">
      <div className="flex items-start gap-4 mb-6">
        <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-indigo-900 to-slate-900 flex items-center justify-center shadow-lg">
          <span className="text-3xl">🔍</span>
        </div>
        <div>
          <h1 className="text-3xl font-bold text-gray-900">CCAudit</h1>
          <p className="text-gray-600 mt-1">v0.1.0 — AI Tool Auditor</p>
        </div>
      </div>

      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">About</h2>
        <p className="text-gray-700 leading-relaxed">
          CCAudit is a cross-platform desktop app that provides a complete audit trail
          for AI coding tools. It tracks file access, network connections, subprocess
          execution, and real-time CPU/GPU/memory usage for popular AI tools — all
          stored locally on your machine.
        </p>
        <p className="text-gray-700 leading-relaxed mt-3">
          Built for developers who want to <strong>know what their AI tools are doing</strong>:
          which files they read, what hostnames they connect to, and whether they're
          accidentally about to commit a secret.
        </p>
      </div>

      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Why CCAudit</h2>
        <ul className="space-y-2 text-gray-700">
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span><strong>Local-first.</strong> No telemetry, no cloud, no phone-home. All data in <code className="text-sm bg-gray-100 px-1 rounded">~/.cca-audit/</code>.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span><strong>Tool-aware.</strong> Knows the difference between Cursor, Claude Code, Ollama, etc.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span><strong>Confidence-tagged.</strong> Every event marked High / Ambiguous / Verified.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span><strong>Secret guardrail.</strong> 14 patterns scanned before push (GitHub, Anthropic, OpenAI, AWS, …).</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span><strong>Open source.</strong> MIT license. Inspect the source, audit the auditor.</span>
          </li>
        </ul>
      </div>

      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Supported Tools</h2>
        <div className="grid grid-cols-2 gap-2">
          {supportedTools.map((tool) => (
            <div key={tool.name} className="flex items-baseline gap-2 text-sm">
              <span className="text-gray-900 font-medium">{tool.name}</span>
              <span className="text-gray-500">— {tool.description}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Architecture</h2>
        <ul className="space-y-1.5 text-sm text-gray-700">
          <li>
            <strong>Backend:</strong> Rust (Tauri 2) — 9 specialized crates (auditor-core, db, detect, monitors, fs, net, report, guardrail, ipc)
          </li>
          <li>
            <strong>Frontend:</strong> React 18 + TypeScript + Vite + Tailwind CSS + Recharts
          </li>
          <li>
            <strong>Storage:</strong> SQLite (WAL mode) with rolling 10s/1m aggregates
          </li>
          <li>
            <strong>Detection:</strong> sysinfo (1Hz/2s polling), notify (FS), netstat2 (network)
          </li>
          <li>
            <strong>Secret scanning:</strong> 14 inline regex patterns + git2 staged-diff
          </li>
        </ul>
      </div>

      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Resources</h2>
        <ul className="space-y-1.5 text-sm">
          <li>
            <a
              href="https://github.com/masudjbd/cca-auditor"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-600 hover:underline"
            >
              GitHub: masudjbd/cca-auditor
            </a>
          </li>
          <li>
            <a
              href="https://github.com/masudjbd/cca-auditor/issues"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-600 hover:underline"
            >
              Report a bug or request a feature
            </a>
          </li>
          <li>
            <a
              href="https://github.com/masudjbd/cca-auditor/blob/main/docs/threat-model.md"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-600 hover:underline"
            >
              Threat model
            </a>
          </li>
        </ul>
      </div>

      <div className="text-center text-sm text-gray-500 mt-8 pb-4">
        <p>
          Built by{' '}
          <a
            href="mailto:masudjbd@gmail.com"
            className="text-blue-600 hover:underline"
          >
            Masudur Rahman
          </a>
        </p>
        <p className="mt-1">MIT License · © 2026</p>
      </div>
    </div>
  )
}
