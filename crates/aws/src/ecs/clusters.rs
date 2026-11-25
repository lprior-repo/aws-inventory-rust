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
        .await?;
    let returned = output
        .clusters()
        .iter()
        .filter_map(|cluster| cluster.cluster_arn())
        .collect();
    unresolved(
        ctx,
        "aws_ecs_cluster_description",
        arns,
        &returned,
        output.failures(),
    )
    .await?;
    output
        .clusters()
        .iter()
        .map(|cluster| map_cluster(ctx, cluster))
        .collect()
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

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Resolution {
    pub(super) missing: Vec<String>,
    pub(super) unavailable: Vec<(String, String)>,
}

pub(super) fn resolve(
    requested: &[String],
    returned: &BTreeSet<&str>,
    failures: &[Failure],
) -> Resolution {
    let mut reasons: BTreeMap<&str, (Option<&str>, Option<&str>)> = BTreeMap::new();
    for failure in failures {
        if let Some(arn) = failure.arn() {
            reasons
                .entry(arn)
                .or_insert((failure.reason(), failure.detail()));
        }
    }
    let unreturned: BTreeSet<&str> = requested
        .iter()
        .map(String::as_str)
        .filter(|arn| !returned.contains(arn))
        .collect();
    let mut resolution = Resolution::default();
    for arn in unreturned {
        match reasons.get(arn).copied().unwrap_or((None, None)) {
            (None | Some("MISSING"), _) => resolution.missing.push(arn.to_owned()),
            (Some(reason), detail) => resolution.unavailable.push((
                arn.to_owned(),
                format!("{reason}: {}", detail.unwrap_or("no detail returned")),
            )),
        }
    }
    resolution
}

#[tracing::instrument(skip_all)]
pub(super) async fn unresolved(
    ctx: &Context,
    collector: &str,
    requested: &[String],
    returned: &BTreeSet<&str>,
    failures: &[Failure],
) -> Result<(), AwsScanError> {
    let resolution = resolve(requested, returned, failures);
    collect_bounded(resolution.missing.iter(), 8, |arn| async move {
        ctx.failure(&ctx.scope(collector, Some(arn), Source::Direct), "describe_missing",
            "resource enumerated by List but absent from Describe response; it may have changed or access may be restricted").await
    }).await?;
    let Some((first, reason)) = resolution.unavailable.first() else {
        return Ok(());
    };
    let message = format!(
        "{collector}: {} requested resource(s) not described; first {first} reported {reason}",
        resolution.unavailable.len()
    );
    collect_bounded(
        resolution.unavailable.iter(),
        8,
        |(arn, reason)| async move {
            ctx.failure(
                &ctx.scope(collector, Some(arn), Source::Direct),
                "describe_failed",
                reason.as_str(),
            )
            .await
        },
    )
    .await?;
    Err(AwsScanError::Api(ApiError::Transient(message)))
}
