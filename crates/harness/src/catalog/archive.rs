//! Pinned schema archive access and reviewed-addition manifest loading.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::Path;

use serde_json::Value;
use zip::ZipArchive;

use crate::HarnessError;
use crate::catalog::json::Json;
use crate::catalog::{ADDITIONS, ARCHIVE, ARCHIVE_SHA256};
use crate::provenance::file_sha256;

/// One retained provider schema and its exact archive bytes.
pub struct Entry {
    /// Archive member name.
    pub file: String,
    /// Original member bytes.
    pub raw: Vec<u8>,
    /// Parsed schema.
    pub schema: Value,
}

/// Every retained schema, keyed by CloudFormation type name.
pub struct Archive {
    /// Schemas ordered by type name, as Python's sorted schema map.
    pub entries: BTreeMap<String, Entry>,
    /// Count of JSON members in the archive.
    pub all_count: usize,
    /// Count of members declaring both read and list handlers.
    pub eligible_count: usize,
}

impl Archive {
    /// Borrow one retained schema by type name.
    pub fn get(&self, name: &str) -> Option<&Entry> {
        self.entries.get(name)
    }

    /// Borrow one retained schema by archive member name.
    pub fn by_file(&self, file: &str) -> Option<&Entry> {
        self.entries.values().find(|entry| entry.file == file)
    }
}

/// Verify the pinned archive and load every JSON member.
pub fn load(root: &Path) -> Result<Archive, HarnessError> {
    let mut archive = open_archive(&root.join(ARCHIVE))?;
    let mut entries = BTreeMap::new();
    let mut all_count = 0_usize;
    let mut eligible_count = 0_usize;
    for index in 0..archive.len() {
        let Some((name, entry)) = read_member(&mut archive, index)? else {
            continue;
        };
        all_count = all_count.saturating_add(1);
        if eligible(&entry.schema) {
            eligible_count = eligible_count.saturating_add(1);
        }
        entries.insert(name, entry);
    }
    Ok(Archive {
        entries,
        all_count,
        eligible_count,
    })
}

fn open_archive(path: &Path) -> Result<ZipArchive<Cursor<Vec<u8>>>, HarnessError> {
    if file_sha256(path)? != ARCHIVE_SHA256 {
        return Err(HarnessError::invalid("upstream archive checksum mismatch"));
    }
    let raw = std::fs::read(path)?;
    ZipArchive::new(Cursor::new(raw))
        .map_err(|error| HarnessError::invalid(format!("schema archive: {error}")))
}

fn read_member(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    index: usize,
) -> Result<Option<(String, Entry)>, HarnessError> {
    let mut member = archive
        .by_index(index)
        .map_err(|error| HarnessError::invalid(format!("schema archive: {error}")))?;
    let file = member
        .name()
        .map_err(|error| HarnessError::invalid(format!("schema archive filename: {error}")))?
        .into_owned();
    if !file.ends_with(".json") {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    member.read_to_end(&mut bytes)?;
    let schema: Value = serde_json::from_slice(&bytes)?;
    let name = schema_name(&schema, &file)?;
    let entry = Entry {
        file,
        raw: bytes,
        schema,
    };
    Ok(Some((name, entry)))
}

fn schema_name(schema: &Value, file: &str) -> Result<String, HarnessError> {
    schema
        .get("typeName")
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid(format!("schema lacks typeName: {file}")))
        .map(str::to_string)
}

fn eligible(schema: &Value) -> bool {
    handlers(schema).is_some_and(|keys| {
        keys.iter().any(|key| key == "read") && keys.iter().any(|key| key == "list")
    })
}

/// Handler names declared by a schema.
pub fn handlers(schema: &Value) -> Option<Vec<String>> {
    schema
        .get("handlers")
        .and_then(Value::as_object)
        .map(|object| object.keys().cloned().collect())
}

/// Load the reviewed regional additions in manifest order.
pub fn additions(root: &Path) -> Result<Vec<(String, String)>, HarnessError> {
    let text = std::fs::read_to_string(root.join(ADDITIONS))?;
    let manifest = Json::parse(&text)?;
    let families = reviewed_families(&manifest)?;
    let mut total = 0_usize;
    let mut output = Vec::new();
    for (service, value) in families {
        let names = family_names(value)?;
        total = total.saturating_add(names.split_whitespace().count());
        output.push((service.clone(), names.to_string()));
    }
    if total != 197 {
        return Err(HarnessError::invalid("reviewed addition count changed"));
    }
    Ok(output)
}

fn reviewed_families(manifest: &Json) -> Result<&[(String, Json)], HarnessError> {
    let version_ok = matches!(
        manifest.get("schema_version"),
        Some(Json::Number(raw)) if raw == "1"
    );
    match manifest.get("resource_families") {
        Some(Json::Object(families)) if version_ok => Ok(families),
        _ => Err(HarnessError::invalid(
            "unsupported reviewed-addition manifest",
        )),
    }
}

fn family_names(value: &Json) -> Result<&str, HarnessError> {
    value
        .as_str()
        .ok_or_else(|| HarnessError::invalid("unsupported reviewed-addition manifest"))
}
