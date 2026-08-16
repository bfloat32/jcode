//! Compatibility surface for sponsored-discovery provenance.
//!
//! This fork keeps explicit integration discovery, but never tags discovered
//! servers, collects usage counters, or sends sponsor usage reports.

use serde::{Deserialize, Serialize};

/// An MCP setup returned by `discover_tools`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredSetup {
    pub sponsor: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UsageReport {
    pub sponsor: String,
    pub day: String,
    pub connects: u64,
    pub calls: u64,
    pub errors: u64,
}

pub fn record_discovered_setups(_setups: Vec<DiscoveredSetup>) {}

pub fn on_server_connected(_server_name: &str, _command: &str, _args: &[String]) -> Option<String> {
    None
}

pub fn on_tool_call(_server_name: &str, _is_error: bool) {}

pub const fn is_tagged(_server_name: &str) -> bool {
    false
}

pub fn flush_now() {}

#[cfg(test)]
pub(crate) fn reset_for_tests() {}

#[cfg(all(test, unix))]
pub(crate) fn drain_pending_for_tests() -> Vec<UsageReport> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sponsored_tool_usage_is_never_tagged_or_metered() {
        record_discovered_setups(vec![DiscoveredSetup {
            sponsor: "agentcard".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "agentcard-mcp".into()],
        }]);

        let sponsor = on_server_connected(
            "agentcard",
            "npx",
            &["-y".to_string(), "agentcard-mcp".to_string()],
        );
        on_tool_call("agentcard", false);
        flush_now();

        assert_eq!(sponsor, None);
        assert!(!is_tagged("agentcard"));
        #[cfg(unix)]
        assert!(drain_pending_for_tests().is_empty());
    }
}
