use serde::Deserialize;

use crate::error::{EvalError, classify_toml};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvalResult {
    pub(crate) result_id: String,
    pub(crate) stage_id: String,
    pub(crate) result_kind: String,
    pub(crate) lifecycle_state: String,
    pub(crate) disposition: String,
    pub(crate) freshness: String,
    pub(crate) corpus_ids: Vec<String>,
    pub(crate) config_id: String,
    pub(crate) hardware_id: String,
    pub(crate) gate_ids: Vec<String>,
    pub(crate) exclusion_ids: Vec<String>,
    pub(crate) measurement_revision: String,
    pub(crate) stage_first_implementation_revision: String,
    pub(crate) measurement_started_at: String,
    pub(crate) measurement_finished_at: String,
    pub(crate) silent_drop_count: u64,
    pub(crate) artifact_digests: Vec<String>,
    pub(crate) coverage: Coverage,
    pub(crate) identity_records: IdentityRecords,
    pub(crate) metrics: Metrics,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Coverage {
    pub(crate) state: String,
    pub(crate) corpus_ids: Vec<String>,
    pub(crate) gate_ids: Vec<String>,
    pub(crate) required_check_count: usize,
    pub(crate) silent_drop_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityRecords {
    pub(crate) corpora: Vec<CorpusIdentity>,
    pub(crate) config: ConfigIdentity,
    pub(crate) hardware: HardwareIdentity,
    pub(crate) exclusions: Vec<ExclusionIdentity>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CorpusIdentity {
    pub(crate) corpus_id: String,
    pub(crate) spec_id: String,
    pub(crate) content_digest: String,
    pub(crate) license_id: String,
    pub(crate) provenance_uri: String,
    pub(crate) source_kind: String,
    pub(crate) source_revision_kind: String,
    pub(crate) source_revision: String,
    pub(crate) generator_digest: String,
    pub(crate) case_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfigIdentity {
    pub(crate) config_id: String,
    pub(crate) jcode_revision: String,
    pub(crate) jcode_binary_sha256: String,
    pub(crate) provider_identities: Vec<String>,
    pub(crate) feature_set: Vec<String>,
    pub(crate) execution_limits: String,
    pub(crate) render_limits: String,
    pub(crate) environment_policy_digest: String,
    pub(crate) sandbox_policy_digest: String,
    pub(crate) trust_mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HardwareIdentity {
    pub(crate) hardware_id: String,
    pub(crate) cpu_model: String,
    pub(crate) physical_core_count: u64,
    pub(crate) logical_core_count: u64,
    pub(crate) memory_bytes: u64,
    pub(crate) operating_system: String,
    pub(crate) architecture: String,
    pub(crate) kernel_version: String,
    pub(crate) storage_kind: String,
    pub(crate) runner_image_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExclusionIdentity {
    pub(crate) exclusion_id: String,
    pub(crate) gate_id: String,
    pub(crate) corpus_id: String,
    pub(crate) scope: String,
    pub(crate) reason_code: String,
    pub(crate) rationale: String,
    pub(crate) approver: String,
    pub(crate) reviewed_at: String,
    pub(crate) expires_at: String,
    pub(crate) review_state: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Metrics {
    pub(crate) passed_checks: Vec<String>,
}

pub(crate) fn parse_result(text: &str) -> Result<EvalResult, EvalError> {
    toml::from_str(text).map_err(|error| classify_toml(&error, true))
}
