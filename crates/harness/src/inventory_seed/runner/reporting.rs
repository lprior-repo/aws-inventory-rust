use super::*;
use super::super::{SeedRunReport, StepOutcome, report};

pub async fn run_report(
    catalog: &RuntimeCatalog<'_>,
    config: &SeedConfig,
    credentials: &Credentials,
    scenario: &SeedScenario,
) -> Result<SeedRunReport, SeedError> {
    let models = SeedModels::load(catalog, scenario)?;
    let client = seed_client(config)?;
    let context = SeedContext { models: &models, client: &client, credentials, config,
        mutation_permit: config.mutation_permit() };
    let (account, partition) = caller_account(&context).await?;
    let state = SeedState::new(account, config.region.clone(), partition);
    let steps = models.steps.iter().map(Ok::<_, SeedError>);
    let (state, outcomes) = futures_util::stream::iter(steps)
        .try_fold((state, Vec::new()), |(mut state, mut outcomes), step| async {
            outcomes.push(execute(&mut state, step, &context).await?);
            Ok((state, outcomes))
        }).await?;
    let relationships: Vec<_> = models.relationships.into_iter().filter(|relationship| {
        state.output(relationship.source).is_ok() && state.output(relationship.target).is_ok()
    }).collect();
    Ok(SeedRunReport { oracle: state.generic_oracle(&relationships)?, outcomes })
}

async fn execute(state: &mut SeedState, step: &PreparedStep<'_>, context: &SeedContext<'_, '_>) -> Result<StepOutcome, SeedError> {
    let dependencies = super::dependencies::dependencies(step)?;
    if let Some(dependency) = dependencies.iter().find(|id| state.output(**id).is_err()) {
        return Ok(StepOutcome::Skipped { dependency: *dependency });
    }
    match prepare_output(state, step, context).await {
        Ok(output) => {
            record_output(state, output.step, output.resource, output.record)?;
            Ok(StepOutcome::Accepted)
        }
        Err(error) => Ok(report::rejection(error)),
    }
}
