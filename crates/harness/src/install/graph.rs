use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::HarnessError;

mod archive;
mod build;
mod download;
#[cfg(test)]
mod installer_tests;
mod receipt;
#[cfg(test)]
mod tests;

pub const DEFAULT_JOBS: usize = 2;

const MAX_JOBS: usize = 8;
const REDIS_VERSION: &str = "8.10.3";
const REDIS_ROOT: &str = "redis-8.10.3";
const FALKOR_VERSION: &str = "6.0.4";
const REDIS_URL: &str = "https://download.redis.io/releases/redis-8.10.3.tar.gz";
const MODULE_URL: &str =
    "https://github.com/FalkorDB/FalkorDB/releases/download/v6.0.4/falkordb-x64.so";
const REDIS_SHA256: &str = "edde778f6bdee7581e42fc139ca9e07e2822212092f7501f99989656e3629d02";
const MODULE_SHA256: &str = "0e71f1b4add13ab1831dc68dc03fdb715e0e813e23216163cc5ab37b0ae3f1f2";
const MAX_DOWNLOAD: u64 = 128 * 1024 * 1024;
const MAX_EXTRACT: u64 = 256 * 1024 * 1024;
const MAX_MEMBERS: usize = 20_000;
const EXPECTED_LINKS: [(&str, &str); 2] = [
    (
        "redis-8.10.3/modules/redisearch/src/.claude/skills",
        "../.skills",
    ),
    ("redis-8.10.3/modules/redisearch/src/CLAUDE.md", "AGENTS.md"),
];

pub fn valid_jobs(jobs: usize) -> bool {
    (1..=MAX_JOBS).contains(&jobs)
}

struct Layout {
    root: PathBuf,
    archive: PathBuf,
    module: PathBuf,
    source: PathBuf,
    server: PathBuf,
    client: PathBuf,
    receipt: PathBuf,
    lock: PathBuf,
}

impl Layout {
    fn new(root: &Path) -> Self {
        let source = root.join(REDIS_ROOT);
        Self {
            root: root.to_owned(),
            archive: root.join(format!("{REDIS_ROOT}.tar.gz")),
            module: root.join("falkordb.so"),
            server: source.join("src/redis-server"),
            client: source.join("src/redis-cli"),
            receipt: root.join("installation.json"),
            lock: root.join(".graph-install.lock"),
            source,
        }
    }
}

pub fn install(destination: &Path, jobs: usize) -> Result<Value, HarnessError> {
    require_platform()?;
    if !valid_jobs(jobs) {
        return Err(HarnessError::invalid("build jobs must be 1..8"));
    }
    std::fs::create_dir_all(destination)?;
    let layout = Layout::new(&std::fs::canonicalize(destination)?);
    let lock = OwnedDirectory::claim(&layout.lock)?;
    complete(install_locked(&layout, jobs), lock.remove_empty())
}

fn install_locked(layout: &Layout, jobs: usize) -> Result<Value, HarnessError> {
    if let Some(cached) = receipt::cached(layout)? {
        return Ok(cached);
    }
    require_absent(&layout.source)?;
    download::pinned(REDIS_URL, &layout.archive, REDIS_SHA256)?;
    download::pinned(MODULE_URL, &layout.module, MODULE_SHA256)?;
    receipt::regular(layout, &layout.archive)?;
    receipt::regular(layout, &layout.module)?;
    set_executable(&layout.module)?;
    archive::extract(&layout.archive, &layout.source)?;
    let source = OwnedDirectory::capture(&layout.source)?;
    if let Err(error) = build::compile(layout, jobs) {
        return complete(Err(error), source.remove_tree());
    }
    receipt::publish(layout)
}

fn require_platform() -> Result<(), HarnessError> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Ok(());
    }
    Err(HarnessError::invalid(
        "the pinned FalkorDB module requires Linux x86_64",
    ))
}

fn set_executable(path: &Path) -> Result<(), HarnessError> {
    use std::os::unix::fs::PermissionsExt;
    Ok(std::fs::set_permissions(
        path,
        std::fs::Permissions::from_mode(0o755),
    )?)
}

fn require_absent(path: &Path) -> Result<(), HarnessError> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
        Ok(_) => Err(HarnessError::invalid(format!(
            "unreceipted or concurrent installation path already exists: {}",
            path.display()
        ))),
    }
}

fn complete<T>(
    result: Result<T, HarnessError>,
    cleanup: Result<(), HarnessError>,
) -> Result<T, HarnessError> {
    match (result, cleanup) {
        (result, Ok(())) => result,
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(original), Err(cleanup)) => Err(HarnessError::invalid(format!(
            "{original}; cleanup failed: {cleanup}"
        ))),
    }
}

struct OwnedDirectory {
    path: PathBuf,
    identity: std::fs::Metadata,
}

impl OwnedDirectory {
    fn claim(path: &Path) -> Result<Self, HarnessError> {
        std::fs::create_dir(path)?;
        Self::capture(path)
    }

    fn capture(path: &Path) -> Result<Self, HarnessError> {
        let identity = std::fs::symlink_metadata(path)?;
        if !identity.file_type().is_dir() {
            return Err(HarnessError::invalid("owned directory is not a directory"));
        }
        Ok(Self {
            path: path.to_owned(),
            identity,
        })
    }

    fn verify(&self) -> Result<(), HarnessError> {
        use std::os::unix::fs::MetadataExt;
        let current = std::fs::symlink_metadata(&self.path)?;
        if current.file_type().is_dir()
            && current.dev() == self.identity.dev()
            && current.ino() == self.identity.ino()
        {
            return Ok(());
        }
        Err(HarnessError::invalid("owned directory identity changed"))
    }

    fn remove_empty(self) -> Result<(), HarnessError> {
        self.verify()?;
        Ok(std::fs::remove_dir(self.path)?)
    }

    fn remove_tree(self) -> Result<(), HarnessError> {
        self.verify()?;
        Ok(std::fs::remove_dir_all(self.path)?)
    }
}
