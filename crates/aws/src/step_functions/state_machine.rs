use std::collections::BTreeMap;

use aws_sdk_sfn::{
    Client,
    operation::describe_state_machine::{
        DescribeStateMachineOutput, builders::DescribeStateMachineFluentBuilder,
    },
    types::{IncludedData, StateMachineListItem},
};
use inventory_core::{Edge, Resource};
use inventory_engine::limiter::ApiError;
use serde_json::{Value, json};

use super::{
    arn,
    failure::{failure, reject},
    page::Observation,
    values,
};
use crate::{
    AwsScanError,
    common::{Context, sdk_error},
};

const KIND: &str = "aws_step_functions_state_machine";

pub(super) async fn enrich_state_machine(
    ctx: &Context,
    client: &Client,
    item: &StateMachineListItem,
) -> Result<Option<Observation>, AwsScanError> {
    let arn = match listed(ctx, item) {
        Ok(value) => value,
        Err(error) => return reject(ctx, KIND, None, error).await,
    };
    let described = describe(ctx, client, arn).await?;
    let tags = tags(ctx, client, arn).await?;
    match map_state_machine(ctx, item, described.as_ref(), tags) {
        Ok(observation) => Ok(Some(observation)),
        Err(error) => reject(ctx, KIND, Some(arn), error).await,
    }
}

fn listed<'a>(ctx: &Context, item: &'a StateMachineListItem) -> Result<&'a str, AwsScanError> {
    let arn = item.state_machine_arn();
    if arn.is_empty() {
        return Err(values::invalid("ListStateMachines lacks StateMachineArn"));
    }
    let parts = arn::split_state_machine_arn(arn).map_err(values::invalid)?;
    if !arn::is_in_scope(&parts, ctx) {
        return Err(values::invalid(
            "Step Functions state machine ARN is outside the scan scope",
        ));
    }
    let name = item.name();
    if name.is_empty() || parts.name != name {
        return Err(values::invalid(
            "ListStateMachines Name does not match StateMachineArn",
        ));
    }
    Ok(arn)
}

async fn describe(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<DescribeStateMachineOutput>, AwsScanError> {
    match describe_call(ctx, client, arn).await {
        Ok(output) => Ok(Some(output)),
        Err(error) => {
            record(ctx, "aws_step_functions_state_machine_describe", arn, error).await?;
            Ok(None)
        }
    }
}

async fn describe_call(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<DescribeStateMachineOutput, ApiError> {
    ctx.execute("sfn", "control_read", || async {
        describe_request(client, arn)
            .send()
            .await
            .map_err(sdk_error)
    })
    .await
}

/// Inventory never reads the definition, so ask for metadata only: an
/// encrypted definition would otherwise require `kms:Decrypt`.
pub(super) fn describe_request(client: &Client, arn: &str) -> DescribeStateMachineFluentBuilder {
    client
        .describe_state_machine()
        .state_machine_arn(arn)
        .included_data(IncludedData::MetadataOnly)
}

async fn tags(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    match tags_call(ctx, client, arn).await {
        Ok(tags) => Ok(Some(tags)),
        Err(error) => {
            record(ctx, "aws_step_functions_state_machine_tags", arn, error).await?;
            Ok(None)
        }
    }
}

async fn tags_call(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<BTreeMap<String, String>, ApiError> {
    let output = ctx
        .execute("sfn", "control_read", || async {
            client
                .list_tags_for_resource()
                .resource_arn(arn)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await?;
    Ok(values::tags_from(output.tags()))
}

async fn record(
    ctx: &Context,
    collector: &str,
    arn: &str,
    error: ApiError,
) -> Result<(), AwsScanError> {
    failure(ctx, collector, Some(arn), &AwsScanError::Api(error)).await
}

pub(super) fn map_state_machine(
    ctx: &Context,
    listed: &StateMachineListItem,
    described: Option<&DescribeStateMachineOutput>,
    tags: Option<BTreeMap<String, String>>,
) -> Result<Observation, AwsScanError> {
    let (id, name) = match described {
        Some(described) => (
            described.state_machine_arn(),
            identity(ctx, listed, described)?.name,
        ),
        None => {
            let arn = self::listed(ctx, listed)?;
            (arn, listed.name())
        }
    };
    let tags_collected = tags.is_some();
    let resource = ctx.resource(
        KIND,
        id.to_owned(),
        Some(name.to_owned()),
        properties(listed, described, tags_collected)?,
        tags.unwrap_or_default(),
    )?;
    let edges = described.map_or_else(Vec::new, |described| edges(ctx, &resource, described));
    Ok(Observation { resource, edges })
}

fn identity<'a>(
    ctx: &Context,
    listed: &StateMachineListItem,
    described: &'a DescribeStateMachineOutput,
) -> Result<arn::ArnRef<'a>, AwsScanError> {
    if described.state_machine_arn() != listed.state_machine_arn()
        || described.name() != listed.name()
    {
        return Err(values::invalid(
            "DescribeStateMachine does not match the listed state machine",
        ));
    }
    let parts =
        arn::split_state_machine_arn(described.state_machine_arn()).map_err(values::invalid)?;
    if !arn::is_in_scope(&parts, ctx) || parts.name != described.name() {
        return Err(values::invalid(
            "DescribeStateMachine identity is outside the scan scope",
        ));
    }
    Ok(parts)
}

/// `describe_collected` states the coverage outcome of the DescribeStateMachine
/// enrichment: when it is false only list-derived fields are populated.
fn properties(
    listed: &StateMachineListItem,
    described: Option<&DescribeStateMachineOutput>,
    tags_collected: bool,
) -> Result<Value, AwsScanError> {
    let created = described.map_or_else(|| listed.creation_date(), |d| d.creation_date());
    Ok(json!({
        "state_machine_arn": listed.state_machine_arn(),
        "name": listed.name(),
        "state_machine_type": described.map(|d| d.r#type().as_str()),
        "type": listed.r#type().as_str(),
        "status": described.and_then(|d| d.status()).map(|status| status.as_str()),
        "creation_date": values::millis(created, "state machine")?,
        "role_arn": described.map(|d| d.role_arn()),
        "revision_id": described.and_then(|d| d.revision_id()),
        "description": described.and_then(|d| d.description()),
        "logging_configuration": values::logging(described.and_then(|d| d.logging_configuration())),
        "tracing_configuration": values::tracing(described.and_then(|d| d.tracing_configuration())),
        "encryption_configuration": values::encryption(
            described.and_then(|d| d.encryption_configuration())
        ),
        "describe_collected": described.is_some(),
        "tags_collected": tags_collected,
    }))
}

fn edges(ctx: &Context, resource: &Resource, described: &DescribeStateMachineOutput) -> Vec<Edge> {
    let mut edges = Vec::new();
    if let Some(role) = arn::role_target(described.role_arn(), ctx) {
        edges.push(ctx.edge(resource, role, "ASSUMES_ROLE"));
    }
    let key = described
        .encryption_configuration()
        .and_then(|config| config.kms_key_id())
        .and_then(|key| arn::kms_target(key, ctx));
    if let Some(key) = key {
        edges.push(ctx.edge(resource, key, "ENCRYPTED_BY"));
    }
    edges
}
