use super::*;

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
