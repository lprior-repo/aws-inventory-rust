#[path = "support/runtime_provenance.rs"]
mod runtime_provenance;
#[path = "support/synthetic_catalog.rs"]
mod synthetic_catalog;

use inventory_core::{
    catalog::runtime::{IdentityPart, IdentityPlan, RuntimeCatalog},
    record::{FieldId, Key, MapRef, Value, ValueRef},
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Fields = BTreeMap<Key, Value>;
const BASE: &[u8] = include_bytes!("../../../catalog/aws-api.catalog");

fn field(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn fields(value: &mut Value) -> TestResult<&mut Fields> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("compiled catalog fixture expected a map".into()),
    }
}

fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(entries.into_iter().map(|(id, value)| Ok((field(id)?, value)))
        .collect::<TestResult<Fields>>()?))
}

#[test]
fn loader_preserves_all_eight_identity_keys_at_the_exact_tuple_bound() -> TestResult {
    // Given distinct ordered paths at the accepted bound; nine is already
    // covered by runtime_plan_adversarial, so this is the missing inclusive side.
    let parts = (1..=8).map(|id| Value::List(vec![Value::Int(id)])).collect();
    let identity = map([
        (1, Value::Int(2)),
        (2, Value::List(parts)),
    ])?;
    let output_fields: Vec<u32> = (1..=8).collect();
    let bytes = synthetic_catalog::identity_catalog(&output_fields, identity)?;
    // When the public consumer loads a modeled catalog artifact.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime.plans().iter().find(|plan| plan.operation_name() == "ListThings")
        .ok_or("loaded plan missing")?;
    // Then neither an early bound check nor truncation can lose the eighth key.
    let expected = (1..=8).map(|id| Ok(vec![FieldId::new(id)?].into_boxed_slice()))
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(plan.identity(), &IdentityPlan::Key(expected.into_boxed_slice()));
    Ok(())
}

#[test]
fn loader_preserves_all_sixty_four_template_parts_at_the_exact_bound() -> TestResult {
    // Given 32 distinguishable literal/field pairs, not repeated identical tags.
    let pairs = (1..=32).map(|id| -> TestResult<[Value; 2]> {
        Ok([
            map([(1, Value::Int(0)), (2, Value::Text(format!("part-{id}/")))])?,
            map([(1, Value::Int(1)), (2, Value::List(vec![Value::Int(id)]))])?,
        ])
    }).collect::<TestResult<Vec<_>>>()?;
    let identity = map([
        (1, Value::Int(1)),
        (2, Value::List(pairs.into_iter().flatten().collect())),
    ])?;
    let output_fields: Vec<u32> = (1..=32).collect();
    let bytes = synthetic_catalog::identity_catalog(&output_fields, identity)?;
    // When loading through the public artifact decoder.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime.plans().iter().find(|plan| plan.operation_name() == "ListThings")
        .ok_or("loaded plan missing")?;
    // Then all 64 parts retain their kind, contents and order. The existing
    // adversarial suite supplies the complementary 65-part Bounds rejection.
    let expected = (1..=32).map(|id| -> TestResult<[IdentityPart; 2]> {
        Ok([
            IdentityPart::Literal(format!("part-{id}/")),
            IdentityPart::Field(vec![FieldId::new(id)?].into_boxed_slice()),
        ])
    }).collect::<TestResult<Vec<_>>>()?;
    assert_eq!(plan.identity(), &IdentityPlan::Template(
        expected.into_iter().flatten().collect::<Vec<_>>().into_boxed_slice(),
    ));
    Ok(())
}
