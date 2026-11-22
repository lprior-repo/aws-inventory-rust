use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::HarnessError;
use crate::json::canonical;

pub const MAX_SOURCE_FILES: usize = 4_096;

/// 32 MiB of sources plus one embedded catalog at its 64 MiB container bound.
pub const MAX_SOURCE_BYTES: u64 = 96 * 1024 * 1024;

const MAX_CATALOG_ROWS: usize = 512;

pub fn file_sha256(path: &Path) -> Result<String, HarnessError> {
    let bytes = fs::read(path)?;
    Ok(sha256_hex(&bytes))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let digest = sha2::Sha256::digest(bytes);
    format!("{:x}", base16ct::HexDisplay(&digest))
}

pub fn canonical_sha256(value: &Value) -> String {
    sha256_hex(canonical(value, true).as_bytes())
}

fn resource_rows(merged: &mut Value) -> Result<&mut Vec<Value>, HarnessError> {
    merged
        .get_mut("resource_types")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| HarnessError::invalid("invalid runtime catalog identity"))
}

fn with_plan(child: &Value, plan: &Value) -> Value {
    let mut row = child.clone();
    if let Some(object) = row.as_object_mut() {
        object.insert("parent_workflow".to_owned(), plan.clone());
    }
    row
}

fn require_disjoint(
    root_rows: &BTreeMap<String, Value>,
    child_rows: &BTreeMap<String, Value>,
    plan_rows: &BTreeMap<String, Value>,
) -> Result<(), HarnessError> {
    if root_rows.keys().any(|name| child_rows.contains_key(name))
        || plan_rows.keys().ne(child_rows.keys())
    {
        return Err(HarnessError::invalid(
            "overlapping or missing parent workflow identity",
        ));
    }
    Ok(())
}

fn require_plans(plans: &[Value], root_rows: &BTreeMap<String, Value>) -> Result<(), HarnessError> {
    for plan in plans {
        let parent = plan
            .get("parent_type")
            .and_then(Value::as_str)
            .ok_or_else(|| HarnessError::invalid("parent runtime catalog plan identity"))?;
        let prerequisite = root_rows
            .iter()
            .find(|(name, _)| name.as_str() == parent)
            .map(|(_, row)| row)
            .ok_or_else(|| HarnessError::invalid("parent runtime catalog prerequisite"))?;
        if prerequisite.get("list_requires_model") != Some(&Value::Bool(false)) {
            return Err(HarnessError::invalid(
                "parent runtime catalog requires reviewed root prerequisites",
            ));
        }
    }
    Ok(())
}

fn require_children(children: &[Value]) -> Result<(), HarnessError> {
    if children
        .iter()
        .any(|row| row.get("list_requires_model") != Some(&Value::Bool(true)))
    {
        return Err(HarnessError::invalid(
            "parent runtime catalog requires reviewed root prerequisites",
        ));
    }
    Ok(())
}

fn parent_evidence(parent_catalog: &Value) -> Result<&Value, HarnessError> {
    parent_catalog
        .get("evidence")
        .filter(|value| value.is_object())
        .ok_or_else(|| HarnessError::invalid("parent runtime catalog evidence missing"))
}

pub fn expected_runtime_catalog(
    root_catalog: &Value,
    parent_catalog: &Value,
) -> Result<Value, HarnessError> {
    let (roots, root_rows) = named_rows(root_catalog, "resource_types")?;
    let (children, child_rows) = named_rows(parent_catalog, "resource_types")?;
    let (plans, plan_rows) = named_rows(parent_catalog, "parent_plans")?;
    validate_bounds(roots.len(), children.len(), plans.len())?;
    require_disjoint(&root_rows, &child_rows, &plan_rows)?;
    require_plans(&plans, &root_rows)?;
    require_children(&children)?;
    assemble(
        root_catalog,
        &children,
        &plan_rows,
        parent_evidence(parent_catalog)?,
    )
}

fn assemble(
    root_catalog: &Value,
    children: &[Value],
    plans: &BTreeMap<String, Value>,
    evidence: &Value,
) -> Result<Value, HarnessError> {
    let mut merged = root_catalog.clone();
    let target = resource_rows(&mut merged)?;
    for child in children {
        let name = child
            .get("type_name")
            .and_then(Value::as_str)
            .ok_or_else(|| HarnessError::invalid("invalid runtime catalog identity"))?;
        let plan = plans.get(name).ok_or_else(|| {
            HarnessError::invalid("overlapping or missing parent workflow identity")
        })?;
        target.push(with_plan(child, plan));
    }
    if let Some(object) = merged.as_object_mut() {
        object.insert("parent_evidence".to_owned(), evidence.clone());
    }
    Ok(merged)
}

fn row_name<'a>(row: &'a Value, field: &str) -> Result<&'a str, HarnessError> {
    row.get("type_name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| HarnessError::invalid(format!("invalid runtime catalog {field} identity")))
}

fn named_rows(
    value: &Value,
    field: &str,
) -> Result<(Vec<Value>, BTreeMap<String, Value>), HarnessError> {
    let rows = value
        .get(field)
        .and_then(Value::as_array)
        .filter(|rows| !rows.is_empty() && rows.len() <= MAX_CATALOG_ROWS)
        .ok_or_else(|| HarnessError::invalid(format!("invalid runtime catalog {field} bounds")))?;
    let mut named = BTreeMap::new();
    for row in rows {
        let name = row_name(row, field)?;
        if named.insert(name.to_owned(), row.clone()).is_some() {
            return Err(HarnessError::invalid(format!(
                "duplicate runtime catalog {field} identity"
            )));
        }
    }
    Ok((rows.clone(), named))
}

fn validate_bounds(roots: usize, children: usize, plans: usize) -> Result<(), HarnessError> {
    if roots.saturating_add(children) > MAX_CATALOG_ROWS || plans != children {
        return Err(HarnessError::invalid(
            "invalid root/parent runtime catalog bounds",
        ));
    }
    Ok(())
}

pub fn source_fingerprint(root: &Path) -> Result<Value, HarnessError> {
    let files = source_files(root)?;
    let mut total = 0_u64;
    let mut rows = Vec::with_capacity(files.len());
    for path in &files {
        total = total.saturating_add(fs::metadata(path)?.len());
        if total > MAX_SOURCE_BYTES {
            return Err(HarnessError::invalid(
                "fixture source fingerprint byte budget exceeded",
            ));
        }
        let relative = path.strip_prefix(root).map_err(|_| {
            HarnessError::invalid("fixture source fingerprint escaped the workspace root")
        })?;
        rows.push(json!({
            "file": slash_path(relative),
            "sha256": file_sha256(path)?,
        }));
    }
    let rows = Value::Array(rows);
    Ok(json!({
        "sha256": canonical_sha256(&rows),
        "files": rows,
    }))
}

fn crate_sources(crates: &Path, files: &mut BTreeSet<PathBuf>) -> Result<(), HarnessError> {
    if !crates.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(crates)? {
        let crate_dir = entry?.path();
        if !crate_dir.is_dir() {
            continue;
        }
        collect_extension(&crate_dir.join("src"), "rs", files)?;
        let manifest = crate_dir.join("Cargo.toml");
        if manifest.is_file() {
            files.insert(manifest);
        }
    }
    Ok(())
}

fn insert_existing(root: &Path, names: &[&str], files: &mut BTreeSet<PathBuf>) {
    for name in names {
        let path = root.join(name);
        if path.is_file() {
            files.insert(path);
        }
    }
}

const ROOT_FILES: [&str; 3] = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"];

const EMBEDDED_FILES: [&str; 7] = [
    ".cargo/config.toml",
    "catalog/aws-api.catalog",
    "catalog/cloudcontrol-capabilities.json",
    "catalog/cloudcontrol-universal.json",
    "catalog/cloudcontrol-resources.json",
    "catalog/cloudcontrol-parent-resources.toml",
    "catalog/cloudcontrol-v04-types.toml",
];

fn source_files(root: &Path) -> Result<Vec<PathBuf>, HarnessError> {
    let mut files = BTreeSet::new();
    collect_extension(&root.join("src"), "rs", &mut files)?;
    crate_sources(&root.join("crates"), &mut files)?;
    for name in ROOT_FILES {
        files.insert(root.join(name));
    }
    collect_extension(&root.join("crates/engine/sql"), "sql", &mut files)?;
    insert_existing(root, &EMBEDDED_FILES, &mut files);
    if files.is_empty() || files.len() > MAX_SOURCE_FILES {
        return Err(HarnessError::invalid(
            "fixture source fingerprint file budget exceeded",
        ));
    }
    Ok(files.into_iter().collect())
}

fn collect_extension(
    directory: &Path,
    extension: &str,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), HarnessError> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_extension(&path, extension, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.insert(path);
        }
    }
    Ok(())
}

fn slash_path(relative: &Path) -> String {
    let mut text = String::new();
    for component in relative.components() {
        if !text.is_empty() {
            text.push('/');
        }
        text.push_str(&component.as_os_str().to_string_lossy());
    }
    text
}
