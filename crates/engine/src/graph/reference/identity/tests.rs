use super::*;
use inventory_core::Source;

fn make_resource(id: &str, account: &str, region: &str) -> Resource {
    Resource {
        id: id.to_owned(),
        account_id: account.to_owned(),
        region: region.to_owned(),
        kind: "test".to_owned(),
        name: None,
        properties: serde_json::Value::Object(serde_json::Map::new()),
        tags: std::collections::BTreeMap::new(),
        source: Source::Direct,
        observed_at: 0,
    }
}

#[test]
fn bare_name_still_resolves_to_arn_identified_target() {
    let mut index = IdentityIndex::default();
    let resource = make_resource(
        "arn:aws:lambda:us-east-1:123456789012:function:my-func",
        "123456789012",
        "us-east-1",
    );
    index.insert(&resource);

    let source = make_resource("my-func", "123456789012", "us-east-1");
    let lookup = index.lookup(&source, "my-func", 1024);
    assert!(
        matches!(lookup, Lookup::Identifier(_)),
        "bare name should resolve to ARN target"
    );
}

#[test]
fn arn_lookup_requires_service_type_match() {
    let mut index = IdentityIndex::default();
    let resource = make_resource(
        "arn:aws:lambda:us-east-1:123456789012:function:my-func",
        "123456789012",
        "us-east-1",
    );
    index.insert(&resource);

    let source = make_resource(
        "arn:aws:rds:us-east-1:123456789012:db:my-func",
        "123456789012",
        "us-east-1",
    );
    let lookup = index.lookup(
        &source,
        "arn:aws:rds:us-east-1:123456789012:db:my-func",
        1024,
    );
    assert!(
        matches!(lookup, Lookup::Unresolved),
        "ARN lookup should not resolve across services"
    );
}

#[test]
fn arn_without_service_type_fails_open() {
    let mut index = IdentityIndex::default();
    let resource = make_resource(
        "arn:aws:rds:us-east-1:123456789012:db:my-func",
        "123456789012",
        "us-east-1",
    );
    index.insert(&resource);

    let source = make_resource(
        "arn:aws:lambda:us-east-1:123456789012::my-func",
        "123456789012",
        "us-east-1",
    );
    let lookup = index.lookup(
        &source,
        "arn:aws:lambda:us-east-1:123456789012::my-func",
        1024,
    );
    assert!(
        matches!(lookup, Lookup::Unresolved),
        "ARN with empty type should not resolve across services"
    );
}

#[test]
fn same_service_arn_lookup_resolves() {
    let mut index = IdentityIndex::default();
    let resource = make_resource(
        "arn:aws:lambda:us-east-1:123456789012:function:my-func",
        "123456789012",
        "us-east-1",
    );
    index.insert(&resource);

    let source = make_resource(
        "arn:aws:lambda:us-east-1:123456789012:function:other",
        "123456789012",
        "us-east-1",
    );
    let lookup = index.lookup(
        &source,
        "arn:aws:lambda:us-east-1:123456789012:function:my-func",
        1024,
    );
    assert!(
        matches!(lookup, Lookup::Arn(_)),
        "same-service ARN lookup should resolve"
    );
}

#[test]
fn absent_service_stays_unresolved() {
    let mut index = IdentityIndex::default();
    let resource = make_resource(
        "arn:aws:rds:us-east-1:123456789012:db:my-func",
        "123456789012",
        "us-east-1",
    );
    index.insert(&resource);

    let source = make_resource(
        "arn:aws:lambda:us-east-1:123456789012:function:caller",
        "123456789012",
        "us-east-1",
    );
    let lookup = index.lookup(
        &source,
        "arn:aws:lambda:us-east-1:123456789012:function:my-func",
        1024,
    );
    assert!(
        matches!(lookup, Lookup::Unresolved),
        "Lambda ARN lookup should not resolve to RDS resource"
    );
}
