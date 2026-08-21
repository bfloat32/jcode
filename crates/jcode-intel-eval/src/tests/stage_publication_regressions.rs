fn remove_second_corpus_identity(text: &str) -> String {
    let marker = ", { corpus_id = \"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"";
    let ending = "case_count = 1000000 }";
    let start = text.find(marker).expect("second corpus identity");
    let relative_end = text[start..].find(ending).expect("second corpus ending");
    let end = start + relative_end + ending.len();
    let mut result = text.to_owned();
    result.replace_range(start..end, "");
    result
}

fn complete_f7_result(artifact_digit: char) -> String {
    let common_gates = "\"workspace-health-v1\", \"privacy-v1\", \"baseline-timing-v1\", \"supply-chain-v1\", \"f7-end-task-quality-v1\", \"f7-idle-overhead-v1\"";
    let checks = [
        "cargo_fmt_all_check_passes",
        "cargo_check_workspace_all_targets_passes",
        "affected_workspace_tests_pass",
        "independent_intelligence_analytics_content_transport_count_eq_0",
        "telemetry_identity_count_eq_0",
        "source_query_artifact_content_in_local_operational_metrics_count_eq_0",
        "every_enabled_stage_has_pre_implementation_content_addressed_baseline",
        "baseline_binds_corpus_config_hardware_fixed_gates_exclusions",
        "missing_cells_fail",
        "every_root_and_standalone_provider_lockfile_pinned",
        "direct_source_reuse_has_license_maintenance_approval",
        "unreviewed_critical_high_advisory_count_eq_0_under_pinned_policy",
        "pinned_rust_task_count_gte_200",
        "successful_completion_percentage_point_improvement_gte_10",
        "missed_affected_sites_reduction_gte_0_30",
        "median_source_tool_output_token_reduction_on_joint_successes_gte_0_25",
        "build_test_success_regression_eq_0",
        "lazy_startup_p95_added_milliseconds_lte_25",
        "pre_activation_rss_added_mib_lte_10",
    ]
    .map(|check| format!("\"{check}\""))
    .join(", ");
    let mut result = COMPLETE_F1_RESULT
        .replacen("stage_id = \"f1\"", "stage_id = \"f7\"", 1)
        .replacen("spec_id = \"search-supported-v1\"", "spec_id = \"rust-change-history-v1\"", 1)
        .replacen("gate_id = \"f1-t0-match-correctness-v1\"", "gate_id = \"f7-end-task-quality-v1\"", 1)
        .replacen(
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            &format!("sha256:{}", artifact_digit.to_string().repeat(64)),
            1,
        );
    result = remove_second_corpus_identity(&result);
    result = replace_line(
        &result,
        "corpus_ids =",
        "corpus_ids = [\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"]",
    );
    result = replace_line(&result, "gate_ids =", &format!("gate_ids = [{common_gates}]"));
    result = replace_line(
        &result,
        "coverage =",
        &format!("coverage = {{ state = \"complete\", corpus_ids = [\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"], gate_ids = [{common_gates}], required_check_count = 19, silent_drop_count = 0 }}"),
    );
    replace_line(&result, "passed_checks =", &format!("passed_checks = [{checks}]"))
}

fn record_f7(fixture: &Fixture, text: &str, name: &str) {
    let path = fixture.root.join(name);
    fs::write(&path, text).expect("write F7 result");
    record_baseline(fixture.request_for("f7", &path, &fixture.root.join("baselines")))
        .expect("record F7 result");
}

#[test]
fn one_f7_artifact_is_insufficient_and_two_distinct_artifacts_pass() {
    // Given: one complete F7 release run.
    let fixture = Fixture::new();
    record_f7(&fixture, &complete_f7_result('8'), "f7-first.toml");
    let manifests = load_manifest_root(&fixture.root).expect("load F7 root");
    // When: F7 stage evidence is validated with one and then two runs.
    let one = manifests
        .validate_stage_with("f7", &fixture.ancestry)
        .expect_err("one F7 run must fail");
    record_f7(&fixture, &complete_f7_result('9'), "f7-second.toml");
    let two = manifests
        .validate_stage_with("f7", &fixture.ancestry)
        .expect("two F7 runs pass");
    // Then: the run-count contract is enforced exactly.
    assert_eq!(one.kind(), EvalErrorKind::MissingStageEvidence);
    assert_eq!(two, "f7");
}

#[test]
fn stage_scan_rejects_a_validly_named_later_malformed_artifact() {
    // Given: a valid F1 artifact followed lexically by a malformed baseline artifact.
    let fixture = Fixture::new();
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    record_baseline(fixture.request(&result, &fixture.root.join("baselines")))
        .expect("record valid first artifact");
    fs::write(
        fixture.root.join("baselines").join(format!("{}.toml", "f".repeat(64))),
        "not = [valid",
    )
    .expect("write malformed later artifact");
    let manifests = load_manifest_root(&fixture.root).expect("load stage root");
    // When: the requested stage is validated.
    let error = manifests
        .validate_stage_with("f1", &fixture.ancestry)
        .expect_err("later malformed artifact must fail");
    // Then: validation is exhaustive rather than first-match successful.
    assert_eq!(error.kind(), EvalErrorKind::MalformedToml);
}

#[test]
fn stage_scan_rejects_a_later_identity_conflict() {
    // Given: valid F1 evidence followed by a valid TOML with a conflicting key binding.
    let fixture = Fixture::new();
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    record_baseline(fixture.request(&result, &fixture.root.join("baselines")))
        .expect("record valid first artifact");
    let conflicting = COMPLETE_F1_RESULT.replacen(
        "result_id = \"\"",
        &format!("result_id = \"{}\"", "e".repeat(64)),
        1,
    );
    fs::write(
        fixture
            .root
            .join("baselines")
            .join(format!("{}.toml", "e".repeat(64))),
        conflicting,
    )
    .expect("write conflicting later artifact");
    let manifests = load_manifest_root(&fixture.root).expect("load conflict root");
    // When: every discovered artifact is validated.
    let error = manifests
        .validate_stage_with("f1", &fixture.ancestry)
        .expect_err("later identity conflict must fail");
    // Then: the later conflict is not hidden by the earlier valid artifact.
    assert_eq!(error.kind(), EvalErrorKind::IdentityMismatch);
}

#[test]
fn valid_recording_creates_output_only_after_validation() {
    // Given: absent output directories for valid and invalid results.
    let fixture = Fixture::new();
    let valid = fixture.write_result(COMPLETE_F1_RESULT);
    let valid_output = fixture.root.join("created-after-validation");
    let invalid_path = fixture.root.join("invalid-result.toml");
    let invalid = COMPLETE_F1_RESULT.replacen("freshness = \"current\"", "freshness = \"stale\"", 1);
    fs::write(&invalid_path, invalid).expect("write invalid result");
    let invalid_output = fixture.root.join("never-created");
    // When: invalid and valid recording are attempted.
    let invalid_error = record_baseline(fixture.request(&invalid_path, &invalid_output))
        .expect_err("invalid result fails before directory creation");
    let artifact = record_baseline(fixture.request(&valid, &valid_output))
        .expect("valid result creates output");
    // Then: only the validated path creates a directory and artifact.
    assert_eq!(invalid_error.kind(), EvalErrorKind::StaleResult);
    assert!(!invalid_output.exists());
    assert!(artifact.path().is_file());
}

#[test]
fn stale_first_temp_collision_retries_and_leaves_only_final_artifact() {
    // Given: the first process-local temporary publication name already exists.
    let fixture = Fixture::new();
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    let output = fixture.root.join("collision-output");
    fs::create_dir(&output).expect("create collision output");
    let stale = output.join(format!(
        ".intel-eval-{KEY}-{}-0.tmp",
        std::process::id()
    ));
    fs::write(&stale, "stale").expect("write stale temp collision");
    // When: a valid result is published.
    let artifact = record_baseline(fixture.request(&result, &output)).expect("retry stale temp");
    // Then: publication succeeds without touching the unowned stale file.
    assert!(artifact.path().is_file());
    assert_eq!(fs::read_to_string(&stale).expect("read stale temp"), "stale");
    assert_eq!(fs::read_dir(&output).expect("read collision output").count(), 2);
}
