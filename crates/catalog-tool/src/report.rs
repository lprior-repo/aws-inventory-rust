use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use thiserror::Error;

const MAX_REPORT_BYTES: usize = 4_194_304;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReportMode {
    #[default]
    Print,
    Write,
    Check,
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("{file}: {source}")]
    Io { file: PathBuf, source: io::Error },
    #[error("{file}: comparison file exceeds {limit} bytes")]
    TooLarge { file: PathBuf, limit: usize },
    #[error("{0}")]
    Mismatch(String),
    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

pub(crate) fn publish(path: &Path, text: &str, mode: ReportMode) -> Result<(), ReportError> {
    match mode {
        ReportMode::Print => write_stdout(text),
        ReportMode::Check => check(path, text),
        ReportMode::Write => write_report(path, text),
    }
}

fn check(path: &Path, expected: &str) -> Result<(), ReportError> {
    let actual = read_text(path)?;
    let differences = differences(expected, &actual)?;
    if differences.is_empty() {
        Ok(())
    } else {
        Err(ReportError::Mismatch(differences))
    }
}

fn write_report(path: &Path, text: &str) -> Result<(), ReportError> {
    crate::package::atomic_write(path, |file| -> io::Result<()> {
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        Ok(())
    })
    .map_err(|source| io_error(path, source))
}

pub(crate) fn read_text(path: &Path) -> Result<String, ReportError> {
    let file = fs::File::open(path).map_err(|source| io_error(path, source))?;
    let mut bytes = Vec::new();
    let read_limit = u64::try_from(MAX_REPORT_BYTES)
        .map_err(|source| io_error(path, io::Error::new(io::ErrorKind::InvalidInput, source)))?;
    file.take(read_limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(path, source))?;
    if bytes.len() > MAX_REPORT_BYTES {
        return Err(ReportError::TooLarge {
            file: path.to_path_buf(),
            limit: MAX_REPORT_BYTES,
        });
    }
    String::from_utf8(bytes)
        .map_err(|source| io_error(path, io::Error::new(io::ErrorKind::InvalidData, source)))
}

pub(crate) fn write_stdout(text: &str) -> Result<(), ReportError> {
    io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .map_err(|source| io_error(Path::new("<stdout>"), source))
}

fn io_error(file: &Path, source: io::Error) -> ReportError {
    ReportError::Io {
        file: file.to_path_buf(),
        source,
    }
}

pub(crate) fn differences(expected: &str, actual: &str) -> Result<String, std::fmt::Error> {
    use std::fmt::Write as _;
    let mut expected = expected.split_inclusive('\n');
    let mut actual = actual.split_inclusive('\n');
    std::iter::from_fn(|| match (expected.next(), actual.next()) {
        (None, None) => None,
        pair => Some(pair),
    })
    .enumerate()
    .try_fold(String::new(), |mut text, (index, (left, right))| {
        if left != right {
            writeln!(
                text,
                "line {} differs: expected {left:?}, got {right:?}",
                index.saturating_add(1)
            )?;
        }
        Ok(text)
    })
}
