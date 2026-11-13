"""Bounded, schema-backed architecture references for the reviewed catalog.

This module never walks provider values or guesses identifiers from arbitrary
JSON keys. It resolves the retained schema's declared properties and emits
explicit runtime rules. Unknown/compound names remain opaque identifiers.
"""

from __future__ import annotations

import json
from pathlib import Path
import re

CATALOG = Path(__file__).resolve().parent.parent
ADDITIONS = CATALOG / "cloudcontrol-regional-additions.json"
MAX_SCHEMA_NODES = 40_000
MAX_SCHEMA_DEPTH = 32
MAX_RELATIONSHIP_RULES = 256

# The identifier grammar is checked again by the Rust adapter before an ARN is
# formed. Image/snapshot ARNs intentionally omit the account segment, as the
# official EC2 resource-ARN table specifies.
EC2_IDENTIFIERS = {
    "vpcid": ("IN_VPC", "aws_vpc", "vpc", "vpc-", True),
    "subnetid": ("IN_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", True),
    "subnetids": ("USES_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", True),
    "subnets": ("USES_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", True),
    "vpcsubnetids": ("USES_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", True),
    "clientsubnetids": ("USES_SUBNET", "aws_vpc_subnet", "subnet", "subnet-", True),
    "securitygroupid": ("USES_SECURITY_GROUP", "aws_vpc_security_group", "security-group", "sg-", True),
    "securitygroupids": ("USES_SECURITY_GROUP", "aws_vpc_security_group", "security-group", "sg-", True),
    "securitygroups": ("USES_SECURITY_GROUP", "aws_vpc_security_group", "security-group", "sg-", True),
    "vpcsecuritygroupids": ("USES_SECURITY_GROUP", "aws_vpc_security_group", "security-group", "sg-", True),
    "networkinterfaceid": ("USES_NETWORK_INTERFACE", "aws_ec2_network_interface", "network-interface", "eni-", True),
    "instanceid": ("USES_INSTANCE", "aws_ec2_instance", "instance", "i-", True),
    "volumeid": ("USES_VOLUME", "aws_ebs_volume", "volume", "vol-", True),
    "transitgatewayid": ("USES_TRANSIT_GATEWAY", "aws_ec2_transit_gateway", "transit-gateway", "tgw-", True),
    "routetableid": ("USES_ROUTE_TABLE", "aws_vpc_route_table", "route-table", "rtb-", True),
    "networkaclid": ("USES_NETWORK_ACL", "aws_vpc_network_acl", "network-acl", "acl-", True),
    "internetgatewayid": ("USES_INTERNET_GATEWAY", "aws_vpc_internet_gateway", "internet-gateway", "igw-", True),
    "dhcpoptionsid": ("USES_DHCP_OPTIONS", "aws_ec2_dhcp_options", "dhcp-options", "dopt-", True),
    "vpcendpointid": ("USES_VPC_ENDPOINT", "aws_vpc_endpoint", "vpc-endpoint", "vpce-", True),
    "natgatewayid": ("USES_NAT_GATEWAY", "aws_vpc_nat_gateway", "natgateway", "nat-", True),
    "allocationid": ("USES_ELASTIC_IP", "aws_ec2_eip", "elastic-ip", "eipalloc-", True),
    "snapshotid": ("USES_SNAPSHOT", "aws_ec2_snapshot", "snapshot", "snap-", False),
    "imageid": ("USES_IMAGE", "aws_ec2_image", "image", "ami-", False),
}

ARN_TARGETS = {
    "aws_iam_role": ("iam", "role/"),
    "aws_kms_key": ("kms", "key/"),
    "aws_secrets_manager_secret": ("secretsmanager", "secret:"),
    "aws_ecs_task_definition": ("ecs", "task-definition/"),
    "aws_lambda_function": ("lambda", "function:"),
    "aws_elbv2_target_group": ("elasticloadbalancing", "targetgroup/"),
    "aws_vpc_lattice_target_group": ("vpc-lattice", "targetgroup/"),
    "aws_elbv2_load_balancer": ("elasticloadbalancing", "loadbalancer/"),
    "aws_certificate_manager_certificate": ("acm", "certificate/"),
    "aws_sns_topic": ("sns", None),
    "aws_sqs_queue": ("sqs", None),
}

# Identically named fields can have different contracts. These overrides are
# checked against the pinned schema paths; lexical resemblance is insufficient.
IDENTIFIER_PATH_EXCLUSIONS = {
    ("AWS::EC2::Instance", "/SecurityGroups/*"),  # group names, not group IDs
}
ARN_PATH_OVERRIDES = {
    ("AWS::ECS::Service", "/VpcLatticeConfigurations/*/TargetGroupArn"):
        ("USES_TARGET_GROUP", "aws_vpc_lattice_target_group"),
    ("AWS::ECS::Service", "/VpcLatticeConfigurations/*/AdvancedConfiguration/AlternateTargetGroupArn"):
        ("USES_TARGET_GROUP", "aws_vpc_lattice_target_group"),
    ("AWS::MSK::Replicator", "/KafkaClusters/*/ClientAuthentication/SaslOAuthBearer/TokenEndpointTlsCertificateArn"):
        ("USES_SECRET", "aws_secrets_manager_secret"),
}
# CertificateArn is not a universal ACM contract: DMS uses its own certificate
# identifiers and MSK can use a Secrets Manager secret containing CA material.
# Only these schema/documentation-reviewed sites receive the ACM target kind.
ACM_CERTIFICATE_PATHS = {
    ("AWS::EC2::CustomerGateway", "/CertificateArn"),
    ("AWS::ApiGateway::DomainName", "/CertificateArn"),
    ("AWS::ApiGateway::DomainName", "/OwnershipVerificationCertificateArn"),
    ("AWS::ApiGateway::DomainName", "/RegionalCertificateArn"),
    ("AWS::ApiGatewayV2::DomainName", "/DomainNameConfigurations/*/CertificateArn"),
    ("AWS::ApiGatewayV2::DomainName", "/DomainNameConfigurations/*/OwnershipVerificationCertificateArn"),
    ("AWS::VpcLattice::Service", "/CertificateArn"),
    ("AWS::EC2::ClientVpnEndpoint", "/ServerCertificateArn"),
    ("AWS::EC2::VerifiedAccessEndpoint", "/DomainCertificateArn"),
    ("AWS::ApiGateway::DomainNameV2", "/CertificateArn"),
}
ARN_PATH_OVERRIDES.update({path: ("USES_CERTIFICATE", "aws_certificate_manager_certificate")
                           for path in ACM_CERTIFICATE_PATHS})

ADDITIONAL_SENSITIVE = {
    "AWS::ApiGateway::ApiKey": ["/Value"],
    "AWS::ApiGateway::ClientCertificate": ["/PemEncodedCertificate"],
    "AWS::Bedrock::Agent": ["/Instruction", "/PromptOverrideConfiguration"],
    "AWS::Bedrock::Flow": ["/Definition", "/DefinitionString", "/DefinitionSubstitutions"],
    "AWS::Bedrock::Prompt": ["/Variants"],
    "AWS::Bedrock::AutomatedReasoningPolicy": ["/PolicyDefinition"],
    "AWS::SageMaker::Pipeline": ["/PipelineDefinition", "/PipelineDefinitionS3Location"],
    "AWS::SageMaker::ModelCard": ["/Content"],
    "AWS::SageMaker::MonitoringSchedule": ["/MonitoringScheduleConfig/MonitoringJobDefinition/Environment"],
    "AWS::Glue::Blueprint": ["/BlueprintLocation"],
    "AWS::Glue::Schema": ["/SchemaDefinition"],
    "AWS::Glue::DataQualityRuleset": ["/Ruleset"],
    "AWS::Glue::CustomEntityType": ["/RegexString", "/ContextWords"],
    "AWS::CloudFormation::Stack": ["/Parameters", "/Outputs", "/TemplateBody", "/TemplateURL"],
    "AWS::Logs::ScheduledQuery": ["/QueryLanguage", "/QueryString"],
}

SENSITIVE_NODE_NAMES = {
    "password", "passwords", "authtoken", "token", "apikey", "apikeyvalue",
    "secret", "secretstring", "secretbinary", "credentials", "credential",
    "privatekey", "keymaterial", "privatekeycertificate", "certificatepem",
    "domainSigningPrivateKey".lower(), "content", "script", "scriptbody",
    "definition", "definitionstring", "templatedata", "templatebody",
    "userdata", "environment", "environmentvariables", "command", "args",
}


def normalize(value: str) -> str:
    return re.sub(r"[^a-z0-9]", "", value.lower())


def escape(value: str) -> str:
    return value.replace("~", "~0").replace("/", "~1")


def additions() -> dict[str, str]:
    value = json.loads(ADDITIONS.read_text())
    if value["schema_version"] != 1:
        raise ValueError("unsupported reviewed-addition manifest")
    result = value["resource_families"]
    if sum(len(names.split()) for names in result.values()) != 197:
        raise ValueError("reviewed addition count changed")
    return result


def schema_nodes(schema: dict) -> dict[str, dict]:
    """Resolve declared properties/arrays only; opaque maps stay opaque.

    A recursive definition is visited once per branch. A cycle cannot justify
    unbounded generated paths; it is left opaque rather than guessed through.
    """
    output = {}
    visited = 0

    def walk(node, path, refs=(), depth=0):
        nonlocal visited
        visited += 1
        if visited > MAX_SCHEMA_NODES or depth > MAX_SCHEMA_DEPTH:
            raise ValueError(f"schema traversal budget exceeded: {schema.get('typeName')}")
        if not isinstance(node, dict):
            return
        reference = node.get("$ref")
        if reference:
            if reference in refs:
                return
            if not reference.startswith("#/"):
                raise ValueError("external schema reference cannot establish a local mapping")
            target = schema
            for part in reference[2:].split("/"):
                target = target[part.replace("~1", "/").replace("~0", "~")]
            node = {**target, **{key: value for key, value in node.items() if key != "$ref"}}
            refs = (*refs, reference)
        if path:
            output.setdefault(path, node)
        for alternative in ("allOf", "anyOf", "oneOf"):
            branches = node.get(alternative, [])
            if isinstance(branches, dict):
                branches = [branches]
            for branch in branches:
                walk(branch, path, refs, depth + 1)
        for name, child in node.get("properties", {}).items():
            walk(child, path + "/" + escape(name), refs, depth + 1)
        if isinstance(node.get("items"), dict):
            walk(node["items"], path + "/*", refs, depth + 1)

    walk(schema, "")
    return output


def under(path: str, parent: str) -> bool:
    left, right = path.split("/")[1:], parent.split("/")[1:]
    return len(left) >= len(right) and all(
        expected == "*" or expected == actual for actual, expected in zip(left, right)
    )


def sensitive_paths(name: str, schema: dict) -> list[str]:
    """Additional conservative policy for newly admitted types only."""
    paths = set(ADDITIONAL_SENSITIVE.get(name, []))
    for path in schema_nodes(schema):
        field = path.rsplit("/", 1)[-1]
        if normalize(field) in SENSITIVE_NODE_NAMES:
            paths.add(path)
    return sorted(paths)


def string_node(node: dict) -> bool:
    value = node.get("type")
    return value == "string" or isinstance(value, list) and "string" in value


def new_rule(path, relation, kind, *, template=None, identifier_prefix=None):
    rule = {"pointer": path, "relation": relation, "target_kind": kind,
            "target_format": "template" if template else "arn", "arn_template": template,
            "evidence": "schema_ec2_identifier_reference" if template else "schema_arn_property_reference"}
    if identifier_prefix:
        rule["identifier_prefix"] = identifier_prefix
    if kind in ARN_TARGETS and not template:
        service, prefix = ARN_TARGETS[kind]
        rule["arn_service"] = service
        if prefix:
            rule["arn_resource_prefix"] = prefix
    return rule


def arn_semantics(field: str):
    key = normalize(field)
    if key.endswith("executionrolearn"):
        return "USES_EXECUTION_ROLE", "aws_iam_role"
    if key.endswith("taskrolearn"):
        return "USES_TASK_ROLE", "aws_iam_role"
    if key.endswith("rolearn") or key in {"role", "servicerole", "iamrole", "executionrole"}:
        return "ASSUMES_ROLE", "aws_iam_role"
    if key.endswith(("kmskeyarn", "encryptionkeyarn")) or key == "kmskeyid":
        return "ENCRYPTED_BY", "aws_kms_key"
    if key.endswith("secretarn"):
        return "USES_SECRET", "aws_secrets_manager_secret"
    if key.endswith("targetgrouparn"):
        return "USES_TARGET_GROUP", "aws_elbv2_target_group"
    if key.endswith("loadbalancerarn"):
        return "USES_LOAD_BALANCER", "aws_elbv2_load_balancer"
    if key.endswith("functionarn"):
        return "USES_FUNCTION", "aws_lambda_function"
    if key.endswith("topicarn"):
        return "USES_TOPIC", "aws_sns_topic"
    if key.endswith("queuearn"):
        return "USES_QUEUE", "aws_sqs_queue"
    return "REFERENCES", "aws_reference"


def enrich_rules(descriptor: dict, schema: dict, global_sensitive: set[str]) -> list[dict]:
    nodes = schema_nodes(schema)
    protected = descriptor["write_only_properties"] + descriptor["sensitive_properties"]
    readonly = {value.removeprefix("/properties") for value in schema.get("readOnlyProperties", [])}
    self_paths = set(descriptor["arn_pointers"])
    primary = set(descriptor["primary_identifier"])
    result = {}
    for original in descriptor["relationships"]:
        rule = dict(original)
        if rule["pointer"] not in nodes:
            raise ValueError(f"reviewed relationship is absent from schema: {descriptor['type_name']} {rule['pointer']}")
        if (descriptor["type_name"], rule["pointer"]) in IDENTIFIER_PATH_EXCLUSIONS:
            continue
        if any(under(rule["pointer"], path) for path in protected):
            continue
        override = ARN_PATH_OVERRIDES.get((descriptor["type_name"], rule["pointer"]))
        if override:
            rule["relation"], rule["target_kind"] = override
        if rule["target_format"] == "arn" and rule["target_kind"] in ARN_TARGETS:
            service, prefix = ARN_TARGETS[rule["target_kind"]]
            rule["arn_service"] = service
            if prefix:
                rule["arn_resource_prefix"] = prefix
        if rule["target_format"] == "template":
            for _name, (_relation, kind, _resource, prefix, _account) in EC2_IDENTIFIERS.items():
                if kind == rule["target_kind"]:
                    rule["identifier_prefix"] = prefix
                    break
        result[rule["pointer"]] = rule

    for path, node in sorted(nodes.items()):
        if not string_node(node) or path in result or any(under(path, value) for value in protected):
            continue
        if (descriptor["type_name"], path) in IDENTIFIER_PATH_EXCLUSIONS:
            continue
        parts = path.split("/")[1:]
        if any(normalize(part) in global_sensitive for part in parts):
            continue
        field = next((part for part in reversed(parts) if part != "*"), "")
        key = normalize(field)
        if key in EC2_IDENTIFIERS:
            relation, kind, resource, prefix, account_scoped = EC2_IDENTIFIERS[key]
            if path in primary and descriptor["kind"] == kind:
                continue
            account = "{account}" if account_scoped else ""
            template = "arn:{partition}:ec2:{region}:" + account + ":" + resource + "/{identifier}"
            result[path] = new_rule(path, relation, kind, template=template, identifier_prefix=prefix)
            continue
        semantic = ARN_PATH_OVERRIDES.get((descriptor["type_name"], path), arn_semantics(field))
        described_arn = "arn:" in node.get("pattern", "").lower() or node.get("format") == "arn"
        named_arn = key.endswith(("arn", "arns"))
        if not (named_arn or described_arn or semantic[0] != "REFERENCES"):
            continue
        # Root read-only ARN outputs commonly identify this resource. Explicit
        # reviewed rules (e.g. a target group's LoadBalancerArns) were kept above.
        if path in self_paths or path in primary or (path.count("/") == 1 and path in readonly):
            continue
        result[path] = new_rule(path, *semantic)
    if len(result) > MAX_RELATIONSHIP_RULES:
        raise ValueError(f"relationship descriptor budget exceeded: {descriptor['type_name']}")
    return sorted(result.values(), key=lambda value: (value["pointer"], value["relation"]))
