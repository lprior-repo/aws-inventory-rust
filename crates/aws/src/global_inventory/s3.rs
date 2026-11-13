//! General-purpose buckets and selected control-plane configuration. No object
//! listing, object reads, raw bucket policy, ACL grants or credential material.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

use aws_sdk_s3::{
    Client,
    error::{ProvideErrorMetadata, SdkError},
    types::Bucket,
};
use aws_smithy_types::retry::ProvideErrorKind;
use aws_types::region::Region;
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::{Edge, Resource, Source};
use inventory_engine::limiter::ApiError;
use serde_json::{Value, json};

use super::{
    CHILD_CONCURRENCY, MAX_COMPONENTS, SeenIds, checked_text, components, dns_reference, edge_set,
    ensure_active, observed, reference, section, single,
};
use crate::{
    AwsScanError,
    common::{Context, PageBody, collect_bounded, paginate, require_budget, required, sdk_error},
};

const KIND: &str = "aws_s3_bucket";
const SECTIONS: [&str; 9] = [
    "versioning",
    "encryption",
    "public_access_block",
    "policy_status",
    "logging",
    "replication",
    "website",
    "ownership_controls",
    "notifications",
];

#[tracing::instrument(skip_all)]
pub(super) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let client = Client::new(&ctx.config);
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_s3_bucket_list", None, Source::Direct),
        |token, budget| {
            let seen = &seen;
            let client = &client;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("s3", "read", || async {
                        client
                            .list_buckets()
                            .max_buckets(ctx.page_size.clamp(1, 1_000))
                            .set_continuation_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.buckets().len(), budget)?;
                let names = output
                    .buckets()
                    .iter()
                    .map(|bucket| required(bucket.name(), "bucket name"))
                    .collect::<Result<Vec<_>, _>>()?;
                let count = seen.record(names.iter().map(String::as_str), budget)?;
                collect_bounded(output.buckets(), CHILD_CONCURRENCY, |bucket| {
                    collect_bucket(ctx, client, bucket)
                })
                .await?;
                Ok(PageBody {
                    next_token: output.continuation_token().map(str::to_owned),
                    listed_count: count,
                    ..PageBody::default()
                })
            }
        },
    )
    .await?;
    ensure_active(ctx)
}

#[tracing::instrument(skip_all)]
fn collect_bucket<'a>(
    ctx: &'a Context,
    bootstrap: &'a Client,
    bucket: &'a Bucket,
) -> BoxFuture<'a, Result<(), AwsScanError>> {
    Box::pin(async move {
        ensure_active(ctx)?;
        let name = required(bucket.name(), "bucket name")?;
        let scope = ctx.scope(KIND, Some(&name), Source::Direct);
        let fallback = match map_bucket(ctx, bucket, None, &[], None) {
            Ok(body) => body,
            Err(error) => return observed(ctx, scope, Err(error), None).await,
        };
        let location = resolve_location(ctx, bootstrap, bucket, &name).await;
        let result = match location {
            Ok(region) => bucket_details(&bucket_context(ctx, &region), bucket, &name).await,
            Err(error) => Err(error),
        };
        observed(ctx, scope, result, Some(fallback)).await
    })
}

fn bucket_context(ctx: &Context, region: &str) -> Context {
    Context {
        config: ctx
            .config
            .to_builder()
            .region(Region::new(region.to_owned()))
            .build(),
        region: region.to_owned(),
        ..ctx.clone()
    }
}

fn location_region(partition: &str, location: Option<&str>) -> Result<String, AwsScanError> {
    let region = match location {
        None | Some("") => "us-east-1",
        Some("EU") => "eu-west-1",
        Some(region) => region,
    };
    crate::validate_region(region)?;
    let partition_matches = match partition {
        "aws-cn" => region.starts_with("cn-"),
        "aws-us-gov" => region.starts_with("us-gov-"),
        "aws" => !region.starts_with("cn-") && !region.starts_with("us-gov-"),
        _ => false,
    };
    if !partition_matches {
        return Err(AwsScanError::InvalidData(
            "bucket region does not match the scanned partition".into(),
        ));
    }
    Ok(region.to_owned())
}

#[tracing::instrument(skip_all)]
async fn resolve_location(
    ctx: &Context,
    client: &Client,
    bucket: &Bucket,
    name: &str,
) -> Result<String, AwsScanError> {
    if let Some(region) = bucket.bucket_region() {
        return location_region(&ctx.partition, Some(region));
    }
    let region = ctx
        .execute("s3", "read", || async {
            let result = client
                .head_bucket()
                .bucket(name)
                .expected_bucket_owner(&ctx.account)
                .send()
                .await;
            match result {
                Ok(output) => Ok(output.bucket_region().map(str::to_owned)),
                Err(error) => {
                    // HeadBucket intentionally exposes region on redirects and
                    // denied responses. This establishes location only; every
                    // later metadata read still checks the expected owner.
                    let region = error.raw_response().and_then(|response| {
                        location_header(
                            response.status().as_u16(),
                            response.headers().get("x-amz-bucket-region"),
                        )
                    });
                    match region {
                        Some(region) => Ok(Some(region)),
                        None => Err(sdk_error(error)),
                    }
                }
            }
        })
        .await?;
    let region = required(region.as_deref(), "HeadBucket region")?;
    location_region(&ctx.partition, Some(&region))
}

fn location_header(status: u16, region: Option<&str>) -> Option<String> {
    if !matches!(status, 301 | 400 | 403 | 404) {
        return None;
    }
    region
        .filter(|region| crate::validate_region(region).is_ok())
        .map(str::to_owned)
}

#[tracing::instrument(skip_all)]
fn bucket_details<'a>(
    ctx: &'a Context,
    bucket: &'a Bucket,
    name: &'a str,
) -> BoxFuture<'a, Result<PageBody, AwsScanError>> {
    Box::pin(async move {
        let client = Client::new(&ctx.config);
        let mut work = Vec::with_capacity(SECTIONS.len());
        for key in SECTIONS {
            work.push(read_section(ctx, &client, name, key));
        }
        let values: Vec<(&'static str, Option<Value>)> =
            stream::iter(work).buffer_unordered(4).try_collect().await?;
        let tags = read_tags(ctx, &client, name).await?;
        map_bucket(ctx, bucket, Some(&ctx.region), &values, tags)
    })
}

fn read_section<'a>(
    ctx: &'a Context,
    client: &'a Client,
    name: &'a str,
    key: &'static str,
) -> BoxFuture<'a, Result<(&'static str, Option<Value>), AwsScanError>> {
    Box::pin(async move {
        let collector = match key {
            "versioning" => "aws_s3_bucket_versioning",
            "encryption" => "aws_s3_bucket_encryption",
            "public_access_block" => "aws_s3_bucket_public_access_block",
            "policy_status" => "aws_s3_bucket_policy_status",
            "logging" => "aws_s3_bucket_logging",
            "replication" => "aws_s3_bucket_replication",
            "website" => "aws_s3_bucket_website",
            "ownership_controls" => "aws_s3_bucket_ownership_controls",
            "notifications" => "aws_s3_bucket_notifications",
            _ => {
                return Err(AwsScanError::InvalidOptions(
                    "unknown S3 metadata section".into(),
                ));
            }
        };
        let (_, value) = section(ctx, name, collector, || async {
            match key {
                "versioning" => versioning(ctx, client, name).await,
                "encryption" => encryption(ctx, client, name).await,
                "public_access_block" => public_access_block(ctx, client, name).await,
                "policy_status" => policy_status(ctx, client, name).await,
                "logging" => logging(ctx, client, name).await,
                "replication" => replication(ctx, client, name).await,
                "website" => website(ctx, client, name).await,
                "ownership_controls" => ownership(ctx, client, name).await,
                "notifications" => notifications(ctx, client, name).await,
                _ => Err(AwsScanError::InvalidOptions(
                    "unknown S3 metadata section".into(),
                )),
            }
        })
        .await?;
        Ok((key, value))
    })
}

fn optional<T, E: ProvideErrorMetadata + ProvideErrorKind + Debug>(
    result: Result<T, SdkError<E>>,
    absent: &[&str],
) -> Result<Option<T>, ApiError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error)
            if error
                .as_service_error()
                .and_then(ProvideErrorMetadata::code)
                .is_some_and(|code| absent.contains(&code)) =>
        {
            Ok(None)
        }
        Err(error) => Err(sdk_error(error)),
    }
}

#[tracing::instrument(skip_all)]
async fn versioning(ctx: &Context, client: &Client, name: &str) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            client
                .get_bucket_versioning()
                .bucket(name)
                .expected_bucket_owner(&ctx.account)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await?;
    Ok(
        json!({"status":output.status().map(|v| v.as_str()), "mfa_delete":output.mfa_delete().map(|v| v.as_str())}),
    )
}

#[tracing::instrument(skip_all)]
async fn encryption(ctx: &Context, client: &Client, name: &str) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            optional(
                client
                    .get_bucket_encryption()
                    .bucket(name)
                    .expected_bucket_owner(&ctx.account)
                    .send()
                    .await,
                &["ServerSideEncryptionConfigurationNotFoundError"],
            )
        })
        .await?;
    let Some(output) = output else {
        return Ok(json!({"configured":false}));
    };
    let configuration = output
        .server_side_encryption_configuration()
        .ok_or_else(|| {
            AwsScanError::InvalidData("GetBucketEncryption lacks configuration".into())
        })?;
    components(configuration.rules().len(), "S3 encryption rules")?;
    let rules: Vec<Value> = configuration.rules().iter().map(|rule| json!({
        "algorithm":rule.apply_server_side_encryption_by_default().map(|v| v.sse_algorithm().as_str()),
        "kms_key_id":rule.apply_server_side_encryption_by_default().and_then(|v| v.kms_master_key_id()),
        "bucket_key_enabled":rule.bucket_key_enabled()
    })).collect();
    Ok(json!({"configured":true,"rules":rules}))
}

#[tracing::instrument(skip_all)]
async fn public_access_block(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            optional(
                client
                    .get_public_access_block()
                    .bucket(name)
                    .expected_bucket_owner(&ctx.account)
                    .send()
                    .await,
                &["NoSuchPublicAccessBlockConfiguration"],
            )
        })
        .await?;
    let Some(output) = output else {
        return Ok(json!({"configured":false}));
    };
    let config = output.public_access_block_configuration().ok_or_else(|| {
        AwsScanError::InvalidData("GetPublicAccessBlock lacks configuration".into())
    })?;
    Ok(
        json!({"configured":true,"block_public_acls":config.block_public_acls(), "ignore_public_acls":config.ignore_public_acls(),
        "block_public_policy":config.block_public_policy(), "restrict_public_buckets":config.restrict_public_buckets()}),
    )
}

#[tracing::instrument(skip_all)]
async fn policy_status(ctx: &Context, client: &Client, name: &str) -> Result<Value, AwsScanError> {
    let output = ctx
        .execute("s3", "read", || async {
            optional(
                client
                    .get_bucket_policy_status()
                    .bucket(name)
                    .expected_bucket_owner(&ctx.account)
                    .send()
                    .await,
                &["NoSuchBucketPolicy"],
            )
        })
        .await?;
    let Some(output) = output else {
        return Ok(json!({"configured":false}));
    };
    let status = output
        .policy_status()
        .ok_or_else(|| AwsScanError::InvalidData("GetBucketPolicyStatus lacks status".into()))?;
    Ok(json!({"configured":true,"is_public":status.is_public()}))
}

#[tracing::instrument(skip_all)]
async fn logging(ctx: &Context, client: &Client, name: &str) -> Result<Value, AwsScanError> {
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
async fn replication(ctx: &Context, client: &Client, name: &str) -> Result<Value, AwsScanError> {
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
async fn website(ctx: &Context, client: &Client, name: &str) -> Result<Value, AwsScanError> {
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
async fn ownership(ctx: &Context, client: &Client, name: &str) -> Result<Value, AwsScanError> {
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
async fn notifications(ctx: &Context, client: &Client, name: &str) -> Result<Value, AwsScanError> {
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

#[tracing::instrument(skip_all)]
async fn read_tags(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    let (_, value) = section(ctx, name, "aws_s3_bucket_tags", || async {
        let output = ctx
            .execute("s3", "read", || async {
                optional(
                    client
                        .get_bucket_tagging()
                        .bucket(name)
                        .expected_bucket_owner(&ctx.account)
                        .send()
                        .await,
                    &["NoSuchTagSet"],
                )
            })
            .await?;
        let tags = match output {
            None => BTreeMap::new(),
            Some(output) => {
                components(output.tag_set().len(), "S3 tags")?;
                output
                    .tag_set()
                    .iter()
                    .map(|tag| {
                        Ok((
                            checked_text(tag.key(), "S3 tag key")?,
                            tag.value().to_owned(),
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, AwsScanError>>()?
            }
        };
        Ok(json!(tags))
    })
    .await?;
    value
        .map(|value| {
            serde_json::from_value(value)
                .map_err(|_| AwsScanError::InvalidData("invalid mapped bucket tags".into()))
        })
        .transpose()
}

fn map_bucket(
    ctx: &Context,
    bucket: &Bucket,
    region: Option<&str>,
    sections: &[(&str, Option<Value>)],
    tags: Option<BTreeMap<String, String>>,
) -> Result<PageBody, AwsScanError> {
    let name = required(bucket.name(), "bucket name")?;
    validate_bucket_name(&name)?;
    let arn = format!("arn:{}:s3:::{name}", ctx.partition);
    if bucket.bucket_arn().is_some_and(|value| value != arn) {
        return Err(AwsScanError::InvalidData(
            "ListBuckets ARN disagrees with bucket identity".into(),
        ));
    }
    let mut keys = BTreeSet::new();
    if sections.len() > SECTIONS.len()
        || sections
            .iter()
            .any(|(key, _)| !SECTIONS.contains(key) || !keys.insert(*key))
    {
        return Err(AwsScanError::InvalidData(
            "invalid or repeated bucket metadata section".into(),
        ));
    }
    let mut unavailable: Vec<&str> = SECTIONS
        .iter()
        .copied()
        .filter(|key| {
            !sections
                .iter()
                .any(|(section, value)| section == key && value.is_some())
        })
        .collect();
    if tags.is_none() {
        unavailable.push("tags");
    }
    let created = bucket
        .creation_date()
        .map(|value| value.to_millis())
        .transpose()
        .map_err(|_| AwsScanError::InvalidData("bucket creation timestamp out of range".into()))?;
    let mut properties = json!({"name":name,"bucket_name":name,"creation_date":created,"bucket_region":region,
        "region_resolved":region.is_some(),"details_available":region.is_some() && unavailable.is_empty() && sections.len() == SECTIONS.len(),
        "tags_available":tags.is_some(),
        "unavailable_sections":unavailable,"object_data_collected":false,"raw_policy_collected":false,
        "observation_detail":if region.is_some() {"control_plane_metadata"} else {"list_only"}});
    let object = properties
        .as_object_mut()
        .ok_or_else(|| AwsScanError::InvalidData("bucket properties are not an object".into()))?;
    for key in SECTIONS {
        let value = sections
            .iter()
            .find(|(section, _)| *section == key)
            .and_then(|(_, value)| value.clone())
            .unwrap_or(Value::Null);
        object.insert(key.to_owned(), value);
    }
    let mut resource = ctx.resource(KIND, arn, Some(name), properties, tags.unwrap_or_default())?;
    resource.region = region.unwrap_or("global").to_owned();
    resource.validate()?;
    let edges = bucket_edges(ctx, &resource)?;
    Ok(single(resource, edges))
}

fn bucket_edges(ctx: &Context, resource: &Resource) -> Result<Vec<Edge>, AwsScanError> {
    let mut edges = Vec::new();
    if let Some(arn) = resource
        .properties
        .pointer("/replication/role_arn")
        .and_then(Value::as_str)
    {
        edges.push(reference(
            ctx,
            resource,
            arn,
            "ASSUMES_ROLE",
            json!({"field":"/replication/role_arn"}),
        )?);
    }
    if let Some(name) = resource
        .properties
        .pointer("/logging/target_bucket")
        .and_then(Value::as_str)
    {
        validate_bucket_name(name)?;
        edges.push(reference(
            ctx,
            resource,
            &format!("arn:{}:s3:::{name}", ctx.partition),
            "LOGS_TO",
            json!({"field":"logging.target_bucket","target_owner_resolution":"unknown"}),
        )?);
    }
    if let Some(host) = resource
        .properties
        .pointer("/website/redirect_host")
        .and_then(Value::as_str)
    {
        edges.push(reference(
            ctx,
            resource,
            &dns_reference(host)?,
            "REDIRECTS_TO_DNS",
            json!({"field":"website.redirect_host"}),
        )?);
    }
    for (path, field, relation) in [
        ("/encryption/rules", "kms_key_id", "ENCRYPTED_WITH"),
        ("/replication/rules", "destination_bucket", "REPLICATES_TO"),
        (
            "/replication/rules",
            "destination_kms_key_id",
            "REPLICA_ENCRYPTED_WITH",
        ),
        ("/notifications/topics", "arn", "NOTIFIES_TOPIC"),
        ("/notifications/queues", "arn", "NOTIFIES_QUEUE"),
        ("/notifications/functions", "arn", "NOTIFIES_FUNCTION"),
    ] {
        if let Some(values) = resource.properties.pointer(path).and_then(Value::as_array) {
            components(values.len(), "S3 references")?;
            for value in values {
                if let Some(arn) = value
                    .get(field)
                    .and_then(Value::as_str)
                    .filter(|value| value.starts_with("arn:"))
                {
                    if edges.len() >= MAX_COMPONENTS {
                        return Err(AwsScanError::ChildLimit);
                    }
                    edges.push(reference(ctx, resource, arn, relation, json!({"field":format!("{path}/{field}"),"configuration_id":value.get("id")}))?);
                }
            }
        }
    }
    edge_set(edges)
}

fn validate_bucket_name(name: &str) -> Result<(), AwsScanError> {
    // Preserve older general-purpose bucket names, including legacy uppercase
    // and underscore forms, while rejecting identifier/path injection.
    if name.is_empty()
        || name.len() > 255
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(AwsScanError::InvalidData("invalid bucket name".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::context;
    use super::*;

    #[test]
    fn legacy_s3_locations_are_normalized_and_partition_checked() -> Result<(), AwsScanError> {
        assert_eq!(location_region("aws", None)?, "us-east-1");
        assert_eq!(location_region("aws", Some("EU"))?, "eu-west-1");
        assert_eq!(location_region("aws-cn", Some("cn-north-1"))?, "cn-north-1");
        assert!(location_region("aws", Some("cn-north-1")).is_err());
        assert!(location_region("aws-cn", None).is_err());
        assert!(location_region("aws", Some("https://wrong")).is_err());
        assert_eq!(
            location_header(301, Some("eu-west-1")),
            Some("eu-west-1".into())
        );
        assert_eq!(
            location_header(403, Some("us-east-1")),
            Some("us-east-1".into())
        );
        assert_eq!(location_header(503, Some("us-east-1")), None);
        assert_eq!(location_header(429, Some("us-east-1")), None);
        assert_eq!(location_header(400, Some("https://wrong")), None);
        Ok(())
    }

    #[test]
    fn bucket_location_is_real_region_while_arn_stays_global()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let bucket = Bucket::builder()
            .name("synthetic-bucket")
            .bucket_region("eu-west-1")
            .build();
        let body = map_bucket(&ctx, &bucket, Some("eu-west-1"), &[], Some(BTreeMap::new()))?;
        let resource = body.resources.first().ok_or("missing bucket")?;
        assert_eq!(resource.id, "arn:aws:s3:::synthetic-bucket");
        assert_eq!(resource.region, "eu-west-1");
        assert_eq!(
            resource.properties.get("details_available"),
            Some(&json!(false))
        );
        assert_eq!(
            resource.properties.get("object_data_collected"),
            Some(&json!(false))
        );
        Ok(())
    }

    #[test]
    fn absent_and_denied_policy_sections_remain_distinct() -> Result<(), Box<dyn std::error::Error>>
    {
        let (ctx, _rx) = context()?;
        let bucket = Bucket::builder().name("synthetic-bucket").build();
        let denied = map_bucket(
            &ctx,
            &bucket,
            Some("us-east-1"),
            &[("policy_status", None)],
            Some(BTreeMap::new()),
        )?;
        let absent = map_bucket(
            &ctx,
            &bucket,
            Some("us-east-1"),
            &[("policy_status", Some(json!({"configured":false})))],
            Some(BTreeMap::new()),
        )?;
        assert_eq!(
            denied
                .resources
                .first()
                .and_then(|r| r.properties.get("policy_status")),
            Some(&Value::Null)
        );
        assert_eq!(
            absent
                .resources
                .first()
                .and_then(|r| r.properties.pointer("/policy_status/configured")),
            Some(&json!(false))
        );
        Ok(())
    }

    #[test]
    fn bucket_architecture_edges_keep_explicit_arns_and_deduplicate()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let bucket = Bucket::builder().name("synthetic-bucket").build();
        let sections = [
            (
                "encryption",
                Some(
                    json!({"rules":[{"kms_key_id":"arn:aws:kms:us-east-1:123456789012:key/abc"},{"kms_key_id":"alias/unknown-owner"}]}),
                ),
            ),
            (
                "replication",
                Some(
                    json!({"role_arn":"arn:aws:iam::123456789012:role/replication","rules":[{"destination_bucket":"arn:aws:s3:::remote-bucket"}]}),
                ),
            ),
            (
                "notifications",
                Some(
                    json!({"topics":[{"arn":"arn:aws:sns:us-east-1:123456789012:topic"},{"arn":"arn:aws:sns:us-east-1:123456789012:topic"}]}),
                ),
            ),
            ("logging", Some(json!({"target_bucket":"log-bucket"}))),
        ];
        let body = map_bucket(
            &ctx,
            &bucket,
            Some("us-east-1"),
            &sections,
            Some(BTreeMap::new()),
        )?;
        assert_eq!(body.edges.len(), 5);
        assert!(
            body.edges
                .iter()
                .all(|e| !e.to_id.contains("alias/unknown-owner"))
        );
        let topic = body
            .edges
            .iter()
            .find(|e| e.relation == "NOTIFIES_TOPIC")
            .ok_or("missing topic edge")?;
        assert_eq!(
            topic
                .properties
                .get("evidence")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        Ok(())
    }

    #[test]
    fn missing_tags_and_sections_cannot_be_reported_complete()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let bucket = Bucket::builder().name("synthetic-bucket").build();
        let sections: Vec<_> = SECTIONS
            .into_iter()
            .map(|key| (key, Some(json!({}))))
            .collect();
        let unavailable = map_bucket(&ctx, &bucket, Some("us-east-1"), &sections, None)?;
        let available = map_bucket(
            &ctx,
            &bucket,
            Some("us-east-1"),
            &sections,
            Some(BTreeMap::new()),
        )?;
        assert_eq!(
            unavailable
                .resources
                .first()
                .and_then(|r| r.properties.get("details_available")),
            Some(&json!(false))
        );
        assert_eq!(
            unavailable
                .resources
                .first()
                .and_then(|r| r.properties.get("unavailable_sections")),
            Some(&json!(["tags"]))
        );
        assert_eq!(
            available
                .resources
                .first()
                .and_then(|r| r.properties.get("details_available")),
            Some(&json!(true))
        );
        let repeated = [
            ("versioning", Some(json!({}))),
            ("versioning", Some(json!({}))),
        ];
        assert!(map_bucket(&ctx, &bucket, Some("us-east-1"), &repeated, None).is_err());
        Ok(())
    }

    #[test]
    fn conflicting_s3_arn_is_not_silently_accepted() -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let bucket = Bucket::builder()
            .name("one")
            .bucket_arn("arn:aws:s3:::two")
            .build();
        assert!(map_bucket(&ctx, &bucket, None, &[], None).is_err());
        Ok(())
    }
}
