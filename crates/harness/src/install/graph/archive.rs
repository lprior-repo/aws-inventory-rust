use std::collections::BTreeSet;
use std::fs::{File, Metadata};
use std::io::{Read, Seek, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::{EXPECTED_LINKS, MAX_DOWNLOAD, MAX_EXTRACT, MAX_MEMBERS, REDIS_ROOT, REDIS_SHA256};
use crate::HarnessError;

const MAX_LISTING: usize = 16 * 1024 * 1024;
const MAX_NAME: usize = 4096;
const POLLS: usize = 3000;

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

pub(super) fn extract(archive: &Path, source: &Path) -> Result<(), HarnessError> {
    let mut file = File::open(archive)?;
    verify(&mut file, REDIS_SHA256)?;
    extract_open(&mut file, source)
}

fn extract_open(file: &mut File, source: &Path) -> Result<(), HarnessError> {
    std::fs::create_dir(source)?;
    let owned = std::fs::symlink_metadata(source)?;
    let result = extract_claimed(file, source);
    match result {
        Ok(()) => Ok(()),
        Err(error) => match cleanup(source, &owned) {
            Ok(()) => Err(error),
            Err(cleanup) => Err(reject(&format!("{error}; cleanup failed: {cleanup}"))),
        },
    }
}

fn cleanup(source: &Path, owned: &Metadata) -> Result<(), HarnessError> {
    let current = std::fs::symlink_metadata(source)?;
    if !current.is_dir() || (current.dev(), current.ino()) != (owned.dev(), owned.ino()) {
        return Err(reject("source ownership changed"));
    }
    Ok(std::fs::remove_dir_all(source)?)
}

fn extract_claimed(file: &mut File, source: &Path) -> Result<(), HarnessError> {
    let listed = run(
        Command::new("bsdtar").args(["-tvf", "-", "--numeric-owner"]),
        file,
    )?;
    let listing = std::str::from_utf8(&listed).map_err(|_| reject("listing encoding"))?;
    validate(&parse(listing)?)?;
    run(
        Command::new("bsdtar")
            .args([
                "-xf",
                "-",
                "--strip-components",
                "1",
                "--no-same-owner",
                "-C",
            ])
            .arg(source),
        file,
    )?;
    Ok(())
}

struct HashWriter(Sha256);

impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn verify(file: &mut File, expected: &str) -> Result<(), HarnessError> {
    if !file.metadata()?.is_file() {
        return Err(reject("input type"));
    }
    file.rewind()?;
    let mut hash = HashWriter(Sha256::new());
    let size = std::io::copy(&mut file.take(MAX_DOWNLOAD.saturating_add(1)), &mut hash)?;
    let digest = format!("{:x}", base16ct::HexDisplay(&hash.0.finalize()));
    if size > MAX_DOWNLOAD || digest != expected {
        return Err(reject("digest or download byte bound"));
    }
    Ok(())
}

fn run(command: &mut Command, file: &mut File) -> Result<Vec<u8>, HarnessError> {
    file.rewind()?;
    let mut child = command
        .env("LC_ALL", "C")
        .stdin(Stdio::from(file.try_clone()?))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let Some(stdout) = child.stdout.take() else {
        return stop(&mut child, reject("missing listing pipe"));
    };
    std::thread::scope(|scope| {
        let reader = scope.spawn(|| read_output(stdout));
        let status = wait(&mut child);
        let output = reader.join().map_err(|_| reject("listing reader failed"))?;
        if !status?.success() {
            return Err(reject("bsdtar failed"));
        }
        output
    })
}

fn read_output(output: impl Read) -> Result<Vec<u8>, HarnessError> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(MAX_LISTING.saturating_add(1))
        .map_err(|_| reject("listing allocation"))?;
    let limit =
        u64::try_from(MAX_LISTING.saturating_add(1)).map_err(|_| reject("listing bound"))?;
    output.take(limit).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_LISTING {
        return Err(reject("listing byte bound"));
    }
    Ok(bytes)
}

fn wait(child: &mut Child) -> Result<ExitStatus, HarnessError> {
    let result = (0..POLLS).find_map(|_| match child.try_wait() {
        Ok(Some(status)) => Some(Ok(status)),
        Err(error) => Some(Err(HarnessError::from(error))),
        Ok(None) => {
            std::thread::sleep(Duration::from_millis(10));
            None
        }
    });
    match result {
        Some(Ok(status)) => Ok(status),
        Some(Err(error)) => stop(child, error),
        None => stop(child, reject("bsdtar deadline")),
    }
}

fn stop<T>(child: &mut Child, error: HarnessError) -> Result<T, HarnessError> {
    let killed = child.kill();
    let reaped = child.wait();
    match (killed, reaped) {
        (Ok(()), Ok(_)) => Err(error),
        (kill, reap) => Err(reject(&format!("{error}; kill: {kill:?}; reap: {reap:?}"))),
    }
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
