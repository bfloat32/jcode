use serde::Deserialize;

use crate::error::{EvalError, EvalErrorKind};

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct GatesManifest {
    schema_version: u32,
    manifest_kind: String,
    loader_policy: LoaderPolicy,
    identity_contract: IdentityContract,
    #[serde(rename = "gate")]
    pub(crate) gates: Vec<Gate>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct LoaderPolicy {
    unknown_fields: String,
    duplicate_ids: String,
    missing_metrics: String,
    missing_exclusions: String,
    partial_results: String,
    stale_results: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct IdentityContract {
    algorithm: String,
    encoding: String,
    canonicalization: String,
    domain: String,
    id_field: String,
    immutable_fields: Vec<String>,
    ratchet_direction: String,
    loosening_requires: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Gate {
    pub(crate) gate_id: String,
    pub(crate) stage: String,
    pub(crate) corpus_spec_ids: Vec<String>,
    pub(crate) checks: Vec<String>,
    pub(crate) exclusion_policy: String,
}

impl GatesManifest {
    pub(crate) fn validate(&self) -> Result<(), EvalError> {
        if self.schema_version != 1 {
            return Err(EvalError::new(
                EvalErrorKind::UnsupportedSchema,
                "unsupported gates schema version",
            ));
        }
        let fixed = self.manifest_kind == "jcode-intel-eval-gates"
            && self.loader_policy.unknown_fields == "reject"
            && self.loader_policy.duplicate_ids == "reject"
            && self.loader_policy.missing_metrics == "reject"
            && self.loader_policy.missing_exclusions == "reject"
            && self.loader_policy.partial_results == "reject"
            && self.loader_policy.stale_results == "reject"
            && self.identity_contract.algorithm == "sha256"
            && self.identity_contract.encoding == "lowercase-hex"
            && self.identity_contract.canonicalization == "jcode-intel-eval-canonical-toml-v1"
            && self.identity_contract.domain == "jcode-intel-eval-gate-v1"
            && self.identity_contract.id_field == "gate_id"
            && self.identity_contract.immutable_fields
                == [
                    "gate_id",
                    "stage",
                    "corpus_spec_ids",
                    "checks",
                    "exclusion_policy",
                ]
            && self.identity_contract.ratchet_direction == "tighten_only"
            && self.identity_contract.loosening_requires == "reviewed_design_revision"
            && self.gates.len() == 18
            && self.gates.iter().all(|gate| {
                !gate.stage.is_empty()
                    && !gate.checks.is_empty()
                    && !gate.exclusion_policy.is_empty()
            });
        if !fixed {
            return Err(EvalError::new(
                EvalErrorKind::InvalidManifest,
                "invalid gates manifest contract",
            ));
        }
        Ok(())
    }
}
