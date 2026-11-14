use super::*;
fn with_shape(key: CatalogKey, value: Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let Value::Map(mut service) = service()? else {
        return Err("service map".into());
    };
    let Some(Value::List(shapes)) = service.get_mut(&Key::Field(CatalogKey::Shapes.field()?))
    else {
        return Err("shapes".into());
    };
    let Some(Value::Map(shape)) = shapes.first_mut() else {
        return Err("shape map".into());
    };
    shape.insert(Key::Field(key.field()?), value);
    Ok(root(Value::Map(service), 1)?)
}
#[test]
fn invalid_shape_codes_and_references_are_reported_on_first_service_access() -> TestResult {
    // Given invalid kind codes, when a service is first requested, its schema is rejected.
    [-1, 23, i64::MAX]
        .into_iter()
        .try_for_each(|kind| -> TestResult {
            let bytes = with_shape(CatalogKey::Kind, Value::Int(kind))?;
            assert!(matches!(exercise(&bytes), Err(CatalogError::Reference)));
            Ok(())
        })
}
#[test]
fn out_of_range_member_targets_and_invalid_triples_are_not_silently_skipped() -> TestResult {
    // Given hostile member triples, when walking the service, every malformed member fails.
    let invalid = [
        Value::List(vec![Value::Int(7), Value::Int(1), Value::Bool(false)]),
        Value::List(vec![Value::Int(7), Value::Int(-15), Value::Bool(false)]),
        Value::List(vec![
            Value::Int(7),
            Value::Int(i64::MIN),
            Value::Bool(false),
        ]),
        Value::List(vec![Value::Int(7), Value::Int(-3)]),
        Value::List(vec![Value::Int(7), Value::Int(-3), Value::Int(0)]),
    ];
    invalid.into_iter().try_for_each(|member| -> TestResult {
        let bytes = with_shape(CatalogKey::Members, Value::List(vec![member]))?;
        assert!(matches!(exercise(&bytes), Err(CatalogError::Reference)));
        Ok(())
    })
}
#[test]
fn zero_member_identity_retains_the_record_error() -> TestResult {
    // Given registry ID zero, when read, the one FieldId constructor rejects it.
    let member = Value::List(vec![Value::Int(0), Value::Int(-3), Value::Bool(false)]);
    let bytes = with_shape(CatalogKey::Members, Value::List(vec![member]))?;
    assert!(matches!(
        exercise(&bytes),
        Err(CatalogError::Record(RecordError::FieldIdZero { .. }))
    ));
    Ok(())
}
#[test]
fn duplicate_and_descending_member_identities_are_rejected() -> TestResult {
    // Given nonascending registry IDs, when read, members are not reordered or discarded.
    [vec![7, 7], vec![8, 7]]
        .into_iter()
        .try_for_each(|ids| -> TestResult {
            let members = ids
                .into_iter()
                .map(|id| Value::List(vec![Value::Int(id), Value::Int(-3), Value::Bool(false)]))
                .collect();
            let bytes = with_shape(CatalogKey::Members, Value::List(members))?;
            assert!(matches!(exercise(&bytes), Err(CatalogError::Reference)));
            Ok(())
        })
}
#[test]
fn unexpected_container_targets_and_scalar_members_are_rejected() -> TestResult {
    // Given targets on a structure, when read, the inconsistent kind fails.
    let bytes = with_shape(CatalogKey::Element, Value::Int(-3))?;
    assert!(matches!(exercise(&bytes), Err(CatalogError::Reference)));
    // Given members on a scalar, when read, they cannot inflate structure-member totals.
    let bytes = with_shape(CatalogKey::Kind, Value::Int(2))?;
    assert!(matches!(
        exercise(&bytes),
        Err(CatalogError::Field(CatalogKey::Members))
    ));
    Ok(())
}
fn flip_text(marker: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = root(service()?, 1)?;
    let index = bytes
        .windows(marker.len())
        .position(|w| w == marker)
        .ok_or("marker")?;
    *bytes.get_mut(index).ok_or("index")? = 255;
    Ok(bytes)
}
#[test]
fn flipped_root_text_bytes_are_reported_immediately() -> TestResult {
    // Given invalid UTF-8 in a root field, when opened, corruption is immediate.
    let bytes = flip_text(b"test-commit")?;
    assert!(matches!(
        Catalog::borrow(&bytes),
        Err(CatalogError::Record(_))
    ));
    Ok(())
}
#[test]
fn flipped_service_text_bytes_are_reported_when_that_service_is_read() -> TestResult {
    // Given a flipped service byte, opening succeeds but reading that service fails.
    let bytes = flip_text(b"Example")?;
    let catalog = Catalog::borrow(&bytes)?;
    assert!(matches!(
        catalog.services()?.next().ok_or("service")?,
        Err(CatalogError::Record(_))
    ));
    Ok(())
}

#[test]
fn member_target_bound_and_invalid_preludes_are_exact_reference_errors() -> TestResult {
    // Given one shape, a member's exclusive bound and invalid prelude codes are never valid targets.
    [1, -15, i64::MIN]
        .into_iter()
        .try_for_each(|target| -> TestResult {
            let member = Value::List(vec![Value::Int(7), Value::Int(target), Value::Bool(false)]);
            let bytes = with_shape(CatalogKey::Members, Value::List(vec![member]))?;
            assert!(matches!(
                Catalog::borrow(&bytes)?.service("Example"),
                Err(CatalogError::Reference)
            ));
            Ok(())
        })
}
