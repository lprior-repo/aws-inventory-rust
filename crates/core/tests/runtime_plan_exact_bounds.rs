#[path = "support/base_catalog.rs"]
mod base_catalog;
#[path = "support/synthetic_catalog.rs"]
mod synthetic_catalog;

use inventory_core::{
    catalog::runtime::{IdentityPart, IdentityPlan, RuntimeCatalog},
    record::{FieldId, Key, Value},
};
use std::collections::BTreeMap;
use synthetic_catalog::{catalog, column, item_root, member, structure};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Fields = BTreeMap<Key, Value>;

fn field(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        entries
            .into_iter()
            .map(|(id, value)| Ok((field(id)?, value)))
            .collect::<TestResult<Fields>>()?,
    ))
}

fn identity_catalog(output_fields: &[u32], identity: Value) -> TestResult<Vec<u8>> {
    let members = output_fields
        .iter()
        .map(|&id| member(id, -3))
        .collect::<TestResult<Vec<_>>>()?;
    let shapes = vec![structure("Input", vec![])?, structure("Output", members)?];
    let columns = output_fields
        .iter()
        .map(|&id| column([id]))
        .collect::<TestResult<Vec<_>>>()?;
    catalog(
        0,
        1,
        shapes,
        item_root()?,
        identity,
        map([(1, Value::List(vec![]))])?,
        columns,
    )
}

#[test]
fn loader_preserves_all_eight_identity_keys_at_the_exact_tuple_bound() -> TestResult {
    let parts = (1..=8)
        .map(|id| Value::List(vec![Value::Int(id)]))
        .collect();
    let identity = map([(1, Value::Int(2)), (2, Value::List(parts))])?;
    let output_fields: Vec<u32> = (1..=8).collect();
    let bytes = identity_catalog(&output_fields, identity)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .plans()
        .iter()
        .find(|plan| plan.operation_name() == "ListThings")
        .ok_or("loaded plan missing")?;
    let expected = (1..=8)
        .map(|id| Ok(vec![FieldId::new(id)?].into_boxed_slice()))
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Key(expected.into_boxed_slice())
    );
    Ok(())
}

#[test]
fn loader_preserves_all_sixty_four_template_parts_at_the_exact_bound() -> TestResult {
    let pairs = (1..=32)
        .map(|id| -> TestResult<[Value; 2]> {
            Ok([
                map([(1, Value::Int(0)), (2, Value::Text(format!("part-{id}/")))])?,
                map([(1, Value::Int(1)), (2, Value::List(vec![Value::Int(id)]))])?,
            ])
        })
        .collect::<TestResult<Vec<_>>>()?;
    let identity = map([
        (1, Value::Int(1)),
        (2, Value::List(pairs.into_iter().flatten().collect())),
    ])?;
    let output_fields: Vec<u32> = (1..=32).collect();
    let bytes = identity_catalog(&output_fields, identity)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .plans()
        .iter()
        .find(|plan| plan.operation_name() == "ListThings")
        .ok_or("loaded plan missing")?;
    let expected = (1..=32)
        .map(|id| -> TestResult<[IdentityPart; 2]> {
            Ok([
                IdentityPart::Literal(format!("part-{id}/")),
                IdentityPart::Field(vec![FieldId::new(id)?].into_boxed_slice()),
            ])
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Template(
            expected
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    );
    Ok(())
}
