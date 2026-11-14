use std::{fs, path::Path, process::Command};
type Failure = Box<dyn std::error::Error>;

pub(super) fn repository(model: &str) -> Result<tempfile::TempDir, Failure> {
    let source = tempfile::tempdir()?;
    fs::create_dir_all(source.path().join("models/example"))?;
    fs::write(source.path().join("models/example/model.json"), model)?;
    command(source.path(), &["init", "--quiet", "--object-format=sha1"])?;
    command(source.path(), &["add", "--", "models"])?;
    command(
        source.path(),
        &["commit", "--quiet", "-m", "catalog fixture"],
    )?;
    Ok(source)
}

pub(super) fn command(path: &Path, args: &[&str]) -> Result<String, Failure> {
    let output = builder(path).args(args).output()?;
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}

fn builder(path: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=Catalog Fixture",
            "-c",
            "user.email=catalog@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.autocrlf=false",
        ])
        .arg("-C")
        .arg(path);
    command
}
