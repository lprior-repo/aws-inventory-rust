use std::{collections::BTreeSet, error::Error};

use inventory_core::{inventory::Region, record::{Key, Value}};
use inventory_harness::inventory_seed::{
    FieldPath, ScenarioSeedOracle, SeedError, SeedIdentity, SeedIdentityEvidence, SeedInput,
    SeedInputValue, SeedOutputCheck, SeedResourceIdentity, SeedScenario, SeedScope, SeedStep, StepId,
};

// Given an owned LOCAL emulator in the commercial partition, When the shared
// runner creates metadata-only resources, Then independent assertions below
// check canonical identity, provenance, scope, cardinality and absence of edges.
// No objects/items, key material, database credentials or parameter values.

pub(super) fn s3_scenario() -> Result<SeedScenario, SeedError> {
    // CreateBucket without LocationConstraint requires the us-east-1 endpoint.
    // Location is not a bucket ARN; identity is explicitly request-derived.
    SeedScenario::new([
        (0, "inventory-seed-empty-bucket-a"),
        (1, "inventory-seed-empty-bucket-b"),
        (2, "inventory-seed-empty-bucket-c"),
    ].into_iter().map(|(id, name)| {
        Ok(SeedStep::new(
            StepId::new(id), "s3".into(), "CreateBucket".into(), "AWS::S3::Bucket".into(),
            SeedIdentity::Request {
                input_field: FieldPath::new(&["Bucket"])?,
                template: Some("arn:{partition}:s3:::{value}".into()),
            },
            vec![text("Bucket", name)?],
        ).with_scope(SeedScope::Account))
    }).collect::<Result<Vec<_>, SeedError>>()?)
}

pub(super) fn kms_scenario() -> Result<SeedScenario, SeedError> {
    // EXTERNAL creates no material. Never substitute the AWS_KMS default.
    // Three keys/aliases exceed the reviewed ListKeys/ListAliases page size 1.
    let keys = [(0, false), (1, false), (2, true)].into_iter()
        .map(|(id, multi_region)| key_step(id, multi_region));
    let aliases = [
        (3, 0, "alias/inventory-seed-external-key-a"),
        (4, 1, "alias/inventory-seed-external-key-b"),
        (5, 2, "alias/inventory-seed-external-multi-region-primary"),
    ].into_iter().map(|(id, key, name)| alias_step(id, key, name));
    // CreateAlias returns Unit: it cannot confirm an alias-to-key edge.
    SeedScenario::new(keys.chain(aliases).collect::<Result<Vec<_>, _>>()?)
}

fn key_step(id: u8, multi_region: bool) -> Result<SeedStep, SeedError> {
    let description = if multi_region {
        "Inventory seed external multi-region primary without material"
    } else {
        "Inventory seed external key without material"
    };
    Ok(SeedStep::new(
        StepId::new(id), "kms".into(), "CreateKey".into(), "AWS::KMS::Key".into(),
        FieldPath::new(&["KeyMetadata", "Arn"])?,
        vec![
            text("Origin", "EXTERNAL")?, text("KeySpec", "SYMMETRIC_DEFAULT")?,
            text("KeyUsage", "ENCRYPT_DECRYPT")?,
            SeedInput::new(FieldPath::new(&["MultiRegion"])?, SeedInputValue::Boolean(multi_region)),
            text("Description", description)?,
        ],
    ).with_response_checks(checks("KeyMetadata", &[
        "Origin", "KeySpec", "KeyUsage", "MultiRegion", "Description",
    ])?))
}

fn alias_step(id: u8, key: u8, name: &str) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(id), "kms".into(), "CreateAlias".into(), "AWS::KMS::Alias".into(),
        SeedIdentity::Request {
            input_field: FieldPath::new(&["AliasName"])?,
            template: Some("arn:{partition}:kms:{region}:{account}:{value}".into()),
        },
        vec![text("AliasName", name)?, SeedInput::new(
            FieldPath::new(&["TargetKeyId"])?,
            SeedInputValue::PreviousOutput {
                step: StepId::new(key), path: FieldPath::new(&["KeyMetadata", "Arn"])?,
            },
        )],
    ))
}

pub(super) fn dynamodb_scenario() -> Result<SeedScenario, SeedError> {
    SeedScenario::new([
        (0, "inventory-seed-string-key", "S", false),
        (1, "inventory-seed-number-key", "N", false),
        (2, "inventory-seed-binary-key", "B", false),
        (3, "inventory-seed-composite-key", "S", true),
    ].into_iter().map(|(id, name, attribute_type, composite)| {
        table_step(id, name, attribute_type, composite)
    }).collect::<Result<Vec<_>, _>>()?)
}

fn table_step(id: u8, name: &str, attribute_type: &str, composite: bool)
    -> Result<SeedStep, SeedError>
{
    let attributes = std::iter::once(schema("partition_key", "AttributeType", attribute_type))
        .chain(composite.then(|| schema("sort_key", "AttributeType", "N")));
    let key_schema = std::iter::once(schema("partition_key", "KeyType", "HASH"))
        .chain(composite.then(|| schema("sort_key", "KeyType", "RANGE")));
    Ok(SeedStep::new(
        StepId::new(id), "dynamodb".into(), "CreateTable".into(), "AWS::DynamoDB::Table".into(),
        FieldPath::new(&["TableDescription", "TableArn"])?,
        vec![
            text("TableName", name)?, literal("AttributeDefinitions", Value::List(attributes.collect()))?,
            literal("KeySchema", Value::List(key_schema.collect()))?, text("BillingMode", "PAY_PER_REQUEST")?,
        ],
    ).with_response_checks(checks("TableDescription", &[
        "TableName", "AttributeDefinitions", "KeySchema",
    ])?))
}

fn schema(name: &str, field: &str, value: &str) -> Value {
    Value::Map([
        (Key::Name("AttributeName".into()), Value::Text(name.into())),
        (Key::Name(field.into()), Value::Text(value.into())),
    ].into_iter().collect())
}

pub(super) fn rds_scenario() -> Result<SeedScenario, SeedError> {
    // Parameter-group envelopes only: no instances, clusters, managed secrets,
    // credentials or configured parameter values. Creation outputs have no edge.
    SeedScenario::new([
        parameter_group(0, "inventory-seed-db-parameters-a", "DBParameterGroup", "mysql8.0")?,
        parameter_group(1, "inventory-seed-db-parameters-b", "DBParameterGroup", "mysql8.0")?,
        parameter_group(2, "inventory-seed-cluster-parameters-a", "DBClusterParameterGroup", "aurora-mysql8.0")?,
        parameter_group(3, "inventory-seed-cluster-parameters-b", "DBClusterParameterGroup", "aurora-mysql8.0")?,
    ].into())
}

fn parameter_group(id: u8, name: &str, root: &str, family: &str) -> Result<SeedStep, SeedError> {
    let name_field = format!("{root}Name");
    let arn_field = format!("{root}Arn");
    Ok(SeedStep::new(
        StepId::new(id), "rds".into(), format!("Create{root}"), format!("AWS::RDS::{root}"),
        FieldPath::new(&[root, &arn_field])?,
        vec![text(&name_field, name)?, text("DBParameterGroupFamily", family)?,
             text("Description", "Inventory seed parameter group")?],
    ).with_response_checks(checks(root, &[&name_field, "DBParameterGroupFamily", "Description"])?))
}

pub(super) fn assert_s3_oracle(oracle: &ScenarioSeedOracle, account: &str, region: &str)
    -> Result<(), Box<dyn Error>>
{
    assert_eq!(region, "us-east-1", "CreateBucket fixture has no LocationConstraint");
    assert_context(oracle, account, region, 3);
    [
        (0, "arn:aws:s3:::inventory-seed-empty-bucket-a"),
        (1, "arn:aws:s3:::inventory-seed-empty-bucket-b"),
        (2, "arn:aws:s3:::inventory-seed-empty-bucket-c"),
    ].into_iter().try_for_each(|(id, expected)| {
        let resource = resource(oracle, id)?;
        assert_resource(resource, "AWS::S3::Bucket", account, &Region::Global, SeedIdentityEvidence::Request);
        assert_eq!(resource.id.as_str(), expected);
        Ok(())
    })
}

pub(super) fn assert_dynamodb_oracle(oracle: &ScenarioSeedOracle, account: &str, region: &str)
    -> Result<(), Box<dyn Error>>
{
    assert_context(oracle, account, region, 4);
    [
        (0, "table/inventory-seed-string-key"),
        (1, "table/inventory-seed-number-key"),
        (2, "table/inventory-seed-binary-key"),
        (3, "table/inventory-seed-composite-key"),
    ].into_iter().try_for_each(|(id, suffix)| {
        let resource = resource(oracle, id)?;
        assert_resource(resource, "AWS::DynamoDB::Table", account, &oracle.region, SeedIdentityEvidence::Response);
        assert_eq!(resource.id.as_str(), format!("arn:aws:dynamodb:{region}:{account}:{suffix}"));
        Ok(())
    })
}

pub(super) fn assert_rds_oracle(oracle: &ScenarioSeedOracle, account: &str, region: &str)
    -> Result<(), Box<dyn Error>>
{
    assert_context(oracle, account, region, 4);
    [
        (0, "AWS::RDS::DBParameterGroup", "pg:inventory-seed-db-parameters-a"),
        (1, "AWS::RDS::DBParameterGroup", "pg:inventory-seed-db-parameters-b"),
        (2, "AWS::RDS::DBClusterParameterGroup", "cluster-pg:inventory-seed-cluster-parameters-a"),
        (3, "AWS::RDS::DBClusterParameterGroup", "cluster-pg:inventory-seed-cluster-parameters-b"),
    ].into_iter().try_for_each(|(id, kind, suffix)| {
        let resource = resource(oracle, id)?;
        assert_resource(resource, kind, account, &oracle.region, SeedIdentityEvidence::Response);
        assert_eq!(resource.id.as_str(), format!("arn:aws:rds:{region}:{account}:{suffix}"));
        Ok(())
    })
}

pub(super) fn assert_kms_oracle(oracle: &ScenarioSeedOracle, account: &str, region: &str)
    -> Result<(), Box<dyn Error>>
{
    assert_context(oracle, account, region, 6);
    // KMS generates key IDs: a literal expected UUID would fabricate evidence.
    [(0, false), (1, false), (2, true)].into_iter().try_for_each(|(id, multi_region)| {
        let key = resource(oracle, id)?;
        assert_resource(key, "AWS::KMS::Key", account, &oracle.region, SeedIdentityEvidence::Response);
        let prefix = format!("arn:aws:kms:{region}:{account}:key/");
        let generated = key.id.as_str().strip_prefix(&prefix).ok_or("noncanonical key ARN")?;
        assert!(valid_key_id(generated, multi_region), "unexpected KMS key ID {generated}");
        Ok::<(), Box<dyn Error>>(())
    })?;
    [
        (3, "alias/inventory-seed-external-key-a"),
        (4, "alias/inventory-seed-external-key-b"),
        (5, "alias/inventory-seed-external-multi-region-primary"),
    ].into_iter().try_for_each(|(id, suffix)| {
        let alias = resource(oracle, id)?;
        assert_resource(alias, "AWS::KMS::Alias", account, &oracle.region, SeedIdentityEvidence::Request);
        assert_eq!(alias.id.as_str(), format!("arn:aws:kms:{region}:{account}:{suffix}"));
        Ok(())
    })
}

fn valid_key_id(id: &str, multi_region: bool) -> bool {
    if multi_region {
        return id.strip_prefix("mrk-").is_some_and(|hex| hex.len() == 32 && hex.bytes().all(|b| b.is_ascii_hexdigit()));
    }
    let groups: Vec<_> = id.split('-').collect();
    groups.len() == 5 && groups.iter().zip([8, 4, 4, 4, 12])
        .all(|(group, length)| group.len() == length && group.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn assert_context(oracle: &ScenarioSeedOracle, account: &str, region: &str, count: usize) {
    assert_eq!(oracle.account_id.as_str(), account);
    assert_eq!(oracle.region.as_str(), region);
    assert_eq!(oracle.resources.len(), count);
    assert_eq!(oracle.resources.iter().map(|r| r.step).collect::<BTreeSet<_>>().len(), count);
    assert_eq!(oracle.resources.iter().map(|r| r.id.as_str()).collect::<BTreeSet<_>>().len(), count);
    assert!(oracle.edges.is_empty(), "creation outputs do not establish resource edges");
}

fn assert_resource(resource: &SeedResourceIdentity, kind: &str, account: &str,
    region: &Region, evidence: SeedIdentityEvidence)
{
    assert_eq!(resource.kind.as_str(), kind);
    assert_eq!(resource.key.kind, resource.kind);
    assert_eq!(resource.key.account.as_str(), account);
    assert_eq!(&resource.key.region, region);
    assert_eq!(resource.key.id, resource.id);
    assert_eq!(resource.evidence, evidence);
}

fn resource(oracle: &ScenarioSeedOracle, id: u8) -> Result<&SeedResourceIdentity, Box<dyn Error>> {
    oracle.resources.iter().find(|r| r.step == StepId::new(id)).ok_or_else(|| format!("missing seed step {id}").into())
}

fn text(field: &str, value: &str) -> Result<SeedInput, SeedError> {
    Ok(SeedInput::new(FieldPath::new(&[field])?, SeedInputValue::Text(value.into())))
}

fn literal(field: &str, value: Value) -> Result<SeedInput, SeedError> {
    Ok(SeedInput::new(FieldPath::new(&[field])?, SeedInputValue::Literal(value)))
}

fn checks(root: &str, fields: &[&str]) -> Result<Vec<SeedOutputCheck>, SeedError> {
    fields.iter().map(|field| Ok(SeedOutputCheck::new(
        FieldPath::new(&[root, field])?, FieldPath::new(&[field])?,
    ))).collect()
}
