pub(super) mod publish;

use inventory_core::record::FieldId;
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use super::{Error, Registry, RegistryError, io_error, read_model_file, totals::member_names};
use crate::{ReportMode, report::write_stdout};

const FIELD_IDS_PATH: &str = "catalog/field-ids.txt";

const MAX_REGISTRY_BYTES: u64 = 16_777_216;

#[derive(Debug)]
enum Snapshot {
    Missing(io::Error),
    Present(String),
}

impl Snapshot {
    fn text(&self) -> Option<&str> {
        match self {
            Self::Missing(_) => None,
            Self::Present(text) => Some(text),
        }
    }
}

struct PreparedRegistry {
    original: Snapshot,
    next: Registry,
    added: usize,
}

pub fn fields(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    let files = crate::model_source::discover_models(&source.join("models"))?;
    if files.is_empty() {
        return Err(crate::model_source::Error::NoModels.into());
    }
    let names = collect_names(&files)?;
    let path = root.join(FIELD_IDS_PATH);
    let prepared = prepare_registry(&path, names, MAX_REGISTRY_BYTES)?;
    match mode {
        ReportMode::Print => write_stdout(&prepared.next.render()).map_err(Error::from),
        ReportMode::Write => write_fields(&path, &prepared),
        ReportMode::Check => check_fields(&path, prepared),
    }
}

fn collect_names(files: &[PathBuf]) -> Result<BTreeSet<String>, Error> {
    files.iter().try_fold(BTreeSet::new(), |mut names, file| {
        names.extend(member_names(read_model_file(file)?));
        Ok(names)
    })
}

fn read_registry(path: &Path, limit: u64) -> Result<Snapshot, Error> {
    match fs::File::open(path) {
        Ok(file) => read_open_registry(path, file, limit),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(Snapshot::Missing(source)),
        Err(source) => Err(io_error(path, source)),
    }
}

pub(super) fn load_registry(path: &Path) -> Result<Registry, Error> {
    match read_registry(path, MAX_REGISTRY_BYTES)? {
        Snapshot::Present(text) => Ok(Registry::parse(&text)?),
        Snapshot::Missing(source) => Err(io_error(path, source)),
    }
}

fn read_open_registry(path: &Path, file: fs::File, limit: u64) -> Result<Snapshot, Error> {
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(path, source))?;
    check_registry_size(path, &bytes, limit)?;
    let text = String::from_utf8(bytes)
        .map_err(|source| io_error(path, io::Error::new(io::ErrorKind::InvalidData, source)))?;
    Ok(Snapshot::Present(text))
}

fn registry_too_large(path: &Path, limit: u64) -> Error {
    Error::RegistryTooLarge {
        path: path.to_path_buf(),
        limit,
    }
}

fn check_registry_size(path: &Path, bytes: &[u8], limit: u64) -> Result<(), Error> {
    let length = u64::try_from(bytes.len()).map_err(|_| registry_too_large(path, limit))?;
    if length > limit {
        return Err(registry_too_large(path, limit));
    }
    Ok(())
}

fn prepare_registry(
    path: &Path,
    names: BTreeSet<String>,
    limit: u64,
) -> Result<PreparedRegistry, Error> {
    let original = read_registry(path, limit)?;
    let existing = original.text().map(Registry::parse).transpose()?;
    let previous_len = existing.as_ref().map_or(0, Registry::len);
    let next = candidate_registry(existing, names)?;
    let added = next
        .len()
        .checked_sub(previous_len)
        .ok_or(RegistryError::IdOverflow)?;
    Ok(PreparedRegistry {
        original,
        next,
        added,
    })
}

fn candidate_registry(
    existing: Option<Registry>,
    names: BTreeSet<String>,
) -> Result<Registry, Error> {
    let Some(previous) = existing else {
        return extend_registry(Registry::default(), names);
    };
    extend_registry(previous, names)
}

fn extend_registry(previous: Registry, names: BTreeSet<String>) -> Result<Registry, Error> {
    let next = previous.clone().extend(names)?;
    Registry::verify_prefix(&previous, &next)?;
    Ok(next)
}

fn check_fields(path: &Path, prepared: PreparedRegistry) -> Result<(), Error> {
    if let Snapshot::Missing(source) = prepared.original {
        return Err(io_error(path, source));
    }
    if prepared.added != 0 {
        let previous_len = prepared
            .next
            .len()
            .checked_sub(prepared.added)
            .ok_or(RegistryError::IdOverflow)?;
        return Err(stale_error(
            path,
            previous_len,
            prepared.added,
            &prepared.next,
        )?);
    }
    Ok(())
}

fn stale_error(
    path: &Path,
    previous_len: usize,
    added: usize,
    next: &Registry,
) -> Result<Error, Error> {
    let line = previous_len
        .checked_add(1)
        .ok_or(RegistryError::IdOverflow)?;
    let raw = u32::try_from(line).map_err(|_| RegistryError::IdOverflow)?;
    let id = FieldId::new(raw).map_err(|_| RegistryError::IdOverflow)?;
    let first = next.name(id).ok_or(RegistryError::IdOverflow)?.to_owned();
    Ok(Error::RegistryStale {
        path: path.to_path_buf(),
        new_names: added,
        first,
    })
}

fn write_fields(path: &Path, prepared: &PreparedRegistry) -> Result<(), Error> {
    publish::publish(
        path,
        &prepared.original,
        &prepared.next.render(),
        MAX_REGISTRY_BYTES,
    )?;
    Ok(())
}
