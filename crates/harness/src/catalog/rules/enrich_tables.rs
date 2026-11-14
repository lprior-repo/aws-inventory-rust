pub type Ec2Identifier = (&'static str, &'static str, &'static str, &'static str, bool);

#[rustfmt::skip]
pub const EC2_IDENTIFIERS: &[(&str, Ec2Identifier)] = &[
    ("vpcid", ("IN_VPC", "aws_vpc", "vpc", "vpc-", true)),
    ("subnetid", ("IN_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", true)),
    ("subnetids", ("USES_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", true)),
    ("subnets", ("USES_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", true)),
    ("vpcsubnetids", ("USES_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", true)),
    ("clientsubnetids", ("USES_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", true)),
    ("securitygroupid", ("USES_SECURITY_GROUP", "aws_vpc_security_group", "security-group", "sg-", true)),
    ("securitygroupids", ("USES_SECURITY_GROUP", "aws_vpc_security_group", "security-group", "sg-", true)),
    ("securitygroups", ("USES_SECURITY_GROUP", "aws_vpc_security_group", "security-group", "sg-", true)),
    ("vpcsecuritygroupids", ("USES_SECURITY_GROUP", "aws_vpc_security_group", "security-group", "sg-", true)),
    ("networkinterfaceid", ("USES_NETWORK_INTERFACE", "aws_ec2_network_interface", "network-interface", "eni-", true)),
    ("instanceid", ("USES_INSTANCE", "aws_ec2_instance", "instance", "i-", true)),
    ("volumeid", ("USES_VOLUME", "aws_ebs_volume", "volume", "vol-", true)),
    ("transitgatewayid", ("USES_TRANSIT_GATEWAY", "aws_ec2_transit_gateway", "transit-gateway", "tgw-", true)),
    ("routetableid", ("USES_ROUTE_TABLE", "aws_vpc_route_table", "route-table", "rtb-", true)),
    ("networkaclid", ("USES_NETWORK_ACL", "aws_vpc_network_acl", "network-acl", "acl-", true)),
    ("internetgatewayid", ("USES_INTERNET_GATEWAY", "aws_vpc_internet_gateway", "internet-gateway", "igw-", true)),
    ("dhcpoptionsid", ("USES_DHCP_OPTIONS", "aws_ec2_dhcp_options", "dhcp-options", "dopt-", true)),
    ("vpcendpointid", ("USES_VPC_ENDPOINT", "aws_vpc_endpoint", "vpc-endpoint", "vpce-", true)),
    ("natgatewayid", ("USES_NAT_GATEWAY", "aws_vpc_nat_gateway", "natgateway", "nat-", true)),
    ("allocationid", ("USES_ELASTIC_IP", "aws_ec2_eip", "elastic-ip", "eipalloc-", true)),
    ("snapshotid", ("USES_SNAPSHOT", "aws_ec2_snapshot", "snapshot", "snap-", false)),
    ("imageid", ("USES_IMAGE", "aws_ec2_image", "image", "ami-", false)),
];

#[rustfmt::skip]
pub const ARN_TARGETS: &[(&str, (&str, Option<&str>))] = &[
    ("aws_certificate_manager_certificate", ("acm", Some("certificate/"))),
    ("aws_ecs_task_definition", ("ecs", Some("task-definition/"))),
    ("aws_elbv2_load_balancer", ("elasticloadbalancing", Some("loadbalancer/"))),
    ("aws_elbv2_target_group", ("elasticloadbalancing", Some("targetgroup/"))),
    ("aws_iam_role", ("iam", Some("role/"))),
    ("aws_kms_key", ("kms", Some("key/"))),
    ("aws_lambda_function", ("lambda", Some("function:"))),
    ("aws_secrets_manager_secret", ("secretsmanager", Some("secret:"))),
    ("aws_sns_topic", ("sns", None)),
    ("aws_sqs_queue", ("sqs", None)),
    ("aws_vpc_lattice_target_group", ("vpc-lattice", Some("targetgroup/"))),
];

#[rustfmt::skip]
pub const IDENTIFIER_PATH_EXCLUSIONS: &[(&str, &str)] = &[
    ("AWS::EC2::Instance", "/SecurityGroups/*"),
];

#[rustfmt::skip]
pub const ARN_PATH_OVERRIDES: &[(&str, &str, &str, &str)] = &[
    ("AWS::ApiGateway::DomainName", "/CertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::ApiGateway::DomainName", "/OwnershipVerificationCertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::ApiGateway::DomainName", "/RegionalCertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::ApiGateway::DomainNameV2", "/CertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::ApiGatewayV2::DomainName", "/DomainNameConfigurations/*/CertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::ApiGatewayV2::DomainName", "/DomainNameConfigurations/*/OwnershipVerificationCertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::EC2::ClientVpnEndpoint", "/ServerCertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::EC2::CustomerGateway", "/CertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::EC2::VerifiedAccessEndpoint", "/DomainCertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
    ("AWS::ECS::Service", "/VpcLatticeConfigurations/*/AdvancedConfiguration/AlternateTargetGroupArn", "USES_TARGET_GROUP", "aws_vpc_lattice_target_group"),
    ("AWS::ECS::Service", "/VpcLatticeConfigurations/*/TargetGroupArn", "USES_TARGET_GROUP", "aws_vpc_lattice_target_group"),
    ("AWS::MSK::Replicator", "/KafkaClusters/*/ClientAuthentication/SaslOAuthBearer/TokenEndpointTlsCertificateArn", "USES_SECRET", "aws_secrets_manager_secret"),
    ("AWS::VpcLattice::Service", "/CertificateArn", "USES_CERTIFICATE", "aws_certificate_manager_certificate"),
];

#[rustfmt::skip]
pub const ADDITIONAL_SENSITIVE: &[(&str, &[&str])] = &[
    ("AWS::ApiGateway::ApiKey", &["/Value"]),
    ("AWS::ApiGateway::ClientCertificate", &["/PemEncodedCertificate"]),
    ("AWS::Bedrock::Agent", &["/Instruction", "/PromptOverrideConfiguration"]),
    ("AWS::Bedrock::AutomatedReasoningPolicy", &["/PolicyDefinition"]),
    ("AWS::Bedrock::Flow", &["/Definition", "/DefinitionString", "/DefinitionSubstitutions"]),
    ("AWS::Bedrock::Prompt", &["/Variants"]),
    ("AWS::CloudFormation::Stack", &["/Parameters", "/Outputs", "/TemplateBody", "/TemplateURL"]),
    ("AWS::Glue::Blueprint", &["/BlueprintLocation"]),
    ("AWS::Glue::CustomEntityType", &["/RegexString", "/ContextWords"]),
    ("AWS::Glue::DataQualityRuleset", &["/Ruleset"]),
    ("AWS::Glue::Schema", &["/SchemaDefinition"]),
    ("AWS::Logs::ScheduledQuery", &["/QueryLanguage", "/QueryString"]),
    ("AWS::SageMaker::ModelCard", &["/Content"]),
    ("AWS::SageMaker::MonitoringSchedule", &["/MonitoringScheduleConfig/MonitoringJobDefinition/Environment"]),
    ("AWS::SageMaker::Pipeline", &["/PipelineDefinition", "/PipelineDefinitionS3Location"]),
];

#[rustfmt::skip]
pub const SENSITIVE_NODE_NAMES: &[&str] = &[
    "apikey",
    "apikeyvalue",
    "args",
    "authtoken",
    "certificatepem",
    "command",
    "content",
    "credential",
    "credentials",
    "definition",
    "definitionstring",
    "domainsigningprivatekey",
    "environment",
    "environmentvariables",
    "keymaterial",
    "password",
    "passwords",
    "privatekey",
    "privatekeycertificate",
    "script",
    "scriptbody",
    "secret",
    "secretbinary",
    "secretstring",
    "templatebody",
    "templatedata",
    "token",
    "userdata",
];
