use std::fmt::Write;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::report::{ReportError, ReportMode};

const MODELS_LOCK_PATH: &str = "models.lock";
const MANIFEST_PATH: &str = "MANIFEST.sha256";
/// Catalog outputs that scripts/bootstrap-catalog.sh regenerates. They are gitignored,
/// so `git ls-files` does not list them, but the manifest still pins their digests.
const GENERATED_CATALOG_FILES: &[&str] = &[
    "catalog/aws-api.catalog",
    "catalog/aws-models.report",
    "catalog/aws-endpoints.report",
    "catalog/aws-classes.report",
    "catalog/aws-plans.report",
    "catalog/aws-overlay.report",
    "catalog/coverage-denominator.report",
    "catalog/cloudcontrol-parent-candidates.json",
];

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Report(#[from] ReportError),

    #[error("{file}: {source}")]
    Io { file: PathBuf, source: io::Error },

    #[error("git ls-tree failed for .local/aws-models: {source}")]
    GitModelsRevision { source: io::Error },

    #[error("generated catalog file {file} is missing; run scripts/bootstrap-catalog.sh")]
    MissingGenerated { file: PathBuf },

    #[error("no catalog files found")]
    NoCatalogFiles,
}

pub fn run(root: &Path, mode: ReportMode) -> Result<(), Error> {
    let models_lock_path = root.join(MODELS_LOCK_PATH);
    let manifest_path = root.join(MANIFEST_PATH);

    let aws_models_rev = aws_models_revision(root)?;
    let models_lock_content =
        format!("# AWS models source revision\naws-models: {aws_models_rev}\n");

    crate::report::publish(&models_lock_path, &models_lock_content, mode)?;

    let catalog_files = tracked_catalog_files(root)?;
    let manifest_content = manifest_content(&catalog_files)?;
    crate::report::publish(&manifest_path, &manifest_content, mode)?;

    Ok(())
}

fn aws_models_revision(root: &Path) -> Result<String, Error> {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(["ls-tree", "HEAD", ".local/aws-models"])
        .output()
        .map_err(|source| Error::GitModelsRevision { source })?;

    if !output.status.success() {
        return Err(Error::GitModelsRevision {
            source: io::Error::other("git ls-tree returned non-zero"),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Expected: "160000 commit <sha>\t.local/aws-models\n"
    let fields: Vec<&str> = stdout.split_whitespace().collect();
    let sha = fields.get(2).ok_or_else(|| Error::GitModelsRevision {
        source: io::Error::other("unexpected git output"),
    })?;

    Ok(sha.to_string())
}

fn tracked_catalog_files(root: &Path) -> Result<Vec<(PathBuf, String)>, Error> {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(["ls-files", "catalog/"])
        .output()
        .map_err(|source| Error::Io {
            file: root.to_path_buf(),
            source,
        })?;

    if !output.status.success() {
        return Err(Error::Io {
            file: root.to_path_buf(),
            source: io::Error::other("git ls-files returned non-zero"),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| (root.join(line), line.to_string()))
        .collect::<Vec<(PathBuf, String)>>();

    for relative in GENERATED_CATALOG_FILES {
        let path = root.join(relative);
        if !path.is_file() {
            return Err(Error::MissingGenerated { file: path });
        }
        files.push((path, (*relative).to_owned()));
    }

    if files.is_empty() {
        return Err(Error::NoCatalogFiles);
    }
    files.sort_by(|left, right| left.1.cmp(&right.1));

    Ok(files)
}

fn manifest_content(files: &[(PathBuf, String)]) -> Result<String, Error> {
    let mut content = String::new();
    content.push_str("# SHA256 hashes of tracked catalog files\n");

    for (full_path, relative) in files {
        let bytes = fs::read(full_path).map_err(|source| Error::Io {
            file: full_path.to_path_buf(),
            source,
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let digest = hasher.finalize();

        let hash_bytes = digest.as_slice();
        let mut hash = String::with_capacity(64);
        for byte in hash_bytes {
            write!(&mut hash, "{byte:02x}").map_err(ReportError::Formatting)?;
        }

        content.push_str(&format!("{hash}  {relative}\n"));
    }

    Ok(content)
}
