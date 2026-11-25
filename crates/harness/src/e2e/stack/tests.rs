use super::*;
use std::os::unix::fs::PermissionsExt;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_profile_and_config_when_written_then_only_the_owner_can_access_them() -> TestResult {
    let root = tempfile::tempdir()?;
    ["credentials", "config"].iter().try_for_each(|name| {
        let path = root.path().join(name);
        write_secret(&path, "private contents")?;
        assert_eq!(
            std::fs::metadata(&path)?.permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(std::fs::read_to_string(&path)?, "private contents");
        assert!(write_secret(&path, "replacement").is_err());
        assert_eq!(std::fs::read_to_string(&path)?, "private contents");
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn given_emulator_credentials_at_or_above_bound_when_read_then_only_bounded_utf8_is_accepted()
-> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("issued");
    std::fs::write(&path, vec![b'a'; 65_536])?;
    assert_eq!(read_issued(&path)?.len(), 65_536);
    std::fs::write(&path, vec![b'a'; 65_537])?;
    let error = read_issued(&path)
        .err()
        .ok_or("oversized credentials accepted")?;
    assert!(error.to_string().contains("input byte budget exceeded"));
    std::fs::write(&path, [0xff])?;
    assert!(matches!(read_issued(&path), Err(HarnessError::Invalid(_))));
    Ok(())
}
