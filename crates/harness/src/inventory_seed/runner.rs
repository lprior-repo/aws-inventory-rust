mod batches;
mod dependencies;
mod identity;
mod reporting;
pub use reporting::{run_report, run_report_with_concurrency};
mod materialize;
#[path = "runner/oracle.rs"]
mod oracle;
#[cfg(test)]
mod tests;
mod values;
#[cfg(test)]
mod values_tests;

use super::{
    ScenarioSeedOracle, SeedResourceIdentity, SeedResourcePair, TypedSeedOracle,
    config::{MutationPermit, SeedConfig, SeedError},
    model::{PreparedInput, PreparedInputValue, PreparedRelationship, PreparedStep, SeedModels},
    scenario::{SeedScenario, default_scenario},
    wire::{self, RequestSpec},
};
use aws_wire::{
    sign::Credentials,
    transport::{Client, Config as TransportConfig},
};
use futures_util::TryStreamExt;
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{AccountId, Edge, EdgeSource, Kind, Region, ResourceId, ResourceKey},
    record::{FieldId, MapRef, Record, Value, ValueRef},
};
use oracle::*;

pub async fn run_seed(
    catalog: &RuntimeCatalog<'_>,
    config: SeedConfig,
    credentials: &Credentials,
) -> Result<TypedSeedOracle, SeedError> {
    run_scenario(catalog, config, credentials, default_scenario()?).await
}

pub async fn run_scenario(
    catalog: &RuntimeCatalog<'_>,
    config: SeedConfig,
    credentials: &Credentials,
    scenario: SeedScenario,
) -> Result<TypedSeedOracle, SeedError> {
    let models = SeedModels::load(catalog, &scenario)?;
    validate_oracle_plan(&models)?;
    let client = seed_client(&config)?;
    execute_scenario(&models, &config, credentials, &client)
        .await?
        .oracle(&models.relationships)
}

pub async fn run_data_scenario(
    catalog: &RuntimeCatalog<'_>,
    config: SeedConfig,
    credentials: &Credentials,
    scenario: SeedScenario,
) -> Result<ScenarioSeedOracle, SeedError> {
    let models = SeedModels::load(catalog, &scenario)?;
    let client = seed_client(&config)?;
    execute_scenario(&models, &config, credentials, &client)
        .await?
        .generic_oracle(&models.relationships)
}

fn seed_client(config: &SeedConfig) -> Result<Client, SeedError> {
    Ok(Client::new(TransportConfig {
        extra_ca_bundle: Some(config.ca_bundle.clone()),
        ..TransportConfig::default()
    })?)
}

async fn execute_scenario(
    models: &SeedModels<'_>,
    config: &SeedConfig,
    credentials: &Credentials,
    client: &Client,
) -> Result<SeedState, SeedError> {
    let context = SeedContext {
        models,
        client,
        credentials,
        config,
        mutation_permit: config.mutation_permit(),
    };
    let (account, partition) = caller_account(&context).await?;
    let state = SeedState::new(
        account,
        config.region.clone(),
        partition,
        models.steps.len(),
    );
    let steps = models.steps.iter().map(Ok::<_, SeedError>);
    futures_util::stream::iter(steps)
        .try_fold(state, |mut state, step| async {
            execute_step(&mut state, step, &context).await?;
            Ok(state)
        })
        .await
}

struct SeedContext<'c, 'm> {
    models: &'c SeedModels<'m>,
    client: &'c Client,
    credentials: &'c Credentials,
    config: &'c SeedConfig,
    mutation_permit: &'c MutationPermit,
}

struct SeedOutput {
    step: super::scenario::StepId,
    resources: SeedResources,
    record: Record,
}

enum SeedResources {
    None,
    One(SeedResourceIdentity),
    List(Vec<SeedResourceIdentity>),
}

impl SeedResources {
    fn as_slice(&self) -> &[SeedResourceIdentity] {
        match self {
            Self::None => &[],
            Self::One(resource) => std::slice::from_ref(resource),
            Self::List(resources) => resources,
        }
    }
}

struct SeedState {
    account: AccountId,
    region: Region,
    partition: String,
    outputs: Vec<Option<SeedOutput>>,
}

impl SeedState {
    fn new(account: AccountId, region: Region, partition: String, step_count: usize) -> Self {
        Self {
            account,
            region,
            partition,
            outputs: std::iter::repeat_with(|| None).take(step_count).collect(),
        }
    }

    fn accepted_output(&self, step: super::StepId) -> Option<&SeedOutput> {
        self.outputs.get(usize::from(step.0))?.as_ref()
    }
}

async fn caller_account(context: &SeedContext<'_, '_>) -> Result<(AccountId, String), SeedError> {
    let input = wire::input_record(std::iter::empty::<(Vec<FieldId>, Value)>())?;
    let response = wire::call(
        context.client,
        context.credentials,
        RequestSpec {
            operation: context.models.caller,
            input: input.root(),
            endpoint: &context.config.endpoint,
            region: context.config.region.as_str(),
        },
    )
    .await?;
    identity::caller_identity(
        &text_path(response.root(), &context.models.caller_account)?,
        &text_path(response.root(), &context.models.caller_arn)?,
    )
}

async fn execute_step(
    state: &mut SeedState,
    step: &PreparedStep<'_>,
    context: &SeedContext<'_, '_>,
) -> Result<(), SeedError> {
    let output = prepare_output(state, step, context).await?;
    record_output(state, output)
}

async fn prepare_output(
    state: &SeedState,
    step: &PreparedStep<'_>,
    context: &SeedContext<'_, '_>,
) -> Result<SeedOutput, SeedError> {
    let inputs = step
        .inputs
        .iter()
        .map(|input| materialize_input(state, input))
        .collect::<Result<Vec<_>, _>>()?;
    let response = call_step(step, context, &inputs).await?;
    validate_response(&response, step, &inputs)?;
    let resources = match &step.resource {
        Some(resource) => identity::resources(state, step.id, resource, &response, &inputs)?,
        None => SeedResources::None,
    };
    Ok(SeedOutput {
        step: step.id,
        resources,
        record: response,
    })
}

async fn call_step(
    step: &PreparedStep<'_>,
    context: &SeedContext<'_, '_>,
    inputs: &[(Vec<FieldId>, Value)],
) -> Result<Record, SeedError> {
    let record = wire::input_record(inputs.iter().cloned())?;
    let spec = RequestSpec {
        operation: step.operation,
        input: record.root(),
        endpoint: &context.config.endpoint,
        region: context.config.region.as_str(),
    };
    wire::call_mutation(
        context.mutation_permit,
        context.client,
        context.credentials,
        spec,
    )
    .await
}

fn record_output(state: &mut SeedState, output: SeedOutput) -> Result<(), SeedError> {
    let (slot, prior_outputs) = state
        .outputs
        .get_mut(..=usize::from(output.step.0))
        .and_then(|outputs| outputs.split_last_mut())
        .ok_or(SeedError::Invalid(
            "seed output step exceeds scenario bounds",
        ))?;
    let resources = output.resources.as_slice();
    let previous = || {
        prior_outputs
            .iter()
            .flatten()
            .flat_map(|prior| prior.resources.as_slice())
    };
    if resources.iter().enumerate().any(|(index, resource)| {
        resources
            .iter()
            .take(index)
            .chain(previous())
            .any(|prior| prior.key == resource.key)
    }) {
        return Err(SeedError::Invalid(
            "seed operation returned a duplicate resource identity",
        ));
    }
    *slot = Some(output);
    Ok(())
}

fn validate_exact_identity(expected: Option<&str>, identity: &ResourceId) -> Result<(), SeedError> {
    if expected.is_some_and(|expected| expected != identity.as_str()) {
        return Err(SeedError::Invalid(
            "seed operation returned an unexpected exact identity",
        ));
    }
    Ok(())
}

fn materialize_input(
    state: &SeedState,
    input: &PreparedInput,
) -> Result<(Vec<FieldId>, Value), SeedError> {
    let value = values::materialize(state, &input.value)?;
    Ok((input.path.clone(), value))
}

fn validate_response(
    response: &Record,
    step: &PreparedStep<'_>,
    inputs: &[(Vec<FieldId>, Value)],
) -> Result<(), SeedError> {
    step.checks.iter().try_for_each(|check| {
        let sent = inputs
            .iter()
            .find(|(path, _)| path == &check.input_path)
            .map(|(_, value)| value)
            .ok_or(SeedError::Invalid("response check input is missing"))?;
        if value_path(response.root(), &check.output_path)?.to_value()? != *sent {
            return Err(SeedError::Invalid(
                "modeled response did not confirm its requested input",
            ));
        }
        Ok(())
    })
}
