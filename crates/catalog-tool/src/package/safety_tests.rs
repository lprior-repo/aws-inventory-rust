//! Destination isolation and atomic publication regressions.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::{atomic_write, package};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn source(parent: &Path) -> io::Result<std::path::PathBuf> {
    let root = parent.join("source");
    fs::create_dir(&root)?;
    fs::write(root.join("a.txt"), b"precious\n")?;
    Ok(root)
}

fn names(parent: &Path) -> io::Result<Vec<std::ffi::OsString>> {
    let mut names = fs::read_dir(parent)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<io::Result<Vec<_>>>()?;
    names.sort();
    Ok(names)
}

fn invalid_input(root: &Path, destination: &Path) -> TestResult {
    let error = package(root, destination)
        .err()
        .ok_or("source destination accepted")?;
    let io = error
        .downcast_ref::<io::Error>()
        .ok_or("expected typed I/O error")?;
    assert_eq!(io.kind(), io::ErrorKind::InvalidInput);
    let message = io.to_string();
    assert!(message.contains(&root.display().to_string()));
    assert!(message.contains(&destination.display().to_string()));
    assert!(message.contains("choose a destination outside the root"));
    Ok(())
}

#[test]
fn given_directory_destination_when_publish_fails_then_no_temporary_remains() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let destination = fixture.path().join("out.zip");
    fs::create_dir(&destination)?;
    let before = names(fixture.path())?;
    // When
    let error = package(&root, &destination)
        .err()
        .ok_or("directory replaced")?;
    // Then
    assert!(error.downcast_ref::<io::Error>().is_some());
    assert_eq!(names(fixture.path())?, before);
    assert!(destination.is_dir());
    Ok(())
}

#[test]
fn given_symlink_temporary_sibling_when_packaged_then_victim_is_untouched() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let destination = fixture.path().join("out.zip");
    let victim = fixture.path().join("victim");
    let sibling = destination.with_extension("building.zip");
    fs::write(&victim, b"unrelated\n")?;
    std::os::unix::fs::symlink(&victim, &sibling)?;
    // When
    let receipt = package(&root, &destination)?;
    // Then
    assert_eq!(fs::read(&victim)?, b"unrelated\n");
    assert_eq!(fs::read_link(&sibling)?, victim);
    assert_eq!(receipt.path, destination.to_string_lossy());
    Ok(())
}

#[test]
fn given_regular_temporary_sibling_when_packaged_then_existing_file_is_untouched() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let destination = fixture.path().join("out.zip");
    let sibling = destination.with_extension("building.zip");
    fs::write(&sibling, b"unrelated\n")?;
    // When
    let receipt = package(&root, &destination)?;
    // Then
    assert_eq!(fs::read(&sibling)?, b"unrelated\n");
    assert_eq!(receipt.path, destination.to_string_lossy());
    Ok(())
}

#[test]
fn given_destination_inside_root_when_packaged_then_rejected_before_writes() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let destination = root.join("release.zip");
    // When / Then
    invalid_input(&root, &destination)?;
    assert_eq!(names(&root)?, vec![std::ffi::OsString::from("a.txt")]);
    Ok(())
}

#[test]
fn given_source_destination_when_packaged_then_typed_error_preserves_source() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let destination = root.join("a.txt");
    // When / Then
    invalid_input(&root, &destination)?;
    assert_eq!(fs::read(&destination)?, b"precious\n");
    assert_eq!(names(&root)?, vec![std::ffi::OsString::from("a.txt")]);
    Ok(())
}

#[test]
fn given_symlinked_destination_parent_when_inside_root_then_rejected() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let alias = fixture.path().join("alias");
    std::os::unix::fs::symlink(&root, &alias)?;
    invalid_input(&root, &alias.join("new/release.zip"))?;
    assert_eq!(names(&root)?, vec![std::ffi::OsString::from("a.txt")]);
    Ok(())
}

#[test]
fn given_symlinked_root_when_destination_is_inside_then_rejected() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let alias = fixture.path().join("alias");
    std::os::unix::fs::symlink(&root, &alias)?;
    invalid_input(&alias, &root.join("release.zip"))?;
    assert_eq!(names(&root)?, vec![std::ffi::OsString::from("a.txt")]);
    Ok(())
}

#[test]
fn given_destination_symlink_when_packaged_then_link_is_replaced_not_followed() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let destination = fixture.path().join("out.zip");
    let victim = root.join("a.txt");
    std::os::unix::fs::symlink(&victim, &destination)?;
    let receipt = package(&root, &destination)?;
    assert_eq!(fs::read(&victim)?, b"precious\n");
    assert!(fs::symlink_metadata(&destination)?.file_type().is_file());
    assert_eq!(receipt.path, destination.to_string_lossy());
    Ok(())
}

#[test]
fn given_old_external_destination_when_packaged_twice_then_bytes_are_identical() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let root = source(fixture.path())?;
    let destination = fixture.path().join("out.zip");
    fs::write(&destination, b"yesterday\n")?;
    let first = package(&root, &destination)?;
    let bytes = fs::read(&destination)?;
    let second = package(&root, &destination)?;
    assert_eq!(fs::read(&destination)?, bytes);
    assert_eq!(first, second);
    Ok(())
}

#[test]
fn given_writer_failure_when_partial_bytes_exist_then_temporary_is_removed() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let destination = fixture.path().join("out.zip");
    fs::write(&destination, b"old\n")?;
    let before = names(fixture.path())?;
    let result: Result<(), Box<dyn std::error::Error>> = atomic_write(&destination, |file| {
        file.write_all(b"partial")?;
        Err(io::Error::other("injected write failure").into())
    });
    let error = result.err().ok_or("writer failure ignored")?;
    assert_eq!(error.to_string(), "injected write failure");
    assert_eq!(fs::read(&destination)?, b"old\n");
    assert_eq!(names(fixture.path())?, before);
    Ok(())
}
