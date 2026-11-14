use std::collections::{BTreeMap, BTreeSet};

use aws_sdk_cloudfront::{
    Client,
    types::{
        CacheBehavior, DefaultCacheBehavior, DistributionSummary, FunctionAssociation,
        LambdaFunctionAssociation, Method, Origin,
    },
};
use inventory_core::{Edge, Resource, Source};
use serde_json::{Value, json};

use super::{
    CHILD_CONCURRENCY, MAX_COMPONENTS, SeenIds, checked_text, components, dns_reference, edge_set,
    ensure_active, marker, observed, own_arn, reference, section, single,
};
use crate::{
    AwsScanError,
    common::{Context, PageBody, collect_bounded, paginate, sdk_error},
};

const KIND: &str = "aws_cloudfront_distribution";

#[tracing::instrument(skip_all)]
pub(super) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    if ctx.partition != "aws" {
        return ctx
            .failure(
                &ctx.scope("aws_cloudfront_distribution_list", None, Source::Direct),
                "unsupported_partition",
                "CloudFront collector has a reviewed endpoint policy only for the aws partition",
            )
            .await;
    }
    let client = Client::new(&ctx.config);
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_cloudfront_distribution_list", None, Source::Direct),
        |token, budget| {
            let client = &client;
            let seen = &seen;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("cloudfront", "read", || async {
                        client
                            .list_distributions()
                            .set_marker(token.clone())
                            .max_items(ctx.page_size.clamp(1, 100))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                let list = output.distribution_list().ok_or_else(|| {
                    AwsScanError::InvalidData("ListDistributions lacks list".into())
                })?;
                let count =
                    seen.record(list.items().iter().map(DistributionSummary::arn), budget)?;
                let next_token = marker(list.is_truncated(), list.next_marker())?;
                collect_bounded(list.items(), CHILD_CONCURRENCY, |value| {
                    collect_distribution(ctx, client, value)
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
    .await?;
    ensure_active(ctx)
}

#[tracing::instrument(skip_all)]
async fn collect_distribution(
    ctx: &Context,
    client: &Client,
    value: &DistributionSummary,
) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let scope = ctx.scope(KIND, Some(value.arn()), Source::Direct);
    let mut mapped = match map_distribution(ctx, value, None) {
        Ok(body) => body,
        Err(error) => return observed(ctx, scope, Err(error), None).await,
    };
    let tags = distribution_tags(ctx, client, value.arn()).await?;
    if let Some(resource) = mapped.resources.first_mut() {
        if let Some(properties) = resource.properties.as_object_mut() {
            properties.insert("tags_available".to_owned(), json!(tags.is_some()));
        }
        resource.tags = tags.unwrap_or_default();
    }
    observed(ctx, scope, Ok(mapped), None).await
}

#[tracing::instrument(skip_all)]
async fn distribution_tags(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    let (_, value) = section(ctx, arn, "aws_cloudfront_distribution_tags", || async {
        let output = ctx
            .execute("cloudfront", "read", || async {
                client
                    .list_tags_for_resource()
                    .resource(arn)
                    .send()
                    .await
                    .map_err(sdk_error)
            })
            .await?;
        let tags = output.tags().ok_or_else(|| {
            AwsScanError::InvalidData("CloudFront tag response lacks tags".into())
        })?;
        components(tags.items().len(), "CloudFront tags")?;
        let mapped = tags
            .items()
            .iter()
            .map(|tag| {
                Ok((
                    checked_text(tag.key(), "CloudFront tag key")?,
                    tag.value().unwrap_or("").to_owned(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>, AwsScanError>>()?;
        Ok(json!(mapped))
    })
    .await?;
    value
        .map(|value| {
            serde_json::from_value(value)
                .map_err(|_| AwsScanError::InvalidData("invalid mapped CloudFront tags".into()))
        })
        .transpose()
}

mod mapping;
use mapping::*;

mod behavior;
use behavior::*;

mod references;
use references::*;

#[cfg(test)]
mod tests;
