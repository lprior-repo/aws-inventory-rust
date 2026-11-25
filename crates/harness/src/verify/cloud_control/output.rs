use super::{Args, Report};
use crate::HarnessError;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

pub(super) fn validate(args: &Args) -> Result<(), HarnessError> {
    let protected = [
        &args.catalog,
        &args.universal,
        &args.ca_bundle,
        &args.differences,
    ]
    .into_iter()
    .chain(args.oracle.iter())
    .map(|path| destination(path))
    .collect::<Result<Vec<_>, _>>()?;
    let database = if args.database.starts_with("sqlite:") {
        Some(
            sqlx::sqlite::SqliteConnectOptions::from_str(&args.database)?
                .get_filename()
                .canonicalize()?,
        )
    } else {
        None
    };
    let report = destination(&args.report)?;
    let seeds = destination(&args.seeds)?;
    if report.starts_with(&seeds) {
        return Err(HarnessError::invalid(
            "Cloud Control outputs must not replace the seeds directory or its contents",
        ));
    }
    if args.report.is_dir() {
        return Err(HarnessError::invalid(format!(
            "Cloud Control report path is a directory: {}",
            args.report.display()
        )));
    }
    if protected.contains(&report) || database.as_ref() == Some(&report) {
        return Err(HarnessError::invalid(
            "Cloud Control outputs must not replace inventory or input files",
        ));
    }
    Ok(())
}

fn destination(path: &Path) -> Result<PathBuf, HarnessError> {
    if path.try_exists()? {
        return Ok(path.canonicalize()?);
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or(Path::new("."), |parent| parent);
    let name = path
        .file_name()
        .ok_or_else(|| HarnessError::invalid("Cloud Control output needs a file name"))?;
    Ok(parent.canonicalize()?.join(name))
}

pub(super) async fn write(args: &Args, report: Report) -> Result<Report, HarnessError> {
    let report_path = args.report.clone();
    tokio::task::spawn_blocking(move || {
        let file = stage(&report_path, &report.text)?;
        file.persist(&report_path)
            .map_err(|error| HarnessError::Io(error.error))?;
        Ok(report)
    })
    .await
    .map_err(|error| HarnessError::invalid(format!("Cloud Control output task: {error}")))?
}

fn stage(path: &Path, text: &str) -> Result<tempfile::NamedTempFile, HarnessError> {
    use std::io::Write;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or(Path::new("."), |parent| parent);
    let mut temporary = temporary_file(parent)?;
    temporary.write_all(text.as_bytes())?;
    temporary.flush()?;
    Ok(temporary)
}

#[cfg(unix)]
fn temporary_file(parent: &Path) -> std::io::Result<tempfile::NamedTempFile> {
    use std::os::unix::fs::PermissionsExt;
    tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o666))
        .tempfile_in(parent)
}

#[cfg(not(unix))]
fn temporary_file(parent: &Path) -> std::io::Result<tempfile::NamedTempFile> {
    tempfile::Builder::new().tempfile_in(parent)
}

#[cfg(test)]
mod tests;
