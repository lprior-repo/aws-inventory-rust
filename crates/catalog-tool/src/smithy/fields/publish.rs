//! Exclusive publication rejects a stale snapshot and never truncates a live inode.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::{Error, Snapshot, check_registry_size, io_error, read_registry};

#[cfg(test)]
mod tests;

enum Publication {
    Renamed,
    Unchanged,
}

pub(super) fn publish(
    path: &Path,
    original: &Snapshot,
    text: &str,
    limit: u64,
) -> Result<(), Error> {
    check_registry_size(path, text.as_bytes(), limit)?;
    let dir = path
        .parent()
        .ok_or_else(|| io_error(path, io::Error::other("no parent")))?;
    fs::create_dir_all(dir).map_err(|source| io_error(dir, source))?;
    let temporary = dir.join("field-ids.txt.tmp");
    let mut file = create_temporary(&temporary)?;
    let result = publish_locked(path, &temporary, &mut file, original, text, limit);
    drop(file);
    finish_publication(&temporary, result)
}

fn create_temporary(path: &Path) -> Result<File, Error> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| {
            if source.kind() == io::ErrorKind::AlreadyExists {
                Error::WriteInProgress {
                    path: path.to_path_buf(),
                }
            } else {
                io_error(path, source)
            }
        })
}

fn publish_locked(
    path: &Path,
    temporary: &Path,
    file: &mut File,
    original: &Snapshot,
    text: &str,
    limit: u64,
) -> Result<Publication, Error> {
    let current = read_registry(path, limit)?;
    if current.text() != original.text() {
        return Err(Error::RegistryChanged {
            path: path.to_path_buf(),
        });
    }
    if original.text() == Some(text) {
        return Ok(Publication::Unchanged);
    }
    write_and_rename(path, temporary, file, text)
}

fn write_and_rename(
    path: &Path,
    temporary: &Path,
    file: &mut File,
    text: &str,
) -> Result<Publication, Error> {
    file.write_all(text.as_bytes())
        .map_err(|source| io_error(temporary, source))?;
    fs::rename(temporary, path).map_err(|source| io_error(path, source))?;
    Ok(Publication::Renamed)
}

fn finish_publication(temporary: &Path, result: Result<Publication, Error>) -> Result<(), Error> {
    match result {
        Ok(Publication::Renamed) => Ok(()),
        Ok(Publication::Unchanged) => {
            fs::remove_file(temporary).map_err(|source| io_error(temporary, source))
        }
        Err(error) => Err(cleanup_error(temporary, error)),
    }
}

fn cleanup_error(temporary: &Path, original: Error) -> Error {
    match fs::remove_file(temporary) {
        Ok(()) => original,
        Err(cleanup) => Error::PublicationCleanup {
            original: Box::new(original),
            path: temporary.to_path_buf(),
            cleanup,
        },
    }
}
