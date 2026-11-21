use std::collections::BTreeMap;

use inventory_core::{
    inventory::{AccountId, InventoryError, Kind, Region, ResourceId},
    record::{Key, Value},
};

use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn field() -> Result<FieldId, inventory_core::record::RecordError> {
    FieldId::new(1)
}

fn resource() -> Result<ResourceKey, InventoryError> {
    Ok(ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::EC2::VPC")?,
        id: ResourceId::parse("vpc-tags")?,
    })
}

fn record(value: Option<Value>) -> Result<Record, inventory_core::record::RecordError> {
    let fields = match value {
        Some(value) => BTreeMap::from([(Key::Field(field()?), value)]),
        None => BTreeMap::new(),
    };
    Record::encode(&Value::Map(fields))
}

fn map_plan() -> Result<TagPlan, inventory_core::record::RecordError> {
    Ok(TagPlan::Map {
        path: vec![field()?].into_boxed_slice(),
    })
}

fn list_plan() -> Result<TagPlan, inventory_core::record::RecordError> {
    Ok(TagPlan::List {
        path: vec![field()?].into_boxed_slice(),
        key: FieldId::new(2)?,
        value: FieldId::new(3)?,
    })
}

fn pair(key: &str, value: &str) -> Result<Value, inventory_core::record::RecordError> {
    Ok(Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(2)?), Value::Text(key.to_owned())),
        (Key::Field(FieldId::new(3)?), Value::Text(value.to_owned())),
    ])))
}

#[test]
fn given_unicode_tag_at_character_limit_when_normalized_then_tag_is_accepted() -> TestResult {
    let key = "é".repeat(128);
    let value = "é".repeat(256);
    let rec = record(Some(Value::Map(BTreeMap::from([(Key::Name(key.clone().into()), Value::Text(value.clone()))]))))?;
    let plan = map_plan()?;
    let obs = tag_observation(&rec, &resource()?, &plan)?;
    assert_eq!(obs.tags().len(), 1);
    let first = obs.tags().first().ok_or("expected one tag")?;
    assert_eq!(first.key(), key);
    assert_eq!(first.value(), value);
    Ok(())
}

#[test]
fn given_unicode_tag_over_character_limit_when_normalized_then_tag_is_skipped() -> TestResult {
    let key = "é".repeat(129);
    let value = "value";
    let rec = record(Some(Value::Map(BTreeMap::from([(Key::Name(key.into()), Value::Text(value.to_owned()))]))))?;
    let plan = map_plan()?;
    let obs = tag_observation(&rec, &resource()?, &plan)?;
    assert_eq!(obs.tags().len(), 0);
    Ok(())
}

#[test]
fn given_mixed_valid_and_invalid_tags_when_normalized_then_valid_tags_are_preserved() -> TestResult {
    let too_long = "é".repeat(129);
    let valid_pair = pair("env", "prod")?;
    let invalid_pair = Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(2)?), Value::Text(too_long)),
        (Key::Field(FieldId::new(3)?), Value::Text("value".to_string())),
    ]));
    let rec = record(Some(Value::List(vec![valid_pair, invalid_pair])))?;
    let plan = list_plan()?;
    let obs = tag_observation(&rec, &resource()?, &plan)?;
    assert_eq!(obs.tags().len(), 1);
    let first = obs.tags().first().ok_or("expected one tag")?;
    assert_eq!(first.key(), "env");
    assert_eq!(first.value(), "prod");
    // One tag was skipped, so coverage should be Partial.
    assert_eq!(obs.coverage(), TagCoverage::Partial);
    Ok(())
}

#[test]
fn given_exactly_100_valid_tags_when_observed_then_complete() -> TestResult {
    let tags: Vec<Value> = (0..100)
        .map(|i| pair(&format!("key-{i}"), "value"))
        .collect::<Result<_, _>>()?;
    let rec = record(Some(Value::List(tags)))?;
    let obs = tag_observation(&rec, &resource()?, &list_plan()?)?;
    assert_eq!(obs.tags().len(), 100);
    assert_eq!(obs.coverage(), TagCoverage::Complete);
    Ok(())
}

#[test]
fn given_60_invalid_and_90_valid_tags_when_observed_then_partial() -> TestResult {
    let mut tags: Vec<Value> = Vec::new();
    // 60 invalid tags (129-char keys, exceeding 128 limit)
    for i in 0..60 {
        let key = "é".repeat(129);
        tags.push(pair(&key, "value")?);
    }
    // 90 valid tags
    for i in 0..90 {
        tags.push(pair(&format!("valid-{i}"), "value")?);
    }
    let rec = record(Some(Value::List(tags)))?;
    let obs = tag_observation(&rec, &resource()?, &list_plan()?)?;
    assert_eq!(obs.tags().len(), 90);
    assert_eq!(obs.coverage(), TagCoverage::Partial);
    Ok(())
}

#[test]
fn given_101_valid_tags_when_observed_then_truncated_partial() -> TestResult {
    let tags: Vec<Value> = (0..101)
        .map(|i| pair(&format!("key-{i}"), "value"))
        .collect::<Result<_, _>>()?;
    let rec = record(Some(Value::List(tags)))?;
    let obs = tag_observation(&rec, &resource()?, &list_plan()?)?;
    assert_eq!(obs.tags().len(), 100);
    assert_eq!(obs.coverage(), TagCoverage::Partial);
    Ok(())
}

#[test]
fn given_modeled_map_or_list_when_normalized_then_exact_pairs_and_empty_values_survive()
-> TestResult {
    let map = record(Some(Value::Map(BTreeMap::from([
        (Key::Name("z".into()), Value::Text("value".into())),
        (Key::Name("a".into()), Value::Text("".into())),
    ]))))?;
    let list = record(Some(Value::List(vec![pair("z", "value")?, pair("a", "")?])))?;
    let map = tag_observation(&map, &resource()?, &map_plan()?)?;
    let list = tag_observation(&list, &resource()?, &list_plan()?)?;
    assert_eq!(map, list);
    assert_eq!(map.coverage(), TagCoverage::Complete);
    assert_eq!(
        map.tags()
            .iter()
            .map(|tag| (tag.key(), tag.value()))
            .collect::<Vec<_>>(),
        vec![("a", ""), ("z", "value")]
    );
    Ok(())
}

#[test]
fn given_missing_and_explicit_empty_tags_when_normalized_then_coverage_is_not_conflated()
-> TestResult {
    let absent = tag_observation(&record(None)?, &resource()?, &map_plan()?)?;
    assert_eq!(absent.coverage(), TagCoverage::Unobserved);
    let empty_map = tag_observation(
        &record(Some(Value::Map(BTreeMap::new())))?,
        &resource()?,
        &map_plan()?,
    )?;
    let empty_list = tag_observation(
        &record(Some(Value::List(vec![])))?,
        &resource()?,
        &list_plan()?,
    )?;
    assert_eq!(empty_map.coverage(), TagCoverage::Complete);
    assert_eq!(empty_list.coverage(), TagCoverage::Complete);
    assert!(empty_map.tags().is_empty());
    assert!(empty_list.tags().is_empty());
    assert!(matches!(
        tag_observation(&record(Some(Value::Null))?, &resource()?, &map_plan()?),
        Err(SweepError::Operation(
            "modeled tags do not match the compiled shape"
        ))
    ));
    Ok(())
}

#[test]
fn given_duplicate_or_malformed_snapshot_when_normalized_then_partial_coverage()
-> TestResult {
    // Duplicate keys are skipped, and coverage becomes Partial.
    let duplicates = record(Some(Value::List(vec![
        pair("same", "a")?,
        pair("same", "b")?,
    ])))?;
    let dup_obs = tag_observation(&duplicates, &resource()?, &list_plan()?)?;
    assert_eq!(dup_obs.tags().len(), 1);
    assert_eq!(dup_obs.coverage(), TagCoverage::Partial);
    // Malformed values still fail the entire observation.
    let malformed = record(Some(Value::List(vec![
        pair("good", "a")?,
        Value::Text("bad".into()),
    ])))?;
    assert!(matches!(
        tag_observation(&malformed, &resource()?, &list_plan()?),
        Err(SweepError::Operation(
            "modeled tag list contains a non-map pair"
        ))
    ));
    let oversized = (0..101)
        .map(|index| pair(&index.to_string(), "v"))
        .collect::<Result<Vec<_>, _>>()?;
    // 101 tags are truncated to 100 with Partial coverage, not an error.
    let oversized_obs = tag_observation(
        &record(Some(Value::List(oversized)))?,
        &resource()?,
        &list_plan()?,
    )?;
    assert_eq!(oversized_obs.coverage(), TagCoverage::Partial);
    assert_eq!(oversized_obs.tags().len(), 100);
    Ok(())
}

#[test]
fn given_missing_or_wrong_typed_pair_fields_when_normalized_then_exact_pair_error_is_reported()
-> TestResult {
    let bad_pairs = [
        BTreeMap::from([(Key::Field(FieldId::new(2)?), Value::Text("key".into()))]),
        BTreeMap::from([(Key::Field(FieldId::new(3)?), Value::Text("value".into()))]),
        BTreeMap::from([
            (Key::Field(FieldId::new(2)?), Value::Int(1)),
            (Key::Field(FieldId::new(3)?), Value::Text("value".into())),
        ]),
        BTreeMap::from([
            (Key::Field(FieldId::new(2)?), Value::Text("key".into())),
            (Key::Field(FieldId::new(3)?), Value::Null),
        ]),
    ];
    for pair in bad_pairs {
        let malformed = record(Some(Value::List(vec![Value::Map(pair)])))?;
        assert!(matches!(
            tag_observation(&malformed, &resource()?, &list_plan()?),
            Err(SweepError::Operation(
                "modeled tag pair requires string key and value"
            ))
        ));
    }
    Ok(())
}

#[test]
fn given_nontext_map_or_nonmap_ancestor_when_normalized_then_exact_shape_error_is_reported()
-> TestResult {
    let malformed = record(Some(Value::Map(BTreeMap::from([(
        Key::Name("key".into()),
        Value::Int(1),
    )]))))?;
    assert!(matches!(
        tag_observation(&malformed, &resource()?, &map_plan()?),
        Err(SweepError::Operation(
            "modeled tag map requires string keys and values"
        ))
    ));
    let plan = TagPlan::Map {
        path: vec![field()?, FieldId::new(2)?].into_boxed_slice(),
    };
    assert!(matches!(
        tag_observation(&record(Some(Value::Null))?, &resource()?, &plan),
        Err(SweepError::Operation(
            "modeled tag path contains a non-map ancestor"
        ))
    ));
    Ok(())
}
