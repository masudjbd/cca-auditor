import { create } from 'zustand'

export interface AuditSession {
  id: string
  tool_id: string
  pid: number
  confidence: 'High' | 'Ambiguous' | 'Verified'
  started_at: number
  ended_at: number | null
}

export interface AuditEvent {
  id: number
  session_id: string
  tool_id: string
  kind: 'FsRead' | 'FsWrite' | 'FsDelete' | 'NetConnect' | 'ProcessSpawn' | 'LocalArtifact'
  path?: string
  dest_addr?: string
  dest_port?: number
  confidence: 'High' | 'Ambiguous' | 'Verified'
  timestamp: number
}

export interface ResourceSample {
  pid: number
  cpu_pct: number
  rss_bytes: number
  gpu_mem_bytes: number
  timestamp: number
}

export interface Alert {
  id: number
  kind: string
  severity: 'high' | 'medium' | 'low'
  detail: string
  timestamp: number
  dismissed: boolean
}

export interface AuditState {
  sessions: AuditSession[]
  events: AuditEvent[]
  samples: Map<number, ResourceSample[]>
  alerts: Alert[]

  addSession: (session: AuditSession) => void
  removeSession: (sessionId: string) => void
  addEvent: (event: AuditEvent) => void
  addSample: (sample: ResourceSample) => void
  addAlert: (alert: Alert) => void
  dismissAlert: (alertId: number) => void
  setSessions: (sessions: AuditSession[]) => void
  setEvents: (events: AuditEvent[]) => void
  setSamples: (samples: ResourceSample[]) => void
  setAlerts: (alerts: Alert[]) => void
  clearAll: () => void
}

export const useAuditStore = create<AuditState>((set) => ({
  sessions: [],
  events: [],
  samples: new Map(),
  alerts: [],

  addSession: (session) =>
    set((state) => ({
      sessions: [session, ...state.sessions],
    })),

  removeSession: (sessionId) =>
    set((state) => ({
      sessions: state.sessions.filter((s) => s.id !== sessionId),
    })),

  addEvent: (event) =>
    set((state) => ({
      events: [event, ...state.events].slice(0, 5000),
    })),

  addSample: (sample) =>
    set((state) => {
      const samples = new Map(state.samples)
      const pidSamples = samples.get(sample.pid) || []
      samples.set(sample.pid, [...pidSamples, sample].slice(-600))
      return { samples }
    }),

  addAlert: (alert) =>
    set((state) => ({
      alerts: [alert, ...state.alerts],
    })),

  dismissAlert: (alertId) =>
    set((state) => ({
      alerts: state.alerts.map((a) =>
        a.id === alertId ? { ...a, dismissed: true } : a
      ),
    })),

  setSessions: (sessions) => set({ sessions }),
  setEvents: (events) => set({ events }),
  setSamples: (samples) =>
    set({
      samples: new Map(samples.map((s) => [s.pid, [s]])),
    }),
  setAlerts: (alerts) => set({ alerts }),

  clearAll: () =>
    set({
      sessions: [],
      events: [],
      samples: new Map(),
      alerts: [],
    }),
}))
