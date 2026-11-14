include!("smithy_helpers.rs");

use catalog_tool::smithy::{self, Error, ReportMode};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const ADDED: &str = r#"{"shapes":{
"n#Svc":{"type":"service"},
"n#Input":{"type":"structure","members":{"Shared":{},"Aardvark":{}}}}}"#;

fn registry_path(root: &Path) -> PathBuf {
    root.join("catalog/field-ids.txt")
}

fn registry_fixture() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = fixture()?;
    write_fixture(root.path(), "catalog/field-ids.txt", "Shared\n")?;
    Ok(root)
}

fn fields(root: &Path, mode: &str) -> std::io::Result<std::process::Output> {
    std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["models", "fields", "--root"])
        .arg(root)
        .arg("--source")
        .arg(root)
        .arg(mode)
        .output()
}

fn set_old_modified(root: &Path) -> std::io::Result<SystemTime> {
    let file = std::fs::File::open(registry_path(root))?;
    let old = std::time::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(1))
        .ok_or_else(|| std::io::Error::other("old modification time overflow"))?;
    file.set_times(std::fs::FileTimes::new().set_modified(old))?;
    file.metadata()?.modified()
}

fn assert_unchanged(root: &Path, before: SystemTime) -> std::io::Result<()> {
    assert_eq!(std::fs::read(registry_path(root))?, b"Shared\n");
    assert_eq!(std::fs::metadata(registry_path(root))?.modified()?, before);
    Ok(())
}

#[test]
fn write_replaces_registry_without_mutating_its_old_hard_link()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a registry with a second hard link to the same original inode.
    let root = registry_fixture()?;
    let previous = root.path().join("previous-registry.txt");
    std::fs::hard_link(registry_path(root.path()), &previous)?;
    write_fixture(root.path(), "models/added.json", ADDED)?;
    // When publishing an appended member, then replacement preserves the old inode's bytes.
    success(&fields(root.path(), "--write")?, "field-ids 2 (+1)\n");
    assert_eq!(
        std::fs::read(registry_path(root.path()))?,
        b"Shared\nAardvark\n"
    );
    assert_eq!(std::fs::read(previous)?, b"Shared\n");
    Ok(())
}

#[test]
fn successful_check_preserves_registry_bytes_and_modification_time()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a current registry with an old modification time.
    let root = registry_fixture()?;
    let before = set_old_modified(root.path())?;
    // When checking successfully, then even an identical rewrite is forbidden.
    success(&fields(root.path(), "--check")?, "field-ids 1 (+0)\n");
    assert_unchanged(root.path(), before)?;
    Ok(())
}

#[test]
fn stale_check_preserves_registry_bytes_and_modification_time()
-> Result<(), Box<dyn std::error::Error>> {
    // Given an old registry and a newly modeled member.
    let root = registry_fixture()?;
    write_fixture(root.path(), "models/added.json", ADDED)?;
    let before = set_old_modified(root.path())?;
    // When checking fails, then the exact diagnostic is emitted without touching the file.
    let output = fields(root.path(), "--check")?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr)?,
        concat!(
            "1 member names are not in the registry, first: Aardvark; ",
            "run `catalog-tool models fields --write`\n"
        )
    );
    assert_unchanged(root.path(), before)?;
    Ok(())
}

#[test]
fn library_check_returns_typed_stale_path_count_and_first_name()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a registry missing one independently named member.
    let root = registry_fixture()?;
    write_fixture(root.path(), "models/added.json", ADDED)?;
    // When the public library checks it, then callers receive all typed stale details.
    let result = smithy::fields(root.path(), root.path(), ReportMode::Check);
    let Err(Error::RegistryStale {
        path,
        new_names,
        first,
    }) = result
    else {
        return Err(format!("expected RegistryStale, got {result:?}").into());
    };
    assert_eq!(path, root.path().join("catalog/field-ids.txt"));
    assert_eq!(new_names, 1);
    assert_eq!(first, "Aardvark");
    Ok(())
}

fn assert_write_in_progress(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = fields(root, "--write")?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!(
            "{}: another run is writing or a previous run crashed; remove the temporary file\n",
            root.join("catalog/field-ids.txt.tmp").display()
        )
    );
    assert_eq!(std::fs::read(registry_path(root))?, b"Shared\n");
    Ok(())
}

#[test]
fn existing_temporary_file_rejects_write_without_touching_registry_or_lock()
-> Result<(), Box<dyn std::error::Error>> {
    // Given another run's temporary file, including when the candidate is unchanged.
    let root = registry_fixture()?;
    write_fixture(root.path(), "catalog/field-ids.txt.tmp", "other writer\n")?;
    // When writing, then the lock is respected rather than reused or removed.
    assert_write_in_progress(root.path())?;
    assert_eq!(
        std::fs::read(root.path().join("catalog/field-ids.txt.tmp"))?,
        b"other writer\n"
    );
    Ok(())
}

#[test]
fn temporary_symlink_rejects_write_without_truncating_live_registry()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a temporary-path symlink to the live registry and a changed candidate.
    let root = registry_fixture()?;
    write_fixture(root.path(), "models/added.json", ADDED)?;
    let temporary = root.path().join("catalog/field-ids.txt.tmp");
    std::os::unix::fs::symlink(registry_path(root.path()), &temporary)?;
    // When writing, then exclusive creation fails without following or removing the symlink.
    assert_write_in_progress(root.path())?;
    assert_eq!(std::fs::read_link(temporary)?, registry_path(root.path()));
    Ok(())
}

#[test]
fn oversized_registry_is_rejected_before_parsing_and_creates_no_temporary_file()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a registry one byte over the limit, whose content is not a valid name list.
    let root = registry_fixture()?;
    std::fs::OpenOptions::new()
        .write(true)
        .open(registry_path(root.path()))?
        .set_len(16_777_217)?;
    // When preparing a write, then the typed size error takes precedence over parsing.
    let result = smithy::fields(root.path(), root.path(), ReportMode::Write);
    let Err(Error::RegistryTooLarge { path, limit }) = result else {
        return Err(format!("expected RegistryTooLarge, got {result:?}").into());
    };
    assert_eq!(path, root.path().join("catalog/field-ids.txt"));
    assert_eq!(limit, 16_777_216);
    assert_eq!(std::fs::metadata(path)?.len(), 16_777_217);
    assert!(!root.path().join("catalog/field-ids.txt.tmp").exists());
    Ok(())
}

fn assert_registry_open_failure(
    root: &Path,
    flags: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["models", "fields", "--root"])
        .arg(root)
        .arg("--source")
        .arg(root)
        .args(flags)
        .output()?;
    let path = registry_path(root);
    let expected = format!("{}: Not a directory (os error 20)\n", path.display());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(String::from_utf8(output.stderr)?, expected);
    Ok(())
}

#[test]
fn non_directory_catalog_fails_print_and_write_without_bootstrapping()
-> Result<(), Box<dyn std::error::Error>> {
    // Given valid models but a regular file where the catalog directory belongs.
    let root = fixture()?;
    write_fixture(root.path(), "catalog", "not a directory\n")?;
    // When printing or writing, then the named open error prevents bootstrap and candidate output.
    [&[][..], &["--write"][..]]
        .into_iter()
        .try_for_each(|flags| assert_registry_open_failure(root.path(), flags))?;
    assert_eq!(
        std::fs::read(root.path().join("catalog"))?,
        b"not a directory\n"
    );
    Ok(())
}
