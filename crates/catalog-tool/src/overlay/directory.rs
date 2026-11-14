use super::{
    MAX_DIRECTORY_ENTRIES, MAX_FILE_BYTES, MAX_FILES, OverlayError, Rule, ServiceOverlay,
    error::Context,
};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

/// Read `root/catalog/overlay/*.toml`, sorted by filename, with stems as service names.
/// Non-TOML entries are ignored; at most 4,096 matching files, each at most 1 MiB.
/// Traversal is capped at 65,536 entries, including ignored non-TOML entries.
/// # Errors
/// Returns filesystem, invalid stem, byte/count limit or contextual parsing errors.
pub fn load_dir(root: impl AsRef<Path>) -> Result<Vec<ServiceOverlay>, OverlayError> {
    let dir = root.as_ref().join("catalog/overlay");
    let paths = collect_paths(&dir, MAX_FILES, MAX_DIRECTORY_ENTRIES)?;
    paths.iter().map(|p| load_file(p)).collect()
}
fn load_file(path: &Path) -> Result<ServiceOverlay, OverlayError> {
    let service = service_stem(path)?;
    let context = Context {
        service,
        operation: None,
        prefix: String::new(),
    };
    let text = read_file(&context, path)?;
    ServiceOverlay::parse(service, &text)
}
fn service_stem(path: &Path) -> Result<&str, OverlayError> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            let context = Context {
                service: "",
                operation: None,
                prefix: String::new(),
            };
            context.error("service", path.to_string_lossy(), Rule::Service)
        })
}
fn read_file(context: &Context<'_>, path: &Path) -> Result<String, OverlayError> {
    let file = File::open(path).map_err(|source| io(path, source))?;
    let (limit, capacity) = file_budget(context, path, &file)?;
    let mut text = String::new();
    text.try_reserve(capacity.saturating_add(1))
        .map_err(|_| context.error("", "", Rule::Resource))?;
    file.take(limit.saturating_add(1))
        .read_to_string(&mut text)
        .map_err(|source| io(path, source))?;
    Ok(text)
}
fn file_budget(
    context: &Context<'_>,
    path: &Path,
    file: &File,
) -> Result<(u64, usize), OverlayError> {
    let metadata = file.metadata().map_err(|source| io(path, source))?;
    let limit = u64::try_from(MAX_FILE_BYTES).map_err(|_| context.error("", "", Rule::Resource))?;
    if metadata.len() > limit {
        return Err(context.error(
            "",
            metadata.len(),
            Rule::Limit {
                max: MAX_FILE_BYTES,
            },
        ));
    }
    let capacity =
        usize::try_from(metadata.len()).map_err(|_| context.error("", "", Rule::Resource))?;
    Ok((limit, capacity))
}
fn collect_paths(dir: &Path, files: usize, entries: usize) -> Result<Vec<PathBuf>, OverlayError> {
    let context = Context {
        service: "",
        operation: None,
        prefix: String::new(),
    };
    let source = std::fs::read_dir(dir).map_err(|source| io(dir, source))?;
    let (_, mut paths) = source.take(entries.saturating_add(1)).try_fold(
        (0usize, Vec::new()),
        |(count, mut paths), entry| {
            entry_budget(&context, count, entries)?;
            let path = entry.map_err(|source| io(dir, source))?.path();
            append_toml(&context, &mut paths, path, files)?;
            Ok((count.saturating_add(1), paths))
        },
    )?;
    paths.sort();
    Ok(paths)
}
fn entry_budget(context: &Context<'_>, count: usize, entries: usize) -> Result<(), OverlayError> {
    if count == entries {
        return Err(context.error(
            "entries",
            count.saturating_add(1),
            Rule::Limit { max: entries },
        ));
    }
    Ok(())
}
fn append_toml(
    context: &Context<'_>,
    paths: &mut Vec<PathBuf>,
    path: PathBuf,
    files: usize,
) -> Result<(), OverlayError> {
    if path
        .extension()
        .is_some_and(|extension| extension == "toml")
    {
        if paths.len() == files {
            return Err(context.error(
                "files",
                files.saturating_add(1),
                Rule::Limit { max: files },
            ));
        }
        paths
            .try_reserve(1)
            .map_err(|_| context.error("files", "", Rule::Resource))?;
        paths.push(path);
    }
    Ok(())
}
fn io(path: &Path, source: std::io::Error) -> OverlayError {
    OverlayError::Io {
        path: PathBuf::from(path),
        source,
    }
}

#[cfg(test)]
#[path = "directory_tests.rs"]
mod tests;
