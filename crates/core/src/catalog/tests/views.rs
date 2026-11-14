use super::*;

#[test]
fn shape_lookup_retains_kind_name_and_missing_index() -> TestResult {
    // Given a single structure, when looked up, the shape view resolves only its own index.
    let bytes = root(service()?, 1)?;
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
    // Given a writer-created member, when read, its registry identity and target survive.
    let bytes = root(service()?, 1)?;
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
    // Given an operation with local input and Unit output, when read, both target forms resolve.
    let bytes = root(service()?, 1)?;
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
