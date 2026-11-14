use serde_json::{Value, json};

use crate::HarnessError;

const SEED_POLL_LIMIT: u8 = 40;
const SEED_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(150);

struct SeededTopic {
    name: String,
    identifier: String,
}

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
) -> Result<String, HarnessError> {
    let created = client
        .create_resource()
        .type_name("AWS::SNS::Topic")
        .desired_state(format!(r#"{{"TopicName":"{name}"}}"#))
        .send()
        .await
        .map_err(sdk)?;
    let event = created
        .progress_event()
        .ok_or_else(|| HarnessError::invalid("create returned no progress event"))?;
    if let Some(identifier) = completed_identifier(event)? {
        return Ok(identifier.to_owned());
    }
    let token = event
        .request_token()
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

fn topic_report(
    prefix: &str,
    topics: &[SeededTopic],
    observed: Vec<String>,
) -> Result<Value, HarnessError> {
    topics.iter().try_for_each(|topic| {
        if observed.contains(&topic.identifier) {
            return Ok(());
        }
        Err(HarnessError::invalid(format!(
            "seeded topic {} ({}) is not listed by the emulator",
            topic.name, topic.identifier
        )))
    })?;
    let names: Vec<&str> = topics.iter().map(|topic| topic.name.as_str()).collect();
    Ok(json!({"prefix": prefix, "topics": names, "listed": observed}))
}

pub async fn cloud_control_topics(prefix: &str, count: usize) -> Result<Value, HarnessError> {
    require_seed(prefix, count)?;
    let shared = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .load()
        .await;
    let client = aws_sdk_cloudcontrol::Client::new(&shared);
    let mut topics = Vec::with_capacity(count);
    for index in 0..count {
        let name = format!("{prefix}-{index}");
        let identifier = create_topic(&client, &name).await?;
        topics.push(SeededTopic { name, identifier });
    }
    let observed = listed_identifiers(&client).await?;
    topic_report(prefix, &topics, observed)
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

fn completed_identifier(
    event: &aws_sdk_cloudcontrol::types::ProgressEvent,
) -> Result<Option<&str>, HarnessError> {
    match event.operation_status().map(|status| status.as_str()) {
        Some("SUCCESS") => event
            .identifier()
            .filter(|identifier| !identifier.is_empty())
            .map(Some)
            .ok_or_else(|| {
                HarnessError::invalid(
                    "Cloud Control create succeeded without a resource identifier",
                )
            }),
        Some("FAILED") => Err(HarnessError::invalid(format!(
            "Cloud Control create failed: {:?}",
            event.error_code()
        ))),
        _ => Ok(None),
    }
}

async fn wait_until_available(
    client: &aws_sdk_cloudcontrol::Client,
    token: &str,
) -> Result<String, HarnessError> {
    for _ in 0..SEED_POLL_LIMIT {
        let event = progress_event(client, token).await?;
        if let Some(identifier) = completed_identifier(&event)? {
            return Ok(identifier.to_owned());
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

#[cfg(test)]
mod tests;
