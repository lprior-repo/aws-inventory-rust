use super::super::prepare_registry;
use super::*;

#[test]
fn changed_snapshot_rejects_publication_and_preserves_the_other_writer()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["Beta".to_owned()].into(), 64)?;
    fs::write(&path, "old\nAlpha\n")?;

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
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::create_dir(&path)?;
    fs::write(path.join("keep"), "preserve\n")?;
    let temporary = root.path().join("field-ids.txt.tmp");
    let mut file = create_temporary(&temporary)?;

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
    let root = tempfile::tempdir()?;
    let temporary = root.path().join("field-ids.txt.tmp");
    fs::create_dir(&temporary)?;
    fs::write(temporary.join("keep"), "preserve\n")?;
    let original = Error::RegistryChanged {
        path: root.path().join("field-ids.txt"),
    };

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
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["Beta".to_owned()].into(), 16)?;
    fs::OpenOptions::new()
        .write(true)
        .open(&path)?
        .set_len(17)?;

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
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["Beta".to_owned()].into(), 64)?;
    fs::write(&path, "new\n")?;

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
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    let prepared = prepare_registry(&path, ["abc".to_owned()].into(), 4)?;

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
    let root = tempfile::tempdir()?;
    let path = root.path().join("catalog/field-ids.txt");
    let prepared = prepare_registry(&path, ["abcd".to_owned()].into(), 4)?;

    let result = publish(&path, &prepared.original, &prepared.next.render(), 4);
    assert_registry_too_large(result, &path, 4)?;
    assert!(!root.path().join("catalog").exists());
    Ok(())
}

#[test]
fn append_at_byte_limit_publishes_and_survives_both_reads() -> Result<(), Box<dyn std::error::Error>>
{
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["new".to_owned()].into(), 8)?;

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
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "old\n")?;
    let prepared = prepare_registry(&path, ["new".to_owned()].into(), 7)?;

    let result = publish(&path, &prepared.original, &prepared.next.render(), 7);
    assert_registry_too_large(result, &path, 7)?;
    assert_eq!(fs::read(&path)?, b"old\n");
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}

#[test]
fn preparation_rejects_registry_one_byte_over_limit() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("field-ids.txt");
    fs::write(&path, "abcd\n")?;

    let result = prepare_registry(&path, [].into(), 4).map(|_| ());
    assert_registry_too_large(result, &path, 4)?;
    assert_eq!(fs::read(&path)?, b"abcd\n");
    assert!(!root.path().join("field-ids.txt.tmp").exists());
    Ok(())
}
