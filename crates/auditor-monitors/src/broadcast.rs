use auditor_core::session::AuditSession;
use auditor_core::samples::ResourceSample;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonitorEvent {
    #[serde(rename = "session-opened")]
    SessionOpened { session: AuditSession },
    #[serde(rename = "session-closed")]
    SessionClosed { session_id: String },
    #[serde(rename = "resource-sample")]
    ResourceSample { sample: ResourceSample },
    #[serde(rename = "alert-raised")]
    AlertRaised {
        id: i64,
        kind: String,
        severity: String,
        detail: String,
    },
}

pub type EventSender = broadcast::Sender<MonitorEvent>;
pub type EventReceiver = broadcast::Receiver<MonitorEvent>;

pub fn create_channel() -> (EventSender, EventReceiver) {
    broadcast::channel(1024)
}
