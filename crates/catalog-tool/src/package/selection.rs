use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::limits::{Limits, exceeded};

const EXCLUDED_DIRECTORIES: [&str; 6] = [
    "target",
    ".git",
    "__pycache__",
    ".pytest_cache",
    "mutants.out.old",
    ".local",
];

const EXCLUDED_ARTIFACT_TREES: [&str; 4] = [
    "emulator-seed-preflight",
    "emulator-v04-first",
    "emulator-v04-second",
    "emulator-v04-expanded",
];

const INCLUDED_DATABASES: [&str; 6] = [
    "artifacts/emulator-v04/inventory.sqlite",
    "artifacts/emulator-v05/inventory.sqlite",
    "artifacts/emulator-v06/inventory.sqlite",
    "artifacts/emulator-v06-verified/inventory.sqlite",
    "artifacts/emulator-v06-final/inventory.sqlite",
    "artifacts/emulator-v06-complete/inventory.sqlite",
];

const CREDENTIAL_NAMES: [&str; 5] = [
    "credentials",
    "emulator_credentials",
    "ca.pem",
    "server.key",
    "leaf.key",
];

pub(super) const MANIFEST_NAME: &str = "MANIFEST.sha256";

pub(super) fn walk(root: &Path, limits: Limits) -> io::Result<Vec<PathBuf>> {
    let mut scanner = Scanner {
        root,
        limits,
        paths: Vec::new(),
        entries: 0,
    };
    let mut directories = vec![root.to_path_buf()];
    (0..=limits.depth).try_for_each(|depth| {
        directories = scanner.read_level(&directories, depth)?;
        Ok::<_, io::Error>(())
    })?;
    Ok(scanner.paths)
}

struct Scanner<'a> {
    root: &'a Path,
    limits: Limits,
    paths: Vec<PathBuf>,
    entries: usize,
}

impl Scanner<'_> {
    fn read_level(&mut self, directories: &[PathBuf], depth: usize) -> io::Result<Vec<PathBuf>> {
        directories
            .iter()
            .try_fold(Vec::new(), |mut next, directory| {
                fs::read_dir(directory)?
                    .try_for_each(|entry| self.visit(entry?, depth, &mut next))?;
                Ok(next)
            })
    }

    fn visit(
        &mut self,
        entry: fs::DirEntry,
        depth: usize,
        next: &mut Vec<PathBuf>,
    ) -> io::Result<()> {
        let path = entry.path();
        self.count_entry(&path)?;
        let relative = path
            .strip_prefix(self.root)
            .map_err(|_| io::Error::other("discovered path is outside the payload root"))?;
        let kind = entry.file_type()?;
        if kind.is_dir() && !excluded_directory(relative) {
            return self.queue_directory(path, depth, next);
        }
        if kind.is_file() && included(relative) {
            store_path(&mut self.paths, relative.to_path_buf())?;
        }
        Ok(())
    }

    fn count_entry(&mut self, path: &Path) -> io::Result<()> {
        self.entries = self
            .entries
            .checked_add(1)
            .ok_or_else(|| exceeded(path, "visited entries", self.limits.entries))?;
        if self.entries > self.limits.entries {
            return Err(exceeded(path, "visited entries", self.limits.entries));
        }
        Ok(())
    }

    fn queue_directory(
        &self,
        path: PathBuf,
        depth: usize,
        next: &mut Vec<PathBuf>,
    ) -> io::Result<()> {
        if depth >= self.limits.depth {
            return Err(exceeded(&path, "directory depth", self.limits.depth));
        }
        store_path(next, path)
    }
}

fn store_path(paths: &mut Vec<PathBuf>, path: PathBuf) -> io::Result<()> {
    paths.try_reserve(1).map_err(io::Error::other)?;
    paths.push(path);
    Ok(())
}

fn included(relative: &Path) -> bool {
    if excluded_directory(relative) {
        return false;
    }
    let Some(name) = relative.file_name() else {
        return false;
    };
    !excluded_name(name) && !excluded_database(relative, name)
}

fn excluded_directory(relative: &Path) -> bool {
    relative.iter().any(|part| {
        EXCLUDED_DIRECTORIES
            .iter()
            .any(|name| part == OsStr::new(name))
    }) || excluded_artifact_tree(&relative.iter().collect::<Vec<_>>())
}

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

fn excluded_database(relative: &Path, name: &OsStr) -> bool {
    if !has_database_suffix(&name.to_string_lossy()) {
        return false;
    }
    !INCLUDED_DATABASES.contains(&slash_path(relative).as_str())
}

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
