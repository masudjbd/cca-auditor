import { useEffect } from 'react'
import { useAuditStore } from '../store/auditStore'
import { useAuditStream } from '../hooks/useAuditStream'
import { getAlerts, dismissAlert } from '../lib/tauri'

export default function Alerts() {
  const { alerts } = useAuditStore()
  const dismissAlertLocal = useAuditStore((state) => state.dismissAlert)
  useAuditStream()

  useEffect(() => {
    const loadAlerts = async () => {
      const initialAlerts = await getAlerts(false)
      useAuditStore.setState({ alerts: initialAlerts })
    }
    loadAlerts()
  }, [])

  const handleDismiss = async (alertId: number) => {
    await dismissAlert(alertId)
    dismissAlertLocal(alertId)
  }

  const undismissedAlerts = alerts.filter((a) => !a.dismissed)

  return (
    <div className="p-8">
      <h2 className="text-3xl font-bold text-gray-900">Alerts</h2>
      <p className="text-gray-600 mt-2">Sensitive path access, security findings</p>

      {undismissedAlerts.length === 0 ? (
        <div className="mt-8 bg-white rounded-lg shadow p-8 text-center">
          <p className="text-gray-500">No active alerts</p>
        </div>
      ) : (
        <div className="mt-8 space-y-4">
          {undismissedAlerts.map((alert) => (
            <div
              key={alert.id}
              className={`bg-white rounded-lg shadow p-6 border-l-4 ${
                alert.severity === 'high'
                  ? 'border-l-red-500'
                  : alert.severity === 'medium'
                    ? 'border-l-yellow-500'
                    : 'border-l-blue-500'
              }`}
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="text-lg font-semibold text-gray-900">
                      {alert.kind}
                    </h3>
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium ${
                        alert.severity === 'high'
                          ? 'bg-red-100 text-red-700'
                          : alert.severity === 'medium'
                            ? 'bg-yellow-100 text-yellow-700'
                            : 'bg-blue-100 text-blue-700'
                      }`}
                    >
                      {alert.severity.toUpperCase()}
                    </span>
                  </div>
                  <p className="text-gray-600 mt-2">{alert.detail}</p>
                  <p className="text-xs text-gray-400 mt-2">
                    {new Date(alert.timestamp).toLocaleString()}
                  </p>
                </div>
                <button
                  onClick={() => handleDismiss(alert.id)}
                  className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded transition-colors flex-shrink-0"
                >
                  Dismiss
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
