mod cloudfront;
pub(crate) mod iam;
mod route53;
mod s3;

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::Mutex;

use aws_types::region::Region;
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::{CollectionEvent, CollectionPage, CollectionScope, Edge, Resource, Source};
use inventory_engine::limiter::ApiError;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::AwsScanError;
use crate::common::{Context, MAX_CHILDREN, PageBody, error_category, require_budget};

pub(crate) const GLOBAL_SERVICES: [&str; 4] = ["s3", "iam", "route53", "cloudfront"];
const CHILD_CONCURRENCY: usize = 4;
const MAX_COMPONENTS: usize = 4_096;
const MAX_LIST_ITEMS: usize = 1_000;

#[tracing::instrument(skip_all, fields(account = %base.account))]
pub(crate) async fn collect(
    base: &Context,
    services: &BTreeSet<String>,
) -> Result<(), AwsScanError> {
    ensure_active(base)?;
    let ctx = global_context(base)?;

    let mut work = Vec::with_capacity(GLOBAL_SERVICES.len());
    for service in GLOBAL_SERVICES {
        if services.contains(service) {
            work.push(service_future(&ctx, service));
        }
    }
    stream::iter(work)
        .buffer_unordered(4)
        .try_for_each(|()| async { Ok(()) })
        .await?;
    ensure_active(&ctx)
}

fn service_future<'a>(ctx: &'a Context, service: &str) -> BoxFuture<'a, Result<(), AwsScanError>> {
    match service {
        "s3" => Box::pin(s3::collect(ctx)),
        "iam" => Box::pin(iam::collect(ctx)),
        "route53" => Box::pin(route53::collect(ctx)),
        "cloudfront" => Box::pin(cloudfront::collect(ctx)),
        _ => Box::pin(async {
            Err(AwsScanError::InvalidOptions(
                "unknown global collector".into(),
            ))
        }),
    }
}

fn endpoint_region(partition: &str) -> Result<&'static str, AwsScanError> {
    match partition {
        "aws" => Ok("us-east-1"),
        "aws-cn" => Ok("cn-north-1"),
        "aws-us-gov" => Ok("us-gov-west-1"),
        _ => Err(AwsScanError::InvalidOptions(format!(
            "global inventory has no reviewed endpoint policy for partition {partition}"
        ))),
    }
}

fn global_context(base: &Context) -> Result<Context, AwsScanError> {
    let config = base
        .config
        .to_builder()
        .region(Region::new(endpoint_region(&base.partition)?))
        .build();
    Ok(Context {
        config,
        region: "global".to_owned(),
        ..base.clone()
    })
}

fn ensure_active(ctx: &Context) -> Result<(), AwsScanError> {
    if ctx.executor.is_cancelled() {
        Err(ApiError::Cancelled.into())
    } else {
        Ok(())
    }
}

fn components(count: usize, what: &'static str) -> Result<(), AwsScanError> {
    if count > MAX_COMPONENTS {
        return Err(AwsScanError::InvalidData(format!(
            "{what} exceeds {MAX_COMPONENTS} components"
        )));
    }
    Ok(())
}

mod identity;
use identity::*;

mod observation;
use observation::*;

mod pagination;
use pagination::*;

#[cfg(test)]
mod tests;
