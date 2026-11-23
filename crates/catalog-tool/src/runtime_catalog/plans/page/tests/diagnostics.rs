use super::*;
use crate::runtime_catalog::{ServiceIndex, plan_validation};
use inventory_core::catalog::{Operation, Service};

fn with_model(
    check: impl FnOnce(Service<'_>, Operation<'_>, &ServiceIndex<'_>) -> TestResult,
) -> TestResult {
    let directory = tempfile::tempdir()?;
    let models = directory.path().join("models/example/service/2020-01-01");
    std::fs::create_dir_all(&models)?;
    std::fs::write(models.join("model.json"), MODEL)?;
    let fields = crate::smithy::Registry::parse(REGISTRY)?;
    let bytes = crate::smithy::build_catalog(directory.path(), &fields, "diagnostics")?;
    let catalog = inventory_core::catalog::Catalog::borrow(&bytes)?;
    let services = ServiceIndex::build(catalog)?;
    let service = services.get("example")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    check(service, operation, &services)
}

fn overlay() -> Result<OperationOverlay, crate::overlay::NameError> {
    Ok(OperationOverlay {
        items: Some(ItemSelector::parse("Items")?),
        identity: Some(Identity::Key(vec![MemberPath::new("Id")?])),
        ..OperationOverlay::default()
    })
}

fn assert_path<T>(result: Result<T, RuntimeCatalogCompileError>, expected: &str) {
    assert!(matches!(result, Err(RuntimeCatalogCompileError::Path { path }) if path == expected));
}

#[test]
fn given_unknown_service_when_indexed_then_service_is_reported() -> TestResult {
    with_model(|_, _, services| {
        assert!(
            matches!(services.get("absent"), Err(RuntimeCatalogCompileError::CapabilityContext { service, operation, path })
            if service == "absent" && operation == "-" && path == "-")
        );
        Ok(())
    })
}

#[test]
fn given_unknown_member_when_indexed_then_member_is_reported() -> TestResult {
    with_model(|_, _, services| {
        assert!(
            matches!(services.field_id("Absent"), Err(RuntimeCatalogCompileError::CapabilityContext { service, operation, path })
            if service == "-" && operation == "-" && path == "Absent")
        );
        Ok(())
    })
}

#[test]
fn given_missing_identity_when_validated_then_identity_path_is_reported() -> TestResult {
    with_model(|service, operation, _| {
        for identity in [
            Identity::Arn(MemberPath::new("Absent")?),
            Identity::Key(vec![MemberPath::new("Absent")?]),
            Identity::Template("{Absent}".to_owned()),
        ] {
            let mut plan = overlay()?;
            plan.identity = Some(identity);
            assert_path(
                plan_validation::validate(service, operation, &plan),
                "Absent",
            );
        }
        Ok(())
    })
}

#[test]
fn given_wrong_binding_shape_when_validated_then_binding_path_is_reported() -> TestResult {
    with_model(|service, operation, _| {
        let mut plan = overlay()?;
        plan.bind.insert(
            MemberPath::new("StartName")?,
            crate::overlay::Binding::RegionList,
        );
        assert_path(
            plan_validation::validate(service, operation, &plan),
            "StartName",
        );
        Ok(())
    })
}

#[test]
fn given_missing_exclusion_when_validated_then_exclusion_path_is_reported() -> TestResult {
    with_model(|service, operation, _| {
        let mut plan = overlay()?;
        plan.exclude_from_hash.push(MemberPath::new("Absent")?);
        assert_path(
            plan_validation::validate(service, operation, &plan),
            "Absent",
        );
        Ok(())
    })
}

#[test]
fn given_invalid_transform_when_encoded_then_binding_path_is_reported() -> TestResult {
    with_model(|_, _, services| {
        let binding = crate::overlay::Binding::Transformed {
            binding: Box::new(crate::overlay::Binding::Region),
            transform: crate::overlay::BindingTransform::StripPrefix("prefix".to_owned()),
        };
        let path = MemberPath::new("StartName")?;
        assert_path(
            crate::runtime_catalog::plans::binding_value("example", &path, &binding, services),
            "StartName",
        );
        Ok(())
    })
}

#[test]
fn given_invalid_tokens_when_compiled_then_exact_token_path_is_reported() -> TestResult {
    for (pairs, expected) in [
        (vec![("StartName", "IsTruncated", false)], "IsTruncated"),
        (vec![("StartName", "Items", false)], "Items"),
    ] {
        let error = compile_fixture(MODEL, &pairs)
            .err()
            .ok_or("accepted invalid token")?;
        let error = error
            .downcast_ref::<RuntimeCatalogCompileError>()
            .ok_or("wrong error type")?;
        assert!(matches!(error, RuntimeCatalogCompileError::Path { path } if path == expected));
    }
    Ok(())
}
