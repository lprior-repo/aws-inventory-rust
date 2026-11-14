use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use super::{Error, ReportMode, build_report};

const REPORT_PATH: &str = "catalog/aws-endpoints.report";

/// Print a complete report, then optionally check exact bytes or overwrite the report.
///
/// # Errors
/// Returns input errors before printing/writing, or named stdout/report I/O failures.
pub fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    let text = build_report(source)?;
    io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .map_err(|source| io_error(Path::new("<stdout>"), source))?;
    persist(root, &text, mode)
}

fn persist(root: &Path, text: &str, mode: ReportMode) -> Result<(), Error> {
    let path = root.join(REPORT_PATH);
    match mode {
        ReportMode::Print => Ok(()),
        ReportMode::Check => check_report(&path, text),
        ReportMode::Write => {
            fs::create_dir_all(root.join("catalog")).map_err(|e| io_error(&path, e))?;
            fs::write(&path, text).map_err(|e| io_error(&path, e))
        }
    }
}

fn check_report(path: &Path, text: &str) -> Result<(), Error> {
    let existing = fs::read(path).map_err(|e| io_error(path, e))?;
    if existing == text.as_bytes() {
        Ok(())
    } else {
        Err(Error::ReportMismatch(path.to_path_buf()))
    }
}

fn io_error(path: &Path, source: io::Error) -> Error {
    Error::Io {
        file: path.to_path_buf(),
        source,
    }
}
