use std::collections::BTreeMap;

use serde_json::json;

use super::*;

#[test]
fn maps_activity_identity_and_safe_metadata() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = activity_item(ACTIVITY_ARN, ACTIVITY_NAME)?;
    let described = activity_description(ACTIVITY_ARN, ACTIVITY_NAME, Some(KEY_ARN))?;
    let tags = BTreeMap::from([("team".to_owned(), "shipping".to_owned())]);
    let observation = map_activity(&ctx, &item, &described, Some(tags))?;
    assert_eq!(observation.resource.id, ACTIVITY_ARN);
    assert_eq!(observation.resource.kind, "aws_step_functions_activity");
    assert_eq!(observation.resource.name.as_deref(), Some(ACTIVITY_NAME));
    assert_eq!(
        observation.resource.properties.get("activity_arn"),
        Some(&json!(ACTIVITY_ARN))
    );
    assert_eq!(
        observation.resource.properties.get("creation_date"),
        Some(&json!(4_000))
    );
    assert_eq!(
        observation.resource.properties.get("tags_collected"),
        Some(&json!(true))
    );
    assert_eq!(
        observation.resource.tags.get("team"),
        Some(&"shipping".to_owned())
    );
    assert_eq!(observation.edges.len(), 1);
    let edge = observation.edges.first().ok_or("missing encryption edge")?;
    assert_eq!(edge.relation, "ENCRYPTED_BY");
    assert_eq!(edge.to_id, KEY_ARN);
    assert!(!serde_json::to_string(&observation.resource.properties)?.contains("definition"));
    Ok(())
}

#[test]
fn mismatched_activity_identity_is_rejected() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = activity_item(ACTIVITY_ARN, ACTIVITY_NAME)?;
    assert!(
        map_activity(
            &ctx,
            &item,
            &activity_description(ACTIVITY_ARN, "other", None)?,
            None
        )
        .is_err()
    );
    let other = "arn:aws:states:us-east-1:111111111111:activity:other";
    let other_item = activity_item(other, "other")?;
    assert!(
        map_activity(
            &ctx,
            &item,
            &activity_description(other, "other", None)?,
            None
        )
        .is_err()
    );
    assert!(
        map_activity(
            &ctx,
            &other_item,
            &activity_description(ACTIVITY_ARN, ACTIVITY_NAME, None)?,
            None
        )
        .is_err()
    );
    let foreign = "arn:aws:states:us-west-2:111111111111:activity:ship";
    let foreign_item = activity_item(foreign, ACTIVITY_NAME)?;
    let foreign_description = activity_description(foreign, ACTIVITY_NAME, None)?;
    assert!(map_activity(&ctx, &foreign_item, &foreign_description, None).is_err());
    Ok(())
}

#[test]
fn tag_failure_retains_the_activity() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = activity_item(ACTIVITY_ARN, ACTIVITY_NAME)?;
    let described = activity_description(ACTIVITY_ARN, ACTIVITY_NAME, None)?;
    let observation = map_activity(&ctx, &item, &described, None)?;
    assert_eq!(observation.resource.id, ACTIVITY_ARN);
    assert_eq!(
        observation.resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    assert!(observation.resource.tags.is_empty());
    assert!(observation.edges.is_empty());
    Ok(())
}

#[test]
fn non_exact_activity_encryption_references_do_not_emit_edges() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = activity_item(ACTIVITY_ARN, ACTIVITY_NAME)?;
    for key in [
        Some("alias/aws/states"),
        Some("arn:aws:kms:us-east-1:222222222222:key/1234abcd-12ab-34cd-56ef-1234567890ab"),
        None,
    ] {
        let described = activity_description(ACTIVITY_ARN, ACTIVITY_NAME, key)?;
        let observation = map_activity(&ctx, &item, &described, None)?;
        assert!(observation.edges.is_empty());
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

#[tokio::test]
async fn activity_failures_are_recorded_under_the_exact_arn() -> TestResult {
    let (ctx, mut receiver) = context("us-east-1")?;
    let error = AwsScanError::InvalidData("read failed".into());
    failure(
        &ctx,
        "aws_step_functions_activity_describe",
        Some(ACTIVITY_ARN),
        &error,
    )
    .await?;
    failure(
        &ctx,
        "aws_step_functions_activity_tags",
        Some(ACTIVITY_ARN),
        &error,
    )
    .await?;
    for collector in [
        "aws_step_functions_activity_describe",
        "aws_step_functions_activity_tags",
    ] {
        let event = receiver.recv().await.ok_or("missing failure event")?;
        let CollectionEvent::Failure(recorded) = event else {
            return Err("expected a failure event".into());
        };
        assert_eq!(recorded.scope.collector, collector);
        assert_eq!(recorded.scope.parent_id.as_deref(), Some(ACTIVITY_ARN));
    }
    Ok(())
}
