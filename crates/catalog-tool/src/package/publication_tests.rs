//! Manifest alias, output permission and receipt-before-publication contracts.

use std::fs::{self, File};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::PermissionsExt;

use super::package;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn expected_manifest() -> String {
    format!("{}  a.txt\n", super::sha256_hex(b"precious\n"))
}

fn entries(path: &std::path::Path) -> std::io::Result<usize> {
    Ok(fs::read_dir(path)?
        .collect::<std::io::Result<Vec<_>>>()?
        .len())
}

#[test]
fn given_manifest_symlink_when_packaged_then_source_bytes_are_preserved() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("source");
    fs::create_dir(&root)?;
    let source = root.join("a.txt");
    let manifest = root.join("MANIFEST.sha256");
    fs::write(&source, b"precious\n")?;
    std::os::unix::fs::symlink(&source, &manifest)?;
    // When
    let receipt = package(&root, &fixture.path().join("out.zip"))?;
    // Then
    assert_eq!(fs::read(&source)?, b"precious\n");
    assert!(fs::symlink_metadata(&manifest)?.file_type().is_file());
    assert_eq!(fs::read_to_string(manifest)?, expected_manifest());
    assert_eq!(receipt.files, 2);
    Ok(())
}

#[test]
fn given_manifest_hard_link_when_packaged_then_source_bytes_are_preserved() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("source");
    fs::create_dir(&root)?;
    let source = root.join("a.txt");
    let manifest = root.join("MANIFEST.sha256");
    fs::write(&source, b"precious\n")?;
    fs::hard_link(&source, &manifest)?;
    // When
    let receipt = package(&root, &fixture.path().join("out.zip"))?;
    // Then
    assert_eq!(fs::read(&source)?, b"precious\n");
    assert_eq!(fs::read_to_string(manifest)?, expected_manifest());
    assert_eq!(receipt.files, 2);
    Ok(())
}

#[test]
fn given_current_umask_when_packaged_then_outputs_match_file_create_mode() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("source");
    fs::create_dir(&root)?;
    let reference = File::create(fixture.path().join("reference"))?;
    let expected = reference.metadata()?.permissions().mode();
    let destination = fixture.path().join("out.zip");
    // When
    package(&root, &destination)?;
    // Then
    assert_eq!(fs::metadata(destination)?.permissions().mode(), expected);
    assert_eq!(
        fs::metadata(root.join("MANIFEST.sha256"))?
            .permissions()
            .mode(),
        expected
    );
    Ok(())
}

#[test]
fn given_non_utf8_destination_when_receipt_fails_then_sentinel_is_preserved() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("source");
    fs::create_dir(&root)?;
    let destination = fixture
        .path()
        .join(std::ffi::OsString::from_vec(vec![0xff]));
    fs::write(&destination, b"sentinel\n")?;
    let before = entries(fixture.path())?;
    // When
    let error = package(&root, &destination)
        .err()
        .ok_or("invalid receipt accepted")?;
    // Then
    assert_eq!(error.to_string(), "packaged path is not valid UTF-8");
    assert_eq!(fs::read(destination)?, b"sentinel\n");
    assert_eq!(entries(fixture.path())?, before);
    Ok(())
}
