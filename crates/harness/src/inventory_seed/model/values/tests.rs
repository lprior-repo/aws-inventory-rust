use super::*;
use inventory_core::{catalog::Catalog, record::Value};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn catalog_bytes() -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"),
    )
}

fn operation(bytes: &[u8]) -> TestResult<SeedOperation<'_>> {
    Ok(locate(Catalog::borrow(bytes)?, "bedrock-runtime", "StartAsyncInvoke")?)
}

fn document() -> Value {
    Value::Map(BTreeMap::from([(
        Key::Name(String::from("arbitrary")),
        Value::List(vec![Value::Map(BTreeMap::from([(
            Key::Name(String::from("nested")),
            Value::Bool(true),
        )]))]),
    )]))
}

#[test]
fn given_nested_document_when_seed_compiled_then_names_and_arrays_are_preserved() -> TestResult {
    let bytes = catalog_bytes()?;
    let operation = operation(&bytes)?;
    let value = SeedInputValue::Structure(vec![(
        String::from("arbitrary"),
        SeedInputValue::List(vec![SeedInputValue::Structure(vec![(
            String::from("nested"),
            SeedInputValue::Boolean(true),
        )])]),
    )]);
    let prepared = prepare(&[operation], operation, &FieldPath::new(&["modelInput"])?, &value)?;
    let PreparedInputValue::Structure(fields) = prepared else {
        return Err("document structure was not preserved".into());
    };
    let Some((Key::Name(name), PreparedInputValue::List(items))) = fields.first() else {
        return Err("document name or list was not preserved".into());
    };
    assert_eq!(name, "arbitrary");
    assert!(matches!(items.first(), Some(PreparedInputValue::Structure(fields))
        if matches!(fields.first(), Some((Key::Name(name), PreparedInputValue::Boolean(true)))
            if name == "nested")));
    Ok(())
}

#[test]
fn given_document_literal_when_resolved_then_arbitrary_nested_keys_remain_names() -> TestResult {
    let bytes = catalog_bytes()?;
    let operation = operation(&bytes)?;
    let target = members::path_target(
        operation.service,
        operation.operation.input()?,
        &FieldPath::new(&["modelInput"])?,
    )?;
    let value = document();
    let resolved = crate::inventory_seed::literal::resolve(operation.service, target, &value)?;
    assert_eq!(resolved, value);
    Ok(())
}

#[test]
fn given_prelude_document_when_literal_resolved_then_arrays_are_supported() -> TestResult {
    let bytes = catalog_bytes()?;
    let operation = operation(&bytes)?;
    let value = Value::List(vec![document()]);
    let resolved = crate::inventory_seed::literal::resolve(
        operation.service,
        Target::Prelude("document"),
        &value,
    )?;
    assert_eq!(resolved, value);
    Ok(())
}

#[test]
fn given_modeled_structure_when_unknown_member_compiled_then_it_is_still_rejected() -> TestResult {
    let bytes = catalog_bytes()?;
    let operation = operation(&bytes)?;
    let value = SeedInputValue::Structure(vec![(
        String::from("arbitrary"),
        SeedInputValue::Boolean(true),
    )]);
    assert!(prepare(
        &[operation],
        operation,
        &FieldPath::new(&["outputDataConfig"])?,
        &value,
    ).is_err());
    Ok(())
}
