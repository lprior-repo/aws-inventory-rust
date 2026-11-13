use std::collections::{BTreeMap, BTreeSet};

use aws_sdk_ecs::{
    Client,
    types::{Cluster, ClusterField, DesiredStatus, Service, ServiceField, Tag, Task, TaskField},
};
use inventory_core::{CollectionScope, Edge, Resource, Source};
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{
        Context, PageBody, collect_bounded, error_category, paginate, require_budget, required,
        sdk_error,
    },
};

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
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope("aws_ecs_cluster", None, Source::Direct),
        |token, budget| {
            let client = &client;
            async move {
                let list = ctx
                    .execute("ecs", "read", || async {
                        client
                            .list_clusters()
                            .max_results(ctx.page_size.clamp(1, 100))
                            .set_next_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(list.cluster_arns().len(), budget)?;
                let mut resources = Vec::new();
                if !list.cluster_arns().is_empty() {
                    resources = describe_clusters(ctx, client, list.cluster_arns()).await?;
                    collect_bounded(list.cluster_arns().iter(), 4, |arn| {
                        children(ctx, client, arn)
                    })
                    .await?;
                }
                Ok(PageBody {
                    resources,
                    listed_count: list.cluster_arns().len(),
                    next_token: list.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn describe_clusters(
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

fn map_cluster(ctx: &Context, cluster: &Cluster) -> Result<Resource, AwsScanError> {
    ctx.resource("aws_ecs_cluster", required(cluster.cluster_arn(), "ECS cluster ARN")?,
        cluster.cluster_name().map(str::to_owned), json!({
            "cluster_arn": cluster.cluster_arn(), "cluster_name": cluster.cluster_name(), "status": cluster.status(),
            "registered_container_instances_count": cluster.registered_container_instances_count(),
            "running_tasks_count": cluster.running_tasks_count(), "pending_tasks_count": cluster.pending_tasks_count(),
            "active_services_count": cluster.active_services_count(), "capacity_providers": cluster.capacity_providers(),
        }), tags(cluster.tags()))
}

#[tracing::instrument(skip_all)]
async fn children(ctx: &Context, client: &Client, cluster: &str) -> Result<(), AwsScanError> {
    tokio::try_join!(
        services(ctx, client, cluster),
        tasks(ctx, client, cluster, DesiredStatus::Running),
        tasks(ctx, client, cluster, DesiredStatus::Stopped)
    )?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn missing(
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

#[tracing::instrument(skip_all)]
async fn services(ctx: &Context, client: &Client, cluster: &str) -> Result<(), AwsScanError> {
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
async fn describe_services(
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

fn map_service(ctx: &Context, body: &mut PageBody, service: &Service) -> Result<(), AwsScanError> {
    let network = service
        .network_configuration()
        .and_then(|network| network.awsvpc_configuration());
    let components = service
        .load_balancers()
        .len()
        .saturating_add(network.map_or(0, |value| value.subnets().len()))
        .saturating_add(network.map_or(0, |value| value.security_groups().len()));
    if components > MAX_SERVICE_REFERENCES {
        return Err(AwsScanError::InvalidData(
            "ECS service reference capacity exceeded".into(),
        ));
    }
    let target_groups: BTreeSet<_> = service
        .load_balancers()
        .iter()
        .flat_map(|value| {
            [
                value.target_group_arn(),
                value
                    .advanced_configuration()
                    .and_then(|value| value.alternate_target_group_arn()),
            ]
        })
        .flatten()
        .collect();
    let classic_balancers: BTreeSet<_> = service
        .load_balancers()
        .iter()
        .filter_map(|value| value.load_balancer_name())
        .collect();
    let resource = ctx.resource("aws_ecs_service", required(service.service_arn(), "ECS service ARN")?,
        service.service_name().map(str::to_owned), json!({
            "service_arn": service.service_arn(), "service_name": service.service_name(), "cluster_arn": service.cluster_arn(),
            "status": service.status(), "task_definition": service.task_definition(), "desired_count": service.desired_count(),
            "running_count": service.running_count(), "pending_count": service.pending_count(),
            "launch_type": service.launch_type().map(|value|value.as_str()), "platform_version": service.platform_version(),
            "scheduling_strategy": service.scheduling_strategy().map(|value|value.as_str()),
            "subnet_ids": network.map(|network|network.subnets()).unwrap_or_default(),
            "security_group_ids": network.map(|network|network.security_groups()).unwrap_or_default(),
            "target_group_arns": target_groups, "classic_load_balancer_names": classic_balancers,
            "load_balancers": service.load_balancers().iter().map(|value| json!({
                "target_group_arn": value.target_group_arn(), "load_balancer_name": value.load_balancer_name(),
                "container_name": value.container_name(), "container_port": value.container_port(),
                "advanced_configuration": value.advanced_configuration().map(|value| json!({
                    "alternate_target_group_arn": value.alternate_target_group_arn(),
                    "production_listener_rule": value.production_listener_rule(),
                    "test_listener_rule": value.test_listener_rule(), "role_arn": value.role_arn(),
                })),
            })).collect::<Vec<_>>(),
        }), tags(service.tags()))?;
    resource.validate()?;
    let mut links = BTreeMap::new();
    if let Some(arn) = service.cluster_arn() {
        merge_reference_edge(
            &mut links,
            ctx.edge(&resource, arn.to_owned(), "IN_CLUSTER"),
            json!({"field":"cluster_arn"}),
        )?;
    }
    if let Some(arn) = service.task_definition() {
        merge_reference_edge(
            &mut links,
            ctx.edge(&resource, arn.to_owned(), "USES_TASK_DEFINITION"),
            json!({"field":"task_definition"}),
        )?;
    }
    if let Some(network) = network {
        for subnet in network.subnets() {
            merge_reference_edge(
                &mut links,
                ctx.edge(&resource, ctx.ec2_arn("subnet", subnet), "USES_SUBNET"),
                json!({"field":"subnet_ids"}),
            )?;
        }
        for group in network.security_groups() {
            merge_reference_edge(
                &mut links,
                ctx.edge(
                    &resource,
                    ctx.ec2_arn("security-group", group),
                    "USES_SECURITY_GROUP",
                ),
                json!({"field":"security_group_ids"}),
            )?;
        }
    }
    for (index, balancer) in service.load_balancers().iter().enumerate() {
        for (arn, role) in [
            (balancer.target_group_arn(), "primary"),
            (
                balancer
                    .advanced_configuration()
                    .and_then(|value| value.alternate_target_group_arn()),
                "alternate",
            ),
        ] {
            if let Some(arn) = arn {
                let edge = target_group_link(ctx, &resource, arn)?;
                merge_reference_edge(
                    &mut links,
                    edge,
                    json!({
                        "load_balancer_index":index, "configuration_role":role,
                        "container_name":balancer.container_name(), "container_port":balancer.container_port(),
                    }),
                )?;
            }
        }
        if let Some(name) = balancer.load_balancer_name() {
            let name = required(Some(name), "Classic ELB name")?;
            let mut edge = ctx.edge(
                &resource,
                ctx.arn("elasticloadbalancing", &format!("loadbalancer/{name}")),
                "USES_CLASSIC_LOAD_BALANCER",
            );
            edge.properties = json!({"target_resource_id":name,"target_kind":"aws_ec2_classic_load_balancer",
                "target_account_id":ctx.account,"target_region":ctx.region,"owner_resolution":"assumed_scanning_account"});
            merge_reference_edge(
                &mut links,
                edge,
                json!({"load_balancer_index":index,
                "container_name":balancer.container_name(),"container_port":balancer.container_port()}),
            )?;
        }
    }
    for edge in links.into_values() {
        edge.validate()?;
        body.edges.push(edge);
    }
    body.resources.push(resource);
    Ok(())
}

const MAX_SERVICE_REFERENCES: usize = 1_024;

/// One relationship can carry several container/port registrations. Retain the
/// observations while matching the store's (from,to,relation) identity.
pub(crate) fn merge_reference_edge(
    links: &mut BTreeMap<(String, String, String), Edge>,
    mut edge: Edge,
    evidence: Value,
) -> Result<(), AwsScanError> {
    let key = (
        edge.from_id.clone(),
        edge.to_id.clone(),
        edge.relation.clone(),
    );
    if let Some(existing) = links.get_mut(&key) {
        let observations = existing
            .properties
            .get_mut("evidence")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                AwsScanError::InvalidData("resource edge evidence is not an array".into())
            })?;
        if observations.len() >= MAX_SERVICE_REFERENCES {
            return Err(AwsScanError::InvalidData(
                "resource edge evidence capacity exceeded".into(),
            ));
        }
        observations.push(evidence);
    } else {
        let properties = edge.properties.as_object_mut().ok_or_else(|| {
            AwsScanError::InvalidData("resource edge properties are not an object".into())
        })?;
        properties.insert("evidence".into(), json!([evidence]));
        links.insert(key, edge);
    }
    Ok(())
}

fn target_group_link(ctx: &Context, resource: &Resource, arn: &str) -> Result<Edge, AwsScanError> {
    let mut parts = arn.splitn(6, ':');
    let (
        Some("arn"),
        Some(partition),
        Some("elasticloadbalancing"),
        Some(region),
        Some(account),
        Some(path),
    ) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    )
    else {
        return Err(AwsScanError::InvalidData(
            "ECS load balancer target group has an invalid ARN".into(),
        ));
    };
    if partition.is_empty()
        || region.is_empty()
        || account.is_empty()
        || !path
            .strip_prefix("targetgroup/")
            .and_then(|value| value.split_once('/'))
            .is_some_and(|(name, id)| !name.is_empty() && !id.is_empty())
    {
        return Err(AwsScanError::InvalidData(
            "ECS load balancer target group has an invalid ARN".into(),
        ));
    }
    let mut edge = ctx.edge(resource, arn.to_owned(), "USES_TARGET_GROUP");
    edge.properties = json!({"target_resource_id":arn,"target_kind":"aws_elbv2_target_group",
        "target_account_id":account,"target_region":region,"owner_resolution":"aws_returned_arn"});
    Ok(edge)
}

#[tracing::instrument(skip_all)]
async fn tasks(
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

fn map_task(ctx: &Context, body: &mut PageBody, task: &Task) -> Result<(), AwsScanError> {
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
async fn describe_failure(
    ctx: &Context,
    scope: &CollectionScope,
    error: AwsScanError,
) -> Result<(), AwsScanError> {
    ctx.failure(scope, error_category(&error), error.to_string())
        .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use aws_sdk_ecs::types::{
        AdvancedConfiguration, AwsVpcConfiguration, LoadBalancer, NetworkConfiguration,
    };
    use inventory_core::CollectionEvent;
    use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
    use tokio::sync::mpsc;

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    const SERVICE: &str = "arn:aws:ecs:us-east-1:111111111111:service/demo/web";
    const GROUP: &str =
        "arn:aws:elasticloadbalancing:us-west-2:222222222222:targetgroup/web/0123456789abcdef";
    const DEFINITION: &str = "arn:aws:ecs:us-east-1:111111111111:task-definition/web:7";

    fn context() -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
        let (sender, receiver) = mpsc::channel(8);
        Ok((
            Context {
                config: aws_types::SdkConfig::builder().build(),
                executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
                sender,
                account: "111111111111".into(),
                partition: "aws".into(),
                region: "us-east-1".into(),
                max_pages: 3,
                page_size: 100,
            },
            receiver,
        ))
    }

    #[test]
    fn service_groups_aggregate_primary_alternate_and_container_evidence() -> TestResult {
        let (ctx, _receiver) = context()?;
        let service = Service::builder()
            .service_arn(SERVICE)
            .service_name("web")
            .load_balancers(
                LoadBalancer::builder()
                    .target_group_arn(GROUP)
                    .container_name("web")
                    .container_port(8080)
                    .advanced_configuration(
                        AdvancedConfiguration::builder()
                            .alternate_target_group_arn(GROUP)
                            .production_listener_rule("arn:aws:elasticloadbalancing:us-west-2:222222222222:listener-rule/app/demo/lb/listener/rule")
                            .role_arn("arn:aws:iam::222222222222:role/deploy")
                            .build(),
                    )
                    .build(),
            )
            .load_balancers(
                LoadBalancer::builder()
                    .target_group_arn(GROUP)
                    .container_name("admin")
                    .container_port(9090)
                    .build(),
            )
            .build();
        let mut body = PageBody::default();
        map_service(&ctx, &mut body, &service)?;
        assert_eq!(body.edges.len(), 1);
        let edge = body.edges.first().ok_or("missing target group edge")?;
        assert_eq!(edge.to_id, GROUP);
        assert_eq!(edge.relation, "USES_TARGET_GROUP");
        assert_eq!(
            edge.properties.get("target_account_id"),
            Some(&json!("222222222222"))
        );
        assert_eq!(
            edge.properties.get("target_region"),
            Some(&json!("us-west-2"))
        );
        assert_eq!(
            edge.properties.get("owner_resolution"),
            Some(&json!("aws_returned_arn"))
        );
        assert_eq!(
            edge.properties.pointer("/evidence/1/configuration_role"),
            Some(&json!("alternate"))
        );
        assert_eq!(
            edge.properties.pointer("/evidence/2/container_port"),
            Some(&json!(9090))
        );
        let resource = body.resources.first().ok_or("missing ECS service")?;
        assert_eq!(
            resource.properties.get("target_group_arns"),
            Some(&json!([GROUP]))
        );
        assert_eq!(
            resource
                .properties
                .pointer("/load_balancers/0/advanced_configuration/role_arn"),
            Some(&json!("arn:aws:iam::222222222222:role/deploy"))
        );
        Ok(())
    }

    #[test]
    fn classic_name_is_a_distinct_unobserved_reference_with_owner_assumption() -> TestResult {
        let (ctx, _receiver) = context()?;
        let balancer = LoadBalancer::builder()
            .load_balancer_name("legacy-web")
            .container_name("web")
            .container_port(80)
            .build();
        let service = Service::builder()
            .service_arn(SERVICE)
            .load_balancers(balancer.clone())
            .load_balancers(balancer)
            .build();
        let mut body = PageBody::default();
        map_service(&ctx, &mut body, &service)?;
        assert_eq!(body.resources.len(), 1);
        assert_eq!(body.edges.len(), 1);
        let edge = body.edges.first().ok_or("missing Classic ELB edge")?;
        assert_eq!(edge.relation, "USES_CLASSIC_LOAD_BALANCER");
        assert_eq!(
            edge.to_id,
            "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/legacy-web"
        );
        assert_eq!(
            edge.properties.get("target_kind"),
            Some(&json!("aws_ec2_classic_load_balancer"))
        );
        assert_eq!(
            edge.properties.get("owner_resolution"),
            Some(&json!("assumed_scanning_account"))
        );
        assert_eq!(
            edge.properties
                .get("evidence")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        Ok(())
    }

    #[test]
    fn service_network_duplicates_do_not_duplicate_edges_and_revision_is_retained() -> TestResult {
        let (ctx, _receiver) = context()?;
        let service = Service::builder()
            .service_arn(SERVICE)
            .cluster_arn("arn:aws:ecs:us-east-1:111111111111:cluster/demo")
            .task_definition(DEFINITION)
            .network_configuration(
                NetworkConfiguration::builder()
                    .awsvpc_configuration(
                        AwsVpcConfiguration::builder()
                            .subnets("subnet-one")
                            .subnets("subnet-one")
                            .security_groups("sg-one")
                            .security_groups("sg-one")
                            .build()?,
                    )
                    .build(),
            )
            .build();
        let mut body = PageBody::default();
        map_service(&ctx, &mut body, &service)?;
        assert_eq!(body.edges.len(), 4);
        assert!(
            body.edges
                .iter()
                .any(|edge| edge.relation == "USES_TASK_DEFINITION" && edge.to_id == DEFINITION)
        );
        let subnet = body
            .edges
            .iter()
            .find(|edge| edge.relation == "USES_SUBNET")
            .ok_or("missing subnet")?;
        assert_eq!(
            subnet.properties.get("owner_resolution"),
            Some(&json!("assumed_scanning_account"))
        );
        assert_eq!(
            subnet
                .properties
                .get("evidence")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        Ok(())
    }

    #[test]
    fn malformed_service_load_balancer_references_do_not_emit_resources() -> TestResult {
        let (ctx, _receiver) = context()?;
        for balancer in [
            LoadBalancer::builder().target_group_arn("web").build(),
            LoadBalancer::builder()
                .target_group_arn(
                    "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/web/id",
                )
                .build(),
            LoadBalancer::builder().load_balancer_name("").build(),
        ] {
            let service = Service::builder()
                .service_arn(SERVICE)
                .load_balancers(balancer)
                .build();
            let mut body = PageBody::default();
            assert!(map_service(&ctx, &mut body, &service).is_err());
            assert!(body.resources.is_empty());
        }
        Ok(())
    }

    #[test]
    fn service_reference_bound_precedes_mapping() -> TestResult {
        let (ctx, _receiver) = context()?;
        let service = Service::builder()
            .service_arn(SERVICE)
            .set_load_balancers(Some(vec![
                LoadBalancer::builder().build();
                MAX_SERVICE_REFERENCES.saturating_add(1)
            ]))
            .build();
        let mut body = PageBody::default();
        assert!(matches!(
            map_service(&ctx, &mut body, &service),
            Err(AwsScanError::InvalidData(_))
        ));
        assert!(body.resources.is_empty() && body.edges.is_empty());
        Ok(())
    }
}
