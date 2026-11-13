//! roles responsibilities for iam.
use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn roles(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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
pub(super) async fn role_details(
    ctx: &Context,
    client: &Client,
    listed: &Role,
) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let scope = ctx.scope(ROLE, Some(listed.arn()), Source::Direct);
    let fallback = match map_role(ctx, listed, ObservationDetail::Listed) {
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
            map_role(ctx, role, ObservationDetail::Fetched)
        });
    observed(ctx, scope, result, Some(fallback)).await?;
    attached_policies(ctx, client, listed, &source).await
}

#[tracing::instrument(skip_all)]
pub(super) async fn attached_policies(
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
