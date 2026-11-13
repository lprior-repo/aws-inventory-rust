//! IAM identity metadata and declared policy references. Policy documents and
//! trust-condition values are deliberately excluded from persisted records.

use std::collections::BTreeMap;

use aws_sdk_iam::{
    Client,
    types::{InstanceProfile, Policy, PolicyScopeType, Role, Tag},
};
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::{Edge, Resource, Source};
use serde_json::{Value, json};

use super::{
    CHILD_CONCURRENCY, SeenIds, checked_text, components, edge_set, ensure_active, marker,
    observed, own_arn, reference, single,
};
use crate::AwsScanError;
use crate::common::{
    Context, PageBody, collect_bounded, paginate, require_budget, required, sdk_error,
};

const ROLE: &str = "aws_iam_role";
const PROFILE: &str = "aws_iam_instance_profile";
const POLICY: &str = "aws_iam_policy";
const MAX_TRUST_BYTES: usize = 65_536;
const MAX_TRUST_STATEMENTS: usize = 128;
const MAX_PRINCIPALS: usize = 256;

#[tracing::instrument(skip_all)]
pub(super) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let client = Client::new(&ctx.config);
    let work: [BoxFuture<'_, Result<(), AwsScanError>>; 3] = [
        Box::pin(roles(ctx, &client)),
        Box::pin(profiles(ctx, &client)),
        Box::pin(policies(ctx, &client)),
    ];
    stream::iter(work)
        .buffer_unordered(3)
        .try_for_each(|()| async { Ok(()) })
        .await?;
    ensure_active(ctx)
}

#[tracing::instrument(skip_all)]
async fn roles(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_iam_role_list", None, Source::Direct),
        |token, budget| {
            let seen = &seen;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("iam", "read", || async {
                        client
                            .list_roles()
                            .set_marker(token.clone())
                            .max_items(ctx.page_size.clamp(1, 1_000))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                let count = seen.record(output.roles().iter().map(Role::arn), budget)?;
                let next_token = marker(output.is_truncated(), output.marker())?;
                collect_bounded(output.roles(), CHILD_CONCURRENCY, |role| {
                    role_details(ctx, client, role)
                })
                .await?;
                Ok(PageBody {
                    next_token,
                    listed_count: count,
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn role_details(ctx: &Context, client: &Client, listed: &Role) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let scope = ctx.scope(ROLE, Some(listed.arn()), Source::Direct);
    let fallback = match map_role(ctx, listed, false) {
        Ok(body) => body,
        Err(error) => return observed(ctx, scope, Err(error), None).await,
    };
    let source = fallback
        .resources
        .first()
        .cloned()
        .ok_or_else(|| AwsScanError::InvalidData("role mapping yielded no resource".into()))?;
    let result = ctx
        .execute("iam", "read", || async {
            client
                .get_role()
                .role_name(listed.role_name())
                .send()
                .await
                .map_err(sdk_error)
        })
        .await
        .map_err(AwsScanError::from)
        .and_then(|output| {
            let role = output
                .role()
                .ok_or_else(|| AwsScanError::InvalidData("GetRole lacks role".into()))?;
            if role.arn() != listed.arn() {
                return Err(AwsScanError::InvalidData("GetRole identity changed".into()));
            }
            map_role(ctx, role, true)
        });
    observed(ctx, scope, result, Some(fallback)).await?;
    attached_policies(ctx, client, listed, &source).await
}

#[tracing::instrument(skip_all)]
async fn attached_policies(
    ctx: &Context,
    client: &Client,
    role: &Role,
    source: &Resource,
) -> Result<(), AwsScanError> {
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope(
            "aws_iam_role_attached_policies",
            Some(role.arn()),
            Source::Direct,
        ),
        |token, budget| {
            let seen = &seen;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("iam", "read", || async {
                        client
                            .list_attached_role_policies()
                            .role_name(role.role_name())
                            .set_marker(token.clone())
                            .max_items(ctx.page_size.clamp(1, 1_000))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.attached_policies().len(), budget)?;
                let arns = output
                    .attached_policies()
                    .iter()
                    .map(|policy| required(policy.policy_arn(), "attached policy ARN"))
                    .collect::<Result<Vec<_>, _>>()?;
                let count = seen.record(arns.iter().map(String::as_str), budget)?;
                let edges = arns
                    .iter()
                    .map(|arn| {
                        reference(
                            ctx,
                            source,
                            arn,
                            "ATTACHED_POLICY",
                            json!({"field":"ListAttachedRolePolicies.PolicyArn"}),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(PageBody {
                    edges,
                    next_token: marker(output.is_truncated(), output.marker())?,
                    listed_count: count,
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn profiles(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_iam_instance_profile_list", None, Source::Direct),
        |token, budget| {
            let seen = &seen;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("iam", "read", || async {
                        client
                            .list_instance_profiles()
                            .set_marker(token.clone())
                            .max_items(ctx.page_size.clamp(1, 1_000))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                let count = seen.record(
                    output.instance_profiles().iter().map(InstanceProfile::arn),
                    budget,
                )?;
                let next_token = marker(output.is_truncated(), output.marker())?;
                collect_bounded(output.instance_profiles(), CHILD_CONCURRENCY, |profile| {
                    profile_details(ctx, client, profile)
                })
                .await?;
                Ok(PageBody {
                    next_token,
                    listed_count: count,
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn profile_details(
    ctx: &Context,
    client: &Client,
    listed: &InstanceProfile,
) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let scope = ctx.scope(PROFILE, Some(listed.arn()), Source::Direct);
    let fallback = match map_profile(ctx, listed, false) {
        Ok(body) => body,
        Err(error) => return observed(ctx, scope, Err(error), None).await,
    };
    let result = ctx
        .execute("iam", "read", || async {
            client
                .get_instance_profile()
                .instance_profile_name(listed.instance_profile_name())
                .send()
                .await
                .map_err(sdk_error)
        })
        .await
        .map_err(AwsScanError::from)
        .and_then(|output| {
            let value = output.instance_profile().ok_or_else(|| {
                AwsScanError::InvalidData("GetInstanceProfile lacks profile".into())
            })?;
            if value.arn() != listed.arn() {
                return Err(AwsScanError::InvalidData(
                    "GetInstanceProfile identity changed".into(),
                ));
            }
            map_profile(ctx, value, true)
        });
    observed(ctx, scope, result, Some(fallback)).await
}

#[tracing::instrument(skip_all)]
async fn policies(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_iam_policy_list", Some("scope=Local"), Source::Direct),
        |token, budget| {
            let seen = &seen;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("iam", "read", || async {
                        client
                            .list_policies()
                            .scope(PolicyScopeType::Local)
                            .only_attached(false)
                            .set_marker(token.clone())
                            .max_items(ctx.page_size.clamp(1, 1_000))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.policies().len(), budget)?;
                let arns = output
                    .policies()
                    .iter()
                    .map(|policy| required(policy.arn(), "managed policy ARN"))
                    .collect::<Result<Vec<_>, _>>()?;
                let count = seen.record(arns.iter().map(String::as_str), budget)?;
                let next_token = marker(output.is_truncated(), output.marker())?;
                collect_bounded(output.policies(), CHILD_CONCURRENCY, |policy| {
                    policy_details(ctx, client, policy)
                })
                .await?;
                Ok(PageBody {
                    next_token,
                    listed_count: count,
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn policy_details(
    ctx: &Context,
    client: &Client,
    listed: &Policy,
) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let arn = required(listed.arn(), "managed policy ARN")?;
    let scope = ctx.scope(POLICY, Some(&arn), Source::Direct);
    let fallback = match map_policy(ctx, listed, false) {
        Ok(body) => body,
        Err(error) => return observed(ctx, scope, Err(error), None).await,
    };
    let result = ctx
        .execute("iam", "read", || async {
            client
                .get_policy()
                .policy_arn(&arn)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await
        .map_err(AwsScanError::from)
        .and_then(|output| {
            let value = output
                .policy()
                .ok_or_else(|| AwsScanError::InvalidData("GetPolicy lacks policy".into()))?;
            if value.arn() != Some(arn.as_str()) {
                return Err(AwsScanError::InvalidData(
                    "GetPolicy identity changed".into(),
                ));
            }
            map_policy(ctx, value, true)
        });
    observed(ctx, scope, result, Some(fallback)).await
}

fn tags(values: &[Tag]) -> Result<BTreeMap<String, String>, AwsScanError> {
    components(values.len(), "IAM tags")?;
    values
        .iter()
        .map(|tag| {
            Ok((
                checked_text(tag.key(), "IAM tag key")?,
                tag.value().to_owned(),
            ))
        })
        .collect()
}

fn millis(value: &aws_smithy_types::DateTime) -> Result<i64, AwsScanError> {
    value
        .to_millis()
        .map_err(|_| AwsScanError::InvalidData("IAM timestamp out of range".into()))
}

fn map_role(ctx: &Context, role: &Role, detailed: bool) -> Result<PageBody, AwsScanError> {
    let arn = own_arn(ctx, role.arn(), "iam", "role")?;
    let trust = if detailed {
        trust_summary(role.assume_role_policy_document())?
    } else {
        Vec::new()
    };
    let boundary = role
        .permissions_boundary()
        .and_then(|boundary| boundary.permissions_boundary_arn());
    let resource = ctx.resource(ROLE, arn, Some(checked_text(role.role_name(), "IAM role name")?), json!({
        "role_name":role.role_name(), "role_id":role.role_id(), "path":role.path(),
        "create_date":millis(role.create_date())?, "description":role.description(),
        "max_session_duration":role.max_session_duration(), "permissions_boundary_arn":boundary,
        "last_used_date":role.role_last_used().and_then(|v| v.last_used_date()).map(millis).transpose()?,
        "last_used_region":role.role_last_used().and_then(|v| v.region()),
        "trust_statements":trust, "trust_policy_values_omitted":true,
        "details_available":detailed, "observation_detail":if detailed {"get_role"} else {"list_only"}
    }), if detailed { tags(role.tags())? } else { BTreeMap::new() })?;
    let mut edges = trust_edges(ctx, &resource, &trust)?;
    if let Some(boundary) = boundary {
        edges.push(reference(
            ctx,
            &resource,
            boundary,
            "USES_PERMISSIONS_BOUNDARY",
            json!({"field":"PermissionsBoundary.PermissionsBoundaryArn"}),
        )?);
    }
    resource.validate()?;
    Ok(single(resource, edge_set(edges)?))
}

fn map_profile(
    ctx: &Context,
    profile: &InstanceProfile,
    detailed: bool,
) -> Result<PageBody, AwsScanError> {
    components(profile.roles().len(), "instance-profile roles")?;
    let arn = own_arn(ctx, profile.arn(), "iam", "instance-profile")?;
    let roles: Vec<&str> = profile.roles().iter().map(Role::arn).collect();
    let resource = ctx.resource(PROFILE, arn, Some(checked_text(profile.instance_profile_name(), "instance profile name")?), json!({
        "instance_profile_name":profile.instance_profile_name(), "instance_profile_id":profile.instance_profile_id(),
        "path":profile.path(), "create_date":millis(profile.create_date())?, "role_arns":roles,
        "details_available":detailed, "observation_detail":if detailed {"get_instance_profile"} else {"list_only"}
    }), if detailed { tags(profile.tags())? } else { BTreeMap::new() })?;
    let edges = roles
        .iter()
        .map(|arn| {
            reference(
                ctx,
                &resource,
                arn,
                "HAS_ROLE",
                json!({"field":"Roles.Arn"}),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    resource.validate()?;
    Ok(single(resource, edge_set(edges)?))
}

fn map_policy(ctx: &Context, policy: &Policy, detailed: bool) -> Result<PageBody, AwsScanError> {
    let arn = own_arn(
        ctx,
        &required(policy.arn(), "managed policy ARN")?,
        "iam",
        "policy",
    )?;
    let name = required(policy.policy_name(), "managed policy name")?;
    let resource = ctx.resource(POLICY, arn, Some(name.clone()), json!({
        "policy_name":name, "policy_id":policy.policy_id(), "path":policy.path(),
        "default_version_id":policy.default_version_id(), "attachment_count":policy.attachment_count(),
        "permissions_boundary_usage_count":policy.permissions_boundary_usage_count(), "is_attachable":policy.is_attachable(),
        "description":policy.description(), "create_date":policy.create_date().map(millis).transpose()?,
        "update_date":policy.update_date().map(millis).transpose()?, "policy_scope":"Local",
        "policy_document_collected":false, "details_available":detailed,
        "observation_detail":if detailed {"get_policy"} else {"list_only"}
    }), if detailed { tags(policy.tags())? } else { BTreeMap::new() })?;
    resource.validate()?;
    Ok(single(resource, Vec::new()))
}

fn trust_summary(document: Option<&str>) -> Result<Vec<Value>, AwsScanError> {
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
    statements.into_iter().map(trust_statement).collect()
}

fn string_values(value: Option<&Value>, limit: usize) -> Result<Vec<String>, AwsScanError> {
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

fn trust_statement(statement: &Value) -> Result<Value, AwsScanError> {
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

fn trust_edges(
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
                if edges.len() >= super::MAX_COMPONENTS {
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

fn principal_target(ctx: &Context, kind: &str, value: &str) -> Result<String, AwsScanError> {
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

#[cfg(test)]
mod tests {
    use super::super::tests::context;
    use super::*;

    fn role() -> Result<Role, Box<dyn std::error::Error>> {
        Ok(Role::builder()
            .path("/application/")
            .role_name("worker")
            .role_id("AROAEXAMPLE")
            .arn("arn:aws:iam::123456789012:role/application/worker")
            .create_date(aws_smithy_types::DateTime::from_secs(1))
            .build()?)
    }

    #[test]
    fn role_maps_identity_and_boundary_without_policy_payload()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let mut value = role()?;
        value.permissions_boundary = Some(
            aws_sdk_iam::types::AttachedPermissionsBoundary::builder()
                .permissions_boundary_arn("arn:aws:iam::123456789012:policy/boundary")
                .build(),
        );
        value.assume_role_policy_document = Some(r#"{"Statement":{"Effect":"Allow","Principal":{"Service":"ecs-tasks.amazonaws.com"},"Action":"sts:AssumeRole","Condition":{"StringEquals":{"secret":"NEVER_PERSIST"}}}}"#.into());
        let body = map_role(&ctx, &value, true)?;
        let resource = body.resources.first().ok_or("missing role")?;
        assert_eq!(resource.region, "global");
        assert_eq!(
            resource.properties.get("path").and_then(Value::as_str),
            Some("/application/")
        );
        let serialized = serde_json::to_string(&resource)?;
        assert!(!serialized.contains("NEVER_PERSIST"));
        assert_eq!(body.edges.len(), 2);
        assert!(
            body.edges
                .iter()
                .any(|edge| edge.relation == "USES_PERMISSIONS_BOUNDARY")
        );
        assert!(
            body.edges
                .iter()
                .any(|edge| edge.relation == "DECLARES_TRUST")
        );
        Ok(())
    }

    #[test]
    fn list_only_roles_do_not_claim_detail_fields_or_trust_edges()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let mut value = role()?;
        value.assume_role_policy_document = Some("malformed list-only document".into());
        let body = map_role(&ctx, &value, false)?;
        assert!(body.edges.is_empty());
        let resource = body.resources.first().ok_or("missing role")?;
        assert_eq!(
            resource.properties.get("details_available"),
            Some(&json!(false))
        );
        assert!(resource.tags.is_empty());
        Ok(())
    }

    #[test]
    fn trust_parser_decodes_rfc3986_and_does_not_treat_plus_as_space() -> Result<(), AwsScanError> {
        let value = trust_summary(Some(
            "%7B%22Statement%22%3A%7B%22Effect%22%3A%22Allow%22%2C%22Principal%22%3A%7B%22Service%22%3A%22a+b.example%22%7D%2C%22Action%22%3A%22sts%3AAssumeRole%22%7D%7D",
        ))?;
        assert_eq!(
            value
                .first()
                .and_then(|v| v.pointer("/principals/Service/0"))
                .and_then(Value::as_str),
            Some("a+b.example")
        );
        assert!(trust_summary(Some(&"a".repeat(MAX_TRUST_BYTES.saturating_add(1)))).is_err());
        assert!(trust_summary(Some("%FF")).is_err());
        Ok(())
    }

    #[test]
    fn deny_and_not_principal_never_become_allow_edges() -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let mut value = role()?;
        value.assume_role_policy_document = Some(r#"{"Statement":[{"Effect":"Deny","Principal":"*","Action":"sts:AssumeRole"},{"Effect":"Allow","NotPrincipal":{"AWS":"*"},"Action":"sts:AssumeRole"}]}"#.into());
        assert!(map_role(&ctx, &value, true)?.edges.is_empty());
        Ok(())
    }

    #[test]
    fn profile_links_exact_role_arn_and_policy_metadata_omits_document()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let profile = InstanceProfile::builder()
            .path("/application/")
            .instance_profile_name("worker")
            .instance_profile_id("AIPAEXAMPLE")
            .arn("arn:aws:iam::123456789012:instance-profile/application/worker")
            .create_date(aws_smithy_types::DateTime::from_secs(2))
            .roles(role()?)
            .build()?;
        let body = map_profile(&ctx, &profile, true)?;
        assert_eq!(
            body.edges.first().map(|e| e.to_id.as_str()),
            Some("arn:aws:iam::123456789012:role/application/worker")
        );
        let policy = Policy::builder()
            .policy_name("custom")
            .arn("arn:aws:iam::123456789012:policy/application/custom")
            .default_version_id("v3")
            .attachment_count(4)
            .is_attachable(true)
            .build();
        let body = map_policy(&ctx, &policy, true)?;
        assert_eq!(
            body.resources
                .first()
                .and_then(|r| r.properties.get("policy_document_collected")),
            Some(&json!(false))
        );
        assert_eq!(
            body.resources
                .first()
                .and_then(|r| r.properties.get("default_version_id")),
            Some(&json!("v3"))
        );
        Ok(())
    }

    #[test]
    fn own_iam_identities_cannot_silently_change_account() -> Result<(), Box<dyn std::error::Error>>
    {
        let (ctx, _rx) = context()?;
        let mut value = role()?;
        value.arn = "arn:aws:iam::999999999999:role/other".into();
        assert!(map_role(&ctx, &value, true).is_err());
        Ok(())
    }
}
