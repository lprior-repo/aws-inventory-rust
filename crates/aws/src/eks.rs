use std::collections::BTreeMap;

use aws_sdk_eks::{
    Client,
    types::{Cluster, Nodegroup},
};
use inventory_core::Source;
use serde_json::json;

use crate::{
    AwsScanError,
    common::{
        Context, PageBody, collect_bounded, error_category, paginate, require_budget, required,
        sdk_error,
    },
};

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope("aws_eks_cluster", None, Source::Direct),
        |token, budget| {
            let client = &client;
            async move {
                let output = ctx
                    .execute("eks", "read", || async {
                        client
                            .list_clusters()
                            .max_results(ctx.page_size.clamp(1, 100))
                            .set_next_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.clusters().len(), budget)?;
                let pages = collect_bounded(output.clusters().iter(), 4, |name| {
                    cluster(ctx, client, name)
                })
                .await?;
                let mut body = PageBody {
                    listed_count: output.clusters().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                };
                for mut page in pages {
                    body.resources.append(&mut page.resources);
                    body.edges.append(&mut page.edges);
                }
                Ok(body)
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(cluster_name = name))]
async fn cluster(ctx: &Context, client: &Client, name: &str) -> Result<PageBody, AwsScanError> {
    let output = ctx
        .execute("eks", "read", || async {
            client
                .describe_cluster()
                .name(name)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    let body = match output {
        Ok(output) => map_cluster(
            ctx,
            output
                .cluster()
                .ok_or_else(|| AwsScanError::InvalidData("DescribeCluster lacks cluster".into()))?,
        )?,
        Err(error) => {
            let error = AwsScanError::Api(error);
            ctx.failure(
                &ctx.scope("aws_eks_cluster_description", Some(name), Source::Direct),
                error_category(&error),
                error.to_string(),
            )
            .await?;
            PageBody::default()
        }
    };
    nodegroups(ctx, client, name).await?;
    Ok(body)
}

fn map_cluster(ctx: &Context, cluster: &Cluster) -> Result<PageBody, AwsScanError> {
    let vpc = cluster.resources_vpc_config();
    let tags: BTreeMap<String, String> = cluster
        .tags()
        .map(|tags| {
            tags.iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        })
        .unwrap_or_default();
    let resource = ctx.resource("aws_eks_cluster", required(cluster.arn(), "EKS cluster ARN")?, cluster.name().map(str::to_owned), json!({
        "name": cluster.name(), "arn": cluster.arn(), "status": cluster.status().map(|value|value.as_str()),
        "version": cluster.version(), "endpoint": cluster.endpoint(), "platform_version": cluster.platform_version(),
        "role_arn": cluster.role_arn(), "vpc_id": vpc.and_then(|vpc|vpc.vpc_id()),
        "subnet_ids": vpc.as_ref().map(|vpc| vpc.subnet_ids().to_vec()),
        "security_group_ids": vpc.as_ref().map(|vpc| vpc.security_group_ids().to_vec()),
        "endpoint_public_access": vpc.map(|vpc|vpc.endpoint_public_access()),
        "endpoint_private_access": vpc.map(|vpc|vpc.endpoint_private_access()),
        "public_access_cidrs": vpc.as_ref().map(|vpc| vpc.public_access_cidrs().to_vec()),
    }), tags)?;
    let mut body = PageBody::default();
    if let Some(role) = cluster.role_arn() {
        body.edges
            .push(ctx.edge(&resource, role.to_owned(), "ASSUMES_ROLE"));
    }
    if let Some(vpc) = vpc {
        if let Some(id) = vpc.vpc_id() {
            body.edges
                .push(ctx.edge(&resource, ctx.ec2_arn("vpc", id), "IN_VPC"));
        }
        for subnet in vpc.subnet_ids() {
            body.edges
                .push(ctx.edge(&resource, ctx.ec2_arn("subnet", subnet), "USES_SUBNET"));
        }
        for group in vpc.security_group_ids() {
            body.edges.push(ctx.edge(
                &resource,
                ctx.ec2_arn("security-group", group),
                "USES_SECURITY_GROUP",
            ));
        }
        if let Some(group) = vpc.cluster_security_group_id() {
            body.edges.push(ctx.edge(
                &resource,
                ctx.ec2_arn("security-group", group),
                "USES_SECURITY_GROUP",
            ));
        }
    }
    body.resources.push(resource);
    Ok(body)
}

#[tracing::instrument(skip_all)]
async fn nodegroups(ctx: &Context, client: &Client, cluster: &str) -> Result<(), AwsScanError> {
    let parent = ctx.arn("eks", &format!("cluster/{cluster}"));
    paginate(
        ctx,
        ctx.scope("aws_eks_node_group", Some(&parent), Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("eks", "read", || async {
                    client
                        .list_nodegroups()
                        .cluster_name(cluster)
                        .max_results(ctx.page_size.clamp(1, 100))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.nodegroups().len(), budget)?;
            let pages = collect_bounded(output.nodegroups().iter(), 8, |name| {
                nodegroup(ctx, client, cluster, name)
            })
            .await?;
            let mut body = PageBody {
                listed_count: output.nodegroups().len(),
                next_token: output.next_token().map(str::to_owned),
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

#[tracing::instrument(skip_all, fields(cluster_name = cluster, nodegroup_name = name))]
async fn nodegroup(
    ctx: &Context,
    client: &Client,
    cluster: &str,
    name: &str,
) -> Result<PageBody, AwsScanError> {
    let output = ctx
        .execute("eks", "read", || async {
            client
                .describe_nodegroup()
                .cluster_name(cluster)
                .nodegroup_name(name)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match output {
        Ok(output) => map_nodegroup(
            ctx,
            output.nodegroup().ok_or_else(|| {
                AwsScanError::InvalidData("DescribeNodegroup lacks nodegroup".into())
            })?,
        ),
        Err(error) => {
            let error = AwsScanError::Api(error);
            ctx.failure(
                &ctx.scope(
                    "aws_eks_node_group_description",
                    Some(&format!("{cluster}/{name}")),
                    Source::Direct,
                ),
                error_category(&error),
                error.to_string(),
            )
            .await?;
            Ok(PageBody::default())
        }
    }
}

fn map_nodegroup(ctx: &Context, nodegroup: &Nodegroup) -> Result<PageBody, AwsScanError> {
    let tags = nodegroup.tags().map_or_else(BTreeMap::new, |tags| {
        tags.iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    });
    let scaling = nodegroup.scaling_config();
    let resource = ctx.resource("aws_eks_node_group", required(nodegroup.nodegroup_arn(), "EKS node group ARN")?,
        nodegroup.nodegroup_name().map(str::to_owned), json!({
            "nodegroup_name": nodegroup.nodegroup_name(), "cluster_name": nodegroup.cluster_name(),
            "status": nodegroup.status().map(|value|value.as_str()), "version": nodegroup.version(),
            "release_version": nodegroup.release_version(), "instance_types": nodegroup.instance_types(),
            "node_role": nodegroup.node_role(), "subnets": nodegroup.subnets(), "disk_size": nodegroup.disk_size(),
            "capacity_type": nodegroup.capacity_type().map(|value|value.as_str()),
            "ami_type": nodegroup.ami_type().map(|value|value.as_str()),
            "desired_size": scaling.and_then(|value|value.desired_size()),
            "min_size": scaling.and_then(|value|value.min_size()), "max_size": scaling.and_then(|value|value.max_size()),
            "autoscaling_groups": nodegroup.resources().map(|resources| resources.auto_scaling_groups().iter().filter_map(|group| group.name()).collect::<Vec<_>>()),
        }), tags)?;
    let mut body = PageBody::default();
    if let Some(cluster) = nodegroup.cluster_name() {
        body.edges.push(ctx.edge(
            &resource,
            ctx.arn("eks", &format!("cluster/{cluster}")),
            "IN_CLUSTER",
        ));
    }
    if let Some(role) = nodegroup.node_role() {
        body.edges
            .push(ctx.edge(&resource, role.to_owned(), "ASSUMES_ROLE"));
    }
    for subnet in nodegroup.subnets() {
        body.edges
            .push(ctx.edge(&resource, ctx.ec2_arn("subnet", subnet), "USES_SUBNET"));
    }
    body.resources.push(resource);
    Ok(body)
}
