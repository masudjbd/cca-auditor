import { invoke } from '@tauri-apps/api/tauri'
import { AuditSession, AuditEvent, ResourceSample, Alert } from '../store/auditStore'

export async function getLiveSessions(): Promise<AuditSession[]> {
  try {
    return await invoke<AuditSession[]>('get_live_sessions')
  } catch (error) {
    console.error('Failed to get live sessions:', error)
    return []
  }
}

export async function getEvents(
  sessionId: string,
  limit: number = 1000
): Promise<AuditEvent[]> {
  try {
    return await invoke<AuditEvent[]>('get_events', { sessionId, limit })
  } catch (error) {
    console.error('Failed to get events:', error)
    return []
  }
}

export async function getSamples(
  pid: number,
  from: number,
  to: number
): Promise<ResourceSample[]> {
  try {
    return await invoke<ResourceSample[]>('get_samples', { pid, from, to })
  } catch (error) {
    console.error('Failed to get samples:', error)
    return []
  }
}

export async function getAlerts(dismissed: boolean = false): Promise<Alert[]> {
  try {
    return await invoke<Alert[]>('get_alerts', { dismissed })
  } catch (error) {
    console.error('Failed to get alerts:', error)
    return []
  }
}

export async function dismissAlert(id: number): Promise<void> {
  try {
    await invoke('dismiss_alert', { id })
  } catch (error) {
    console.error('Failed to dismiss alert:', error)
  }
}

export interface ReportOptions {
  session_ids: string[]
  format: 'html' | 'pdf' | 'markdown' | 'json'
}

export async function generateReport(options: ReportOptions): Promise<string> {
  try {
    return await invoke<string>('generate_report', {
      sessionIds: options.session_ids,
      format: options.format,
    })
  } catch (error) {
    console.error('Failed to generate report:', error)
    throw error
  }
}

export interface GuardrailResult {
  allowed: boolean
  findings?: Array<{
    rule_id: string
    file: string
    line: number
    severity: 'high' | 'medium' | 'low'
    redacted_value: string
  }>
}

export async function pushWithGuardrail(
  remote: string,
  refspec: string,
  repoPath?: string
): Promise<GuardrailResult> {
  return await invoke<GuardrailResult>('push_with_guardrail', {
    remote,
    refspec,
    repoPath: repoPath ?? null,
  })
}

export async function executePush(
  remote: string,
  refspec: string,
  repoPath?: string
): Promise<void> {
  await invoke('execute_push', {
    remote,
    refspec,
    repoPath: repoPath ?? null,
  })
}

export interface AppSettings {
  watch_paths: string[]
  enabled_tools: string[]
  encryption: boolean
}

export async function saveSettingsToBackend(settings: AppSettings): Promise<void> {
  try {
    // Use save_settings_with_reload for instant FS watcher reload
    await invoke('save_settings_with_reload', { settings })
  } catch (error) {
    console.warn('Failed to save settings to backend, using localStorage only:', error)
    throw error
  }
}

export async function loadSettingsFromBackend(): Promise<AppSettings | null> {
  try {
    return await invoke<AppSettings>('load_settings')
  } catch (error) {
    console.warn('Failed to load settings from backend:', error)
    return null
  }
}

export interface DbStats {
  total_sessions: number
  active_sessions: number
  total_events: number
  total_samples: number
  total_alerts: number
  undismissed_alerts: number
  db_size_bytes: number
  oldest_event_ts: number | null
  newest_event_ts: number | null
  events_by_kind: [string, number][]
}

export async function getDbStats(): Promise<DbStats | null> {
  try {
    return await invoke<DbStats>('get_db_stats')
  } catch (error) {
    console.error('Failed to get DB stats:', error)
    return null
  }
}

export async function purgeAllData(): Promise<void> {
  try {
    await invoke('purge_all_data')
  } catch (error) {
    console.error('Failed to purge data:', error)
    throw error
  }
}

export async function saveReportToFile(path: string, content: string): Promise<void> {
  try {
    await invoke('save_report_to_file', { path, content })
  } catch (error) {
    console.error('Failed to save report:', error)
    throw error
  }
}

export async function openPathInFinder(path: string): Promise<void> {
  try {
    await invoke('open_path_in_finder', { path })
  } catch (error) {
    console.error('Failed to open path:', error)
  }
}

export interface UserSensitivePath {
  pattern: string
  severity: 'high' | 'medium' | 'low'
  reason: string
}

export async function getUserSensitivePaths(): Promise<UserSensitivePath[]> {
  try {
    return await invoke<UserSensitivePath[]>('get_user_sensitive_paths')
  } catch (error) {
    console.error('Failed to load user sensitive paths:', error)
    return []
  }
}

export async function saveUserSensitivePaths(
  paths: UserSensitivePath[]
): Promise<void> {
  try {
    await invoke('save_user_sensitive_paths', { paths })
  } catch (error) {
    console.error('Failed to save user sensitive paths:', error)
    throw error
  }
}
