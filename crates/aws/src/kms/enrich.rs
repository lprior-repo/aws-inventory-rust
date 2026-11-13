//! Per-key enrichment: describe, customer-managed rotation, and tags. Every
//! child failure is recorded in its own scope and never discards the key.

use std::collections::BTreeMap;

use aws_sdk_kms::{
    Client,
    types::{KeyListEntry, KeyManagerType, KeyMetadata},
};
use inventory_core::Resource;

use super::arn;
use super::failure::{fail, failure};
use super::key::{Rotation, map_key, tags_from};
use crate::{
    AwsScanError,
    common::{Context, sdk_error},
};

/// KMS permits at most 50 tags on one key, so a truncated listing is anomalous.
pub(super) const MAX_TAGS_PER_KEY: i32 = 50;

/// Which per-key child reads apply to a key. AWS-managed keys cannot be tagged
/// and their rotation is not customer-configurable, so both reads are skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Enrichment {
    pub(super) rotation: bool,
    pub(super) tags: bool,
}

pub(super) fn enrichment(metadata: &KeyMetadata) -> Enrichment {
    let customer_managed = matches!(metadata.key_manager(), Some(KeyManagerType::Customer));
    Enrichment {
        rotation: customer_managed,
        tags: customer_managed,
    }
}

pub(super) async fn enrich_key(
    ctx: &Context,
    client: &Client,
    entry: &KeyListEntry,
) -> Result<Option<Resource>, AwsScanError> {
    let Some(arn) = entry.key_arn().filter(|value| !value.is_empty()) else {
        return fail(
            ctx,
            "aws_kms_key",
            None,
            AwsScanError::InvalidData("ListKeys lacks KeyArn".into()),
        )
        .await;
    };
    let key_id = match arn::split_key_arn(arn) {
        Ok((parts, key_id)) if arn::is_in_scope(&parts, ctx) => key_id,
        Ok(_) => {
            return fail(
                ctx,
                "aws_kms_key",
                None,
                AwsScanError::InvalidData("KMS key ARN is outside the scan scope".into()),
            )
            .await;
        }
        Err(message) => {
            return fail(ctx, "aws_kms_key", None, AwsScanError::InvalidData(message)).await;
        }
    };
    if entry.key_id().is_some_and(|value| value != key_id) {
        return fail(
            ctx,
            "aws_kms_key",
            None,
            AwsScanError::InvalidData("ListKeys KeyId does not match KeyArn".into()),
        )
        .await;
    }
    let Some(metadata) = describe_key(ctx, client, arn).await? else {
        return Ok(None);
    };
    let plan = enrichment(&metadata);
    let rotation = if plan.rotation {
        key_rotation(ctx, client, arn).await?
    } else {
        None
    };
    let tags = if plan.tags {
        key_tags(ctx, client, arn).await?
    } else {
        None
    };
    match map_key(ctx, arn, &metadata, rotation, tags) {
        Ok(resource) => Ok(Some(resource)),
        Err(error) => fail(ctx, "aws_kms_key", Some(arn), error).await,
    }
}

async fn describe_key(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<KeyMetadata>, AwsScanError> {
    let result = ctx
        .execute("kms", "control_read", || async {
            client
                .describe_key()
                .key_id(arn)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match result {
        Ok(output) => match output.key_metadata() {
            Some(metadata) => Ok(Some(metadata.clone())),
            None => {
                let error = AwsScanError::InvalidData("DescribeKey lacks KeyMetadata".into());
                failure(ctx, "aws_kms_key_describe", Some(arn), &error).await?;
                Ok(None)
            }
        },
        Err(error) => {
            failure(
                ctx,
                "aws_kms_key_describe",
                Some(arn),
                &AwsScanError::Api(error),
            )
            .await?;
            Ok(None)
        }
    }
}

/// Automatic rotation is configurable only for customer-managed keys. Failure is
/// recorded in a child scope and the key is retained without a rotation claim.
async fn key_rotation(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<Rotation>, AwsScanError> {
    let result = ctx
        .execute("kms", "control_read", || async {
            client
                .get_key_rotation_status()
                .key_id(arn)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match result {
        Ok(output) => Ok(Some(Rotation {
            enabled: output.key_rotation_enabled(),
            period_in_days: output.rotation_period_in_days(),
        })),
        Err(error) => {
            failure(
                ctx,
                "aws_kms_key_rotation",
                Some(arn),
                &AwsScanError::Api(error),
            )
            .await?;
            Ok(None)
        }
    }
}

/// Tag failure is recorded but does not discard an otherwise valid key.
async fn key_tags(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    let result = ctx
        .execute("kms", "control_read", || async {
            client
                .list_resource_tags()
                .key_id(arn)
                .limit(MAX_TAGS_PER_KEY)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match result {
        Ok(output) if !output.truncated() => Ok(Some(tags_from(output.tags()))),
        Ok(_) => {
            let error = AwsScanError::InvalidData(
                "ListResourceTags exceeded the 50-tag KMS key quota".into(),
            );
            failure(ctx, "aws_kms_key_tags", Some(arn), &error).await?;
            Ok(None)
        }
        Err(error) => {
            failure(
                ctx,
                "aws_kms_key_tags",
                Some(arn),
                &AwsScanError::Api(error),
            )
            .await?;
            Ok(None)
        }
    }
}
