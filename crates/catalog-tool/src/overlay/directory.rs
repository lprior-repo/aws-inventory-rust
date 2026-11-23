use super::{Origin, ServiceOverlay};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub const MAX_FILES: usize = 4096;
pub const MAX_DIRECTORY_ENTRIES: usize = 8192;

#[derive(Debug, Error)]
pub enum OverlayError {
    #[error("{file}: {source}")]
    Io {
        file: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Name(#[from] super::NameError),
    #[error(transparent)]
    Decode(#[from] toml::de::Error),
    #[error("invalid overlay: {0}")]
    Invalid(String),
    #[error("overlay resource limit exceeded: {0}")]
    Limit(PathBuf),
}

pub fn load(root: &Path) -> Result<Vec<ServiceOverlay>, OverlayError> {
    super::merge::merge(super::embedded()?, load_generated_dir(root)?)
}

pub fn load_generated_dir(root: &Path) -> Result<Vec<ServiceOverlay>, OverlayError> {
    let directory = root.join("catalog/overlay/generated");
    let mut policies = Vec::new();
    for file in generated_files(&directory)? {
        let service = file
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| OverlayError::Invalid(file.display().to_string()))?;
        let text = read_text(&file)?;
        policies.push(ServiceOverlay::parse(service, &text, Origin::Generated)?);
    }
    Ok(policies)
}

pub(crate) fn generated_files(directory: &Path) -> Result<Vec<PathBuf>, OverlayError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(io(directory, source)),
    };
    let mut files = Vec::new();
    for (index, entry) in entries.enumerate() {
        if index >= MAX_DIRECTORY_ENTRIES {
            return Err(OverlayError::Limit(directory.to_path_buf()));
        }
        collect_file(entry.map_err(|source| io(directory, source))?, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn collect_file(entry: fs::DirEntry, files: &mut Vec<PathBuf>) -> Result<(), OverlayError> {
    let path = entry.path();
    if path.extension().is_none_or(|extension| extension != "toml") {
        return Ok(());
    }
    let kind = entry.file_type().map_err(|source| io(&path, source))?;
    if !kind.is_file() {
        return Err(OverlayError::Invalid(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    if files.len() >= MAX_FILES {
        return Err(OverlayError::Limit(path));
    }
    files.push(path);
    Ok(())
}

pub(crate) fn read_text(file: &Path) -> Result<String, OverlayError> {
    let input = fs::File::open(file).map_err(|source| io(file, source))?;
    let mut text = String::new();
    input
        .take(1_048_577)
        .read_to_string(&mut text)
        .map_err(|source| io(file, source))?;
    if text.len() > super::MAX_FILE_BYTES {
        return Err(OverlayError::Limit(file.to_path_buf()));
    }
    Ok(text)
}

pub(crate) fn io(file: &Path, source: std::io::Error) -> OverlayError {
    OverlayError::Io {
        file: file.to_path_buf(),
        source,
    }
}
