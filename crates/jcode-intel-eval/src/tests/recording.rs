#[test]
fn records_the_exact_content_addressed_filename() {
    // Given: a complete canonical F1 result and its independently computed SHA-256 key.
    let fixture = Fixture::new();
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    let output = fixture.root.join("output");
    fs::create_dir(&output).expect("create output");
    let expected_filename = format!("{KEY}.toml");
    assert_eq!(expected_content_key(COMPLETE_F1_RESULT), KEY);
    // When: the recorder writes the baseline artifact.
    let artifact = record_baseline(fixture.request(&result, &output)).expect("record baseline");
    // Then: the returned key and filename bind exactly to the independent digest.
    assert_eq!(artifact.content_key(), KEY);
    assert_eq!(
        artifact.path().file_name().unwrap(),
        expected_filename.as_str()
    );
}

#[test]
fn changing_one_input_changes_the_content_address() {
    // Given: two complete inputs that differ in one measured artifact digest.
    let fixture = Fixture::new();
    let first = fixture.write_result(COMPLETE_F1_RESULT);
    let second_text = COMPLETE_F1_RESULT.replacen(
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "sha256:9999999999999999999999999999999999999999999999999999999999999999",
        1,
    );
    let second = fixture.root.join("second-result.toml");
    fs::write(&second, &second_text).expect("write changed result");
    let first_key = expected_content_key(COMPLETE_F1_RESULT);
    let second_key = expected_content_key(&second_text);
    let first_output = fixture.root.join("first-output");
    let second_output = fixture.root.join("second-output");
    fs::create_dir(&first_output).expect("create first output");
    fs::create_dir(&second_output).expect("create second output");
    // When: each input is recorded independently.
    let first_artifact = record_baseline(fixture.request(&first, &first_output)).expect("first");
    let second_artifact =
        record_baseline(fixture.request(&second, &second_output)).expect("second");
    // Then: each key matches its input digest and the keys differ.
    assert_eq!(first_artifact.content_key(), first_key);
    assert_eq!(second_artifact.content_key(), second_key);
    assert_ne!(first_artifact.content_key(), second_artifact.content_key());
}

#[test]
fn refuses_to_overwrite_an_existing_content_address() {
    // Given: a complete result already recorded at its content address.
    let fixture = Fixture::new();
    let result = fixture.write_result(COMPLETE_F1_RESULT);
    let output = fixture.root.join("output");
    fs::create_dir(&output).expect("create output");
    let artifact = record_baseline(fixture.request(&result, &output)).expect("initial record");
    let original = fs::read(artifact.path()).expect("read original artifact");
    // When: the recorder targets an existing content key.
    let error = record_baseline(fixture.request(&result, &output))
        .expect_err("overwrite must be refused");
    // Then: refusal is explicit and the existing artifact remains authoritative.
    assert_eq!(error.kind(), EvalErrorKind::OverwriteRefused);
    assert_eq!(fs::read(artifact.path()).unwrap(), original);
}
