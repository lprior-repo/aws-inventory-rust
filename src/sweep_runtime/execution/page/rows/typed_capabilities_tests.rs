mod fixture;
mod native;
use super::*;
use inventory_core::{inventory::AccountId, record::{FieldId, Key, Value}};
use std::collections::BTreeMap;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn given_required_typed_operands_when_exact_limits_or_type_mismatches_arrive_then_errors_are_precise() -> TestResult {
    use inventory_core::catalog::runtime::{IdentityFieldPlan, IdentityPlan};
    let field = FieldId::new(1)?;
    let row = |value| Record::encode(&Value::Map(BTreeMap::from([(Key::Field(field), value)])));
    let id = |row: &Record, operand| record::resource_id(row, &IdentityPlan::CorrelatedKey(vec![operand].into_boxed_slice()), None);
    let integer = || IdentityFieldPlan::SelectedInteger { path: vec![field].into_boxed_slice(), minimum: -10, maximum: 10 };
    assert_eq!(id(&row(Value::Int(10))?, integer())?.as_str(), "10");
    assert_eq!(id(&row(Value::Int(-10))?, integer())?.as_str(), "-10");
    assert_eq!(id(&row(Value::Int(0))?, integer())?.as_str(), "0");
    assert!(matches!(id(&row(Value::Int(-11))?, integer()), Err(SweepError::Operation("modeled identity integer is out of range"))));
    assert!(matches!(id(&row(Value::Int(11))?, integer()), Err(SweepError::Operation("modeled identity integer is out of range"))));
    assert!(matches!(id(&row(Value::Text("10".into()))?, integer()), Err(SweepError::Operation("modeled identity operand must be integer"))));
    let missing = Record::encode(&Value::Map(BTreeMap::new()))?;
    assert!(id(&missing, integer()).is_err());
    assert!(id(&row(Value::Null)?, integer()).is_err());
    let list = || IdentityFieldPlan::SelectedStringList { path: vec![field].into_boxed_slice(), max_items: 64, max_item_bytes: 4096 };
    let exact = "a".repeat(4084);
    assert_eq!(id(&row(Value::List(Vec::new()))?, list())?.as_str(), "list:0");
    assert_eq!(id(&row(Value::List(vec![Value::Text(exact)]))?, list())?.as_str().len(), 4096);
    assert!(matches!(id(&row(Value::List(vec![Value::Text("a".repeat(4085))]))?, list()), Err(SweepError::Operation("modeled resource identity is too long"))));
    assert!(id(&row(Value::List(vec![Value::Text("a".into()); 64]))?, list()).is_ok());
    assert!(matches!(id(&row(Value::List(vec![Value::Text("a".into()); 65]))?, list()), Err(SweepError::Operation("modeled identity string list count is out of range"))));
    Ok(())
}

#[test]
fn given_synthetic_list_delimiter_collisions_when_raw_recipes_are_framed_then_components_remain_distinct() -> TestResult {
    use inventory_core::catalog::runtime::{IdentityFieldPlan, IdentityPlan};
    let field = FieldId::new(1)?;
    let recipe = IdentityPlan::CorrelatedKey(vec![
        IdentityFieldPlan::SelectedStringList { path: vec![field].into_boxed_slice(), max_items: 64, max_item_bytes: 255 },
    ].into_boxed_slice());
    for (components, expected) in [
        (["a:b", "c"], "list:2|3:a:b|1:c"),
        (["a", "b:c"], "list:2|1:a|3:b:c"),
    ] {
        let row = Record::encode(&Value::Map(BTreeMap::from([(
            Key::Field(field), Value::List(components.into_iter().map(|value| Value::Text(value.into())).collect()),
        )])))?;
        assert_eq!(record::resource_id(&row, &recipe, None)?.as_str(), expected);
    }
    Ok(())
}

#[test]
fn given_empty_mandatory_text_when_selected_or_request_id_is_built_then_rejection_precedes_length_framing() -> TestResult {
    use inventory_core::catalog::runtime::{IdentityFieldPlan, IdentityPlan};
    let first = FieldId::new(1)?;
    let second = FieldId::new(2)?;
    let row = |parent: &str| Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(first), Value::Text(parent.into())), (Key::Field(second), Value::Text("child".into())),
    ])));
    let empty = row("")?;
    let valid = row("parent")?;
    for request in [false, true] {
        for multiple in [false, true] {
            let operand = if request { IdentityFieldPlan::Request(vec![first].into_boxed_slice()) }
                else { IdentityFieldPlan::Selected(vec![first].into_boxed_slice()) };
            let fields = if multiple { vec![operand, IdentityFieldPlan::Selected(vec![second].into_boxed_slice())] }
                else { vec![operand] };
            let identity = IdentityPlan::CorrelatedKey(fields.into_boxed_slice());
            let discovery = DiscoveryContext::empty();
            assert!(matches!(record::resource_id_with_context(&empty, &identity, None, &discovery, &empty),
                Err(SweepError::Operation("modeled identity text operand is empty"))));
            assert_eq!(record::resource_id_with_context(&valid, &identity, None, &discovery, &valid)?.as_str(),
                if multiple { "6:parent|5:child" } else { "parent" });
        }
    }
    Ok(())
}

#[test]
fn given_structural_digest_when_typed_values_reorder_or_change_then_canonical_order_and_type_distinctions_hold() -> TestResult {
    use inventory_core::record::{selected_content_hash, scalar_record, Exclusions};
    let field = FieldId::new(1)?;
    let wrapped = |value| Record::encode(&Value::Map(BTreeMap::from([(Key::Field(field), value)])));
    let selected = |row: &Record| -> TestResult<_> { Ok(selected_content_hash(row.root().get_field(field)?.ok_or("digest value absent")?)?) };
    let scalar = wrapped(Value::Text("existing".into()))?;
    assert_eq!(selected(&scalar)?, scalar_record(scalar.root().get_field(field)?.ok_or("scalar absent")?)?.content_hash(&Exclusions::default())?);
    let original = wrapped(Value::Map(BTreeMap::from([(Key::Name("name".into()), Value::Text("one".into())), (Key::Name("version".into()), Value::Int(1))])))?;
    let reordered = wrapped(Value::Map([(Key::Name("version".into()), Value::Int(1)), (Key::Name("name".into()), Value::Text("one".into()))].into()))?;
    assert_eq!(selected(&original)?, selected(&reordered)?);
    let digest = inventory_core::catalog::runtime::IdentityPlan::CorrelatedKey(vec![
        inventory_core::catalog::runtime::IdentityFieldPlan::SelectedDigest(vec![field].into_boxed_slice())
    ].into_boxed_slice());
    let resource = record::resource_id(&original, &digest, None)?;
    let expected = selected(&original)?.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    assert_eq!(resource.as_str(), format!("blake3:{expected}"));
    assert_eq!(resource, record::resource_id(&reordered, &digest, None)?);
    let changed_type = wrapped(Value::Map([(Key::Name("name".into()), Value::Text("one".into())), (Key::Name("version".into()), Value::Text("1".into()))].into()))?;
    assert_ne!(selected(&original)?, selected(&changed_type)?);
    assert_ne!(resource, record::resource_id(&changed_type, &digest, None)?);
    assert_ne!(selected(&wrapped(Value::Map(BTreeMap::new()))?)?, selected(&wrapped(Value::List(Vec::new()))?)?);
    assert_ne!(selected(&wrapped(Value::Map([(Key::Name("name".into()), Value::Null)].into()))?)?, selected(&wrapped(Value::Map(BTreeMap::new()))?)?);
    Ok(())
}

#[test]
fn given_structural_digest_when_selected_depth_or_size_exceeds_bounds_then_hashing_rejects_without_tree_copy() -> TestResult {
    use inventory_core::record::{selected_content_hash, MapRef, RecordError, ValueRef, MAX_DEPTH, MAX_RECORD_BYTES};
    for (depth, accepted) in [(MAX_DEPTH - 1, true), (MAX_DEPTH, false)] {
        let mut bytes = vec![0xa1, 0x61, b'x'];
        bytes.extend(std::iter::repeat_n(0x81, depth));
        bytes.push(0xf6);
        let selected = MapRef::from_bytes(&bytes)?.get_name("x")?.ok_or("selected list absent")?;
        let digest = selected_content_hash(selected);
        if accepted { assert!(digest.is_ok()); } else { assert!(matches!(digest, Err(RecordError::DepthLimit))); }
    }
    let too_large = "x".repeat(MAX_RECORD_BYTES + 1);
    assert!(matches!(selected_content_hash(ValueRef::Text(&too_large)), Err(RecordError::SizeLimit)));
    Ok(())
}

