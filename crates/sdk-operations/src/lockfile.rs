//! Minimal `Cargo.lock` scanner for the completeness gate.
//!
//! Only the `name`/`version` pair of each `[[package]]` block is needed, so the
//! stable lockfile text format is read directly instead of pulling a TOML
//! parser into the inventory crate.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::Error;

/// Every `aws-sdk-*` package pinned by the lockfile, sorted by name then version.
pub fn sdk_packages(path: &Path) -> Result<Vec<(String, String)>, Error> {
    let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    let mut packages = BTreeSet::new();
    let mut current = Package::default();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("[[package]]") {
            current.flush(&mut packages);
            current.in_package = true;
        } else if line.starts_with('[') {
            current.flush(&mut packages);
            current.in_package = false;
        } else if current.in_package {
            if let Some(value) = quoted_assignment(line, "name") {
                current.name = Some(value);
            } else if let Some(value) = quoted_assignment(line, "version") {
                current.version = Some(value);
            }
        }
    }
    current.flush(&mut packages);
    if packages.is_empty() {
        return Err(Error::EmptyLockfile {
            path: path.to_path_buf(),
        });
    }
    Ok(packages.into_iter().collect())
}

#[derive(Default)]
struct Package {
    name: Option<String>,
    version: Option<String>,
    in_package: bool,
}

impl Package {
    fn flush(&mut self, packages: &mut BTreeSet<(String, String)>) {
        let name = self.name.take();
        let version = self.version.take();
        if let (Some(name), Some(version)) = (name, version)
            && name.starts_with("aws-sdk-")
        {
            packages.insert((name, version));
        }
    }
}

fn quoted_assignment(line: &str, key: &str) -> Option<String> {
    let remainder = line.strip_prefix(key)?;
    let remainder = remainder.strip_prefix(" = \"")?;
    remainder.strip_suffix('"').map(str::to_owned)
}
