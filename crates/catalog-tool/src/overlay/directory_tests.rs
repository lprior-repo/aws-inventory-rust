use super::collect_paths;
use crate::overlay::{OverlayError, Rule};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn directory_entry_budget_counts_ignored_entries_at_bound_and_one_past() -> TestResult {
    // Given ignored files at the injected bound, when listed, then only excess fails.
    let dir = tempfile::tempdir()?;
    (0..3).try_for_each(|i| std::fs::write(dir.path().join(format!("{i}.txt")), ""))?;
    assert_eq!(collect_paths(dir.path(), 2, 3)?.len(), 0);
    std::fs::write(dir.path().join("extra.txt"), "")?;
    failure(collect_paths(dir.path(), 2, 3), "entries", "4", 3);
    Ok(())
}
#[test]
fn directory_file_budget_counts_only_toml_at_bound_and_one_past() -> TestResult {
    // Given ignored entries plus TOML files, when listed, then only TOML consumes file budget.
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join("ignored.txt"), "")?;
    std::fs::write(dir.path().join("b.toml"), "")?;
    std::fs::write(dir.path().join("a.toml"), "")?;
    assert_eq!(
        collect_paths(dir.path(), 2, 4)?,
        vec![dir.path().join("a.toml"), dir.path().join("b.toml")]
    );
    std::fs::write(dir.path().join("extra.toml"), "")?;
    failure(collect_paths(dir.path(), 2, 4), "files", "3", 2);
    Ok(())
}
fn failure(
    result: Result<Vec<std::path::PathBuf>, OverlayError>,
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
            String::new(),
            None,
            key.into(),
            value.into(),
            Rule::Limit { max }
        ))
    );
}
