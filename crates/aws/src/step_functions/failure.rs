use inventory_core::Source;

use crate::{
    AwsScanError,
    common::{Context, error_category},
};

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
