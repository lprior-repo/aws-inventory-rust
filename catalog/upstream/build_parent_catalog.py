#!/usr/bin/env python3
"""Reproduce the reviewed, parent-scoped Cloud Control catalog.

This is a build-time audit of the pinned official schemas. Runtime discovery
is Rust and never infers a parent relationship from an arbitrary property name.
The original 397-type catalog is deliberately not rewritten.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import zipfile

from jsonschema import Draft7Validator

from build_cloudcontrol_catalog import (
    ARCHIVE, EXPECTED_SHA256, GLOBAL_SENSITIVE, RESERVED, ROOT,
    basic_type, pointer, relationship, snake,
)
from cloudcontrol_enrichment import enrich_rules, sensitive_paths

# Parent property -> child ResourceModel property. Every binding has been
# checked against both retained schemas and the child's list handler schema.
GROUPS = (
    ("EKS", "Cluster", "Name", "ClusterName", "Addon FargateProfile Nodegroup"),
    ("ElasticLoadBalancingV2", "LoadBalancer", "LoadBalancerArn", "LoadBalancerArn", "Listener"),
    ("ApiGateway", "RestApi", "RestApiId", "RestApiId", "Authorizer Deployment DocumentationVersion GatewayResponse Model RequestValidator Resource Stage"),
    ("ApiGatewayV2", "Api", "ApiId", "ApiId", "Authorizer Deployment Integration Route Stage"),
    ("Logs", "LogGroup", "LogGroupName", "LogGroupName", "LogStream SubscriptionFilter"),
    ("AppConfig", "Application", "ApplicationId", "ApplicationId", "Environment ConfigurationProfile"),
)
NATIVE_KINDS = {
    "AWS::EKS::Nodegroup": "aws_eks_node_group",
    "AWS::ElasticLoadBalancingV2::Listener": "aws_elbv2_listener",
}
SENSITIVE = {
    "AWS::EKS::Addon": ["/ConfigurationValues"],
    "AWS::ApiGateway::Stage": ["/Variables", "/CanarySetting/StageVariableOverrides"],
    "AWS::ApiGatewayV2::Stage": ["/StageVariables"],
    "AWS::ApiGateway::Deployment": ["/StageDescription/Variables", "/StageDescription/CanarySetting/StageVariableOverrides"],
    "AWS::ApiGateway::GatewayResponse": ["/ResponseParameters", "/ResponseTemplates"],
    "AWS::ApiGatewayV2::Integration": ["/RequestParameters", "/RequestTemplates", "/ResponseParameters"],
    "AWS::AppConfig::ConfigurationProfile": ["/Validators/*/Content"],
}
RELATIONSHIPS = {
    # NodeRole is an IAM role ARN despite lacking an Arn suffix; preserve the
    # same semantic relation as the native DescribeNodegroup adapter.
    "AWS::EKS::Nodegroup": [relationship("/NodeRole", "ASSUMES_ROLE", "aws_iam_role")],
}


def schema_member(archive: zipfile.ZipFile, family: str, resource: str):
    filename = f"aws-{family.lower()}-{resource.lower()}.json"
    raw = archive.read(filename)
    schema = json.loads(raw)
    if schema["typeName"] != f"AWS::{family}::{resource}":
        raise ValueError("schema archive filename/type mismatch")
    return filename, raw, schema


def make_descriptor(family: str, resource: str, filename: str, raw: bytes, schema: dict) -> dict:
    name = schema["typeName"]
    properties = schema["properties"]
    primary = [pointer(value) for value in schema["primaryIdentifier"]]
    write_only = sorted({pointer(value) for value in schema.get("writeOnlyProperties", [])})
    sensitive = sorted(set(SENSITIVE.get(name, [])) | set(sensitive_paths(name, schema)) |
                       {f"/{key}" for key in properties if snake(key).replace("_", "") in GLOBAL_SENSITIVE})
    redacted = {p.removeprefix("/") for p in write_only + sensitive if p.count("/") == 1}
    columns, names = [], set()
    for key, prop in sorted(properties.items()):
        if key in redacted:
            continue
        column = snake(key)
        if column in RESERVED:
            column = "provider_" + column
        if column in RESERVED or column in names or not re.fullmatch(r"[a-z][a-z0-9_]*", column):
            raise ValueError(f"invalid/duplicate provider column {name}:{column}")
        names.add(column)
        columns.append({"name": column, "pointer": "/" + key, "json_type": basic_type(prop, schema)})
    arns = {f"/{key}" for key in ("Arn", "ARN", resource + "Arn") if key in properties}
    arns.update(value for value in primary if value.lower().endswith("arn"))
    descriptor = {
        "type_name": name, "kind": NATIVE_KINDS.get(name, f"aws_{snake(family)}_{snake(resource)}"),
        "view_name": f"aws_cc_{snake(family)}_{snake(resource)}", "service": snake(family),
        "scope": "regional", "home_region": None, "read_supported": True, "list_supported": True,
        "list_requires_model": True, "list_model_validation": "reviewed_parent_model",
        "primary_identifier": primary, "identifier_is_arn": len(primary) == 1 and primary[0] in arns,
        "arn_pointers": sorted(arns), "arn_template": None, "arn_identifier_pointer": None,
        "name_pointers": [f"/{key}" for key in ("Name", resource + "Name") if key in properties],
        "write_only_properties": write_only, "sensitive_properties": sensitive,
        "relationships": RELATIONSHIPS.get(name, []), "schema_file": filename, "schema_sha256": hashlib.sha256(raw).hexdigest(),
        "read_permissions": sorted(set(schema["handlers"]["read"].get("permissions", []))),
        "list_permissions": sorted(set(schema["handlers"]["list"].get("permissions", []))), "columns": columns,
    }
    descriptor["relationships"] = enrich_rules(descriptor, schema, GLOBAL_SENSITIVE)
    return descriptor


def build() -> dict:
    if hashlib.sha256(ARCHIVE.read_bytes()).hexdigest() != EXPECTED_SHA256:
        raise ValueError("pinned CloudFormation schema archive checksum differs")
    root_catalog = json.loads((ROOT / "catalog/cloudcontrol-resources.json").read_text())
    parents = {d["type_name"]: d for d in root_catalog["resource_types"]}
    resources, plans = [], []
    with zipfile.ZipFile(ARCHIVE) as archive:
        for family, parent, source, target, children in GROUPS:
            _, _, parent_schema = schema_member(archive, family, parent)
            parent_name = f"AWS::{family}::{parent}"
            if parent_name not in parents or basic_type(parent_schema["properties"][source], parent_schema) != "string":
                raise ValueError(f"unreviewed or non-string parent source {parent_name}/{source}")
            for child in children.split():
                filename, raw, schema = schema_member(archive, family, child)
                handlers = schema.get("handlers", {})
                if not all(key in handlers for key in ("read", "list")):
                    raise ValueError(f"child lacks read/list handlers: {schema['typeName']}")
                listing = handlers["list"]["handlerSchema"]
                validator = Draft7Validator(listing)
                if validator.is_valid({}) or target not in listing.get("properties", {}):
                    raise ValueError(f"child is not a reviewed parent-only list: {schema['typeName']}")
                # Shape audit with a literal string; full AWS name patterns are
                # checked by its provider, while no secret fields are injected.
                if basic_type(schema["properties"][target], schema) != "string":
                    raise ValueError(f"non-string child parent property {schema['typeName']}/{target}")
                required = listing.get("required", [])
                alternatives = [part.get("required", []) for part in listing.get("oneOf", [])]
                if required != [target] and [target] not in alternatives:
                    raise ValueError(f"list requires an additional unreviewed context: {schema['typeName']}")
                descriptor = make_descriptor(family, child, filename, raw, schema)
                if f"/{target}" in descriptor["write_only_properties"] + descriptor["sensitive_properties"]:
                    raise ValueError("parent binding targets a redacted property")
                resources.append(descriptor)
                plans.append({"type_name": schema["typeName"], "parent_type": parent_name,
                              "bindings": [{"source_pointer": "/" + source, "model_property": target}],
                              "relation": "CHILD_OF",
                              "evidence": "reviewed_parent_resource_model_and_matching_read_properties"})
    if len(resources) != 21 or len({row["type_name"] for row in resources}) != 21:
        raise ValueError("expected exactly 21 distinct reviewed parent-dependent types")
    return {"schema_version": 1, "evidence": {
        "source_kind": "official_aws_cloudformation_registry_schema_archive",
        "archive_file": "upstream/" + ARCHIVE.name, "archive_sha256": EXPECTED_SHA256,
        "root_catalog_sha256": hashlib.sha256((ROOT / "catalog/cloudcontrol-resources.json").read_bytes()).hexdigest(),
        "global_sensitive_property_names": sorted(GLOBAL_SENSITIVE),
        "list_semantics_url": "https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-operations-list.html",
        "list_api_url": "https://docs.aws.amazon.com/cloudcontrolapi/latest/APIReference/API_ListResources.html",
        "scope_review": "Six reviewed regional root types; one dependency level; exact string property bindings",
        "live_validated": False,
        "limitations": ["Schema and request-model audit is not live AWS provider validation.",
                        "Two-level parents, account/global contexts, and other unreviewed bindings remain excluded.",
                        "Parent discovery uses complete sanitized GetResource properties; missing context records a failure."]},
        "resource_types": sorted(resources, key=lambda d: d["type_name"]),
        "parent_plans": sorted(plans, key=lambda d: d["type_name"])}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    output = json.dumps(build(), indent=2) + "\n"
    path = ROOT / "catalog/cloudcontrol-parent-resources.json"
    if args.check:
        if path.read_text() != output:
            raise SystemExit("parent catalog differs; regenerate with build_parent_catalog.py")
    else:
        path.write_text(output)
    print("21 reviewed parent-dependent Cloud Control types " + ("verified" if args.check else "generated"))


if __name__ == "__main__":
    main()
