//! configuration responsibilities for s3.
use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn logging(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            client
                .get_bucket_logging()
                .bucket(name)
                .expected_bucket_owner(&ctx.account)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await?;
    let Some(config) = output.logging_enabled() else {
        return Ok(json!({"configured":false}));
    };
    Ok(
        json!({"configured":true,"target_bucket":checked_text(config.target_bucket(), "S3 log target bucket")?,
        "target_prefix_omitted":true,"target_grants_collected":false}),
    )
}

#[tracing::instrument(skip_all)]
pub(super) async fn replication(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            optional(
                client
                    .get_bucket_replication()
                    .bucket(name)
                    .expected_bucket_owner(&ctx.account)
                    .send()
                    .await,
                &["ReplicationConfigurationNotFoundError"],
            )
        })
        .await?;
    let Some(output) = output else {
        return Ok(json!({"configured":false}));
    };
    let config = output.replication_configuration().ok_or_else(|| {
        AwsScanError::InvalidData("GetBucketReplication lacks configuration".into())
    })?;
    components(config.rules().len(), "S3 replication rules")?;
    let rules: Vec<Value> = config.rules().iter().map(|rule| json!({
        "id":rule.id(), "priority":rule.priority(), "status":rule.status().as_str(),
        "destination_bucket":rule.destination().map(|v| v.bucket()),
        "destination_account":rule.destination().and_then(|v| v.account()),
        "storage_class":rule.destination().and_then(|v| v.storage_class()).map(|v| v.as_str()),
        "destination_kms_key_id":rule.destination().and_then(|v| v.encryption_configuration()).and_then(|v| v.replica_kms_key_id()),
        "filter_values_omitted":true
    })).collect();
    Ok(json!({"configured":true,"role_arn":config.role(),"rules":rules}))
}

#[tracing::instrument(skip_all)]
pub(super) async fn website(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            optional(
                client
                    .get_bucket_website()
                    .bucket(name)
                    .expected_bucket_owner(&ctx.account)
                    .send()
                    .await,
                &["NoSuchWebsiteConfiguration"],
            )
        })
        .await?;
    let Some(output) = output else {
        return Ok(json!({"configured":false}));
    };
    components(output.routing_rules().len(), "S3 website routing rules")?;
    Ok(json!({"configured":true,
        "redirect_host":output.redirect_all_requests_to().map(|v| v.host_name()),
        "redirect_protocol":output.redirect_all_requests_to().and_then(|v| v.protocol()).map(|v| v.as_str()),
        "index_document_configured":output.index_document().is_some(),
        "error_document_configured":output.error_document().is_some(),
        "routing_rule_count":output.routing_rules().len(), "object_key_values_omitted":true}))
}

#[tracing::instrument(skip_all)]
pub(super) async fn ownership(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            optional(
                client
                    .get_bucket_ownership_controls()
                    .bucket(name)
                    .expected_bucket_owner(&ctx.account)
                    .send()
                    .await,
                &["OwnershipControlsNotFoundError"],
            )
        })
        .await?;
    let Some(output) = output else {
        return Ok(json!({"configured":false}));
    };
    let config = output.ownership_controls().ok_or_else(|| {
        AwsScanError::InvalidData("GetBucketOwnershipControls lacks configuration".into())
    })?;
    components(config.rules().len(), "S3 ownership rules")?;
    Ok(
        json!({"configured":true,"rules":config.rules().iter().map(|v| v.object_ownership().as_str()).collect::<Vec<_>>()}),
    )
}

#[tracing::instrument(skip_all)]
pub(super) async fn notifications(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            client
                .get_bucket_notification_configuration()
                .bucket(name)
                .expected_bucket_owner(&ctx.account)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await?;
    let total = output
        .topic_configurations()
        .len()
        .saturating_add(output.queue_configurations().len())
        .saturating_add(output.lambda_function_configurations().len());
    components(total, "S3 notification configurations")?;
    let event_count = output
        .topic_configurations()
        .iter()
        .map(|v| v.events().len())
        .chain(
            output
                .queue_configurations()
                .iter()
                .map(|v| v.events().len()),
        )
        .chain(
            output
                .lambda_function_configurations()
                .iter()
                .map(|v| v.events().len()),
        )
        .fold(total, usize::saturating_add);
    components(event_count, "S3 notification configurations and events")?;
    let topics: Vec<Value> = output.topic_configurations().iter().map(|v| json!({
        "id":v.id(),"arn":v.topic_arn(),"events":v.events().iter().map(|v| v.as_str()).collect::<Vec<_>>()
    })).collect();
    let queues: Vec<Value> = output.queue_configurations().iter().map(|v| json!({
        "id":v.id(),"arn":v.queue_arn(),"events":v.events().iter().map(|v| v.as_str()).collect::<Vec<_>>()
    })).collect();
    let functions: Vec<Value> = output.lambda_function_configurations().iter().map(|v| json!({
        "id":v.id(),"arn":v.lambda_function_arn(),"events":v.events().iter().map(|v| v.as_str()).collect::<Vec<_>>()
    })).collect();
    Ok(
        json!({"topics":topics,"queues":queues,"functions":functions,
        "eventbridge_enabled":output.event_bridge_configuration().is_some(),"filter_values_omitted":true}),
    )
}
