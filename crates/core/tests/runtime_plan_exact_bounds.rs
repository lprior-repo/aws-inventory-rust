#[path = "support/runtime_provenance.rs"]
mod runtime_provenance;

use inventory_core::{
    catalog::runtime::{IdentityPart, IdentityPlan, RuntimeCatalog},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
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

fn with_identity(kind: i64, parts: Vec<Value>) -> TestResult<Vec<u8>> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(BASE)?).to_value()?;
    let extension = fields(&mut root)?.get_mut(&field(60_000)?)
        .ok_or("compiled catalog has no runtime extension")?;
    // The tag-aware artifact contract is version two, even when this plan has
    // no tag observation metadata.
    fields(extension)?.insert(field(1)?, Value::Int(2));
    let plans = fields(extension)?.get_mut(&field(2)?)
        .ok_or("compiled catalog has no runtime plans")?;
    let Value::List(plans) = plans else {
        return Err("compiled catalog expected a plan list".into());
    };
    let operation = field(2)?;
    let plan = plans.iter_mut().find(|plan| match plan {
        Value::Map(fields) => fields.get(&operation)
            == Some(&Value::Text("DescribeVpcs".to_owned())),
        _ => false,
    }).ok_or("compiled catalog has no DescribeVpcs plan")?;
    fields(plan)?.insert(field(6)?, map([
        (1, Value::Int(kind)), (2, Value::List(parts)),
    ])?);
    runtime_provenance::reseal(&mut root, BASE)?;
    Ok(Record::encode(&root)?.into_bytes())
}

#[test]
fn loader_preserves_all_eight_identity_keys_at_the_exact_tuple_bound() -> TestResult {
    // Given distinct ordered paths at the accepted bound; nine is already
    // covered by runtime_plan_adversarial, so this is the missing inclusive side.
    let parts = (1..=8).map(|id| Value::List(vec![Value::Int(id)])).collect();
    let bytes = with_identity(2, parts)?;
    // When the public consumer loads a real compiled catalog artifact.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime.lookup("ec2", "DescribeVpcs").ok_or("loaded plan missing")?;
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
    let bytes = with_identity(1, pairs.into_iter().flatten().collect())?;
    // When loading through the public artifact decoder.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime.lookup("ec2", "DescribeVpcs").ok_or("loaded plan missing")?;
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
