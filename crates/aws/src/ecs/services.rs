//! services responsibilities for ecs.
use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn services(
    ctx: &Context,
    client: &Client,
    cluster: &str,
) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_ecs_service", Some(cluster), Source::Direct),
        |token, budget| async move {
            let list = ctx
                .execute("ecs", "read", || async {
                    client
                        .list_services()
                        .cluster(cluster)
                        .max_results(ctx.page_size.clamp(1, 100))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(list.service_arns().len(), budget)?;
            let pages = collect_bounded(list.service_arns().chunks(10), 4, |arns| {
                describe_services(ctx, client, cluster, arns)
            })
            .await?;
            let mut body = PageBody {
                listed_count: list.service_arns().len(),
                next_token: list.next_token().map(str::to_owned),
                ..PageBody::default()
            };
            for mut page in pages {
                body.resources.append(&mut page.resources);
                body.edges.append(&mut page.edges);
            }
            Ok(body)
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn describe_services(
    ctx: &Context,
    client: &Client,
    cluster: &str,
    arns: &[String],
) -> Result<PageBody, AwsScanError> {
    let output = ctx
        .execute("ecs", "read", || async {
            client
                .describe_services()
                .cluster(cluster)
                .set_services(Some(arns.to_vec()))
                .include(ServiceField::Tags)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match output {
        Ok(output) => {
            let returned = output
                .services()
                .iter()
                .filter_map(|service| service.service_arn())
                .collect();
            missing(ctx, "aws_ecs_service_description", arns, &returned).await?;
            let mut body = PageBody::default();
            for service in output.services() {
                map_service(ctx, &mut body, service)?;
            }
            Ok(body)
        }
        Err(error) => {
            describe_failure(
                ctx,
                &ctx.scope("aws_ecs_service_description", Some(cluster), Source::Direct),
                error.into(),
            )
            .await?;
            Ok(PageBody::default())
        }
    }
}
