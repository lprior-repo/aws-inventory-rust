use super::{Error, GENERATED_CATALOG_FILES, manifest_content, tracked_catalog_files};
use std::{fs, path::Path, process::Command};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const KINDS: &[&str] = &[
    "catalog/kinds.generated.tsv",
    "catalog/aws-kinds-generated.report",
    "catalog/aws-names.report",
];

fn given_generated_artifacts() -> TestResult<tempfile::TempDir> {
    let root = tempfile::tempdir()?;
    let output = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(root.path())
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8(output.stderr)?
    );
    fs::create_dir(root.path().join("catalog"))?;
    for path in GENERATED_CATALOG_FILES {
        fs::write(root.path().join(path), b"fixture\n")?;
    }
    Ok(root)
}

#[test]
fn given_kind_artifacts_when_manifest_is_rendered_then_all_three_are_pinned() -> TestResult {
    let root = given_generated_artifacts()?;
    let files = tracked_catalog_files(root.path())?;
    let manifest = manifest_content(&files)?;
    for path in KINDS {
        assert!(manifest.lines().any(|line| line.ends_with(path)));
    }
    Ok(())
}

#[test]
fn given_missing_kind_artifact_when_manifest_is_built_then_exact_missing_path_is_rejected()
-> TestResult {
    let root = given_generated_artifacts()?;
    for path in KINDS {
        then_missing_is_rejected(root.path(), path)?;
    }
    Ok(())
}

fn then_missing_is_rejected(root: &Path, relative: &str) -> TestResult {
    let path = root.join(relative);
    fs::remove_file(&path)?;
    assert!(
        matches!(tracked_catalog_files(root), Err(Error::MissingGenerated { file }) if file == path)
    );
    fs::write(path, b"fixture\n")?;
    Ok(())
}
