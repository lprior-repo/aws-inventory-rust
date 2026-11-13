//! clusters responsibilities for ecs.
use super::*;

pub(crate) fn tags(values: &[Tag]) -> BTreeMap<String, String> {
    values
        .iter()
        .filter_map(|tag| {
            Some((
                tag.key()?.to_owned(),
                tag.value().unwrap_or_default().to_owned(),
            ))
        })
        .collect()
}

#[tracing::instrument(skip_all)]
pub(super) async fn describe_clusters(
    ctx: &Context,
    client: &Client,
    arns: &[String],
) -> Result<Vec<Resource>, AwsScanError> {
    let output = ctx
        .execute("ecs", "read", || async {
            client
                .describe_clusters()
                .set_clusters(Some(arns.to_vec()))
                .include(ClusterField::Tags)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match output {
        Ok(output) => {
            let returned = output
                .clusters()
                .iter()
                .filter_map(|cluster| cluster.cluster_arn())
                .collect();
            missing(ctx, "aws_ecs_cluster_description", arns, &returned).await?;
            output
                .clusters()
                .iter()
                .map(|cluster| map_cluster(ctx, cluster))
                .collect()
        }
        Err(error) => {
            describe_failure(
                ctx,
                &ctx.scope("aws_ecs_cluster_description", None, Source::Direct),
                error.into(),
            )
            .await?;
            Ok(Vec::new())
        }
    }
}

pub(super) fn map_cluster(ctx: &Context, cluster: &Cluster) -> Result<Resource, AwsScanError> {
    ctx.resource("aws_ecs_cluster", required(cluster.cluster_arn(), "ECS cluster ARN")?,
        cluster.cluster_name().map(str::to_owned), json!({
            "cluster_arn": cluster.cluster_arn(), "cluster_name": cluster.cluster_name(), "status": cluster.status(),
            "registered_container_instances_count": cluster.registered_container_instances_count(),
            "running_tasks_count": cluster.running_tasks_count(), "pending_tasks_count": cluster.pending_tasks_count(),
            "active_services_count": cluster.active_services_count(), "capacity_providers": cluster.capacity_providers(),
        }), tags(cluster.tags()))
}

#[tracing::instrument(skip_all)]
pub(super) async fn children(
    ctx: &Context,
    client: &Client,
    cluster: &str,
) -> Result<(), AwsScanError> {
    tokio::try_join!(
        services(ctx, client, cluster),
        tasks(ctx, client, cluster, DesiredStatus::Running),
        tasks(ctx, client, cluster, DesiredStatus::Stopped)
    )?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn missing(
    ctx: &Context,
    collector: &str,
    requested: &[String],
    returned: &BTreeSet<&str>,
) -> Result<(), AwsScanError> {
    collect_bounded(requested.iter().filter(|arn|!returned.contains(arn.as_str())), 8, |arn| async move {
        ctx.failure(&ctx.scope(collector, Some(arn), Source::Direct), "describe_missing",
            "resource enumerated by List but absent from Describe response; it may have changed or access may be restricted").await
    }).await?;
    Ok(())
}
