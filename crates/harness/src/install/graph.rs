use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::HarnessError;

mod archive;
mod build;
mod build_cleanup;
mod build_version;
mod download;
mod ownership;
mod receipt;

use ownership::{DIRECTORY_FLAGS, OwnedDirectory, descriptor_path};
#[cfg(test)]
mod installer_tests;
#[cfg(all(test, feature = "native-graph-integration"))]
mod native_tests;
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
    set_module_executable(&layout.module)?;
    let source = archive::extract(&layout.archive, &layout.source)?;
    let result = build::compile(&source, jobs)
        .map_err(receipt::PublicationError::Uncommitted)
        .and_then(|()| receipt::publish(layout, &source));
    finish_installation(source, result)
}

fn finish_installation(
    source: OwnedDirectory,
    result: Result<Value, receipt::PublicationError>,
) -> Result<Value, HarnessError> {
    match result {
        Ok(value) => Ok(value),
        Err(receipt::PublicationError::Committed(error)) => Err(error),
        Err(receipt::PublicationError::Uncommitted(error)) => {
            complete(Err(error), source.remove_tree())
        }
    }
}

fn require_platform() -> Result<(), HarnessError> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Ok(());
    }
    Err(HarnessError::invalid(
        "the pinned FalkorDB module requires Linux x86_64",
    ))
}

fn set_module_executable(path: &Path) -> Result<(), HarnessError> {
    let file = open_regular(path)?;
    if receipt::digest_file(&file)? != MODULE_SHA256 {
        return Err(HarnessError::invalid(
            "module changed before executable permission update",
        ));
    }
    make_executable(&file)
}

#[cfg(test)]
fn set_executable(path: &Path) -> Result<(), HarnessError> {
    make_executable(&open_regular(path)?)
}

fn make_executable(file: &File) -> Result<(), HarnessError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    if file.metadata()?.nlink() != 1 {
        return Err(HarnessError::invalid(
            "refusing executable permission change on aliased inode",
        ));
    }
    Ok(file.set_permissions(std::fs::Permissions::from_mode(0o755))?)
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

pub(super) fn executable(file: &File) -> Result<(), HarnessError> {
    match rustix::fs::accessat(
        rustix::fs::CWD,
        descriptor_path(file),
        rustix::fs::Access::EXEC_OK,
        rustix::fs::AtFlags::EACCESS,
    ) {
        Ok(()) => Ok(()),
        Err(rustix::io::Errno::ACCESS) => Err(HarnessError::invalid(
            "Redis binary has no executable permissions",
        )),
        Err(error) => Err(std::io::Error::from(error).into()),
    }
}
pub(super) fn regular_file(metadata: &std::fs::Metadata) -> Result<(), HarnessError> {
    if !metadata.is_file() || metadata.len() > MAX_DOWNLOAD {
        return Err(HarnessError::invalid(
            "installation file is not a bounded regular file",
        ));
    }
    Ok(())
}

pub(super) fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, HarnessError> {
    let bound = maximum
        .checked_add(1)
        .ok_or_else(|| HarnessError::invalid("file read limit overflow"))?;
    let mut bytes = Vec::new();
    open_regular(path)?.take(bound).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).map_err(|error| HarnessError::invalid(error.to_string()))?
        > maximum
    {
        return Err(HarnessError::invalid(
            "installation file exceeds read limit",
        ));
    }
    Ok(bytes)
}
fn open_regular(path: &Path) -> Result<File, HarnessError> {
    let flags = rustix::fs::OFlags::RDONLY
        | rustix::fs::OFlags::NOFOLLOW
        | rustix::fs::OFlags::NONBLOCK
        | rustix::fs::OFlags::CLOEXEC;
    let file = File::from(
        rustix::fs::open(path, flags, rustix::fs::Mode::empty()).map_err(std::io::Error::from)?,
    );
    regular_file(&file.metadata()?)?;
    Ok(file)
}
