use inventory_core::{
    CollectionPage, CollectionScope, Edge, MAX_IDENTIFIER_BYTES, MAX_PROPERTY_BYTES, Resource,
    Source,
};
use serde_json::json;
use std::collections::BTreeMap;

fn resource() -> Resource {
    Resource {
        id: "arn:aws:ec2:us-east-1:123456789012:instance/i-1".into(),
        account_id: "123456789012".into(),
        region: "us-east-1".into(),
        kind: "aws_ec2_instance".into(),
        name: None,
        properties: json!({}),
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: 1,
    }
}

#[test]
fn serialized_payload_bounds_include_json_escaping() {
    let mut record = resource();
    record.properties = json!({"value":"\"".repeat(MAX_PROPERTY_BYTES / 2)});
    assert!(record.validate().is_err());
}

#[test]
fn tag_values_can_be_empty_but_cannot_be_unbounded() {
    let mut record = resource();
    record.tags.insert("empty".into(), String::new());
    assert!(record.validate().is_ok());
    record.tags.insert(
        "oversized".into(),
        "x".repeat(MAX_IDENTIFIER_BYTES.saturating_add(1)),
    );
    assert!(record.validate().is_err());
}

#[test]
fn pages_cannot_use_zero_or_empty_continuation() {
    let mut page = CollectionPage {
        scope: CollectionScope {
            account_id: "123456789012".into(),
            region: "us-east-1".into(),
            collector: "ec2".into(),
            parent_id: None,
            source: Source::Direct,
        },
        page_number: 0,
        next_token: None,
        resources: vec![resource()],
        edges: vec![],
    };
    assert!(page.validate().is_err());
    page.page_number = 1;
    page.next_token = Some(String::new());
    assert!(page.validate().is_err());
    page.next_token = None;
    assert!(page.validate().is_ok());
}

#[test]
fn negative_relationship_time_is_rejected() {
    let edge = Edge {
        from_id: "a".into(),
        to_id: "b".into(),
        relation: "IN_VPC".into(),
        source: Source::Direct,
        observed_at: -1,
        properties: json!({}),
    };
    assert!(edge.validate().is_err());
}

#[test]
fn aggregate_page_bound_applies_to_many_valid_records() {
    let mut record = resource();
    record.properties = json!({"value":"x".repeat(900_000)});
    assert!(record.validate().is_ok());
    let page = CollectionPage {
        scope: CollectionScope {
            account_id: "123456789012".into(),
            region: "us-east-1".into(),
            collector: "ec2".into(),
            parent_id: None,
            source: Source::Direct,
        },
        page_number: 1,
        next_token: None,
        resources: vec![record; 20],
        edges: vec![],
    };
    assert!(page.validate().is_err());
}
