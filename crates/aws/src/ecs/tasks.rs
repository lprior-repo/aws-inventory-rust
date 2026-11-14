use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn tasks(
    ctx: &Context,
    client: &Client,
    cluster: &str,
    desired: DesiredStatus,
) -> Result<(), AwsScanError> {
    let parent = format!("{cluster}#desired={}", desired.as_str());
    paginate(
        ctx,
        ctx.scope("aws_ecs_task", Some(&parent), Source::Direct),
        |token, budget| {
            let desired = desired.clone();
            async move {
                let list = ctx
                    .execute("ecs", "read", || async {
                        client
                            .list_tasks()
                            .cluster(cluster)
                            .desired_status(desired.clone())
                            .max_results(ctx.page_size.clamp(1, 100))
                            .set_next_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(list.task_arns().len(), budget)?;
                let mut body = PageBody {
                    listed_count: list.task_arns().len(),
                    next_token: list.next_token().map(str::to_owned),
                    ..PageBody::default()
                };
                if !list.task_arns().is_empty() {
                    let described = ctx
                        .execute("ecs", "read", || async {
                            client
                                .describe_tasks()
                                .cluster(cluster)
                                .set_tasks(Some(list.task_arns().to_vec()))
                                .include(TaskField::Tags)
                                .send()
                                .await
                                .map_err(sdk_error)
                        })
                        .await;
                    match described {
                        Ok(output) => {
                            let returned = output
                                .tasks()
                                .iter()
                                .filter_map(|task| task.task_arn())
                                .collect();
                            missing(ctx, "aws_ecs_task_description", list.task_arns(), &returned)
                                .await?;
                            for task in output.tasks() {
                                map_task(ctx, &mut body, task)?;
                            }
                        }
                        Err(error) => {
                            describe_failure(
                                ctx,
                                &ctx.scope(
                                    "aws_ecs_task_description",
                                    Some(cluster),
                                    Source::Direct,
                                ),
                                error.into(),
                            )
                            .await?
                        }
                    }
                }
                Ok(body)
            }
        },
    )
    .await
}

pub(super) fn map_task(
    ctx: &Context,
    body: &mut PageBody,
    task: &Task,
) -> Result<(), AwsScanError> {
    let resource = ctx.resource("aws_ecs_task", required(task.task_arn(), "ECS task ARN")?, None, json!({
        "task_arn": task.task_arn(), "cluster_arn": task.cluster_arn(), "task_definition_arn": task.task_definition_arn(),
        "last_status": task.last_status(), "desired_status": task.desired_status(),
        "launch_type": task.launch_type().map(|value|value.as_str()), "cpu": task.cpu(), "memory": task.memory(),
        "availability_zone": task.availability_zone(), "group": task.group(),
        "container_instance_arn": task.container_instance_arn(), "stop_code": task.stop_code().map(|value|value.as_str()),
        "stopped_reason": task.stopped_reason(),
        "containers": task.containers().iter().map(|container|json!({"name":container.name(),"container_arn":container.container_arn(),"image":container.image(),"last_status":container.last_status(),"runtime_id":container.runtime_id()})).collect::<Vec<_>>(),
    }), tags(task.tags()))?;
    if let Some(arn) = task.cluster_arn() {
        body.edges
            .push(ctx.edge(&resource, arn.to_owned(), "IN_CLUSTER"));
    }
    if let Some(arn) = task.task_definition_arn() {
        body.edges
            .push(ctx.edge(&resource, arn.to_owned(), "USES_TASK_DEFINITION"));
    }
    if let Some(arn) = task.container_instance_arn() {
        body.edges
            .push(ctx.edge(&resource, arn.to_owned(), "RUNS_ON"));
    }
    for attachment in task.attachments() {
        for detail in attachment.details() {
            if let (Some(name), Some(value)) = (detail.name(), detail.value()) {
                let link = match name {
                    "networkInterfaceId" => Some(("network-interface", "USES_NETWORK_INTERFACE")),
                    "subnetId" => Some(("subnet", "IN_SUBNET")),
                    _ => None,
                };
                if let Some((kind, relation)) = link {
                    body.edges
                        .push(ctx.edge(&resource, ctx.ec2_arn(kind, value), relation));
                }
            }
        }
    }
    body.resources.push(resource);
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn describe_failure(
    ctx: &Context,
    scope: &CollectionScope,
    error: AwsScanError,
) -> Result<(), AwsScanError> {
    ctx.failure(scope, error_category(&error), error.to_string())
        .await
}
