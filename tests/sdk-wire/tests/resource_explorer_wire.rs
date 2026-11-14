 
 
 
 
 
 
#![forbid(unsafe_code)]

use std::fmt::Debug;
use std::time::Duration;

use aws_sdk_resourceexplorer2::Client;
use aws_sdk_resourceexplorer2::config::{BehaviorVersion, Credentials, Region, retry::RetryConfig};
use aws_sdk_resourceexplorer2::error::{ProvideErrorMetadata, SdkError};
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::DateTime;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::timeout::TimeoutConfig;
use serde_json::{Value, json};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const ENDPOINT: &str = "https://resource-explorer.fixture.invalid";
const VIEW: &str = "arn:aws:resource-explorer-2:us-east-1:000000000000:view/wire-fixture/00000000-0000-0000-0000-000000000002";
const RESOURCE: &str = "arn:aws:sqs:us-east-1:000000000000:wire-fixture";
const CREATED_MILLIS: i64 = 1_704_164_645_006;
const UPDATED_MILLIS: i64 = 1_704_164_646_007;
const NUMERIC_EPOCH: f64 = 1_704_164_645.006;
const INDEX_BODY: &str = include_str!("../fixtures/get_index_iso8601.json");
const VIEW_BODY: &str = include_str!("../fixtures/get_view_iso8601.json");
const RESOURCES_BODY: &str = include_str!("../fixtures/list_resources_iso8601.json");

fn replay_client(
    operation: &str,
    expected_body: Option<Value>,
    response_body: Value,
) -> Result<(Client, StaticReplayClient)> {
    let expected_body = match expected_body {
        Some(value) => serde_json::to_vec(&value)?,
        None => Vec::new(),
    };
    let request = http::Request::builder()
        .method("POST")
        .uri(format!("{ENDPOINT}/{operation}"))
        .body(SdkBody::from(expected_body))?;
    let response = http::Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("x-amzn-requestid", "offline-wire-fixture")
        .body(SdkBody::from(serde_json::to_vec(&response_body)?))?;
    let replay = StaticReplayClient::new(vec![ReplayEvent::new(request, response)]);
    let config = aws_sdk_resourceexplorer2::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url(ENDPOINT)
         
         
        .credentials_provider(Credentials::new(
            "offline-wire-fixture-access-key",
            "offline-wire-fixture-secret",
            None,
            None,
            "offline-wire-fixture",
        ))
        .retry_config(RetryConfig::standard().with_max_attempts(1))
        .timeout_config(
            TimeoutConfig::builder()
                .operation_timeout(Duration::from_secs(2))
                .operation_attempt_timeout(Duration::from_secs(2))
                .build(),
        )
        .http_client(replay.clone())
        .build();
    Ok((Client::from_conf(config), replay))
}

fn assert_one_request(replay: &StaticReplayClient) -> Result<()> {
    {
        let mut requests = replay.actual_requests();
        let request = requests
            .next()
            .ok_or("SDK did not send the replay request")?;
        assert_eq!(request.method(), "POST");
        assert!(
            requests.next().is_none(),
            "a second SDK request was recorded"
        );
    }
     
     
    replay.assert_requests_match(&[]);
    Ok(())
}

fn millis(value: Option<&DateTime>) -> Result<i64> {
    Ok(value.ok_or("decoded timestamp was absent")?.to_millis()?)
}

fn with_numeric_timestamp(body: &str, pointer: &str) -> Result<Value> {
    let mut value: Value = serde_json::from_str(body)?;
    let timestamp = value
        .pointer_mut(pointer)
        .ok_or("timestamp pointer is missing from the positive fixture")?;
    *timestamp = json!(NUMERIC_EPOCH);
    Ok(value)
}

fn assert_numeric_rejected<T, E: ProvideErrorMetadata + Debug>(
    result: std::result::Result<T, SdkError<E>>,
    replay: &StaticReplayClient,
) -> Result<()> {
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err("numeric epoch timestamp was accepted as a modeled date-time".into()),
    };
     
     
    assert_eq!(
        error
            .raw_response()
            .map(|response| response.status().as_u16()),
        Some(200)
    );
    let service = error
        .as_service_error()
        .ok_or("expected the SDK's unhandled output-deserialization error")?;
    assert!(service.code().is_none());
    assert_one_request(replay)
}

#[tokio::test]
async fn get_index_decodes_modeled_iso8601_timestamps() -> Result<()> {
    let (client, replay) = replay_client("GetIndex", None, serde_json::from_str(INDEX_BODY)?)?;
    let output = client.get_index().send().await?;
    assert_eq!(output.state().map(|state| state.as_str()), Some("ACTIVE"));
    assert_eq!(output.r#type().map(|kind| kind.as_str()), Some("LOCAL"));
    assert_eq!(millis(output.created_at())?, CREATED_MILLIS);
    assert_eq!(millis(output.last_updated_at())?, UPDATED_MILLIS);
    assert_one_request(&replay)
}

#[tokio::test]
async fn get_index_rejects_each_numeric_timestamp_field() -> Result<()> {
    for pointer in ["/CreatedAt", "/LastUpdatedAt"] {
        let (client, replay) = replay_client(
            "GetIndex",
            None,
            with_numeric_timestamp(INDEX_BODY, pointer)?,
        )?;
        assert_numeric_rejected(client.get_index().send().await, &replay)?;
    }
    Ok(())
}

#[tokio::test]
async fn get_view_decodes_modeled_iso8601_timestamp() -> Result<()> {
    let (client, replay) = replay_client(
        "GetView",
        Some(json!({"ViewArn":VIEW})),
        serde_json::from_str(VIEW_BODY)?,
    )?;
    let output = client.get_view().view_arn(VIEW).send().await?;
    let view = output.view().ok_or("GetView omitted the fixture view")?;
    assert_eq!(view.view_arn(), Some(VIEW));
    assert_eq!(view.owner(), Some("000000000000"));
    assert_eq!(millis(view.last_updated_at())?, CREATED_MILLIS);
    assert_one_request(&replay)
}

#[tokio::test]
async fn get_view_rejects_numeric_timestamp() -> Result<()> {
    let (client, replay) = replay_client(
        "GetView",
        Some(json!({"ViewArn":VIEW})),
        with_numeric_timestamp(VIEW_BODY, "/View/LastUpdatedAt")?,
    )?;
    assert_numeric_rejected(client.get_view().view_arn(VIEW).send().await, &replay)
}

#[tokio::test]
async fn list_resources_decodes_resource_and_property_iso8601_timestamps() -> Result<()> {
    let (client, replay) = replay_client(
        "ListResources",
        Some(json!({"ViewArn":VIEW,"MaxResults":1})),
        serde_json::from_str(RESOURCES_BODY)?,
    )?;
    let output = client
        .list_resources()
        .view_arn(VIEW)
        .max_results(1)
        .send()
        .await?;
    assert_eq!(output.view_arn(), Some(VIEW));
    assert_eq!(output.next_token(), Some("offline-fixture-next-page"));
    assert_eq!(output.resources().len(), 1);
    let resource = output
        .resources()
        .first()
        .ok_or("resource fixture was not decoded")?;
    assert_eq!(resource.arn(), Some(RESOURCE));
    assert_eq!(millis(resource.last_reported_at())?, CREATED_MILLIS);
    let property = resource
        .properties()
        .first()
        .ok_or("resource property fixture was not decoded")?;
    assert_eq!(property.name(), Some("tags"));
    assert_eq!(millis(property.last_reported_at())?, UPDATED_MILLIS);
    assert_one_request(&replay)
}

#[tokio::test]
async fn list_resources_rejects_each_numeric_timestamp_level() -> Result<()> {
    for pointer in [
        "/Resources/0/LastReportedAt",
        "/Resources/0/Properties/0/LastReportedAt",
    ] {
        let (client, replay) = replay_client(
            "ListResources",
            Some(json!({"ViewArn":VIEW,"MaxResults":1})),
            with_numeric_timestamp(RESOURCES_BODY, pointer)?,
        )?;
        assert_numeric_rejected(
            client
                .list_resources()
                .view_arn(VIEW)
                .max_results(1)
                .send()
                .await,
            &replay,
        )?;
    }
    Ok(())
}
