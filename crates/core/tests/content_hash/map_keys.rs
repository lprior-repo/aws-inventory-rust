use super::support::{TestResult, excluding, fields, hash};
use inventory_core::record::*;
use std::collections::BTreeMap;

fn names(entries: &[(&str, i64)]) -> Value {
    Value::Map(
        entries
            .iter()
            .map(|(key, value)| (Key::Name((*key).to_owned()), Value::Int(*value)))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn mask(path: &[u32], key: &str) -> Result<Exclusions, RecordError> {
    let path = path
        .iter()
        .copied()
        .map(FieldId::new)
        .collect::<Result<Vec<_>, _>>()?;
    Exclusions::none().with_map_keys(vec![MapKeyExclusion::new(path, key)?])
}

#[test]
fn literal_map_mask_omits_only_the_exact_path_and_key_without_changing_record() -> TestResult {
    let original = fields([
        (
            1,
            names(&[("SubscriptionsConfirmed", 1), ("DisplayName", 9)]),
        ),
        (2, names(&[("SubscriptionsConfirmed", 7)])),
    ])?;
    let changed = fields([
        (
            1,
            names(&[("SubscriptionsConfirmed", 42), ("DisplayName", 9)]),
        ),
        (2, names(&[("SubscriptionsConfirmed", 7)])),
    ])?;
    let absent = fields([
        (1, names(&[("DisplayName", 9)])),
        (2, names(&[("SubscriptionsConfirmed", 7)])),
    ])?;
    let exclusions = mask(&[1], "SubscriptionsConfirmed")?;
    let record = Record::encode(&original)?;
    let bytes = record.as_bytes().to_vec();
    assert_eq!(record.content_hash(&exclusions)?, hash(&absent)?);
    assert_eq!(
        excluding(&original, &exclusions)?,
        excluding(&changed, &exclusions)?
    );
    assert_ne!(hash(&original)?, hash(&changed)?);
    assert_eq!(record.as_bytes(), bytes);
    let map = record
        .root()
        .get_field(FieldId::new(1)?)?
        .ok_or("missing map")?;
    let ValueRef::Map(map) = map else {
        return Err("wrong map kind".into());
    };
    assert_eq!(
        map.get_name("SubscriptionsConfirmed")?
            .ok_or("missing counter")?
            .to_value()?,
        Value::Int(1)
    );
    let kept_change = fields([
        (
            1,
            names(&[("SubscriptionsConfirmed", 1), ("DisplayName", 10)]),
        ),
        (2, names(&[("SubscriptionsConfirmed", 7)])),
    ])?;
    let other_path_change = fields([
        (
            1,
            names(&[("SubscriptionsConfirmed", 1), ("DisplayName", 9)]),
        ),
        (2, names(&[("SubscriptionsConfirmed", 8)])),
    ])?;
    assert_ne!(
        excluding(&original, &exclusions)?,
        excluding(&kept_change, &exclusions)?
    );
    assert_ne!(
        excluding(&original, &exclusions)?,
        excluding(&other_path_change, &exclusions)?
    );
    Ok(())
}

#[test]
fn literal_numeric_key_is_distinct_from_numeric_field_and_combines_with_field_masks() -> TestResult
{
    let map = Value::Map(BTreeMap::from([
        (Key::Name("65535".to_owned()), Value::Int(1)),
        (Key::Field(FieldId::new(65535)?), Value::Int(2)),
        (Key::Name("65536".to_owned()), Value::Int(3)),
    ]));
    let value = fields([(1, map), (2, Value::Int(4))])?;
    let expected = fields([(
        1,
        Value::Map(BTreeMap::from([
            (Key::Field(FieldId::new(65535)?), Value::Int(2)),
            (Key::Name("65536".to_owned()), Value::Int(3)),
        ])),
    )])?;
    let exclusions = Exclusions::new([vec![FieldId::new(2)?]])?
        .with_map_keys(vec![MapKeyExclusion::new(vec![FieldId::new(1)?], "65535")?])?;
    assert_eq!(excluding(&value, &exclusions)?, hash(&expected)?);
    Ok(())
}

#[test]
fn map_masks_follow_numeric_list_paths_but_not_unmatched_named_containers() -> TestResult {
    let entry = || fields([(2, names(&[("volatile", 1), ("kept", 2)]))]);
    let original = fields([
        (1, Value::List(vec![entry()?, entry()?])),
        (
            3,
            Value::Map(BTreeMap::from([(Key::Name("outer".to_owned()), entry()?)])),
        ),
    ])?;
    let reduced = || fields([(2, names(&[("kept", 2)]))]);
    let expected = fields([
        (1, Value::List(vec![reduced()?, reduced()?])),
        (
            3,
            Value::Map(BTreeMap::from([(Key::Name("outer".to_owned()), entry()?)])),
        ),
    ])?;
    let exclusions = mask(&[1, 2], "volatile")?;
    assert_eq!(excluding(&original, &exclusions)?, hash(&expected)?);
    assert_eq!(
        excluding(&original, &mask(&[3, 2], "volatile")?)?,
        hash(&original)?
    );
    Ok(())
}

#[test]
fn map_exclusion_constructor_enforces_exact_byte_count_and_duplicate_bounds() -> TestResult {
    let path = vec![FieldId::new(1)?];
    assert_eq!(
        MapKeyExclusion::new(Vec::new(), "x"),
        Err(RecordError::ExclusionPath { length: 0 })
    );
    assert_eq!(
        MapKeyExclusion::new(vec![FieldId::new(1)?; MAX_DEPTH + 1], "x"),
        Err(RecordError::ExclusionPath {
            length: MAX_DEPTH + 1
        })
    );
    assert_eq!(
        MapKeyExclusion::new(path.clone(), ""),
        Err(RecordError::ExclusionKey { length: 0 })
    );
    assert_eq!(
        MapKeyExclusion::new(path.clone(), "é".repeat(2049)),
        Err(RecordError::ExclusionKey { length: 4098 })
    );
    assert_eq!(
        MapKeyExclusion::new(path.clone(), "é".repeat(2048))?
            .key()
            .len(),
        4096
    );
    assert_eq!(
        MapKeyExclusion::new(vec![FieldId::new(1)?; MAX_DEPTH], "x")?
            .path()
            .len(),
        MAX_DEPTH
    );
    let masks = (0..65)
        .map(|index| MapKeyExclusion::new(path.clone(), index.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        Exclusions::none().with_map_keys(masks.clone()),
        Err(RecordError::ExclusionCount { length: 65 })
    );
    assert_eq!(
        Exclusions::none()
            .with_map_keys(masks.into_iter().take(64).collect())?
            .map_keys()
            .len(),
        64
    );
    let duplicate = MapKeyExclusion::new(path, "x")?;
    assert_eq!(
        Exclusions::none().with_map_keys(vec![duplicate.clone(), duplicate]),
        Err(RecordError::DuplicateExclusion)
    );
    Ok(())
}

proptest::proptest! {
    #[test]
    fn arbitrary_excluded_counter_values_do_not_change_hash(left in proptest::prelude::any::<i64>(), right in proptest::prelude::any::<i64>()) {
        let digest = |counter| -> Result<ContentHash, RecordError> {
            excluding(&fields([(1, names(&[("counter", counter), ("kept", 8)]))])?, &mask(&[1], "counter")?)
        };
        let left = digest(left).map_err(|error| proptest::test_runner::TestCaseError::fail(error.to_string()))?;
        let right = digest(right).map_err(|error| proptest::test_runner::TestCaseError::fail(error.to_string()))?;
        proptest::prop_assert_eq!(left, right);
    }
}
