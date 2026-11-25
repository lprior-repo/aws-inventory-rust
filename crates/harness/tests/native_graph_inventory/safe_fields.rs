use super::{
    TestResult,
    safe_shape::{self, Violations},
};
use cmdb_view::StoredResource;
use inventory_core::{
    catalog::{Catalog, runtime::RuntimeCatalog},
    inventory::ResourceKey,
    record::{Key, KeyRef, Value, ValueRef},
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

type Inputs = &'static [(&'static str, &'static str)];

fn fields(step: &SeedStep, key: &ResourceKey) -> TestResult<BTreeMap<String, Value>> {
    let kind = key.kind.as_str();
    let mut expected = match identity_fields(key)? {
        Some(fields) => return Ok(fields),
        None => BTreeMap::new(),
    };
    let inputs = network_inputs(kind)
        .or_else(|| compute_and_storage_inputs(kind))
        .or_else(|| messaging_and_global_inputs(kind))
        .ok_or_else(|| format!("fixture has no independent safe fields for {kind}"))?;
    inputs
        .iter()
        .try_for_each(|&(input, output)| -> TestResult {
            expected.insert(output.into(), configured_value(step, input)?);
            Ok(())
        })?;
    if kind == "AWS::IAM::Role" {
        expected.insert("AssumeRolePolicyDocument".into(), role_trust(step)?);
    } else if kind == "AWS::CloudWatch::Alarm" {
        expected.insert("Metrics".into(), alarm_metrics(step)?);
    } else if kind == "AWS::StepFunctions::Activity" {
        expected.insert(
            "encryptionConfiguration".into(),
            named_map([("type", Value::Text("AWS_OWNED_KEY".into()))]),
        );
    }
    Ok(expected)
}

fn alarm_metrics(step: &SeedStep) -> TestResult<Value> {
    let mut metrics = configured_value(step, "Metrics")?;
    let Value::List(queries) = &mut metrics else {
        return Err("expected a list of metric queries in the seed input".into());
    };
    let [query] = queries.as_mut_slice() else {
        return Err("expected one CPU metric query in the seed input".into());
    };
    let mut value = query;
    for name in ["MetricStat", "Metric"] {
        let Value::Map(fields) = value else {
            return Err(format!("expected a seed map containing {name}").into());
        };
        value = fields
            .iter_mut()
            .find_map(|(key, item)| matches!(key, Key::Name(key) if key == name).then_some(item))
            .ok_or_else(|| format!("seed metric is missing {name}"))?;
    }
    let Value::Map(metric) = value else {
        return Err("expected a Metric map in the seed input".into());
    };
    assert!(
        metric
            .insert(Key::Name("Dimensions".into()), Value::List(Vec::new()))
            .is_none(),
        "DescribeAlarms emits an empty Dimensions list for the undimensioned seed metric"
    );
    Ok(metrics)
}

fn role_trust(step: &SeedStep) -> TestResult<Value> {
    let Value::Text(document) = configured_value(step, "AssumeRolePolicyDocument")? else {
        return Err("fixture IAM trust document must be text".into());
    };
    let policy: serde_json::Value = serde_json::from_str(&document)?;
    let Some([statement]) = policy
        .get("Statement")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
    else {
        return Err("fixture IAM trust must contain exactly one statement".into());
    };
    let statement = statement
        .as_object()
        .filter(|fields| fields.len() == 3)
        .ok_or("fixture IAM trust statement must contain Effect, Action and Principal")?;
    let (Some(effect), Some(action), Some(service)) = (
        statement.get("Effect").and_then(serde_json::Value::as_str),
        statement.get("Action").and_then(serde_json::Value::as_str),
        statement
            .get("Principal")
            .and_then(serde_json::Value::as_object)
            .filter(|principal| principal.len() == 1)
            .and_then(|principal| principal.get("Service"))
            .and_then(serde_json::Value::as_str),
    ) else {
        return Err(
            "fixture IAM trust requires literal Effect, Action and a Service principal".into(),
        );
    };
    let statement = named_map([
        ("effect", Value::Text(effect.into())),
        ("actions", Value::List(vec![Value::Text(action.into())])),
        (
            "principals",
            named_map([("Service", Value::List(vec![Value::Text(service.into())]))]),
        ),
        ("has_not_principal", Value::Bool(false)),
        ("has_not_action", Value::Bool(false)),
        ("condition_operators", Value::List(Vec::new())),
        ("condition_values_omitted", Value::Bool(false)),
    ]);
    Ok(named_map([
        ("trust_statements", Value::List(vec![statement])),
        ("trust_policy_values_omitted", Value::Bool(true)),
    ]))
}

fn named_map<const N: usize>(fields: [(&str, Value); N]) -> Value {
    Value::Map(
        fields
            .into_iter()
            .map(|(key, value)| (Key::Name(key.into()), value))
            .collect(),
    )
}

fn configured_value(step: &SeedStep, input: &str) -> TestResult<Value> {
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
    Ok(match &value.value {
        SeedInputValue::Text(value) => Value::Text(value.clone()),
        SeedInputValue::Boolean(value) => Value::Bool(*value),
        SeedInputValue::Integer(value) => Value::Int(*value),
        SeedInputValue::Literal(value) => value.clone(),
        other => {
            return Err(format!("expected independent literal for {input}, got {other:?}").into());
        }
    })
}

fn identity_fields(key: &ResourceKey) -> TestResult<Option<BTreeMap<String, Value>>> {
    let id = key.id.as_str();
    let fields: &[(&str, &str)] = match key.kind.as_str() {
        "AWS::EC2::RouteTable" => &[("RouteTableId", id)],
        "AWS::ECS::Cluster" | "AWS::ECS::Service" => &[("$", id)],
        "AWS::KMS::Key" => &[
            ("KeyArn", id),
            (
                "KeyId",
                id.rsplit('/').next().ok_or("KMS key ARN has no ID")?,
            ),
        ],
        "AWS::SQS::Queue" => &[("Attributes.QueueArn", id)],
        "AWS::SNS::Topic" => &[("TopicArn", id)],
        "AWS::SNS::Subscription" => &[
            ("Attributes.SubscriptionArn", id),
            ("Attributes.Protocol", "sqs"),
            (
                "Attributes.SubscriptionPrincipal",
                "arn:aws:iam::000000000000:user/emulator",
            ),
        ],
        "AWS::Organizations::Organization" => {
            let (_, organization_id) = id
                .rsplit_once(":organization/")
                .ok_or("organization ARN lacks an organization ID")?;
            &[
                ("Arn", id),
                ("Id", organization_id),
                ("MasterAccountId", key.account.as_str()),
                ("FeatureSet", "ALL"),
            ]
        }
        "AWS::ApiOnly::OrganizationsRoot" => {
            let (_, root_id) = id.rsplit_once('/').ok_or("root ARN lacks a root ID")?;
            &[("Arn", id), ("Id", root_id)]
        }
        _ => return Ok(None),
    };
    Ok(Some(
        fields
            .iter()
            .map(|&(name, value)| (name.into(), Value::Text(value.into())))
            .collect(),
    ))
}

fn network_inputs(kind: &str) -> Option<Inputs> {
    Some(match kind {
        "AWS::EC2::VPC" | "AWS::EC2::Subnet" => &[("CidrBlock", "CidrBlock")],
        "AWS::EC2::SecurityGroup" => &[("GroupName", "GroupName"), ("Description", "Description")],
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
        _ => return None,
    })
}

fn compute_and_storage_inputs(kind: &str) -> Option<Inputs> {
    Some(match kind {
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
        "AWS::IAM::Role" => &[("RoleName", "RoleName")],
        "AWS::IAM::Group" => &[("GroupName", "GroupName"), ("Path", "Path")],
        "AWS::S3::Bucket" => &[("Bucket", "Name")],
        "AWS::KMS::Alias" => &[("AliasName", "AliasName")],
        "AWS::DynamoDB::Table" => &[
            ("TableName", "$"),
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
        _ => return None,
    })
}

fn messaging_and_global_inputs(kind: &str) -> Option<Inputs> {
    Some(match kind {
        "AWS::StepFunctions::Activity" => &[("name", "name")],
        "AWS::StepFunctions::StateMachine" => &[("name", "name"), ("type", "type")],
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
        _ => return None,
    })
}

pub(super) fn inspect_detail(
    detail: &StoredResource,
    expected: &BTreeMap<String, Value>,
    runtime: &RuntimeCatalog<'_>,
    issues: &mut Violations,
) -> TestResult {
    let kind = detail.key.kind.as_str();
    let actual = named_value(ValueRef::Map(detail.record.root()), runtime.catalog())?;
    safe_shape::inspect(kind, &actual, issues);
    inspect_expected(kind, &actual, expected, issues);
    if kind == "AWS::Lambda::CodeSigningConfig" {
        let path = "AllowedPublishers.SigningProfileVersionArns";
        let publishers = at_path(&actual, path);
        let prefix = "arn:aws:signer:us-east-1:000000000000:/signing-profiles/inventory_seed_lambda_signing/";
        if !matches!(
            publishers,
            Some(Value::List(arns))
                if matches!(arns.as_slice(), [Value::Text(arn)]
                    if arn.starts_with(prefix) && arn.len() > prefix.len())
        ) {
            issues.add(
                kind,
                path,
                publishers,
                "expected one version of the independently seeded publisher",
            );
        }
    }
    if let Some(name) = &detail.name {
        issues.add(
            kind,
            "<display name>",
            Some(&Value::Text(name.to_string())),
            "expected absent display name",
        );
    }
    if runtime
        .name_paths()
        .iter()
        .any(|path| path.kind == detail.key.kind)
    {
        issues.add(
            kind,
            "<display name>",
            None,
            "catalog naming metadata differs from the independent fixture",
        );
    }
    inspect_hash(detail, runtime, issues)
}

fn inspect_hash(
    detail: &StoredResource,
    runtime: &RuntimeCatalog<'_>,
    issues: &mut Violations,
) -> TestResult {
    for emission in runtime
        .plans()
        .iter()
        .flat_map(|plan| plan.resource_emissions().iter())
        .filter(|emission| emission.kind() == &detail.key.kind)
    {
        if detail.record.content_hash(emission.hash_exclusions())? == detail.content_hash {
            return Ok(());
        }
    }
    issues.add(
        detail.key.kind.as_str(),
        "<content hash>",
        None,
        "no reviewed emission hashes to the stored digest",
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

fn inspect_expected(
    kind: &str,
    actual: &Value,
    expected: &BTreeMap<String, Value>,
    issues: &mut Violations,
) {
    for (path, value) in expected {
        compare_value(kind, path, at_path(actual, path), Some(value), issues);
    }
}

fn compare_value(
    kind: &str,
    path: &str,
    actual: Option<&Value>,
    expected: Option<&Value>,
    issues: &mut Violations,
) {
    match (actual, expected) {
        (Some(Value::Map(actual)), Some(Value::Map(expected))) => {
            for (key, value) in expected {
                compare_value(
                    kind,
                    &child_path(path, key),
                    actual.get(key),
                    Some(value),
                    issues,
                );
            }
            for (key, value) in actual {
                if !expected.contains_key(key) {
                    compare_value(kind, &child_path(path, key), Some(value), None, issues);
                }
            }
        }
        (Some(Value::List(actual)), Some(Value::List(expected))) => {
            for index in 0..actual.len().max(expected.len()) {
                compare_value(
                    kind,
                    &format!("{path}[{index}]"),
                    actual.get(index),
                    expected.get(index),
                    issues,
                );
            }
        }
        _ if actual == expected => {}
        _ => {
            let mut reason = match expected {
                Some(value) => format!(
                    "differs from configured {:?} value",
                    safe_shape::value_kind(value)
                ),
                None => "unexpected member in exact configured value".to_owned(),
            };
            if let Some(source) = fixture_value_source(kind, path) {
                reason.push_str("; ");
                reason.push_str(source);
            }
            issues.add(kind, path, actual, &reason);
        }
    }
}

fn fixture_value_source(kind: &str, path: &str) -> Option<&'static str> {
    match (kind, path) {
        ("AWS::CloudWatch::Alarm", path)
            if path.starts_with("Metrics[0].MetricStat.Metric.Dimensions") =>
        {
            Some("DescribeAlarms emits Dimensions=[] for the undimensioned metric")
        }
        ("AWS::SNS::Subscription", "Attributes.SubscriptionPrincipal") => {
            Some("GetSubscriptionAttributes returns the emulator STS caller ARN")
        }
        ("AWS::StepFunctions::Activity", path) if path.starts_with("encryptionConfiguration") => {
            Some("DescribeActivity emits type AWS_OWNED_KEY for the seeded activity")
        }
        _ => None,
    }
}

fn child_path(path: &str, key: &Key) -> String {
    match key {
        Key::Name(name) => safe_shape::member_path(path, name),
        Key::Field(field) => format!("{path}.<field {}>", field.value()),
    }
}

fn validate_fixture(kind: &str, actual: &Value, expected: &BTreeMap<String, Value>) -> TestResult {
    let mut issues = Violations::default();
    safe_shape::inspect(kind, actual, &mut issues);
    inspect_expected(kind, actual, expected, &mut issues);
    issues.finish()
}

pub(super) fn assert_oracle_rejects_mutations(records: &Expectations) -> TestResult {
    use inventory_core::record::Key;
    let expected = BTreeMap::from([("CidrBlock".into(), Value::Text("10.89.0.0/16".into()))]);
    let original = Value::Map(BTreeMap::from([(
        Key::Name("CidrBlock".into()),
        Value::Text("10.89.0.0/16".into()),
    )]));
    validate_fixture("AWS::EC2::VPC", &original, &expected)?;
    let mut issues = Violations::default();
    let leaked = named_map([
        ("CidrBlock", Value::Int(1)),
        ("SecretAccessKey", Value::Text("unsafe-sentinel".into())),
        (
            "UnreviewedProviderMember",
            Value::Text("unsafe-sentinel".into()),
        ),
    ]);
    safe_shape::inspect("AWS::EC2::VPC", &leaked, &mut issues);
    inspect_expected("AWS::EC2::VPC", &leaked, &expected, &mut issues);
    let nested = named_map([(
        "State",
        named_map([
            ("Code", Value::Text("active".into())),
            ("UnsafeSentinel", Value::Text("must-not-land".into())),
        ]),
    )]);
    safe_shape::inspect(
        "AWS::ElasticLoadBalancingV2::LoadBalancer",
        &nested,
        &mut issues,
    );
    let Err(error) = issues.finish() else {
        return Err("safe oracle accepted rejected fields in multiple resources".into());
    };
    let report = error.to_string();
    for issue in [
        "AWS::EC2::VPC | CidrBlock | Int | differs from configured Text value",
        "AWS::EC2::VPC | SecretAccessKey | Text | forbidden safe member",
        "AWS::EC2::VPC | UnreviewedProviderMember | Text | unreviewed safe member",
        "AWS::ElasticLoadBalancingV2::LoadBalancer | State.UnsafeSentinel | Text | forbidden safe member",
    ] {
        assert!(
            report.contains(issue),
            "missing mutation diagnostic: {issue}"
        );
    }
    assert!(!report.contains("unsafe-sentinel"));
    assert!(!report.contains("must-not-land"));
    assert!(validate_fixture("AWS::EC2::VPC", &Value::Map(BTreeMap::new()), &expected).is_err());
    assert_organization_mutation(records)
}

fn assert_organization_mutation(records: &Expectations) -> TestResult {
    use inventory_core::record::Key;
    let kind = "AWS::Organizations::Organization";
    let (_, expected) = records
        .iter()
        .find(|(key, _)| key.kind.as_str() == kind)
        .ok_or("organization fixture missing")?;
    let original: BTreeMap<_, _> = expected
        .iter()
        .map(|(name, value)| (Key::Name(name.clone()), value.clone()))
        .collect();
    let actual = Value::Map(original.clone());
    validate_fixture(kind, &actual, expected)?;

    let mut dropped = original;
    assert!(
        dropped
            .remove(&Key::Name("MasterAccountId".into()))
            .is_some()
    );
    let mutated = Value::Map(dropped);
    let Err(error) = validate_fixture(kind, &mutated, expected) else {
        return Err("organization oracle accepted a missing MasterAccountId".into());
    };
    assert!(error.to_string().contains(
        "AWS::Organizations::Organization | MasterAccountId | Absent | differs from configured Text value"
    ));
    Ok(())
}
