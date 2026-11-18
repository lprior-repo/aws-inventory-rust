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
fn given_duplicate_or_malformed_snapshot_when_normalized_then_no_complete_snapshot_is_returned()
-> TestResult {
    let duplicates = record(Some(Value::List(vec![
        pair("same", "a")?,
        pair("same", "b")?,
    ])))?;
    assert!(matches!(
        tag_observation(&duplicates, &resource()?, &list_plan()?),
        Err(SweepError::Inventory(InventoryError::DuplicateTag))
    ));
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
    assert!(matches!(
        tag_observation(
            &record(Some(Value::List(oversized)))?,
            &resource()?,
            &list_plan()?
        ),
        Err(SweepError::Inventory(InventoryError::TagCount))
    ));
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
