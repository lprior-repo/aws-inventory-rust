use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path};

use serde_json::{Value, json};
use sha2::Digest;

use super::{
    FALKOR_VERSION, Layout, MAX_DOWNLOAD, MODULE_SHA256, MODULE_URL, REDIS_SHA256, REDIS_URL,
    REDIS_VERSION, build, complete,
};
use crate::HarnessError;

const MAX_RECEIPT: u64 = 16_384;

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
    if field(value, name)? == digest(layout, path)? {
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

pub(super) fn publish(layout: &Layout) -> Result<Value, HarnessError> {
    let value = create(layout)?;
    store(layout, &value)?;
    Ok(value)
}

pub(super) fn store(layout: &Layout, value: &Value) -> Result<(), HarnessError> {
    let path = layout.lock.join("installation.json");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    complete(
        write_receipt(layout, &path, &mut file, value),
        std::fs::remove_file(path).map_err(Into::into),
    )
}

fn write_receipt(
    layout: &Layout,
    temporary: &Path,
    file: &mut File,
    value: &Value,
) -> Result<(), HarnessError> {
    let bytes = serde_json::to_vec(value)?;
    if u64::try_from(bytes.len()).map_err(|error| HarnessError::invalid(error.to_string()))?
        > MAX_RECEIPT
    {
        return Err(HarnessError::invalid("graph receipt exceeds byte limit"));
    }
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(std::fs::hard_link(temporary, &layout.receipt)?)
}

fn create(layout: &Layout) -> Result<Value, HarnessError> {
    verify_downloads(layout)?;
    let (server, client) = build::versions(layout)?;
    Ok(json!({
        "schema_version": 2,
        "redis_version": REDIS_VERSION,
        "falkordb_version": FALKOR_VERSION,
        "redis_url": REDIS_URL,
        "redis_archive_sha256": REDIS_SHA256,
        "module_url": MODULE_URL,
        "module_sha256": MODULE_SHA256,
        "versions": {"server": server, "client": client},
        "redis_server": layout.server,
        "redis_cli": layout.client,
        "falkor_module": layout.module,
        "redis_server_sha256": digest(layout, &layout.server)?,
        "redis_cli_sha256": digest(layout, &layout.client)?,
    }))
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

pub(super) fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, HarnessError> {
    let bound = maximum
        .checked_add(1)
        .ok_or_else(|| HarnessError::invalid("file read limit overflow"))?;
    let mut bytes = Vec::new();
    File::open(path)?.take(bound).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).map_err(|error| HarnessError::invalid(error.to_string()))?
        > maximum
    {
        return Err(HarnessError::invalid(
            "installation file exceeds read limit",
        ));
    }
    Ok(bytes)
}

pub(super) fn digest(layout: &Layout, path: &Path) -> Result<String, HarnessError> {
    regular(layout, path)?;
    let bound = MAX_DOWNLOAD
        .checked_add(1)
        .ok_or_else(|| HarnessError::invalid("digest limit overflow"))?;
    let mut writer = HashWriter(sha2::Sha256::new());
    let count = std::io::copy(&mut File::open(path)?.take(bound), &mut writer)?;
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
