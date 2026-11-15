#[path = "runner/oracle.rs"]
mod oracle;
use super::{
    SeedResourcePair, TypedSeedOracle,
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
    run_scenario(catalog, config, credentials, default_scenario()).await
}

pub async fn run_scenario(
    catalog: &RuntimeCatalog<'_>,
    config: SeedConfig,
    credentials: &Credentials,
    scenario: SeedScenario,
) -> Result<TypedSeedOracle, SeedError> {
    let scenario = SeedScenario::with_relationships(
        scenario.steps().to_vec(),
        scenario.relationships().to_vec(),
    )?;
    let models = SeedModels::load(catalog, &scenario)?;
    validate_oracle_plan(&models)?;
    let client = Client::new(TransportConfig {
        extra_ca_bundle: Some(config.ca_bundle.clone()),
        ..TransportConfig::default()
    })?;
    execute_scenario(&models, &config, credentials, &client).await
}

async fn execute_scenario(
    models: &SeedModels<'_>,
    config: &SeedConfig,
    credentials: &Credentials,
    client: &Client,
) -> Result<TypedSeedOracle, SeedError> {
    let context = SeedContext {
        models,
        client,
        credentials,
        config,
        mutation_permit: config.mutation_permit(),
    };
    let account = caller_account(&context).await?;
    let state = SeedState::new(account, config.region.clone());
    let steps = models.steps.iter().map(Ok::<_, SeedError>);
    let completed = futures_util::stream::iter(steps)
        .try_fold(state, |mut state, step| async {
            execute_step(&mut state, step, &context).await?;
            Ok(state)
        })
        .await?;
    completed.oracle(&models.relationships)
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
    kind: Kind,
    identity: ResourceId,
    key: ResourceKey,
    record: Record,
}

struct SeedState {
    account: AccountId,
    region: Region,
    outputs: Vec<SeedOutput>,
}

impl SeedState {
    fn new(account: AccountId, region: Region) -> Self {
        Self {
            account,
            region,
            outputs: Vec::with_capacity(12),
        }
    }
}

async fn caller_account(context: &SeedContext<'_, '_>) -> Result<AccountId, SeedError> {
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
    AccountId::parse(&text_path(response.root(), &context.models.caller_account)?)
        .map_err(SeedError::from)
}

async fn execute_step(
    state: &mut SeedState,
    step: &PreparedStep<'_>,
    context: &SeedContext<'_, '_>,
) -> Result<(), SeedError> {
    let inputs = step
        .inputs
        .iter()
        .map(|input| materialize_input(state, input))
        .collect::<Result<Vec<_>, _>>()?;
    let response = call_step(step, context, &inputs).await?;
    validate_response(&response, step, &inputs)?;
    let identity = ResourceId::parse(&text_path(response.root(), &step.identity)?)?;
    record_output(state, step, identity, response)
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

fn record_output(
    state: &mut SeedState,
    step: &PreparedStep<'_>,
    identity: ResourceId,
    record: Record,
) -> Result<(), SeedError> {
    if state
        .outputs
        .iter()
        .any(|output| output.identity == identity)
    {
        let message = "seed operation returned a duplicate resource identity";
        return Err(SeedError::Invalid(message));
    }
    let key = resource_key(state, step, identity.clone());
    let (step, kind) = (step.id, step.kind.clone());
    state.outputs.push(SeedOutput {
        step,
        kind,
        identity,
        key,
        record,
    });
    Ok(())
}

fn materialize_input(
    state: &SeedState,
    input: &PreparedInput,
) -> Result<(Vec<FieldId>, Value), SeedError> {
    let value = match &input.value {
        PreparedInputValue::Text(text) => Value::Text(text.clone()),
        PreparedInputValue::Boolean(value) => Value::Bool(*value),
        PreparedInputValue::Integer(value) => Value::Int(*value),
        PreparedInputValue::PreviousOutput { step, path } => {
            let output = state
                .outputs
                .get(usize::from(step.0))
                .filter(|output| output.step == *step)
                .ok_or(SeedError::Invalid("seed binding output is not available"))?;
            value_path(output.record.root(), path)?.to_value()?
        }
    };
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

fn resource_key(state: &SeedState, step: &PreparedStep<'_>, identity: ResourceId) -> ResourceKey {
    ResourceKey {
        account: state.account.clone(),
        region: state.region.clone(),
        kind: step.kind.clone(),
        id: identity,
    }
}
