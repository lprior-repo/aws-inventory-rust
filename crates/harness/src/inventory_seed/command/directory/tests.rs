use super::files;
use crate::inventory_seed::SeedError;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_512_seed_entries_when_listed_then_every_recipe_is_sorted() -> TestResult {
    let directory = tempfile::tempdir()?;
    (0..512).try_for_each(|index| {
        std::fs::write(directory.path().join(format!("seed-{index:03}.toml")), "")
    })?;
    let paths = files(directory.path())?;
    assert_eq!(paths.len(), 512);
    assert!(paths.windows(2).all(|pair| matches!(pair, [a, b] if a < b)));
    Ok(())
}

#[test]
fn given_513_directory_entries_when_listed_then_bound_is_enforced() -> TestResult {
    let directory = tempfile::tempdir()?;
    (0..513).try_for_each(|index| {
        std::fs::write(directory.path().join(format!("entry-{index}.txt")), "")
    })?;
    assert!(matches!(files(directory.path()), Err(SeedError::Invalid("seed directory exceeds 512 entries"))));
    Ok(())
}

#[test]
fn given_no_toml_entries_when_listed_then_empty_directory_is_rejected() -> TestResult {
    let directory = tempfile::tempdir()?;
    std::fs::write(directory.path().join("notes.txt"), "")?;
    assert!(matches!(files(directory.path()), Err(SeedError::Invalid("seed directory has no TOML files"))));
    Ok(())
}
