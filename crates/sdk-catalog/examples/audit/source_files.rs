//! source files operations.
use super::*;

pub(super) fn source_text(path: &Path, totals: &mut Totals) -> Result<String> {
    let bytes = read_bounded(path, MAX_FILE_BYTES)?;
    add_bytes(
        &mut totals.source_bytes,
        u64::try_from(bytes.len())?,
        MAX_SOURCE_TOTAL_BYTES,
    )?;
    Ok(String::from_utf8(bytes)?)
}

pub(super) fn source_digest_add(digest: &mut Sha256, relative_path: &str, text: &str) {
    // NUL separates paths from fixed-width SHA-256 strings; generated paths
    // cannot contain NUL. Files are processed in deterministic path order.
    digest.update(relative_path.as_bytes());
    digest.update(b"\0");
    digest.update(sha256(text.as_bytes()).as_bytes());
    digest.update(b"\n");
}

pub(super) fn regular_rs_files(directory: &Path) -> Result<Vec<PathBuf>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(directory)?;
    let mut paths = Vec::new();
    for _ in 0..MAX_DIRECTORY_ENTRIES {
        let Some(entry) = entries.next() else { break };
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().is_some_and(|value| value == "rs")
        {
            paths.push(entry.path());
        }
    }
    if entries.next().is_some() {
        return Err(format!("directory entry bound exceeded: {}", directory.display()).into());
    }
    paths.sort();
    Ok(paths)
}

pub(super) fn module_name(path: &Path) -> Result<&str> {
    path.file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "invalid module path".into())
}

pub(super) fn declared_modules(index: &str) -> Result<BTreeSet<String>> {
    let mut modules = BTreeSet::new();
    for module in index.lines().filter_map(|line| {
        line.trim()
            .strip_prefix("pub mod ")
            .and_then(|value| value.strip_suffix(';'))
    }) {
        if !valid_component(module) || modules.len() >= MAX_DIRECTORY_ENTRIES {
            return Err("invalid or excessive module declarations".into());
        }
        if !modules.insert(module.to_owned()) {
            return Err("duplicate module declaration".into());
        }
    }
    Ok(modules)
}

pub(super) fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+'))
}
