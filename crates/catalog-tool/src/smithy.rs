mod compile;
mod fields;
mod model;
mod registry;
mod report;
mod stats;
mod totals;

pub use compile::{CompileError, build_catalog, compile};
pub use fields::fields;
pub use registry::{Error as RegistryError, Registry};
pub(crate) use report::differences as report_differences;
pub(crate) use stats::{READ_PREFIXES, no_required as input_has_no_required};

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;

const MAX_FILE_SIZE: u64 = 67_108_864;

const MAX_MODEL_ENTRIES: usize = 65_536;
const REPORT_PATH: &str = "catalog/aws-models.report";

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("{file}: {source}")]
    Io { file: PathBuf, source: io::Error },

    #[error("file exceeds {MAX_FILE_SIZE} bytes: {0}")]
    FileTooLarge(PathBuf),

    #[error("more than {limit} directory entries under {directory}")]
    TooManyEntries { directory: PathBuf, limit: usize },

    #[error("invalid JSON in {file}: {source}")]
    InvalidJson {
        file: PathBuf,
        source: serde_json::Error,
    },
    #[error("invalid Smithy model {file}: {message}")]
    InvalidModel { file: PathBuf, message: String },

    #[error("no service shape in {0}")]
    NoService(PathBuf),

    #[error("several service shapes in {0}")]
    SeveralServices(PathBuf),

    #[error("no models found")]
    NoModels,

    #[error(transparent)]
    Compile(#[from] compile::CompileError),

    #[error("{0}")]
    ReportMismatch(String),

    #[error("report formatting failed: {0}")]
    Formatting(#[from] std::fmt::Error),

    #[error(transparent)]
    Registry(#[from] RegistryError),

    #[error(
        "{new_names} member names are not in the registry, first: {first}; run `catalog-tool models fields --write`"
    )]
    RegistryStale {
        path: PathBuf,
        new_names: usize,
        first: String,
    },

    #[error("registry changed during the run; rerun")]
    RegistryChanged { path: PathBuf },

    #[error("{path}: another run is writing or a previous run crashed; remove the temporary file")]
    WriteInProgress { path: PathBuf },

    #[error("{path}: registry exceeds {limit} bytes")]
    RegistryTooLarge { path: PathBuf, limit: u64 },

    #[error("{original}; failed to remove temporary {path}: {cleanup}")]
    PublicationCleanup {
        #[source]
        original: Box<Error>,
        path: PathBuf,
        cleanup: io::Error,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReportMode {
    #[default]
    Print,

    Write,

    Check,
}

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
    let model = read_json(file)?;
    model::mixins::expand(model).map_err(|message| Error::InvalidModel {
        file: file.to_path_buf(),
        message,
    })
}

pub(crate) fn read_json<T: serde::de::DeserializeOwned>(file: &Path) -> Result<T, Error> {
    let bytes = read_bounded(file, MAX_FILE_SIZE)?;
    serde_json::from_slice(&bytes).map_err(|source| Error::InvalidJson {
        file: file.to_path_buf(),
        source,
    })
}

fn read_bounded(file: &Path, limit: u64) -> Result<Vec<u8>, Error> {
    let handle = fs::File::open(file).map_err(|source| io_error(file, source))?;
    let mut bytes = Vec::new();
    handle
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(file, source))?;
    let length = u64::try_from(bytes.len()).map_err(|_| Error::FileTooLarge(file.to_path_buf()))?;
    if length > limit {
        return Err(Error::FileTooLarge(file.to_path_buf()));
    }
    Ok(bytes)
}

pub(crate) fn find_model_files(models: &Path) -> Result<Vec<PathBuf>, Error> {
    find_model_files_within(models, MAX_MODEL_ENTRIES)
}

struct Walk {
    pending: Vec<PathBuf>,
    files: Vec<PathBuf>,
    remaining: usize,
    limit: usize,
}

fn find_model_files_within(models: &Path, limit: usize) -> Result<Vec<PathBuf>, Error> {
    let mut walk = Walk {
        pending: vec![models.to_path_buf()],
        files: Vec::new(),
        remaining: limit,
        limit,
    };

    std::iter::from_fn(|| {
        walk.pending
            .pop()
            .map(|dir| scan_directory(&dir, &mut walk))
    })
    .try_for_each(std::convert::identity)?;
    walk.files.sort();
    Ok(walk.files)
}

fn scan_directory(dir: &Path, walk: &mut Walk) -> Result<(), Error> {
    fs::read_dir(dir)
        .map_err(|source| io_error(dir, source))?
        .try_for_each(|entry| {
            let entry = entry.map_err(|source| io_error(dir, source))?;
            walk.remaining =
                walk.remaining
                    .checked_sub(1)
                    .ok_or_else(|| Error::TooManyEntries {
                        directory: dir.to_path_buf(),
                        limit: walk.limit,
                    })?;
            let path = entry.path();
            let kind = entry
                .file_type()
                .map_err(|source| io_error(&path, source))?;
            if kind.is_dir() {
                walk.pending.push(path);
            } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                walk.files.push(path);
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
mod bounds_tests;
#[cfg(test)]
mod tests;
