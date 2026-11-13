//! Source selection rules of the retired the retired Python packager.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories excluded at any depth.
const EXCLUDED_DIRECTORIES: [&str; 5] = [
    "target",
    ".git",
    "__pycache__",
    ".pytest_cache",
    "mutants.out.old",
];
/// Artifact trees that are development intermediates, never evidence.
const EXCLUDED_ARTIFACT_TREES: [&str; 4] = [
    "emulator-seed-preflight",
    "emulator-v04-first",
    "emulator-v04-second",
    "emulator-v04-expanded",
];
/// The only database files admitted into the payload.
const INCLUDED_DATABASES: [&str; 6] = [
    "artifacts/emulator-v04/inventory.sqlite",
    "artifacts/emulator-v05/inventory.sqlite",
    "artifacts/emulator-v06/inventory.sqlite",
    "artifacts/emulator-v06-verified/inventory.sqlite",
    "artifacts/emulator-v06-final/inventory.sqlite",
    "artifacts/emulator-v06-complete/inventory.sqlite",
];
/// Credential material is never distributable.
const CREDENTIAL_NAMES: [&str; 5] = [
    "credentials",
    "emulator_credentials",
    "ca.pem",
    "server.key",
    "leaf.key",
];
/// Manifest file name written beside the sources.
pub(super) const MANIFEST_NAME: &str = "MANIFEST.sha256";

/// Recursively collect included files without following symlinked entries.
pub(super) fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            walk(&path, root, out)?;
        } else if file_type.is_file() {
            let relative = path.strip_prefix(root).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "discovered path is outside the payload root",
                )
            })?;
            if included(relative) {
                out.push(relative.to_path_buf());
            }
        }
    }
    Ok(())
}

/// Selection rules of the original `included()`, applied to a relative path.
fn included(relative: &Path) -> bool {
    let parts: Vec<&OsStr> = relative.iter().collect();
    if parts.iter().any(|part| {
        EXCLUDED_DIRECTORIES
            .iter()
            .any(|name| part == &OsStr::new(name))
    }) {
        return false;
    }
    let Some(name) = relative.file_name() else {
        return false;
    };
    !excluded_name(name) && !excluded_artifact_tree(&parts) && !excluded_database(relative, name)
}

/// Names never distributed: manifest, compiled artifacts and credentials.
fn excluded_name(name: &OsStr) -> bool {
    if name == OsStr::new(MANIFEST_NAME)
        || CREDENTIAL_NAMES
            .iter()
            .any(|credential| name == OsStr::new(credential))
    {
        return true;
    }
    Path::new(name)
        .extension()
        .is_some_and(|extension| matches!(extension.to_str(), Some("pyc" | "deb" | "so")))
}

/// Development artifact trees are never evidence.
fn excluded_artifact_tree(parts: &[&OsStr]) -> bool {
    if !parts
        .first()
        .is_some_and(|part| *part == OsStr::new("artifacts"))
    {
        return false;
    }
    let Some(second) = parts.get(1) else {
        return false;
    };
    EXCLUDED_ARTIFACT_TREES
        .iter()
        .any(|tree| second == &OsStr::new(tree))
        || second.to_string_lossy().starts_with("emulator-oracle-")
}

/// Databases are excluded unless the path is one of the published fixtures.
fn excluded_database(relative: &Path, name: &OsStr) -> bool {
    if !has_database_suffix(&name.to_string_lossy()) {
        return false;
    }
    !INCLUDED_DATABASES.contains(&slash_path(relative).as_str())
}

/// Whether a file name carries one of the database suffixes.
fn has_database_suffix(name: &str) -> bool {
    const SUFFIXES: [&str; 6] = [
        ".sqlite",
        ".sqlite-wal",
        ".sqlite-shm",
        ".db",
        ".db-wal",
        ".db-shm",
    ];
    SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
}

/// Slash-separated relative name, matching Python's `Path.as_posix()`.
pub(super) fn slash_path(path: &Path) -> String {
    let mut out = String::new();
    for component in path.components() {
        if let std::path::Component::Normal(part) = component {
            if !out.is_empty() {
                out.push('/');
            }
            out.push_str(&part.to_string_lossy());
        }
    }
    out
}
