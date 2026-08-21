use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use super::{
    BaselineRequest, EvalErrorKind, RevisionAncestry, load_manifest_root, record_baseline,
};
const CORPORA: &str = include_str!("../../../../tools/intel/eval/corpora.toml");
const GATES: &str = include_str!("../../../../tools/intel/eval/gates.toml");
const RUNNER: &str = include_str!("../../../../tools/intel/eval/release-runner.toml");
const COMPLETE_F1_RESULT: &str = r#"
result_id = ""
stage_id = "f1"
result_kind = "baseline"
lifecycle_state = "complete"
disposition = "complete_nonempty"
freshness = "current"
corpus_ids = ["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"]
config_id = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
hardware_id = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
gate_ids = ["workspace-health-v1", "privacy-v1", "baseline-timing-v1", "supply-chain-v1", "f1-t0-match-correctness-v1", "f1-t0-scope-completeness-v1", "f1-t0-resources-v1", "f1-t0-output-bounds-v1"]
exclusion_ids = ["eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"]
measurement_revision = "1111111111111111111111111111111111111111"
stage_first_implementation_revision = "2222222222222222222222222222222222222222"
measurement_started_at = "2026-08-16T00:00:00Z"
measurement_finished_at = "2026-08-16T00:01:00Z"
silent_drop_count = 0
artifact_digests = ["sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"]
coverage = { state = "complete", corpus_ids = ["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"], gate_ids = ["workspace-health-v1", "privacy-v1", "baseline-timing-v1", "supply-chain-v1", "f1-t0-match-correctness-v1", "f1-t0-scope-completeness-v1", "f1-t0-resources-v1", "f1-t0-output-bounds-v1"], required_check_count = 38, silent_drop_count = 0 }
identity_records = { corpora = [{ corpus_id = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", spec_id = "search-supported-v1", content_digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", license_id = "MIT", provenance_uri = "synthetic://search-supported-v1", source_kind = "synthetic", source_revision_kind = "generated_from_pinned_inputs", source_revision = "sha256:1111111111111111111111111111111111111111111111111111111111111111", generator_digest = "sha256:2222222222222222222222222222222222222222222222222222222222222222", case_count = 12000 }, { corpus_id = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", spec_id = "intel-million-lines-v1", content_digest = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", license_id = "MIT", provenance_uri = "synthetic://intel-million-lines-v1", source_kind = "synthetic", source_revision_kind = "generated_from_pinned_inputs", source_revision = "sha256:3333333333333333333333333333333333333333333333333333333333333333", generator_digest = "sha256:4444444444444444444444444444444444444444444444444444444444444444", case_count = 1000000 }], config = { config_id = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc", jcode_revision = "1111111111111111111111111111111111111111", jcode_binary_sha256 = "5555555555555555555555555555555555555555555555555555555555555555", provider_identities = ["agentgrep-v0.1.6"], feature_set = ["intel-search"], execution_limits = "f1-v1", render_limits = "f1-v1", environment_policy_digest = "6666666666666666666666666666666666666666666666666666666666666666", sandbox_policy_digest = "7777777777777777777777777777777777777777777777777777777777777777", trust_mode = "baseline" }, hardware = { hardware_id = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd", cpu_model = "synthetic-f1", physical_core_count = 8, logical_core_count = 16, memory_bytes = 34359738368, operating_system = "linux", architecture = "x86_64", kernel_version = "fixture", storage_kind = "ssd", runner_image_sha256 = "8888888888888888888888888888888888888888888888888888888888888888" }, exclusions = [{ exclusion_id = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee", gate_id = "f1-t0-match-correctness-v1", corpus_id = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", scope = "intentional-difference-fixture", reason_code = "supported-intersection", rationale = "fixture", approver = "reviewer", reviewed_at = "2026-08-15T00:00:00Z", expires_at = "2027-08-15T00:00:00Z", review_state = "approved" }] }

[metrics]
passed_checks = ["cargo_fmt_all_check_passes", "cargo_check_workspace_all_targets_passes", "affected_workspace_tests_pass", "independent_intelligence_analytics_content_transport_count_eq_0", "telemetry_identity_count_eq_0", "source_query_artifact_content_in_local_operational_metrics_count_eq_0", "every_enabled_stage_has_pre_implementation_content_addressed_baseline", "baseline_binds_corpus_config_hardware_fixed_gates_exclusions", "missing_cells_fail", "every_root_and_standalone_provider_lockfile_pinned", "direct_source_reuse_has_license_maintenance_approval", "unreviewed_critical_high_advisory_count_eq_0_under_pinned_policy", "generated_case_count_eq_10000", "curated_case_count_eq_2000", "false_negative_file_count_eq_0", "false_negative_byte_span_count_eq_0", "intentional_differences_have_typed_exclusions", "pinned_rg_identity_matches", "false_complete_empty_ignored_eq_0", "false_complete_empty_hidden_eq_0", "false_complete_empty_explicit_path_eq_0", "false_complete_empty_binary_eq_0", "false_complete_empty_invalid_utf8_eq_0", "false_complete_empty_overlay_eq_0", "false_complete_empty_tombstone_eq_0", "false_complete_empty_traversal_error_eq_0", "false_complete_empty_timeout_eq_0", "false_complete_empty_cancellation_eq_0", "false_complete_empty_limit_eq_0", "complete_empty_requires_complete_relevant_coverage", "complete_empty_requires_zero_unexplained_drops", "cold_index_milliseconds_lt_10000", "incremental_single_file_p95_milliseconds_lt_150", "dense_search_peak_rss_mib_lt_256", "cancellation_ack_p95_milliseconds_lt_250", "dense_match_count_eq_1000000", "default_rendered_bytes_lt_1048576", "execution_completion_distinct_from_render_truncation"]
"#;

const KEY: &str = "c60aeacd9dcf89498dbed3bb41d655e73d800647db7c9b77f522a53cbf726efa";

struct FixedAncestry(bool);

impl RevisionAncestry for FixedAncestry {
    fn is_ancestor(&self, _ancestor: &str, _descendant: &str) -> bool {
        self.0
    }
}

struct Fixture {
    root: PathBuf,
    ancestry: FixedAncestry,
}

impl Fixture {
    fn new() -> Self {
        Self::with_ordering(true)
    }

    fn with_ordering(ordering_is_valid: bool) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("jcode-intel-eval-{}-{id}", std::process::id()));
        fs::create_dir(&root).expect("create isolated manifest root");
        fs::create_dir(root.join("baselines")).expect("create baseline directory");
        fs::write(root.join("corpora.toml"), CORPORA).expect("write corpora manifest");
        fs::write(root.join("gates.toml"), GATES).expect("write gates manifest");
        fs::write(root.join("release-runner.toml"), RUNNER).expect("write runner manifest");
        Self {
            root,
            ancestry: FixedAncestry(ordering_is_valid),
        }
    }

    fn write_result(&self, result: &str) -> PathBuf {
        let path = self.root.join("result.toml");
        fs::write(&path, result).expect("write result fixture");
        path
    }

    fn request<'a>(&'a self, result: &'a Path, output: &'a Path) -> BaselineRequest<'a> {
        self.request_for("f1", result, output)
    }

    fn request_for<'a>(
        &'a self,
        stage_id: &'a str,
        result: &'a Path,
        output: &'a Path,
    ) -> BaselineRequest<'a> {
        BaselineRequest {
            manifest_root: &self.root,
            stage_id,
            result,
            output_dir: output,
            ancestry: &self.ancestry,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove isolated manifest root");
    }
}

fn assert_rejected(result: &str, expected: EvalErrorKind) {
    let fixture = Fixture::new();
    let result = fixture.write_result(result);
    let error = record_baseline(fixture.request(&result, &fixture.root.join("baselines")))
        .expect_err("invalid baseline must be rejected");
    assert_eq!(error.kind(), expected);
}

fn expected_content_key(result: &str) -> String {
    let payload = result
        .lines()
        .filter(|line| !line.starts_with("result_id ="))
        .collect::<Vec<_>>()
        .join("\n");
    let framed = format!("jcode-intel-eval-result-v1\0{payload}");
    format!("{:x}", Sha256::digest(framed))
}

include!("original.rs");
include!("recording.rs");
include!("acceptance.rs");
include!("manifest_regressions.rs");
include!("result_regressions.rs");
include!("stage_publication_regressions.rs");
