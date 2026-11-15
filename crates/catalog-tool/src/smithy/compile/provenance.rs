use super::CompileError;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourcePin {
    pub commit: String,
    pub sha256: [u8; 32],
}

pub(super) fn capture(source: &Path) -> Result<SourcePin, CompileError> {
    let commit = commit(source)?;
    let sha256 = models_sha256(source)?;
    if self::commit(source)? != commit {
        return Err(CompileError::Commit("model source changed during capture".into()));
    }
    Ok(SourcePin { commit, sha256 })
}

pub(super) fn verify(source: &Path, expected: &SourcePin) -> Result<(), CompileError> {
    if capture(source)? != *expected {
        return Err(CompileError::Commit("model source changed during compilation".into()));
    }
    Ok(())
}

fn models_sha256(source: &Path) -> Result<[u8; 32], CompileError> {
    let models = source.join("models");
    let files = crate::model_source::discover_models(&models).map_err(commit_error)?;
    let mut hash = Sha256::new();
    hash.update(b"aws-model-source-v1\0");
    for path in files {
        let relative = path.strip_prefix(&models).map_err(commit_error)?;
        let relative = relative.to_str().ok_or_else(|| CompileError::Commit("non-UTF-8 model path".into()))?;
        hash.update(u64::try_from(relative.len()).map_err(commit_error)?.to_le_bytes());
        hash.update(relative.as_bytes());
        hash_file(&path, &mut hash)?;
    }
    Ok(hash.finalize().into())
}

fn hash_file(path: &Path, hash: &mut Sha256) -> Result<(), CompileError> {
    const MAX_FILE_BYTES: u64 = 67_108_864;
    let mut file = fs::File::open(path).map_err(commit_error)?;
    let length = file.metadata().map_err(commit_error)?.len();
    if length > MAX_FILE_BYTES {
        return Err(CompileError::Commit(format!("model exceeds {MAX_FILE_BYTES} bytes: {}", path.display())));
    }
    hash.update(length.to_le_bytes());
    let mut total = 0_u64;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(commit_error)?;
        if read == 0 {
            break;
        }
        total = total.checked_add(u64::try_from(read).map_err(commit_error)?)
            .filter(|total| *total <= length)
            .ok_or_else(|| CompileError::Commit("model source changed during capture".into()))?;
        hash.update(buffer.get(..read).ok_or_else(|| CompileError::Commit("invalid model read length".into()))?);
    }
    if total != length {
        return Err(CompileError::Commit("model source changed during capture".into()));
    }
    Ok(())
}

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
    committed_models(source, commit)?;
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

fn committed_models(source: &Path, commit: &str) -> Result<(), CompileError> {
    let tree = git(source, &["ls-tree", "-r", "-z", "--full-tree", commit, "--", "models"])?;
    let mut expected = committed_model_objects(&tree)?;
    let files = crate::model_source::discover_models(&source.join("models")).map_err(commit_error)?;
    // Read bytes directly, without index stat caches, sparse flags, clean filters, or external diffs.
    for batch in files.chunks(32) {
        let relative = batch.iter().map(|path| path.strip_prefix(source).map_err(commit_error))
            .collect::<Result<Vec<_>, _>>()?;
        let output = Command::new("git")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .arg("-C").arg(source)
            .args(["hash-object", "--no-filters", "--"])
            .args(&relative)
            .output().map_err(commit_error)?;
        if !output.status.success() {
            return Err(CompileError::Commit(String::from_utf8_lossy(&output.stderr).into_owned()));
        }
        let output = String::from_utf8(output.stdout).map_err(commit_error)?;
        let mut objects = output.lines();
        for path in relative {
            let path_text = path.to_str().ok_or_else(|| CompileError::Commit("non-UTF-8 model path".into()))?;
            let object = objects.next().ok_or_else(|| CompileError::Commit("missing model object digest".into()))?;
            if expected.remove(path_text) != Some(object) {
                return Err(CompileError::SourceModified(path.to_path_buf()));
            }
        }
        if objects.next().is_some() {
            return Err(CompileError::Commit("unexpected model object digest".into()));
        }
    }
    if let Some((path, _)) = expected.first_key_value() {
        return Err(CompileError::SourceModified(PathBuf::from(path)));
    }
    Ok(())
}

fn committed_model_objects(tree: &str) -> Result<BTreeMap<&str, &str>, CompileError> {
    let mut objects = BTreeMap::new();
    for entry in tree.split('\0').filter(|entry| !entry.is_empty()) {
        let (header, path) = entry.split_once('\t')
            .ok_or_else(|| CompileError::Commit("invalid committed model tree entry".into()))?;
        if !Path::new(path).extension().is_some_and(|extension| extension == "json") {
            continue;
        }
        let mut header = header.split_whitespace();
        let mode = header.next();
        let kind = header.next();
        let object = header.next().ok_or_else(|| CompileError::Commit("missing committed model object".into()))?;
        if !matches!(mode, Some("100644" | "100755")) || kind != Some("blob") {
            return Err(CompileError::SourceModified(PathBuf::from(path)));
        }
        if header.next().is_some() || objects.insert(path, object).is_some() || objects.len() > 65_536 {
            return Err(CompileError::Commit("invalid committed model object list".into()));
        }
    }
    Ok(objects)
}

fn git(source: &Path, args: &[&str]) -> Result<String, CompileError> {
    let output = Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
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
