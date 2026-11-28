use super::*;
use inventory_core::{
    catalog::runtime::{IdentityChoicePlan, IdentityLeafPlan, SelectionKind},
    record::{Key, Value},
};
use std::collections::BTreeMap;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn compiled_ec2_query_xml_projects_instance_ids_and_reservation_owners() -> TestResult {
    use aws_wire::{request::Protocol, response::{DecodeLimits, ResponseContext}};
    use inventory_core::catalog::runtime::{RuntimeCatalog, RuntimeProtocol};
    let catalog = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog/aws-api.catalog"))?;
    let catalog = RuntimeCatalog::from_bytes(&catalog)?;
    let plan = catalog.lookup("ec2", "DescribeInstances").ok_or("DescribeInstances absent")?;
    assert_eq!(plan.kind().as_str(), "AWS::EC2::Instance");
    assert_eq!(plan.protocol(), RuntimeProtocol::Ec2Query);
    let xml = br#"<DescribeInstancesResponse xmlns="http://ec2.amazonaws.com/doc/2016-11-15/"><reservationSet>
      <item><ownerId>123456789012</ownerId><instancesSet><item><instanceId>i-one</instanceId></item><item><instanceId>i-two</instanceId></item></instancesSet></item>
      <item><ownerId>999999999999</ownerId><instancesSet><item><instanceId>i-other</instanceId></item></instancesSet></item>
      </reservationSet><nextToken>page-two</nextToken></DescribeInstancesResponse>"#;
    let record = aws_wire::response::decode(ResponseContext {
        service: plan.service(), operation: plan.operation(), protocol: Protocol::Ec2Query,
        body: xml, headers: &http::HeaderMap::new(), status: http::StatusCode::OK,
        limits: DecodeLimits::default(),
    })?;
    use crate::execution::page::cursor::Cursor;
    let next = Cursor::default().next(plan.pagination(), &record, &http::HeaderMap::new(), 10)?
        .ok_or("XML nextToken did not produce a checkpoint")?;
    let resumed = Cursor::decode(Some(next.cursor.as_bytes()))?;
    let fields = resumed.request_fields(plan.pagination())?;
    let base = crate::input::root_bindings(plan, None, None)?;
    let request = crate::request::encode(plan, "https://ec2.example.test", &base, &fields)?;
    assert_eq!(request.method(), http::Method::POST);
    let form = std::str::from_utf8(request.body())?;
    assert!(form.split('&').any(|field| field == "Action=DescribeInstances"));
    assert_eq!(form.split('&').filter(|field| field.starts_with("NextToken=")).collect::<Vec<_>>(), ["NextToken=page-two"]);
    let account = AccountId::parse("123456789012")?;
    let expected = [("i-one", false), ("i-two", false), ("i-other", true)];
    let rows = item_records(&record, plan.items())?;
    assert_eq!(rows.len(), expected.len());
    for (row, (id, foreign)) in rows.iter().zip(expected) {
        assert_eq!(resource_id(row, plan.identity(), None)?.as_str(), id);
        assert_eq!(nested_foreign_owned(row, plan.items(), &account)?, foreign);
    }
    Ok(())
}

fn page(value: Value) -> Result<Record, inventory_core::record::RecordError> {
    Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(1)?),
        value,
    )])))
}

#[test]
fn explicit_map_entries_preserve_keys_and_full_modeled_values() -> TestResult {
    let member = FieldId::new(2)?;
    let value = Value::Map(BTreeMap::from([(
        Key::Field(member),
        Value::Text("member-value".to_owned()),
    )]));
    let source = page(Value::Map(BTreeMap::from([
        (Key::Name("a".to_owned()), value.clone()),
        (Key::Name("b".to_owned()), value),
    ])))?;
    let rows = item_records(
        &source,
        &ItemPlan::Select {
            path: vec![FieldId::new(1)?].into_boxed_slice(),
            kind: SelectionKind::MapEntries,
        },
    )?;
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
    let keys = select(SelectionKind::MapKeys)?;
    let values = select(SelectionKind::MapValues)?;
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
    let identity = IdentityPlan::Choice(IdentityChoicePlan::PreferArn {
        path: vec![arn].into_boxed_slice(),
        fallback: IdentityLeafPlan::Key(vec![vec![name].into_boxed_slice()].into_boxed_slice()),
        review: "actual ARN wins; reviewed absence fallback".into(),
    });
    let absent = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(name),
        Value::Text("fallback-name".to_owned()),
    )])))?;
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
    let source = page(Value::List(vec![
        Value::Text("first".to_owned()),
        Value::List(Vec::new()),
    ]))?;
    let plan = ItemPlan::Select {
        path: vec![FieldId::new(1)?].into_boxed_slice(),
        kind: SelectionKind::List,
    };
    let (count, mut selected) = item_selection(&source, &plan)?;
    assert_eq!(count, 2);
    let first = selected.next().ok_or("first row absent")??;
    assert!(matches!(
        scalar_item(first.root())?,
        Some(ValueRef::Text("first"))
    ));
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
