use super::*;

#[test]
fn shape_lookup_retains_kind_name_and_missing_index() -> TestResult {
    let bytes = root(service()?, CATALOG_VERSION)?;
    let service = Catalog::borrow(&bytes)?
        .service("Example")?
        .ok_or("service")?;
    let shape = service.shape(0)?.ok_or("shape")?;
    assert_eq!(shape.name()?, "Input");
    assert_eq!(shape.kind()?, "structure");
    assert_eq!(service.shape(1)?.map(|s| s.name()).transpose()?, None);
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
    let service = map([
        (CatalogKey::Name, Value::Text("Example".into())),
        (CatalogKey::ApiVersion, Value::Text("1".into())),
        (CatalogKey::ServiceShape, Value::Text("n#Example".into())),
        (CatalogKey::Protocols, Value::List(vec![])),
        (CatalogKey::Operations, Value::List(vec![])),
        (CatalogKey::Shapes, Value::List(vec![shape])),
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
    assert!(matches!(member.default_value()?, None));
    assert!(!member.client_optional()?);
    Ok(())
}
