//! Builds the operation inventory from a registry and enforces the
//! lockfile completeness gate.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::Error;
use crate::digest;
use crate::lockfile;
use crate::model::{CrateEntry, Inventory, Operation, SCHEMA_VERSION, Source, Summary};
use crate::registry::{self, CrateDir};

/// Inventory every SDK crate in `registry_root`.
///
/// Fails when the registry omits a crate pinned by `lockfile_path`, so a
/// partial download can never masquerade as complete coverage.
pub fn build(registry_root: &Path, lockfile_path: &Path) -> Result<Inventory, Error> {
    let registry_path =
        fs::canonicalize(registry_root).map_err(|source| Error::io(registry_root, source))?;
    let directories = registry::discover(&registry_path)?;
    verify_completeness(&registry_path, lockfile_path, &directories)?;
    let mut crates = Vec::with_capacity(directories.len());
    let mut operations = 0_usize;
    for directory in &directories {
        let entry = crate_entry(directory)?;
        operations = operations.saturating_add(entry.operations.len());
        crates.push(entry);
    }
    Ok(Inventory {
        schema_version: SCHEMA_VERSION,
        source: Source {
            registry: registry_path.to_string_lossy().into_owned(),
        },
        summary: Summary {
            crates: crates.len(),
            operations,
        },
        crates,
    })
}

/// Render the canonical pretty form with one trailing newline.
pub fn render(inventory: &Inventory) -> Result<Vec<u8>, Error> {
    let mut bytes = serde_json::to_vec_pretty(inventory)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Write the rendered inventory to `path`.
pub fn write(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    fs::write(path, bytes).map_err(|source| Error::io(path, source))
}

/// Require `path` to be byte-identical to a fresh render.
pub fn verify(path: &Path, expected: &[u8]) -> Result<(), Error> {
    let existing = fs::read(path).map_err(|source| Error::io(path, source))?;
    if existing == expected {
        Ok(())
    } else {
        Err(Error::StaleOutput {
            path: path.to_path_buf(),
        })
    }
}

fn verify_completeness(
    registry_root: &Path,
    lockfile_path: &Path,
    directories: &[CrateDir],
) -> Result<(), Error> {
    let available: BTreeSet<(&str, &str)> = directories
        .iter()
        .map(|directory| (directory.name.as_str(), directory.version.as_str()))
        .collect();
    let missing: Vec<String> = lockfile::sdk_packages(lockfile_path)?
        .into_iter()
        .filter(|(name, version)| !available.contains(&(name.as_str(), version.as_str())))
        .map(|(name, version)| format!("{name}-{version}"))
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    Err(Error::MissingFromRegistry {
        path: lockfile_path.to_path_buf(),
        registry: registry_root.to_path_buf(),
        missing: missing.join(", "),
    })
}

fn crate_entry(directory: &CrateDir) -> Result<CrateEntry, Error> {
    let operation_dir = directory.path.join("src").join("operation");
    let mut modules = operation_modules(directory, &operation_dir)?;
    modules.sort_by(|left, right| left.0.cmp(&right.0));
    if modules.is_empty() {
        return Err(Error::EmptyOperations {
            name: directory.name.clone(),
            path: operation_dir,
        });
    }
    let mut operations = Vec::with_capacity(modules.len());
    for (module, path) in modules {
        operations.push(Operation {
            api_operation: pascal_case(&module),
            source_sha256: digest::file_sha256(&path)?,
            module,
        });
    }
    Ok(CrateEntry {
        name: directory.name.clone(),
        version: directory.version.clone(),
        sha256: digest::tree_digest(&directory.path)?,
        operations,
    })
}

fn operation_modules(
    directory: &CrateDir,
    operation_dir: &Path,
) -> Result<Vec<(String, std::path::PathBuf)>, Error> {
    if !operation_dir.is_dir() {
        return Err(Error::MissingOperations {
            name: directory.name.clone(),
            path: operation_dir.to_path_buf(),
        });
    }
    let entries = fs::read_dir(operation_dir).map_err(|source| Error::io(operation_dir, source))?;
    let mut modules = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::io(operation_dir, source))?;
        let path = entry.path();
        if !entry
            .file_type()
            .map_err(|source| Error::io(&path, source))?
            .is_file()
        {
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if stem == "mod" {
            continue;
        }
        modules.push((stem.to_owned(), path));
    }
    Ok(modules)
}

/// Convert an operation module name to its PascalCase API operation name.
///
/// `list_resources` becomes `ListResources`; generated module names are the
/// SDK's stable operation identifiers.
pub fn pascal_case(module: &str) -> String {
    let mut out = String::with_capacity(module.len());
    for segment in module.split('_') {
        let mut characters = segment.chars();
        if let Some(first) = characters.next() {
            out.extend(first.to_uppercase());
            out.push_str(characters.as_str());
        }
    }
    out
}
