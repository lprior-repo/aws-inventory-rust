use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn profiles(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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
pub(super) async fn profile_details(
    ctx: &Context,
    client: &Client,
    listed: &InstanceProfile,
) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let scope = ctx.scope(PROFILE, Some(listed.arn()), Source::Direct);
    let fallback = match map_profile(ctx, listed, ObservationDetail::Listed) {
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
            map_profile(ctx, value, ObservationDetail::Fetched)
        });
    observed(ctx, scope, result, Some(fallback)).await
}
