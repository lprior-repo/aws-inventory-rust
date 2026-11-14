mod model;
mod nodes;

use serde_json::Value;

use crate::HarnessError;

pub use model::empty_model_valid;
pub use nodes::schema_nodes;

pub const MAX_RELATIONSHIP_RULES: usize = 256;

pub fn snake(value: &str) -> String {
    let first = insert_boundaries(value, true);
    let second = insert_boundaries(&first, false);
    let lowered: Vec<char> = second.to_lowercase().chars().collect();
    let mut out = String::new();
    let mut pending = false;
    for character in lowered {
        if character.is_ascii_digit() || character.is_ascii_lowercase() {
            if pending && !out.is_empty() {
                out.push('_');
            }
            pending = false;
            out.push(character);
        } else {
            pending = true;
        }
    }
    out
}

fn insert_boundaries(value: &str, upper_run: bool) -> String {
    let characters: Vec<char> = value.chars().collect();
    let mut out = String::new();
    for (index, character) in characters.iter().enumerate() {
        if index > 0 {
            let previous = characters.get(index.saturating_sub(1)).copied();
            let next = characters.get(index.saturating_add(1)).copied();
            let boundary = if upper_run {
                previous.is_some_and(|value| value.is_ascii_uppercase())
                    && character.is_ascii_uppercase()
                    && next.is_some_and(|value| value.is_ascii_lowercase())
            } else {
                previous.is_some_and(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
                    && character.is_ascii_uppercase()
            };
            if boundary {
                out.push('_');
            }
        }
        out.push(*character);
    }
    out
}

pub fn valid_column(column: &str) -> bool {
    let mut characters = column.chars();
    match characters.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    characters.all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    })
}

pub fn pointer(value: &str) -> Result<String, HarnessError> {
    value
        .strip_prefix("/properties/")
        .map(|rest| format!("/{rest}"))
        .ok_or_else(|| HarnessError::invalid(format!("unexpected schema pointer {value}")))
}

pub fn escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

pub fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn basic_type(prop: &Value, schema: &Value) -> Result<&'static str, HarnessError> {
    let mut current = prop.clone();
    for _ in 0..8 {
        let Some(reference) = current.get("$ref").and_then(Value::as_str) else {
            break;
        };
        if !reference.starts_with("#/") {
            return Ok("mixed");
        }
        current = target(schema, reference)?.clone();
    }
    Ok(type_name(&current))
}

fn target<'a>(schema: &'a Value, reference: &str) -> Result<&'a Value, HarnessError> {
    let mut target = schema;
    for piece in ref_tail(reference)?.split('/') {
        let key = piece.replace("~1", "/").replace("~0", "~");
        target = target.get(&key).ok_or_else(|| {
            HarnessError::invalid(format!("unresolved schema reference {reference}"))
        })?;
    }
    Ok(target)
}

fn type_name(node: &Value) -> &'static str {
    match node.get("type").and_then(Value::as_str) {
        Some("string") => "string",
        Some("number") => "number",
        Some("integer") => "integer",
        Some("boolean") => "boolean",
        Some("object") => "object",
        Some("array") => "array",
        Some("null") => "null",
        _ => "mixed",
    }
}

pub(super) fn ref_tail(reference: &str) -> Result<&str, HarnessError> {
    reference
        .get(2..)
        .ok_or_else(|| HarnessError::invalid(format!("unresolved schema reference {reference}")))
}

pub fn string_node(node: &Value) -> bool {
    match node.get("type") {
        Some(Value::String(value)) => value == "string",
        Some(Value::Array(values)) => values.iter().any(|value| value.as_str() == Some("string")),
        _ => false,
    }
}

pub fn under(path: &str, parent: &str) -> bool {
    let left: Vec<&str> = path.split('/').skip(1).collect();
    let right: Vec<&str> = parent.split('/').skip(1).collect();
    if left.len() < right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .all(|(actual, expected)| expected == &"*" || expected == actual)
}
