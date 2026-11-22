use std::future::Future;
use std::path::Path;

use serde_json::Value;

use inventory_harness::HarnessError;
use inventory_harness::provenance::file_sha256;
use inventory_harness::snapshot;

use super::Fixture;

pub(super) fn create_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn write_text(path: &Path, text: &str) -> Result<String, Box<dyn std::error::Error>> {
    create_parent(path)?;
    std::fs::write(path, text.as_bytes())?;
    Ok(file_sha256(path)?)
}

pub fn write_json(path: &Path, value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    create_parent(path)?;
    let mut encoded = serde_json::to_string_pretty(value)?;
    encoded.push('\n');
    std::fs::write(path, encoded.as_bytes())?;
    Ok(file_sha256(path)?)
}

pub fn write_toml(path: &Path, value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    create_parent(path)?;
    let encoded = toml::to_string_pretty(value)?;
    std::fs::write(path, encoded.as_bytes())?;
    Ok(file_sha256(path)?)
}

pub(super) fn insert(
    target: &mut Value,
    key: &str,
    value: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    target
        .as_object_mut()
        .ok_or("execution is not an object")?
        .insert(key.to_owned(), value);
    Ok(())
}

pub fn save_summary(fixture: &Fixture) -> Result<(), Box<dyn std::error::Error>> {
    write_json(&fixture.evidence.join("summary.json"), &fixture.summary)?;
    Ok(())
}

pub fn free(fixture: &Fixture) -> impl Future<Output = Result<Value, HarnessError>> {
    let evidence = fixture.evidence.clone();
    let destination = fixture.destination.clone();
    let binary = fixture.binary.clone();
    let root = fixture.root.clone();
    async move { snapshot::freeze(&evidence, &destination, &binary, &root).await }
}
