//! Both fixture sweeps and arbitrary bytes exercise the same complete public reader traversal.
use super::*;
mod fixture;

pub(super) fn exercise(bytes: &[u8]) -> Result<String, CatalogError> {
    let catalog = Catalog::borrow(bytes)?;
    catalog.models_commit()?;
    catalog.services()?.try_for_each(|service| {
        let service = service?;
        inspect_service(service)?;
        let lookup = catalog
            .service(service.name()?)?
            .ok_or(CatalogError::Reference)?;
        assert_eq!(lookup.shape_id()?, service.shape_id()?);
        inspect_service(lookup)
    })?;
    let counts = catalog.counts()?;
    assert!(counts.get("services").is_some());
    assert_eq!(counts.get("not-a-counter"), None);
    counts.render()
}

fn inspect_service(service: Service<'_>) -> Result<(), CatalogError> {
    service.name()?;
    service.shape_id()?;
    service
        .protocols()?
        .try_for_each(|value| value.map(|_| ()))?;
    service
        .resources()?
        .try_for_each(|value| value.map(|_| ()))?;
    service.operations()?.try_for_each(|operation| {
        let operation = operation?;
        operation.name()?;
        assert!(operation.flags()? <= 31);
        check_target(operation.input()?, service.shape_count());
        check_target(operation.output()?, service.shape_count());
        Ok::<_, CatalogError>(())
    })?;
    inspect_shapes(service)
}

fn inspect_shapes(service: Service<'_>) -> Result<(), CatalogError> {
    service
        .shapes()?
        .enumerate()
        .try_for_each(|(index, shape)| {
            let shape = shape?;
            inspect_shape(shape, service.shape_count())?;
            let indexed = service.shape(index)?.ok_or(CatalogError::Reference)?;
            assert_eq!(indexed.name()?, shape.name()?);
            inspect_shape(indexed, service.shape_count())
        })?;
    assert!(service.shape(service.shape_count())?.is_none());
    Ok(())
}

fn inspect_shape(shape: Shape<'_>, count: usize) -> Result<(), CatalogError> {
    shape.name()?;
    assert!(kind_code(shape.kind()?).is_some());
    shape.arn()?;
    shape.members()?.try_for_each(|member| {
        let Member {
            id,
            target,
            required: _,
        } = member?;
        assert_ne!(id.value(), 0);
        check_target(target, count);
        Ok::<_, CatalogError>(())
    })?;
    [shape.element()?, shape.map_key()?, shape.map_value()?]
        .into_iter()
        .flatten()
        .for_each(|target| check_target(target, count));
    Ok(())
}

fn check_target(target: Target, count: usize) {
    if let Target::Shape(index) = target {
        assert!(
            index < count,
            "returned local target {index} outside service shape count {count}"
        );
    }
}

#[test]
fn rich_catalog_mutations_traverse_every_public_accessor_and_check_local_bounds() -> TestResult {
    // Given three services and all reference forms, every byte is swept with two fixed XOR masks.
    let bytes = fixture::bytes()?;
    exercise(&bytes)?;
    assert_eq!(Catalog::borrow(&bytes)?.counts()?.get("services"), Some(3));
    (0..bytes.len()).try_for_each(|position| -> TestResult {
        [1, 128]
            .into_iter()
            .try_for_each(|mask| check_flip(&bytes, position, mask))?;
        assert!(exercise(bytes.get(..position).ok_or("truncation")?).is_err());
        Ok(())
    })
}
