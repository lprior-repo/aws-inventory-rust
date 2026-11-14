use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn policies(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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
pub(super) async fn policy_details(
    ctx: &Context,
    client: &Client,
    listed: &Policy,
) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let arn = required(listed.arn(), "managed policy ARN")?;
    let scope = ctx.scope(POLICY, Some(&arn), Source::Direct);
    let fallback = match map_policy(ctx, listed, ObservationDetail::Listed) {
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
            map_policy(ctx, value, ObservationDetail::Fetched)
        });
    observed(ctx, scope, result, Some(fallback)).await
}
