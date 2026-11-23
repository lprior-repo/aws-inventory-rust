use super::{
    ALB_NAME, CLOUDWATCH_KINDS, EC2_EDGES, EC2_KINDS, ELBV2_EDGES, ELBV2_KINDS, TARGET_GROUP_NAME,
    cloudwatch_identities,
};
use inventory_core::inventory::EdgeSource;
use inventory_harness::inventory_seed::{
    ScenarioSeedOracle, SeedIdentityEvidence, SeedResourceIdentity, StepId,
};
use std::{error::Error, fmt::Debug, io};

type Check = Result<(), Box<dyn Error>>;

pub(crate) fn assert_ec2_network_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> Check {
    resources(
        oracle,
        account,
        region,
        &EC2_KINDS,
        SeedIdentityEvidence::Response,
    )?;
    [(0, "vpc-"), (1, "subnet-"), (2, "sg-"), (3, "rtb-")]
        .iter()
        .try_for_each(|(step, prefix)| identity_prefix(oracle, *step, prefix))?;
    edges(oracle, &EC2_EDGES)
}

pub(crate) fn assert_elbv2_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> Check {
    resources(
        oracle,
        account,
        region,
        &ELBV2_KINDS,
        SeedIdentityEvidence::Response,
    )?;
    [(0, "vpc-"), (1, "subnet-"), (2, "subnet-"), (3, "sg-")]
        .iter()
        .try_for_each(|(step, prefix)| identity_prefix(oracle, *step, prefix))?;
    identity_prefix(
        oracle,
        4,
        &format!("arn:aws:elasticloadbalancing:{region}:{account}:loadbalancer/app/{ALB_NAME}/"),
    )?;
    identity_prefix(
        oracle,
        5,
        &format!(
            "arn:aws:elasticloadbalancing:{region}:{account}:targetgroup/{TARGET_GROUP_NAME}/"
        ),
    )?;
    edges(oracle, &ELBV2_EDGES)
}

pub(crate) fn assert_cloudwatch_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> Check {
    resources(
        oracle,
        account,
        region,
        &CLOUDWATCH_KINDS,
        SeedIdentityEvidence::Request,
    )?;
    let expected = cloudwatch_identities(account, region, "aws");
    CLOUDWATCH_KINDS
        .iter()
        .zip(expected.iter())
        .try_for_each(|((step, _), id)| {
            equal(
                resource(oracle, *step)?.id.as_str(),
                id.as_str(),
                "alarm identity",
            )
        })?;
    edges(oracle, &[])
}

fn resources(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
    kinds: &[(u16, &str)],
    evidence: SeedIdentityEvidence,
) -> Check {
    equal(oracle.account_id.as_str(), account, "oracle account")?;
    equal(oracle.region.as_str(), region, "oracle region")?;
    equal(oracle.resources.len(), kinds.len(), "resource count")?;
    kinds.iter().try_for_each(|(step, kind)| {
        let actual = resource(oracle, *step)?;
        equal(actual.kind.as_str(), *kind, "resource kind")?;
        equal(actual.evidence, evidence, "identity evidence")?;
        equal(actual.key.account.as_str(), account, "resource account")?;
        equal(actual.key.region.as_str(), region, "resource region")?;
        equal(actual.key.kind.as_str(), *kind, "key kind")?;
        equal(&actual.key.id, &actual.id, "key identity")?;
        equal(actual.id.as_str().is_empty(), false, "identity is empty")
    })?;
    oracle
        .resources
        .iter()
        .enumerate()
        .try_for_each(|(index, actual)| {
            equal(
                oracle
                    .resources
                    .iter()
                    .take(index)
                    .any(|previous| previous.key == actual.key),
                false,
                "duplicate resource key",
            )
        })
}

fn resource(
    oracle: &ScenarioSeedOracle,
    step: u16,
) -> Result<&SeedResourceIdentity, Box<dyn Error>> {
    oracle
        .resources
        .iter()
        .find(|resource| resource.step == StepId::new(step))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("missing resource step {step}"),
            )
            .into()
        })
}

fn identity_prefix(oracle: &ScenarioSeedOracle, step: u16, prefix: &str) -> Check {
    let actual = resource(oracle, step)?.id.as_str();
    equal(
        actual
            .strip_prefix(prefix)
            .is_some_and(|suffix| !suffix.is_empty()),
        true,
        "generated identity prefix and suffix",
    )
}

fn edges(oracle: &ScenarioSeedOracle, expected: &[(u16, u16, &str)]) -> Check {
    equal(oracle.edges.len(), expected.len(), "relationship count")?;
    expected.iter().try_for_each(|(from, to, relation)| {
        let source = resource(oracle, *from)?;
        let target = resource(oracle, *to)?;
        let matching = oracle
            .edges
            .iter()
            .filter(|edge| {
                edge.from == source.key
                    && edge.to == target.key
                    && edge.relation.as_str() == *relation
                    && edge.source == EdgeSource::ReviewedRule
            })
            .count();
        equal(matching, 1, "exact reviewed relationship")
    })
}

fn equal<T: PartialEq + Debug>(actual: T, expected: T, label: &str) -> Check {
    if actual == expected {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{label}: expected {expected:?}, received {actual:?}"),
    )
    .into())
}
