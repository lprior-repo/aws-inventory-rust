use super::*;

pub(super) fn returned_arn_edge(
    ctx: &Context,
    resource: &Resource,
    arn: &str,
    relation: &str,
    kind: &str,
) -> Result<Edge, AwsScanError> {
    let parts = arn_parts(arn)?;
    if kind == "aws_iam_role"
        && (parts.service != "iam"
            || !parts.region.is_empty()
            || !parts
                .resource
                .strip_prefix("role/")
                .is_some_and(|name| !name.is_empty()))
    {
        return Err(AwsScanError::InvalidData(
            "task-definition role is not an IAM role ARN".into(),
        ));
    }
    if kind == "aws_s3files_file_system"
        && (parts.service != "s3files"
            || parts.region.is_empty()
            || !parts
                .resource
                .strip_prefix("file-system/fs-")
                .is_some_and(|id| !id.is_empty()))
    {
        return Err(AwsScanError::InvalidData(
            "task-definition S3 Files volume is not a file-system ARN".into(),
        ));
    }
    let mut edge = ctx.edge(resource, arn.to_owned(), relation);
    edge.properties = json!({"target_resource_id":arn,"target_kind":kind,"target_account_id":parts.account,
        "target_region":parts.region,"owner_resolution":"aws_returned_arn"});
    Ok(edge)
}

pub(super) fn file_system_edge(
    ctx: &Context,
    resource: &Resource,
    id: &str,
    service: &str,
    relation: &str,
    kind: &str,
) -> Result<Edge, AwsScanError> {
    let id = required(Some(id), "volume file-system ID")?;
    let mut edge = ctx.edge(
        resource,
        ctx.arn(service, &format!("file-system/{id}")),
        relation,
    );
    edge.properties = json!({"target_resource_id":id,"target_kind":kind,"target_account_id":ctx.account,
        "target_region":ctx.region,"owner_resolution":"assumed_scanning_account"});
    Ok(edge)
}
