use catalog_tool::overlay::{MAX_FILE_BYTES, MAX_FILES, OverlayError, Rule, load_dir};
pub mod overlay_support;
use overlay_support::{EXAMPLE, HEADER, TestResult, write};

#[test]
fn directory_loads_sorted_stems_and_ignores_non_toml_without_touching_them() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "catalog/overlay/z-service.toml", HEADER)?;
    write(root.path(), "catalog/overlay/a-service.toml", EXAMPLE)?;
    write(
        root.path(),
        "catalog/overlay/must-deny.txt",
        "not TOML; keep untouched",
    )?;
    write(root.path(), "catalog/overlay/ignored.TOML", "@")?;
    write(root.path(), "catalog/overlay/ignored.toml.bak", "@")?;
    let overlays = load_dir(root.path())?;
    assert_eq!(
        overlays.iter().map(|o| o.service()).collect::<Vec<_>>(),
        vec!["a-service", "z-service"]
    );
    assert_eq!(
        overlays
            .iter()
            .map(|o| o.operations().len())
            .collect::<Vec<_>>(),
        vec![5, 0]
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("catalog/overlay/must-deny.txt"))?,
        "not TOML; keep untouched"
    );
    Ok(())
}
#[test]
fn empty_directory_returns_empty_list_and_missing_directory_returns_io() -> TestResult {
    let root = tempfile::tempdir()?;
    match load_dir(root.path()) {
        Err(OverlayError::Io { path, source }) => {
            assert_eq!(path, root.path().join("catalog/overlay"));
            assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        }
        _ => return Err("expected missing-directory I/O error".into()),
    }
    std::fs::create_dir_all(root.path().join("catalog/overlay"))?;
    assert_eq!(load_dir(root.path())?, vec![]);
    Ok(())
}
#[test]
fn oversized_file_is_rejected_before_invalid_toml_is_parsed() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "catalog/overlay/ec2.toml", "@")?;
    let path = root.path().join("catalog/overlay/ec2.toml");
    let len = u64::try_from(MAX_FILE_BYTES.saturating_add(1))?;
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)?
        .set_len(len)?;
    limit(
        load_dir(root.path()),
        "ec2",
        "",
        &len.to_string(),
        MAX_FILE_BYTES,
    );
    Ok(())
}
#[test]
fn directory_file_limit_accepts_exact_bound_and_rejects_one_past() -> TestResult {
    let root = tempfile::tempdir()?;
    (0..MAX_FILES)
        .try_for_each(|i| write(root.path(), &format!("catalog/overlay/a{i}.toml"), HEADER))?;
    assert_eq!(load_dir(root.path())?.len(), MAX_FILES);
    write(root.path(), "catalog/overlay/extra.toml", "@")?;
    limit(
        load_dir(root.path()),
        "",
        "files",
        &MAX_FILES.saturating_add(1).to_string(),
        MAX_FILES,
    );
    Ok(())
}
fn limit(
    result: Result<Vec<catalog_tool::overlay::ServiceOverlay>, OverlayError>,
    service: &str,
    key: &str,
    value: &str,
    max: usize,
) {
    let actual = result.err().map(|e| match e {
        OverlayError::Rule {
            service,
            operation,
            key,
            value,
            rule,
        } => (service, operation, key, value.into_string(), rule),
        other => (
            other.to_string(),
            None,
            String::new(),
            String::new(),
            Rule::Resource,
        ),
    });
    assert_eq!(
        actual,
        Some((
            service.into(),
            None,
            key.into(),
            value.into(),
            Rule::Limit { max }
        ))
    );
}
#[test]
fn directory_reports_invalid_stems_and_non_utf8_file_contents() -> TestResult {
    let root = tempfile::tempdir()?;
    write(root.path(), "catalog/overlay/Bad.toml", HEADER)?;
    match load_dir(root.path()) {
        Err(OverlayError::Rule {
            service,
            operation,
            key,
            value,
            rule,
        }) => assert_eq!(
            (service, operation, key, value, rule),
            (
                "Bad".into(),
                None,
                "service".into(),
                "Bad".into(),
                Rule::Service
            )
        ),
        _ => return Err("expected invalid service".into()),
    }
    std::fs::remove_file(root.path().join("catalog/overlay/Bad.toml"))?;
    std::fs::write(root.path().join("catalog/overlay/ec2.toml"), [255])?;
    match load_dir(root.path()) {
        Err(OverlayError::Io { path, source }) => {
            assert_eq!(path, root.path().join("catalog/overlay/ec2.toml"));
            assert_eq!(source.kind(), std::io::ErrorKind::InvalidData);
        }
        _ => return Err("expected invalid UTF-8 I/O error".into()),
    }
    Ok(())
}

#[test]
fn directory_accepts_a_file_at_the_exact_byte_bound() -> TestResult {
    let root = tempfile::tempdir()?;
    let padding = MAX_FILE_BYTES
        .checked_sub(HEADER.len())
        .ok_or("header exceeds cap")?;
    let text = format!("{HEADER}#{}", "x".repeat(padding.saturating_sub(1)));
    write(root.path(), "catalog/overlay/ec2.toml", &text)?;
    let overlays = load_dir(root.path())?;
    assert_eq!(overlays.len(), 1);
    let overlay = overlays.first().ok_or("missing overlay")?;
    assert_eq!((overlay.service(), overlay.operations().len()), ("ec2", 0));
    Ok(())
}
