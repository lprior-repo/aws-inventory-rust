//! collection operations.
use super::*;

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    // These roots are independent: unattached target groups must also be inventoried.
    collect_bounded([false, true], 2, |groups| {
        let client = &client;
        async move {
            if groups {
                target_groups(ctx, client).await
            } else {
                load_balancers(ctx, client).await
            }
        }
    })
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn load_balancers(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_elbv2_load_balancer", None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("elasticloadbalancing", "read", || async {
                    client
                        .describe_load_balancers()
                        .page_size(ctx.page_size.clamp(1, 400))
                        .set_marker(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            let body = map_page(
                output.load_balancers(),
                budget,
                output.next_marker(),
                |model| map_load_balancer(ctx, model),
            )?;
            collect_bounded(
                output.load_balancers().iter(),
                CHILD_CONCURRENCY,
                |model| async move {
                    let arn = required(model.load_balancer_arn(), "load balancer ARN")?;
                    listeners(ctx, client, &arn, application_rules(model)).await
                },
            )
            .await?;
            Ok(body)
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(load_balancer_arn))]
pub(super) async fn listeners(
    ctx: &Context,
    client: &Client,
    load_balancer_arn: &str,
    collect_rules: bool,
) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(
            "aws_elbv2_listener",
            Some(load_balancer_arn),
            Source::Direct,
        ),
        |token, budget| async move {
            let output = ctx
                .execute("elasticloadbalancing", "read", || async {
                    client
                        .describe_listeners()
                        .load_balancer_arn(load_balancer_arn)
                        .page_size(ctx.page_size.clamp(1, 400))
                        .set_marker(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            let body = map_page(output.listeners(), budget, output.next_marker(), |model| {
                map_listener(ctx, load_balancer_arn, model)
            })?;
            if collect_rules {
                collect_bounded(
                    output.listeners().iter(),
                    CHILD_CONCURRENCY,
                    |model| async move {
                        let arn = required(model.listener_arn(), "listener ARN")?;
                        rules(ctx, client, &arn).await
                    },
                )
                .await?;
            }
            Ok(body)
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(listener_arn))]
pub(super) async fn rules(
    ctx: &Context,
    client: &Client,
    listener_arn: &str,
) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_elbv2_rule", Some(listener_arn), Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("elasticloadbalancing", "read", || async {
                    client
                        .describe_rules()
                        .listener_arn(listener_arn)
                        .page_size(ctx.page_size.clamp(1, 400))
                        .set_marker(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            map_page(output.rules(), budget, output.next_marker(), |model| {
                map_rule(ctx, listener_arn, model)
            })
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn target_groups(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_elbv2_target_group", None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("elasticloadbalancing", "read", || async {
                    client
                        .describe_target_groups()
                        .page_size(ctx.page_size.clamp(1, 400))
                        .set_marker(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            let body = map_page(
                output.target_groups(),
                budget,
                output.next_marker(),
                |model| map_target_group(ctx, model),
            )?;
            collect_bounded(
                output.target_groups().iter(),
                CHILD_CONCURRENCY,
                |model| async move {
                    let arn = required(model.target_group_arn(), "target group ARN")?;
                    let target_type =
                        required(model.target_type().map(|kind| kind.as_str()), "target type")?;
                    target_health_with(ctx, &arn, &target_type, || {
                        ctx.execute("elasticloadbalancing", "read", || async {
                            client
                                .describe_target_health()
                                .target_group_arn(&arn)
                                .send()
                                .await
                                .map_err(sdk_error)
                        })
                    })
                    .await
                },
            )
            .await?;
            Ok(body)
        },
    )
    .await
}

/// DescribeTargetHealth has no pagination. Reuse the scope driver so a failed
/// request never becomes an empty successful result, including in injected tests.
#[tracing::instrument(skip_all, fields(target_group_arn, target_type))]
pub(super) async fn target_health_with<F, Fut>(
    ctx: &Context,
    target_group_arn: &str,
    target_type: &str,
    fetch: F,
) -> Result<(), AwsScanError>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = Result<DescribeTargetHealthOutput, ApiError>> + Send,
{
    paginate(
        ctx,
        ctx.scope(
            "aws_elbv2_target_health",
            Some(target_group_arn),
            Source::Direct,
        ),
        |token, budget| {
            let fetch = &fetch;
            async move {
                if token.is_some() {
                    return Err(AwsScanError::InvalidData(
                        "DescribeTargetHealth does not support continuation tokens".into(),
                    ));
                }
                let output = fetch().await?;
                map_page(output.target_health_descriptions(), budget, None, |model| {
                    map_target_health(ctx, target_group_arn, target_type, model)
                })
            }
        },
    )
    .await
}

pub(super) fn application_rules(model: &LoadBalancer) -> bool {
    model
        .r#type()
        .is_some_and(|kind| kind.as_str() == "application")
}
