use super::*;
use inventory_core::{
    catalog::runtime::{IdentityChoicePlan, IdentityLeafPlan, SelectionKind},
    record::{Key, Value},
};
use std::collections::BTreeMap;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn page(value: Value) -> Result<Record, inventory_core::record::RecordError> {
    Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(1)?),
        value,
    )])))
}

#[test]
fn explicit_map_entries_preserve_keys_and_full_modeled_values() -> TestResult {
    // Given a map with two distinct keys and structured values.
    let member = FieldId::new(2)?;
    let value = Value::Map(BTreeMap::from([(
        Key::Field(member),
        Value::Text("member-value".to_owned()),
    )]));
    let source = page(Value::Map(BTreeMap::from([
        (Key::Name("a".to_owned()), value.clone()),
        (Key::Name("b".to_owned()), value),
    ])))?;
    // When the reviewed selector emits map entries.
    let rows = item_records(
        &source,
        &ItemPlan::Select {
            path: vec![FieldId::new(1)?].into_boxed_slice(),
            kind: SelectionKind::MapEntries,
        },
    )?;
    // Then no key or modeled value is silently discarded, and identity paths address values.
    assert_eq!(rows.len(), 2);
    let first = rows.first().ok_or("missing first entry")?;
    assert!(matches!(
        first.root().get_name("$key")?,
        Some(ValueRef::Text("a"))
    ));
    assert!(matches!(
        path_value(first.root(), &[member])?,
        Some(ValueRef::Text("member-value"))
    ));
    Ok(())
}

#[test]
fn explicit_map_keys_and_values_support_scalar_items_without_heuristics() -> TestResult {
    // Given modeled string-valued map entries.
    let source = page(Value::Map(BTreeMap::from([
        (
            Key::Name("owner-a".to_owned()),
            Value::Text("value-a".to_owned()),
        ),
        (
            Key::Name("owner-b".to_owned()),
            Value::Text("value-b".to_owned()),
        ),
    ])))?;
    let select = |kind| {
        item_records(
            &source,
            &ItemPlan::Select {
                path: vec![FieldId::new(1)?].into_boxed_slice(),
                kind,
            },
        )
    };
    // When independently selecting map keys and map values.
    let keys = select(SelectionKind::MapKeys)?;
    let values = select(SelectionKind::MapValues)?;
    // Then exact scalar values survive and mismatched selector shapes fail.
    assert_eq!(keys.len(), 2);
    assert_eq!(values.len(), 2);
    assert!(matches!(
        path_value(keys.first().ok_or("missing key")?.root(), &[])?,
        Some(ValueRef::Text("owner-a"))
    ));
    assert!(matches!(
        path_value(values.first().ok_or("missing value")?.root(), &[])?,
        Some(ValueRef::Text("value-a"))
    ));
    assert!(select(SelectionKind::List).is_err());
    assert!(select(SelectionKind::Scalar).is_err());
    Ok(())
}

#[test]
fn arn_preference_falls_back_only_for_absence_not_malformed_present_values() -> TestResult {
    let arn = FieldId::new(1)?;
    let name = FieldId::new(2)?;
    // Given a reviewed absence fallback alongside an actual ARN field.
    let identity = IdentityPlan::Choice(IdentityChoicePlan::PreferArn {
        path: vec![arn].into_boxed_slice(),
        fallback: IdentityLeafPlan::Key(vec![vec![name].into_boxed_slice()].into_boxed_slice()),
        review: "actual ARN wins; reviewed absence fallback".into(),
    });
    let absent = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(name),
        Value::Text("fallback-name".to_owned()),
    )])))?;
    // When the ARN is absent, malformed, null, or valid, then only absence uses the fallback.
    assert_eq!(
        resource_id(&absent, &identity, None)?.as_str(),
        "fallback-name"
    );
    let malformed = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(arn), Value::Text("not-an-arn".to_owned())),
        (Key::Field(name), Value::Text("fallback-name".to_owned())),
    ])))?;
    assert!(resource_id(&malformed, &identity, None).is_err());
    let explicit_null = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(arn), Value::Null),
        (Key::Field(name), Value::Text("fallback-name".to_owned())),
    ])))?;
    assert!(resource_id(&explicit_null, &identity, None).is_err());
    let actual = Record::encode(&Value::Map(BTreeMap::from([
        (
            Key::Field(arn),
            Value::Text("arn:aws:lambda:us-east-1:123456789012:function:actual".to_owned()),
        ),
        (Key::Field(name), Value::Text("fallback-name".to_owned())),
    ])))?;
    assert_eq!(
        resource_id(&actual, &identity, None)?.as_str(),
        "arn:aws:lambda:us-east-1:123456789012:function:actual"
    );
    Ok(())
}

#[test]
fn given_lazy_list_with_invalid_later_item_when_consumed_then_valid_prefix_is_not_eagerly_materialized_and_whole_page_still_fails()
-> TestResult {
    // Given a valid encoded list whose second element is an invalid enumeration row.
    let source = page(Value::List(vec![
        Value::Text("first".to_owned()),
        Value::List(Vec::new()),
    ]))?;
    let plan = ItemPlan::Select {
        path: vec![FieldId::new(1)?].into_boxed_slice(),
        kind: SelectionKind::List,
    };
    // When obtaining the cardinality and consuming one row, no later row is materialized.
    let (count, mut selected) = item_selection(&source, &plan)?;
    assert_eq!(count, 2);
    let first = selected.next().ok_or("first row absent")??;
    assert!(matches!(
        scalar_item(first.root())?,
        Some(ValueRef::Text("first"))
    ));
    // Then the malformed later item and the complete synchronous page both fail.
    assert!(matches!(
        selected.next(),
        Some(Err(SweepError::Operation(
            "enumeration item is neither a structure nor a scalar"
        )))
    ));
    assert!(item_records(&source, &plan).is_err());
    Ok(())
}

#[test]
fn given_empty_or_non_text_key_maps_when_selected_then_empty_maps_preserve_zero_rows_and_malformed_keys_fail()
-> TestResult {
    let empty = page(Value::Map(BTreeMap::new()))?;
    let malformed = page(Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(2)?),
        Value::Text("not-a-modeled-map-key".to_owned()),
    )])))?;
    for kind in [
        SelectionKind::MapKeys,
        SelectionKind::MapValues,
        SelectionKind::MapEntries,
    ] {
        let plan = ItemPlan::Select {
            path: vec![FieldId::new(1)?].into_boxed_slice(),
            kind,
        };
        let (count, mut selected) = item_selection(&empty, &plan)?;
        assert_eq!(count, 0);
        assert!(selected.next().is_none());
        assert!(item_records(&empty, &plan)?.is_empty());
        assert!(matches!(
            item_records(&malformed, &plan),
            Err(SweepError::Operation("modeled map has a non-text key"))
        ));
    }
    Ok(())
}

#[test]
fn given_map_at_exact_page_limit_or_one_over_when_selected_then_exact_rows_survive_and_oversized_page_is_rejected()
-> TestResult {
    let plan = ItemPlan::Select {
        path: vec![FieldId::new(1)?].into_boxed_slice(),
        kind: SelectionKind::MapValues,
    };
    let mut entries = (0..MAX_PAGE_ITEMS)
        .map(|ordinal| {
            (
                Key::Name(format!("key-{ordinal:06}")),
                Value::Text("value".to_owned()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let exact = page(Value::Map(entries.clone()))?;
    let (count, _) = item_selection(&exact, &plan)?;
    assert_eq!(count, MAX_PAGE_ITEMS);
    let rows = item_records(&exact, &plan)?;
    assert_eq!(rows.len(), MAX_PAGE_ITEMS);
    assert!(matches!(
        scalar_item(rows.first().ok_or("first map value absent")?.root())?,
        Some(ValueRef::Text("value"))
    ));
    assert!(matches!(
        scalar_item(rows.last().ok_or("last map value absent")?.root())?,
        Some(ValueRef::Text("value"))
    ));
    entries.insert(
        Key::Name("one-over".to_owned()),
        Value::Text("value".to_owned()),
    );
    let oversized = page(Value::Map(entries))?;
    assert!(matches!(
        item_records(&oversized, &plan),
        Err(SweepError::Operation(
            "page exceeds the atomic observation bound"
        ))
    ));
    Ok(())
}
