//! Step Functions state machine and activity inventory. `ListStateMachines` and
//! `ListActivities` paginate independently; each listed item is described and
//! its tags read in per-item child scopes. Definitions, variable references and
//! execution data are never retained.

use aws_sdk_sfn::{
    Client,
    operation::{
        list_activities::ListActivitiesOutput, list_state_machines::ListStateMachinesOutput,
    },
};
use inventory_core::Source;

use crate::{
    AwsScanError,
    common::{Context, PageBody, collect_bounded, paginate, require_budget, sdk_error},
};

mod activity;
mod arn;
mod failure;
mod page;
mod state_machine;
mod values;

use activity::enrich_activity;
use page::{Observation, activity_page_body, dedupe, page_limit, state_machine_page_body};
use state_machine::enrich_state_machine;

const ENRICHMENT_CONCURRENCY: usize = 16;

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    tokio::try_join!(state_machines(ctx, &client), activities(ctx, &client))?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn state_machines(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    let scope = ctx.scope("aws_step_functions_state_machine", None, Source::Direct);
    paginate(ctx, scope, |marker, budget| {
        state_machine_page(ctx, client, marker, budget)
    })
    .await
}

async fn state_machine_page(
    ctx: &Context,
    client: &Client,
    marker: Option<String>,
    budget: usize,
) -> Result<PageBody, AwsScanError> {
    let (output, observations) = state_machine_batch(ctx, client, marker, budget).await?;
    Ok(state_machine_page_body(&output, dedupe(observations)))
}

async fn state_machine_batch(
    ctx: &Context,
    client: &Client,
    marker: Option<String>,
    budget: usize,
) -> Result<(ListStateMachinesOutput, Vec<Observation>), AwsScanError> {
    let output = list_state_machines(ctx, client, marker).await?;
    require_budget(output.state_machines().len(), budget)?;
    let observations = enrich_state_machines(ctx, client, &output).await?;
    Ok((output, observations))
}

async fn list_state_machines(
    ctx: &Context,
    client: &Client,
    marker: Option<String>,
) -> Result<ListStateMachinesOutput, AwsScanError> {
    ctx.execute("sfn", "control_read", || async {
        client
            .list_state_machines()
            .max_results(page_limit(ctx.page_size))
            .set_next_token(marker.clone())
            .send()
            .await
            .map_err(sdk_error)
    })
    .await
    .map_err(AwsScanError::Api)
}

async fn enrich_state_machines(
    ctx: &Context,
    client: &Client,
    output: &ListStateMachinesOutput,
) -> Result<Vec<Observation>, AwsScanError> {
    let observations = collect_bounded(
        output.state_machines().iter(),
        ENRICHMENT_CONCURRENCY,
        |item| enrich_state_machine(ctx, client, item),
    )
    .await?;
    Ok(observations.into_iter().flatten().collect())
}

#[tracing::instrument(skip_all)]
async fn activities(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    let scope = ctx.scope("aws_step_functions_activity", None, Source::Direct);
    paginate(ctx, scope, |marker, budget| {
        activity_page(ctx, client, marker, budget)
    })
    .await
}

async fn activity_page(
    ctx: &Context,
    client: &Client,
    marker: Option<String>,
    budget: usize,
) -> Result<PageBody, AwsScanError> {
    let (output, observations) = activity_batch(ctx, client, marker, budget).await?;
    Ok(activity_page_body(&output, dedupe(observations)))
}

async fn activity_batch(
    ctx: &Context,
    client: &Client,
    marker: Option<String>,
    budget: usize,
) -> Result<(ListActivitiesOutput, Vec<Observation>), AwsScanError> {
    let output = ctx
        .execute("sfn", "control_read", || async {
            client
                .list_activities()
                .max_results(page_limit(ctx.page_size))
                .set_next_token(marker.clone())
                .send()
                .await
                .map_err(sdk_error)
        })
        .await?;
    require_budget(output.activities().len(), budget)?;
    let observations =
        collect_bounded(output.activities().iter(), ENRICHMENT_CONCURRENCY, |item| {
            enrich_activity(ctx, client, item)
        })
        .await?;
    Ok((output, observations.into_iter().flatten().collect()))
}

#[cfg(test)]
mod tests;
