#[path = "support/bindings.rs"]
mod fixture;
mod stages;
use catalog_tool::overlay::{self, Binding, ContextSource};
use fixture::{TestResult, compiled};
use inventory_core::{catalog::runtime::{BindingSource, RuntimeCatalog}, record::{FieldId, Key, MapRef, Record, Value, ValueRef}};
use std::collections::BTreeMap;

#[test]
fn given_eight_producers_when_compiled_then_each_namespace_and_projection_survives_exactly() -> TestResult {
    let bytes = compiled(8, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let child = catalog.lookup("bindings", "ListChildren").ok_or("child missing")?;
    let alternatives = child.bindings().iter().filter_map(|binding| match &binding.source {
        BindingSource::Alternatives(alternatives) => Some(alternatives), _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 2);
    for (index, plan) in catalog.plans().iter().filter(|plan| plan.operation_name() != "ListChildren").enumerate() {
        assert_eq!(plan.operation_name(), format!("ListParents{index}"));
        assert_eq!(plan.context_projection().selected.len(), 1);
        assert_eq!(plan.context_projection().request.len(), 1);
        for set in &alternatives {
            assert_eq!(set.len(), 8);
            let alternative = set.get(index).ok_or("producer missing")?;
            assert_eq!(alternative.operation.as_ref(), plan.operation_name());
            assert_eq!(alternative.service.as_ref(), "bindings");
            let projection = match alternative.source {
                inventory_core::catalog::runtime::ContextSource::SelectedItem => &plan.context_projection().selected,
                inventory_core::catalog::runtime::ContextSource::Request => &plan.context_projection().request,
                _ => return Err("unexpected namespace".into()),
            };
            assert_eq!(projection.first().ok_or("projection missing")?.as_ref(), alternative.path.as_ref());
        }
    }
    assert_eq!(child.facet_precedence(), 1);
    assert!(child.bindings().iter().any(|binding| matches!(binding.source, BindingSource::RegionList)));
    Ok(())
}

#[test]
fn given_empty_nine_duplicate_missing_or_incompatible_producers_when_compiled_then_checker_rejects_every_case() -> TestResult {
    for count in [0, 9] { assert!(compiled(count, |_| {}).is_err(), "accepted producer count {count}"); }
    for corruption in 0..5 {
        let result = compiled(2, |child| {
            if let Some(Binding::FromAny(alternatives)) = child.bind.values_mut().find(|binding|
                matches!(binding, Binding::FromAny(values) if values.first().is_some_and(|alternative| alternative.source == ContextSource::SelectedItem))) {
                match corruption {
                    0 => alternatives.push(alternatives.first().expect("fixture has producer").clone()),
                    1 => alternatives.first_mut().expect("fixture has producer").path = overlay::MemberPath::new("Missing").expect("valid path"),
                    2 => alternatives.last_mut().expect("fixture has producer").path = overlay::MemberPath::new("Count").expect("valid path"),
                    3 => alternatives.last_mut().expect("fixture has producer").operation = overlay::OperationName::new("ListMissing").expect("valid operation"),
                    _ => { alternatives.pop(); }
                }
            }
        });
        assert!(result.is_err(), "accepted producer corruption {corruption}");
    }
    assert!(compiled(1, |child| {
        child.bind.insert(overlay::MemberPath::new("EntityId").expect("valid path"), Binding::RegionList);
    }).is_err(), "region list accepted for scalar destination");
    Ok(())
}

fn map(value: &mut Value) -> TestResult<&mut BTreeMap<Key, Value>> {
    match value { Value::Map(value) => Ok(value), _ => Err("not map".into()) }
}
fn field(number: u32) -> TestResult<Key> { Ok(Key::Field(FieldId::new(number)?)) }
fn tamper(bytes: &[u8], corrupt: usize) -> TestResult<Record> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let extension = map(map(&mut root)?.get_mut(&field(60_000)?).ok_or("extension missing")?)?;
    let Value::List(plans) = extension.get_mut(&field(2)?).ok_or("plans missing")? else { return Err("not plans".into()); };
    let child = plans.iter_mut().find(|plan| match plan {
        Value::Map(fields) => fields.values().any(|value| matches!(value, Value::Text(value) if value == "ListChildren")), _ => false,
    }).ok_or("child metadata missing")?;
    let Value::List(bindings) = map(child)?.get_mut(&field(7)?).ok_or("bindings missing")? else { return Err("not bindings".into()); };
    let binding = bindings.iter_mut().find(|binding| match binding {
        Value::Map(fields) => fields.get(&field(2).expect("field")).is_some_and(|value| matches!(value, Value::Int(12))), _ => false,
    }).ok_or("alternatives missing")?;
    let Value::List(alternatives) = map(binding)?.get_mut(&field(3)?).ok_or("entries missing")? else { return Err("not entries".into()); };
    let alternative = map(alternatives.last_mut().ok_or("entry missing")?)?;
    match corrupt {
        0 => { alternative.insert(field(1)?, Value::Text("foreign-service".into())); }
        1 => { alternative.insert(field(2)?, Value::Text("ListMissing".into())); }
        2 => { alternative.insert(field(2)?, Value::Text("ListChildren".into())); }
        3 => { alternative.insert(field(4)?, Value::List(vec![Value::Int(999_999)])); }
        4 => { alternative.insert(field(3)?, Value::Int(7)); }
        _ => { alternatives.pop(); }
    }
    let encoded = Record::encode(&Value::Map(extension.clone()))?;
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
    map(extension.get_mut(&field(7)?).ok_or("receipt missing")?)?.insert(field(7)?, Value::Bytes(digest.to_vec()));
    Ok(Record::encode(&root)?)
}

#[test]
fn given_rehashed_corrupt_alternative_metadata_when_loaded_then_every_producer_is_model_checked() -> TestResult {
    let bytes = compiled(2, |_| {})?;
    for corruption in 0..6 {
        let record = tamper(&bytes, corruption)?;
        assert!(matches!(RuntimeCatalog::from_bytes(record.as_bytes()), Err(inventory_core::catalog::runtime::RuntimeCatalogError::Capability)), "accepted corruption {corruption}");
    }
    Ok(())
}
