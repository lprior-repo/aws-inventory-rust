//! Child-scope failure recording shared by Step Functions listing and
//! enrichment.

use inventory_core::Source;

use crate::{
    AwsScanError,
    common::{Context, error_category},
};

/// Records the failure in its own child scope and returns no observation.
pub(super) async fn reject<T>(
    ctx: &Context,
    collector: &str,
    parent: Option<&str>,
    error: AwsScanError,
) -> Result<Option<T>, AwsScanError> {
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
