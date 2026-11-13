#!/usr/bin/env python3
"""Reproduce the reviewed catalog from the retained official schema ZIP.

Acquisition/build-time helper only. Production discovery is Rust. Requires
jsonschema (verified with 4.26.0), and never calls an AWS account API.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import zipfile

from jsonschema import Draft7Validator
from cloudcontrol_enrichment import additions, enrich_rules, sensitive_paths

ROOT = Path(__file__).resolve().parents[2]
ARCHIVE = ROOT / "catalog/upstream/cloudformation-us-east-1-2026-11-12.zip"
EXPECTED_SHA256 = "8226b9cd82f432997fe3918b134ef155245636f35ac37f8006de5abe88dd54a8"

# Deliberately selected by architectural role, not the first 200 alphabetically.
# These types have regional enumeration. Global inventories, parent-model list
# handlers and legacy schemas without read/list handlers are excluded.
SELECTED = {
    "EC2": "Instance VPC Subnet SecurityGroup NetworkInterface Volume Snapshot InternetGateway NatGateway RouteTable NetworkAcl VPCEndpoint VPCPeeringConnection EgressOnlyInternetGateway EIP CustomerGateway VPNGateway VPNConnection TransitGateway TransitGatewayVpcAttachment TransitGatewayPeeringAttachment TransitGatewayRouteTable LaunchTemplate PlacementGroup CapacityReservation Host FlowLog PrefixList DHCPOptions InstanceConnectEndpoint NetworkInsightsPath TrafficMirrorSession TrafficMirrorTarget VPCEndpointService SecurityGroupIngress SecurityGroupEgress SubnetRouteTableAssociation SubnetNetworkAclAssociation",
    "ECS": "Cluster Service TaskDefinition CapacityProvider",
    "EKS": "Cluster",
    "ElasticLoadBalancingV2": "LoadBalancer TargetGroup",
    "Lambda": "Function EventSourceMapping CodeSigningConfig",
    "DynamoDB": "Table",
    "RDS": "DBInstance DBCluster DBSubnetGroup DBParameterGroup DBClusterParameterGroup DBProxy DBProxyEndpoint DBClusterEndpoint DBSnapshot OptionGroup",
    "ElastiCache": "CacheCluster ReplicationGroup SubnetGroup ParameterGroup ServerlessCache",
    "MemoryDB": "Cluster SubnetGroup ParameterGroup ACL",
    "Neptune": "DBCluster DBInstance DBSubnetGroup",
    "DocDB": "DBSubnetGroup DBClusterParameterGroup",
    "Redshift": "Cluster ClusterParameterGroup ClusterSubnetGroup EndpointAccess Snapshot",
    "RedshiftServerless": "Namespace Workgroup",
    "OpenSearchServerless": "Collection",
    "EFS": "FileSystem AccessPoint",
    "FSx": "Volume StorageVirtualMachine DataRepositoryAssociation",
    "S3": "AccessPoint AccessGrantsInstance AccessGrantsLocation",
    "S3Express": "DirectoryBucket",
    "S3Tables": "TableBucket",
    "ECR": "Repository RegistryScanningConfiguration ReplicationConfiguration",
    "AutoScaling": "AutoScalingGroup LaunchConfiguration",
    "ApplicationAutoScaling": "ScalableTarget",
    "Batch": "ComputeEnvironment JobDefinition JobQueue",
    "ElasticBeanstalk": "Application Environment",
    "ApiGateway": "RestApi DomainName VpcLink UsagePlan",
    "ApiGatewayV2": "Api DomainName VpcLink",
    "Kinesis": "Stream StreamConsumer",
    "KinesisFirehose": "DeliveryStream",
    "KinesisAnalyticsV2": "Application",
    "SQS": "Queue",
    "SNS": "Topic Subscription",
    "Events": "EventBus Rule Archive ApiDestination Connection",
    "EventSchemas": "Registry Discoverer",
    "StepFunctions": "StateMachine Activity",
    "Logs": "LogGroup Destination ResourcePolicy QueryDefinition MetricFilter",
    "CloudWatch": "Alarm CompositeAlarm InsightRule MetricStream",
    "CloudTrail": "EventDataStore Channel",
    "KMS": "Key Alias",
    "SecretsManager": "Secret",
    "SSM": "Parameter Document Association MaintenanceWindow",
    "Backup": "BackupVault BackupPlan RestoreTestingPlan",
    "DMS": "Endpoint ReplicationSubnetGroup ReplicationTask ReplicationConfig",
    "MSK": "Cluster ServerlessCluster Configuration",
    "AmazonMQ": "Broker Configuration",
    "Route53Resolver": "ResolverEndpoint ResolverRule ResolverRuleAssociation ResolverQueryLoggingConfig FirewallDomainList FirewallRuleGroup",
    "NetworkFirewall": "Firewall FirewallPolicy RuleGroup",
    "ServiceDiscovery": "HttpNamespace PrivateDnsNamespace PublicDnsNamespace Service",
    "AppRunner": "Service VpcConnector VpcIngressConnection",
    "AppConfig": "Application DeploymentStrategy",
    "Athena": "WorkGroup DataCatalog",
    "Glue": "Database Crawler Job Workflow",
    "CodeBuild": "Project ReportGroup",
    "CodePipeline": "Pipeline",
    "CodeCommit": "Repository",
    "CodeArtifact": "Domain Repository",
    "Config": "ConfigurationAggregator ConfigurationRecorder ConfigRule DeliveryChannel",
    "AccessAnalyzer": "Analyzer",
    "CertificateManager": "Certificate",
    "ACMPCA": "CertificateAuthority",
    "GuardDuty": "Detector",
    "SecurityHub": "Hub",
    "Scheduler": "ScheduleGroup",
    "Pipes": "Pipe",
    "RAM": "ResourceShare",
    "VpcLattice": "Service ServiceNetwork TargetGroup",
}

# Primary AWS list-operations documentation explicitly requires context for
# these types, regardless of whether a particular schema snapshot encodes it.
MODEL_REQUIRED = {
    "AWS::ApiGateway::DocumentationVersion", "AWS::ApiGateway::Stage",
    "AWS::ApiGateway::Step", "AWS::CloudFormation::ResourceVersion",
    "AWS::CustomerProfiles::Integration", "AWS::CustomerProfiles::ObjectType",
    "AWS::EC2::TransitGatewayMulticastGroupMember", "AWS::EC2::TransitGatewayMulticastGroupSource",
    "AWS::ECS::TaskSet", "AWS::EKS::Addon", "AWS::EKS::AddOn", "AWS::EKS::FargateProfile",
    "AWS::ElasticLoadBalancingV2::Listener", "AWS::ElasticLoadBalancingV2::ListenerRule",
    "AWS::Glue::SchemaVersion", "AWS::Glue::SchemaVersionMetadata",
    "AWS::IoTSiteWise::AccessPolicy", "AWS::IoTSiteWise::Dashboard", "AWS::IoTSiteWise::Project",
    "AWS::Kendra::DataSource", "AWS::Kendra::Faq", "AWS::MediaConnect::FlowEntitlement",
    "AWS::MediaConnect::FlowOutput", "AWS::MediaConnect::FlowSource", "AWS::MediaConnect::FlowVpcInterface",
    "AWS::MediaPackage::Asset", "AWS::MediaPackage::PackagingConfiguration",
    "AWS::NetworkFirewall::LoggingConfiguration", "AWS::RDS::DBProxyTargetGroup",
    "AWS::S3Outposts::AccessPoint", "AWS::S3Outposts::Bucket",
    "AWS::SSO::Assignment", "AWS::SSO::InstanceAccessControlAttributeConfiguration", "AWS::SSO::PermissionSet",
    "AWS::WAFv2::WebACL",
    *(f"AWS::QuickSight::{name}" for name in ("Analysis", "Dashboard", "DataSet", "DataSource", "Template", "Theme")),
}

RESERVED = set("id arn name region account_id kind source source_priority observed_at properties tags resource_json scan_id resource_identifier provider_properties cloudformation_type".split())
GLOBAL_SENSITIVE = set("password masteruserpassword secretstring secretbinary secretaccesskey clientsecret privatekey apikeyvalue accesstoken refreshtoken sessiontoken authtoken authorizationtoken hectoken credentials userdata env environment environmentvariables command args entrypoint buildspec connectionstring connectionstrings code content definition definitionstring defaultarguments nonoverridablearguments invocationhttpparameters oauthhttpparameters".split())

SENSITIVE = {
    "AWS::EC2::Instance": ["/UserData", "/SsmAssociations"],
    "AWS::EC2::LaunchTemplate": ["/LaunchTemplateData/UserData"],
    "AWS::EC2::VPNConnection": ["/VpnTunnelOptionsSpecifications/*/PreSharedKey"],
    "AWS::ECS::TaskDefinition": ["/ContainerDefinitions/*/Environment", "/ContainerDefinitions/*/Command", "/ContainerDefinitions/*/EntryPoint", "/ContainerDefinitions/*/HealthCheck/Command", "/ContainerDefinitions/*/LogConfiguration/Options", "/ContainerDefinitions/*/FirelensConfiguration/Options", "/ProxyConfiguration/Properties"],
    "AWS::Lambda::Function": ["/Environment", "/Code", "/ImageConfig/Command", "/ImageConfig/EntryPoint"],
    "AWS::SSM::Parameter": ["/Value"],
    "AWS::SSM::Document": ["/Content"],
    "AWS::SSM::Association": ["/Parameters"],
    "AWS::SecretsManager::Secret": ["/SecretString", "/SecretBinary", "/GenerateSecretString"],
    "AWS::Events::Connection": ["/AuthParameters"],
    "AWS::Events::ApiDestination": ["/InvocationEndpoint"],
    "AWS::Events::Rule": ["/Targets/*/Input", "/Targets/*/InputTransformer/InputTemplate", "/Targets/*/HttpParameters"],
    "AWS::StepFunctions::StateMachine": ["/Definition", "/DefinitionString", "/DefinitionSubstitutions"],
    "AWS::ApiGateway::RestApi": ["/Body"],
    "AWS::ApiGatewayV2::Api": ["/Body"],
    "AWS::CodeBuild::Project": ["/Environment/EnvironmentVariables", "/Source/BuildSpec", "/Source/Auth", "/SecondarySources/*/BuildSpec", "/SecondarySources/*/Auth"],
    "AWS::CodePipeline::Pipeline": ["/Stages/*/Actions/*/Configuration", "/Variables"],
    "AWS::Glue::Job": ["/DefaultArguments", "/NonOverridableArguments"],
    "AWS::Glue::Workflow": ["/DefaultRunProperties"],
    "AWS::Glue::Database": ["/DatabaseInput/Parameters"],
    "AWS::Glue::Crawler": ["/Configuration"],
    "AWS::DMS::Endpoint": ["/ExtraConnectionAttributes"],
    "AWS::DMS::ReplicationTask": ["/ReplicationTaskSettings", "/TableMappings", "/SupplementalSettings"],
    "AWS::DMS::ReplicationConfig": ["/ReplicationSettings", "/SupplementalSettings", "/TableMappings"],
    "AWS::MSK::Configuration": ["/ServerProperties"],
    "AWS::AmazonMQ::Configuration": ["/Data"],
    "AWS::AmazonMQ::Broker": ["/Users/*/Password"],
    "AWS::ElasticBeanstalk::Environment": ["/OptionSettings"],
    "AWS::AppRunner::Service": ["/SourceConfiguration/CodeRepository/CodeConfiguration/CodeConfigurationValues/RuntimeEnvironmentVariables", "/SourceConfiguration/CodeRepository/CodeConfiguration/CodeConfigurationValues/StartCommand", "/SourceConfiguration/CodeRepository/CodeConfiguration/CodeConfigurationValues/BuildCommand", "/SourceConfiguration/ImageRepository/ImageConfiguration/RuntimeEnvironmentVariables", "/SourceConfiguration/ImageRepository/ImageConfiguration/StartCommand"],
    "AWS::Athena::DataCatalog": ["/Parameters"],
    "AWS::Athena::WorkGroup": ["/WorkGroupConfiguration/EngineConfiguration/AdditionalConfigs"],
    "AWS::Logs::QueryDefinition": ["/QueryString"],
    "AWS::KinesisAnalyticsV2::Application": ["/ApplicationConfiguration/ApplicationCodeConfiguration/CodeContent", "/ApplicationConfiguration/EnvironmentProperties"],
    "AWS::Pipes::Pipe": ["/EnrichmentParameters/InputTemplate", "/EnrichmentParameters/HttpParameters", "/TargetParameters/InputTemplate"],
    "AWS::Config::ConfigRule": ["/InputParameters"],
    "AWS::Batch::JobDefinition": ["/Parameters", "/ContainerProperties/LogConfiguration/Options", "/EcsProperties/TaskProperties/*/Containers/*/LogConfiguration/Options", "/NodeProperties/NodeRangeProperties/*/Container/LogConfiguration/Options", "/NodeProperties/NodeRangeProperties/*/EcsProperties/TaskProperties/*/Containers/*/LogConfiguration/Options"],
    "AWS::KinesisFirehose::DeliveryStream": ["/SplunkDestinationConfiguration/HECToken", "/HttpEndpointDestinationConfiguration/EndpointConfiguration/AccessKey"],
    "AWS::SNS::Subscription": ["/Endpoint"],
    "AWS::RDS::OptionGroup": ["/OptionConfigurations/*/OptionSettings/*/Value"],
}
for family in ("RDS", "ElastiCache", "MemoryDB", "Neptune", "DocDB", "Redshift"):
    for resource in ("DBParameterGroup", "DBClusterParameterGroup", "ParameterGroup", "ClusterParameterGroup"):
        SENSITIVE[f"AWS::{family}::{resource}"] = ["/Parameters"]

NATIVE = {
    "AWS::EC2::Instance": "aws_ec2_instance", "AWS::EC2::VPC": "aws_vpc",
    "AWS::EC2::Subnet": "aws_vpc_subnet", "AWS::EC2::SecurityGroup": "aws_vpc_security_group",
    "AWS::EC2::NetworkInterface": "aws_ec2_network_interface", "AWS::EC2::Volume": "aws_ebs_volume",
    "AWS::EC2::RouteTable": "aws_vpc_route_table", "AWS::EC2::InternetGateway": "aws_vpc_internet_gateway",
    "AWS::EC2::NatGateway": "aws_vpc_nat_gateway", "AWS::EC2::VPCEndpoint": "aws_vpc_endpoint",
    "AWS::EC2::NetworkAcl": "aws_vpc_network_acl", "AWS::EC2::VPCPeeringConnection": "aws_vpc_peering_connection",
    "AWS::DynamoDB::Table": "aws_dynamodb_table", "AWS::Lambda::Function": "aws_lambda_function",
    "AWS::ECS::Cluster": "aws_ecs_cluster", "AWS::ECS::Service": "aws_ecs_service",
    "AWS::ECS::TaskDefinition": "aws_ecs_task_definition", "AWS::EKS::Cluster": "aws_eks_cluster",
    "AWS::ElasticLoadBalancingV2::LoadBalancer": "aws_elbv2_load_balancer",
    "AWS::ElasticLoadBalancingV2::TargetGroup": "aws_elbv2_target_group",
}
EC2_ARN = {
    "Instance": "instance", "VPC": "vpc", "Subnet": "subnet", "SecurityGroup": "security-group",
    "NetworkInterface": "network-interface", "Volume": "volume", "Snapshot": "snapshot",
    "InternetGateway": "internet-gateway", "NatGateway": "natgateway", "RouteTable": "route-table",
    "NetworkAcl": "network-acl", "VPCEndpoint": "vpc-endpoint", "VPCPeeringConnection": "vpc-peering-connection",
    "EgressOnlyInternetGateway": "egress-only-internet-gateway", "EIP": "elastic-ip",
    "CustomerGateway": "customer-gateway", "VPNGateway": "vpn-gateway", "VPNConnection": "vpn-connection",
    "TransitGateway": "transit-gateway", "TransitGatewayVpcAttachment": "transit-gateway-attachment",
    "TransitGatewayPeeringAttachment": "transit-gateway-attachment", "TransitGatewayRouteTable": "transit-gateway-route-table",
    "LaunchTemplate": "launch-template", "CapacityReservation": "capacity-reservation", "Host": "dedicated-host",
    "PrefixList": "prefix-list", "DHCPOptions": "dhcp-options", "InstanceConnectEndpoint": "instance-connect-endpoint",
}
ARN_TEMPLATES = {
    **{f"AWS::EC2::{name}": f"arn:{{partition}}:ec2:{{region}}:{{account}}:{kind}/{{identifier}}" for name, kind in EC2_ARN.items()},
    "AWS::DynamoDB::Table": "arn:{partition}:dynamodb:{region}:{account}:table/{identifier}",
    "AWS::Lambda::Function": "arn:{partition}:lambda:{region}:{account}:function:{identifier}",
    "AWS::EKS::Cluster": "arn:{partition}:eks:{region}:{account}:cluster/{identifier}",
    "AWS::ECS::Cluster": "arn:{partition}:ecs:{region}:{account}:cluster/{identifier}",
}
# Unlike most EC2 resources, the official snapshot ARN grammar has no account
# segment. Keep this independent of the account-scoped EC2 template family.
ARN_TEMPLATES["AWS::EC2::Snapshot"] = "arn:{partition}:ec2:{region}::snapshot/{identifier}"
SELF_ARNS = {
    **{f"AWS::EC2::{name}": f"/{name}Arn" for name in ("TransitGateway", "CapacityReservation", "InstanceConnectEndpoint", "NetworkInsightsPath")},
    "AWS::Lambda::EventSourceMapping": "/EventSourceMappingArn",
    **{f"AWS::RDS::{name}": f"/{name}Arn" for name in ("DBInstance", "DBCluster", "DBSubnetGroup", "DBParameterGroup", "DBProxy", "DBProxyEndpoint")},
    **{f"AWS::FSx::{name}": "/ResourceARN" for name in ("Volume", "StorageVirtualMachine", "DataRepositoryAssociation")},
    "AWS::S3::AccessGrantsLocation": "/AccessGrantsLocationArn",
    "AWS::AutoScaling::AutoScalingGroup": "/AutoScalingGroupARN",
    "AWS::Batch::JobDefinition": "/JobDefinitionArn",
    "AWS::ApiGateway::DomainName": "/DomainNameArn",
    "AWS::ApiGatewayV2::DomainName": "/DomainNameArn",
    **{f"AWS::Backup::{name}": f"/{name}Arn" for name in ("BackupVault", "BackupPlan", "RestoreTestingPlan")},
    "AWS::Config::ConfigurationAggregator": "/ConfigurationAggregatorArn",
    "AWS::Config::ConfigurationRecorder": "/ResourceARN",
}


def snake(value: str) -> str:
    value = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", value)
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", value)
    return re.sub(r"[^a-z0-9]+", "_", value.lower()).strip("_")


def pointer(value: str) -> str:
    if not value.startswith("/properties/"):
        raise ValueError(f"unexpected schema pointer {value}")
    return value.removeprefix("/properties")


def basic_type(prop: dict, schema: dict) -> str:
    for _ in range(8):
        ref = prop.get("$ref")
        if not ref:
            break
        if not ref.startswith("#/"):
            return "mixed"
        target = schema
        for piece in ref[2:].split("/"):
            target = target[piece.replace("~1", "/").replace("~0", "~")]
        prop = target
    value = prop.get("type")
    return value if value in ("string", "number", "integer", "boolean", "object", "array", "null") else "mixed"


def relationship(path: str, relation: str, kind: str, arn: str | None = None) -> dict:
    return {"pointer": path, "relation": relation, "target_kind": kind,
            "target_format": "arn" if arn is None else "template", "arn_template": arn,
            "evidence": "provider_property_reference"}


def ec2_ref(path: str, relation: str, kind: str) -> dict:
    return relationship(path, relation, NATIVE[f"AWS::EC2::{kind}"], ARN_TEMPLATES[f"AWS::EC2::{kind}"])


RELATIONSHIPS = {
    "AWS::EC2::Instance": [ec2_ref("/VpcId", "IN_VPC", "VPC"), ec2_ref("/SubnetId", "IN_SUBNET", "Subnet"), ec2_ref("/SecurityGroupIds/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::EC2::Subnet": [ec2_ref("/VpcId", "IN_VPC", "VPC")],
    "AWS::EC2::SecurityGroup": [ec2_ref("/VpcId", "IN_VPC", "VPC")],
    "AWS::EC2::NetworkInterface": [ec2_ref("/SubnetId", "IN_SUBNET", "Subnet"), ec2_ref("/GroupSet/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::EC2::NatGateway": [ec2_ref("/SubnetId", "IN_SUBNET", "Subnet")],
    "AWS::EC2::RouteTable": [ec2_ref("/VpcId", "IN_VPC", "VPC")],
    "AWS::EC2::NetworkAcl": [ec2_ref("/VpcId", "IN_VPC", "VPC")],
    "AWS::EC2::VPCEndpoint": [ec2_ref("/VpcId", "IN_VPC", "VPC"), ec2_ref("/SubnetIds/*", "USES_SUBNET", "Subnet"), ec2_ref("/SecurityGroupIds/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::EC2::TransitGatewayVpcAttachment": [ec2_ref("/VpcId", "IN_VPC", "VPC"), ec2_ref("/SubnetIds/*", "USES_SUBNET", "Subnet")],
    "AWS::Lambda::Function": [relationship("/Role", "ASSUMES_ROLE", "aws_iam_role"), ec2_ref("/VpcConfig/SubnetIds/*", "USES_SUBNET", "Subnet"), ec2_ref("/VpcConfig/SecurityGroupIds/*", "USES_SECURITY_GROUP", "SecurityGroup"), relationship("/KmsKeyArn", "ENCRYPTED_BY", "aws_kms_key"), relationship("/DeadLetterConfig/TargetArn", "DELIVERS_TO", "aws_reference")],
    "AWS::EKS::Cluster": [relationship("/RoleArn", "ASSUMES_ROLE", "aws_iam_role"), ec2_ref("/ResourcesVpcConfig/SubnetIds/*", "USES_SUBNET", "Subnet"), ec2_ref("/ResourcesVpcConfig/SecurityGroupIds/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::ECS::Service": [relationship("/TaskDefinition", "USES_TASK_DEFINITION", "aws_ecs_task_definition"), relationship("/LoadBalancers/*/TargetGroupArn", "USES_TARGET_GROUP", "aws_elbv2_target_group"), ec2_ref("/NetworkConfiguration/AwsvpcConfiguration/Subnets/*", "USES_SUBNET", "Subnet"), ec2_ref("/NetworkConfiguration/AwsvpcConfiguration/SecurityGroups/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::ECS::TaskDefinition": [relationship("/TaskRoleArn", "USES_TASK_ROLE", "aws_iam_role"), relationship("/ExecutionRoleArn", "USES_EXECUTION_ROLE", "aws_iam_role")],
    "AWS::ElasticLoadBalancingV2::LoadBalancer": [ec2_ref("/Subnets/*", "USES_SUBNET", "Subnet"), ec2_ref("/SubnetMappings/*/SubnetId", "USES_SUBNET", "Subnet"), ec2_ref("/SecurityGroups/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::ElasticLoadBalancingV2::TargetGroup": [ec2_ref("/VpcId", "IN_VPC", "VPC"), relationship("/LoadBalancerArns/*", "TARGET_GROUP_OF", "aws_elbv2_load_balancer")],
    "AWS::RDS::DBSubnetGroup": [ec2_ref("/SubnetIds/*", "USES_SUBNET", "Subnet")],
    "AWS::RDS::DBProxy": [relationship("/RoleArn", "ASSUMES_ROLE", "aws_iam_role"), ec2_ref("/VpcSubnetIds/*", "USES_SUBNET", "Subnet"), ec2_ref("/VpcSecurityGroupIds/*", "USES_SECURITY_GROUP", "SecurityGroup"), relationship("/Auth/*/SecretArn", "USES_SECRET", "aws_secrets_manager_secret")],
    "AWS::ElastiCache::SubnetGroup": [ec2_ref("/SubnetIds/*", "USES_SUBNET", "Subnet")],
    "AWS::MemoryDB::SubnetGroup": [ec2_ref("/SubnetIds/*", "USES_SUBNET", "Subnet")],
    "AWS::Neptune::DBSubnetGroup": [ec2_ref("/SubnetIds/*", "USES_SUBNET", "Subnet")],
    "AWS::Redshift::ClusterSubnetGroup": [ec2_ref("/SubnetIds/*", "USES_SUBNET", "Subnet")],
    "AWS::Batch::ComputeEnvironment": [relationship("/ServiceRole", "ASSUMES_ROLE", "aws_iam_role"), ec2_ref("/ComputeResources/Subnets/*", "USES_SUBNET", "Subnet"), ec2_ref("/ComputeResources/SecurityGroupIds/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::Events::Rule": [relationship("/Targets/*/Arn", "DELIVERS_TO", "aws_reference"), relationship("/Targets/*/RoleArn", "ASSUMES_ROLE", "aws_iam_role")],
    "AWS::Events::Connection": [relationship("/SecretArn", "USES_SECRET", "aws_secrets_manager_secret")],
    "AWS::StepFunctions::StateMachine": [relationship("/RoleArn", "ASSUMES_ROLE", "aws_iam_role")],
    "AWS::Logs::LogGroup": [relationship("/KmsKeyId", "ENCRYPTED_BY", "aws_kms_key")],
    "AWS::Logs::Destination": [relationship("/RoleArn", "ASSUMES_ROLE", "aws_iam_role"), relationship("/TargetArn", "DELIVERS_TO", "aws_reference")],
    "AWS::DMS::ReplicationSubnetGroup": [ec2_ref("/SubnetIds/*", "USES_SUBNET", "Subnet")],
    "AWS::DMS::ReplicationTask": [relationship("/SourceEndpointArn", "READS_FROM", "aws_dms_endpoint"), relationship("/TargetEndpointArn", "DELIVERS_TO", "aws_dms_endpoint")],
    "AWS::MSK::Cluster": [ec2_ref("/BrokerNodeGroupInfo/ClientSubnets/*", "USES_SUBNET", "Subnet"), ec2_ref("/BrokerNodeGroupInfo/SecurityGroups/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::Route53Resolver::ResolverEndpoint": [ec2_ref("/IpAddresses/*/SubnetId", "USES_SUBNET", "Subnet"), ec2_ref("/SecurityGroupIds/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::NetworkFirewall::Firewall": [ec2_ref("/VpcId", "IN_VPC", "VPC"), ec2_ref("/SubnetMappings/*/SubnetId", "USES_SUBNET", "Subnet"), relationship("/FirewallPolicyArn", "USES_FIREWALL_POLICY", "aws_network_firewall_firewall_policy")],
    "AWS::AppRunner::VpcConnector": [ec2_ref("/Subnets/*", "USES_SUBNET", "Subnet"), ec2_ref("/SecurityGroups/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::CodeBuild::Project": [relationship("/ServiceRole", "ASSUMES_ROLE", "aws_iam_role"), ec2_ref("/VpcConfig/VpcId", "IN_VPC", "VPC"), ec2_ref("/VpcConfig/Subnets/*", "USES_SUBNET", "Subnet"), ec2_ref("/VpcConfig/SecurityGroupIds/*", "USES_SECURITY_GROUP", "SecurityGroup")],
    "AWS::CodePipeline::Pipeline": [relationship("/RoleArn", "ASSUMES_ROLE", "aws_iam_role")],
    "AWS::Pipes::Pipe": [relationship("/RoleArn", "ASSUMES_ROLE", "aws_iam_role"), relationship("/Source", "READS_FROM", "aws_reference"), relationship("/Target", "DELIVERS_TO", "aws_reference"), relationship("/Enrichment", "ENRICHED_BY", "aws_reference")],
}


def main() -> None:
    archive_bytes = ARCHIVE.read_bytes()
    if hashlib.sha256(archive_bytes).hexdigest() != EXPECTED_SHA256:
        raise ValueError("upstream archive checksum mismatch")
    types = []
    counts = {}
    all_count = eligible_count = 0
    with zipfile.ZipFile(ARCHIVE) as archive:
        schemas = {}
        for filename in archive.namelist():
            if filename.endswith(".json"):
                raw = archive.read(filename)
                schema = json.loads(raw)
                schemas[schema["typeName"]] = (filename, raw, schema)
                all_count += 1
                if {"read", "list"} <= schema.get("handlers", {}).keys():
                    eligible_count += 1
        added = additions()
        new_names = {f"AWS::{service}::{resource}" for service, resources in added.items() for resource in resources.split()}
        # Keep every original descriptor/column in its original order. Reviewed
        # additions follow them, so migration 003 never needs to be regenerated.
        for service, resources in [*SELECTED.items(), *added.items()]:
            for resource in resources.split():
                name = f"AWS::{service}::{resource}"
                filename, raw, schema = schemas[name]
                handlers = schema.get("handlers", {})
                if not {"read", "list"} <= handlers.keys() or name in MODEL_REQUIRED:
                    raise ValueError(f"not eligible: {name}")
                # Validate the complete schema, including oneOf/anyOf/allOf.
                # Empty {} never traverses property-only refs. Required refs
                # would fail closed if not resolvable by this validator.
                handler_schema = handlers["list"].get("handlerSchema", {})
                if not Draft7Validator(handler_schema).is_valid({}):
                    raise ValueError(f"list model required: {name}")
                properties = schema.get("properties", {})
                primary = [pointer(value) for value in schema["primaryIdentifier"]]
                write_only = sorted({pointer(value) for value in schema.get("writeOnlyProperties", [])})
                sensitive = sorted(set(SENSITIVE.get(name, [])) | {f"/{key}" for key in properties if snake(key).replace("_", "") in GLOBAL_SENSITIVE})
                if name in new_names:
                    sensitive = sorted(set(sensitive) | set(sensitive_paths(name, schema)))
                redacted_top = {value.removeprefix("/") for value in write_only + sensitive if value.count("/") == 1}
                columns = []
                column_names = set()
                for key, prop in sorted(properties.items()):
                    if key in redacted_top:
                        continue
                    column = snake(key)
                    if column in RESERVED:
                        column = "provider_" + column
                    if column in RESERVED:
                        column = "configuration_" + snake(key)
                    if column in RESERVED or column in column_names or not re.fullmatch(r"[a-z][a-z0-9_]*", column):
                        raise ValueError(f"invalid/duplicate column {name}:{column}")
                    column_names.add(column)
                    columns.append({"name": column, "pointer": "/" + key.replace("~", "~0").replace("/", "~1"), "json_type": basic_type(prop, schema)})
                if len(columns) > 200:
                    raise ValueError(f"too many columns: {name}")
                arns = {f"/{key}" for key in ("Arn", "ARN") if key in properties}
                arns.update(value for value in primary if value.lower().endswith("arn"))
                if name == "AWS::SecretsManager::Secret":
                    arns.add("/Id")
                if name in SELF_ARNS:
                    arns.add(SELF_ARNS[name])
                if name in new_names:
                    read_only = {pointer(value) for value in schema.get("readOnlyProperties", [])}
                    arns.update(f"/{key}" for key in (resource + "Arn", resource + "ARN", "ResourceArn", "ResourceARN")
                                if key in properties and f"/{key}" in read_only)
                name_pointers = [f"/{key}" for key in ("Name", resource + "Name") if key in properties]
                name_pointers += [value for value in primary if value.endswith("Name") and value not in name_pointers]
                kind = NATIVE.get(name, f"aws_{snake(service)}_{snake(resource)}")
                descriptor = {
                    "type_name": name, "kind": kind, "view_name": f"aws_cc_{snake(service)}_{snake(resource)}",
                    "service": snake(service), "scope": "regional", "home_region": None,
                    "read_supported": True, "list_supported": True, "list_requires_model": False,
                    "list_model_validation": "empty_object_accepted" if handler_schema else "no_handler_schema",
                    "primary_identifier": primary, "identifier_is_arn": len(primary) == 1 and (primary[0] in arns),
                    "arn_pointers": sorted(arns), "arn_template": ARN_TEMPLATES.get(name),
                    "arn_identifier_pointer": "/AllocationId" if name == "AWS::EC2::EIP" else (primary[0] if len(primary) == 1 and name in ARN_TEMPLATES else None),
                    "name_pointers": name_pointers,
                    "write_only_properties": write_only, "sensitive_properties": sensitive,
                    "relationships": RELATIONSHIPS.get(name, []),
                    "schema_file": filename, "schema_sha256": hashlib.sha256(raw).hexdigest(),
                    "read_permissions": sorted(set(handlers["read"].get("permissions", []))),
                    "list_permissions": sorted(set(handlers["list"].get("permissions", []))),
                    "columns": columns,
                }
                descriptor["relationships"] = enrich_rules(descriptor, schema, GLOBAL_SENSITIVE)
                types.append(descriptor)
                counts[service] = counts.get(service, 0) + 1
    if len(types) != 397 or len({item["type_name"] for item in types}) != 397:
        raise ValueError(f"expected exactly 397 reviewed unique types, got {len(types)}")
    evidence = {
        "source_kind": "official_aws_cloudformation_registry_schema_archive",
        "url": "https://schema.cloudformation.us-east-1.amazonaws.com/CloudformationSchema.zip",
        "archive_file": "upstream/" + ARCHIVE.name, "archive_sha256": EXPECTED_SHA256,
        "region": "us-east-1", "retrieved_at": "2026-11-12T15:31:46.572550+00:00",
        "archive_schema_count": all_count, "archive_read_and_list_handler_count": eligible_count,
        "selection": "397 reviewed regional types across architectural service families; read and list handlers; empty list model accepted; documented parent-model exceptions excluded",
        "list_model_exceptions_url": "https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-operations-list.html",
        "support_semantics_url": "https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-types.html",
        "schema_semantics_url": "https://docs.aws.amazon.com/cloudformation-cli/latest/userguide/resource-type-schema.html",
        "live_validated": False,
        "limitations": [
            "Pinned schema capabilities are not proof that every selected region, account, permission set or provider version will succeed.",
            "No claimed intersection with the supported-resources HTML table: direct acquisition returned a SiteUnavailable page and was discarded.",
            "Regional scope is a reviewed catalog decision; global IAM, CloudFront, Route53 and S3 Bucket enumeration are deliberately excluded.",
            "Readable provider properties can contain sensitive values; schema write-only plus explicit descriptor and common-key redaction applies before storage and edges.",
        ],
        "global_sensitive_property_names": sorted(GLOBAL_SENSITIVE),
    }
    output = {"schema_version": 1, "evidence": evidence, "resource_types": types}
    target = ROOT / "catalog/cloudcontrol-resources.json"
    target.write_text(json.dumps(output, indent=2) + "\n")
    selected = {descriptor["type_name"]: descriptor for descriptor in types}
    capabilities = []
    for name, (filename, raw, schema) in sorted(schemas.items()):
        handlers = schema.get("handlers", {})
        read, listing = "read" in handlers, "list" in handlers
        handler_schema = handlers.get("list", {}).get("handlerSchema", {})
        empty_model = listing and Draft7Validator(handler_schema).is_valid({})
        descriptor = selected.get(name)
        exclusion = ("missing_read_or_list_handler" if not (read and listing) else
                     "documented_parent_model_required" if name in MODEL_REQUIRED else
                     "schema_parent_model_required" if not empty_model else
                     "scope_unreviewed" if descriptor is None else None)
        family = name.split("::")[1]
        capabilities.append({
            "type_name": name, "service": snake(family), "service_name": family,
            "selected": descriptor is not None, "exclusion": exclusion,
            "scope_review": "reviewed_regional" if descriptor else "unreviewed",
            "read_supported": read, "list_supported": listing,
            "list_model_validation": ("no_list_handler" if not listing else
                                      "empty_object_rejected" if not empty_model else
                                      "empty_object_accepted" if handler_schema else "no_handler_schema"),
            "schema_file": filename, "schema_sha256": hashlib.sha256(raw).hexdigest(),
            "relationship_rule_count": len(descriptor["relationships"]) if descriptor else 0,
            "view_name": descriptor["view_name"] if descriptor else None,
            "emulator_verification": {"status": "not_run", "summary_file": None,
                                      "list_attempted": False, "get_attempted": False,
                                      "read_observed": False, "list_http_statuses": {}, "get_http_statuses": {}},
        })
    exclusions = {reason: sum(row["exclusion"] == reason for row in capabilities)
                  for reason in sorted({row["exclusion"] for row in capabilities if row["exclusion"]})}
    coverage = {"schema_version": 1,
                "evidence": {"archive_file": "upstream/" + ARCHIVE.name, "archive_sha256": EXPECTED_SHA256,
                             "source_url": evidence["url"], "catalog_sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
                             "selection_policy": evidence["selection"],
                             "warning": "Schema eligibility, reviewed mappings and executed provider evidence are separate claims."},
                "summary": {"schema_count": len(capabilities), "read_and_list_count": eligible_count,
                            "selected_count": len(types), "excluded_count": len(capabilities) - len(types),
                            "exclusions": exclusions, "relationship_rule_count": sum(len(row["relationships"]) for row in types),
                            "types_with_relationships": sum(bool(row["relationships"]) for row in types),
                            "emulator_list_attempted": 0, "emulator_read_observed": 0},
                "resource_types": capabilities}
    (ROOT / "catalog/cloudcontrol-capabilities.json").write_text(json.dumps(coverage, indent=2) + "\n")
    manifest = json.loads((ROOT / "catalog/upstream/cloudcontrol-upstream.json").read_text())
    manifest["sources"] = [entry for entry in manifest["sources"] if entry["file"] == ARCHIVE.name]
    manifest["discarded_acquisitions"] = [{"url": "https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/supported-resources.html", "reason": "HTTP 200 returned a 195-byte SiteUnavailable page; not retained or used as capability evidence"}]
    (ROOT / "catalog/upstream/cloudcontrol-upstream.json").write_text(json.dumps(manifest, indent=2) + "\n")
    invalid = ROOT / "catalog/upstream/cloudcontrol-supported-resources-2026-11-12.html"
    invalid.unlink(missing_ok=True)
    print(json.dumps({"types": len(types), "service_families": len(counts), "columns": sum(len(item["columns"]) for item in types), "max_columns": max(len(item["columns"]) for item in types), "relationship_rules": sum(len(item["relationships"]) for item in types), "catalog_bytes": target.stat().st_size, "by_service": counts}, indent=2))


if __name__ == "__main__":
    main()
