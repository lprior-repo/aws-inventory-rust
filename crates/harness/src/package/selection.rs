//! Source selection and ordering rules of the retired Python packager.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::HarnessError;

/// Directories excluded at any depth.
const EXCLUDED_DIRECTORIES: [&str; 6] = [
    "target",
    ".git",
    "__pycache__",
    ".pytest_cache",
    "mutants.out.old",
    ".local",
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

/// Recursively collect candidate files without following symlinked entries.
pub(super) fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) -> Result<(), HarnessError> {
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
            let relative = path
                .strip_prefix(root)
                .map_err(|_| HarnessError::invalid("discovered path is outside the root"))?;
            if included(relative) {
                out.push(relative.to_path_buf());
            }
        }
    }
    Ok(())
}

/// Selection rules of `included()`, applied to a root-relative path.
fn included(relative: &Path) -> bool {
    let parts: Vec<&OsStr> = relative.iter().collect();
    let Some(name) = relative.file_name() else {
        return false;
    };
    if is_excluded_directory(&parts) || name == OsStr::new(MANIFEST_NAME) {
        return false;
    }
    if is_excluded_extension(relative) || is_excluded_artifact(&parts) {
        return false;
    }
    if is_excluded_database(name, relative) {
        return false;
    }
    !CREDENTIAL_NAMES
        .iter()
        .any(|credential| name == OsStr::new(credential))
}

/// Whether any path component is a directory excluded at any depth.
fn is_excluded_directory(parts: &[&OsStr]) -> bool {
    parts.iter().any(|part| {
        EXCLUDED_DIRECTORIES
            .iter()
            .any(|name| part == &OsStr::new(name))
    })
}

/// Whether the file carries a never-distributable compiled extension.
fn is_excluded_extension(relative: &Path) -> bool {
    let Some(extension) = relative.extension() else {
        return false;
    };
    extension == OsStr::new("pyc")
        || extension == OsStr::new("deb")
        || extension == OsStr::new("so")
}

/// Whether the file sits inside an artifact tree excluded as intermediate.
fn is_excluded_artifact(parts: &[&OsStr]) -> bool {
    let Some(first) = parts.first() else {
        return false;
    };
    if *first != OsStr::new("artifacts") {
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

/// Whether a database file is outside the reviewed inclusion list.
fn is_excluded_database(name: &OsStr, relative: &Path) -> bool {
    if !ends_with_any(
        &name.to_string_lossy(),
        &[
            ".sqlite",
            ".sqlite-wal",
            ".sqlite-shm",
            ".db",
            ".db-wal",
            ".db-shm",
        ],
    ) {
        return false;
    }
    !INCLUDED_DATABASES.contains(&slash_path(relative).as_str())
}

/// Whether a file name ends with any of the database suffixes.
fn ends_with_any(name: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| name.ends_with(suffix))
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
