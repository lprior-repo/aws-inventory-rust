//! Deterministic identity contracts; the owned-emulator pagination scenario
//! exercises the real create/status/list transport separately.

use aws_sdk_cloudcontrol::types::{HandlerErrorCode, OperationStatus, ProgressEvent};

use super::{HarnessError, SeededTopic, completed_identifier, topic_report};
use serde_json::json;

const PREFIX: &str = "inventory-page-topic";
const NAME: &str = "inventory-page-topic-0";
const IDENTIFIER: &str = "arn:aws:sns:us-east-1:000000000000:inventory-page-topic-0";
type TestResult = Result<(), HarnessError>;

#[test]
fn given_provider_id_differs_when_listed_then_requested_name_is_retained() -> TestResult {
    // Given a provider identifier that differs from the requested name.
    let topics = [SeededTopic {
        name: NAME.to_owned(),
        identifier: IDENTIFIER.to_owned(),
    }];
    // When the exact authoritative identifier is listed.
    let report = topic_report(PREFIX, &topics, vec![IDENTIFIER.to_owned()])?;
    // Then names and provider identifiers keep distinct report meanings.
    assert_eq!(
        report,
        json!({"prefix": PREFIX, "topics": [NAME], "listed": [IDENTIFIER]})
    );
    Ok(())
}

#[test]
fn given_success_when_completed_then_exact_provider_identifier_is_returned() -> TestResult {
    // Given the successful create/status event's authoritative identity.
    let event = ProgressEvent::builder()
        .operation_status(OperationStatus::Success)
        .identifier(IDENTIFIER)
        .build();
    // When completion is interpreted.
    let identifier = completed_identifier(&event)?;
    // Then no requested name or resource-model heuristic replaces that identity.
    assert_eq!(identifier, Some(IDENTIFIER));
    Ok(())
}

#[test]
fn given_only_requested_name_listed_when_reported_then_provider_id_is_missing() {
    // Given a provider identifier unrelated to the requested name's spelling.
    let topics = [SeededTopic {
        name: NAME.to_owned(),
        identifier: "opaque-provider-id".to_owned(),
    }];
    // When only the requested bare name appears in ListResources.
    let error = topic_report(PREFIX, &topics, vec![NAME.to_owned()])
        .err()
        .map(|error| error.to_string());
    // Then the report fails rather than falling back to that name.
    assert_eq!(
        error.as_deref(),
        Some("seeded topic inventory-page-topic-0 (opaque-provider-id) is not listed by the emulator")
    );
}

#[test]
fn given_same_suffix_when_foreign_identifier_listed_then_report_rejects_it() {
    // Given one exact provider identity.
    let topics = [SeededTopic {
        name: NAME.to_owned(),
        identifier: IDENTIFIER.to_owned(),
    }];
    // When a different identity happens to share its topic-name suffix.
    let foreign = "arn:aws:sns:us-east-1:111111111111:inventory-page-topic-0";
    let error = topic_report(PREFIX, &topics, vec![foreign.to_owned()])
        .err()
        .map(|error| error.to_string());
    // Then suffix equality does not satisfy the listing gate.
    assert_eq!(
        error.as_deref(),
        Some(concat!(
            "seeded topic inventory-page-topic-0 ",
            "(arn:aws:sns:us-east-1:000000000000:inventory-page-topic-0) ",
            "is not listed by the emulator"
        ))
    );
}

#[test]
fn given_success_without_identifier_when_completed_then_seed_fails() {
    // Given a terminal success event without its required resource identity.
    let event = ProgressEvent::builder()
        .operation_status(OperationStatus::Success)
        .build();
    // When completion is interpreted.
    let error = completed_identifier(&event)
        .err()
        .map(|error| error.to_string());
    // Then absence remains an operator-visible seed error.
    assert_eq!(
        error.as_deref(),
        Some("Cloud Control create succeeded without a resource identifier")
    );
}

#[test]
fn given_success_with_empty_identifier_when_completed_then_seed_fails() {
    // Given a terminal success event with an unusable empty resource identity.
    let event = ProgressEvent::builder()
        .operation_status(OperationStatus::Success)
        .identifier("")
        .build();
    // When completion is interpreted.
    let error = completed_identifier(&event)
        .err()
        .map(|error| error.to_string());
    // Then an empty identifier cannot satisfy the listing gate.
    assert_eq!(
        error.as_deref(),
        Some("Cloud Control create succeeded without a resource identifier")
    );
}

#[test]
fn given_failed_create_with_identifier_when_completed_then_failure_is_preserved() {
    // Given a failed operation that already has an identifier.
    let event = ProgressEvent::builder()
        .operation_status(OperationStatus::Failed)
        .identifier("opaque-provider-id")
        .error_code(HandlerErrorCode::AccessDenied)
        .build();
    // When completion is interpreted.
    let error = completed_identifier(&event)
        .err()
        .map(|error| error.to_string());
    // Then an identifier cannot turn a provider failure into a successful seed.
    assert_eq!(
        error.as_deref(),
        Some("Cloud Control create failed: Some(AccessDenied)")
    );
}

#[test]
fn given_nonterminal_identifier_when_completed_then_seed_still_waits() -> TestResult {
    // Given a provider identifier available before successful completion.
    [OperationStatus::Pending, OperationStatus::InProgress]
        .into_iter()
        .try_for_each(|status| {
            let event = ProgressEvent::builder()
                .operation_status(status)
                .identifier("opaque-provider-id")
                .build();
            // When interpreted, then that identifier is not yet accepted.
            assert_eq!(completed_identifier(&event)?, None);
            Ok(())
        })
}

#[test]
fn given_name_valued_id_when_listed_then_exact_identity_is_accepted() -> TestResult {
    // Given a provider whose authoritative identifier itself is a bare name.
    let topics = [SeededTopic {
        name: NAME.to_owned(),
        identifier: NAME.to_owned(),
    }];
    // When the exact identifier is listed.
    let report = topic_report(PREFIX, &topics, vec![NAME.to_owned()])?;
    // Then older name-valued providers still work without an ARN special case.
    assert_eq!(
        report,
        json!({"prefix": PREFIX, "topics": [NAME], "listed": [NAME]})
    );
    Ok(())
}
