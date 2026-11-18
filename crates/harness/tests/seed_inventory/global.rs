use inventory_core::inventory::{Kind, Region};
use inventory_harness::inventory_seed::{
    FieldPath, ScenarioSeedOracle, SeedError, SeedIdentityEvidence, SeedInput, SeedInputValue,
    SeedOutputCheck, SeedResourceIdentity, SeedScenario, SeedScope, SeedStep, StepId,
};
use std::error::Error;

// Given a local emulator, when the shared runner executes a recipe, then its
// response-derived identity and echoed configuration become independent evidence
// for comparison with collection. Generated IDs are never guessed in advance.

pub(super) fn iam() -> Result<SeedScenario, SeedError> {
    let step = SeedStep::new(
        StepId::new(0),
        "iam".into(),
        "CreateGroup".into(),
        "AWS::IAM::Group".into(),
        FieldPath::new(&["Group", "Arn"])?,
        vec![
            text(&["GroupName"], "inventory-seed-group")?,
            text(&["Path"], "/inventory-seed/")?,
        ],
    )
    .with_response_checks(vec![
        check(&["Group", "GroupName"], &["GroupName"])?,
        check(&["Group", "Path"], &["Path"])?,
    ]);
    account_scenario(step)
}

// Requires an emulator account without an existing organization.
// CreateAccount is deliberately absent: its CreateAccountStatus.Id identifies a
// request, not an account. A bounded DescribeCreateAccountStatus waiter must
// require State=SUCCEEDED, reject FAILED and obtain AccountId before publication.
// ListRoots can supply an OU ParentId, but complete recursive OU collection is
// deferred by the Organizations overlay, so this recipe creates no OUs.
pub(super) fn organizations() -> Result<SeedScenario, SeedError> {
    let step = SeedStep::new(
        StepId::new(0),
        "organizations".into(),
        "CreateOrganization".into(),
        "AWS::Organizations::Organization".into(),
        FieldPath::new(&["Organization", "Arn"])?,
        vec![text(&["FeatureSet"], "ALL")?],
    )
    .with_response_checks(vec![check(
        &["Organization", "FeatureSet"],
        &["FeatureSet"],
    )?]);
    account_scenario(step)
}

// Public-zone metadata only: no records, registration or VPC association.
// Preserve the returned HostedZone.Id, including any /hostedzone/ prefix;
// ChangeInfo.Id is not the hosted-zone identity.
pub(super) fn route53() -> Result<SeedScenario, SeedError> {
    let step = SeedStep::new(
        StepId::new(0),
        "route-53".into(),
        "CreateHostedZone".into(),
        "AWS::Route53::HostedZone".into(),
        FieldPath::new(&["HostedZone", "Id"])?,
        vec![
            text(&["Name"], "inventory-seed.example.com.")?,
            text(
                &["CallerReference"],
                "inventory-seed-route53-public-zone-v1",
            )?,
        ],
    )
    .with_response_checks(vec![
        check(&["HostedZone", "Name"], &["Name"])?,
        check(&["HostedZone", "CallerReference"], &["CallerReference"])?,
    ]);
    account_scenario(step)
}

// An unattached origin access control is configuration metadata only. It creates
// no distribution, origin request, function, content or key material.
pub(super) fn cloudfront() -> Result<SeedScenario, SeedError> {
    let fields = [
        ("Name", "inventory-seed-origin-access-control"),
        ("SigningProtocol", "sigv4"),
        ("SigningBehavior", "always"),
        ("OriginAccessControlOriginType", "s3"),
    ];
    let inputs = fields
        .iter()
        .map(|(field, value)| text(&["OriginAccessControlConfig", field], value))
        .collect::<Result<Vec<_>, _>>()?;
    let checks = fields
        .iter()
        .map(|(field, _)| {
            check(
                &["OriginAccessControl", "OriginAccessControlConfig", field],
                &["OriginAccessControlConfig", field],
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let step = SeedStep::new(
        StepId::new(0),
        "cloudfront".into(),
        "CreateOriginAccessControl".into(),
        "AWS::CloudFront::OriginAccessControl".into(),
        FieldPath::new(&["OriginAccessControl", "Id"])?,
        inputs,
    )
    .with_response_checks(checks);
    account_scenario(step)
}

fn account_scenario(step: SeedStep) -> Result<SeedScenario, SeedError> {
    SeedScenario::new(vec![step.with_scope(SeedScope::Account)])
}

fn text(field: &[&str], value: &str) -> Result<SeedInput, SeedError> {
    Ok(SeedInput::new(
        FieldPath::new(field)?,
        SeedInputValue::Text(value.to_owned()),
    ))
}

fn check(output: &[&str], input: &[&str]) -> Result<SeedOutputCheck, SeedError> {
    Ok(SeedOutputCheck::new(
        FieldPath::new(output)?,
        FieldPath::new(input)?,
    ))
}

type OracleResult = Result<(), Box<dyn Error>>;

pub(super) fn assert_iam_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> OracleResult {
    let resource = account_resource(oracle, account, region, "AWS::IAM::Group")?;
    assert_eq!(
        resource.id.as_str(),
        format!(
            "arn:{}:iam::{account}:group/inventory-seed/inventory-seed-group",
            partition(region)
        ),
    );
    Ok(())
}

pub(super) fn assert_organizations_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> OracleResult {
    let resource = account_resource(oracle, account, region, "AWS::Organizations::Organization")?;
    let prefix = format!(
        "arn:{}:organizations::{account}:organization/o-",
        partition(region)
    );
    let organization_id = resource
        .id
        .as_str()
        .strip_prefix(&prefix)
        .ok_or("organization identity does not match its account and partition")?;
    assert!(!organization_id.is_empty());
    Ok(())
}

pub(super) fn assert_route53_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> OracleResult {
    let resource = account_resource(oracle, account, region, "AWS::Route53::HostedZone")?;
    let id = resource
        .id
        .as_str()
        .strip_prefix("/hostedzone/")
        .map_or(resource.id.as_str(), |id| id);
    assert!(!id.is_empty());
    assert!(!id.contains('/'));
    Ok(())
}

pub(super) fn assert_cloudfront_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> OracleResult {
    account_resource(
        oracle,
        account,
        region,
        "AWS::CloudFront::OriginAccessControl",
    )?;
    Ok(())
}

fn account_resource<'a>(
    oracle: &'a ScenarioSeedOracle,
    account: &str,
    region: &str,
    kind: &str,
) -> Result<&'a SeedResourceIdentity, Box<dyn Error>> {
    assert_eq!(oracle.account_id.as_str(), account);
    assert_eq!(oracle.region, Region::parse(region)?);
    assert_eq!(oracle.resources.len(), 1);
    assert!(oracle.edges.is_empty());
    let resource = oracle
        .resources
        .first()
        .ok_or("missing seeded global resource")?;
    assert_eq!(resource.step, StepId::new(0));
    assert_eq!(resource.kind, Kind::parse(kind)?);
    assert_eq!(resource.key.kind, resource.kind);
    assert_eq!(resource.key.account, oracle.account_id);
    assert_eq!(resource.key.region, Region::Global);
    assert_eq!(resource.key.id, resource.id);
    assert_eq!(resource.evidence, SeedIdentityEvidence::Response);
    assert!(!resource.id.as_str().is_empty());
    Ok(resource)
}

fn partition(region: &str) -> &'static str {
    if region.starts_with("cn-") {
        "aws-cn"
    } else if region.starts_with("us-gov-") {
        "aws-us-gov"
    } else {
        "aws"
    }
}
