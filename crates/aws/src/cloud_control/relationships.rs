use super::*;

pub(super) fn reference_arn_matches(rule: &Relationship, value: &str) -> bool {
    if arn_parts(value).is_none() || value.chars().any(char::is_control) {
        return false;
    }
    let mut parts = value.splitn(6, ':');
    let (Some(_), Some(partition), Some(service), Some(region), Some(account), Some(resource)) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return false;
    };
    let component = |value: &str| {
        value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    };
    if !component(partition)
        || !component(service)
        || !component(region)
        || (!account.is_empty()
            && !(account.len() == 12 && account.bytes().all(|byte| byte.is_ascii_digit()))
            && !(service == "iam" && account == "aws" && resource.starts_with("policy/")))
    {
        return false;
    }

    let concrete_log_group = service == "logs"
        && resource.strip_suffix(":*").is_some_and(|group| {
            group.starts_with("log-group:") && !group.contains(['*', '?', '{', '}'])
        });
    if value.contains(['?', '{', '}']) || (value.contains('*') && !concrete_log_group) {
        return false;
    }
    rule.arn_service
        .as_deref()
        .is_none_or(|expected| service == expected)
        && rule.arn_resource_prefix.as_deref().is_none_or(|prefix| {
            resource
                .strip_prefix(prefix)
                .is_some_and(|identifier| !identifier.is_empty())
        })
}

pub(super) fn reference_identifier_matches(rule: &Relationship, value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES {
        return false;
    }
    match rule.identifier_prefix.as_deref() {
        Some(prefix) => value.strip_prefix(prefix).is_some_and(|suffix| {
            !suffix.is_empty()
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        }),
        None => !value.contains(['\0', '|']),
    }
}

pub(super) fn map_relationships(
    ctx: &Context,
    descriptor: &ResourceType,
    resource: &Resource,
) -> Result<Vec<Edge>, AwsScanError> {
    let provider = resource
        .properties
        .get("provider_properties")
        .ok_or_else(|| invalid("missing sanitized provider properties"))?;
    let mut merged: BTreeMap<(String, String), Edge> = BTreeMap::new();
    for rule in &descriptor.relationships {
        for path in expand_pointer(provider, &rule.pointer)? {
            let Some(value) = provider
                .pointer(&path)
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            let (target, owner_resolution) = if rule.target_format == "arn" {
                if !reference_arn_matches(rule, value) {
                    continue;
                }
                (value.to_owned(), "returned_arn")
            } else {
                if !reference_identifier_matches(rule, value) {
                    continue;
                }
                let template = rule
                    .arn_template
                    .as_deref()
                    .ok_or_else(|| invalid("relationship ARN template absent"))?;
                (
                    expand_arn(ctx, template, value)?,
                    if template.contains("{account}") {
                        "assumed_scanning_account"
                    } else {
                        "arn_has_no_account_component"
                    },
                )
            };
            if target == resource.id {
                continue;
            }
            let key = (target.clone(), rule.relation.clone());
            if let Some(edge) = merged.get_mut(&key) {
                let paths = edge
                    .properties
                    .get_mut("property_paths")
                    .and_then(Value::as_array_mut)
                    .ok_or_else(|| invalid("edge path evidence is malformed"))?;
                if !paths.iter().any(|value| value.as_str() == Some(&path)) {
                    paths.push(Value::String(path));
                }
            } else {
                if merged.len() >= MAX_RESOURCE_EDGES {
                    return Err(invalid("resource relationship bound exceeded"));
                }
                let edge = Edge {
                    from_id: resource.id.clone(),
                    to_id: target,
                    relation: rule.relation.clone(),
                    source: Source::CloudControl,
                    observed_at: resource.observed_at,
                    properties: json!({"target_resource_id":value,"target_kind":rule.target_kind,"owner_resolution":owner_resolution,
                        "cloudformation_type":descriptor.type_name,"property_paths":[path],"evidence":rule.evidence,
                        "observation_detail":resource.properties.get("observation_detail")}),
                };
                edge.validate()?;
                merged.insert(key, edge);
            }
        }
    }
    Ok(merged.into_values().collect())
}
