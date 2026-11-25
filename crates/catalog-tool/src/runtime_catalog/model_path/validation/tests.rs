use super::*;
use crate::kind_map::embedded;
use crate::runtime_catalog::ServiceIndex;
use std::path::Path;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn with_context(
    callable: bool,
    check: impl FnOnce(&ValidationContext<'_, '_>) -> TestResult,
) -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let kinds = embedded(&std::fs::read(
        root.join("catalog/cloudcontrol-universal.json"),
    )?)?;
    let bytes = std::fs::read(root.join("catalog/aws-api.catalog"))?;
    let catalog = inventory_core::catalog::Catalog::borrow(&bytes)?;
    let services = ServiceIndex::build(catalog)?;
    let source = root.join(".local/aws-models");
    let checked = if callable {
        crate::overlay::check(&root, &source)?
    } else {
        crate::overlay::check_policies(&source, &[], &crate::classes::DenyList::builtin())?
    };
    check(&ValidationContext {
        kinds: &kinds,
        checked: &checked,
        services: &services,
    })
}

#[test]
fn given_callable_emissions_when_reviewed_paths_validate_then_live_relations_pass() -> TestResult {
    with_context(true, |context| {
        validate_reviewed_paths(context)?;
        validate_kind_path(context, "AWS::EC2::Subnet", "VpcId")?;
        Ok(())
    })
}

#[test]
fn given_missing_kind_when_path_validates_then_kind_and_path_are_reported() -> TestResult {
    with_context(false, |context| {
        let result = validate_kind_path(context, "AWS::ApiOnly::Absent", "MacAddress");
        assert!(
            matches!(result, Err(RuntimeCatalogCompileError::KindContext { kind, path, reason })
            if kind == "AWS::ApiOnly::Absent" && path == "MacAddress" && reason == "no operation row produces this kind")
        );
        Ok(())
    })
}

#[test]
fn given_deferred_operation_when_path_validates_then_operation_and_path_are_reported() -> TestResult
{
    with_context(false, |context| {
        let result = validate_kind_path(context, "AWS::EC2::SecurityGroup", "VpcId");
        assert!(
            matches!(result, Err(RuntimeCatalogCompileError::CapabilityContext { service, operation, path })
            if service == "ec2" && operation == "DescribeSecurityGroups" && path == "VpcId")
        );
        Ok(())
    })
}

#[test]
fn given_typo_when_path_validates_then_operation_and_path_are_reported() -> TestResult {
    with_context(true, |context| {
        let result = validate_kind_path(context, "AWS::EC2::Subnet", "VpcID");
        assert!(
            matches!(result, Err(RuntimeCatalogCompileError::CapabilityContext { service, operation, path })
            if service == "ec2" && operation == "DescribeSubnets" && path == "VpcID")
        );
        Ok(())
    })
}

#[test]
fn given_nontext_target_when_path_validates_then_kind_and_reason_are_reported() -> TestResult {
    with_context(true, |context| {
        let result = validate_kind_path(context, "AWS::EC2::VPC", "IsDefault");
        assert!(
            matches!(result, Err(RuntimeCatalogCompileError::KindContext { kind, path, reason })
            if kind == "AWS::EC2::VPC" && path == "IsDefault" && reason == "path does not end in a string")
        );
        Ok(())
    })
}
