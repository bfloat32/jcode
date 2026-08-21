fn assert_manifest_rejected(file: &str, text: String, expected: EvalErrorKind) {
    let fixture = Fixture::new();
    fs::write(fixture.root.join(file), text).expect("write manifest regression fixture");
    let error = load_manifest_root(&fixture.root).expect_err("manifest mutation must fail");
    assert_eq!(error.kind(), expected);
}

#[test]
fn rejects_gate_stage_mismatch_and_coordinated_weakening() {
    // Given: individually resolved but unauthorized gate and stage contracts.
    let stage_mismatch = GATES.replacen("stage = \"f1\"", "stage = \"f2\"", 1);
    let weakened_gates = GATES
        .replacen("gate_id = \"f1-t0-output-bounds-v1\"", "gate_id = \"f1-weakened-v1\"", 1)
        .replacen("dense_match_count_eq_1000000", "dense_match_count_gte_1", 1)
        .replacen("exclusion_policy = \"none\"", "exclusion_policy = \"reviewed_any\"", 1);
    let weakened_runner = RUNNER.replacen(
        "\"f1-t0-output-bounds-v1\",",
        "\"f1-weakened-v1\",",
        1,
    );
    let fixture = Fixture::new();
    fs::write(fixture.root.join("gates.toml"), weakened_gates).expect("write weakened gates");
    fs::write(fixture.root.join("release-runner.toml"), weakened_runner)
        .expect("write weakened runner");
    // When: exact root contracts are loaded.
    let weakened = load_manifest_root(&fixture.root).expect_err("coordinated weakening must fail");
    // Then: both mutations are invalid manifests despite resolving references.
    assert_eq!(weakened.kind(), EvalErrorKind::InvalidManifest);
    assert_manifest_rejected("gates.toml", stage_mismatch, EvalErrorKind::InvalidManifest);
}

#[test]
fn rejects_duplicate_checks_references_and_changed_stage_definition() {
    // Given: duplicate complete-set members and a changed authored stage field.
    let duplicate_check = GATES.replacen(
        "\"curated_case_count_eq_2000\"",
        "\"generated_case_count_eq_10000\"",
        1,
    );
    let duplicate_reference = RUNNER.replacen(
        "\"f1-t0-scope-completeness-v1\",",
        "\"f1-t0-match-correctness-v1\",",
        1,
    );
    let changed_stage = RUNNER.replacen(
        "first_implementation_task = \"task-6\"",
        "first_implementation_task = \"task-999\"",
        1,
    );
    // When/Then: each closed contract mutation is rejected.
    assert_manifest_rejected("gates.toml", duplicate_check, EvalErrorKind::InvalidManifest);
    assert_manifest_rejected(
        "release-runner.toml",
        duplicate_reference,
        EvalErrorKind::InvalidManifest,
    );
    assert_manifest_rejected(
        "release-runner.toml",
        changed_stage,
        EvalErrorKind::InvalidManifest,
    );
}

#[test]
fn rejects_cross_variant_arbitrary_stage_and_changed_corpus_lists() {
    // Given: corpus records carrying fields or values outside their exact authored variant.
    let cross_variant = CORPORA.replacen(
        "generated_case_count = 10000",
        "generated_case_count = 10000\nminimum_task_count = 200",
        1,
    );
    let arbitrary_stage = CORPORA.replacen("stage = \"f1\"", "stage = \"f9\"", 1);
    let empty_labels = CORPORA.replacen(
        "required_labels = [\"relation\", \"identity\", \"diagnostic\", \"construct\"]",
        "required_labels = []",
        1,
    );
    let changed_fixtures = CORPORA.replacen(
        "required_fixtures = [\"cold_index\", \"incremental_single_file\", \"dense_search\", \"cancellation\"]",
        "required_fixtures = [\"cold_index\"]",
        1,
    );
    // When/Then: every variant violation fails the immutable v1 contract.
    for text in [cross_variant, arbitrary_stage, empty_labels, changed_fixtures] {
        assert_manifest_rejected("corpora.toml", text, EvalErrorKind::InvalidManifest);
    }
}

#[test]
fn duplicate_specs_gates_and_stages_have_one_classification() {
    // Given: one duplicate identity in each authored identity domain.
    let duplicate_spec = format!(
        "{CORPORA}\n[[corpus_spec]]\nspec_id = \"search-supported-v1\"\nstage = \"f1\"\nsource_kinds = [\"public\", \"synthetic\"]\n"
    );
    let duplicate_gate = GATES.replacen(
        "gate_id = \"f1-t0-match-correctness-v1\"",
        "gate_id = \"workspace-health-v1\"",
        1,
    );
    let duplicate_stage = format!(
        "{RUNNER}\n[[stage]]\nstage_id = \"f0\"\nfirst_implementation_task = \"task-1\"\ndefault_feature_state = \"all_intel_features_off\"\nrequired_corpus_spec_ids = []\nrequired_gate_ids = []\nbaseline_required_before_implementation = false\nbaseline_required_before_enable = true\nenable_only_after_passing_result = true\n"
    );
    // When/Then: uniqueness runs before semantic contract validation.
    assert_manifest_rejected("corpora.toml", duplicate_spec, EvalErrorKind::DuplicateIdentity);
    assert_manifest_rejected("gates.toml", duplicate_gate, EvalErrorKind::DuplicateIdentity);
    assert_manifest_rejected(
        "release-runner.toml",
        duplicate_stage,
        EvalErrorKind::DuplicateIdentity,
    );
}
