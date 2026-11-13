//! Independent examples and generated identity contracts for schema-derived edges.

use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};
use tokio::sync::mpsc;

use super::*;

type TestError = Box<dyn std::error::Error>;
type TestResult = Result<(), TestError>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixtures {
    schema_version: u32,
    fixtures: Vec<Fixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    name: String,
    type_name: String,
    identifier: String,
    provider: Value,
    expected_resource_id: Option<String>,
    opaque_identity: bool,
    edges: Vec<ExpectedEdge>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedEdge {
    relation: String,
    target: String,
    kind: String,
    paths: BTreeSet<String>,
    owner: String,
}

fn context() -> Result<Context, ApiError> {
    let (sender, _receiver) = mpsc::channel(64);
    Ok(Context {
        config: aws_types::SdkConfig::builder().build(),
        executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
        sender,
        account: "111111111111".into(),
        partition: "aws".into(),
        region: "us-east-1".into(),
        max_pages: 4,
        page_size: 100,
    })
}

fn field<'a>(properties: &'a Value, name: &str) -> Result<&'a str, TestError> {
    properties
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string evidence {name}").into())
}

fn observed_paths(properties: &Value) -> Result<BTreeSet<String>, TestError> {
    let values = properties
        .get("property_paths")
        .and_then(Value::as_array)
        .ok_or("missing property-path evidence")?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "non-string property-path evidence".into())
        })
        .collect()
}

#[test]
fn independently_authored_architecture_fixtures_preserve_exact_targets_and_evidence() -> TestResult
{
    let fixtures: Fixtures = serde_json::from_str(include_str!(
        "../../tests/fixtures/cloudcontrol-architecture.json"
    ))?;
    assert_eq!(fixtures.schema_version, 1);
    assert_eq!(fixtures.fixtures.len(), 18);
    let catalog = catalog()?;
    let ctx = context()?;
    let mut names = BTreeSet::new();
    for fixture in fixtures.fixtures {
        assert!(names.insert(fixture.name.clone()), "duplicate fixture name");
        let descriptor = catalog
            .iter()
            .find(|descriptor| descriptor.type_name == fixture.type_name)
            .ok_or_else(|| format!("missing fixture type {}", fixture.type_name))?;
        let description = ResourceDescription::builder()
            .identifier(&fixture.identifier)
            .properties(fixture.provider.to_string())
            .build();
        let before = now_millis()?;
        let body = map_description(&ctx, descriptor, &fixture.identifier, &description, true)?;
        let after = now_millis()?;
        assert_eq!(body.resources.len(), 1, "{}", fixture.name);
        let resource = body.resources.first().ok_or("missing fixture resource")?;
        assert_eq!(resource.source, Source::CloudControl, "{}", fixture.name);
        assert!(
            (before..=after).contains(&resource.observed_at),
            "{} observation time must be local and current",
            fixture.name
        );
        assert!(
            !serde_json::to_string(resource)?.contains("HIDDEN"),
            "{} leaked a deliberately sensitive fixture value",
            fixture.name
        );
        match fixture.expected_resource_id {
            Some(expected) => assert_eq!(resource.id, expected, "{}", fixture.name),
            None => {
                assert!(fixture.opaque_identity, "{}", fixture.name);
                assert!(
                    resource.id.starts_with(&format!(
                        "awscc:aws:us-east-1:111111111111:{}:",
                        fixture.type_name
                    )),
                    "{} compound identifier must remain opaque",
                    fixture.name
                );
                assert_eq!(
                    field(&resource.properties, "identity_resolution")?,
                    "namespaced_provider_identifier",
                    "{}",
                    fixture.name
                );
            }
        }
        if fixture.name == "snapshot_self_identity_uses_accountless_arn" {
            assert_eq!(
                field(&resource.properties, "identity_resolution")?,
                "canonical_template_without_account_component"
            );
        }
        assert_eq!(body.edges.len(), fixture.edges.len(), "{}", fixture.name);
        for expected in fixture.edges {
            let edge = body
                .edges
                .iter()
                .find(|edge| edge.relation == expected.relation && edge.to_id == expected.target)
                .ok_or_else(|| {
                    format!(
                        "{} lacks expected {} edge to {}",
                        fixture.name, expected.relation, expected.target
                    )
                })?;
            assert_eq!(edge.from_id, resource.id, "{}", fixture.name);
            assert_eq!(edge.source, Source::CloudControl, "{}", fixture.name);
            assert_eq!(edge.observed_at, resource.observed_at, "{}", fixture.name);
            assert_eq!(field(&edge.properties, "target_kind")?, expected.kind);
            assert_eq!(field(&edge.properties, "owner_resolution")?, expected.owner);
            assert_eq!(
                field(&edge.properties, "cloudformation_type")?,
                fixture.type_name
            );
            assert_eq!(observed_paths(&edge.properties)?, expected.paths);
        }
        // List-only properties never become architecture assertions, even if
        // a provider happens to return the same rich properties in both calls.
        let partial = map_description(&ctx, descriptor, &fixture.identifier, &description, false)?;
        assert!(partial.edges.is_empty(), "{}", fixture.name);
    }
    Ok(())
}

#[test]
fn new_types_remove_sensitive_values_before_identity_edges_or_serialization() -> TestResult {
    let fixtures = [
        (
            "AWS::ApiGateway::ApiKey",
            json!({"APIKeyId":"key-id","Value":"HIDDEN","Name":"visible"}),
            "/Value",
        ),
        (
            "AWS::ElastiCache::User",
            json!({"UserId":"user-id","Passwords":["HIDDEN"],"UserName":"visible"}),
            "/Passwords",
        ),
        (
            "AWS::SageMaker::Pipeline",
            json!({"PipelineName":"visible","PipelineDefinition":{"PipelineDefinitionBody":"HIDDEN"}}),
            "/PipelineDefinition",
        ),
        (
            "AWS::CloudFormation::Stack",
            json!({"StackName":"visible","Parameters":{"password":"HIDDEN"},"Outputs":[{"OutputKey":"Secret","OutputValue":"HIDDEN"}]}),
            "/Outputs",
        ),
    ];
    let catalog = catalog()?;
    for (type_name, provider, removed_path) in fixtures {
        let descriptor = catalog
            .iter()
            .find(|descriptor| descriptor.type_name == type_name)
            .ok_or_else(|| format!("missing redaction fixture type {type_name}"))?;
        let (safe, paths) = sanitized_properties(descriptor, Some(&provider.to_string()))?;
        assert!(!safe.to_string().contains("HIDDEN"), "{type_name}");
        assert!(safe.pointer(removed_path).is_none(), "{type_name}");
        assert!(paths.iter().any(|path| path == removed_path), "{type_name}");
    }
    Ok(())
}

fn arn_rule(service: Option<&str>, resource_prefix: Option<&str>) -> Relationship {
    Relationship {
        pointer: "/RoleArn".into(),
        relation: "USES_ROLE".into(),
        target_kind: "aws_iam_role".into(),
        target_format: "arn".into(),
        arn_template: None,
        evidence: "independent_test_constraint".into(),
        arn_service: service.map(str::to_owned),
        arn_resource_prefix: resource_prefix.map(str::to_owned),
        identifier_prefix: None,
    }
}

fn identifier_rule() -> Relationship {
    Relationship {
        pointer: "/SubnetIds/*".into(),
        relation: "USES_SUBNET".into(),
        target_kind: "aws_vpc_subnet".into(),
        target_format: "template".into(),
        arn_template: Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}".into()),
        evidence: "independent_test_constraint".into(),
        arn_service: None,
        arn_resource_prefix: None,
        identifier_prefix: Some("subnet-".into()),
    }
}

fn configuration() -> Config {
    Config {
        cases: 128,
        max_shrink_iters: 4_096,
        rng_seed: RngSeed::Fixed(0x4343_5f41_524e_3035),
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn returned_role_arns_require_the_declared_service_and_resource_kind(
        account in "[0-9]{12}",
        role in "[a-z][a-z0-9-]{0,48}",
        partition in prop::sample::select(vec!["aws", "aws-us-gov", "aws-cn"]),
    ) {
        let rule = arn_rule(Some("iam"), Some("role/"));
        let valid = format!("arn:{partition}:iam::{account}:role/team/{role}");
        prop_assert!(reference_arn_matches(&rule, &valid), "a concrete cross-account role ARN must survive");
        let wrong_service = format!("arn:{partition}:sns:us-east-1:{account}:role/team/{role}");
        prop_assert!(!reference_arn_matches(&rule, &wrong_service), "a role-shaped SNS resource is not an IAM role");
        let wrong_kind = format!("arn:{partition}:iam::{account}:policy/team/{role}");
        prop_assert!(!reference_arn_matches(&rule, &wrong_kind), "an IAM policy is not an IAM role");
        prop_assert!(!reference_arn_matches(&rule, &format!("{valid}*")), "wildcard selectors are not concrete resources");
    }

    #[test]
    fn template_references_require_single_identifiers_with_the_reviewed_prefix(
        suffix in "[a-f0-9]{8,17}",
        illegal in prop::sample::select(vec!["|", "/", ":", "*", "?", " ", "\0"]),
    ) {
        let rule = identifier_rule();
        let valid = format!("subnet-{suffix}");
        prop_assert!(reference_identifier_matches(&rule, &valid), "valid reviewed subnet ID");
        prop_assert!(!reference_identifier_matches(&rule, &format!("sg-{suffix}")), "security group IDs cannot become subnet ARNs");
        prop_assert!(!reference_identifier_matches(&rule, &format!("{valid}{illegal}{valid}")), "compound, ARN, path and pattern values stay unexpanded");
        prop_assert!(!reference_identifier_matches(&rule, "subnet-"), "the prefix alone is not an identifier");
    }

    #[test]
    fn general_arn_rules_preserve_accountless_resources_and_concrete_log_group_suffixes(
        name in "[a-z][a-z0-9-]{1,40}",
        account in "[0-9]{12}",
    ) {
        let rule = arn_rule(None, None);
        prop_assert!(reference_arn_matches(&rule, &format!("arn:aws:s3:::{name}")), "accountless S3 ARN");
        prop_assert!(reference_arn_matches(&rule, &format!("arn:aws:iam::aws:policy/{name}")), "AWS-managed policy ARN");
        prop_assert!(reference_arn_matches(&rule, &format!("arn:aws:logs:us-west-2:{account}:log-group:{name}:*")), "the documented concrete CloudWatch Logs suffix is retained");
        prop_assert!(!reference_arn_matches(&rule, &format!("arn:aws:logs:us-west-2:{account}:log-group:{name}*:*")), "a wildcard log group remains a selector");
        prop_assert!(!reference_arn_matches(&rule, &format!("arn:aws:s3:::{name}?")), "a wildcard ARN is not an observed dependency");
        prop_assert!(!reference_arn_matches(&rule, &format!("arn:aws:iam::aws:role/{name}")), "the AWS pseudo-account applies to managed policies only");
    }
}
