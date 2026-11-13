//! Safe property projections shared by state machines and activities. Logging
//! destinations are retained as properties only; no CloudWatch log-group edge
//! or definition body is produced.

use std::collections::BTreeMap;

use aws_sdk_sfn::types::{
    EncryptionConfiguration, LoggingConfiguration, Tag, TracingConfiguration,
};
use aws_smithy_types::DateTime;
use serde_json::{Value, json};

use crate::AwsScanError;

const MAX_TAG_BYTES: usize = 4_096;

pub(super) fn tags_from(tags: &[Tag]) -> BTreeMap<String, String> {
    tags.iter()
        .filter_map(|tag| {
            let key = tag.key()?;
            let value = tag.value().unwrap_or_default();
            let valid = !key.is_empty()
                && key.len() <= MAX_TAG_BYTES
                && !key.contains('\0')
                && value.len() <= MAX_TAG_BYTES
                && !value.contains('\0');
            valid.then(|| (key.to_owned(), value.to_owned()))
        })
        .collect()
}

pub(super) fn millis(value: &DateTime, label: &str) -> Result<i64, AwsScanError> {
    value.to_millis().map_err(|_| {
        AwsScanError::InvalidData(format!("Step Functions {label} timestamp is out of range"))
    })
}

pub(super) fn encryption(config: Option<&EncryptionConfiguration>) -> Value {
    config.map_or(Value::Null, |config| {
        json!({
            "type": config.r#type().as_str(),
            "kms_key_id": config.kms_key_id(),
            "kms_data_key_reuse_period_seconds": config.kms_data_key_reuse_period_seconds(),
        })
    })
}

pub(super) fn logging(config: Option<&LoggingConfiguration>) -> Value {
    config.map_or(Value::Null, |config| {
        let groups: Vec<&str> = config
            .destinations()
            .iter()
            .filter_map(|destination| {
                destination
                    .cloud_watch_logs_log_group()
                    .and_then(|group| group.log_group_arn())
            })
            .collect();
        json!({
            "level": config.level().map(|level| level.as_str()),
            "include_execution_data": config.include_execution_data(),
            "log_group_arns": groups,
        })
    })
}

pub(super) fn tracing(config: Option<&TracingConfiguration>) -> Value {
    config.map_or(Value::Null, |config| json!({"enabled": config.enabled()}))
}

pub(super) fn invalid(message: impl Into<String>) -> AwsScanError {
    AwsScanError::InvalidData(message.into())
}
