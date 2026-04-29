import { useEffect } from 'react'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useAuditStore, AuditSession, AuditEvent, ResourceSample, Alert } from '../store/auditStore'

interface SessionOpenedPayload {
  session: AuditSession
}

interface SessionClosedPayload {
  session_id: string
}

interface ResourceSamplePayload {
  sample: ResourceSample
}

interface AuditEventPayload {
  event: AuditEvent
}

interface AlertRaisedPayload {
  alert: Alert
}

export function useAuditStream() {
  const {
    addSession,
    removeSession,
    addEvent,
    addSample,
    addAlert,
  } = useAuditStore()

  useEffect(() => {
    const unlisteners: UnlistenFn[] = []

    async function setupListeners() {
      try {
        const sessionOpenedUnlisten = await listen<SessionOpenedPayload>(
          'session-opened',
          (event) => {
            addSession(event.payload.session)
          }
        )
        unlisteners.push(sessionOpenedUnlisten)

        const sessionClosedUnlisten = await listen<SessionClosedPayload>(
          'session-closed',
          (event) => {
            removeSession(event.payload.session_id)
          }
        )
        unlisteners.push(sessionClosedUnlisten)

        const resourceSampleUnlisten = await listen<ResourceSamplePayload>(
          'resource-sample',
          (event) => {
            addSample(event.payload.sample)
          }
        )
        unlisteners.push(resourceSampleUnlisten)

        const auditEventUnlisten = await listen<AuditEventPayload>(
          'audit-event',
          (event) => {
            addEvent(event.payload.event)
          }
        )
        unlisteners.push(auditEventUnlisten)

        const alertRaisedUnlisten = await listen<AlertRaisedPayload>(
          'alert-raised',
          (event) => {
            addAlert(event.payload.alert)
          }
        )
        unlisteners.push(alertRaisedUnlisten)
      } catch (error) {
        console.error('Failed to setup event listeners:', error)
      }
    }

    setupListeners()

    return () => {
      unlisteners.forEach((unlisten) => unlisten())
    }
  }, [addSession, removeSession, addEvent, addSample, addAlert])
}
