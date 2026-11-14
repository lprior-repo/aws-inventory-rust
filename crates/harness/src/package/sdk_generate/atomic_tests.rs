//! The SDK generator uses the shared exclusive atomic-file publication contract.

use std::fs;

use super::write;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_preexisting_tmp_file_when_generated_then_unrelated_file_is_preserved() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let destination = fixture.path().join("lib.rs");
    let sibling = fixture.path().join("lib.rs.tmp");
    fs::write(&sibling, b"precious\n")?;
    // When
    write(&destination, "generated\n")?;
    // Then
    assert_eq!(fs::read(&destination)?, b"generated\n");
    assert_eq!(fs::read(&sibling)?, b"precious\n");
    Ok(())
}

#[test]
fn given_tmp_symlink_when_generated_then_victim_and_link_are_preserved() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let destination = fixture.path().join("lib.rs");
    let sibling = fixture.path().join("lib.rs.tmp");
    let victim = fixture.path().join("victim");
    fs::write(&victim, b"precious\n")?;
    std::os::unix::fs::symlink(&victim, &sibling)?;
    // When
    write(&destination, "generated\n")?;
    // Then
    assert_eq!(fs::read(&destination)?, b"generated\n");
    assert_eq!(fs::read(&victim)?, b"precious\n");
    assert_eq!(fs::read_link(&sibling)?, victim);
    Ok(())
}

#[test]
fn given_directory_destination_when_generated_then_no_temporary_file_remains() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let destination = fixture.path().join("lib.rs");
    fs::create_dir(&destination)?;
    // When
    let error = write(&destination, "generated\n")
        .err()
        .ok_or("directory replaced")?;
    // Then
    assert!(matches!(error, crate::HarnessError::Io(_)));
    let entries = fs::read_dir(fixture.path())?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(entries.len(), 1);
    assert!(destination.is_dir());
    Ok(())
}

#[test]
fn given_destination_symlink_when_generated_then_only_link_is_replaced() -> TestResult {
    // Given
    let fixture = tempfile::tempdir()?;
    let destination = fixture.path().join("lib.rs");
    let victim = fixture.path().join("victim");
    fs::write(&victim, b"precious\n")?;
    std::os::unix::fs::symlink(&victim, &destination)?;
    // When
    write(&destination, "generated\n")?;
    // Then
    assert_eq!(fs::read(&destination)?, b"generated\n");
    assert_eq!(fs::read(&victim)?, b"precious\n");
    assert!(fs::symlink_metadata(&destination)?.file_type().is_file());
    Ok(())
}

#[cfg(unix)]
#[test]
fn given_current_umask_when_generated_then_mode_matches_file_create() -> TestResult {
    use std::os::unix::fs::PermissionsExt;
    // Given
    let fixture = tempfile::tempdir()?;
    let reference = fs::File::create(fixture.path().join("reference"))?;
    let expected = reference.metadata()?.permissions().mode();
    let destination = fixture.path().join("lib.rs");
    // When
    write(&destination, "generated\n")?;
    // Then
    assert_eq!(fs::metadata(destination)?.permissions().mode(), expected);
    Ok(())
}
