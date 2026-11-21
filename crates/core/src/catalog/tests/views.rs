use super::*;

#[test]
fn shape_lookup_retains_kind_name_and_missing_index() -> TestResult {
    let second_shape = map([
        (CatalogKey::Name, Value::Text("Output".into())),
        (CatalogKey::Kind, Value::Int(14)),
        (CatalogKey::Members, Value::List(vec![])),
    ])?;
    let shapes = vec![shape()?, second_shape];
    let Value::Bytes(offsets) = shape_offsets(&shapes)? else {
        return Err("offset fixture".into());
    };
    let bytes = service_with_shapes(shapes, offsets)?;
    let service = Catalog::borrow(&bytes)?
        .service("Example")?
        .ok_or("service")?;
    let shapes = service.shapes()?.collect::<Result<Vec<_>, _>>()?;
    shapes.iter().enumerate().try_for_each(|(index, shape)| {
        let found = service.shape(index)?.ok_or("missing indexed shape")?;
        if found.name()? != shape.name()? || found.kind()? != shape.kind()? {
            return Err("indexed shape differs from iterator item".into());
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    assert!(service.shape(usize::MAX)?.is_none());
    assert_eq!(service.shape(2)?.map(|s| s.name()).transpose()?, None);
    Ok(())
}

#[test]
fn members_preserve_field_identity_target_and_requiredness() -> TestResult {
    let bytes = root(service()?, CATALOG_VERSION)?;
    let service = Catalog::borrow(&bytes)?
        .service("Example")?
        .ok_or("service")?;
    let shape = service.shape(0)?.ok_or("shape")?;
    let member = shape.members()?.next().ok_or("member")??;
    assert_eq!(member.id, crate::record::FieldId::new(7)?);
    assert_eq!(member.target, Target::Prelude("string"));
    assert!(!member.required);
    Ok(())
}

#[test]
fn operations_resolve_flags_and_input_output_targets() -> TestResult {
    let bytes = root(service()?, CATALOG_VERSION)?;
    let service = Catalog::borrow(&bytes)?
        .service("Example")?
        .ok_or("service")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    assert_eq!(operation.name()?, "ListThings");
    assert_eq!(operation.flags()?, 31);
    assert_eq!(operation.input()?, Target::Shape(0));
    assert_eq!(operation.output()?, Target::Prelude("unit"));
    Ok(())
}

#[test]
fn member_reads_present_null_default_and_client_optional() -> TestResult {
    let member = map([
        (CatalogKey::MemberId, Value::Int(7)),
        (CatalogKey::MemberTarget, Value::Int(-3)),
        (CatalogKey::MemberRequired, Value::Bool(false)),
        (CatalogKey::DefaultValue, Value::Null),
        (CatalogKey::ClientOptional, Value::Bool(true)),
    ])?;
    let shape = map([
        (CatalogKey::Name, Value::Text("Input".into())),
        (CatalogKey::Kind, Value::Int(14)),
        (CatalogKey::Members, Value::List(vec![member])),
    ])?;
    let shapes = vec![shape];
    let offsets = shape_offsets(&shapes)?;
    let service = map([
        (CatalogKey::Name, Value::Text("Example".into())),
        (CatalogKey::ApiVersion, Value::Text("1".into())),
        (CatalogKey::ServiceShape, Value::Text("n#Example".into())),
        (CatalogKey::Protocols, Value::List(vec![])),
        (CatalogKey::Operations, Value::List(vec![])),
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, offsets),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    let record = root(service, CATALOG_VERSION)?;
    let service = Catalog::borrow(&record)?
        .service("Example")?
        .ok_or("service")?;
    let member = service
        .shape(0)?
        .ok_or("shape")?
        .members()?
        .next()
        .ok_or("member")??;
    assert!(matches!(
        member.default_value()?,
        Some(crate::record::ValueRef::Null)
    ));
    assert!(member.client_optional()?);
    Ok(())
}
#[test]
fn member_defaults_are_absent_by_default() -> TestResult {
    let bytes = root(service()?, CATALOG_VERSION)?;
    let service = Catalog::borrow(&bytes)?
        .service("Example")?
        .ok_or("service")?;
    let member = service
        .shape(0)?
        .ok_or("shape")?
        .members()?
        .next()
        .ok_or("member")??;
    assert!(member.default_value()?.is_none());
    assert!(!member.client_optional()?);
    Ok(())
}
#[test]
fn shape_offsets_reject_corrupt_tables() -> TestResult {
    let shapes = vec![shape()?, shape()?];
    let Value::Bytes(valid) = shape_offsets(&shapes)? else {
        return Err("offset fixture".into());
    };
    invalid_offsets(valid)?.into_iter().try_for_each(|offsets| {
        let bytes = service_with_shapes(shapes.clone(), offsets)?;
        assert!(Catalog::borrow(&bytes)?.service("Example").is_err());
        Ok(())
    })
}

fn invalid_offsets(valid: Vec<u8>) -> Result<[Vec<u8>; 4], Box<dyn std::error::Error>> {
    let mut wrong_length = valid.clone();
    wrong_length.truncate(4);
    let mut non_monotonic = valid.clone();
    let first = non_monotonic.get(..4).ok_or("first offset")?.to_vec();
    non_monotonic
        .get_mut(4..8)
        .ok_or("second offset")?
        .copy_from_slice(&first);
    let mut out_of_bounds = valid.clone();
    out_of_bounds
        .get_mut(..4)
        .ok_or("first offset")?
        .copy_from_slice(&u32::MAX.to_be_bytes());
    let mut misaligned = valid;
    misaligned
        .get_mut(..4)
        .ok_or("first offset")?
        .copy_from_slice(&2u32.to_be_bytes());
    Ok([wrong_length, non_monotonic, out_of_bounds, misaligned])
}

fn service_with_shapes(shapes: Vec<Value>, offsets: Vec<u8>) -> Result<Vec<u8>, RecordError> {
    let Value::Map(mut fields) = service()? else {
        return Err(RecordError::NotMap);
    };
    fields.insert(Key::Field(CatalogKey::Shapes.field()?), Value::List(shapes));
    fields.insert(
        Key::Field(CatalogKey::ShapeOffsets.field()?),
        Value::Bytes(offsets),
    );
    root(Value::Map(fields), CATALOG_VERSION)
}
