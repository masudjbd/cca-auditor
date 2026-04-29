// Tool-to-watch-paths intelligent mapping
// Each tool has known artifact/data directories worth monitoring.
// Returns paths that should be monitored when the tool is enabled.

export interface ToolPathSuggestion {
  tool_id: string
  tool_name: string
  paths: string[]
  description: string
}

const HOME = '~'

export const TOOL_PATH_SUGGESTIONS: Record<string, ToolPathSuggestion> = {
  cursor: {
    tool_id: 'cursor',
    tool_name: 'Cursor',
    paths: [
      `${HOME}/Library/Application Support/Cursor/User/globalStorage`,
      `${HOME}/Library/Application Support/Cursor/User/workspaceStorage`,
      `${HOME}/Library/Application Support/Cursor/logs`,
    ],
    description: 'Settings, workspace storage, and logs',
  },
  'claude-code': {
    tool_id: 'claude-code',
    tool_name: 'Claude Code',
    paths: [
      `${HOME}/.claude/projects`,
      `${HOME}/.claude/todos`,
      `${HOME}/.claude/settings.json`,
    ],
    description: 'Project memory, todos, and settings',
  },
  'claude-desktop': {
    tool_id: 'claude-desktop',
    tool_name: 'Claude Desktop',
    paths: [
      `${HOME}/Library/Application Support/Claude`,
      `${HOME}/Library/Logs/Claude`,
    ],
    description: 'Application data and logs',
  },
  windsurf: {
    tool_id: 'windsurf',
    tool_name: 'Windsurf',
    paths: [
      `${HOME}/Library/Application Support/Windsurf`,
      `${HOME}/.codeium/windsurf`,
    ],
    description: 'Application data and Codeium config',
  },
  ollama: {
    tool_id: 'ollama',
    tool_name: 'Ollama',
    paths: [
      `${HOME}/.ollama`,
      `${HOME}/.ollama/logs`,
      `${HOME}/.ollama/models`,
    ],
    description: 'Models, logs, and config',
  },
  lmstudio: {
    tool_id: 'lmstudio',
    tool_name: 'LM Studio',
    paths: [
      `${HOME}/.lmstudio`,
      `${HOME}/.cache/lm-studio`,
    ],
    description: 'Models and cache',
  },
  aider: {
    tool_id: 'aider',
    tool_name: 'Aider',
    paths: [
      `${HOME}/.aider.conf.yml`,
      `${HOME}/.aider.input.history`,
      `${HOME}/.aider.chat.history.md`,
    ],
    description: 'Chat history and configuration',
  },
  cline: {
    tool_id: 'cline',
    tool_name: 'Cline',
    paths: [
      `${HOME}/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev`,
      `${HOME}/.vscode/extensions`,
    ],
    description: 'VS Code extension storage',
  },
  continue: {
    tool_id: 'continue',
    tool_name: 'Continue.dev',
    paths: [
      `${HOME}/.continue/sessions`,
      `${HOME}/.continue/dev_data`,
      `${HOME}/.continue/config.json`,
    ],
    description: 'Sessions, dev data, and config',
  },
  'copilot-chat': {
    tool_id: 'copilot-chat',
    tool_name: 'GitHub Copilot Chat',
    paths: [
      `${HOME}/Library/Application Support/Code/User/globalStorage/github.copilot-chat`,
    ],
    description: 'VS Code extension storage',
  },
  tabnine: {
    tool_id: 'tabnine',
    tool_name: 'Tabnine',
    paths: [
      `${HOME}/.config/TabNine`,
      `${HOME}/Library/Application Support/Code/User/globalStorage/tabnine.tabnine-vscode`,
    ],
    description: 'Configuration and VS Code storage',
  },
  supermaven: {
    tool_id: 'supermaven',
    tool_name: 'Supermaven',
    paths: [
      `${HOME}/Library/Application Support/Code/User/globalStorage/supermaven.supermaven`,
    ],
    description: 'VS Code extension storage',
  },
}

// Common workspace paths that apply across all tools
export const COMMON_WORKSPACE_PATHS: ToolPathSuggestion = {
  tool_id: '_common',
  tool_name: 'Common Project Locations',
  paths: [
    `${HOME}/projects`,
    `${HOME}/workspace`,
    `${HOME}/code`,
    `${HOME}/Documents/code`,
    `${HOME}/dev`,
  ],
  description: 'Standard project directories',
}

export function getSuggestedPathsForTools(enabledTools: string[]): ToolPathSuggestion[] {
  const suggestions: ToolPathSuggestion[] = []

  // Add tool-specific suggestions
  for (const toolId of enabledTools) {
    if (TOOL_PATH_SUGGESTIONS[toolId]) {
      suggestions.push(TOOL_PATH_SUGGESTIONS[toolId])
    }
  }

  return suggestions
}

const STORAGE_KEY = 'cca-audit:settings'

export interface PersistedSettings {
  watchPaths: string[]
  enabledTools: string[]
  encryption: boolean
}

export function loadSettings(): PersistedSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      return JSON.parse(raw) as PersistedSettings
    }
  } catch (error) {
    console.error('Failed to load settings:', error)
  }
  return {
    watchPaths: [`${HOME}/projects`, `${HOME}/workspace`],
    enabledTools: [
      'cursor',
      'claude-code',
      'windsurf',
      'ollama',
      'lmstudio',
      'aider',
      'cline',
      'continue',
    ],
    encryption: false,
  }
}

export function saveSettings(settings: PersistedSettings): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
  } catch (error) {
    console.error('Failed to save settings:', error)
  }
}
