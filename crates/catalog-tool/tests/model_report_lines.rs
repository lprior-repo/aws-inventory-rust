include!("smithy_helpers.rs");

#[test]
fn check_detects_a_missing_final_newline() -> Result<(), Box<dyn std::error::Error>> {
    // Given a report differing only in its final newline.
    let dir = fixture()?;
    let changed = REPORT
        .strip_suffix('\n')
        .ok_or("fixture has no final newline")?;
    write_fixture(dir.path(), "catalog/aws-models.report", changed)?;
    // When checking, then the byte difference is not trimmed away.
    let output = report(dir.path(), &["--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, REPORT.as_bytes());
    assert_eq!(
        String::from_utf8(output.stderr)?,
        concat!(
            "line 16 differs: expected Some(\"protocol aws.protocols#restJson1 1\\n\"), ",
            "got Some(\"protocol aws.protocols#restJson1 1\")\n\n"
        )
    );
    Ok(())
}

#[test]
fn check_reports_each_missing_line_not_only_a_line_count() -> Result<(), Box<dyn std::error::Error>>
{
    // Given only the first fourteen lines of a report.
    let dir = fixture()?;
    let changed: String = REPORT.split_inclusive('\n').take(14).collect();
    write_fixture(dir.path(), "catalog/aws-models.report", &changed)?;
    // When checking, then both absent lines are diagnosed explicitly.
    let output = report(dir.path(), &["--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, REPORT.as_bytes());
    assert_eq!(
        String::from_utf8(output.stderr)?,
        concat!(
            "line 15 differs: expected Some(\"cbor_services 0\\n\"), got None\n",
            "line 16 differs: expected Some(\"protocol aws.protocols#restJson1 1\\n\"), got None\n\n"
        )
    );
    Ok(())
}

#[test]
fn sparse_oversized_file_fails_the_binary_before_output() -> Result<(), Box<dyn std::error::Error>>
{
    // Given a valid model followed by an oversized sparse model.
    let dir = fixture()?;
    let path = dir.path().join("models/z-oversized.json");
    std::fs::File::create(&path)?.set_len(67_108_865)?;
    // When reporting, then the size error names the file and stdout stays empty.
    let output = report(dir.path(), &[])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!("file exceeds 67108864 bytes: {}\n", path.display())
    );
    Ok(())
}

#[test]
fn malformed_models_fail_in_ascending_path_order() -> Result<(), Box<dyn std::error::Error>> {
    // Given malformed files created in reverse lexical order.
    let dir = fixture()?;
    write_fixture(dir.path(), "models/z-last.json", "{")?;
    write_fixture(dir.path(), "models/a-first.json", "{")?;
    // When reporting, then the lexically first filename wins, with no partial output.
    let output = report(dir.path(), &[])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!(
            "invalid JSON in {}: EOF while parsing an object at line 1 column 1\n",
            dir.path().join("models/a-first.json").display()
        )
    );
    Ok(())
}
