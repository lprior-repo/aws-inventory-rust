use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ObservationDetail {
    Listed,
    Fetched,
}

pub(super) fn tags(values: &[Tag]) -> Result<BTreeMap<String, String>, AwsScanError> {
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

pub(super) fn millis(value: &aws_smithy_types::DateTime) -> Result<i64, AwsScanError> {
    value
        .to_millis()
        .map_err(|_| AwsScanError::InvalidData("IAM timestamp out of range".into()))
}

pub(super) fn map_role(
    ctx: &Context,
    role: &Role,
    detail: ObservationDetail,
) -> Result<PageBody, AwsScanError> {
    let arn = own_arn(ctx, role.arn(), "iam", "role")?;
    let trust = if detail == ObservationDetail::Fetched {
        crate::iam_trust::summarize(role.assume_role_policy_document())?
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
        "details_available":detail == ObservationDetail::Fetched, "observation_detail":if detail == ObservationDetail::Fetched {"get_role"} else {"list_only"}
    }), if detail == ObservationDetail::Fetched { tags(role.tags())? } else { BTreeMap::new() })?;
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

pub(super) fn map_profile(
    ctx: &Context,
    profile: &InstanceProfile,
    detail: ObservationDetail,
) -> Result<PageBody, AwsScanError> {
    components(profile.roles().len(), "instance-profile roles")?;
    let arn = own_arn(ctx, profile.arn(), "iam", "instance-profile")?;
    let roles: Vec<&str> = profile.roles().iter().map(Role::arn).collect();
    let resource = ctx.resource(PROFILE, arn, Some(checked_text(profile.instance_profile_name(), "instance profile name")?), json!({
        "instance_profile_name":profile.instance_profile_name(), "instance_profile_id":profile.instance_profile_id(),
        "path":profile.path(), "create_date":millis(profile.create_date())?, "role_arns":roles,
        "details_available":detail == ObservationDetail::Fetched, "observation_detail":if detail == ObservationDetail::Fetched {"get_instance_profile"} else {"list_only"}
    }), if detail == ObservationDetail::Fetched { tags(profile.tags())? } else { BTreeMap::new() })?;
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

pub(super) fn map_policy(
    ctx: &Context,
    policy: &Policy,
    detail: ObservationDetail,
) -> Result<PageBody, AwsScanError> {
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
        "policy_document_collected":false, "details_available":detail == ObservationDetail::Fetched,
        "observation_detail":if detail == ObservationDetail::Fetched {"get_policy"} else {"list_only"}
    }), if detail == ObservationDetail::Fetched { tags(policy.tags())? } else { BTreeMap::new() })?;
    resource.validate()?;
    Ok(single(resource, Vec::new()))
}
