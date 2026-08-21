fn replace_line(text: &str, prefix: &str, replacement: &str) -> String {
    text.lines()
        .map(|line| if line.starts_with(prefix) { replacement } else { line })
        .collect::<Vec<_>>()
        .join("\n")
}

fn reverse_array_line(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|line| {
            if !line.starts_with(prefix) {
                return line.to_owned();
            }
            let (name, values) = line.split_once(" = [").expect("array assignment");
            let mut items = values.trim_end_matches(']').split(", ").collect::<Vec<_>>();
            items.reverse();
            format!("{name} = [{}]", items.join(", "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn rejects_empty_artifacts_revision_shape_mismatch_and_none_policy_exclusion() {
    // Given: three semantically incomplete identity/result variants.
    let empty_artifacts = replace_line(COMPLETE_F1_RESULT, "artifact_digests =", "artifact_digests = []");
    let wrong_revision_shape = COMPLETE_F1_RESULT.replacen(
        "source_revision = \"sha256:1111111111111111111111111111111111111111111111111111111111111111\"",
        "source_revision = \"1111111111111111111111111111111111111111\"",
        1,
    );
    let none_policy = COMPLETE_F1_RESULT
        .replacen("gate_id = \"f1-t0-match-correctness-v1\"", "gate_id = \"f1-t0-output-bounds-v1\"", 1)
        .replacen(
            "corpus_id = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\", scope = \"intentional-difference-fixture\"",
            "corpus_id = \"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\", scope = \"intentional-difference-fixture\"",
            1,
        );
    // When/Then: none can satisfy complete_nonempty evidence.
    for result in [empty_artifacts, wrong_revision_shape, none_policy] {
        assert_rejected(&result, EvalErrorKind::InvalidResult);
    }
}

#[test]
fn rejects_malformed_utc_components() {
    // Given: lexically ordered timestamps with impossible calendar components.
    let result = COMPLETE_F1_RESULT.replacen(
        "measurement_started_at = \"2026-08-16T00:00:00Z\"",
        "measurement_started_at = \"2026-02-30T00:00:00Z\"",
        1,
    );
    // When/Then: strict UTC parsing rejects the impossible day.
    assert_rejected(&result, EvalErrorKind::InvalidResult);
}

#[test]
fn missing_metrics_and_coverage_have_stable_kinds() {
    // Given: each required nested result table is entirely absent.
    let missing_metrics = COMPLETE_F1_RESULT
        .lines()
        .filter(|line| *line != "[metrics]" && !line.starts_with("passed_checks ="))
        .collect::<Vec<_>>()
        .join("\n");
    let missing_coverage = COMPLETE_F1_RESULT
        .lines()
        .filter(|line| !line.starts_with("coverage ="))
        .collect::<Vec<_>>()
        .join("\n");
    // When/Then: missing metrics and coverage are classified deterministically.
    assert_rejected(&missing_metrics, EvalErrorKind::MissingMetric);
    assert_rejected(&missing_coverage, EvalErrorKind::InvalidResult);
}

#[test]
fn complete_result_sets_are_order_insensitive() {
    // Given: complete identities, gates, coverage, and checks in different orders.
    let result = reverse_array_line(COMPLETE_F1_RESULT, "corpus_ids =");
    let result = reverse_array_line(&result, "gate_ids =");
    let result = reverse_array_line(&result, "passed_checks =");
    let fixture = Fixture::new();
    let path = fixture.write_result(&result);
    let output = fixture.root.join("order-output");
    fs::create_dir(&output).expect("create order output");
    // When: the complete result is recorded.
    let artifact = record_baseline(fixture.request(&path, &output)).expect("unordered sets pass");
    // Then: order alone does not invalidate evidence.
    assert!(artifact.path().is_file());
}

#[test]
fn complete_result_sets_reject_duplicates_missing_and_extra_members() {
    // Given: duplicate, missing, and extra set members.
    let duplicate_gate = COMPLETE_F1_RESULT.replacen(
        "\"privacy-v1\"",
        "\"workspace-health-v1\"",
        1,
    );
    let missing_corpus = COMPLETE_F1_RESULT.replacen(
        ", \"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"",
        "",
        1,
    );
    let extra_check = COMPLETE_F1_RESULT.replacen(
        "\"execution_completion_distinct_from_render_truncation\"]",
        "\"execution_completion_distinct_from_render_truncation\", \"extra_check\"]",
        1,
    );
    // When/Then: exact unique-set semantics reject all three.
    assert_rejected(&duplicate_gate, EvalErrorKind::DuplicateIdentity);
    assert_rejected(&missing_corpus, EvalErrorKind::IdentityMismatch);
    assert_rejected(&extra_check, EvalErrorKind::MissingMetric);
}

#[test]
fn nonempty_wrong_recording_result_id_is_identity_mismatch() {
    // Given: recording input claims an unrelated nonempty content identity.
    let result = COMPLETE_F1_RESULT.replacen(
        "result_id = \"\"",
        "result_id = \"9999999999999999999999999999999999999999999999999999999999999999\"",
        1,
    );
    // When/Then: the binding failure is classified as identity mismatch.
    assert_rejected(&result, EvalErrorKind::IdentityMismatch);
}
