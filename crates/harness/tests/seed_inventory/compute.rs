use std::{collections::BTreeSet, error::Error, io};

use inventory_core::{inventory::{Edge, EdgeSource, Region, Relation}, record::Value};
use inventory_harness::inventory_seed::{
    FieldPath, ScenarioSeedOracle, SeedError, SeedIdentityEvidence, SeedInput, SeedInputValue,
    SeedNestedBinding, SeedOutputCheck, SeedRelationship, SeedResourceIdentity, SeedScenario,
    SeedScope, SeedStep, SeedTemplate, SeedValuePart, StepId,
};

const SIGNING_PROFILE: &str = "inventory_seed_lambda_signing";
const EKS_CLUSTER: &str = "inventory-seed-eks";
const EKS_ROLE: &str = "inventory-seed-eks-control-plane";
const ECS_CLUSTER: &str = "inventory-seed-ecs";
const ECS_SERVICE: &str = "inventory-seed-external-service";
const EKS_TRUST: &str = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"eks.amazonaws.com"},"Action":"sts:AssumeRole"}]}"#;

// Given an isolated local account, create only signing metadata: never function
// code, signing jobs, certificates or key material. Opaque config IDs are observed;
// the profile ARN is independently derived from caller-supplied account context.
pub fn lambda(partition: &str, region: &str, account: &str) -> Result<SeedScenario, SeedError> {
    let profile = SeedStep::new(
        StepId::new(0), "signer".into(), "PutSigningProfile".into(),
        "AWS::Signer::SigningProfile".into(), FieldPath::new(&["arn"])?,
        vec![text(&["profileName"], SIGNING_PROFILE)?,
             text(&["platformId"], "AWSLambda-SHA384-ECDSA")?],
    ).with_expected_identity(format!(
        "arn:{partition}:signer:{region}:{account}:/signing-profiles/{SIGNING_PROFILE}"
    ));
    let config = SeedStep::new(
        StepId::new(1), "lambda".into(), "CreateCodeSigningConfig".into(),
        "AWS::Lambda::CodeSigningConfig".into(),
        FieldPath::new(&["CodeSigningConfig", "CodeSigningConfigArn"])?,
        vec![text(&["Description"], "Local inventory metadata fixture")?,
             reference_list(&["AllowedPublishers", "SigningProfileVersionArns"],
                 &[(0, &["profileVersionArn"])])?,
             text(&["CodeSigningPolicies", "UntrustedArtifactOnDeployment"], "Enforce")?],
    ).with_response_checks(vec![
        check(&["CodeSigningConfig", "Description"], &["Description"])?,
        check(&["CodeSigningConfig", "AllowedPublishers", "SigningProfileVersionArns"],
            &["AllowedPublishers", "SigningProfileVersionArns"])?,
        check(&["CodeSigningConfig", "CodeSigningPolicies", "UntrustedArtifactOnDeployment"],
            &["CodeSigningPolicies", "UntrustedArtifactOnDeployment"])?,
    ]);
    SeedScenario::new(vec![profile, config])
}

// Given two real zones and an in-account control-plane role, create a cluster
// with actual EC2 subnet dependencies, but no node groups or workloads. Then the
// oracle asserts both canonical subnet-to-VPC edges, not invented EKS mappings.
pub fn eks(partition: &str, region: &str, account: &str) -> Result<SeedScenario, SeedError> {
    SeedScenario::with_relationships(
        vec![
            SeedStep::auxiliary(StepId::new(0), "ec2".into(),
                "DescribeAvailabilityZones".into(), vec![]),
            eks_role(partition, account)?,
            eks_vpc()?,
            eks_subnet(3, "10.89.1.0/24", "0")?,
            eks_subnet(4, "10.89.2.0/24", "1")?,
            eks_cluster(partition, region, account)?,
        ],
        vec![
            SeedRelationship::new(StepId::new(3), StepId::new(2), "IN_VPC".into(),
                FieldPath::new(&["Subnet", "VpcId"])?),
            SeedRelationship::new(StepId::new(4), StepId::new(2), "IN_VPC".into(),
                FieldPath::new(&["Subnet", "VpcId"])?),
        ],
    )
}

fn eks_role(partition: &str, account: &str) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(1), "iam".into(), "CreateRole".into(), "AWS::IAM::Role".into(),
        FieldPath::new(&["Role", "Arn"])?,
        vec![text(&["RoleName"], EKS_ROLE)?,
             text(&["AssumeRolePolicyDocument"], EKS_TRUST)?],
    ).with_scope(SeedScope::Account)
        .with_expected_identity(format!("arn:{partition}:iam::{account}:role/{EKS_ROLE}"))
        .with_response_checks(vec![check(&["Role", "RoleName"], &["RoleName"])?]))
}

fn eks_vpc() -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(2), "ec2".into(), "CreateVpc".into(), "AWS::EC2::VPC".into(),
        FieldPath::new(&["Vpc", "VpcId"])?, vec![text(&["CidrBlock"], "10.89.0.0/16")?],
    ).with_response_checks(vec![check(&["Vpc", "CidrBlock"], &["CidrBlock"])?]))
}

fn eks_subnet(id: u8, cidr: &str, zone_index: &str) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(id), "ec2".into(), "CreateSubnet".into(), "AWS::EC2::Subnet".into(),
        FieldPath::new(&["Subnet", "SubnetId"])?,
        vec![previous(&["VpcId"], 2, &["Vpc", "VpcId"])?,
             text(&["CidrBlock"], cidr)?,
             previous(&["AvailabilityZone"], 0,
                 &["AvailabilityZones", zone_index, "ZoneName"])?],
    ).with_response_checks(vec![
        check(&["Subnet", "VpcId"], &["VpcId"])?,
        check(&["Subnet", "CidrBlock"], &["CidrBlock"])?,
        check(&["Subnet", "AvailabilityZone"], &["AvailabilityZone"])?,
    ]))
}

fn eks_cluster(partition: &str, region: &str, account: &str) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(5), "eks".into(), "CreateCluster".into(), "AWS::EKS::Cluster".into(),
        FieldPath::new(&["cluster", "arn"])?,
        vec![text(&["name"], EKS_CLUSTER)?,
             previous(&["roleArn"], 1, &["Role", "Arn"])?,
             reference_list(&["resourcesVpcConfig", "subnetIds"],
                 &[(3, &["Subnet", "SubnetId"]), (4, &["Subnet", "SubnetId"])])?,
             input(&["resourcesVpcConfig", "endpointPublicAccess"], SeedInputValue::Boolean(false))?,
             input(&["resourcesVpcConfig", "endpointPrivateAccess"], SeedInputValue::Boolean(true))?],
    ).with_expected_identity(format!("arn:{partition}:eks:{region}:{account}:cluster/{EKS_CLUSTER}"))
        .with_response_checks(vec![
            check(&["cluster", "name"], &["name"])?,
            check(&["cluster", "roleArn"], &["roleArn"])?,
            check(&["cluster", "resourcesVpcConfig", "subnetIds"],
                &["resourcesVpcConfig", "subnetIds"])?,
            check(&["cluster", "resourcesVpcConfig", "endpointPublicAccess"],
                &["resourcesVpcConfig", "endpointPublicAccess"])?,
            check(&["cluster", "resourcesVpcConfig", "endpointPrivateAccess"],
                &["resourcesVpcConfig", "endpointPrivateAccess"])?,
        ]))
}

// Given a cluster, EXTERNAL permits a metadata-only service with no task
// definition, image, network or execution role. Then names, controller, zero
// desired count and both independently constructed ARNs must match the response.
pub fn ecs(partition: &str, region: &str, account: &str) -> Result<SeedScenario, SeedError> {
    let cluster = SeedStep::new(
        StepId::new(0), "ecs".into(), "CreateCluster".into(), "AWS::ECS::Cluster".into(),
        FieldPath::new(&["cluster", "clusterArn"])?, vec![text(&["clusterName"], ECS_CLUSTER)?],
    ).with_expected_identity(format!("arn:{partition}:ecs:{region}:{account}:cluster/{ECS_CLUSTER}"))
        .with_response_checks(vec![check(&["cluster", "clusterName"], &["clusterName"])?]);
    let service = SeedStep::new(
        StepId::new(1), "ecs".into(), "CreateService".into(), "AWS::ECS::Service".into(),
        FieldPath::new(&["service", "serviceArn"])?,
        vec![previous(&["cluster"], 0, &["cluster", "clusterArn"])?,
             text(&["serviceName"], ECS_SERVICE)?,
             text(&["deploymentController", "type"], "EXTERNAL")?,
             input(&["desiredCount"], SeedInputValue::Integer(0))?],
    ).with_expected_identity(format!(
        "arn:{partition}:ecs:{region}:{account}:service/{ECS_CLUSTER}/{ECS_SERVICE}"
    )).with_response_checks(vec![
        check(&["service", "clusterArn"], &["cluster"])?,
        check(&["service", "serviceName"], &["serviceName"])?,
        check(&["service", "deploymentController", "type"], &["deploymentController", "type"])?,
        check(&["service", "desiredCount"], &["desiredCount"])?,
    ]);
    SeedScenario::new(vec![cluster, service])
}

fn input(field: &[&str], value: SeedInputValue) -> Result<SeedInput, SeedError> {
    Ok(SeedInput::new(FieldPath::new(field)?, value))
}

fn text(field: &[&str], value: &str) -> Result<SeedInput, SeedError> {
    input(field, SeedInputValue::Text(value.into()))
}

fn previous(field: &[&str], step: u8, path: &[&str]) -> Result<SeedInput, SeedError> {
    input(field, SeedInputValue::PreviousOutput {
        step: StepId::new(step), path: FieldPath::new(path)?,
    })
}

fn check(path: &[&str], field: &[&str]) -> Result<SeedOutputCheck, SeedError> {
    Ok(SeedOutputCheck::new(FieldPath::new(path)?, FieldPath::new(field)?))
}

fn reference_list(field: &[&str], sources: &[(u8, &[&str])]) -> Result<SeedInput, SeedError> {
    let bindings = sources.iter().enumerate().map(|(index, (step, path))| {
        Ok(SeedNestedBinding {
            destination: vec![SeedValuePart::Index(index)], step: StepId::new(*step),
            path: FieldPath::new(path)?,
        })
    }).collect::<Result<Vec<_>, SeedError>>()?;
    input(field, SeedInputValue::Template(SeedTemplate {
        value: Value::List(sources.iter().map(|_| Value::Null).collect()), bindings,
    }))
}

pub(super) fn assert_lambda_oracle(oracle: &ScenarioSeedOracle, account: &str, region: &str)
    -> Result<(), Box<dyn Error>> {
    assert_resources(oracle, account, region, &[
        (0, "AWS::Signer::SigningProfile", region),
        (1, "AWS::Lambda::CodeSigningConfig", region),
    ])?;
    assert!(oracle.edges.is_empty());
    assert_eq!(resource(oracle, 0)?.id.as_str(),
        format!("arn:aws:signer:{region}:{account}:/signing-profiles/{SIGNING_PROFILE}"));
    let prefix = format!("arn:aws:lambda:{region}:{account}:code-signing-config:");
    let config = resource(oracle, 1)?.id.as_str();
    assert!(config.starts_with(&prefix) && config.len() > prefix.len());
    Ok(())
}

pub(super) fn assert_eks_oracle(oracle: &ScenarioSeedOracle, account: &str, region: &str)
    -> Result<(), Box<dyn Error>> {
    assert_resources(oracle, account, region, &[
        (1, "AWS::IAM::Role", "global"),
        (2, "AWS::EC2::VPC", region),
        (3, "AWS::EC2::Subnet", region),
        (4, "AWS::EC2::Subnet", region),
        (5, "AWS::EKS::Cluster", region),
    ])?;
    assert_eq!(resource(oracle, 1)?.id.as_str(),
        format!("arn:aws:iam::{account}:role/{EKS_ROLE}"));
    assert_eq!(resource(oracle, 5)?.id.as_str(),
        format!("arn:aws:eks:{region}:{account}:cluster/{EKS_CLUSTER}"));
    assert!(resource(oracle, 2)?.id.as_str().starts_with("vpc-"));
    assert!(resource(oracle, 3)?.id.as_str().starts_with("subnet-"));
    assert!(resource(oracle, 4)?.id.as_str().starts_with("subnet-"));
    assert_ne!(resource(oracle, 3)?.id, resource(oracle, 4)?.id);
    let edges = [3, 4].into_iter().map(|step| {
        Ok(Edge {
            from: resource(oracle, step)?.key.clone(),
            to: resource(oracle, 2)?.key.clone(),
            relation: Relation::parse("IN_VPC")?,
            source: EdgeSource::ReviewedRule,
        })
    }).collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    assert_eq!(oracle.edges, edges);
    Ok(())
}

pub(super) fn assert_ecs_oracle(oracle: &ScenarioSeedOracle, account: &str, region: &str)
    -> Result<(), Box<dyn Error>> {
    assert_resources(oracle, account, region, &[
        (0, "AWS::ECS::Cluster", region),
        (1, "AWS::ECS::Service", region),
    ])?;
    assert!(oracle.edges.is_empty());
    assert_eq!(resource(oracle, 0)?.id.as_str(),
        format!("arn:aws:ecs:{region}:{account}:cluster/{ECS_CLUSTER}"));
    assert_eq!(resource(oracle, 1)?.id.as_str(),
        format!("arn:aws:ecs:{region}:{account}:service/{ECS_CLUSTER}/{ECS_SERVICE}"));
    Ok(())
}

fn assert_resources(oracle: &ScenarioSeedOracle, account: &str, region: &str,
    expected: &[(u8, &str, &str)]) -> Result<(), Box<dyn Error>> {
    assert_eq!(oracle.account_id.as_str(), account);
    assert_eq!(oracle.region, Region::parse(region)?);
    assert_eq!(oracle.resources.len(), expected.len());
    assert_eq!(oracle.resources.iter().map(|item| &item.key).collect::<BTreeSet<_>>().len(),
        expected.len());
    oracle.resources.iter().zip(expected).try_for_each(|(item, (step, kind, scope))| {
        assert_eq!(item.step, StepId::new(*step));
        assert_eq!(item.kind.as_str(), *kind);
        assert_eq!(item.key.kind, item.kind);
        assert_eq!(item.key.id, item.id);
        assert_eq!(item.key.account.as_str(), account);
        assert_eq!(item.key.region, Region::parse(scope)?);
        assert_eq!(item.evidence, SeedIdentityEvidence::Response);
        assert!(!item.id.as_str().is_empty());
        Ok::<(), Box<dyn Error>>(())
    })
}

fn resource(oracle: &ScenarioSeedOracle, step: u8)
    -> Result<&SeedResourceIdentity, Box<dyn Error>> {
    oracle.resources.iter().find(|item| item.step == StepId::new(step))
        .ok_or_else(|| io::Error::other(format!("missing compute resource step {step}")).into())
}
