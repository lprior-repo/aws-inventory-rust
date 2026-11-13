//! General-purpose buckets and selected control-plane configuration. No object
//! listing, object reads, raw bucket policy, ACL grants or credential material.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

use aws_sdk_s3::{
    Client,
    error::{ProvideErrorMetadata, SdkError},
    types::Bucket,
};
use aws_smithy_types::retry::ProvideErrorKind;
use aws_types::region::Region;
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::{Edge, Resource, Source};
use inventory_engine::limiter::ApiError;
use serde_json::{Value, json};

use super::{
    CHILD_CONCURRENCY, MAX_COMPONENTS, SeenIds, checked_text, components, dns_reference, edge_set,
    ensure_active, observed, reference, section, single,
};
use crate::{
    AwsScanError,
    common::{Context, PageBody, collect_bounded, paginate, require_budget, required, sdk_error},
};

const KIND: &str = "aws_s3_bucket";
const SECTIONS: [&str; 9] = [
    "versioning",
    "encryption",
    "public_access_block",
    "policy_status",
    "logging",
    "replication",
    "website",
    "ownership_controls",
    "notifications",
];

#[tracing::instrument(skip_all)]
pub(super) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let client = Client::new(&ctx.config);
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_s3_bucket_list", None, Source::Direct),
        |token, budget| {
            let seen = &seen;
            let client = &client;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("s3", "read", || async {
                        client
                            .list_buckets()
                            .max_buckets(ctx.page_size.clamp(1, 1_000))
                            .set_continuation_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.buckets().len(), budget)?;
                let names = output
                    .buckets()
                    .iter()
                    .map(|bucket| required(bucket.name(), "bucket name"))
                    .collect::<Result<Vec<_>, _>>()?;
                let count = seen.record(names.iter().map(String::as_str), budget)?;
                collect_bounded(output.buckets(), CHILD_CONCURRENCY, |bucket| {
                    collect_bucket(ctx, client, bucket)
                })
                .await?;
                Ok(PageBody {
                    next_token: output.continuation_token().map(str::to_owned),
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
fn collect_bucket<'a>(
    ctx: &'a Context,
    bootstrap: &'a Client,
    bucket: &'a Bucket,
) -> BoxFuture<'a, Result<(), AwsScanError>> {
    Box::pin(async move {
        ensure_active(ctx)?;
        let name = required(bucket.name(), "bucket name")?;
        let scope = ctx.scope(KIND, Some(&name), Source::Direct);
        let fallback = match map_bucket(ctx, bucket, None, &[], None) {
            Ok(body) => body,
            Err(error) => return observed(ctx, scope, Err(error), None).await,
        };
        let location = resolve_location(ctx, bootstrap, bucket, &name).await;
        let result = match location {
            Ok(region) => bucket_details(&bucket_context(ctx, &region), bucket, &name).await,
            Err(error) => Err(error),
        };
        observed(ctx, scope, result, Some(fallback)).await
    })
}

mod location;
use location::*;

mod details;
use details::*;

mod security;
use security::*;

mod configuration;
use configuration::*;

mod mapping;
use mapping::*;

#[cfg(test)]
mod tests;
