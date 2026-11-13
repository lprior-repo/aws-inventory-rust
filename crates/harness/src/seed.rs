//! Minimal fixture seeding through the real SDK transport.
//!
//! These run inside a child process whose environment already points at the
//! owned emulator (`AWS_ENDPOINT_URL`, generated credentials, TLS authority), so
//! the SDK performs normal request signing and response decoding. Nothing here
//! is a shortcut around the collector's own path.

use serde_json::{Value, json};

use crate::HarnessError;

const SEED_POLL_LIMIT: u8 = 40;
const SEED_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(150);

fn require_seed(prefix: &str, count: usize) -> Result<(), HarnessError> {
    if count == 0 || count > 20 || !prefix.starts_with("inventory-page-") {
        return Err(HarnessError::invalid(
            "Cloud Control seeding needs 1..=20 topics and an inventory-page- prefix",
        ));
    }
    Ok(())
}

async fn create_topic(
    client: &aws_sdk_cloudcontrol::Client,
    name: &str,
) -> Result<(), HarnessError> {
    let created = client
        .create_resource()
        .type_name("AWS::SNS::Topic")
        .desired_state(format!(r#"{{"TopicName":"{name}"}}"#))
        .send()
        .await
        .map_err(sdk)?;
    let token = created
        .progress_event()
        .and_then(|event| event.request_token())
        .ok_or_else(|| HarnessError::invalid("create returned no request token"))?;
    wait_until_available(client, token).await
}

async fn listed_identifiers(
    client: &aws_sdk_cloudcontrol::Client,
) -> Result<Vec<String>, HarnessError> {
    let listed = client
        .list_resources()
        .type_name("AWS::SNS::Topic")
        .send()
        .await
        .map_err(sdk)?;
    Ok(listed
        .resource_descriptions()
        .iter()
        .filter_map(|description| description.identifier().map(str::to_owned))
        .collect())
}

fn require_listed(names: &[String], observed: &[String]) -> Result<(), HarnessError> {
    for name in names {
        if !observed.contains(name) {
            return Err(HarnessError::invalid(format!(
                "seeded topic {name} is not listed by the emulator"
            )));
        }
    }
    Ok(())
}

/// Create SNS topics through Cloud Control and confirm the emulator lists them.
///
/// Used by the live pagination gate: several resources of one type force the
/// collector to follow continuation tokens instead of trusting one page.
pub async fn cloud_control_topics(prefix: &str, count: usize) -> Result<Value, HarnessError> {
    require_seed(prefix, count)?;
    let shared = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .load()
        .await;
    let client = aws_sdk_cloudcontrol::Client::new(&shared);
    let mut names = Vec::new();
    for index in 0..count {
        let name = format!("{prefix}-{index}");
        create_topic(&client, &name).await?;
        names.push(name);
    }
    let observed = listed_identifiers(&client).await?;
    require_listed(&names, &observed)?;
    Ok(json!({"prefix": prefix, "topics": names, "listed": observed}))
}

async fn progress_event(
    client: &aws_sdk_cloudcontrol::Client,
    token: &str,
) -> Result<aws_sdk_cloudcontrol::types::ProgressEvent, HarnessError> {
    let status = client
        .get_resource_request_status()
        .request_token(token)
        .send()
        .await
        .map_err(sdk)?;
    status
        .progress_event()
        .cloned()
        .ok_or_else(|| HarnessError::invalid("status returned no progress event"))
}

async fn wait_until_available(
    client: &aws_sdk_cloudcontrol::Client,
    token: &str,
) -> Result<(), HarnessError> {
    for _ in 0..SEED_POLL_LIMIT {
        let event = progress_event(client, token).await?;
        match event.operation_status().map(|status| status.as_str()) {
            Some("SUCCESS") => return Ok(()),
            Some("FAILED") => {
                return Err(HarnessError::invalid(format!(
                    "Cloud Control create failed: {:?}",
                    event.error_code()
                )));
            }
            _ => {}
        }
        tokio::time::sleep(SEED_POLL_INTERVAL).await;
    }
    Err(HarnessError::invalid(
        "Cloud Control resource did not become available",
    ))
}

async fn s3_client() -> aws_sdk_s3::Client {
    let shared = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .load()
        .await;
    aws_sdk_s3::Client::from_conf(
        aws_sdk_s3::config::Builder::from(&shared)
            .force_path_style(true)
            .build(),
    )
}

fn require_bucket(name: &str, buckets: &[&str]) -> Result<(), HarnessError> {
    if !buckets.contains(&name) {
        return Err(HarnessError::invalid(
            "seeded bucket is not listed by the emulator",
        ));
    }
    Ok(())
}

/// Create one S3 bucket and confirm the emulator lists it back.
pub async fn s3_bucket(name: &str) -> Result<Value, HarnessError> {
    let client = s3_client().await;
    client
        .create_bucket()
        .bucket(name)
        .send()
        .await
        .map_err(sdk)?;
    let listed = client.list_buckets().send().await.map_err(sdk)?;
    let buckets: Vec<&str> = listed
        .buckets()
        .iter()
        .filter_map(|bucket| bucket.name())
        .collect();
    require_bucket(name, &buckets)?;
    Ok(json!({"bucket": name, "buckets": buckets}))
}

fn sdk(error: impl std::fmt::Display) -> HarnessError {
    HarnessError::invalid(error.to_string())
}
