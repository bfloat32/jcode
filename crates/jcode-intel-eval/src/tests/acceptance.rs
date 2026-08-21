#[test]
fn rejects_unknown_fields_at_nested_manifest_boundaries() {
    // Given: a valid manifest with an unknown nested oracle-contract field.
    let fixture = Fixture::new();
    let corpora = CORPORA.replacen(
        "comparison_scope = \"supported_intersection\"",
        "comparison_scope = \"supported_intersection\"\nunexpected_nested = true",
        1,
    );
    fs::write(fixture.root.join("corpora.toml"), corpora).expect("write nested unknown");
    // When: the strict boundary loads the root.
    let error = load_manifest_root(&fixture.root).expect_err("nested unknown must fail");
    // Then: nested unknowns retain the stable structural classification.
    assert_eq!(error.kind(), EvalErrorKind::UnknownField);
}

#[test]
fn rejects_unresolved_manifest_references() {
    // Given: F1 references a gate absent from the gate manifest.
    let fixture = Fixture::new();
    let runner = RUNNER.replacen(
        "\"f1-t0-output-bounds-v1\",",
        "\"missing-gate-v1\",",
        1,
    );
    fs::write(fixture.root.join("release-runner.toml"), runner)
        .expect("write unresolved reference");
    // When: the root is loaded.
    let error = load_manifest_root(&fixture.root).expect_err("unresolved reference must fail");
    // Then: reference failure is distinct from malformed TOML.
    assert_eq!(error.kind(), EvalErrorKind::UnresolvedReference);
}

#[test]
fn rejects_results_missing_required_gate_metrics() {
    // Given: a complete result omitting one F1-required check.
    let result = COMPLETE_F1_RESULT.replacen(
        "\"execution_completion_distinct_from_render_truncation\"",
        "\"unrelated_metric\"",
        1,
    );
    // When: the recorder reconciles required metrics.
    // Then: the missing metric has its own stable classification.
    assert_rejected(&result, EvalErrorKind::MissingMetric);
}

#[test]
fn reports_the_exact_missing_f1_stage_evidence_error() {
    // Given: valid manifests and an empty official-shaped baseline directory.
    let fixture = Fixture::new();
    let manifests = load_manifest_root(&fixture.root).expect("valid manifest root");
    // When: F1 stage evidence is validated.
    let error = manifests
        .validate_stage_with("f1", &fixture.ancestry)
        .expect_err("empty F1 baselines must fail");
    // Then: the deterministic control message is exact.
    assert_eq!(error.kind(), EvalErrorKind::MissingStageEvidence);
    assert_eq!(error.to_string(), "concrete F1 baseline missing");
}

#[test]
fn stored_result_id_filename_and_recomputation_are_bound() {
    // Given: a complete result with a blank recording-time result identity.
    let fixture = Fixture::new();
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    let output = fixture.root.join("output");
    fs::create_dir(&output).expect("create output");
    // When: the result is recorded.
    let artifact = record_baseline(fixture.request(&result, &output)).expect("record baseline");
    // Then: stored identity, filename, and canonical recomputation all equal the fixed key.
    let stored = fs::read_to_string(artifact.path()).expect("read stored artifact");
    assert!(stored.contains(&format!("result_id = \"{KEY}\"")));
    assert_eq!(expected_content_key(&stored), KEY);
    assert_eq!(artifact.path().file_stem().unwrap(), KEY);
}

#[test]
fn validates_a_recorded_stage_from_discovered_baseline_evidence() {
    // Given: complete F1 evidence recorded in the manifest baseline directory.
    let fixture = Fixture::new();
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    record_baseline(fixture.request(&result, &fixture.root.join("baselines")))
        .expect("record stage baseline");
    let manifests = load_manifest_root(&fixture.root).expect("reload manifest root");
    // When: F1 stage evidence is validated.
    let validated = manifests
        .validate_stage_with("f1", &fixture.ancestry)
        .expect("validate F1 stage");
    // Then: validation identifies the requested stage.
    assert_eq!(validated, "f1");
}

#[test]
fn concurrent_recorders_publish_once_without_overwrite() {
    use std::sync::{Arc, Barrier};

    // Given: two recorders released simultaneously for the same content key.
    let fixture = Arc::new(Fixture::new());
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    let output = fixture.root.join("output");
    fs::create_dir(&output).expect("create output");
    let barrier = Arc::new(Barrier::new(3));
    let handles = (0..2)
        .map(|_| {
            let fixture = Arc::clone(&fixture);
            let result = result.clone();
            let output = output.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                record_baseline(fixture.request(&result, &output))
            })
        })
        .collect::<Vec<_>>();
    // When: both attempts race through publication.
    barrier.wait();
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().expect("recorder thread"))
        .collect::<Vec<_>>();
    // Then: exactly one publishes and one receives overwrite refusal.
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter_map(|outcome| outcome.as_ref().err())
            .map(super::EvalError::kind)
            .collect::<Vec<_>>(),
        [EvalErrorKind::OverwriteRefused]
    );
}

#[test]
fn cleans_owned_temporary_artifact_when_prepublication_fails() {
    // Given: a recoverable injected failure after the owned temporary file is synced.
    let fixture = Fixture::new();
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    let output = fixture.root.join("output");
    fs::create_dir(&output).expect("create output");
    // When: publication is interrupted before the no-replace link.
    let error = super::recording::record_baseline_with_prepublication_failure(
        fixture.request(&result, &output),
    )
    .expect_err("injected publication failure");
    // Then: no final or owned temporary artifact remains discoverable.
    assert_eq!(error.kind(), EvalErrorKind::Io);
    assert_eq!(fs::read_dir(&output).expect("read output").count(), 0);
}
