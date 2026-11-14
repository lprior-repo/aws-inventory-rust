//! Model boundaries checked against serialized bytes and public contracts.

use inventory_core::{
    CollectionEvent, CollectionPage, CollectionScope, Edge, MAX_IDENTIFIER_BYTES, MAX_PAGE_EDGES,
    MAX_PAGE_RESOURCES, MAX_PAGES, MAX_PROPERTY_BYTES, Resource, ScanStatus, Source,
    ValidationError, validate_identifier, validate_serialized_size,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        max_shrink_iters: 2_048,
        rng_seed: RngSeed::Fixed(0x4157_5349_4e56_0001),
        ..ProptestConfig::default()
    }
}

fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..256)
        .prop_map(|characters| characters.into_iter().collect())
}

fn identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9:/ ._-]{1,128}"
}

fn invalid_identifier() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        text().prop_map(|value| format!("{value}\0")),
        (4_097usize..4_105).prop_map(|length| "x".repeat(length)),
    ]
}

fn resource() -> Resource {
    Resource {
        id: "arn:aws:ec2:us-east-1:123456789012:instance/i-fixture".into(),
        account_id: "123456789012".into(),
        region: "us-east-1".into(),
        kind: "aws_ec2_instance".into(),
        name: None,
        properties: json!({}),
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: 0,
    }
}

fn edge() -> Edge {
    Edge {
        from_id: "instance".into(),
        to_id: "vpc".into(),
        relation: "IN_VPC".into(),
        source: Source::Direct,
        observed_at: 0,
        properties: json!({}),
    }
}

fn scope() -> CollectionScope {
    CollectionScope {
        account_id: "123456789012".into(),
        region: "us-east-1".into(),
        collector: "ec2".into(),
        parent_id: None,
        source: Source::Direct,
    }
}

fn page() -> CollectionPage {
    CollectionPage {
        scope: scope(),
        page_number: 1,
        next_token: None,
        resources: vec![resource()],
        edges: vec![edge()],
    }
}

#[test]
fn exact_identifier_text_token_and_record_boundaries() {
    assert!(validate_identifier(&"x".repeat(MAX_IDENTIFIER_BYTES), "id").is_ok());
    assert!(
        validate_identifier(&"x".repeat(MAX_IDENTIFIER_BYTES.saturating_add(1)), "id").is_err()
    );
    assert!(validate_identifier(&"é".repeat(MAX_IDENTIFIER_BYTES.saturating_div(2)), "id").is_ok());
    assert!(
        validate_identifier(
            &"é".repeat(MAX_IDENTIFIER_BYTES.saturating_div(2).saturating_add(1)),
            "id"
        )
        .is_err()
    );
    for value in [String::new(), "x".repeat(MAX_IDENTIFIER_BYTES)] {
        let mut record = resource();
        record.name = Some(value.clone());
        record.tags.insert("key".into(), value);
        assert!(record.validate().is_ok());
    }
    for value in [
        "\0".to_owned(),
        "x".repeat(MAX_IDENTIFIER_BYTES.saturating_add(1)),
    ] {
        let mut named = resource();
        named.name = Some(value.clone());
        assert!(named.validate().is_err());
        let mut tagged = resource();
        tagged.tags.insert("key".into(), value);
        assert!(tagged.validate().is_err());
    }
    let mut observation = page();
    observation.next_token = Some("x".repeat(8_192));
    assert!(observation.validate().is_ok());
    observation.next_token = Some("x".repeat(8_193));
    assert!(observation.validate().is_err());
    observation.next_token = None;
    observation.page_number = MAX_PAGES;
    observation.resources = vec![resource(); MAX_PAGE_RESOURCES];
    observation.edges.clear();
    assert!(observation.validate().is_ok());
    observation.resources.push(resource());
    assert!(matches!(
        observation.validate(),
        Err(ValidationError::Limit("page records"))
    ));
    observation.resources.clear();
    observation.edges = vec![edge(); MAX_PAGE_EDGES];
    // Record cardinality is valid even if another aggregate bound is exceeded.
    assert!(!matches!(
        observation.validate(),
        Err(ValidationError::Limit("page records"))
    ));
    observation.edges.push(edge());
    assert!(matches!(
        observation.validate(),
        Err(ValidationError::Limit("page records"))
    ));
}

#[test]
fn exact_json_size_and_zero_timestamp_boundaries() {
    let mut record = resource();
    let mut relationship = edge();
    assert!(record.validate().is_ok());
    assert!(relationship.validate().is_ok());
    for size in [
        MAX_PROPERTY_BYTES.saturating_sub(2),
        MAX_PROPERTY_BYTES.saturating_sub(1),
    ] {
        let value = Value::String("x".repeat(size));
        let accepted =
            serde_json::to_vec(&value).is_ok_and(|bytes| bytes.len() <= MAX_PROPERTY_BYTES);
        record.properties = value.clone();
        relationship.properties = value;
        assert_eq!(record.validate().is_ok(), accepted);
        assert_eq!(relationship.validate().is_ok(), accepted);
    }
}

#[test]
fn source_order_and_status_names_match_the_wire_contract() -> Result<(), Box<dyn std::error::Error>>
{
    for (source, name, priority) in [
        (Source::Direct, "direct", 4),
        (Source::CloudControl, "cloud_control", 3),
        (Source::AwsConfig, "aws_config", 2),
        (Source::ResourceExplorer, "resource_explorer", 1),
    ] {
        assert_eq!(source.as_str(), name);
        assert_eq!(source.priority(), priority);
        assert_eq!(serde_json::to_value(source)?, Value::String(name.into()));
    }
    for (status, name) in [
        (ScanStatus::Running, "running"),
        (ScanStatus::Complete, "complete"),
        (ScanStatus::Partial, "partial"),
        (ScanStatus::Cancelled, "cancelled"),
    ] {
        assert_eq!(status.as_str(), name);
        assert_eq!(serde_json::to_value(status)?, Value::String(name.into()));
    }
    Ok(())
}

struct SerializationFailure;

impl Serialize for SerializationFailure {
    fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(
            "intentional serialization failure",
        ))
    }
}

#[test]
fn serialization_failure_is_not_mislabeled_as_a_size_limit() {
    assert!(matches!(
        validate_serialized_size(&SerializationFailure, 128, "budget"),
        Err(ValidationError::Json(_))
    ));
}

#[path = "properties/arbitrary_unicode_identifiers_obey_the_byte_contract.rs"]
mod arbitrary_unicode_identifiers_obey_the_byte_contract;
