//! Native kind identity and ARN templates for reviewed types.

#[rustfmt::skip]
pub const NATIVE: &[(&str, &str)] = &[
    ("AWS::DynamoDB::Table", "aws_dynamodb_table"),
    ("AWS::EC2::Instance", "aws_ec2_instance"),
    ("AWS::EC2::InternetGateway", "aws_vpc_internet_gateway"),
    ("AWS::EC2::NatGateway", "aws_vpc_nat_gateway"),
    ("AWS::EC2::NetworkAcl", "aws_vpc_network_acl"),
    ("AWS::EC2::NetworkInterface", "aws_ec2_network_interface"),
    ("AWS::EC2::RouteTable", "aws_vpc_route_table"),
    ("AWS::EC2::SecurityGroup", "aws_vpc_security_group"),
    ("AWS::EC2::Subnet", "aws_vpc_subnet"),
    ("AWS::EC2::VPC", "aws_vpc"),
    ("AWS::EC2::VPCEndpoint", "aws_vpc_endpoint"),
    (
        "AWS::EC2::VPCPeeringConnection",
        "aws_vpc_peering_connection",
    ),
    ("AWS::EC2::Volume", "aws_ebs_volume"),
    ("AWS::ECS::Cluster", "aws_ecs_cluster"),
    ("AWS::ECS::Service", "aws_ecs_service"),
    ("AWS::ECS::TaskDefinition", "aws_ecs_task_definition"),
    ("AWS::EKS::Cluster", "aws_eks_cluster"),
    (
        "AWS::ElasticLoadBalancingV2::LoadBalancer",
        "aws_elbv2_load_balancer",
    ),
    (
        "AWS::ElasticLoadBalancingV2::TargetGroup",
        "aws_elbv2_target_group",
    ),
    ("AWS::Lambda::Function", "aws_lambda_function"),
];

#[rustfmt::skip]
pub const ARN_TEMPLATES: &[(&str, &str)] = &[
    (
        "AWS::DynamoDB::Table",
        "arn:{partition}:dynamodb:{region}:{account}:table/{identifier}",
    ),
    (
        "AWS::EC2::CapacityReservation",
        "arn:{partition}:ec2:{region}:{account}:capacity-reservation/{identifier}",
    ),
    (
        "AWS::EC2::CustomerGateway",
        "arn:{partition}:ec2:{region}:{account}:customer-gateway/{identifier}",
    ),
    (
        "AWS::EC2::DHCPOptions",
        "arn:{partition}:ec2:{region}:{account}:dhcp-options/{identifier}",
    ),
    (
        "AWS::EC2::EIP",
        "arn:{partition}:ec2:{region}:{account}:elastic-ip/{identifier}",
    ),
    (
        "AWS::EC2::EgressOnlyInternetGateway",
        "arn:{partition}:ec2:{region}:{account}:egress-only-internet-gateway/{identifier}",
    ),
    (
        "AWS::EC2::Host",
        "arn:{partition}:ec2:{region}:{account}:dedicated-host/{identifier}",
    ),
    (
        "AWS::EC2::Instance",
        "arn:{partition}:ec2:{region}:{account}:instance/{identifier}",
    ),
    (
        "AWS::EC2::InstanceConnectEndpoint",
        "arn:{partition}:ec2:{region}:{account}:instance-connect-endpoint/{identifier}",
    ),
    (
        "AWS::EC2::InternetGateway",
        "arn:{partition}:ec2:{region}:{account}:internet-gateway/{identifier}",
    ),
    (
        "AWS::EC2::LaunchTemplate",
        "arn:{partition}:ec2:{region}:{account}:launch-template/{identifier}",
    ),
    (
        "AWS::EC2::NatGateway",
        "arn:{partition}:ec2:{region}:{account}:natgateway/{identifier}",
    ),
    (
        "AWS::EC2::NetworkAcl",
        "arn:{partition}:ec2:{region}:{account}:network-acl/{identifier}",
    ),
    (
        "AWS::EC2::NetworkInterface",
        "arn:{partition}:ec2:{region}:{account}:network-interface/{identifier}",
    ),
    (
        "AWS::EC2::PrefixList",
        "arn:{partition}:ec2:{region}:{account}:prefix-list/{identifier}",
    ),
    (
        "AWS::EC2::RouteTable",
        "arn:{partition}:ec2:{region}:{account}:route-table/{identifier}",
    ),
    (
        "AWS::EC2::SecurityGroup",
        "arn:{partition}:ec2:{region}:{account}:security-group/{identifier}",
    ),
    (
        "AWS::EC2::Snapshot",
        "arn:{partition}:ec2:{region}::snapshot/{identifier}",
    ),
    (
        "AWS::EC2::Subnet",
        "arn:{partition}:ec2:{region}:{account}:subnet/{identifier}",
    ),
    (
        "AWS::EC2::TransitGateway",
        "arn:{partition}:ec2:{region}:{account}:transit-gateway/{identifier}",
    ),
    (
        "AWS::EC2::TransitGatewayPeeringAttachment",
        "arn:{partition}:ec2:{region}:{account}:transit-gateway-attachment/{identifier}",
    ),
    (
        "AWS::EC2::TransitGatewayRouteTable",
        "arn:{partition}:ec2:{region}:{account}:transit-gateway-route-table/{identifier}",
    ),
    (
        "AWS::EC2::TransitGatewayVpcAttachment",
        "arn:{partition}:ec2:{region}:{account}:transit-gateway-attachment/{identifier}",
    ),
    (
        "AWS::EC2::VPC",
        "arn:{partition}:ec2:{region}:{account}:vpc/{identifier}",
    ),
    (
        "AWS::EC2::VPCEndpoint",
        "arn:{partition}:ec2:{region}:{account}:vpc-endpoint/{identifier}",
    ),
    (
        "AWS::EC2::VPCPeeringConnection",
        "arn:{partition}:ec2:{region}:{account}:vpc-peering-connection/{identifier}",
    ),
    (
        "AWS::EC2::VPNConnection",
        "arn:{partition}:ec2:{region}:{account}:vpn-connection/{identifier}",
    ),
    (
        "AWS::EC2::VPNGateway",
        "arn:{partition}:ec2:{region}:{account}:vpn-gateway/{identifier}",
    ),
    (
        "AWS::EC2::Volume",
        "arn:{partition}:ec2:{region}:{account}:volume/{identifier}",
    ),
    (
        "AWS::ECS::Cluster",
        "arn:{partition}:ecs:{region}:{account}:cluster/{identifier}",
    ),
    (
        "AWS::EKS::Cluster",
        "arn:{partition}:eks:{region}:{account}:cluster/{identifier}",
    ),
    (
        "AWS::Lambda::Function",
        "arn:{partition}:lambda:{region}:{account}:function:{identifier}",
    ),
];

#[rustfmt::skip]
pub const SELF_ARNS: &[(&str, &str)] = &[
    ("AWS::ApiGateway::DomainName", "/DomainNameArn"),
    ("AWS::ApiGatewayV2::DomainName", "/DomainNameArn"),
    ("AWS::AutoScaling::AutoScalingGroup", "/AutoScalingGroupARN"),
    ("AWS::Backup::BackupPlan", "/BackupPlanArn"),
    ("AWS::Backup::BackupVault", "/BackupVaultArn"),
    ("AWS::Backup::RestoreTestingPlan", "/RestoreTestingPlanArn"),
    ("AWS::Batch::JobDefinition", "/JobDefinitionArn"),
    (
        "AWS::Config::ConfigurationAggregator",
        "/ConfigurationAggregatorArn",
    ),
    ("AWS::Config::ConfigurationRecorder", "/ResourceARN"),
    ("AWS::EC2::CapacityReservation", "/CapacityReservationArn"),
    (
        "AWS::EC2::InstanceConnectEndpoint",
        "/InstanceConnectEndpointArn",
    ),
    ("AWS::EC2::NetworkInsightsPath", "/NetworkInsightsPathArn"),
    ("AWS::EC2::TransitGateway", "/TransitGatewayArn"),
    ("AWS::FSx::DataRepositoryAssociation", "/ResourceARN"),
    ("AWS::FSx::StorageVirtualMachine", "/ResourceARN"),
    ("AWS::FSx::Volume", "/ResourceARN"),
    ("AWS::Lambda::EventSourceMapping", "/EventSourceMappingArn"),
    ("AWS::RDS::DBCluster", "/DBClusterArn"),
    ("AWS::RDS::DBInstance", "/DBInstanceArn"),
    ("AWS::RDS::DBParameterGroup", "/DBParameterGroupArn"),
    ("AWS::RDS::DBProxy", "/DBProxyArn"),
    ("AWS::RDS::DBProxyEndpoint", "/DBProxyEndpointArn"),
    ("AWS::RDS::DBSubnetGroup", "/DBSubnetGroupArn"),
    ("AWS::S3::AccessGrantsLocation", "/AccessGrantsLocationArn"),
];
