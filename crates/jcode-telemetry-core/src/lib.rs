//! Offline compatibility facade for the telemetry API.
//!
//! `jcode-sol` intentionally performs no telemetry collection or delivery:
//! no installation identifier, session state, filesystem markers, project
//! scanning, background worker, or network request is created here.  The small
//! API surface remains so the rest of the application can stay decoupled while
//! telemetry call sites are removed gradually during normal refactors.
//!
//! Authentication events continue to be written to jcode's local diagnostic
//! log.  They never leave the machine through this crate.

#![forbid(unsafe_code)]

use serde_json::Value;

pub use jcode_usage_types::{ErrorCategory, SessionEndReason};

/// Telemetry is permanently disabled in this distribution.
#[inline]
pub fn is_enabled() -> bool {
    false
}

/// Prompt and transcript sharing is permanently disabled.
#[inline]
pub fn content_sharing_enabled() -> bool {
    false
}

/// Disabling content sharing succeeds; enabling it is unsupported.
#[inline]
pub fn set_content_sharing_enabled(enabled: bool) -> bool {
    !enabled
}

#[inline]
pub fn record_setup_step_once(_step: &'static str) {}

#[inline]
pub fn record_feedback(_text: &str) {}

#[inline]
pub fn record_command_family(_command: &str) {}

#[inline]
pub fn record_install_if_first_run() {}

#[inline]
pub fn record_upgrade_if_needed() {}

#[inline]
pub fn record_provider_selected(_provider: &str) {}

/// Preserve useful local auth diagnostics without collecting telemetry.
pub fn record_auth_started(provider: &str, method: &str) {
    jcode_logging::auth_event("auth_started", provider, &[("method", method)]);
}

pub fn record_auth_failed(provider: &str, method: &str) {
    record_auth_failed_reason(provider, method, "unknown");
}

pub fn record_auth_failed_reason(provider: &str, method: &str, reason: &str) {
    jcode_logging::auth_event(
        "auth_failed",
        provider,
        &[("method", method), ("reason", reason)],
    );
}

pub fn record_auth_cancelled(provider: &str, method: &str) {
    jcode_logging::auth_event("auth_cancelled", provider, &[("method", method)]);
}

pub fn record_auth_surface_blocked(provider: &str, method: &str) {
    jcode_logging::auth_event("auth_surface_blocked", provider, &[("method", method)]);
}

pub fn record_auth_surface_blocked_reason(provider: &str, method: &str, reason: &str) {
    jcode_logging::auth_event(
        "auth_surface_blocked",
        provider,
        &[("method", method), ("reason", reason)],
    );
}

pub fn record_auth_success(provider: &str, method: &str) {
    jcode_logging::auth_event("auth_success", provider, &[("method", method)]);
}

#[inline]
pub fn begin_session(_provider: &str, _model: &str) {}

#[inline]
pub fn begin_session_with_parent(
    _provider: &str,
    _model: &str,
    _parent_session_id: Option<String>,
    _resumed_session: bool,
) {
}

#[inline]
pub fn begin_resumed_session(_provider: &str, _model: &str) {}

#[inline]
pub fn record_turn() {}

#[inline]
pub fn record_assistant_response() {}

#[inline]
pub fn record_memory_injected(_count: usize, _age_ms: u64) {}

#[inline]
pub fn record_tool_call() {}

#[inline]
pub fn record_tool_failure() {}

#[inline]
pub fn record_connection_type(_connection: &str) {}

#[inline]
pub fn record_token_usage(
    _input_tokens: u64,
    _output_tokens: u64,
    _cache_read_input_tokens: Option<u64>,
    _cache_creation_input_tokens: Option<u64>,
) {
}

#[inline]
pub fn record_error(_category: ErrorCategory) {}

#[inline]
pub fn record_provider_switch() {}

#[inline]
pub fn record_model_switch() {}

#[inline]
pub fn record_user_cancelled() {}

#[inline]
pub fn record_tool_execution(
    _name: &str,
    _input: &Value,
    _succeeded: bool,
    _latency_ms: u64,
) {
}

#[inline]
pub fn end_session(_provider_end: &str, _model_end: &str) {}

#[inline]
pub fn end_session_with_reason(
    _provider_end: &str,
    _model_end: &str,
    _reason: SessionEndReason,
) {
}

#[inline]
pub fn record_crash(_provider_end: &str, _model_end: &str, _reason: SessionEndReason) {}

#[inline]
pub fn current_provider_model() -> Option<(String, String)> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_and_content_sharing_are_permanently_disabled() {
        assert!(!is_enabled());
        assert!(!content_sharing_enabled());
        assert!(set_content_sharing_enabled(false));
        assert!(!set_content_sharing_enabled(true));
        assert_eq!(current_provider_model(), None);
    }
}
