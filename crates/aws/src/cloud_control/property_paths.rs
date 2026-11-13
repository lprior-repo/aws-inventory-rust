//! property paths operations.
use super::*;

pub(super) fn map_get(
    ctx: &Context,
    descriptor: &ResourceType,
    listed_identifier: &str,
    output: &GetResourceOutput,
) -> Result<PageBody, AwsScanError> {
    validate_response_type(&descriptor.type_name, output.type_name())?;
    let description = output
        .resource_description()
        .ok_or_else(|| invalid("GetResource response lacks ResourceDescription"))?;
    if identifier(description)? != listed_identifier {
        return Err(invalid("GetResource returned a different identifier"));
    }
    if description.properties().is_none() {
        return Err(invalid("GetResource response lacks Properties"));
    }
    map_description(ctx, descriptor, listed_identifier, description, true)
}

pub(super) fn escape_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

pub(super) fn normalized_key(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

/// Expand a reviewed wildcard pointer without recursion. Both arrays and
/// string-keyed property maps are supported; array positions never shift.
pub(super) fn expand_pointer(root: &Value, pattern: &str) -> Result<Vec<String>, AwsScanError> {
    validate_pointer(pattern)?;
    let mut paths = vec![String::new()];
    for segment in pattern.split('/').skip(1) {
        let mut next = Vec::new();
        let mut bytes = 0_usize;
        for parent in paths {
            let Some(value) = root.pointer(&parent) else {
                continue;
            };
            let parts: Vec<String> = if segment == "*" {
                match value {
                    Value::Array(values) => values
                        .iter()
                        .enumerate()
                        .map(|(index, _)| index.to_string())
                        .collect(),
                    Value::Object(values) => values.keys().map(|key| escape_pointer(key)).collect(),
                    _ => Vec::new(),
                }
            } else {
                vec![segment.to_owned()]
            };
            for part in parts {
                let path = format!("{parent}/{part}");
                if root.pointer(&path).is_some() {
                    bytes = bytes
                        .checked_add(path.len())
                        .ok_or_else(|| invalid("property path byte overflow"))?;
                    if next.len() >= MAX_PATH_MATCHES || bytes > MAX_PATH_BYTES {
                        return Err(invalid("property pointer expansion exceeds its bound"));
                    }
                    next.push(path);
                }
            }
        }
        paths = next;
    }
    Ok(paths)
}

pub(super) fn common_redactions(
    root: &Value,
    sensitive: &BTreeSet<String>,
) -> Result<BTreeSet<String>, AwsScanError> {
    let mut paths = BTreeSet::new();
    let mut stack = vec![(String::new(), root, 0_usize)];
    let mut nodes = 0_usize;
    let mut path_bytes = 0_usize;
    for _ in 0..MAX_JSON_NODES {
        let Some((path, value, depth)) = stack.pop() else {
            return Ok(paths);
        };
        nodes = nodes
            .checked_add(1)
            .ok_or_else(|| invalid("JSON node counter overflow"))?;
        if depth > MAX_JSON_DEPTH || path.len() > MAX_PATH_BYTES {
            return Err(invalid("provider JSON nesting exceeds its bound"));
        }
        let next_depth = depth
            .checked_add(1)
            .ok_or_else(|| invalid("JSON depth overflow"))?;
        match value {
            Value::Object(values) => {
                for (key, child) in values {
                    let child_path = format!("{path}/{}", escape_pointer(key));
                    path_bytes = path_bytes
                        .checked_add(child_path.len())
                        .ok_or_else(|| invalid("JSON path byte overflow"))?;
                    if sensitive.contains(&normalized_key(key)) {
                        paths.insert(child_path);
                    } else {
                        stack.push((child_path, child, next_depth));
                    }
                }
            }
            Value::Array(values) => {
                for (index, child) in values.iter().enumerate() {
                    let child_path = format!("{path}/{index}");
                    path_bytes = path_bytes
                        .checked_add(child_path.len())
                        .ok_or_else(|| invalid("JSON path byte overflow"))?;
                    stack.push((child_path, child, next_depth));
                }
            }
            _ => {}
        }
        if nodes.saturating_add(stack.len()) > MAX_JSON_NODES
            || paths.len() > MAX_PATH_MATCHES
            || path_bytes > MAX_PATH_BYTES
        {
            return Err(invalid(
                "provider JSON traversal exceeds its node or path budget",
            ));
        }
    }
    if stack.is_empty() {
        Ok(paths)
    } else {
        Err(invalid("provider JSON node cap reached"))
    }
}

pub(super) fn remove_pointer(root: &mut Value, pointer: &str) -> Result<(), AwsScanError> {
    let (parent, key) = pointer
        .rsplit_once('/')
        .ok_or_else(|| invalid("redaction path has no parent"))?;
    let key = key.replace("~1", "/").replace("~0", "~");
    match root.pointer_mut(parent) {
        Some(Value::Object(values)) => {
            values.remove(&key);
        }
        Some(Value::Array(values)) => {
            let index = key
                .parse::<usize>()
                .map_err(|_| invalid("redaction array index is invalid"))?;
            if let Some(value) = values.get_mut(index) {
                *value = Value::Null;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn sanitized_properties(
    descriptor: &ResourceType,
    raw: Option<&str>,
) -> Result<(Value, Vec<String>), AwsScanError> {
    let raw = raw.unwrap_or("{}");
    if raw.len() > MAX_PROVIDER_BYTES {
        return Err(invalid("Cloud Control Properties exceeds 262144 bytes"));
    }
    let mut value: Value = serde_json::from_str(raw)
        .map_err(|error| invalid(format!("invalid provider JSON: {error}")))?;
    if !value.is_object() {
        return Err(invalid("provider Properties must be a JSON object"));
    }
    let mut redactions = common_redactions(&value, &descriptor.sensitive_names)?;
    for pattern in descriptor
        .write_only_properties
        .iter()
        .chain(&descriptor.sensitive_properties)
    {
        redactions.extend(expand_pointer(&value, pattern)?);
        if redactions.len() > MAX_PATH_MATCHES {
            return Err(invalid("redacted-path count exceeds its bound"));
        }
    }
    let bytes = redactions.iter().try_fold(0_usize, |total, value| {
        total
            .checked_add(value.len())
            .ok_or_else(|| invalid("redacted-path byte overflow"))
    })?;
    if bytes > MAX_PROVIDER_BYTES {
        return Err(invalid("redacted-path metadata exceeds its byte bound"));
    }
    for pointer in &redactions {
        remove_pointer(&mut value, pointer)?;
    }
    Ok((value, redactions.into_iter().collect()))
}
