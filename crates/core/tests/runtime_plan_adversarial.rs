#[path = "support/runtime_provenance.rs"]
mod runtime_provenance;
#[path = "support/synthetic_catalog.rs"]
mod synthetic_catalog;

use inventory_core::catalog::CatalogKey;
use inventory_core::{
    catalog::runtime::{
        IdentityPart, IdentityPlan, RuntimeCatalog, RuntimeCatalogError, TokenType,
    },
    record::{FieldId, Key, MapRef, RecordError, Value, ValueRef},
};
use std::collections::BTreeMap;
use synthetic_catalog::{catalog, catalog_map, column, item_root, member, structure};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Fields = BTreeMap<Key, Value>;
#[path = "support/base_catalog.rs"]
mod base_catalog;
use base_catalog::base;
const OPERATION: &str = "DescribeVpcs";

fn key(id: u32) -> Result<Key, RecordError> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn fields(value: &mut Value) -> TestResult<&mut Fields> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("fixture expected a field map".into()),
    }
}

fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        entries
            .into_iter()
            .map(|(id, value)| Ok((key(id)?, value)))
            .collect::<Result<Fields, RecordError>>()?,
    ))
}

pub fn scalar_item_catalog(identity: Value, pagination: Value) -> TestResult<Vec<u8>> {
    let items = catalog_map([
        (CatalogKey::Name, Value::Text("Items".to_owned())),
        (CatalogKey::Kind, Value::Int(16)),
        (CatalogKey::Element, Value::Int(-3)),
    ])?;
    let shapes = vec![
        structure("Input", vec![])?,
        items,
        structure("Output", vec![member(1, 1)?])?,
    ];
    let item = map([(1, Value::Int(1)), (2, path([1]))])?;
    catalog(0, 2, shapes, item, identity, pagination, vec![column([])?])
}

pub fn nested_structure_catalog(
    depth: u32,
    identity: Value,
    pagination: Value,
) -> TestResult<Vec<u8>> {
    let mut shapes = vec![catalog_map([
        (CatalogKey::Name, Value::Text("Scalar".to_owned())),
        (CatalogKey::Kind, Value::Int(2)),
    ])?];
    for level in 1..=depth {
        shapes.push(structure(
            &format!("Level{level}"),
            vec![member(
                1,
                i64::from(level.checked_sub(1).ok_or("zero level")?),
            )?],
        )?);
    }
    let input = u32::try_from(shapes.len())?;
    shapes.push(structure("Input", vec![])?);
    let ids = vec![1; usize::try_from(depth)?];
    catalog(
        input,
        depth,
        shapes,
        item_root()?,
        identity,
        pagination,
        vec![column(ids)?],
    )
}

pub fn pagination_catalog(
    input_fields: &[u32],
    output_fields: &[u32],
    pagination: Value,
) -> TestResult<Vec<u8>> {
    let input = input_fields
        .iter()
        .map(|&id| member(id, -3))
        .collect::<TestResult<Vec<_>>>()?;
    let output = output_fields
        .iter()
        .map(|&id| member(id, -3))
        .collect::<TestResult<Vec<_>>>()?;
    let shapes = vec![structure("Input", input)?, structure("Output", output)?];
    let identity = map([(1, Value::Int(2)), (2, Value::List(vec![path([11])]))])?;
    catalog(
        0,
        1,
        shapes,
        item_root()?,
        identity,
        pagination,
        vec![column([11])?],
    )
}

pub fn pagination_token(
    request_path: Vec<Value>,
    response_path: Vec<Value>,
    optional: bool,
    value_type: i64,
) -> TestResult<Value> {
    let request_placement = map([(1, Value::Int(3)), (2, Value::List(request_path.clone()))])?;
    let response_placement = map([(1, Value::Int(3)), (2, Value::List(response_path.clone()))])?;
    map([
        (1, Value::List(request_path)),
        (2, Value::List(response_path)),
        (3, Value::Int(value_type)),
        (4, request_placement),
        (5, response_placement),
        (6, Value::Bool(optional)),
    ])
}

fn path(ids: impl IntoIterator<Item = i64>) -> Value {
    Value::List(ids.into_iter().map(Value::Int).collect())
}

fn artifact(change: impl FnOnce(&mut Fields) -> TestResult) -> TestResult<Vec<u8>> {
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        base()?,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    let extension = fields(&mut root)?
        .get_mut(&key(60_000)?)
        .ok_or("base catalog has no runtime extension")?;
    let plans = fields(extension)?
        .get_mut(&key(2)?)
        .ok_or("base extension has no plans")?;
    let Value::List(plans) = plans else {
        return Err("fixture expected plan list".into());
    };
    let operation_key = key(2)?;
    let plan = plans
        .iter_mut()
        .find(|plan| match plan {
            Value::Map(fields) => {
                fields.get(&operation_key) == Some(&Value::Text(OPERATION.to_owned()))
            }
            _ => false,
        })
        .ok_or("base catalog has no DescribeVpcs plan")?;
    change(fields(plan)?)?;
    runtime_provenance::reseal(&mut root, base()?)?;
    Ok(inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
}

fn replacing(id: u32, value: Value) -> TestResult<Vec<u8>> {
    artifact(|fields| {
        fields.insert(key(id)?, value);
        Ok(())
    })
}

fn identity(kind: i64, parts: Vec<Value>) -> TestResult<Value> {
    map([(1, Value::Int(kind)), (2, Value::List(parts))])
}

fn template(parts: Vec<Value>) -> TestResult<Vec<u8>> {
    replacing(6, identity(1, parts)?)
}

fn token(request: Value, response: Value, optional: bool) -> TestResult<Value> {
    map([
        (1, request.clone()),
        (2, response.clone()),
        (3, Value::Int(0)),
        (4, map([(1, Value::Int(3)), (2, request)])?),
        (5, map([(1, Value::Int(3)), (2, response)])?),
        (6, Value::Bool(optional)),
    ])
}

fn pagination(tokens: Vec<Value>) -> TestResult<Vec<u8>> {
    replacing(
        8,
        map([(1, map([(1, Value::Int(0))])?), (2, Value::List(tokens))])?,
    )
}

fn rejects_capability(bytes: &[u8]) -> TestResult {
    match RuntimeCatalog::from_bytes(bytes) {
        Err(RuntimeCatalogError::Capability) => Ok(()),
        Err(error) => Err(format!("expected Capability, got {error:?}").into()),
        Ok(_) => Err("hostile artifact was accepted".into()),
    }
}

fn rejects_bounds(bytes: &[u8]) -> TestResult {
    match RuntimeCatalog::from_bytes(bytes) {
        Err(RuntimeCatalogError::Bounds) => Ok(()),
        Err(error) => Err(format!("expected Bounds, got {error:?}").into()),
        Ok(_) => Err("oversized artifact was accepted".into()),
    }
}

#[test]
fn loader_preserves_scalar_self_and_each_identity_context_when_template_is_canonical() -> TestResult
{
    let identity = map([
        (1, Value::Int(1)),
        (
            2,
            Value::List(vec![
                map([(1, Value::Int(0)), (2, Value::Text("arn:".to_owned()))])?,
                map([(1, Value::Int(2))])?,
                map([(1, Value::Int(3))])?,
                map([(1, Value::Int(4))])?,
                map([(1, Value::Int(1)), (2, path([]))])?,
            ]),
        ),
    ])?;
    let pagination = map([(1, Value::List(vec![]))])?;
    let bytes = scalar_item_catalog(identity, pagination)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .plans()
        .iter()
        .find(|plan| plan.operation_name() == "ListThings")
        .ok_or("loaded plan missing")?;
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Template(
            vec![
                IdentityPart::Literal("arn:".to_owned()),
                IdentityPart::Partition,
                IdentityPart::Account,
                IdentityPart::Region,
                IdentityPart::Field(Box::new([])),
            ]
            .into_boxed_slice()
        )
    );
    Ok(())
}

#[test]
fn loader_rejects_capability_when_identity_context_discriminant_is_unknown() -> TestResult {
    [-1, 5, i64::MAX]
        .into_iter()
        .try_for_each(|tag| rejects_capability(&template(vec![map([(1, Value::Int(tag))])?])?))
}

#[test]
fn loader_rejects_capability_when_identity_parts_are_mistyped_or_noncanonical() -> TestResult {
    let mut named_context = BTreeMap::new();
    named_context.insert(key(1)?, Value::Int(2));
    named_context.insert(Key::Name("payload".to_owned()), Value::Null);
    let parts = vec![
        Value::Text("partition".to_owned()),
        map([(1, Value::Text("2".to_owned()))])?,
        map([(1, Value::Int(0)), (2, Value::Int(7))])?,
        map([(1, Value::Int(0))])?,
        map([(1, Value::Int(1))])?,
        map([(1, Value::Int(1)), (2, Value::Text("self".to_owned()))])?,
        Value::Map(named_context),
    ];
    parts
        .into_iter()
        .try_for_each(|part| rejects_capability(&template(vec![part])?))
}

#[test]
fn loader_rejects_capability_when_context_parts_carry_even_null_payloads() -> TestResult {
    [2, 3, 4].into_iter().try_for_each(|tag| {
        rejects_capability(&template(vec![map([
            (1, Value::Int(tag)),
            (2, Value::Null),
        ])?])?)
    })
}

#[test]
fn loader_rejects_capability_when_identity_tuple_or_template_has_no_parts() -> TestResult {
    [1, 2]
        .into_iter()
        .try_for_each(|kind| rejects_capability(&replacing(6, identity(kind, vec![])?)?))
}

#[test]
fn loader_rejects_bounds_when_identity_key_tuple_has_nine_parts() -> TestResult {
    let bytes = replacing(6, identity(2, vec![path([1]); 9])?)?;
    rejects_bounds(&bytes)
}

#[test]
fn loader_rejects_bounds_when_identity_template_has_sixty_five_parts() -> TestResult {
    let bytes = template(vec![map([(1, Value::Int(2))])?; 65])?;
    rejects_bounds(&bytes)
}

#[test]
fn loader_rejects_capability_when_scalar_selected_path_contains_non_field_values() -> TestResult {
    [
        Value::Text("1".to_owned()),
        Value::Bool(true),
        Value::Int(-1),
        Value::Int(i64::from(u32::MAX) + 1),
    ]
    .into_iter()
    .try_for_each(|part| {
        rejects_capability(&template(vec![map([
            (1, Value::Int(1)),
            (2, Value::List(vec![part])),
        ])?])?)
    })
}

#[test]
fn loader_preserves_record_error_when_scalar_selected_path_contains_field_zero() -> TestResult {
    let bytes = template(vec![map([(1, Value::Int(1)), (2, path([0]))])?])?;
    match RuntimeCatalog::from_bytes(&bytes) {
        Err(RuntimeCatalogError::Record(error)) => {
            assert_eq!(error, RecordError::FieldIdZero { offset: 0 });
            Ok(())
        }
        Err(error) => Err(format!("expected Record(FieldIdZero), got {error:?}").into()),
        Ok(_) => Err("zero selected field ID was accepted".into()),
    }
}

#[test]
fn loader_rejects_bounds_when_scalar_selected_path_has_nine_segments() -> TestResult {
    rejects_bounds(&template(vec![map([
        (1, Value::Int(1)),
        (2, path([1; 9])),
    ])?])?)
}

#[test]
fn loader_preserves_eight_segment_selected_path_at_the_exact_bound() -> TestResult {
    let identity = identity(2, vec![path([1; 8]); 8])?;
    let pagination = map([(1, Value::List(vec![]))])?;
    let bytes = nested_structure_catalog(8, identity, pagination)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .plans()
        .iter()
        .find(|plan| plan.operation_name() == "ListThings")
        .ok_or("loaded plan missing")?;
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Key(
            vec![vec![FieldId::new(1)?; 8].into_boxed_slice(); 8].into_boxed_slice(),
        )
    );
    Ok(())
}

#[test]
fn loader_rejects_capability_when_any_required_plan_field_is_missing() -> TestResult {
    (1..=13).try_for_each(|id| {
        let bytes = artifact(|fields| {
            fields
                .remove(&key(id)?)
                .ok_or("required fixture field missing")?;
            Ok(())
        })?;
        rejects_capability(&bytes)
    })
}

#[test]
fn loader_rejects_capability_when_plan_has_unknown_numeric_fields() -> TestResult {
    let runtime = RuntimeCatalog::from_bytes(base()?)?;
    assert_eq!(
        runtime
            .lookup("ec2", OPERATION)
            .ok_or("control plan missing")?
            .operation_name(),
        OPERATION
    );
    rejects_capability(&replacing(65_535, Value::Null)?)
}

#[test]
fn loader_rejects_capability_when_plan_has_named_fields() -> TestResult {
    let bytes = artifact(|fields| {
        fields.insert(Key::Name("identity".to_owned()), Value::Null);
        Ok(())
    })?;
    rejects_capability(&bytes)
}

#[test]
fn loader_rejects_capability_when_tag_metadata_is_not_a_map() -> TestResult {
    rejects_capability(&replacing(14, Value::Null)?)
}

#[test]
fn loader_rejects_capability_when_emit_field_is_not_boolean_int() -> TestResult {
    rejects_capability(&replacing(25, Value::Int(2))?)
}

#[test]
fn loader_rejects_capability_when_emit_field_is_null() -> TestResult {
    rejects_capability(&replacing(25, Value::Null)?)
}

#[test]
fn loader_rejects_capability_when_cursor_tuple_component_path_is_empty() -> TestResult {
    [(path([]), path([2])), (path([1]), path([]))]
        .into_iter()
        .try_for_each(|(request, response)| {
            rejects_capability(&pagination(vec![token(request, response, false)?])?)
        })
}

#[test]
fn loader_rejects_bounds_when_cursor_tuple_has_nine_components() -> TestResult {
    let tokens = (1..=9)
        .map(|id| token(path([id]), path([id + 10]), false))
        .collect::<TestResult<Vec<_>>>()?;
    rejects_bounds(&pagination(tokens)?)
}

#[test]
fn loader_rejects_capability_when_cursor_components_overwrite_the_same_request_path() -> TestResult
{
    let bytes = pagination(vec![
        token(path([1]), path([2]), false)?,
        token(path([1]), path([3]), true)?,
    ])?;
    rejects_capability(&bytes)
}

#[test]
fn loader_preserves_required_and_optional_components_at_cursor_tuple_limit() -> TestResult {
    let input_fields: Vec<u32> = (1..=8).collect();
    let output_fields: Vec<u32> = (11..=18).collect();
    let tokens = (1..=8)
        .map(|id| {
            pagination_token(
                vec![Value::Int(i64::from(id))],
                vec![Value::Int(i64::from(id + 10))],
                id != 1,
                0,
            )
        })
        .collect::<TestResult<Vec<_>>>()?;
    let pagination = map([(1, Value::List(tokens))])?;
    let bytes = pagination_catalog(&input_fields, &output_fields, pagination)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .plans()
        .iter()
        .find(|plan| plan.operation_name() == "ListThings")
        .ok_or("loaded plan missing")?;
    let actual = plan
        .pagination()
        .tokens
        .iter()
        .map(|token| {
            (
                token
                    .request_path
                    .iter()
                    .map(|id| id.value())
                    .collect::<Vec<_>>(),
                token
                    .response_path
                    .iter()
                    .map(|id| id.value())
                    .collect::<Vec<_>>(),
                token.optional,
                token.value_type,
            )
        })
        .collect::<Vec<_>>();
    let expected = (1..=8)
        .map(|id| (vec![id], vec![id + 10], id != 1, TokenType::String))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    Ok(())
}
