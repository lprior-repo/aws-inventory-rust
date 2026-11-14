use super::*;

fn rejected(output: &Output, root: &Path, message: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains(message));
    assert!(!root.join("catalog/aws-api.catalog").exists());
}

#[test]
fn nested_plain_model_directory_cannot_claim_its_parent_repository_commit() -> TestResult {
    // Given models under a plain directory inside a checkout, compiling rejects the wrong root.
    let parent = source()?;
    let nested = parent.path().join("plain");
    fs::create_dir_all(nested.join("models/example"))?;
    fs::write(nested.join("models/example/model.json"), MODEL)?;
    let root = root()?;
    let output = compile(&nested, root.path(), &["--write"])?;
    rejected(&output, root.path(), "not its repository root");
    Ok(())
}

#[test]
fn modified_tracked_model_reports_the_first_uncommitted_path() -> TestResult {
    // Given modified tracked input, compiling must not label its bytes with unchanged HEAD.
    let source = source()?;
    fs::write(
        source.path().join("models/example/model.json"),
        MODEL.replace("ListItems", "GetItems"),
    )?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    rejected(
        &output,
        root.path(),
        "model source has uncommitted changes: models/example/model.json",
    );
    Ok(())
}

#[test]
fn untracked_model_reports_the_first_uncommitted_path() -> TestResult {
    // Given a clean committed model plus a new untracked model, compiling rejects the extra input.
    let source = source()?;
    fs::write(source.path().join("models/example/extra.json"), MODEL)?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    rejected(
        &output,
        root.path(),
        "model source has uncommitted changes: models/example/extra.json",
    );
    Ok(())
}

#[test]
fn clean_repository_is_accepted_and_its_own_commit_is_embedded() -> TestResult {
    // Given an independently committed source, the catalog records that repository's real HEAD.
    let source = source()?;
    let root = root()?;
    let head = git::command(source.path(), &["rev-parse", "HEAD"])?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    let bytes = fs::read(root.path().join("catalog/aws-api.catalog"))?;
    success(&output, bytes.len());
    assert_eq!(Catalog::borrow(&bytes)?.models_commit()?, head.trim());
    Ok(())
}

fn ignored_source() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let source = source()?;
    fs::write(source.path().join(".git/info/exclude"), "/models/extra/\n")?;
    fs::create_dir_all(source.path().join("models/extra"))?;
    fs::write(
        source.path().join("models/extra/model.json"),
        r#"{"shapes":{"extra#Service":{"type":"service"}}}"#,
    )?;
    Ok(source)
}

#[test]
fn ignored_untracked_model_is_rejected_even_when_git_status_is_clean() -> TestResult {
    // Given an ignored extra model invisible to ordinary status, compilation still rejects it.
    let source = ignored_source()?;
    let status = git::command(
        source.path(),
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--",
            "models",
        ],
    )?;
    assert!(status.is_empty());
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    rejected(
        &output,
        root.path(),
        "model source has uncommitted changes: models/extra/model.json",
    );
    Ok(())
}
