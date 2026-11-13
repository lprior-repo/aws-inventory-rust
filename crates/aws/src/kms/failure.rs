//! Child-scope failure recording shared by KMS listing and enrichment.

use inventory_core::{Resource, Source};

use crate::{
    AwsScanError,
    common::{Context, error_category},
};

pub(super) async fn fail(
    ctx: &Context,
    collector: &str,
    parent: Option<&str>,
    error: AwsScanError,
) -> Result<Option<Resource>, AwsScanError> {
    failure(ctx, collector, parent, &error).await?;
    Ok(None)
}

pub(super) async fn failure(
    ctx: &Context,
    collector: &str,
    parent: Option<&str>,
    error: &AwsScanError,
) -> Result<(), AwsScanError> {
    ctx.failure(
        &ctx.scope(collector, parent, Source::Direct),
        error_category(error),
        error.to_string(),
    )
    .await
}
