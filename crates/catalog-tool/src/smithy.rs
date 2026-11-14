//! One-model-at-a-time AWS statistics, with pure counters and canonical reports.
//! Reads every `.json` below `source/models`, using case-sensitive operation
//! prefixes and exact input targets. Only the global member-name set survives
//! between models. No report is printed or overwritten until all models load.

mod fields;
mod model;
mod registry;
mod report;
mod stats;
mod totals;

pub use fields::fields;
pub use registry::{Error as RegistryError, Registry};

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;

const MAX_FILE_SIZE: u64 = 67_108_864;
const REPORT_PATH: &str = "catalog/aws-models.report";

/// Failures from model input, report persistence, comparison, or command syntax.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// A filesystem operation failed at the named path.
    #[error("{file}: {source}")]
    Io { file: PathBuf, source: io::Error },
    /// The file is too large to read.
    #[error("file exceeds {MAX_FILE_SIZE} bytes: {0}")]
    FileTooLarge(PathBuf),
    /// JSON syntax or required model fields are invalid.
    #[error("invalid JSON in {file}: {source}")]
    InvalidJson {
        file: PathBuf,
        source: serde_json::Error,
    },
    /// A file has no service shape.
    #[error("no service shape in {0}")]
    NoService(PathBuf),
    /// A file has more than one service shape.
    #[error("several service shapes in {0}")]
    SeveralServices(PathBuf),
    /// The models directory contains no JSON files.
    #[error("no models found")]
    NoModels,
    /// Every differing line, including whitespace and line terminators.
    #[error("{0}")]
    ReportMismatch(String),
    /// Formatting a canonical report or line comparison failed.
    #[error("report formatting failed: {0}")]
    Formatting(#[from] std::fmt::Error),
    /// The field-ID registry is invalid or inconsistent.
    #[error(transparent)]
    Registry(#[from] RegistryError),
    /// New names are not in the registry.
    #[error(
        "{new_names} member names are not in the registry, first: {first}; run `catalog-tool models fields --write`"
    )]
    RegistryStale {
        path: PathBuf,
        new_names: usize,
        first: String,
    },
    /// Another writer changed the registry after this run read it.
    #[error("registry changed during the run; rerun")]
    RegistryChanged { path: PathBuf },
    /// A temporary publication file already exists and must be removed by its owner.
    #[error("{path}: another run is writing or a previous run crashed; remove the temporary file")]
    WriteInProgress { path: PathBuf },
    /// Registry input exceeds the bounded reader's byte limit.
    #[error("{path}: registry exceeds {limit} bytes")]
    RegistryTooLarge { path: PathBuf, limit: u64 },
    /// Publication failed and removing its owned temporary file also failed.
    #[error("{original}; failed to remove temporary {path}: {cleanup}")]
    PublicationCleanup {
        #[source]
        original: Box<Error>,
        path: PathBuf,
        cleanup: io::Error,
    },
}

/// Optional persistence or verification after printing a complete report.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReportMode {
    /// Print without touching the committed report.
    #[default]
    Print,
    /// Overwrite or create the report.
    Write,
    /// Compare the complete report without normalizing whitespace.
    Check,
}

/// Load every model, print the report, then optionally write or check it.
///
/// # Errors
/// Returns a named input error, output I/O error, or all differing report lines.
pub fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    let text = build_report(source)?;
    write_stdout(&text)?;
    let path = root.join(REPORT_PATH);
    match mode {
        ReportMode::Print => Ok(()),
        ReportMode::Check => check_report(&path, &text),
        ReportMode::Write => {
            fs::create_dir_all(root.join("catalog")).map_err(|source| io_error(&path, source))?;
            fs::write(&path, text).map_err(|source| io_error(&path, source))
        }
    }
}

fn check_report(path: &Path, text: &str) -> Result<(), Error> {
    let existing = fs::read_to_string(path).map_err(|source| io_error(path, source))?;
    let differences = report::differences(text, &existing)?;
    if differences.is_empty() {
        Ok(())
    } else {
        Err(Error::ReportMismatch(differences))
    }
}

/// Read JSON files in ascending path order and render their complete totals.
///
/// # Errors
/// Returns a typed, filename-bearing error for invalid or oversized model files.
pub fn build_report(source: &Path) -> Result<String, Error> {
    let files = find_model_files(&source.join("models"))?;
    if files.is_empty() {
        return Err(Error::NoModels);
    }
    let totals = files.iter().try_fold(
        totals::Totals::default(),
        |mut totals, file| -> Result<_, Error> {
            totals.add_model(read_model_file(file)?, file)?;
            Ok(totals)
        },
    )?;
    Ok(report::render_report(&totals)?)
}

fn write_stdout(text: &str) -> Result<(), Error> {
    io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .map_err(|source| io_error(Path::new("<stdout>"), source))
}

fn read_model_file(file: &Path) -> Result<model::ModelFile, Error> {
    let metadata = fs::metadata(file).map_err(|source| io_error(file, source))?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(Error::FileTooLarge(file.to_path_buf()));
    }
    let bytes = fs::read(file).map_err(|source| io_error(file, source))?;
    serde_json::from_slice(&bytes).map_err(|source| Error::InvalidJson {
        file: file.to_path_buf(),
        source,
    })
}

fn find_model_files(models: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut pending = vec![models.to_path_buf()];
    let mut files = Vec::new();
    // Each real directory is removed once; symlinks are not followed.
    std::iter::from_fn(|| {
        pending
            .pop()
            .map(|dir| scan_directory(&dir, &mut pending, &mut files))
    })
    .try_for_each(std::convert::identity)?;
    files.sort();
    Ok(files)
}

fn scan_directory(
    dir: &Path,
    pending: &mut Vec<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    fs::read_dir(dir)
        .map_err(|source| io_error(dir, source))?
        .try_for_each(|entry| {
            let entry = entry.map_err(|source| io_error(dir, source))?;
            let path = entry.path();
            let kind = entry
                .file_type()
                .map_err(|source| io_error(&path, source))?;
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                files.push(path);
            }
            Ok(())
        })
}

fn io_error(file: &Path, source: io::Error) -> Error {
    Error::Io {
        file: file.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;
