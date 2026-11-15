include!("smithy_helpers.rs");

#[test]
fn explicit_modes_keep_stdout_empty_and_default_prints() -> Result<(), Box<dyn std::error::Error>> {
    let dir = fixture()?;
    let output = report(dir.path(), &["--write"])?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        std::fs::read(dir.path().join("catalog/aws-models.report"))?,
        REPORT.as_bytes()
    );
    let checked = report(dir.path(), &["--check"])?;
    assert_eq!(checked.status.code(), Some(0));
    assert_eq!(checked.stdout, b"");
    assert_eq!(checked.stderr, b"");
    let printed = report(dir.path(), &[])?;
    assert_eq!(printed.status.code(), Some(0));
    assert_eq!(printed.stdout, REPORT.as_bytes());
    assert_eq!(printed.stderr, b"");
    Ok(())
}

#[test]
fn explicit_mode_failure_keeps_stdout_empty_and_existing_file()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = fixture()?;
    let path = dir.path().join("catalog/aws-models.report");
    write_fixture(dir.path(), "catalog/aws-models.report", "preserve")?;
    let output = report(dir.path(), &["--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(!output.stderr.is_empty());
    assert_eq!(std::fs::read(&path)?, b"preserve");
    Ok(())
}

#[cfg(unix)]
#[test]
fn write_replaces_symlink_without_changing_its_target() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;
    let dir = fixture()?;
    let destination = dir.path().join("catalog/aws-models.report");
    let target = dir.path().join("target");
    std::fs::create_dir_all(destination.parent().ok_or("missing parent")?)?;
    std::fs::write(&target, "original")?;
    symlink(&target, &destination)?;
    let output = report(dir.path(), &["--write"])?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"");
    assert_eq!(std::fs::read(&target)?, b"original");
    assert!(
        !std::fs::symlink_metadata(destination)?
            .file_type()
            .is_symlink()
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn write_replaces_only_the_report_hardlink() -> Result<(), Box<dyn std::error::Error>> {
    let dir = fixture()?;
    let destination = dir.path().join("catalog/aws-models.report");
    let alias = dir.path().join("alias");
    write_fixture(dir.path(), "catalog/aws-models.report", "original")?;
    std::fs::hard_link(&destination, &alias)?;
    let output = report(dir.path(), &["--write"])?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"");
    assert_eq!(std::fs::read(&alias)?, b"original");
    assert_eq!(std::fs::read(destination)?, REPORT.as_bytes());
    Ok(())
}

#[test]
fn failed_atomic_write_preserves_destination_and_cleans_temporary_files()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = fixture()?;
    let destination = dir.path().join("catalog/aws-models.report");
    std::fs::create_dir_all(&destination)?;
    std::fs::write(destination.join("sentinel"), b"keep")?;
    let output = report(dir.path(), &["--write"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(std::fs::read(destination.join("sentinel"))?, b"keep");
    let entries = std::fs::read_dir(destination.parent().ok_or("missing parent")?)?;
    assert!(
        entries
            .filter_map(Result::ok)
            .all(|entry| { !entry.file_name().to_string_lossy().starts_with(".tmp") })
    );
    Ok(())
}

#[test]
fn oversized_comparison_is_bounded_and_silent_on_stdout() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = fixture()?;
    let path = dir.path().join("catalog/aws-models.report");
    std::fs::create_dir_all(path.parent().ok_or("missing parent")?)?;
    std::fs::File::create(&path)?.set_len(4_194_305)?;
    let output = report(dir.path(), &["--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(String::from_utf8(output.stderr)?.contains("comparison file exceeds 4194304 bytes"));
    Ok(())
}
