use std::fs;

use super::{Error, find_model_files_within, read_bounded};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn a_file_of_exactly_the_limit_is_read_whole() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("model.json");
    fs::write(&path, b"12345678")?;

    assert_eq!(read_bounded(&path, 8)?, b"12345678");
    Ok(())
}

#[test]
fn one_byte_past_the_limit_is_rejected_by_the_read_itself() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("model.json");
    fs::write(&path, b"123456789")?;

    assert!(matches!(read_bounded(&path, 8), Err(Error::FileTooLarge(file)) if file == path));
    Ok(())
}

#[test]
fn discovery_visits_exactly_its_entry_budget_and_sorts_the_models() -> TestResult {
    let dir = tempfile::tempdir()?;
    fs::create_dir(dir.path().join("nested"))?;
    fs::write(dir.path().join("b.json"), "{}")?;
    fs::write(dir.path().join("a.json"), "{}")?;
    fs::write(dir.path().join("nested/c.json"), "{}")?;
    fs::write(dir.path().join("notes.txt"), "")?;

    let files = find_model_files_within(dir.path(), 5)?;
    assert_eq!(
        files,
        vec![
            dir.path().join("a.json"),
            dir.path().join("b.json"),
            dir.path().join("nested/c.json"),
        ]
    );

    assert!(matches!(
        find_model_files_within(dir.path(), 4),
        Err(Error::TooManyEntries { limit: 4, .. })
    ));
    Ok(())
}

#[test]
fn empty_directories_consume_the_budget_too() -> TestResult {
    let dir = tempfile::tempdir()?;
    ["one", "two", "three"]
        .iter()
        .try_for_each(|name| fs::create_dir(dir.path().join(name)))?;

    assert!(matches!(
        find_model_files_within(dir.path(), 2),
        Err(Error::TooManyEntries { directory, limit: 2 }) if directory == dir.path()
    ));

    assert!(find_model_files_within(dir.path(), 3)?.is_empty());
    Ok(())
}
