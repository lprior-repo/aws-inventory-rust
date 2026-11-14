use std::io::Read;

use serde_json::Value;
use zip::ZipArchive;

pub(super) type Archive = ZipArchive<std::io::Cursor<Vec<u8>>>;

pub(super) fn schema_required(
    archive: &mut Archive,
    filename: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut file = match archive.by_name(filename) {
        Ok(file) => file,
        Err(_) => return Ok(Vec::new()),
    };
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;
    let schema: Value = serde_json::from_str(&raw)?;
    Ok(required_names(&schema))
}

fn required_names(schema: &Value) -> Vec<String> {
    let mut required = Vec::new();
    if let Some(values) = schema
        .pointer("/handlers/list/handlerSchema/required")
        .and_then(Value::as_array)
    {
        push_names(values, &mut required);
    }
    if let Some(values) = schema.get("required").and_then(Value::as_array) {
        push_names(values, &mut required);
    }
    if let Some(values) = schema.get("createOnlyProperties").and_then(Value::as_array) {
        push_create_only(values, &mut required);
    }
    required
}

fn push_create_only(values: &[Value], required: &mut Vec<String>) {
    for value in values.iter().filter_map(Value::as_str) {
        if let Some(name) = value.strip_prefix("/properties/")
            && !name.contains('/')
        {
            push_unique(name, required);
        }
    }
}

pub(super) fn schema_filename(type_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = type_name.split("::").collect();
    let [provider, service, resource] = parts.as_slice() else {
        return Err(format!("unexpected type name {type_name}").into());
    };
    if *provider != "AWS" {
        return Err(format!("unexpected type name {type_name}").into());
    }
    Ok(format!(
        "aws-{}-{}.json",
        service.to_ascii_lowercase(),
        resource.to_ascii_lowercase()
    ))
}

fn push_names(values: &[Value], required: &mut Vec<String>) {
    for value in values.iter().filter_map(Value::as_str) {
        push_unique(value, required);
    }
}

fn push_unique(value: &str, required: &mut Vec<String>) {
    if !required.iter().any(|existing| existing == value) {
        required.push(value.to_owned());
    }
}
