type RelationshipSeed = (
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
);

#[rustfmt::skip]
pub const RELATIONSHIPS: &[(&str, &[RelationshipSeed])] = &[
    ("AWS::AppRunner::VpcConnector", &[
        ("/Subnets/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/SecurityGroups/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::Batch::ComputeEnvironment", &[
        ("/ServiceRole", "ASSUMES_ROLE", "aws_iam_role", None),
        ("/ComputeResources/Subnets/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/ComputeResources/SecurityGroupIds/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::CodeBuild::Project", &[
        ("/ServiceRole", "ASSUMES_ROLE", "aws_iam_role", None),
        ("/VpcConfig/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
        ("/VpcConfig/Subnets/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/VpcConfig/SecurityGroupIds/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::CodePipeline::Pipeline", &[
        ("/RoleArn", "ASSUMES_ROLE", "aws_iam_role", None),
    ]),
    ("AWS::DMS::ReplicationSubnetGroup", &[
        ("/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
    ]),
    ("AWS::DMS::ReplicationTask", &[
        ("/SourceEndpointArn", "READS_FROM", "aws_dms_endpoint", None),
        ("/TargetEndpointArn", "DELIVERS_TO", "aws_dms_endpoint", None),
    ]),
    ("AWS::EC2::Instance", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
        ("/SubnetId", "IN_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/SecurityGroupIds/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::EC2::NatGateway", &[
        ("/SubnetId", "IN_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
    ]),
    ("AWS::EC2::NetworkAcl", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
    ]),
    ("AWS::EC2::NetworkInterface", &[
        ("/SubnetId", "IN_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/GroupSet/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::EC2::RouteTable", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
    ]),
    ("AWS::EC2::SecurityGroup", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
    ]),
    ("AWS::EC2::Subnet", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
    ]),
    ("AWS::EC2::TransitGatewayVpcAttachment", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
        ("/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
    ]),
    ("AWS::EC2::VPCEndpoint", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
        ("/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/SecurityGroupIds/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::ECS::Service", &[
        ("/TaskDefinition", "USES_TASK_DEFINITION", "aws_ecs_task_definition", None),
        ("/LoadBalancers/*/TargetGroupArn", "USES_TARGET_GROUP", "aws_elbv2_target_group", None),
        ("/NetworkConfiguration/AwsvpcConfiguration/Subnets/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/NetworkConfiguration/AwsvpcConfiguration/SecurityGroups/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::ECS::TaskDefinition", &[
        ("/TaskRoleArn", "USES_TASK_ROLE", "aws_iam_role", None),
        ("/ExecutionRoleArn", "USES_EXECUTION_ROLE", "aws_iam_role", None),
    ]),
    ("AWS::EKS::Cluster", &[
        ("/RoleArn", "ASSUMES_ROLE", "aws_iam_role", None),
        ("/ResourcesVpcConfig/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/ResourcesVpcConfig/SecurityGroupIds/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::ElastiCache::SubnetGroup", &[
        ("/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
    ]),
    ("AWS::ElasticLoadBalancingV2::LoadBalancer", &[
        ("/Subnets/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/SubnetMappings/*/SubnetId", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/SecurityGroups/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::ElasticLoadBalancingV2::TargetGroup", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
        ("/LoadBalancerArns/*", "TARGET_GROUP_OF", "aws_elbv2_load_balancer", None),
    ]),
    ("AWS::Events::Connection", &[
        ("/SecretArn", "USES_SECRET", "aws_secrets_manager_secret", None),
    ]),
    ("AWS::Events::Rule", &[
        ("/Targets/*/Arn", "DELIVERS_TO", "aws_reference", None),
        ("/Targets/*/RoleArn", "ASSUMES_ROLE", "aws_iam_role", None),
    ]),
    ("AWS::Lambda::Function", &[
        ("/Role", "ASSUMES_ROLE", "aws_iam_role", None),
        ("/VpcConfig/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/VpcConfig/SecurityGroupIds/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
        ("/KmsKeyArn", "ENCRYPTED_BY", "aws_kms_key", None),
        ("/DeadLetterConfig/TargetArn", "DELIVERS_TO", "aws_reference", None),
    ]),
    ("AWS::Logs::Destination", &[
        ("/RoleArn", "ASSUMES_ROLE", "aws_iam_role", None),
        ("/TargetArn", "DELIVERS_TO", "aws_reference", None),
    ]),
    ("AWS::Logs::LogGroup", &[
        ("/KmsKeyId", "ENCRYPTED_BY", "aws_kms_key", None),
    ]),
    ("AWS::MSK::Cluster", &[
        ("/BrokerNodeGroupInfo/ClientSubnets/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/BrokerNodeGroupInfo/SecurityGroups/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::MemoryDB::SubnetGroup", &[
        ("/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
    ]),
    ("AWS::Neptune::DBSubnetGroup", &[
        ("/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
    ]),
    ("AWS::NetworkFirewall::Firewall", &[
        ("/VpcId", "IN_VPC", "aws_vpc", Some("arn:{partition}:ec2:{region}:{account}:vpc/{identifier}")),
        ("/SubnetMappings/*/SubnetId", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/FirewallPolicyArn", "USES_FIREWALL_POLICY", "aws_network_firewall_firewall_policy", None),
    ]),
    ("AWS::Pipes::Pipe", &[
        ("/RoleArn", "ASSUMES_ROLE", "aws_iam_role", None),
        ("/Source", "READS_FROM", "aws_reference", None),
        ("/Target", "DELIVERS_TO", "aws_reference", None),
        ("/Enrichment", "ENRICHED_BY", "aws_reference", None),
    ]),
    ("AWS::RDS::DBProxy", &[
        ("/RoleArn", "ASSUMES_ROLE", "aws_iam_role", None),
        ("/VpcSubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/VpcSecurityGroupIds/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
        ("/Auth/*/SecretArn", "USES_SECRET", "aws_secrets_manager_secret", None),
    ]),
    ("AWS::RDS::DBSubnetGroup", &[
        ("/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
    ]),
    ("AWS::Redshift::ClusterSubnetGroup", &[
        ("/SubnetIds/*", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
    ]),
    ("AWS::Route53Resolver::ResolverEndpoint", &[
        ("/IpAddresses/*/SubnetId", "USES_SUBNET", "aws_vpc_subnet", Some("arn:{partition}:ec2:{region}:{account}:subnet/{identifier}")),
        ("/SecurityGroupIds/*", "USES_SECURITY_GROUP", "aws_vpc_security_group", Some("arn:{partition}:ec2:{region}:{account}:security-group/{identifier}")),
    ]),
    ("AWS::StepFunctions::StateMachine", &[
        ("/RoleArn", "ASSUMES_ROLE", "aws_iam_role", None),
    ]),
];
