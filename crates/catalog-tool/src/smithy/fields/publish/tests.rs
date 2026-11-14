use super::super::prepare_registry;
use super::*;

#[test]
fn changed_snapshot_rejects_publication_and_preserves_the_other_writer()
-> Result<(), Box<dyn std::error::Error>> {
    // Given writer B's prepared successor and writer A's intervening publication.
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["Beta".to_owned()].into(), 64)?;
    fs::write(&path, "old\nAlpha\n")?;
    // When B acquires the lock, then it rejects its stale snapshot without reusing A's ID.
    let result = publish(&path, &prepared.original, &prepared.next.render(), 64);
    let Err(Error::RegistryChanged { path: changed }) = result else {
        return Err(format!("expected RegistryChanged, got {result:?}").into());
    };
    assert_eq!(changed, path);
    assert_eq!(fs::read(&path)?, b"old\nAlpha\n");
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}

#[test]
fn rename_failure_removes_owned_temporary_file_and_preserves_original_error()
-> Result<(), Box<dyn std::error::Error>> {
    // Given an exclusively owned temporary file and a nonempty target directory.
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::create_dir(&path)?;
    fs::write(path.join("keep"), "preserve\n")?;
    let temporary = root.path().join("field-ids.txt.tmp");
    let mut file = create_temporary(&temporary)?;
    // When rename fails, then publication cleanup removes only its temporary file.
    let result = write_and_rename(&path, &temporary, &mut file, "new\n");
    drop(file);
    let result = finish_publication(&temporary, result);
    let Err(Error::Io {
        file: failed,
        source,
    }) = result
    else {
        return Err(format!("expected rename I/O error, got {result:?}").into());
    };
    assert_eq!(failed, path);
    assert_eq!(source.kind(), io::ErrorKind::IsADirectory);
    assert!(!temporary.exists());
    assert_eq!(fs::read(path.join("keep"))?, b"preserve\n");
    Ok(())
}

#[test]
fn cleanup_failure_reports_both_the_original_and_cleanup_errors()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a cleanup path that cannot be removed as a file.
    let root = tempfile::tempdir()?;
    let temporary = root.path().join("field-ids.txt.tmp");
    fs::create_dir(&temporary)?;
    fs::write(temporary.join("keep"), "preserve\n")?;
    let original = Error::RegistryChanged {
        path: root.path().join("field-ids.txt"),
    };
    // When cleanup also fails, then the diagnostic retains both failures and the temp path.
    let error = cleanup_error(&temporary, original);
    assert_eq!(
        error.to_string(),
        format!(
            "registry changed during the run; rerun; failed to remove temporary {}: \
         Is a directory (os error 21)",
            temporary.display()
        )
    );
    assert_eq!(fs::read(temporary.join("keep"))?, b"preserve\n");
    Ok(())
}

#[test]
fn locked_comparison_rejects_oversized_registry_and_removes_owned_temporary_file()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a valid prepared snapshot that becomes oversized before the lock is acquired.
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["Beta".to_owned()].into(), 16)?;
    fs::OpenOptions::new()
        .write(true)
        .open(&path)?
        .set_len(17)?;
    // When the locked comparison reads it, then the same byte limit applies before parsing.
    let result = publish(&path, &prepared.original, &prepared.next.render(), 16);
    assert_registry_too_large(result, &path, 16)?;
    assert_eq!(fs::metadata(&path)?.len(), 17);
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}

fn assert_registry_too_large(
    result: Result<(), Error>,
    expected: &Path,
    expected_limit: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let Err(Error::RegistryTooLarge { path, limit }) = result else {
        return Err(format!("expected RegistryTooLarge, got {result:?}").into());
    };
    assert_eq!(path, expected);
    assert_eq!(limit, expected_limit);
    Ok(())
}

#[test]
fn same_length_snapshot_change_rejects_publication() -> Result<(), Box<dyn std::error::Error>> {
    // Given history replaced by different bytes of the same length after preparation.
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["Beta".to_owned()].into(), 64)?;
    fs::write(&path, "new\n")?;
    // When publishing, then byte comparison rejects it and removes the owned lock.
    let result = publish(&path, &prepared.original, &prepared.next.render(), 64);
    let Err(Error::RegistryChanged { path: changed }) = result else {
        return Err(format!("expected RegistryChanged, got {result:?}").into());
    };
    assert_eq!(changed, path);
    assert_eq!(fs::read(&path)?, b"new\n");
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}

#[test]
fn bootstrap_at_byte_limit_publishes_and_survives_both_reads()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a bootstrap candidate whose rendered bytes exactly equal the four-byte limit.
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    let prepared = prepare_registry(&path, ["abc".to_owned()].into(), 4)?;
    // When published and prepared again, then both initial and locked reads accept the boundary.
    publish(&path, &prepared.original, &prepared.next.render(), 4)?;
    assert_eq!(fs::read(&path)?, b"abc\n");
    let readback = prepare_registry(&path, [].into(), 4)?;
    assert_eq!(readback.next.render(), "abc\n");
    publish(&path, &readback.original, &readback.next.render(), 4)?;
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}

#[test]
fn bootstrap_one_byte_over_limit_creates_no_output_directory()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a five-byte bootstrap candidate with a four-byte limit.
    let root = tempfile::tempdir()?;
    let path = root.path().join("catalog/field-ids.txt");
    let prepared = prepare_registry(&path, ["abcd".to_owned()].into(), 4)?;
    // When publishing, then size rejection precedes directory and temporary-file creation.
    let result = publish(&path, &prepared.original, &prepared.next.render(), 4);
    assert_registry_too_large(result, &path, 4)?;
    assert!(!root.path().join("catalog").exists());
    Ok(())
}

#[test]
fn append_at_byte_limit_publishes_and_survives_both_reads() -> Result<(), Box<dyn std::error::Error>>
{
    // Given four bytes of history and a four-byte append, with an eight-byte limit.
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["new".to_owned()].into(), 8)?;
    // When appended and read back, then the exact boundary remains usable.
    publish(&path, &prepared.original, &prepared.next.render(), 8)?;
    assert_eq!(fs::read(&path)?, b"old\nnew\n");
    let readback = prepare_registry(&path, [].into(), 8)?;
    assert_eq!(readback.next.render(), "old\nnew\n");
    publish(&path, &readback.original, &readback.next.render(), 8)?;
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}

#[test]
fn append_one_byte_over_limit_preserves_history_and_creates_no_lock()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a valid four-byte registry and a candidate exceeding the seven-byte limit by one.
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["new".to_owned()].into(), 7)?;
    // When publishing, then the original history and absence of the lock are preserved.
    let result = publish(&path, &prepared.original, &prepared.next.render(), 7);
    assert_registry_too_large(result, &path, 7)?;
    assert_eq!(fs::read(&path)?, b"old\n");
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}

#[test]
fn preparation_rejects_registry_one_byte_over_limit() -> Result<(), Box<dyn std::error::Error>> {
    // Given five bytes on disk and a four-byte read limit.
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "abcd\n")?;
    // When preparing, then the initial read rejects the extra byte without changing history.
    let result = prepare_registry(&path, [].into(), 4).map(|_| ());
    assert_registry_too_large(result, &path, 4)?;
    assert_eq!(fs::read(&path)?, b"abcd\n");
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}
