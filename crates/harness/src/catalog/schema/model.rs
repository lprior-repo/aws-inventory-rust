//! Draft 7 emptiness check for list handler request models.

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::schema::target;

/// Draft 7 `is_valid({})` for a list handler request model.
pub fn empty_model_valid(schema: &Value) -> Result<bool, HarnessError> {
    empty_valid(schema, schema)
}

fn empty_valid(node: &Value, root: &Value) -> Result<bool, HarnessError> {
    match node {
        Value::Bool(value) => return Ok(*value),
        Value::Object(_) => {}
        _ => return Ok(true),
    }
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        return resolve(reference, root).and_then(|resolved| empty_valid(resolved, root));
    }
    if !type_allows_empty(node) || !scalar_allows_empty(node) {
        return Ok(false);
    }
    if !combinators_allow_empty(node, root)? || !conditional_allows_empty(node, root)? {
        return Ok(false);
    }
    Ok(true)
}

fn resolve<'a>(reference: &str, root: &'a Value) -> Result<&'a Value, HarnessError> {
    if !reference.starts_with("#/") {
        return Err(HarnessError::invalid(
            "external schema reference cannot establish a local mapping",
        ));
    }
    target(root, reference)
}

fn type_allows_empty(node: &Value) -> bool {
    match node.get("type") {
        Some(declared) => accepts_object(declared),
        None => true,
    }
}

fn accepts_object(declared: &Value) -> bool {
    match declared {
        Value::String(value) => value == "object",
        Value::Array(values) => values.iter().any(|value| value.as_str() == Some("object")),
        _ => true,
    }
}

fn scalar_allows_empty(node: &Value) -> bool {
    enum_allows_empty(node)
        && const_allows_empty(node)
        && required_allows_empty(node)
        && !node
            .get("minProperties")
            .and_then(Value::as_u64)
            .is_some_and(|minimum| minimum > 0)
}

fn enum_allows_empty(node: &Value) -> bool {
    match node.get("enum").and_then(Value::as_array) {
        Some(values) => values.contains(&Value::Object(serde_json::Map::new())),
        None => true,
    }
}

fn const_allows_empty(node: &Value) -> bool {
    match node.get("const") {
        Some(constant) => *constant == Value::Object(serde_json::Map::new()),
        None => true,
    }
}

fn required_allows_empty(node: &Value) -> bool {
    match node.get("required").and_then(Value::as_array) {
        Some(required) => required.is_empty(),
        None => true,
    }
}

fn combinators_allow_empty(node: &Value, root: &Value) -> Result<bool, HarnessError> {
    if !all_of_valid(node, root)? || !any_of_valid(node, root)? || !one_of_valid(node, root)? {
        return Ok(false);
    }
    if let Some(negated) = node.get("not")
        && empty_valid(negated, root)?
    {
        return Ok(false);
    }
    Ok(true)
}

fn all_of_valid(node: &Value, root: &Value) -> Result<bool, HarnessError> {
    let Some(branches) = node.get("allOf").and_then(Value::as_array) else {
        return Ok(true);
    };
    for branch in branches {
        if !empty_valid(branch, root)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn any_of_valid(node: &Value, root: &Value) -> Result<bool, HarnessError> {
    let Some(branches) = node.get("anyOf").and_then(Value::as_array) else {
        return Ok(true);
    };
    let mut matched = false;
    for branch in branches {
        matched = matched || empty_valid(branch, root)?;
    }
    Ok(matched)
}

fn one_of_valid(node: &Value, root: &Value) -> Result<bool, HarnessError> {
    let Some(branches) = node.get("oneOf").and_then(Value::as_array) else {
        return Ok(true);
    };
    let mut matched = 0_usize;
    for branch in branches {
        if empty_valid(branch, root)? {
            matched = matched.saturating_add(1);
        }
    }
    Ok(matched == 1)
}

fn conditional_allows_empty(node: &Value, root: &Value) -> Result<bool, HarnessError> {
    let Some(condition) = node.get("if") else {
        return Ok(true);
    };
    let selected = if empty_valid(condition, root)? {
        node.get("then")
    } else {
        node.get("else")
    };
    match selected {
        Some(branch) => empty_valid(branch, root),
        None => Ok(true),
    }
}
