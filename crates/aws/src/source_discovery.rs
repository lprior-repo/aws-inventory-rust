//! Attach conservative source discovery to an inventory scan. The standalone
//! discovery report carries candidates; the scan persists discovery coverage
//! and only invokes an unambiguous, existing source.
use futures_util::{TryStreamExt, stream};
use inventory_core::{CollectionEvent, CollectionPage, Source};

use crate::{
    AwsScanError,
    common::Context,
    discovery::{self, DiscoveryCoverage, SourceSelection},
    supplemental,
};

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(
    ctx: &Context,
    config: bool,
    explorer: bool,
) -> Result<(), AwsScanError> {
    let report = discovery::supplemental_sources(ctx, config, explorer).await;
    if config {
        record_selection(
            ctx,
            "aws_config_source_discovery",
            &report.config.selection,
            report.config.coverage.as_ref(),
        )
        .await?;
        if let Some(aggregator) = report.config.selection.selected_identifier() {
            supplemental::config(ctx, aggregator).await?;
        }
    }
    if explorer {
        record_selection(
            ctx,
            "aws_resource_explorer_source_discovery",
            &report.resource_explorer.selection,
            report.resource_explorer.coverage.as_ref(),
        )
        .await?;
        if let Some(view) = report.resource_explorer.selection.selected_identifier() {
            supplemental::explorer(ctx, view).await?;
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all, fields(collector))]
async fn record_selection(
    ctx: &Context,
    collector: &str,
    selection: &SourceSelection,
    coverage: Option<&DiscoveryCoverage>,
) -> Result<(), AwsScanError> {
    // The scope's parent identifies the discovered source, or the observed
    // absence. It is a discovery result, never a fabricated resource node.
    let parent = match selection {
        SourceSelection::Selected { identifier, .. } => Some(identifier.as_str()),
        SourceSelection::Absent => Some("absent"),
        _ => None,
    };
    let scope = ctx.scope(collector, parent, Source::Direct);
    if let Some(coverage) = coverage {
        stream::iter(coverage.failures.iter().map(Ok::<_, AwsScanError>))
            .try_for_each(|failure| {
                ctx.failure(
                    &scope,
                    &failure.category,
                    format!("{}: {}", failure.operation, failure.message),
                )
            })
            .await?;
        if coverage.suppressed_failures > 0 {
            ctx.failure(
                &scope,
                "discovery_failure_limit",
                format!(
                    "{} additional discovery failures were suppressed",
                    coverage.suppressed_failures
                ),
            )
            .await?;
        }
    }
    if let Some(reason) = incomplete_selection(selection) {
        ctx.failure(&scope, "source_selection", reason).await?;
    }
    let page = CollectionPage {
        scope,
        page_number: 1,
        next_token: None,
        resources: Vec::new(),
        edges: Vec::new(),
    };
    page.validate()?;
    ctx.sender
        .send(CollectionEvent::Page(page))
        .await
        .map_err(|_| AwsScanError::ChannelClosed)
}

fn incomplete_selection(selection: &SourceSelection) -> Option<String> {
    match selection {
        SourceSelection::NotRequested => {
            Some("requested discovery returned no selection outcome".into())
        }
        SourceSelection::Unavailable { reason } => Some(reason.clone()),
        SourceSelection::Ambiguous { candidates } => Some(format!(
            "{} candidate sources; choose an explicit aggregator/view with the discover command",
            candidates.len()
        )),
        SourceSelection::Absent | SourceSelection::Selected { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absence_is_successful_discovery_but_ambiguity_is_incomplete() {
        assert!(incomplete_selection(&SourceSelection::Absent).is_none());
        assert!(
            incomplete_selection(&SourceSelection::Selected {
                identifier: "view".into(),
                reason: "one".into()
            })
            .is_none()
        );
        assert!(
            incomplete_selection(&SourceSelection::Ambiguous {
                candidates: vec!["one".into(), "two".into()]
            })
            .is_some()
        );
        assert!(
            incomplete_selection(&SourceSelection::Unavailable {
                reason: "denied".into()
            })
            .is_some()
        );
    }
}
