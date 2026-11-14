use std::collections::BTreeMap;

use aws_sdk_sfn::{
    Client, operation::describe_activity::DescribeActivityOutput, types::ActivityListItem,
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

const KIND: &str = "aws_step_functions_activity";

pub(super) async fn enrich_activity(
    ctx: &Context,
    client: &Client,
    item: &ActivityListItem,
) -> Result<Option<Observation>, AwsScanError> {
    let arn = match listed(ctx, item) {
        Ok(value) => value,
        Err(error) => return reject(ctx, KIND, None, error).await,
    };
    let Some(described) = describe(ctx, client, arn).await? else {
        return Ok(None);
    };
    let tags = tags(ctx, client, arn).await?;
    match map_activity(ctx, item, &described, tags) {
        Ok(observation) => Ok(Some(observation)),
        Err(error) => reject(ctx, KIND, Some(arn), error).await,
    }
}

fn listed<'a>(ctx: &Context, item: &'a ActivityListItem) -> Result<&'a str, AwsScanError> {
    let arn = item.activity_arn();
    if arn.is_empty() {
        return Err(values::invalid("ListActivities lacks ActivityArn"));
    }
    let parts = arn::split_activity_arn(arn).map_err(values::invalid)?;
    if !arn::is_in_scope(&parts, ctx) {
        return Err(values::invalid(
            "Step Functions activity ARN is outside the scan scope",
        ));
    }
    let name = item.name();
    if name.is_empty() || parts.name != name {
        return Err(values::invalid(
            "ListActivities Name does not match ActivityArn",
        ));
    }
    Ok(arn)
}

async fn describe(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<DescribeActivityOutput>, AwsScanError> {
    match describe_call(ctx, client, arn).await {
        Ok(output) => Ok(Some(output)),
        Err(error) => {
            record(ctx, "aws_step_functions_activity_describe", arn, error).await?;
            Ok(None)
        }
    }
}

async fn describe_call(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<DescribeActivityOutput, ApiError> {
    ctx.execute("sfn", "control_read", || async {
        client
            .describe_activity()
            .activity_arn(arn)
            .send()
            .await
            .map_err(sdk_error)
    })
    .await
}

async fn tags(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    match tags_call(ctx, client, arn).await {
        Ok(tags) => Ok(Some(tags)),
        Err(error) => {
            record(ctx, "aws_step_functions_activity_tags", arn, error).await?;
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

pub(super) fn map_activity(
    ctx: &Context,
    listed: &ActivityListItem,
    described: &DescribeActivityOutput,
    tags: Option<BTreeMap<String, String>>,
) -> Result<Observation, AwsScanError> {
    let parts = identity(ctx, listed, described)?;
    let resource = ctx.resource(
        KIND,
        described.activity_arn().to_owned(),
        Some(parts.name.to_owned()),
        properties(described, tags.is_some())?,
        tags.unwrap_or_default(),
    )?;
    let edges = edges(ctx, &resource, described);
    Ok(Observation { resource, edges })
}

fn identity<'a>(
    ctx: &Context,
    listed: &ActivityListItem,
    described: &'a DescribeActivityOutput,
) -> Result<arn::ArnRef<'a>, AwsScanError> {
    if described.activity_arn() != listed.activity_arn() || described.name() != listed.name() {
        return Err(values::invalid(
            "DescribeActivity does not match the listed activity",
        ));
    }
    let parts = arn::split_activity_arn(described.activity_arn()).map_err(values::invalid)?;
    if !arn::is_in_scope(&parts, ctx) || parts.name != described.name() {
        return Err(values::invalid(
            "DescribeActivity identity is outside the scan scope",
        ));
    }
    Ok(parts)
}

fn properties(
    described: &DescribeActivityOutput,
    tags_collected: bool,
) -> Result<Value, AwsScanError> {
    Ok(json!({
        "activity_arn": described.activity_arn(),
        "name": described.name(),
        "creation_date": values::millis(described.creation_date(), "activity")?,
        "encryption_configuration": values::encryption(described.encryption_configuration()),
        "tags_collected": tags_collected,
    }))
}

fn edges(ctx: &Context, resource: &Resource, described: &DescribeActivityOutput) -> Vec<Edge> {
    let key = described
        .encryption_configuration()
        .and_then(|config| config.kms_key_id())
        .and_then(|key| arn::kms_target(key, ctx));
    key.map(|key| ctx.edge(resource, key, "ENCRYPTED_BY"))
        .into_iter()
        .collect()
}
