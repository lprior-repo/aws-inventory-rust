use super::*;

fn unchanged_failure(output: &Output, path: &Path, original: &[u8]) -> TestResult {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("catalog differs"));
    assert_eq!(fs::read(path)?, original);
    assert!(!path.with_file_name("aws-api.catalog.tmp").exists());
    Ok(())
}

#[test]
fn check_rejects_same_length_one_byte_drift_without_output_or_rewrite() -> TestResult {
    let source = source()?;
    let root = root()?;
    let path = root.path().join("catalog/aws-api.catalog");
    let output = compile(source.path(), root.path(), &["--write"])?;
    let mut bytes = fs::read(&path)?;
    success(&output);
    *bytes.first_mut().ok_or("generated catalog is empty")? ^= 1;
    fs::write(&path, &bytes)?;
    let output = compile(source.path(), root.path(), &["--check"])?;
    unchanged_failure(&output, &path, &bytes)
}

#[test]
fn check_rejects_existing_empty_placeholder_without_output_or_rewrite() -> TestResult {
    let source = source()?;
    let root = root()?;
    let path = root.path().join("catalog/aws-api.catalog");
    fs::write(&path, b"")?;
    let output = compile(source.path(), root.path(), &["--check"])?;
    unchanged_failure(&output, &path, b"")
}
