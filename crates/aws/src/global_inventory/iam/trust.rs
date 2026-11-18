use super::*;

pub(crate) fn trust_summary(document: Option<&str>) -> Result<Vec<Value>, AwsScanError> {
    let Some(document) = document else {
        return Ok(Vec::new());
    };
    if document.len() > MAX_TRUST_BYTES {
        return Err(AwsScanError::InvalidData(
            "trust policy exceeds 64 KiB".into(),
        ));
    }
    let decoded = percent_encoding::percent_decode_str(document)
        .decode_utf8()
        .map_err(|_| AwsScanError::InvalidData("trust policy is not UTF-8".into()))?;
    let policy: Value = serde_json::from_str(&decoded)
        .map_err(|_| AwsScanError::InvalidData("trust policy is not valid JSON".into()))?;
    let Some(statements) = policy.get("Statement") else {
        return Err(AwsScanError::InvalidData(
            "trust policy lacks Statement".into(),
        ));
    };
    let statements: Vec<&Value> = match statements {
        Value::Array(values) if values.len() <= MAX_TRUST_STATEMENTS => values.iter().collect(),
        Value::Object(_) => vec![statements],
        _ => {
            return Err(AwsScanError::InvalidData(
                "trust policy statements exceed bounds or have invalid shape".into(),
            ));
        }
    };
    let summaries = statements
        .into_iter()
        .map(trust_statement)
        .collect::<Result<Vec<_>, _>>()?;
    let _principal_count = summaries
        .iter()
        .filter_map(|summary| summary.get("principals").and_then(Value::as_object))
        .flat_map(|principals| principals.values())
        .filter_map(Value::as_array)
        .try_fold(0_usize, |count, values| {
            count
                .checked_add(values.len())
                .filter(|count| *count <= MAX_PRINCIPALS)
                .ok_or_else(|| {
                    AwsScanError::InvalidData("trust policy exceeds 256 principals".into())
                })
        })?;
    Ok(summaries)
}

pub(super) fn string_values(
    value: Option<&Value>,
    limit: usize,
) -> Result<Vec<String>, AwsScanError> {
    match value {
        None => Ok(Vec::new()),
        Some(Value::String(value)) => Ok(vec![checked_text(value, "trust reference")?]),
        Some(Value::Array(values)) if values.len() <= limit => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| AwsScanError::InvalidData("trust reference is not text".into()))
                    .and_then(|value| checked_text(value, "trust reference"))
            })
            .collect(),
        _ => Err(AwsScanError::InvalidData(
            "trust reference exceeds bounds or has invalid shape".into(),
        )),
    }
}

pub(super) fn trust_statement(statement: &Value) -> Result<Value, AwsScanError> {
    let object = statement
        .as_object()
        .ok_or_else(|| AwsScanError::InvalidData("trust statement is not an object".into()))?;
    let mut principals = BTreeMap::new();
    match object.get("Principal") {
        Some(Value::String(value)) if value == "*" => {
            principals.insert("AWS", vec!["*".to_owned()]);
        }
        Some(Value::Object(values)) => {
            for kind in ["AWS", "Service", "Federated", "CanonicalUser"] {
                let refs = string_values(values.get(kind), MAX_PRINCIPALS)?;
                if !refs.is_empty() {
                    principals.insert(kind, refs);
                }
            }
        }
        None => {}
        _ => {
            return Err(AwsScanError::InvalidData(
                "invalid trust principal shape".into(),
            ));
        }
    }
    let condition_keys: Vec<&str> = object
        .get("Condition")
        .and_then(Value::as_object)
        .map(|conditions| conditions.keys().map(String::as_str).collect())
        .unwrap_or_default();
    components(condition_keys.len(), "trust condition operators")?;
    Ok(
        json!({"effect":object.get("Effect").and_then(Value::as_str),
        "actions":string_values(object.get("Action"), 64)?, "principals":principals,
        "has_not_principal":object.contains_key("NotPrincipal"), "has_not_action":object.contains_key("NotAction"),
        "condition_operators":condition_keys, "condition_values_omitted":object.contains_key("Condition")}),
    )
}

pub(super) fn trust_edges(
    ctx: &Context,
    resource: &Resource,
    statements: &[Value],
) -> Result<Vec<Edge>, AwsScanError> {
    let mut edges = Vec::new();
    for statement in statements.iter().take(MAX_TRUST_STATEMENTS) {
        if statement.get("effect").and_then(Value::as_str) != Some("Allow") {
            continue;
        }
        let Some(principals) = statement.get("principals").and_then(Value::as_object) else {
            continue;
        };
        for (kind, values) in principals.iter().take(4) {
            let Some(values) = values.as_array() else {
                continue;
            };
            for value in values.iter().take(MAX_PRINCIPALS) {
                let Some(value) = value.as_str() else {
                    continue;
                };
                let target = principal_target(ctx, kind, value)?;
                if edges.len() >= super::super::MAX_COMPONENTS {
                    return Err(AwsScanError::ChildLimit);
                }
                edges.push(reference(ctx, resource, &target, "DECLARES_TRUST", json!({
                    "principal_type":kind, "effect":"Allow", "actions":statement.get("actions"),
                    "condition_operators":statement.get("condition_operators"), "effective_permission_inferred":false
                }))?);
            }
        }
    }
    Ok(edges)
}

pub(super) fn principal_target(
    ctx: &Context,
    kind: &str,
    value: &str,
) -> Result<String, AwsScanError> {
    if value.starts_with("arn:") {
        return checked_text(value, "trust principal ARN");
    }
    if kind == "AWS" && value.len() == 12 && value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Ok(format!("arn:{}:iam::{value}:root", ctx.partition));
    }
    if value == "*" {
        return Ok("aws-principal:any".into());
    }
    checked_text(value, "trust principal")?;
    Ok(format!(
        "aws-principal:{}:{value}",
        kind.to_ascii_lowercase()
    ))
}
