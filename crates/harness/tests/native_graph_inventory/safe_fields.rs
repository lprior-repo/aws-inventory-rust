use super::TestResult;
use cmdb_view::StoredResource;
use inventory_core::{
    catalog::{Catalog, runtime::RuntimeCatalog},
    inventory::ResourceKey,
    record::{KeyRef, Value, ValueRef},
};
use inventory_harness::inventory_seed::{
    ScenarioSeedOracle, SeedInputValue, SeedScenario, SeedStep,
};
use std::collections::BTreeMap;

pub(super) type Expectations = BTreeMap<ResourceKey, BTreeMap<String, Value>>;

pub(super) fn from_fixture(
    scenario: &SeedScenario,
    oracle: &ScenarioSeedOracle,
) -> TestResult<Expectations> {
    oracle
        .resources
        .iter()
        .map(|resource| {
            let step = scenario
                .steps()
                .iter()
                .find(|step| step.id == resource.step)
                .ok_or("seed oracle resource lacks its independent fixture step")?;
            let fields = fields(step, &resource.key)?;
            assert!(
                !fields.is_empty(),
                "every fixture kind needs independent safe record evidence"
            );
            Ok((resource.key.clone(), fields))
        })
        .collect()
}

fn fields(step: &SeedStep, key: &ResourceKey) -> TestResult<BTreeMap<String, Value>> {
    let mut expected = BTreeMap::new();
    // Only metadata actually supplied by the fixture or its separately checked
    // identity receipt is asserted. Provider timestamps/opaque values are not
    // invented; list-only resources deliberately expose fewer safe fields.
    let inputs: &[(&str, &str)] = match key.kind.as_str() {
        "AWS::EC2::VPC" | "AWS::EC2::Subnet" => &[("CidrBlock", "CidrBlock")],
        "AWS::EC2::SecurityGroup" => &[("GroupName", "GroupName"), ("Description", "Description")],
        "AWS::EC2::RouteTable" => {
            expected.insert("RouteTableId".into(), Value::Text(key.id.as_str().into()));
            &[]
        }
        "AWS::ElasticLoadBalancingV2::LoadBalancer" => &[
            ("Name", "LoadBalancerName"),
            ("Scheme", "Scheme"),
            ("Type", "Type"),
        ],
        "AWS::ElasticLoadBalancingV2::TargetGroup" => &[
            ("Name", "TargetGroupName"),
            ("Protocol", "Protocol"),
            ("Port", "Port"),
            ("TargetType", "TargetType"),
        ],
        "AWS::CloudWatch::Alarm" => &[
            ("AlarmName", "AlarmName"),
            ("AlarmDescription", "AlarmDescription"),
            ("ActionsEnabled", "ActionsEnabled"),
            ("Metrics", "Metrics"),
            ("EvaluationPeriods", "EvaluationPeriods"),
            ("Threshold", "Threshold"),
            ("ComparisonOperator", "ComparisonOperator"),
            ("TreatMissingData", "TreatMissingData"),
        ],
        "AWS::CloudWatch::CompositeAlarm" => &[
            ("AlarmName", "AlarmName"),
            ("AlarmDescription", "AlarmDescription"),
            ("ActionsEnabled", "ActionsEnabled"),
            ("AlarmRule", "AlarmRule"),
        ],
        "AWS::Signer::SigningProfile" => {
            &[("profileName", "profileName"), ("platformId", "platformId")]
        }
        "AWS::Lambda::CodeSigningConfig" => &[
            ("Description", "Description"),
            (
                "CodeSigningPolicies.UntrustedArtifactOnDeployment",
                "CodeSigningPolicies.UntrustedArtifactOnDeployment",
            ),
        ],
        "AWS::EKS::Cluster" => &[("name", "$")],
        "AWS::ECS::Cluster" | "AWS::ECS::Service" => {
            expected.insert("$".into(), Value::Text(key.id.as_str().into()));
            &[]
        }
        "AWS::IAM::Role" => &[("RoleName", "RoleName")],
        "AWS::IAM::Group" => &[("GroupName", "GroupName"), ("Path", "Path")],
        "AWS::S3::Bucket" => &[("Bucket", "Name")],
        "AWS::KMS::Key" => {
            expected.insert("KeyArn".into(), Value::Text(key.id.as_str().into()));
            expected.insert(
                "KeyId".into(),
                Value::Text(
                    key.id
                        .as_str()
                        .rsplit('/')
                        .next()
                        .ok_or("KMS key ARN has no ID")?
                        .into(),
                ),
            );
            &[]
        }
        "AWS::KMS::Alias" => &[("AliasName", "AliasName")],
        "AWS::DynamoDB::Table" => &[
            ("TableName", "TableName"),
            ("AttributeDefinitions", "AttributeDefinitions"),
            ("KeySchema", "KeySchema"),
        ],
        "AWS::RDS::DBParameterGroup" => &[
            ("DBParameterGroupName", "DBParameterGroupName"),
            ("DBParameterGroupFamily", "DBParameterGroupFamily"),
            ("Description", "Description"),
        ],
        "AWS::RDS::DBClusterParameterGroup" => &[
            ("DBClusterParameterGroupName", "DBClusterParameterGroupName"),
            ("DBParameterGroupFamily", "DBParameterGroupFamily"),
            ("Description", "Description"),
        ],
        "AWS::SQS::Queue" => {
            expected.insert(
                "Attributes.QueueArn".into(),
                Value::Text(key.id.as_str().into()),
            );
            &[]
        }
        "AWS::SNS::Topic" => {
            expected.insert("TopicArn".into(), Value::Text(key.id.as_str().into()));
            &[]
        }
        "AWS::SNS::Subscription" => {
            expected.insert(
                "Attributes.SubscriptionArn".into(),
                Value::Text(key.id.as_str().into()),
            );
            expected.insert("Attributes.Protocol".into(), Value::Text("sqs".into()));
            &[]
        }
        "AWS::StepFunctions::Activity" => &[("name", "name")],
        "AWS::StepFunctions::StateMachine" => &[("name", "name"), ("type", "type")],
        // The fixture reads the emulator's existing organization and root, so
        // it supplies no configured literal. Only the identity receipt, which
        // the oracle checks against account and partition, is asserted.
        "AWS::Organizations::Organization" | "AWS::ApiOnly::OrganizationsRoot" => {
            expected.insert("Arn".into(), Value::Text(key.id.as_str().into()));
            &[]
        }
        "AWS::Route53::HostedZone" => &[("Name", "Name"), ("CallerReference", "CallerReference")],
        "AWS::CloudFront::OriginAccessControl" => &[
            ("OriginAccessControlConfig.Name", "Name"),
            (
                "OriginAccessControlConfig.SigningProtocol",
                "SigningProtocol",
            ),
            (
                "OriginAccessControlConfig.SigningBehavior",
                "SigningBehavior",
            ),
            (
                "OriginAccessControlConfig.OriginAccessControlOriginType",
                "OriginAccessControlOriginType",
            ),
        ],
        other => return Err(format!("fixture has no independent safe fields for {other}").into()),
    };
    for (input, output) in inputs {
        let value = step
            .inputs
            .iter()
            .find(|value| {
                value
                    .field
                    .parts()
                    .iter()
                    .map(String::as_str)
                    .eq(input.split('.'))
            })
            .ok_or_else(|| format!("fixture {} lacks safe literal {input}", step.kind))?;
        let value = match &value.value {
            SeedInputValue::Text(value) => Value::Text(value.clone()),
            SeedInputValue::Boolean(value) => Value::Bool(*value),
            SeedInputValue::Integer(value) => Value::Int(*value),
            SeedInputValue::Literal(value) => value.clone(),
            other => {
                return Err(
                    format!("expected independent literal for {input}, got {other:?}").into(),
                );
            }
        };
        expected.insert((*output).into(), value);
    }
    Ok(expected)
}

pub(super) fn assert_detail(
    detail: &StoredResource,
    expected: &BTreeMap<String, Value>,
    runtime: &RuntimeCatalog<'_>,
) -> TestResult {
    let actual = named_value(ValueRef::Map(detail.record.root()), runtime.catalog())?;
    super::safe_shape::validate(detail.key.kind.as_str(), &actual)?;
    validate_expected(&actual, expected)?;
    if detail.key.kind.as_str() == "AWS::Lambda::CodeSigningConfig" {
        let Some(Value::List(arns)) =
            at_path(&actual, "AllowedPublishers.SigningProfileVersionArns")
        else {
            return Err(
                "configured Lambda allowed publishers missing from collected metadata".into(),
            );
        };
        let [Value::Text(arn)] = arns.as_slice() else {
            return Err("configured Lambda publisher cardinality/type differs".into());
        };
        let prefix = "arn:aws:signer:us-east-1:000000000000:/signing-profiles/inventory_seed_lambda_signing/";
        assert!(
            arn.starts_with(prefix) && arn.len() > prefix.len(),
            "generated signing-profile version must reference the independently seeded publisher"
        );
    }
    // Each fixture explicitly expects no display name under the frozen mapping;
    // do not let a runtime name-path change choose its own expected value.
    assert_eq!(
        detail.name, None,
        "independent absent display name: {:?}",
        detail.key
    );
    assert!(
        !runtime
            .name_paths()
            .iter()
            .any(|path| path.kind == detail.key.kind),
        "catalog naming metadata differs from the independent fixture contract"
    );
    let hashes = runtime
        .plans()
        .iter()
        .filter(|plan| plan.kind() == &detail.key.kind)
        .map(|plan| detail.record.content_hash(plan.hash_exclusions()))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(
        hashes.contains(&detail.content_hash),
        "public detail hash must match its canonical record and a reviewed exclusion policy: {:?}",
        detail.key
    );
    Ok(())
}

fn at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(value, |current, component| {
        let Value::Map(fields) = current else {
            return None;
        };
        fields.iter().find_map(|(key, value)| match key {
            inventory_core::record::Key::Name(name) if name == component => Some(value),
            _ => None,
        })
    })
}

fn named_value(value: ValueRef<'_>, catalog: Catalog<'_>) -> TestResult<Value> {
    Ok(match value {
        ValueRef::Map(map) => Value::Map(
            map.iter()?
                .map(|entry| {
                    let (key, value) = entry?;
                    let name = match key {
                        KeyRef::Field(id) => catalog.field_name(id)?,
                        KeyRef::Name(name) => name,
                    };
                    Ok((
                        inventory_core::record::Key::Name(name.into()),
                        named_value(value, catalog)?,
                    ))
                })
                .collect::<TestResult<BTreeMap<_, _>>>()?,
        ),
        ValueRef::List(list) => Value::List(
            list.iter()?
                .map(|value| named_value(value?, catalog))
                .collect::<TestResult<Vec<_>>>()?,
        ),
        value => value.to_value()?,
    })
}

fn validate_expected(actual: &Value, expected: &BTreeMap<String, Value>) -> TestResult {
    for (path, value) in expected {
        if at_path(actual, path) != Some(value) {
            return Err(format!("independent configured safe member {path} differs: actual={:?}, expected={value:?}", at_path(actual, path)).into());
        }
    }
    Ok(())
}

pub(super) fn assert_oracle_rejects_mutations() -> TestResult {
    use inventory_core::record::Key;
    let expected = BTreeMap::from([("CidrBlock".into(), Value::Text("10.89.0.0/16".into()))]);
    let original = Value::Map(BTreeMap::from([(
        Key::Name("CidrBlock".into()),
        Value::Text("10.89.0.0/16".into()),
    )]));
    super::safe_shape::validate("AWS::EC2::VPC", &original)?;
    validate_expected(&original, &expected)?;
    // Given valid independently specified metadata, bypassing safe filtering
    // with an extra secret or dropping a configured field must fail the oracle.
    for field in ["SecretAccessKey", "UnreviewedProviderMember"] {
        let Value::Map(mut leaked) = original.clone() else {
            return Err("mutation fixture is not a map".into());
        };
        leaked.insert(
            Key::Name(field.into()),
            Value::Text("unsafe-sentinel".into()),
        );
        assert!(super::safe_shape::validate("AWS::EC2::VPC", &Value::Map(leaked)).is_err());
    }
    assert!(validate_expected(&Value::Map(BTreeMap::new()), &expected).is_err());
    let nested = Value::Map(BTreeMap::from([(
        Key::Name("State".into()),
        Value::Map(BTreeMap::from([
            (Key::Name("Code".into()), Value::Text("active".into())),
            (
                Key::Name("UnsafeSentinel".into()),
                Value::Text("must-not-land".into()),
            ),
        ])),
    )]));
    assert!(
        super::safe_shape::validate("AWS::ElasticLoadBalancingV2::LoadBalancer", &nested).is_err()
    );
    Ok(())
}
