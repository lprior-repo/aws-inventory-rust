//! Bounded AWS source inventory feeding the lexical audit.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::HarnessError;

use super::calls::{CallSite, MAX_CALLS, calls_in_source_with_hint};
use super::files::{MAX_INPUT_BYTES, decode_utf8, read_bytes, slash_path};
use super::mask::mask;
use super::resolve::module_services;

/// Maximum number of AWS source files.
const MAX_FILES: usize = 4_096;
/// Maximum total bytes of AWS source.
const MAX_SOURCE_BYTES: u64 = 32 * 1024 * 1024;

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), HarnessError> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

/// Enumerated AWS sources, their SDK calls and excluded non-empty sends.
pub type Inventory = (BTreeMap<String, String>, Vec<CallSite>, usize);

/// Read the bounded AWS source inventory and enumerate its SDK calls.
pub fn source_inventory(root: &Path) -> Result<Inventory, HarnessError> {
    let paths = rust_paths(root)?;
    let sources = read_sources(root, &paths)?;
    let hints = service_hints(&sources)?;
    let (calls, nonempty) = collect_calls(&sources, &hints)?;
    Ok((sources, calls, nonempty))
}

/// SDK service mentioned by each source, used to infer a parent module's
/// client type for submodules that import it through `use super::*`.
fn service_hints(
    sources: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Option<String>>, HarnessError> {
    let mut direct: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (path, source) in sources {
        let characters: Vec<char> = source.chars().collect();
        let masked =
            mask(&characters).map_err(|error| crate::HarnessError::invalid(error.message))?;
        let services = module_services(&masked);
        if !services.is_empty() {
            direct.insert(path.clone(), services);
        }
    }
    Ok(sources
        .keys()
        .map(|path| (path.clone(), hint_for(path, &direct)))
        .collect())
}

fn hint_for(path: &str, direct: &BTreeMap<String, BTreeSet<String>>) -> Option<String> {
    let mut current = Path::new(path);
    while let Some(parent) = current.parent() {
        let Some(name) = parent.file_name() else {
            break;
        };
        if name == "src" {
            break;
        }
        let candidate = parent.with_file_name(format!("{}.rs", name.to_string_lossy()));
        if let Some(services) = direct.get(&slash_path(&candidate))
            && let Some(service) = services.iter().next()
            && services.len() == 1
        {
            return Some(service.clone());
        }
        current = parent;
    }
    None
}

fn rust_paths(root: &Path) -> Result<Vec<PathBuf>, HarnessError> {
    let mut paths = Vec::new();
    collect_rust_files(&root.join("crates/aws/src"), &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn relative_source<'a>(root: &Path, path: &'a Path) -> Result<&'a Path, HarnessError> {
    path.strip_prefix(root)
        .map_err(|_| HarnessError::invalid("AWS source escaped the workspace root"))
}

fn read_sources(root: &Path, paths: &[PathBuf]) -> Result<BTreeMap<String, String>, HarnessError> {
    let mut sources: BTreeMap<String, String> = BTreeMap::new();
    let mut total = 0_u64;
    for path in paths {
        let symlink = fs::symlink_metadata(path)?.file_type().is_symlink();
        if sources.len() >= MAX_FILES || symlink {
            return Err(HarnessError::invalid("unsupported AWS source inventory"));
        }
        let raw = read_bytes(path, MAX_INPUT_BYTES)?;
        total = total.saturating_add(u64::try_from(raw.len()).unwrap_or(u64::MAX));
        if total > MAX_SOURCE_BYTES {
            return Err(HarnessError::invalid("AWS source byte budget exceeded"));
        }
        let relative = relative_source(root, path)?;
        sources.insert(slash_path(relative), decode_utf8(raw, path)?);
    }
    if sources.is_empty() {
        return Err(HarnessError::invalid("empty AWS source inventory"));
    }
    Ok(sources)
}

fn collect_calls(
    sources: &BTreeMap<String, String>,
    hints: &BTreeMap<String, Option<String>>,
) -> Result<(Vec<CallSite>, usize), HarnessError> {
    let mut calls: Vec<CallSite> = Vec::new();
    let mut nonempty = 0_usize;
    for (path, source) in sources {
        let hint = hints.get(path).and_then(Option::as_deref);
        let (found, skipped) = calls_in_source_with_hint(path, source, hint)?;
        calls.extend(found);
        nonempty = nonempty.saturating_add(skipped);
        if calls.len() > MAX_CALLS {
            return Err(HarnessError::invalid("SDK call budget exceeded"));
        }
    }
    Ok((calls, nonempty))
}
