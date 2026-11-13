//! KMS key and alias inventory. `ListKeys` paginates and enriches each key with
//! `DescribeKey`, customer-managed rotation status and tags; `ListAliases`
//! paginates independently. One key's child failure never discards the key.

use aws_sdk_kms::Client;
use inventory_core::Source;

use crate::{
    AwsScanError,
    common::{Context, collect_bounded, paginate, require_budget, sdk_error},
};

mod alias;
mod arn;
mod enrich;
mod failure;
mod key;
mod page;
use alias::map_alias;
use enrich::enrich_key;
use failure::failure;
use page::{alias_page_body, dedupe, key_page_body, page_limit};

#[cfg(test)]
use enrich::{MAX_TAGS_PER_KEY, enrichment};
#[cfg(test)]
use key::{Rotation, map_key, tags_from};
#[cfg(test)]
use page::LIST_MAX_RESULTS;

const ENRICHMENT_CONCURRENCY: usize = 16;

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    tokio::try_join!(keys(ctx, &client), aliases(ctx, &client))?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn keys(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_kms_key", None, Source::Direct),
        |marker, budget| async move {
            let output = ctx
                .execute("kms", "control_read", || async {
                    client
                        .list_keys()
                        .limit(page_limit(ctx.page_size))
                        .set_marker(marker.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.keys().len(), budget)?;
            let resources =
                collect_bounded(output.keys().iter(), ENRICHMENT_CONCURRENCY, |entry| {
                    enrich_key(ctx, client, entry)
                })
                .await?;
            Ok(key_page_body(
                &output,
                dedupe(resources.into_iter().flatten().collect()),
            ))
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn aliases(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_kms_alias", None, Source::Direct),
        |marker, budget| async move {
            let output = ctx
                .execute("kms", "control_read", || async {
                    client
                        .list_aliases()
                        .limit(page_limit(ctx.page_size))
                        .set_marker(marker.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.aliases().len(), budget)?;
            let mut resources = Vec::new();
            for entry in output.aliases() {
                match map_alias(ctx, entry) {
                    Ok(resource) => resources.push(resource),
                    Err(error) => failure(ctx, "aws_kms_alias", None, &error).await?,
                }
            }
            Ok(alias_page_body(&output, dedupe(resources)))
        },
    )
    .await
}

#[cfg(test)]
mod tests;
