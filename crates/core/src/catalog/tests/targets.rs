use super::*;

fn with_operation(key: CatalogKey, value: i64) -> Result<Vec<u8>, RecordError> {
    let Value::Map(mut fields) = service()? else {
        return Err(RecordError::NotMap);
    };
    let Value::Map(mut operation) = operation()? else {
        return Err(RecordError::NotMap);
    };
    operation.insert(Key::Field(key.field()?), Value::Int(value));
    fields.insert(
        Key::Field(CatalogKey::Operations.field()?),
        Value::List(vec![Value::Map(operation)]),
    );
    root(Value::Map(fields), CATALOG_VERSION)
}

fn with_container(kind: i64, targets: &[(CatalogKey, i64)]) -> Result<Vec<u8>, RecordError> {
    let fields = [
        (CatalogKey::Name, Value::Text("Container".into())),
        (CatalogKey::Kind, Value::Int(kind)),
        (CatalogKey::Members, Value::List(vec![])),
    ];
    let targets = targets
        .iter()
        .map(|(key, value)| (*key, Value::Int(*value)));
    let shape = map(fields.into_iter().chain(targets))?;
    let Value::Map(mut fields) = service()? else {
        return Err(RecordError::NotMap);
    };
    fields.insert(
        Key::Field(CatalogKey::Shapes.field()?),
        Value::List(vec![shape]),
    );
    root(Value::Map(fields), CATALOG_VERSION)
}

#[test]
fn operation_input_and_output_equal_to_shape_count_are_exact_reference_errors() -> TestResult {
    [CatalogKey::Input, CatalogKey::Output]
        .into_iter()
        .try_for_each(|key| -> TestResult {
            let bytes = with_operation(key, 1)?;
            assert!(matches!(
                Catalog::borrow(&bytes)?.service("Example"),
                Err(CatalogError::Reference)
            ));
            Ok(())
        })
}

#[test]
fn invalid_negative_operation_preludes_are_exact_reference_errors() -> TestResult {
    [CatalogKey::Input, CatalogKey::Output]
        .into_iter()
        .try_for_each(|key| -> TestResult {
            [-15, i64::MIN]
                .into_iter()
                .try_for_each(|value| -> TestResult {
                    let bytes = with_operation(key, value)?;
                    assert!(matches!(
                        Catalog::borrow(&bytes)?.service("Example"),
                        Err(CatalogError::Reference)
                    ));
                    Ok(())
                })
        })
}

#[test]
fn list_element_at_shape_count_and_invalid_negative_codes_are_exact_reference_errors() -> TestResult
{
    [1, -15, i64::MIN]
        .into_iter()
        .try_for_each(|target| -> TestResult {
            let bytes = with_container(16, &[(CatalogKey::Element, target)])?;
            assert!(matches!(
                Catalog::borrow(&bytes)?.service("Example"),
                Err(CatalogError::Reference)
            ));
            Ok(())
        })
}

#[test]
fn both_map_targets_at_shape_count_and_invalid_preludes_are_exact_reference_errors() -> TestResult {
    [CatalogKey::MapKey, CatalogKey::MapValue]
        .into_iter()
        .try_for_each(|field| -> TestResult {
            [1, -15, i64::MIN]
                .into_iter()
                .try_for_each(|bad| map_case(field, bad))
        })
}

fn map_case(field: CatalogKey, bad: i64) -> TestResult {
    let targets = match field {
        CatalogKey::MapKey => [(CatalogKey::MapKey, bad), (CatalogKey::MapValue, 0)],
        CatalogKey::MapValue => [(CatalogKey::MapKey, -3), (CatalogKey::MapValue, bad)],
        _ => return Err("not a map target".into()),
    };
    let bytes = with_container(17, &targets)?;
    assert!(matches!(
        Catalog::borrow(&bytes)?.service("Example"),
        Err(CatalogError::Reference)
    ));
    Ok(())
}

#[test]
fn flags_above_31_are_exact_reference_errors() -> TestResult {
    [32, 63, 255, i64::MAX]
        .into_iter()
        .try_for_each(|flags| -> TestResult {
            let bytes = with_operation(CatalogKey::Flags, flags)?;
            assert!(matches!(
                Catalog::borrow(&bytes)?.service("Example"),
                Err(CatalogError::Reference)
            ));
            Ok(())
        })
}
