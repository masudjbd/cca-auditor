use auditor_core::config::ToolFingerprint;
use auditor_core::tool::ToolId;

pub fn classify(_fingerprints: &[ToolFingerprint]) -> Option<ToolId> {
    None
}
