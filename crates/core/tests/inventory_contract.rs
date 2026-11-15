use inventory_core::inventory::{
    AccountId, Column, InventoryError, Kind, KindSchema, OperationId, PageSize, Region, ResourceId,
    ResourceKey, ScalarType, ScopeKey, ServiceId, SqlName,
};
use inventory_core::record::FieldId;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn resource(account: &str, kind: &str, id: &str) -> Result<ResourceKey, InventoryError> {
    Ok(ResourceKey {
        account: AccountId::parse(account)?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse(kind)?,
        id: ResourceId::parse(id)?,
    })
}

#[test]
fn graph_keys_distinguish_accounts_and_tuple_boundaries() -> TestResult {
    let first = resource("123456789012", "AWS::EC2::VPC", "x:y")?;
    let boundary = resource("123456789012", "AWS::EC2::VPCx", ":y")?;
    let account = resource("210987654321", "AWS::EC2::VPC", "x:y")?;
    assert_ne!(first.digest(), boundary.digest());
    assert_ne!(first.digest(), account.digest());
    assert!(matches!(
        ResourceId::parse("vpc\0other"),
        Err(InventoryError::Resource)
    ));
    Ok(())
}

#[test]
fn child_scope_cannot_cross_account_authorization_boundaries() -> TestResult {
    let parent = resource("123456789012", "AWS::EC2::VPC", "vpc-1")?;
    let scope = ScopeKey::new(
        AccountId::parse("210987654321")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("ec2")?,
        OperationId::parse("DescribeSubnets")?,
        Some(parent),
    );
    assert!(matches!(scope, Err(InventoryError::ParentAccount)));
    Ok(())
}

#[test]
fn materialized_paths_reject_conflicting_scalar_types() -> TestResult {
    let field = FieldId::new(1)?;
    let columns = vec![
        Column::new(vec![field], ScalarType::Integer)?,
        Column::new(vec![field], ScalarType::Text)?,
    ];
    let schema = KindSchema::new(
        Kind::parse("AWS::EC2::VPC")?,
        SqlName::parse("aws_ec2_vpc")?,
        columns,
    );
    assert!(matches!(schema, Err(InventoryError::DuplicateColumn)));
    assert!(matches!(
        SqlName::parse("x\"; DROP TABLE resources;--"),
        Err(InventoryError::SqlName)
    ));
    Ok(())
}

#[test]
fn schema_budget_includes_timestamp_and_nonfinite_float_columns() -> TestResult {
    for scalar in [ScalarType::Timestamp, ScalarType::Real] {
        let columns = (1..=997)
            .map(|number| Column::new(vec![FieldId::new(number)?], scalar))
            .collect::<Result<Vec<_>, InventoryError>>()?;
        let schema = KindSchema::new(
            Kind::parse("AWS::EC2::VPC")?,
            SqlName::parse("aws_ec2_vpc")?,
            columns,
        );
        assert!(matches!(schema, Err(InventoryError::Columns)));
    }
    Ok(())
}

#[test]
fn resource_queries_cannot_request_zero_or_unbounded_pages() {
    assert!(matches!(PageSize::new(0), Err(InventoryError::PageSize)));
    assert!(matches!(PageSize::new(1001), Err(InventoryError::PageSize)));
}
