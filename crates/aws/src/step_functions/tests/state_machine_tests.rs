use std::collections::BTreeMap;

use aws_sdk_sfn::types::IncludedData;
use serde_json::{Value, json};

use super::*;

#[test]
fn maps_state_machine_identity_and_safe_metadata() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?;
    let described =
        state_machine_description(STATE_MACHINE_ARN, STATE_MACHINE_NAME, Some(KEY_ARN))?;
    let tags = BTreeMap::from([("team".to_owned(), "orders".to_owned())]);
    let observation = map_state_machine(&ctx, &item, Some(&described), Some(tags))?;
    assert_eq!(
        observation.resource.properties.get("describe_collected"),
        Some(&json!(true))
    );
    assert_eq!(observation.resource.id, STATE_MACHINE_ARN);
    assert_eq!(
        observation.resource.kind,
        "aws_step_functions_state_machine"
    );
    assert_eq!(
        observation.resource.name.as_deref(),
        Some(STATE_MACHINE_NAME)
    );
    assert_eq!(observation.resource.account_id, "111111111111");
    assert_eq!(observation.resource.region, "us-east-1");
    assert_eq!(
        observation.resource.properties.get("state_machine_type"),
        Some(&json!("STANDARD"))
    );
    assert_eq!(
        observation.resource.properties.get("type"),
        Some(&json!("STANDARD"))
    );
    assert_eq!(
        observation.resource.properties.get("status"),
        Some(&json!("ACTIVE"))
    );
    assert_eq!(
        observation.resource.properties.get("role_arn"),
        Some(&json!(ROLE_ARN))
    );
    assert_eq!(
        observation.resource.properties.get("creation_date"),
        Some(&json!(3_000))
    );
    assert_eq!(
        observation.resource.properties.get("tags_collected"),
        Some(&json!(true))
    );
    assert_eq!(
        observation.resource.tags.get("team"),
        Some(&"orders".to_owned())
    );
    assert_eq!(observation.edges.len(), 2);
    let role = observation.edges.first().ok_or("missing role edge")?;
    let key = observation.edges.get(1).ok_or("missing encryption edge")?;
    assert_eq!(role.relation, "ASSUMES_ROLE");
    assert_eq!(role.to_id, ROLE_ARN);
    assert_eq!(key.relation, "ENCRYPTED_BY");
    assert_eq!(key.to_id, KEY_ARN);
    Ok(())
}

#[test]
fn definitions_and_variable_references_are_omitted() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?;
    let described = state_machine_description(STATE_MACHINE_ARN, STATE_MACHINE_NAME, None)?;
    let observation = map_state_machine(&ctx, &item, Some(&described), None)?;
    let serialized = serde_json::to_string(&observation.resource.properties)?;
    assert!(!serialized.contains("definition"));
    assert!(!serialized.contains("SECRET-DEFINITION"));
    assert!(!serialized.contains("variable_references"));
    assert!(!serialized.contains("$.secret"));
    Ok(())
}

#[test]
fn mismatched_or_foreign_descriptions_are_rejected() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?;
    let wrong_name = state_machine_description(STATE_MACHINE_ARN, "other", None)?;
    assert!(map_state_machine(&ctx, &item, Some(&wrong_name), None).is_err());
    let other_arn = "arn:aws:states:us-east-1:111111111111:stateMachine:other";
    let other_item = state_machine_item(other_arn, "other")?;
    assert!(
        map_state_machine(
            &ctx,
            &item,
            Some(&state_machine_description(other_arn, "other", None)?),
            None
        )
        .is_err()
    );
    assert!(
        map_state_machine(
            &ctx,
            &other_item,
            Some(&state_machine_description(
                STATE_MACHINE_ARN,
                STATE_MACHINE_NAME,
                None
            )?),
            None
        )
        .is_err()
    );
    let foreign = "arn:aws:states:us-west-2:111111111111:stateMachine:orders";
    let foreign_item = state_machine_item(foreign, STATE_MACHINE_NAME)?;
    let foreign_description = state_machine_description(foreign, STATE_MACHINE_NAME, None)?;
    assert!(map_state_machine(&ctx, &foreign_item, Some(&foreign_description), None).is_err());
    Ok(())
}

#[test]
fn tag_failure_retains_the_state_machine() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?;
    let described = state_machine_description(STATE_MACHINE_ARN, STATE_MACHINE_NAME, None)?;
    let observation = map_state_machine(&ctx, &item, Some(&described), None)?;
    assert_eq!(observation.resource.id, STATE_MACHINE_ARN);
    assert_eq!(
        observation.resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    assert!(observation.resource.tags.is_empty());
    Ok(())
}

#[test]
fn non_exact_encryption_references_do_not_emit_edges() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?;
    for key in [
        Some("alias/aws/states"),
        Some("arn:aws:kms:us-west-2:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab"),
        None,
    ] {
        let described = state_machine_description(STATE_MACHINE_ARN, STATE_MACHINE_NAME, key)?;
        let observation = map_state_machine(&ctx, &item, Some(&described), None)?;
        assert_eq!(observation.edges.len(), 1);
        let edge = observation.edges.first().ok_or("missing role edge")?;
        assert_eq!(edge.relation, "ASSUMES_ROLE");
        let encryption = observation
            .resource
            .properties
            .get("encryption_configuration")
            .cloned()
            .unwrap_or_default();
        if let Some(key) = key {
            assert_eq!(
                encryption
                    .get("kms_key_id")
                    .and_then(|value| value.as_str()),
                Some(key)
            );
        }
    }
    Ok(())
}

#[test]
fn describe_request_asks_for_metadata_only() {
    let config = aws_sdk_sfn::Config::builder()
        .behavior_version(aws_sdk_sfn::config::BehaviorVersion::latest())
        .build();
    let client = Client::from_conf(config);
    let request = describe_request(&client, STATE_MACHINE_ARN);
    let input = request.as_input();
    assert_eq!(input.get_included_data(), &Some(IncludedData::MetadataOnly));
    assert_eq!(
        input.get_state_machine_arn().as_deref(),
        Some(STATE_MACHINE_ARN)
    );
}

#[test]
fn describe_failure_keeps_the_listed_state_machine_with_explicit_coverage() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?;
    let tags = BTreeMap::from([("team".to_owned(), "orders".to_owned())]);
    let observation = map_state_machine(&ctx, &item, None, Some(tags))?;
    let resource = &observation.resource;
    assert_eq!(resource.id, STATE_MACHINE_ARN);
    assert_eq!(resource.name.as_deref(), Some(STATE_MACHINE_NAME));
    assert_eq!(
        resource.properties.get("describe_collected"),
        Some(&json!(false))
    );
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(true))
    );
    assert_eq!(resource.properties.get("type"), Some(&json!("STANDARD")));
    assert_eq!(
        resource.properties.get("creation_date"),
        Some(&json!(2_000))
    );
    assert_eq!(resource.properties.get("role_arn"), Some(&Value::Null));
    assert_eq!(resource.tags.get("team"), Some(&"orders".to_owned()));
    assert!(observation.edges.is_empty());
    let foreign = state_machine_item(
        "arn:aws:states:us-west-2:111111111111:stateMachine:orders",
        STATE_MACHINE_NAME,
    )?;
    assert!(map_state_machine(&ctx, &foreign, None, None).is_err());
    Ok(())
}

#[tokio::test]
async fn describe_and_tag_failures_are_recorded_under_the_exact_arn() -> TestResult {
    let (ctx, mut receiver) = context("us-east-1")?;
    let error = AwsScanError::InvalidData("read failed".into());
    failure(
        &ctx,
        "aws_step_functions_state_machine_describe",
        Some(STATE_MACHINE_ARN),
        &error,
    )
    .await?;
    failure(
        &ctx,
        "aws_step_functions_state_machine_tags",
        Some(STATE_MACHINE_ARN),
        &error,
    )
    .await?;
    for collector in [
        "aws_step_functions_state_machine_describe",
        "aws_step_functions_state_machine_tags",
    ] {
        let event = receiver.recv().await.ok_or("missing failure event")?;
        let CollectionEvent::Failure(recorded) = event else {
            return Err("expected a failure event".into());
        };
        assert_eq!(recorded.scope.collector, collector);
        assert_eq!(recorded.scope.parent_id.as_deref(), Some(STATE_MACHINE_ARN));
        assert_eq!(recorded.category, error_category(&error));
    }
    Ok(())
}
