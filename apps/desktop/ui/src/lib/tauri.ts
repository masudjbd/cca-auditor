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
  refspec: string
): Promise<GuardrailResult> {
  try {
    return await invoke<GuardrailResult>('push_with_guardrail', { remote, refspec })
  } catch (error) {
    console.error('Failed to push with guardrail:', error)
    throw error
  }
}

export interface AppSettings {
  watch_paths: string[]
  enabled_tools: string[]
  encryption: boolean
}

export async function saveSettingsToBackend(settings: AppSettings): Promise<void> {
  try {
    await invoke('save_settings', { settings })
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
