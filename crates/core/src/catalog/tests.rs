use super::*;
use crate::record::{Key, Record, Value};
use proptest::prelude::*;
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn map(fields: impl IntoIterator<Item = (CatalogKey, Value)>) -> Result<Value, RecordError> {
    fields
        .into_iter()
        .map(|(key, value)| Ok((Key::Field(key.field()?), value)))
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map(Value::Map)
}
fn field_registry_values() -> (Value, Value) {
    (
        Value::Bytes(b"a1a2a3a4a5a6a7a8".to_vec()),
        Value::Bytes(vec![
            0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 6,
            0, 0, 0, 6, 0, 0, 0, 8, 0, 0, 0, 8, 0, 0, 0, 10, 0, 0, 0, 10, 0, 0, 0, 12,
            0, 0, 0, 12, 0, 0, 0, 14, 0, 0, 0, 14, 0, 0, 0, 16,
        ]),
    )
}
fn service() -> Result<Value, RecordError> {
    map([
        (CatalogKey::Name, Value::Text("Example".into())),
        (CatalogKey::ApiVersion, Value::Text("1".into())),
        (CatalogKey::ServiceShape, Value::Text("n#Example".into())),
        (
            CatalogKey::Protocols,
            Value::List(vec![Value::Text("aws.protocols#restJson1".into())]),
        ),
        (CatalogKey::Operations, Value::List(vec![operation()?])),
        (CatalogKey::Shapes, Value::List(vec![shape()?])),
        (CatalogKey::Resources, Value::List(vec![])),
    ])
}
fn shape() -> Result<Value, RecordError> {
    let member = map([
        (CatalogKey::MemberId, Value::Int(7)),
        (CatalogKey::MemberTarget, Value::Int(-3)),
        (CatalogKey::MemberRequired, Value::Bool(false)),
    ])?;
    map([
        (CatalogKey::Name, Value::Text("Input".into())),
        (CatalogKey::Kind, Value::Int(14)),
        (CatalogKey::Members, Value::List(vec![member])),
        (CatalogKey::Arn, Value::Null),
    ])
}
fn operation() -> Result<Value, RecordError> {
    map([
        (CatalogKey::Name, Value::Text("ListThings".into())),
        (CatalogKey::Flags, Value::Int(31)),
        (CatalogKey::Input, Value::Int(0)),
        (CatalogKey::Output, Value::Int(-14)),
    ])
}
fn root(service: Value, version: i64) -> Result<Vec<u8>, RecordError> {
    let bytes = Record::encode(&service)?.into_bytes();
    let (field_names, field_offsets) = field_registry_values();
    Ok(Record::encode(&map([
        (CatalogKey::Format, Value::Int(version)),
        (CatalogKey::ModelsCommit, Value::Text("test-commit".into())),
        (CatalogKey::FieldNames, field_names),
        (CatalogKey::FieldNameOffsets, field_offsets),
        (CatalogKey::Services, Value::List(vec![Value::Bytes(bytes)])),
    ])?)?
    .into_bytes())
}
fn exercise(bytes: &[u8]) -> Result<String, CatalogError> {
    corruption::exercise(bytes)
}

#[test]
fn embedded_empty_placeholder_reports_generation_command() -> TestResult {
    // Given the ungenerated placeholder, when opened, then failure is typed and actionable.
    let Err(error @ CatalogError::Empty) = Catalog::open(b"") else {
        return Err("expected Empty".into());
    };
    assert!(error.to_string().contains("models compile --write"));
    Ok(())
}
#[test]
fn reader_retains_commit_and_service_identities() -> TestResult {
    // Given writer-created records, when looked up, missing names never invent a service.
    let bytes = root(service()?, CATALOG_VERSION)?;
    let catalog = Catalog::borrow(&bytes)?;
    assert_eq!(catalog.models_commit()?, "test-commit");
    assert_eq!(catalog.service("missing")?.map(|s| s.shape_count()), None);
    let service = catalog.service("Example")?.ok_or("missing service")?;
    assert_eq!(service.name()?, "Example");
    assert_eq!(service.shape_id()?, "n#Example");
    assert_eq!(service.shape_count(), 1);
    Ok(())
}
#[test]
fn counts_are_computed_from_members_and_operation_flags() -> TestResult {
    // Given a nested member and root operation, when aggregated, their exact counts appear.
    let bytes = root(service()?, CATALOG_VERSION)?;
    let counts = Catalog::borrow(&bytes)?.counts()?;
    assert_eq!(counts.get("services"), Some(1));
    assert_eq!(counts.get("members"), Some(1));
    assert_eq!(counts.get("distinct_member_names"), Some(1));
    assert_eq!(counts.get("list_describe_root"), Some(1));
    assert_eq!(counts.get("unknown"), None);
    Ok(())
}
#[test]
fn unsupported_root_version_is_reported() -> TestResult {
    // Given valid CBOR with a newer schema, when opened, then version is not guessed.
    let bytes = root(service()?, 1)?;
    assert!(matches!(
        Catalog::borrow(&bytes),
        Err(CatalogError::Version(1))
    ));
    Ok(())
}
#[test]
fn malformed_service_is_deferred_until_first_access() -> TestResult {
    // Given valid outer bytes containing a wrong service field, when opened, no service is read.
    let mut fields = match service()? {
        Value::Map(fields) => fields,
        _ => return Err("map".into()),
    };
    fields.insert(Key::Field(CatalogKey::Name.field()?), Value::Bool(true));
    let bytes = root(Value::Map(fields), CATALOG_VERSION)?;
    let catalog = Catalog::borrow(&bytes)?;
    // Then first access reports corruption rather than silently accepting a count.
    assert!(matches!(
        catalog.services()?.next().ok_or("service")?,
        Err(CatalogError::Field(CatalogKey::Name))
    ));
    Ok(())
}
#[test]
fn every_single_byte_mutation_and_truncation_is_total() -> TestResult {
    // Given a small catalog, when any byte changes or truncates, no path panics.
    let bytes = root(service()?, CATALOG_VERSION)?;
    (0..bytes.len()).try_for_each(|position| -> TestResult {
        (1..=255u8).try_for_each(|mask| check_flip(&bytes, position, mask))?;
        let truncated = bytes.get(..position).ok_or("truncation")?;
        assert!(exercise(truncated).is_err());
        Ok(())
    })
}
fn check_flip(bytes: &[u8], position: usize, mask: u8) -> TestResult {
    let mut changed = bytes.to_vec();
    *changed.get_mut(position).ok_or("position")? ^= mask;
    match exercise(&changed) {
        Ok(report) => assert!(report.starts_with("services ")),
        Err(error) => assert!(!error.to_string().is_empty()),
    }
    Ok(())
}
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        rng_seed: proptest::test_runner::RngSeed::Fixed(0x0CA7_A10C),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]
    #[test]
    fn arbitrary_corruption_returns_only_typed_results(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        // Given bounded hostile bytes, when the complete reader path runs, then it is total.
        let result: Result<String, CatalogError> = exercise(&bytes);
        match result {
            Ok(report) => prop_assert!(report.starts_with("services ")),
            Err(error) => prop_assert!(!error.to_string().is_empty()),
        }
    }
}

mod corruption;
mod depth;
mod errors;
mod targets;
mod views;
