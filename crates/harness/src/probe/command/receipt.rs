use super::ProbeError;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

pub(super) async fn publish(
    report: PathBuf,
    text: String,
    receipt: Option<String>,
) -> Result<(), ProbeError> {
    tokio::task::spawn_blocking(move || {
        refuse_symlink(&report)?;
        if let Some(contents) = receipt {
            let path = receipt_path(&report);
            refuse_symlink(&path).map_err(|source| ProbeError::Receipt {
                path: path.clone(),
                source,
            })?;
            let staged = stage(&report, &text)?;
            write(&path, &contents).map_err(|source| ProbeError::Receipt { path, source })?;
            staged.persist(&report).map_err(|error| error.error)?;
            print!("{contents}");
        } else {
            stage(&report, &text)?
                .persist(&report)
                .map_err(|error| error.error)?;
        }
        Ok(())
    })
    .await?
}

fn receipt_path(report: &Path) -> PathBuf {
    let mut receipt = report.as_os_str().to_os_string();
    receipt.push(".skip.toml");
    PathBuf::from(receipt)
}

fn refuse_symlink(path: &Path) -> io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("probe output is a symbolic link: {}", path.display()),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn stage(path: &Path, contents: &str) -> io::Result<tempfile::NamedTempFile> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or(Path::new("."), |parent| parent);
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(contents.as_bytes())?;
    Ok(temporary)
}

fn write(path: &Path, contents: &str) -> io::Result<()> {
    let temporary = stage(path, contents)?;
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        temporary.path(),
        rustix::fs::CWD,
        path,
        rustix::fs::RenameFlags::NOREPLACE,
    )?;
    Ok(())
}
