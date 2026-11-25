use super::*;
use crate::kind_map::embedded;
use crate::runtime_catalog::ServiceIndex;
use std::path::Path;

#[test]
fn happy_path_reviewed_paths_compile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cfn = std::fs::read(root.join("catalog/cloudcontrol-universal.json"))
        .expect("cfn source exists");
    let kinds = embedded(&cfn).expect("kinds build");
    let binding = std::fs::read(root.join("catalog/aws-api.catalog"))
        .expect("catalog exists");
    let catalog = inventory_core::catalog::Catalog::borrow(&binding)
        .expect("catalog parses");
    let services = ServiceIndex::build(catalog).expect("services build");
    let checked = crate::overlay::check(&root, root.join(".local/aws-models").as_path())
        .expect("overlay checks");
    let context = ValidationContext {
        kinds: &kinds,
        checked: &checked,
        services: &services,
    };
    assert!(validate_reviewed_paths(&context).is_ok());
}

#[test]
fn vpc_relation_validates() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cfn = std::fs::read(root.join("catalog/cloudcontrol-universal.json"))
        .expect("cfn source exists");
    let kinds = embedded(&cfn).expect("kinds build");
    let binding = std::fs::read(root.join("catalog/aws-api.catalog"))
        .expect("catalog exists");
    let catalog = inventory_core::catalog::Catalog::borrow(&binding)
        .expect("catalog parses");
    let services = ServiceIndex::build(catalog).expect("services build");
    let checked = crate::overlay::check(&root, root.join(".local/aws-models").as_path())
        .expect("overlay checks");
    let context = ValidationContext {
        kinds: &kinds,
        checked: &checked,
        services: &services,
    };
    let result = validate_kind_origin_path(
        &context,
        "AWS::EC2::Subnet",
        "VpcId",
        Origin::SelectedOutput,
        Terminal::Text,
    );
    assert!(result.is_ok(), "VpcId relation should validate");
}

#[test]
fn relation_source_kind_with_no_operation_row_names_kind() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cfn = std::fs::read(root.join("catalog/cloudcontrol-universal.json"))
        .expect("cfn source exists");
    let kinds = embedded(&cfn).expect("kinds build");
    let binding = std::fs::read(root.join("catalog/aws-api.catalog"))
        .expect("catalog exists");
    let catalog = inventory_core::catalog::Catalog::borrow(&binding)
        .expect("catalog parses");
    let services = ServiceIndex::build(catalog).expect("services build");
    let checked = crate::overlay::check(&root, root.join(".local/aws-models").as_path())
        .expect("overlay checks");
    let context = ValidationContext {
        kinds: &kinds,
        checked: &checked,
        services: &services,
    };
    let result = validate_kind_path(&context, "AWS::EC2::NetworkInterface", "MacAddress");
    assert!(matches!(result, Err(RuntimeCatalogCompileError::KindContext { kind, path, reason }) if kind == "AWS::EC2::NetworkInterface" && path == "MacAddress" && reason == "no operation row produces this kind"));
}

#[test]
fn relation_source_kind_with_deferred_operation_names_operation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cfn = std::fs::read(root.join("catalog/cloudcontrol-universal.json"))
        .expect("cfn source exists");
    let kinds = embedded(&cfn).expect("kinds build");
    let binding = std::fs::read(root.join("catalog/aws-api.catalog"))
        .expect("catalog exists");
    let catalog = inventory_core::catalog::Catalog::borrow(&binding)
        .expect("catalog parses");
    let services = ServiceIndex::build(catalog).expect("services build");
    let checked = crate::overlay::check(&root, root.join(".local/aws-models").as_path())
        .expect("overlay checks");
    let context = ValidationContext {
        kinds: &kinds,
        checked: &checked,
        services: &services,
    };
    let result = validate_kind_path(&context, "AWS::EC2::SecurityGroup", "VpcId");
    assert!(matches!(result, Err(RuntimeCatalogCompileError::CapabilityContext { service, operation, path }) if service == "ec2" && operation == "DescribeSecurityGroups" && path == "VpcId"));
}

#[test]
fn relation_source_path_with_typo_names_path() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cfn = std::fs::read(root.join("catalog/cloudcontrol-universal.json"))
        .expect("cfn source exists");
    let kinds = embedded(&cfn).expect("kinds build");
    let binding = std::fs::read(root.join("catalog/aws-api.catalog"))
        .expect("catalog exists");
    let catalog = inventory_core::catalog::Catalog::borrow(&binding)
        .expect("catalog parses");
    let services = ServiceIndex::build(catalog).expect("services build");
    let checked = crate::overlay::check(&root, root.join(".local/aws-models").as_path())
        .expect("overlay checks");
    let context = ValidationContext {
        kinds: &kinds,
        checked: &checked,
        services: &services,
    };
    let result = validate_kind_path(&context, "AWS::EC2::Subnet", "VpcID");
    assert!(matches!(result, Err(RuntimeCatalogCompileError::CapabilityContext { service, operation, path }) if service == "ec2" && operation == "DescribeSubnets" && path == "VpcID"));
}

#[test]
fn relation_target_path_with_non_string_type_names_path() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cfn = std::fs::read(root.join("catalog/cloudcontrol-universal.json"))
        .expect("cfn source exists");
    let kinds = embedded(&cfn).expect("kinds build");
    let binding = std::fs::read(root.join("catalog/aws-api.catalog"))
        .expect("catalog exists");
    let catalog = inventory_core::catalog::Catalog::borrow(&binding)
        .expect("catalog parses");
    let services = ServiceIndex::build(catalog).expect("services build");
    let checked = crate::overlay::check(&root, root.join(".local/aws-models").as_path())
        .expect("overlay checks");
    let context = ValidationContext {
        kinds: &kinds,
        checked: &checked,
        services: &services,
    };
    let result = validate_kind_path(&context, "AWS::EC2::VPC", "IsDefault");
    assert!(matches!(result, Err(RuntimeCatalogCompileError::KindContext { kind, path, reason }) if kind == "AWS::EC2::VPC" && path == "IsDefault" && reason == "path does not end in a string"));
}