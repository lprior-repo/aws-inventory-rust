//! Resolve the crate version of every published operation from `Cargo.lock`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::HarnessError;

use super::{CRATE_PREFIX, INVENTORY};

/// Service namespace or crate name to `(crate, version)`.
pub(super) type Active = BTreeMap<String, (String, String)>;

/// One `[[package]]` stanza while scanning `Cargo.lock`.
#[derive(Default)]
struct Package {
    name: Option<String>,
    version: Option<String>,
    in_package: bool,
}

impl Package {
    fn absorb(&mut self, line: &str, packages: &mut BTreeMap<String, BTreeSet<String>>) {
        if line.starts_with("[[package]]") {
            self.flush(packages);
            self.in_package = true;
        } else if line.starts_with('[') {
            self.flush(packages);
            self.in_package = false;
        } else if self.in_package {
            if let Some(value) = quoted_assignment(line, "name") {
                self.name = Some(value);
            } else if let Some(value) = quoted_assignment(line, "version") {
                self.version = Some(value);
            }
        }
    }

    fn flush(&mut self, packages: &mut BTreeMap<String, BTreeSet<String>>) {
        let name = self.name.take();
        let version = self.version.take();
        if let (Some(name), Some(version)) = (name, version)
            && name.starts_with(CRATE_PREFIX)
        {
            packages.entry(name).or_default().insert(version);
        }
    }
}

fn quoted_assignment(line: &str, key: &str) -> Option<String> {
    let remainder = line.strip_prefix(key)?;
    let remainder = remainder.strip_prefix(" = \"")?;
    remainder.strip_suffix('"').map(str::to_owned)
}

/// Locked crate versions by crate name, or `None` without a lockfile.
pub(super) fn lockfile(
    root: &Path,
) -> Result<Option<BTreeMap<String, BTreeSet<String>>>, HarnessError> {
    let path = root.join("Cargo.lock");
    if !path.try_exists()? {
        return Ok(None);
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| HarnessError::invalid(format!("{}: {error}", path.display())))?;
    let mut packages: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut current = Package::default();
    for line in text.lines() {
        current.absorb(line.trim(), &mut packages);
    }
    current.flush(&mut packages);
    if packages.is_empty() {
        return Err(HarnessError::invalid(
            "Cargo.lock lists no aws-sdk packages",
        ));
    }
    Ok(Some(packages))
}

/// Resolve one active crate version per service namespace.
pub(super) fn resolve_active(
    versions: &BTreeMap<String, BTreeSet<(String, String)>>,
    locked: &Option<BTreeMap<String, BTreeSet<String>>>,
) -> Result<(Active, Active), HarnessError> {
    let mut active: Active = BTreeMap::new();
    let mut by_crate: Active = BTreeMap::new();
    for (service, candidates) in versions {
        let chosen = select_version(service, candidates, locked)?;
        by_crate.insert(chosen.0.clone(), (service.clone(), chosen.1.clone()));
        active.insert(service.clone(), chosen);
    }
    Ok((active, by_crate))
}

fn select_version(
    service: &str,
    candidates: &BTreeSet<(String, String)>,
    locked: &Option<BTreeMap<String, BTreeSet<String>>>,
) -> Result<(String, String), HarnessError> {
    match locked {
        Some(locked) => locked_version(service, candidates, locked),
        None => unique_version(service, candidates),
    }
}

fn locked_version(
    service: &str,
    candidates: &BTreeSet<(String, String)>,
    locked: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(String, String), HarnessError> {
    let mut matching = candidates.iter().filter(|(crate_name, version)| {
        locked
            .get(crate_name)
            .is_some_and(|set| set.contains(version))
    });
    let first = matching.next().ok_or_else(|| {
        HarnessError::invalid(format!(
            "Cargo.lock pins no version of {service} present in {INVENTORY}"
        ))
    })?;
    if matching.next().is_some() {
        return Err(HarnessError::invalid(format!(
            "Cargo.lock pins multiple versions of {service}; coverage is ambiguous"
        )));
    }
    Ok(first.clone())
}

fn unique_version(
    service: &str,
    candidates: &BTreeSet<(String, String)>,
) -> Result<(String, String), HarnessError> {
    let mut candidates = candidates.iter();
    let first = candidates
        .next()
        .ok_or_else(|| HarnessError::invalid(format!("{INVENTORY} has no crate for {service}")))?;
    if candidates.next().is_some() {
        return Err(HarnessError::invalid(format!(
            "{service} has multiple crate versions; Cargo.lock is required"
        )));
    }
    Ok(first.clone())
}
