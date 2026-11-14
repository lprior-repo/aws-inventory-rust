use super::*;

pub fn account_role_arn(
    partition: &str,
    account_id: &str,
    role_name_or_path: &str,
) -> Result<String, AwsScanError> {
    validate_partition(partition)?;
    validate_account_id(account_id)?;
    validate_role_path(role_name_or_path)?;
    Ok(format!(
        "arn:{partition}:iam::{account_id}:role/{role_name_or_path}"
    ))
}

#[tracing::instrument(skip_all)]
pub async fn assume_role_config(
    base: &SdkConfig,
    options: &AssumeRoleOptions,
    executor: &ApiExecutor,
) -> Result<SdkConfig, AwsScanError> {
    options.validate()?;
    ensure_running(executor)?;
    if base.credentials_provider().is_none()
        || base.time_source().is_none()
        || base.region().is_none()
    {
        return invalid(
            "AssumeRole requires a loaded base SDK configuration with credentials, time and region providers",
        );
    }
    let mut builder = base
        .to_builder()
        .behavior_version(BehaviorVersion::latest())
        .retry_config(RetryConfig::standard().with_max_attempts(1));

    builder.insert_origin("retry_config", aws_types::origin::Origin::shared_config());
    let base = builder.build();
    let credentials_config = base
        .to_builder()
        .timeout_config(credential_timeouts(base.timeout_config()))
        .build();
    let mut builder = AssumeRoleProvider::builder(&options.role_arn)
        .configure(&credentials_config)
        .session_name(&options.session_name)
        .session_length(Duration::from_secs(u64::from(options.duration_seconds)));
    if let Some(external_id) = &options.external_id {
        builder = builder.external_id(external_id);
    }
    let provider = tokio::select! {
        biased;
        () = executor.cancelled() => return Err(ApiError::Cancelled.into()),
        result = tokio::time::timeout(CONFIGURATION_TIMEOUT, builder.build()) =>
            result.map_err(|_| ApiError::Timeout)?,
    };
    Ok(base
        .to_builder()
        .credentials_provider(SharedCredentialsProvider::new(provider))
        .build())
}

pub(super) fn credential_timeouts(existing: Option<&TimeoutConfig>) -> TimeoutConfig {
    let bound = |value: Option<Duration>, limit: Duration| value.unwrap_or(limit).min(limit);
    TimeoutConfig::builder()
        .operation_timeout(bound(
            existing.and_then(TimeoutConfig::operation_timeout),
            CONFIGURATION_TIMEOUT,
        ))
        .operation_attempt_timeout(bound(
            existing.and_then(TimeoutConfig::operation_attempt_timeout),
            CONFIGURATION_TIMEOUT,
        ))
        .connect_timeout(bound(
            existing.and_then(TimeoutConfig::connect_timeout),
            Duration::from_secs(10),
        ))
        .read_timeout(bound(
            existing.and_then(TimeoutConfig::read_timeout),
            CONFIGURATION_TIMEOUT,
        ))
        .build()
}

pub(super) fn validate_role_arn(arn: &str) -> Result<(), AwsScanError> {
    bounded_text(arn, "role ARN", 2_048)?;
    let mut fields = arn.splitn(6, ':');
    if fields.next() != Some("arn") {
        return invalid("role ARN must start with arn");
    }
    let partition = fields
        .next()
        .ok_or_else(|| data_error("missing role ARN partition"))?;
    validate_partition(partition)?;
    if fields.next() != Some("iam") || fields.next() != Some("") {
        return invalid("role ARN must name IAM without a region");
    }
    validate_account_id(
        fields
            .next()
            .ok_or_else(|| data_error("missing role ARN account"))?,
    )?;
    let path = fields
        .next()
        .and_then(|value| value.strip_prefix("role/"))
        .ok_or_else(|| data_error("ARN must identify an IAM role"))?;
    validate_role_path(path)
}

pub(super) fn validate_role_path(path: &str) -> Result<(), AwsScanError> {
    if path.is_empty()
        || path.len() > 575
        || !path
            .bytes()
            .all(|byte| role_character(byte) || byte == b'/')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
        || path.rsplit('/').next().is_none_or(|name| name.len() > 64)
        || path
            .rsplit_once('/')
            .is_some_and(|(prefix, _)| prefix.len().saturating_add(2) > 512)
    {
        return invalid("invalid IAM role name/path");
    }
    Ok(())
}

pub(super) fn role_character(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'+' | b'=' | b',' | b'.' | b'@' | b'-')
}
