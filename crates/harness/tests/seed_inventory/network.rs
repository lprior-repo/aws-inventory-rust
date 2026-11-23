#[path = "network_oracle.rs"]
mod oracle;
pub(super) use oracle::{assert_cloudwatch_oracle, assert_ec2_network_oracle, assert_elbv2_oracle};

use inventory_core::record::{Float, Key, Value};
use inventory_harness::inventory_seed::{
    FieldPath, SeedError, SeedIdentity, SeedInput, SeedInputValue, SeedNestedBinding,
    SeedOutputCheck, SeedRelationship, SeedScenario, SeedStep, SeedTemplate, SeedValuePart, StepId,
};
use std::collections::BTreeMap;

// DEAD ELBV2_REGION deleted — never used; removed, not allowed.
pub(crate) const ALB_NAME: &str = "inventory-seed-alb";
pub(crate) const TARGET_GROUP_NAME: &str = "inventory-seed-targets";
pub(crate) const CPU_ALARM_NAME: &str = "inventory-seed-cpu";
pub(crate) const COMPOSITE_ALARM_NAME: &str = "inventory-seed-composite";

pub(crate) const EC2_KINDS: [(u16, &str); 4] = [
    (0, "AWS::EC2::VPC"),
    (1, "AWS::EC2::Subnet"),
    (2, "AWS::EC2::SecurityGroup"),
    (3, "AWS::EC2::RouteTable"),
];
pub(crate) const ELBV2_KINDS: [(u16, &str); 6] = [
    (0, "AWS::EC2::VPC"),
    (1, "AWS::EC2::Subnet"),
    (2, "AWS::EC2::Subnet"),
    (3, "AWS::EC2::SecurityGroup"),
    (4, "AWS::ElasticLoadBalancingV2::LoadBalancer"),
    (5, "AWS::ElasticLoadBalancingV2::TargetGroup"),
];
pub(crate) const CLOUDWATCH_KINDS: [(u16, &str); 2] = [
    (0, "AWS::CloudWatch::Alarm"),
    (1, "AWS::CloudWatch::CompositeAlarm"),
];
pub(crate) const EC2_EDGES: [(u16, u16, &str); 1] = [(1, 0, "IN_VPC")];
pub(crate) const ELBV2_EDGES: [(u16, u16, &str); 2] = [(1, 0, "IN_VPC"), (2, 0, "IN_VPC")];

pub(crate) fn ec2_network() -> Result<SeedScenario, SeedError> {
    SeedScenario::with_relationships(
        vec![
            vpc("10.89.0.0/16")?,
            subnet(1, "10.89.1.0/24", None)?,
            security_group(2, "inventory-seed-network")?,
            route_table()?,
        ],
        vec![subnet_edge(1)?],
    )
}

pub(crate) fn elbv2() -> Result<SeedScenario, SeedError> {
    SeedScenario::with_relationships(
        vec![
            vpc("10.90.0.0/16")?,
            subnet(1, "10.90.1.0/24", Some("us-east-1a"))?,
            subnet(2, "10.90.2.0/24", Some("us-east-1b"))?,
            security_group(3, "inventory-seed-elbv2")?,
            load_balancer()?,
            target_group()?,
        ],
        vec![subnet_edge(1)?, subnet_edge(2)?],
    )
}

pub(crate) fn cloudwatch(
    account: &str,
    region: &str,
    partition: &str,
) -> Result<SeedScenario, SeedError> {
    let [metric_arn, composite_arn] = cloudwatch_identities(account, region, partition);
    SeedScenario::new(vec![
        metric_alarm()?.with_expected_identity(metric_arn),
        composite_alarm()?.with_expected_identity(composite_arn),
    ])
}

pub(crate) fn cloudwatch_identities(account: &str, region: &str, partition: &str) -> [String; 2] {
    [
        format!("arn:{partition}:cloudwatch:{region}:{account}:alarm:inventory-seed-cpu"),
        format!("arn:{partition}:cloudwatch:{region}:{account}:alarm:inventory-seed-composite"),
    ]
}

fn text(field: &str, value: &str) -> Result<SeedInput, SeedError> {
    input(field, SeedInputValue::Text(value.into()))
}

fn input(field: &str, value: SeedInputValue) -> Result<SeedInput, SeedError> {
    Ok(SeedInput::new(FieldPath::new(&[field])?, value))
}

fn previous(field: &str, step: u16, path: &[&str]) -> Result<SeedInput, SeedError> {
    input(
        field,
        SeedInputValue::PreviousOutput {
            step: StepId::new(step),
            path: FieldPath::new(path)?,
        },
    )
}

fn check(output: &[&str], input: &str) -> Result<SeedOutputCheck, SeedError> {
    Ok(SeedOutputCheck::new(
        FieldPath::new(output)?,
        FieldPath::new(&[input])?,
    ))
}

fn vpc(cidr: &str) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(0),
        "ec2".into(),
        "CreateVpc".into(),
        "AWS::EC2::VPC".into(),
        FieldPath::new(&["Vpc", "VpcId"])?,
        vec![text("CidrBlock", cidr)?],
    )
    .with_response_checks(vec![check(&["Vpc", "CidrBlock"], "CidrBlock")?]))
}

fn subnet(id: u16, cidr: &str, zone: Option<&str>) -> Result<SeedStep, SeedError> {
    let zone_input = zone
        .map(|value| text("AvailabilityZone", value))
        .transpose()?;
    let zone_check = zone
        .map(|_| check(&["Subnet", "AvailabilityZone"], "AvailabilityZone"))
        .transpose()?;
    let inputs = [
        previous("VpcId", 0, &["Vpc", "VpcId"])?,
        text("CidrBlock", cidr)?,
    ]
    .into_iter()
    .chain(zone_input)
    .collect();
    let checks = [
        check(&["Subnet", "VpcId"], "VpcId")?,
        check(&["Subnet", "CidrBlock"], "CidrBlock")?,
    ]
    .into_iter()
    .chain(zone_check)
    .collect();
    Ok(SeedStep::new(
        StepId::new(id),
        "ec2".into(),
        "CreateSubnet".into(),
        "AWS::EC2::Subnet".into(),
        FieldPath::new(&["Subnet", "SubnetId"])?,
        inputs,
    )
    .with_response_checks(checks))
}

fn security_group(id: u16, name: &str) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(id),
        "ec2".into(),
        "CreateSecurityGroup".into(),
        "AWS::EC2::SecurityGroup".into(),
        FieldPath::new(&["GroupId"])?,
        vec![
            previous("VpcId", 0, &["Vpc", "VpcId"])?,
            text("GroupName", name)?,
            text(
                "Description",
                "Local emulator inventory seed security group",
            )?,
        ],
    ))
}

fn route_table() -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(3),
        "ec2".into(),
        "CreateRouteTable".into(),
        "AWS::EC2::RouteTable".into(),
        FieldPath::new(&["RouteTable", "RouteTableId"])?,
        vec![previous("VpcId", 0, &["Vpc", "VpcId"])?],
    )
    .with_response_checks(vec![check(&["RouteTable", "VpcId"], "VpcId")?]))
}

fn subnet_edge(source: u16) -> Result<SeedRelationship, SeedError> {
    Ok(SeedRelationship::new(
        StepId::new(source),
        StepId::new(0),
        "IN_VPC".into(),
        FieldPath::new(&["Subnet", "VpcId"])?,
    ))
}

fn bound_list(field: &str, sources: &[(u16, &[&str])]) -> Result<SeedInput, SeedError> {
    let bindings = sources
        .iter()
        .enumerate()
        .map(|(index, (step, path))| {
            Ok(SeedNestedBinding {
                destination: vec![SeedValuePart::Index(index)],
                step: StepId::new(*step),
                path: FieldPath::new(path)?,
            })
        })
        .collect::<Result<Vec<_>, SeedError>>()?;
    input(
        field,
        SeedInputValue::Template(SeedTemplate {
            value: Value::List(sources.iter().map(|_| Value::Null).collect()),
            bindings,
        }),
    )
}

fn load_balancer() -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(4),
        "elastic-load-balancing-v2".into(),
        "CreateLoadBalancer".into(),
        "AWS::ElasticLoadBalancingV2::LoadBalancer".into(),
        FieldPath::new(&["LoadBalancers", "0", "LoadBalancerArn"])?,
        vec![
            text("Name", ALB_NAME)?,
            bound_list(
                "Subnets",
                &[(1, &["Subnet", "SubnetId"]), (2, &["Subnet", "SubnetId"])],
            )?,
            bound_list("SecurityGroups", &[(3, &["GroupId"])])?,
            text("Scheme", "internal")?,
            text("Type", "application")?,
        ],
    )
    .with_response_checks(vec![
        check(&["LoadBalancers", "0", "LoadBalancerName"], "Name")?,
        check(&["LoadBalancers", "0", "SecurityGroups"], "SecurityGroups")?,
        check(&["LoadBalancers", "0", "Scheme"], "Scheme")?,
        check(&["LoadBalancers", "0", "Type"], "Type")?,
    ]))
}

fn target_group() -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(5),
        "elastic-load-balancing-v2".into(),
        "CreateTargetGroup".into(),
        "AWS::ElasticLoadBalancingV2::TargetGroup".into(),
        FieldPath::new(&["TargetGroups", "0", "TargetGroupArn"])?,
        vec![
            text("Name", TARGET_GROUP_NAME)?,
            text("Protocol", "HTTP")?,
            input("Port", SeedInputValue::Integer(80))?,
            previous("VpcId", 0, &["Vpc", "VpcId"])?,
            text("TargetType", "ip")?,
        ],
    )
    .with_response_checks(vec![
        check(&["TargetGroups", "0", "TargetGroupName"], "Name")?,
        check(&["TargetGroups", "0", "Protocol"], "Protocol")?,
        check(&["TargetGroups", "0", "Port"], "Port")?,
        check(&["TargetGroups", "0", "VpcId"], "VpcId")?,
        check(&["TargetGroups", "0", "TargetType"], "TargetType")?,
    ]))
}

fn alarm_identity() -> Result<SeedIdentity, SeedError> {
    Ok(SeedIdentity::Request {
        input_field: FieldPath::new(&["AlarmName"])?,
        template: Some("arn:{partition}:cloudwatch:{region}:{account}:alarm:{value}".into()),
    })
}

fn metric_alarm() -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(0),
        "cloudwatch".into(),
        "PutMetricAlarm".into(),
        "AWS::CloudWatch::Alarm".into(),
        alarm_identity()?,
        vec![
            text("AlarmName", CPU_ALARM_NAME)?,
            text("AlarmDescription", "Local emulator CPU alarm configuration")?,
            input("ActionsEnabled", SeedInputValue::Boolean(false))?,
            input("Metrics", SeedInputValue::Literal(cpu_metric()))?,
            input("EvaluationPeriods", SeedInputValue::Integer(1))?,
            input(
                "Threshold",
                SeedInputValue::Literal(Value::Float(Float::new(80.0))),
            )?,
            text("ComparisonOperator", "GreaterThanThreshold")?,
            text("TreatMissingData", "notBreaching")?,
        ],
    ))
}

fn composite_alarm() -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(1),
        "cloudwatch".into(),
        "PutCompositeAlarm".into(),
        "AWS::CloudWatch::CompositeAlarm".into(),
        alarm_identity()?,
        vec![
            text("AlarmName", COMPOSITE_ALARM_NAME)?,
            text(
                "AlarmDescription",
                "Local emulator composite of the seeded CPU alarm",
            )?,
            input("ActionsEnabled", SeedInputValue::Boolean(false))?,
            text("AlarmRule", "ALARM(\"inventory-seed-cpu\")")?,
        ],
    ))
}

fn object<const N: usize>(fields: [(&str, Value); N]) -> Value {
    Value::Map(
        fields
            .into_iter()
            .map(|(name, value)| (Key::Name(name.into()), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn cpu_metric() -> Value {
    Value::List(vec![object([
        ("Id", Value::Text("cpu".into())),
        (
            "MetricStat",
            object([
                (
                    "Metric",
                    object([
                        ("Namespace", Value::Text("AWS/EC2".into())),
                        ("MetricName", Value::Text("CPUUtilization".into())),
                    ]),
                ),
                ("Period", Value::Int(60)),
                ("Stat", Value::Text("Average".into())),
            ]),
        ),
        ("ReturnData", Value::Bool(true)),
    ])])
}
