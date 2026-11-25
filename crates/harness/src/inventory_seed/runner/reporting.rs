use super::super::{SeedRunReport, StepOutcome, report};
use super::*;
use futures_util::StreamExt;
use tokio::sync::Semaphore;

pub async fn run_report(
    catalog: &RuntimeCatalog<'_>,
    config: &SeedConfig,
    credentials: &Credentials,
    scenario: &SeedScenario,
) -> Result<SeedRunReport, SeedError> {
    run_report_with_concurrency(catalog, config, credentials, scenario, 1).await
}

pub async fn run_report_with_concurrency(
    catalog: &RuntimeCatalog<'_>,
    config: &SeedConfig,
    credentials: &Credentials,
    scenario: &SeedScenario,
    concurrency: usize,
) -> Result<SeedRunReport, SeedError> {
    if !(1..=64).contains(&concurrency) {
        return Err(SeedError::Invalid(
            "seed concurrency must be between 1 and 64",
        ));
    }
    let models = SeedModels::load(catalog, scenario)?;
    let client = seed_client(config)?;
    let context = context(&models, &client, credentials, config);
    let (account, partition) = caller_account(&context).await?;
    let state = SeedState::new(
        account,
        config.region.clone(),
        partition,
        models.steps.len(),
    );
    let batches = super::batches::ranges(&models.steps, scenario.steps(), concurrency)?;
    let (state, outcomes) = execute_batches(state, batches, &context, concurrency).await?;
    finish(state, outcomes, models.relationships)
}

fn context<'c, 'm>(
    models: &'c SeedModels<'m>,
    client: &'c Client,
    credentials: &'c Credentials,
    config: &'c SeedConfig,
) -> SeedContext<'c, 'm> {
    SeedContext {
        models,
        client,
        credentials,
        config,
        mutation_permit: config.mutation_permit(),
    }
}

async fn execute_batches(
    state: SeedState,
    batches: Vec<std::ops::Range<usize>>,
    context: &SeedContext<'_, '_>,
    concurrency: usize,
) -> Result<(SeedState, Vec<StepOutcome>), SeedError> {
    let semaphore = Semaphore::new(concurrency);
    futures_util::stream::iter(batches.into_iter().map(Ok::<_, SeedError>))
        .try_fold((state, Vec::new()), |(mut state, mut outcomes), range| {
            let semaphore = &semaphore;
            async move {
                let outputs = batch(&state, range, context, semaphore, concurrency).await?;
                outcomes.extend(
                    outputs
                        .into_iter()
                        .map(|output| publish(&mut state, output)),
                );
                Ok((state, outcomes))
            }
        })
        .await
}

async fn batch(
    state: &SeedState,
    range: std::ops::Range<usize>,
    context: &SeedContext<'_, '_>,
    semaphore: &Semaphore,
    concurrency: usize,
) -> Result<Vec<Attempt>, SeedError> {
    let steps = context
        .models
        .steps
        .get(range)
        .ok_or(SeedError::Invalid("seed execution batch exceeds bounds"))?;
    futures_util::stream::iter(steps)
        .map(|step| execute(state, step, context, semaphore))
        .buffered(concurrency)
        .try_collect()
        .await
}

fn finish(
    state: SeedState,
    outcomes: Vec<StepOutcome>,
    relationships: Vec<PreparedRelationship>,
) -> Result<SeedRunReport, SeedError> {
    let relationships: Vec<_> = relationships
        .into_iter()
        .filter(|relationship| {
            state.output(relationship.source).is_ok() && state.output(relationship.target).is_ok()
        })
        .collect();
    Ok(SeedRunReport {
        oracle: state.generic_oracle(&relationships)?,
        outcomes,
    })
}

enum Attempt {
    Output(SeedOutput),
    Outcome(StepOutcome),
}

async fn execute(
    state: &SeedState,
    step: &PreparedStep<'_>,
    context: &SeedContext<'_, '_>,
    semaphore: &Semaphore,
) -> Result<Attempt, SeedError> {
    let dependencies = super::dependencies::dependencies(step)?;
    if let Some(dependency) = dependencies.iter().find(|id| state.output(**id).is_err()) {
        return Ok(Attempt::Outcome(StepOutcome::Skipped {
            dependency: *dependency,
        }));
    }
    let _permit = semaphore
        .acquire()
        .await
        .map_err(|_| SeedError::Invalid("seed concurrency semaphore closed"))?;
    Ok(match prepare_output(state, step, context).await {
        Ok(output) => Attempt::Output(output),
        Err(error) => Attempt::Outcome(report::rejection(error)),
    })
}

fn publish(state: &mut SeedState, attempt: Attempt) -> StepOutcome {
    match attempt {
        Attempt::Output(output) => {
            match record_output(state, output.step, output.resource, output.record) {
                Ok(()) => StepOutcome::Accepted,
                Err(error) => report::rejection(error),
            }
        }
        Attempt::Outcome(outcome) => outcome,
    }
}
