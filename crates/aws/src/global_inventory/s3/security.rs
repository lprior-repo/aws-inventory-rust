//! security responsibilities for s3.
use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn versioning(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
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
pub(super) async fn encryption(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
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
pub(super) async fn public_access_block(
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
pub(super) async fn policy_status(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Value, AwsScanError> {
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
