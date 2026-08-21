#[test]
fn loads_the_confirmed_manifest_root_when_contracts_are_valid() {
    // Given: the three independently confirmed evaluation manifests.
    let fixture = Fixture::new();
    // When: the strict manifest boundary loads the root.
    let manifests = load_manifest_root(&fixture.root).expect("valid manifest root");
    // Then: all closed stage identities are available to consumers.
    assert_eq!(
        manifests.stage_ids(),
        ["f0", "f1", "f2", "f3", "f4", "f5", "f6", "f7"]
    );
}

#[test]
fn rejects_unknown_fields_in_a_manifest() {
    // Given: an otherwise valid root with one unrecognized top-level field.
    let fixture = Fixture::new();
    fs::write(
        fixture.root.join("corpora.toml"),
        CORPORA.replacen(
            "schema_version = 1",
            "schema_version = 1\nunexpected = true",
            1,
        ),
    )
    .expect("write unknown field fixture");
    // When: the root is loaded.
    let error = load_manifest_root(&fixture.root).expect_err("unknown field must fail");
    // Then: rejection is classified structurally.
    assert_eq!(error.kind(), EvalErrorKind::UnknownField);
}

#[test]
fn rejects_duplicate_manifest_identities() {
    // Given: two complete gate records with the same identity.
    let fixture = Fixture::new();
    let gates = GATES.replacen(
        "gate_id = \"f1-t0-match-correctness-v1\"",
        "gate_id = \"workspace-health-v1\"",
        1,
    );
    fs::write(fixture.root.join("gates.toml"), gates).expect("write duplicate gate fixture");
    // When: the root is loaded.
    let error = load_manifest_root(&fixture.root).expect_err("duplicate ID must fail");
    // Then: duplicate identity is distinct from TOML syntax failure.
    assert_eq!(error.kind(), EvalErrorKind::DuplicateIdentity);
}

#[test]
fn rejects_mutable_revisions() {
    // Given: a corpus identity whose source revision is a mutable branch.
    let result = COMPLETE_F1_RESULT.replacen(
        "source_revision_kind = \"generated_from_pinned_inputs\"",
        "source_revision_kind = \"branch\"",
        1,
    );
    // When: baseline recording validates its identity inputs.
    // Then: mutable revision input is rejected.
    assert_rejected(&result, EvalErrorKind::MutableRevision);
}

#[test]
fn rejects_missing_corpus_config_hardware_gate_or_exclusion_identities() {
    for field in [
        "corpus_ids",
        "config_id",
        "hardware_id",
        "gate_ids",
        "exclusion_ids",
    ] {
        // Given: a complete result missing one required identity field.
        let result = COMPLETE_F1_RESULT
            .lines()
            .filter(|line| !line.starts_with(&format!("{field} =")))
            .collect::<Vec<_>>()
            .join("\n");
        // When: baseline recording validates the result bundle.
        // Then: the missing identity is rejected uniformly.
        assert_rejected(&result, EvalErrorKind::MissingIdentity);
    }
}

#[test]
fn rejects_planned_partial_and_stale_results() {
    let cases = [
        (
            "lifecycle_state = \"complete\"",
            "lifecycle_state = \"planned\"",
            EvalErrorKind::PlannedResult,
        ),
        (
            "lifecycle_state = \"complete\"",
            "lifecycle_state = \"partial\"",
            EvalErrorKind::PartialResult,
        ),
        (
            "freshness = \"current\"",
            "freshness = \"stale\"",
            EvalErrorKind::StaleResult,
        ),
    ];
    for (valid, invalid, expected) in cases {
        // Given: one non-passing result state.
        let result = COMPLETE_F1_RESULT.replacen(valid, invalid, 1);
        // When: the recorder evaluates baseline eligibility.
        // Then: the exact state is machine-classified.
        assert_rejected(&result, expected);
    }
}

#[test]
fn rejects_post_hoc_results_using_revision_ancestry() {
    // Given: complete F1 evidence whose measured revision is not an ancestor of implementation.
    let fixture = Fixture::with_ordering(false);
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    // When: baseline recording checks the injected deterministic revision graph.
    let error = record_baseline(fixture.request(&result, &fixture.root.join("baselines")))
        .expect_err("post-hoc baseline must fail");
    // Then: ancestry ordering, not an invented result kind, classifies the rejection.
    assert_eq!(error.kind(), EvalErrorKind::PostHocBaseline);
}

#[test]
fn rejects_stage_mismatches() {
    // Given: a request for F1 carrying an otherwise complete F2 result.
    let result = COMPLETE_F1_RESULT.replacen("stage_id = \"f1\"", "stage_id = \"f2\"", 1);
    // When: baseline recording compares requested and measured stages.
    // Then: the stage mismatch is rejected independently.
    assert_rejected(&result, EvalErrorKind::StageMismatch);
}

#[test]
fn rejects_identity_reference_mismatches() {
    // Given: a complete result whose config reference differs from its identity record.
    let result = COMPLETE_F1_RESULT.replacen(
        "config_id = \"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc\"",
        "config_id = \"9999999999999999999999999999999999999999999999999999999999999999\"",
        1,
    );
    // When: baseline recording resolves the referenced config.
    // Then: the reference mismatch is rejected independently.
    assert_rejected(&result, EvalErrorKind::IdentityMismatch);
}
