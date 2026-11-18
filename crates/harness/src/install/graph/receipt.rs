use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Component, Path};

use serde_json::{Value, json};
use sha2::Digest;

use super::{
    FALKOR_VERSION, Layout, MAX_DOWNLOAD, MODULE_SHA256, MODULE_URL, REDIS_SHA256, REDIS_URL,
    REDIS_VERSION, build, complete, executable, open_regular, read_bounded,
};
use crate::HarnessError;

const MAX_RECEIPT: u64 = 16_384;

#[derive(Debug, thiserror::Error)]
pub(super) enum PublicationError {
    #[error("{0}")]
    Uncommitted(HarnessError),
    #[error("{0}")]
    Committed(HarnessError),
}

pub(super) fn cached(layout: &Layout) -> Result<Option<Value>, HarnessError> {
    match std::fs::symlink_metadata(&layout.receipt) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
        Ok(_) => {}
    }
    regular(layout, &layout.receipt)?;
    let value = serde_json::from_slice(&read_bounded(&layout.receipt, MAX_RECEIPT)?)?;
    verify(layout, &value)?;
    Ok(Some(value))
}

fn verify(layout: &Layout, value: &Value) -> Result<(), HarnessError> {
    if value.get("schema_version").and_then(Value::as_u64) != Some(2) {
        return Err(HarnessError::invalid(
            "unsupported graph installation receipt",
        ));
    }
    verify_pins(value)?;
    verify_paths(layout, value)?;
    verify_downloads(layout)?;
    verify_binary(layout, value, &layout.server, "redis_server_sha256")?;
    verify_binary(layout, value, &layout.client, "redis_cli_sha256")?;
    let versions = value
        .get("versions")
        .ok_or_else(|| HarnessError::invalid("receipt has no Redis versions"))?;
    build::validate_versions(field(versions, "server")?, field(versions, "client")?)
}

pub(super) fn verify_pins(value: &Value) -> Result<(), HarnessError> {
    [
        ("redis_version", REDIS_VERSION),
        ("falkordb_version", FALKOR_VERSION),
        ("redis_url", REDIS_URL),
        ("module_url", MODULE_URL),
        ("redis_archive_sha256", REDIS_SHA256),
        ("module_sha256", MODULE_SHA256),
    ]
    .into_iter()
    .try_for_each(|(name, expected)| {
        if field(value, name)? == expected {
            return Ok(());
        }
        Err(HarnessError::invalid(format!(
            "receipt pin mismatch: {name}"
        )))
    })
}

pub(super) fn verify_paths(layout: &Layout, value: &Value) -> Result<(), HarnessError> {
    [
        ("redis_server", &layout.server),
        ("redis_cli", &layout.client),
        ("falkor_module", &layout.module),
    ]
    .into_iter()
    .try_for_each(|(name, path)| {
        if value.get(name) == Some(&serde_json::to_value(path)?) {
            return Ok(());
        }
        Err(HarnessError::invalid(format!(
            "receipt path mismatch: {name}"
        )))
    })
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, HarnessError> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid(format!("receipt has no string field: {name}")))
}

pub(super) fn verify_binary(
    layout: &Layout,
    value: &Value,
    path: &Path,
    name: &str,
) -> Result<(), HarnessError> {
    regular(layout, path)?;
    let file = open_regular(path)?;
    executable(&file)?;
    verify_binary_digest(value, &file, name)?;
    regular(layout, path)?;
    same_binary(&file, &open_regular(path)?)
}

fn verify_binary_digest(value: &Value, file: &File, name: &str) -> Result<(), HarnessError> {
    if field(value, name)? == digest_file(file)? {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "receipt binary digest mismatch: {name}"
    )))
}

fn verify_downloads(layout: &Layout) -> Result<(), HarnessError> {
    if digest(layout, &layout.archive)? != REDIS_SHA256
        || digest(layout, &layout.module)? != MODULE_SHA256
    {
        return Err(HarnessError::invalid(
            "installation download digest differs from pin",
        ));
    }
    Ok(())
}

pub(super) fn publish(
    layout: &Layout,
    source: &super::OwnedDirectory,
) -> Result<Value, PublicationError> {
    let value = create(layout, source).map_err(PublicationError::Uncommitted)?;
    verify_layout(layout, source).map_err(PublicationError::Uncommitted)?;
    store(source, &value)?;
    verify_layout(layout, source).map_err(PublicationError::Committed)?;
    Ok(value)
}

pub(super) fn store(source: &super::OwnedDirectory, value: &Value) -> Result<(), PublicationError> {
    let mut file = stage_file(source).map_err(PublicationError::Uncommitted)?;
    let result = write_receipt(source, &mut file, value);
    let cleanup = match &result {
        Ok(()) => rustix::fs::fsync(source.parent())
            .map_err(std::io::Error::from)
            .map_err(Into::into),
        Err(_) => Ok(()),
    };
    publication_result(result, cleanup)
}

pub(super) fn publication_result(
    result: Result<(), HarnessError>,
    cleanup: Result<(), HarnessError>,
) -> Result<(), PublicationError> {
    match result {
        Ok(()) => cleanup.map_err(PublicationError::Committed),
        Err(error) => complete(Err(error), cleanup).map_err(PublicationError::Uncommitted),
    }
}

pub(super) fn write_receipt(
    source: &super::OwnedDirectory,
    file: &mut File,
    value: &Value,
) -> Result<(), HarnessError> {
    write_receipt_observed(source, file, value, |_| {})
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublicationPhase {
    Before,
    After,
}

fn write_receipt_observed(
    source: &super::OwnedDirectory,
    file: &mut File,
    value: &Value,
    mut observe: impl FnMut(PublicationPhase),
) -> Result<(), HarnessError> {
    let bytes = serde_json::to_vec(value)?;
    if u64::try_from(bytes.len()).map_err(|error| HarnessError::invalid(error.to_string()))?
        > MAX_RECEIPT
    {
        return Err(HarnessError::invalid("graph receipt exceeds byte limit"));
    }
    file.write_all(&bytes)?;
    file.sync_all()?;
    source.verify()?;
    observe(PublicationPhase::Before);
    rustix::fs::linkat(
        rustix::fs::CWD,
        super::descriptor_path(file),
        source.parent(),
        "installation.json",
        rustix::fs::AtFlags::SYMLINK_FOLLOW,
    )
    .map_err(std::io::Error::from)?;
    observe(PublicationPhase::After);
    Ok(())
}

pub(super) fn stage_file(source: &super::OwnedDirectory) -> Result<File, HarnessError> {
    let flags =
        rustix::fs::OFlags::RDWR | rustix::fs::OFlags::TMPFILE | rustix::fs::OFlags::CLOEXEC;
    let mode = rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR;
    Ok(File::from(
        rustix::fs::openat(source.parent(), ".", flags, mode).map_err(std::io::Error::from)?,
    ))
}

fn create(layout: &Layout, source: &super::OwnedDirectory) -> Result<Value, HarnessError> {
    verify_layout(layout, source)?;
    verify_downloads(layout)?;
    let binaries = attest(source, build::version)?;
    Ok(json!({
        "schema_version": 2,
        "redis_version": REDIS_VERSION,
        "falkordb_version": FALKOR_VERSION,
        "redis_url": REDIS_URL,
        "redis_archive_sha256": REDIS_SHA256,
        "module_url": MODULE_URL,
        "module_sha256": MODULE_SHA256,
        "versions": {"server": binaries.server_version, "client": binaries.client_version},
        "redis_server": layout.server,
        "redis_cli": layout.client,
        "falkor_module": layout.module,
        "redis_server_sha256": binaries.server_sha256,
        "redis_cli_sha256": binaries.client_sha256,
    }))
}

fn verify_layout(layout: &Layout, source: &super::OwnedDirectory) -> Result<(), HarnessError> {
    use std::os::unix::fs::MetadataExt;
    let root = rustix::fs::open(
        &layout.root,
        super::DIRECTORY_FLAGS,
        rustix::fs::Mode::empty(),
    )
    .map_err(std::io::Error::from)?;
    let current_root = rustix::fs::fstat(&root).map_err(std::io::Error::from)?;
    let retained_root = rustix::fs::fstat(source.parent()).map_err(std::io::Error::from)?;
    if current_root.st_dev != retained_root.st_dev || current_root.st_ino != retained_root.st_ino {
        return Err(HarnessError::invalid(
            "graph installation root identity changed",
        ));
    }
    let current = rustix::fs::statat(
        &root,
        super::REDIS_ROOT,
        rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
    )
    .map_err(std::io::Error::from)?;
    let retained = std::fs::metadata(source.anchor())?;
    if current.st_dev != retained.dev() || current.st_ino != retained.ino() {
        return Err(HarnessError::invalid(
            "graph installation source identity changed",
        ));
    }
    source.verify()
}

struct BinaryAttestation {
    server_version: String,
    client_version: String,
    server_sha256: String,
    client_sha256: String,
}

fn attest(
    source: &super::OwnedDirectory,
    mut probe: impl FnMut(&File) -> Result<String, HarnessError>,
) -> Result<BinaryAttestation, HarnessError> {
    let server = source.open_binary("redis-server")?;
    let client = source.open_binary("redis-cli")?;
    let server_snapshot = snapshot(&server)?;
    let client_snapshot = snapshot(&client)?;
    let server_version = probe(&server_snapshot)?;
    let client_version = probe(&client_snapshot)?;
    build::validate_versions(&server_version, &client_version)?;
    let server_sha256 = digest_file(&server_snapshot)?;
    let client_sha256 = digest_file(&client_snapshot)?;
    source.verify()?;
    verify_source(
        &server,
        &source.open_binary("redis-server")?,
        &server_sha256,
    )?;
    verify_source(&client, &source.open_binary("redis-cli")?, &client_sha256)?;
    Ok(BinaryAttestation {
        server_version,
        client_version,
        server_sha256,
        client_sha256,
    })
}

fn snapshot(original: &File) -> Result<File, HarnessError> {
    let flags = rustix::fs::MemfdFlags::CLOEXEC
        | rustix::fs::MemfdFlags::ALLOW_SEALING
        | rustix::fs::MemfdFlags::EXEC;
    let mut sealed = File::from(
        rustix::fs::memfd_create("graph-binary-attestation", flags)
            .map_err(std::io::Error::from)?,
    );
    let bound = MAX_DOWNLOAD
        .checked_add(1)
        .ok_or_else(|| HarnessError::invalid("snapshot limit overflow"))?;
    let count = std::io::copy(&mut original.take(bound), &mut sealed)?;
    if count > MAX_DOWNLOAD {
        return Err(HarnessError::invalid(
            "installation snapshot exceeds byte limit",
        ));
    }
    let seals = rustix::fs::SealFlags::WRITE
        | rustix::fs::SealFlags::GROW
        | rustix::fs::SealFlags::SHRINK
        | rustix::fs::SealFlags::SEAL;
    rustix::fs::fcntl_add_seals(&sealed, seals).map_err(std::io::Error::from)?;
    sealed.rewind()?;
    Ok(sealed)
}

fn verify_source(original: &File, current: &File, expected: &str) -> Result<(), HarnessError> {
    same_binary(original, current)?;
    if digest_file(current)? == expected {
        return Ok(());
    }
    Err(HarnessError::invalid(
        "Redis binary bytes changed during attestation",
    ))
}

fn same_binary(attested: &File, current: &File) -> Result<(), HarnessError> {
    use std::os::unix::fs::MetadataExt;
    let attested = attested.metadata()?;
    let current = current.metadata()?;
    if attested.dev() == current.dev() && attested.ino() == current.ino() {
        return Ok(());
    }
    Err(HarnessError::invalid(
        "Redis binary identity changed during attestation",
    ))
}

pub(super) fn regular(layout: &Layout, path: &Path) -> Result<(), HarnessError> {
    let relative = path.strip_prefix(&layout.root).map_err(|error| {
        HarnessError::invalid(format!("installation path escapes root: {error}"))
    })?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(HarnessError::invalid("installation path is not contained"));
    }
    path.ancestors()
        .skip(1)
        .take_while(|parent| *parent != layout.root.as_path())
        .try_for_each(directory)?;
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_DOWNLOAD {
        return Err(HarnessError::invalid(
            "installation path is not a bounded regular file",
        ));
    }
    Ok(())
}

fn directory(path: &Path) -> Result<(), HarnessError> {
    if std::fs::symlink_metadata(path)?.file_type().is_dir() {
        return Ok(());
    }
    Err(HarnessError::invalid(
        "installation ancestor is not a regular directory",
    ))
}

pub(super) fn digest(layout: &Layout, path: &Path) -> Result<String, HarnessError> {
    regular(layout, path)?;
    digest_file(&open_regular(path)?)
}

pub(super) fn digest_file(file: &File) -> Result<String, HarnessError> {
    let mut reader = file;
    reader.rewind()?;
    let bound = MAX_DOWNLOAD
        .checked_add(1)
        .ok_or_else(|| HarnessError::invalid("digest limit overflow"))?;
    let mut writer = HashWriter(sha2::Sha256::new());
    let count = std::io::copy(&mut reader.take(bound), &mut writer)?;
    if count > MAX_DOWNLOAD {
        return Err(HarnessError::invalid(
            "installation digest exceeds byte limit",
        ));
    }
    Ok(format!("{:x}", base16ct::HexDisplay(&writer.0.finalize())))
}

struct HashWriter(sha2::Sha256);

impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "receipt_tests.rs"]
mod tests;
