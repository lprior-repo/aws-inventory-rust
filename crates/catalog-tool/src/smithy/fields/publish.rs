use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use super::{Error, Snapshot, check_registry_size, io_error};

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
    publish_bytes(
        path,
        original.text().map(str::as_bytes),
        text.as_bytes(),
        limit,
    )
}

pub(in crate::smithy) fn publish_bytes(
    path: &Path,
    original: Option<&[u8]>,
    bytes: &[u8],
    limit: u64,
) -> Result<(), Error> {
    check_registry_size(path, bytes, limit)?;
    let dir = path
        .parent()
        .ok_or_else(|| io_error(path, io::Error::other("no parent")))?;
    fs::create_dir_all(dir).map_err(|source| io_error(dir, source))?;
    let mut filename = path.as_os_str().to_owned();
    filename.push(".tmp");
    let temporary = std::path::PathBuf::from(filename);
    let mut file = create_temporary(&temporary)?;
    let result = compare_snapshot(path, original, limit)
        .and_then(|()| publish_locked(path, &temporary, &mut file, original, bytes));
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
    original: Option<&[u8]>,
    bytes: &[u8],
) -> Result<Publication, Error> {
    if original == Some(bytes) {
        return Ok(Publication::Unchanged);
    }
    write_and_rename(path, temporary, file, bytes)
}

fn compare_snapshot(path: &Path, original: Option<&[u8]>, limit: u64) -> Result<(), Error> {
    if read_bytes(path, limit)?.as_deref() != original {
        return Err(Error::RegistryChanged {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn write_and_rename(
    path: &Path,
    temporary: &Path,
    file: &mut File,
    bytes: impl AsRef<[u8]>,
) -> Result<Publication, Error> {
    file.write_all(bytes.as_ref())
        .map_err(|source| io_error(temporary, source))?;
    fs::rename(temporary, path).map_err(|source| io_error(path, source))?;
    Ok(Publication::Renamed)
}

pub(in crate::smithy) fn read_bytes(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, Error> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(path, error)),
    };
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| io_error(path, error))?;
    check_registry_size(path, &bytes, limit)?;
    Ok(Some(bytes))
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
