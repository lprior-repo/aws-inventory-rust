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

mod clusters;
pub(crate) use clusters::tags;
use clusters::*;

mod services;
use services::*;

mod service_mapping;
pub(crate) use service_mapping::merge_reference_edge;
use service_mapping::*;

mod tasks;
use tasks::*;

#[cfg(test)]
mod tests;
