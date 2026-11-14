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
    Ok(root(Value::Map(service), CATALOG_VERSION)?)
}
fn member(id: i64, target: i64, required: Value) -> Result<Value, RecordError> {
    map([
        (CatalogKey::MemberId, Value::Int(id)),
        (CatalogKey::MemberTarget, Value::Int(target)),
        (CatalogKey::MemberRequired, required),
    ])
}
#[test]
fn invalid_shape_codes_and_references_are_reported_on_first_service_access() -> TestResult {
    [-1, 23, i64::MAX]
        .into_iter()
        .try_for_each(|kind| -> TestResult {
            let bytes = with_shape(CatalogKey::Kind, Value::Int(kind))?;
            assert!(matches!(exercise(&bytes), Err(CatalogError::Reference)));
            Ok(())
        })
}
#[test]
fn out_of_range_member_targets_and_invalid_member_fields_are_not_silently_skipped() -> TestResult {
    let invalid = [
        member(7, 1, Value::Bool(false))?,
        member(7, -15, Value::Bool(false))?,
        member(7, i64::MIN, Value::Bool(false))?,
        map([
            (CatalogKey::MemberId, Value::Int(7)),
            (CatalogKey::MemberTarget, Value::Int(-3)),
        ])?,
        member(7, -3, Value::Int(0))?,
    ];
    invalid.into_iter().try_for_each(|member| -> TestResult {
        let bytes = with_shape(CatalogKey::Members, Value::List(vec![member]))?;
        assert!(matches!(exercise(&bytes), Err(CatalogError::Reference)));
        Ok(())
    })
}
#[test]
fn zero_member_identity_retains_the_record_error() -> TestResult {
    let member = member(0, -3, Value::Bool(false))?;
    let bytes = with_shape(CatalogKey::Members, Value::List(vec![member]))?;
    assert!(matches!(
        exercise(&bytes),
        Err(CatalogError::Record(RecordError::FieldIdZero { .. }))
    ));
    Ok(())
}
#[test]
fn duplicate_and_descending_member_identities_are_rejected() -> TestResult {
    let ordered = with_shape(
        CatalogKey::Members,
        Value::List(vec![
            member(7, -3, Value::Bool(false))?,
            member(8, -3, Value::Bool(false))?,
        ]),
    )?;
    assert_eq!(Catalog::borrow(&ordered)?.counts()?.get("members"), Some(2));
    [vec![7, 7], vec![8, 7]]
        .into_iter()
        .try_for_each(|ids| -> TestResult {
            let members = ids
                .into_iter()
                .map(|id| member(id, -3, Value::Bool(false)))
                .collect::<Result<Vec<_>, _>>()?;
            let bytes = with_shape(CatalogKey::Members, Value::List(members))?;
            assert!(matches!(exercise(&bytes), Err(CatalogError::Reference)));
            Ok(())
        })
}
#[test]
fn unexpected_container_targets_and_scalar_members_are_rejected() -> TestResult {
    let bytes = with_shape(CatalogKey::Element, Value::Int(-3))?;
    assert!(matches!(exercise(&bytes), Err(CatalogError::Reference)));

    let bytes = with_shape(CatalogKey::Kind, Value::Int(2))?;
    assert!(matches!(
        exercise(&bytes),
        Err(CatalogError::Field(CatalogKey::Members))
    ));
    Ok(())
}
fn flip_text(marker: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = root(service()?, CATALOG_VERSION)?;
    let index = bytes
        .windows(marker.len())
        .position(|w| w == marker)
        .ok_or("marker")?;
    *bytes.get_mut(index).ok_or("index")? = 255;
    Ok(bytes)
}
#[test]
fn flipped_root_text_bytes_are_reported_immediately() -> TestResult {
    let bytes = flip_text(b"test-commit")?;
    assert!(matches!(
        Catalog::borrow(&bytes),
        Err(CatalogError::Record(_))
    ));
    Ok(())
}
#[test]
fn flipped_service_text_bytes_are_reported_when_that_service_is_read() -> TestResult {
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
    [1, -15, i64::MIN]
        .into_iter()
        .try_for_each(|target| -> TestResult {
            let member = member(7, target, Value::Bool(false))?;
            let bytes = with_shape(CatalogKey::Members, Value::List(vec![member]))?;
            assert!(matches!(
                Catalog::borrow(&bytes)?.service("Example"),
                Err(CatalogError::Reference)
            ));
            Ok(())
        })
}
