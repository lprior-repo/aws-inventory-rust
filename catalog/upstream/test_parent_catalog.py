"""Independent reviewed-plan/schema and reproducibility checks."""
from __future__ import annotations

import hashlib
import json
import unittest
import zipfile

from jsonschema import Draft7Validator
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT7

from build_parent_catalog import ARCHIVE, ROOT, build


class ParentCatalogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = build()
        cls.types = {row["type_name"]: row for row in cls.catalog["resource_types"]}
        cls.plans = {row["type_name"]: row for row in cls.catalog["parent_plans"]}

    def test_exact_reviewed_families_and_counts(self):
        expected = {
            "EKS": {"Addon", "FargateProfile", "Nodegroup"},
            "ElasticLoadBalancingV2": {"Listener"},
            "ApiGateway": {"Authorizer", "Deployment", "DocumentationVersion", "GatewayResponse", "Model", "RequestValidator", "Resource", "Stage"},
            "ApiGatewayV2": {"Authorizer", "Deployment", "Integration", "Route", "Stage"},
            "Logs": {"LogStream", "SubscriptionFilter"},
            "AppConfig": {"Environment", "ConfigurationProfile"},
        }
        names = {f"AWS::{family}::{child}" for family, children in expected.items() for child in children}
        self.assertEqual(names, set(self.types))
        self.assertEqual(names, set(self.plans))
        self.assertEqual(len(names), 21)
        self.assertNotIn("AWS::ElasticLoadBalancingV2::ListenerRule", names)
        self.assertNotIn("AWS::ECS::TaskSet", names)

    def test_catalog_is_reproducible_and_frozen_root_catalog_is_unmodified(self):
        path = ROOT / "catalog/cloudcontrol-parent-resources.json"
        self.assertEqual(path.read_text(), json.dumps(self.catalog, indent=2) + "\n")
        root = ROOT / "catalog/cloudcontrol-resources.json"
        self.assertEqual(hashlib.sha256(root.read_bytes()).hexdigest(),
                         "7d551d1ae5d32e83dc7bf45d5ef94fca35b0cecbc510cd5a3fe75aafa73300e4")
        self.assertEqual(self.catalog["evidence"]["root_catalog_sha256"], hashlib.sha256(root.read_bytes()).hexdigest())

    def test_literal_request_models_pass_actual_list_handler_schemas(self):
        literal_models = {
            "EKS": {"ClusterName": "alpha"},
            "ElasticLoadBalancingV2": {"LoadBalancerArn": "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/api/0123456789abcdef"},
            "ApiGateway": {"RestApiId": "abc123"},
            "ApiGatewayV2": {"ApiId": "abc123"},
            "Logs": {"LogGroupName": "/aws/lambda/api"},
            "AppConfig": {"ApplicationId": "app1234"},
        }
        with zipfile.ZipFile(ARCHIVE) as archive:
            for name, descriptor in self.types.items():
                with self.subTest(resource_type=name):
                    raw = archive.read(descriptor["schema_file"])
                    self.assertEqual(hashlib.sha256(raw).hexdigest(), descriptor["schema_sha256"])
                    schema = json.loads(raw)
                    registry = Registry().with_resource("resource-schema.json", Resource.from_contents(schema, default_specification=DRAFT7))
                    validator = Draft7Validator(schema["handlers"]["list"]["handlerSchema"], registry=registry)
                    model = literal_models[name.split("::")[1]]
                    validator.validate(model)
                    self.assertFalse(validator.is_valid({}))
                    self.assertEqual(set(model), {item["model_property"] for item in self.plans[name]["bindings"]})

    def test_one_level_regional_roots_and_no_name_or_arn_guessing(self):
        roots = {row["type_name"] for row in json.loads((ROOT / "catalog/cloudcontrol-resources.json").read_text())["resource_types"]}
        self.assertEqual(len({plan["parent_type"] for plan in self.plans.values()}), 6)
        for name, plan in self.plans.items():
            self.assertIn(plan["parent_type"], roots)
            self.assertNotIn(plan["parent_type"], self.plans)
            self.assertEqual(plan["relation"], "CHILD_OF")
            descriptor = self.types[name]
            self.assertTrue(descriptor["list_requires_model"])
            self.assertEqual(descriptor["scope"], "regional")
            self.assertIsNone(descriptor["home_region"])
            self.assertIsNone(descriptor["arn_template"])
            self.assertEqual(len(plan["bindings"]), 1)

    def test_native_overlap_keeps_compatible_resource_kinds(self):
        self.assertEqual(self.types["AWS::EKS::Nodegroup"]["kind"], "aws_eks_node_group")
        self.assertEqual(self.types["AWS::ElasticLoadBalancingV2::Listener"]["kind"], "aws_elbv2_listener")

    def test_additional_secret_values_are_not_exposed_as_top_level_columns(self):
        for name, forbidden in [
            ("AWS::EKS::Addon", "ConfigurationValues"),
            ("AWS::ApiGateway::Stage", "Variables"),
            ("AWS::ApiGatewayV2::Stage", "StageVariables"),
            ("AWS::ApiGateway::GatewayResponse", "ResponseTemplates"),
            ("AWS::ApiGatewayV2::Integration", "RequestTemplates"),
        ]:
            with self.subTest(resource_type=name):
                descriptor = self.types[name]
                self.assertIn("/" + forbidden, descriptor["sensitive_properties"])
                self.assertNotIn("/" + forbidden, {column["pointer"] for column in descriptor["columns"]})


if __name__ == "__main__":
    unittest.main()
