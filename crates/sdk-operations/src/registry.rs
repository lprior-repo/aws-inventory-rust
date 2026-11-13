//! Discovery and naming of the generated `aws-sdk-*` crates in a registry.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

/// One generated SDK crate directory.
#[derive(Debug, Clone)]
pub struct CrateDir {
    /// Package name without the version, e.g. `aws-sdk-cloudcontrol`.
    pub name: String,
    /// Package version parsed from the directory suffix.
    pub version: String,
    /// Directory that holds the crate sources.
    pub path: PathBuf,
}

/// Enumerate `aws-sdk-<service>-<version>` directories, sorted by name then version.
pub fn discover(registry: &Path) -> Result<Vec<CrateDir>, Error> {
    let entries = fs::read_dir(registry).map_err(|source| Error::io(registry, source))?;
    let mut crates = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::io(registry, source))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| Error::io(&path, source))?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| Error::NonUtf8CrateName { path: path.clone() })?;
        if !name.starts_with("aws-sdk-") {
            continue;
        }
        let (service, version) = parse_directory_name(&name)
            .ok_or_else(|| Error::InvalidCrateDirectory { name: name.clone() })?;
        if !seen.insert((service.clone(), version.clone())) {
            return Err(Error::DuplicateCrate {
                name: service,
                version,
            });
        }
        crates.push(CrateDir {
            name: service,
            version,
            path,
        });
    }
    crates.sort_by(|left, right| (&left.name, &left.version).cmp(&(&right.name, &right.version)));
    if crates.is_empty() {
        return Err(Error::EmptyRegistry {
            path: registry.to_path_buf(),
        });
    }
    Ok(crates)
}

/// Split `aws-sdk-<service>-<version>`, tolerating hyphens in the service name.
///
/// The version is the rightmost hyphen suffix that starts with a digit and is
/// built only from version characters, so a pre-release suffix stays intact.
pub fn parse_directory_name(directory: &str) -> Option<(String, String)> {
    let rest = directory.strip_prefix("aws-sdk-")?;
    for (index, _) in rest.match_indices('-').rev() {
        let (service, tail) = rest.split_at(index);
        let Some(version) = tail.strip_prefix('-') else {
            continue;
        };
        if !service.is_empty() && is_version(version) {
            return Some((format!("aws-sdk-{service}"), version.to_owned()));
        }
    }
    None
}

fn is_version(candidate: &str) -> bool {
    let Some(first) = candidate.as_bytes().first() else {
        return false;
    };
    first.is_ascii_digit()
        && candidate.contains('.')
        && candidate
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
}
