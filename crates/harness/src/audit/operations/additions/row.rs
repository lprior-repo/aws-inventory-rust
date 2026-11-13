//! Structural validation of one reviewed addition.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::HarnessError;

use super::super::super::files::{array_field, object_field, text_field};
use super::{AdditionRow, AdditionSite};

const STATUSES: [&str; 2] = ["unexercised", "pending_execution"];
const SOURCE_PREFIX: &str = "crates/aws/src/";

/// Lowercase hexadecimal SHA-256 text.
pub(super) fn hex64(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid() -> HarnessError {
    HarnessError::invalid("invalid reviewed additions document")
}

fn outcome(id: &str) -> HarnessError {
    HarnessError::invalid(format!(
        "reviewed addition {id} must not claim an emulator outcome"
    ))
}

/// Parse one addition row; every failure is fail-closed.
pub(super) fn parse(value: &Value) -> Result<AdditionRow, HarnessError> {
    if value.get("emulator_coverage").is_some() {
        return Err(outcome(text_field(value, "id")?));
    }
    let (id, service, module, crate_name, version, api_operation) = identity(value)?;
    let sites = sites(value, id)?;
    let source_files = sources(value, &sites)?;
    let module_sha256 = wire(value, id, service, module)?;
    let status = coverage(value, id)?;
    Ok(AdditionRow {
        id: id.to_owned(),
        key: (crate_name.to_owned(), version.to_owned(), module.to_owned()),
        service: service.to_owned(),
        crate_name: crate_name.to_owned(),
        module: module.to_owned(),
        api_operation: api_operation.to_owned(),
        status,
        module_sha256,
        sites,
        source_files,
    })
}

fn identity(value: &Value) -> Result<(&str, &str, &str, &str, &str, &str), HarnessError> {
    let id = text_field(value, "id")?;
    let service = text_field(value, "sdk_service")?;
    let crate_name = text_field(value, "sdk_crate")?;
    let version = text_field(value, "sdk_version")?;
    let module = text_field(value, "sdk_operation")?;
    let api_operation = text_field(value, "api_operation")?;
    let expected = format!("{service}.{module}");
    let explicit = value.get("explicit_sdk_send").and_then(Value::as_bool);
    let boto3 = value
        .get("boto3_service")
        .and_then(Value::as_str)
        .is_some_and(|name| !name.is_empty());
    if !boto3 || explicit != Some(true) || id != expected {
        return Err(HarnessError::invalid(format!(
            "reviewed addition {id} disagrees with its crate/module"
        )));
    }
    Ok((id, service, module, crate_name, version, api_operation))
}

fn sites(value: &Value, id: &str) -> Result<Vec<AdditionSite>, HarnessError> {
    let rows = array_field(value, "source_calls")?;
    if rows.is_empty() {
        return Err(invalid());
    }
    let mut sites: Vec<AdditionSite> = Vec::new();
    for row in rows {
        let site = site(row, id)?;
        if sites
            .iter()
            .any(|known| known.path == site.path && known.function == site.function)
        {
            return Err(invalid());
        }
        sites.push(site);
    }
    Ok(sites)
}

fn site(value: &Value, id: &str) -> Result<AdditionSite, HarnessError> {
    let path = text_field(value, "path")?;
    if !source_path(path) {
        return Err(invalid());
    }
    let builder_line = line(value, "builder_line", id)?;
    let operation_line = line(value, "operation_line", id)?;
    let send_line = line(value, "send_line", id)?;
    if builder_line > operation_line || operation_line > send_line {
        return Err(lines(id));
    }
    Ok(AdditionSite {
        path: path.to_owned(),
        function: text_field(value, "function")?.to_owned(),
        builder_line,
        operation_line,
        send_line,
    })
}

fn line(value: &Value, key: &str, id: &str) -> Result<u64, HarnessError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .filter(|number| *number > 0)
        .ok_or_else(|| lines(id))
}

fn lines(id: &str) -> HarnessError {
    HarnessError::invalid(format!("invalid reviewed callsite lines for {id}"))
}

fn source_path(path: &str) -> bool {
    path.starts_with(SOURCE_PREFIX)
        && !path
            .split('/')
            .any(|part| part.is_empty() || part == ".." || part == ".")
}

fn sources(
    value: &Value,
    sites: &[AdditionSite],
) -> Result<BTreeMap<String, String>, HarnessError> {
    let rows = object_field(value, "source_sha256")?;
    let paths: std::collections::BTreeSet<&str> =
        sites.iter().map(|site| site.path.as_str()).collect();
    if rows.len() != paths.len() {
        return Err(invalid());
    }
    let mut sources: BTreeMap<String, String> = BTreeMap::new();
    for (path, digest) in rows {
        let digest = digest.as_str().ok_or_else(invalid)?;
        if !source_path(path) || !hex64(digest) || !paths.contains(path.as_str()) {
            return Err(invalid());
        }
        sources.insert(path.clone(), digest.to_owned());
    }
    Ok(sources)
}

fn wire(value: &Value, id: &str, service: &str, module: &str) -> Result<String, HarnessError> {
    let wire = object_field(value, "wire_source")?;
    let module_path = format!("src/operation/{module}.rs");
    let module_sha256 = wire
        .get("rust_operation_module_sha256")
        .and_then(Value::as_str)
        .ok_or_else(invalid)?;
    let model_sha256 = wire
        .get("operation_model_sha256")
        .and_then(Value::as_str)
        .ok_or_else(invalid)?;
    if wire.get("service").and_then(Value::as_str) != Some(service)
        || wire.get("rust_operation_module").and_then(Value::as_str) != Some(module_path.as_str())
        || !hex64(module_sha256)
        || !hex64(model_sha256)
    {
        return Err(HarnessError::invalid(format!(
            "pinned SDK operation module changed: {id}"
        )));
    }
    Ok(module_sha256.to_owned())
}

fn coverage(value: &Value, id: &str) -> Result<String, HarnessError> {
    let coverage = object_field(value, "coverage")?;
    let status = coverage
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(invalid)?;
    if !STATUSES.contains(&status) {
        return Err(outcome(id));
    }
    let receipt = coverage.get("execution_receipt").ok_or_else(invalid)?;
    let reason = coverage.get("reason").and_then(Value::as_str);
    if !receipt.is_null() || reason.is_none_or(str::is_empty) {
        return Err(outcome(id));
    }
    Ok(status.to_owned())
}
