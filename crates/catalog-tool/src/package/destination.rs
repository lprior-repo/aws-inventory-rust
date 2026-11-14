//! Destination isolation and exclusive, same-directory atomic publication.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

/// Write through an exclusively created temporary file, then atomically rename it.
///
/// The callback must finish all validation before returning. Any callback, flush,
/// or rename failure drops the temporary file without replacing the destination.
/// Existing destination symlinks are replaced, never followed. Parent directories
/// are created as needed; their creation is not rolled back on failure.
/// On Unix, creation permissions are `0o666` restricted by the current umask,
/// matching `File::create` without reading or changing the process umask.
///
/// # Errors
/// Returns filesystem or callback failures. No temporary pathname is reused.
pub fn atomic_write<T>(
    destination: &Path,
    write: impl FnOnce(&mut File) -> Result<T, Box<dyn std::error::Error>>,
) -> Result<T, Box<dyn std::error::Error>> {
    let absolute = std::path::absolute(destination)?;
    let parent = absolute
        .parent()
        .ok_or_else(|| io::Error::other("missing output parent"))?;
    fs::create_dir_all(parent)?;
    let mut temporary = temporary_file(parent)?;
    let value = write(temporary.as_file_mut())?;
    temporary.as_file_mut().flush()?;
    temporary
        .persist(&absolute)
        .map_err(|error| io::Error::new(error.error.kind(), error.to_string()))?;
    Ok(value)
}

/// Ask the kernel to apply its current umask during exclusive creation.
#[cfg(unix)]
fn temporary_file(parent: &Path) -> io::Result<tempfile::NamedTempFile> {
    use std::os::unix::fs::PermissionsExt;
    tempfile::Builder::new()
        .permissions(fs::Permissions::from_mode(0o666))
        .tempfile_in(parent)
}

/// Exclusive creation with the platform's default access.
#[cfg(not(unix))]
fn temporary_file(parent: &Path) -> io::Result<tempfile::NamedTempFile> {
    tempfile::Builder::new().tempfile_in(parent)
}

/// Resolve directories, but not the final component that rename will replace.
pub(super) fn validate(root: &Path, destination: &Path) -> io::Result<(PathBuf, PathBuf)> {
    let canonical_root = fs::canonicalize(root)?;
    let absolute = std::path::absolute(destination)?;
    let parent = absolute
        .parent()
        .ok_or_else(|| io::Error::other("missing output parent"))?;
    let canonical_parent = parent
        .components()
        .try_fold(PathBuf::new(), resolve_component)?;
    if canonical_parent.starts_with(&canonical_root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "destination {} is inside packaged root {}; choose a destination outside the root",
                destination.display(),
                root.display()
            ),
        ));
    }
    let name = absolute
        .file_name()
        .ok_or_else(|| io::Error::other("missing output name"))?;
    Ok((canonical_root, canonical_parent.join(name)))
}

/// Resolve each existing directory before interpreting subsequent `..` segments.
fn resolve_component(mut path: PathBuf, component: Component<'_>) -> io::Result<PathBuf> {
    if component == Component::ParentDir {
        path.pop();
        return Ok(path);
    }
    path.push(component.as_os_str());
    match fs::canonicalize(&path) {
        Ok(resolved) => Ok(resolved),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(path),
        Err(error) => Err(error),
    }
}
