use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process::Command;
#[cfg(not(test))]
use std::time::Duration;

#[path = "archive_process.rs"]
mod process;
#[path = "archive_snapshot.rs"]
mod snapshot;

#[cfg(test)]
use snapshot::verify;
#[cfg(test)]
pub(super) use tests::lifecycle::{download_program, download_timeout};

use super::{
    EXPECTED_LINKS, MAX_DOWNLOAD, MAX_EXTRACT, MAX_MEMBERS, OwnedDirectory, REDIS_ROOT,
    REDIS_SHA256,
};
use crate::HarnessError;

const MAX_LISTING: usize = 16 * 1024 * 1024;
const MAX_NAME: usize = 4096;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Kind {
    File,
    Directory,
    Symlink(String),
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Entry {
    pub(super) kind: Kind,
    pub(super) size: u64,
    pub(super) name: String,
}

fn reject(reason: &str) -> HarnessError {
    HarnessError::invalid(format!("unexpected Redis archive {reason}"))
}

pub(super) fn extract(archive: &Path, source: &Path) -> Result<OwnedDirectory, HarnessError> {
    extract_pinned(archive, source, REDIS_SHA256)
}

fn extract_pinned(
    archive: &Path,
    source: &Path,
    expected: &str,
) -> Result<OwnedDirectory, HarnessError> {
    let input = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
        .open(archive)?;
    let mut file = snapshot::capture(input)?;
    snapshot::verify(&mut file, expected)?;
    #[cfg(test)]
    tests::acquisition::checkpoint(&file)?;
    extract_claimed(&mut file, OwnedDirectory::claim(source)?)
}

#[cfg(test)]
fn extract_open(file: &mut File, source: &Path) -> Result<OwnedDirectory, HarnessError> {
    extract_claimed(file, OwnedDirectory::claim(source)?)
}

fn extract_claimed(file: &mut File, owned: OwnedDirectory) -> Result<OwnedDirectory, HarnessError> {
    match extract_into(file, &owned) {
        Ok(()) => Ok(owned),
        Err(error) => match owned.remove_tree() {
            Ok(()) => Err(error),
            Err(cleanup) => Err(reject(&format!("{error}; cleanup failed: {cleanup}"))),
        },
    }
}

fn extract_into(file: &mut File, owned: &OwnedDirectory) -> Result<(), HarnessError> {
    let listed = run(tar().args(["-tvf", "-", "--numeric-owner"]), file)?;
    let listing = std::str::from_utf8(&listed).map_err(|_| reject("listing encoding"))?;
    validate(&parse(listing)?)?;
    #[cfg(test)]
    tests::acquisition::checkpoint(file)?;
    run(
        tar()
            .args(["-xf", "-", "--strip-components", "1", "--no-same-owner"])
            .current_dir(owned.anchor()),
        file,
    )?;
    Ok(())
}

fn tar() -> Command {
    #[cfg(test)]
    let program = tests::lifecycle::archive_program();
    #[cfg(not(test))]
    let program = "bsdtar";
    Command::new(program)
}

fn run(command: &mut Command, file: &mut File) -> Result<Vec<u8>, HarnessError> {
    #[cfg(test)]
    let timeout = tests::lifecycle::archive_timeout();
    #[cfg(not(test))]
    let timeout = Duration::from_secs(30);
    process::run(command, file, timeout)
}

pub(super) fn parse(listing: &str) -> Result<Vec<Entry>, HarnessError> {
    if listing.len() > MAX_LISTING
        || listing
            .bytes()
            .any(|byte| byte.is_ascii_control() && byte != b'\n')
        || listing.lines().take(MAX_MEMBERS.saturating_add(1)).count() > MAX_MEMBERS
    {
        return Err(reject("listing bound"));
    }
    let mut entries = Vec::new();
    entries
        .try_reserve(listing.lines().count())
        .map_err(|_| reject("member allocation"))?;
    listing.lines().try_for_each(|line| {
        entries.push(parse_line(line)?);
        Ok::<(), HarnessError>(())
    })?;
    Ok(entries)
}

fn field(input: &str) -> Result<(&str, &str), HarnessError> {
    let input = input.trim_start_matches(' ');
    let end = input.find(' ').ok_or_else(|| reject("listing field"))?;
    let value = input.get(..end).ok_or_else(|| reject("listing field"))?;
    let rest = input
        .get(end.saturating_add(1)..)
        .ok_or_else(|| reject("listing field"))?;
    if value.is_empty() {
        return Err(reject("listing field"));
    }
    Ok((value, rest))
}

fn parse_line(line: &str) -> Result<Entry, HarnessError> {
    let (mode, rest) = field(line)?;
    let (_, rest) = field(rest)?;
    let (_, rest) = field(rest)?;
    let (_, rest) = field(rest)?;
    let (size, rest) = field(rest)?;
    let (_, rest) = field(rest)?;
    let (_, rest) = field(rest)?;
    let (_, name) = field(rest)?;
    let size = size.parse::<u64>().map_err(|_| reject("listing size"))?;
    entry(mode, size, name)
}

fn entry(mode: &str, size: u64, listed: &str) -> Result<Entry, HarnessError> {
    if listed.len() > MAX_NAME || listed.chars().any(|c| c.is_control() || c == '\\') {
        return Err(reject("member name encoding"));
    }
    let (kind, name) = match mode.chars().next() {
        Some('-') => (Kind::File, listed),
        Some('d') => (
            Kind::Directory,
            listed.strip_suffix('/').map_or(listed, |s| s),
        ),
        Some('l') => {
            let (name, target) = listed
                .split_once(" -> ")
                .ok_or_else(|| reject("link target"))?;
            (Kind::Symlink(target.to_owned()), name)
        }
        _ => return Err(reject("member type")),
    };
    Ok(Entry {
        kind,
        size,
        name: name.to_owned(),
    })
}

pub(super) fn validate(entries: &[Entry]) -> Result<(), HarnessError> {
    if entries.is_empty() || entries.len() > MAX_MEMBERS {
        return Err(reject("member count"));
    }
    entries
        .iter()
        .try_fold((BTreeSet::new(), 0_u64), |(mut seen, total), entry| {
            if !name_allowed(&entry.name, &seen) || !kind_allowed(entry) {
                return Err(reject("member or link"));
            }
            let total = total
                .checked_add(entry.size)
                .filter(|n| *n <= MAX_EXTRACT)
                .ok_or_else(|| reject("extraction byte bound"))?;
            seen.insert(entry.name.as_str());
            Ok((seen, total))
        })
        .map(|_| ())
}

fn name_allowed(name: &str, seen: &BTreeSet<&str>) -> bool {
    let mut components = name.split('/');
    components.next() == Some(REDIS_ROOT)
        && name.len() <= MAX_NAME
        && !name.chars().any(|c| c.is_control() || c == '\\')
        && components.all(|part| !part.is_empty() && part != "." && part != "..")
        && !seen.contains(name)
        && !EXPECTED_LINKS.iter().any(|(link, _)| {
            name.strip_prefix(link)
                .is_some_and(|tail| tail.starts_with('/'))
        })
}

fn kind_allowed(entry: &Entry) -> bool {
    if entry.name == REDIS_ROOT && entry.kind != Kind::Directory {
        return false;
    }
    match &entry.kind {
        Kind::Symlink(target) => {
            entry.size == 0
                && EXPECTED_LINKS
                    .iter()
                    .any(|(link, expected)| *link == entry.name && expected == target)
        }
        Kind::Directory => entry.size == 0,
        Kind::File => true,
    }
}

#[cfg(test)]
#[path = "archive_tests.rs"]
mod tests;
