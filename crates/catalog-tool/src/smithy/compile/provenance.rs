use super::CompileError;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn commit(source: &Path) -> Result<String, CompileError> {
    repository_root(source)?;
    clean_models(source)?;
    clean_untracked_models(source)?;
    let text = git(source, &["rev-parse", "HEAD"])?;
    let commit = text.trim();
    if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CompileError::Commit(
            "expected a 40-character Git commit".into(),
        ));
    }
    Ok(commit.to_owned())
}

fn repository_root(source: &Path) -> Result<(), CompileError> {
    let path = source.canonicalize().map_err(commit_error)?;
    let root = git(source, &["rev-parse", "--show-toplevel"])?;
    let repository = Path::new(root.trim_end_matches('\n'))
        .canonicalize()
        .map_err(commit_error)?;
    if path != repository {
        return Err(CompileError::SourceNotRepositoryRoot { path, repository });
    }
    Ok(())
}

fn clean_models(source: &Path) -> Result<(), CompileError> {
    let status = git(
        source,
        &[
            "status",
            "--porcelain",
            "-z",
            "--untracked-files=all",
            "--",
            "models",
        ],
    )?;
    let Some(entry) = status.split('\0').find(|entry| !entry.is_empty()) else {
        return Ok(());
    };
    let path = entry
        .get(3..)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| CompileError::Commit("invalid Git status entry".into()))?;
    Err(CompileError::SourceModified(PathBuf::from(path)))
}

fn clean_untracked_models(source: &Path) -> Result<(), CompileError> {
    let files = git(source, &["ls-files", "--others", "-z", "--", "models"])?;
    match files.split('\0').find(|path| !path.is_empty()) {
        Some(path) => Err(CompileError::SourceModified(PathBuf::from(path))),
        None => Ok(()),
    }
}

fn git(source: &Path, args: &[&str]) -> Result<String, CompileError> {
    let output = Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(source)
        .args(args)
        .output()
        .map_err(commit_error)?;
    if !output.status.success() {
        return Err(CompileError::Commit(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    String::from_utf8(output.stdout).map_err(commit_error)
}

fn commit_error(error: impl std::fmt::Display) -> CompileError {
    CompileError::Commit(error.to_string())
}
