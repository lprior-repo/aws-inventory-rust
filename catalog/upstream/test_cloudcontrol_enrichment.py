"""Offline contracts for the pinned schema audit and relationship compiler.

The semantic examples below are authored independently of the generated
catalog. Archive/capability checks inspect every retained provider schema.
"""

from __future__ import annotations

from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import unittest
import zipfile

from jsonschema import Draft7Validator

from build_cloudcontrol_catalog import ARCHIVE, EXPECTED_SHA256, MODEL_REQUIRED
from cloudcontrol_enrichment import enrich_rules, schema_nodes, sensitive_paths, string_node, under

ROOT = Path(__file__).resolve().parents[2]


def descriptor(**changes):
    value = {
        "type_name": "AWS::Example::Widget",
        "kind": "aws_example_widget",
        "write_only_properties": [],
        "sensitive_properties": [],
        "arn_pointers": ["/Arn"],
        "primary_identifier": ["/Arn"],
        "relationships": [],
    }
    value.update(changes)
    return value


class ReferenceCompilationTests(unittest.TestCase):
    def test_schema_refs_and_array_paths_have_independent_typed_expectations(self):
        schema = {
            "typeName": "AWS::Example::Widget",
            "properties": {"Network": {"$ref": "#/definitions/Network"}},
            "definitions": {
                "Network": {
                    "type": "object",
                    "properties": {
                        "SubnetIds": {"type": "array", "items": {"type": "string"}},
                        "RoleArn": {"type": "string"},
                        "SnapshotId": {"type": "string"},
                        "Opaque": {"type": "object", "additionalProperties": {"type": "string"}},
                    },
                }
            },
        }
        actual = {row["pointer"]: row for row in enrich_rules(descriptor(), schema, set())}
        self.assertEqual(set(actual), {"/Network/SubnetIds/*", "/Network/RoleArn", "/Network/SnapshotId"})
        self.assertEqual(actual["/Network/SubnetIds/*"]["identifier_prefix"], "subnet-")
        self.assertEqual(actual["/Network/SubnetIds/*"]["target_kind"], "aws_vpc_subnet")
        self.assertEqual(actual["/Network/RoleArn"]["arn_service"], "iam")
        self.assertEqual(actual["/Network/RoleArn"]["arn_resource_prefix"], "role/")
        self.assertEqual(actual["/Network/SnapshotId"]["arn_template"],
                         "arn:{partition}:ec2:{region}::snapshot/{identifier}")

    def test_self_arns_write_only_and_sensitive_parents_cannot_be_dependencies(self):
        schema = {
            "properties": {
                "Arn": {"type": "string"},
                "Password": {"type": "object", "properties": {"RoleArn": {"type": "string"}}},
                "Environment": {"type": "object", "properties": {"RoleArn": {"type": "string"}}},
                "Items": {"type": "array", "items": {"type": "object", "properties": {"SecretArn": {"type": "string"}}}},
                "ReadOnlyArn": {"type": "string"},
            },
            "readOnlyProperties": ["/properties/ReadOnlyArn"],
        }
        model = descriptor(write_only_properties=["/Password"], sensitive_properties=["/Items/*/SecretArn"])
        self.assertEqual(enrich_rules(model, schema, {"environment"}), [])

    def test_opaque_property_maps_are_never_searched_for_arn_like_keys(self):
        schema = {"properties": {"Configuration": {"type": "object", "additionalProperties": {"type": "string"}}}}
        self.assertEqual(enrich_rules(descriptor(), schema, set()), [])
        self.assertEqual(set(schema_nodes(schema)), {"/Configuration"})

    def test_recursive_refs_are_bounded_and_external_refs_fail_closed(self):
        schema = {
            "properties": {"Tree": {"$ref": "#/definitions/Node"}},
            "definitions": {"Node": {"type": "object", "properties": {
                "RoleArn": {"type": "string"}, "Next": {"$ref": "#/definitions/Node"},
            }}},
        }
        self.assertEqual(set(schema_nodes(schema)), {"/Tree", "/Tree/RoleArn"})
        with self.assertRaisesRegex(ValueError, "external schema reference"):
            schema_nodes({"properties": {"Outside": {"$ref": "https://example.invalid/schema"}}})
        deep = {"type": "string"}
        for _ in range(34):
            deep = {"properties": {"child": deep}}
        with self.assertRaisesRegex(ValueError, "budget exceeded"):
            schema_nodes(deep)

    def test_only_string_references_are_emitted_and_local_pointer_escaping_is_preserved(self):
        schema = {"properties": {
            "RoleArn": {"type": "integer"},
            "MissingTypeArn": {},
            "Weird/Name": {"$ref": "#/definitions/role~1entry"},
        }, "definitions": {"role/entry": {"type": "object", "properties": {"RoleArn": {"type": "string"}}}}}
        actual = enrich_rules(descriptor(), schema, set())
        self.assertEqual([row["pointer"] for row in actual], ["/Weird~1Name/RoleArn"])

    def test_new_sensitive_policies_are_exact_paths_and_wildcards_cover_only_descendants(self):
        schema = {"properties": {
            "Model": {"properties": {"Environment": {"type": "object"}, "RoleArn": {"type": "string"}}},
        }}
        paths = sensitive_paths("AWS::SageMaker::Pipeline", schema)
        self.assertIn("/Model/Environment", paths)
        self.assertIn("/PipelineDefinition", paths)
        self.assertNotIn("/Model/RoleArn", paths)
        self.assertTrue(under("/Containers/2/Environment/RoleArn", "/Containers/*/Environment"))
        self.assertFalse(under("/Containers/2/EnvironmentArn", "/Containers/*/Environment"))
        self.assertFalse(under("/Containers", "/Containers/*/Environment"))


class RetainedCatalogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = json.loads((ROOT / "catalog/cloudcontrol-resources.json").read_text())
        cls.matrix = json.loads((ROOT / "catalog/cloudcontrol-capabilities.json").read_text())
        cls.schemas = {}
        with zipfile.ZipFile(ARCHIVE) as archive:
            for filename in archive.namelist():
                if filename.endswith(".json"):
                    raw = archive.read(filename)
                    value = json.loads(raw)
                    cls.schemas[value["typeName"]] = (filename, raw, value)

    def test_full_capability_audit_is_a_disjoint_partition_of_every_pinned_schema(self):
        self.assertEqual(hashlib.sha256(ARCHIVE.read_bytes()).hexdigest(), EXPECTED_SHA256)
        self.assertEqual(len(self.schemas), 1881)
        entries = self.matrix["resource_types"]
        self.assertEqual({row["type_name"] for row in entries}, set(self.schemas))
        self.assertEqual(len(entries), len(self.schemas))
        self.assertEqual(sum(row["read_supported"] and row["list_supported"] for row in entries), 1701)
        self.assertEqual(Counter(row["exclusion"] for row in entries), Counter({
            None: 397, "missing_read_or_list_handler": 180, "documented_parent_model_required": 37,
            "schema_parent_model_required": 373, "scope_unreviewed": 894,
        }))
        for row in entries:
            name = row["type_name"]
            filename, raw, schema = self.schemas[name]
            handlers = schema.get("handlers", {})
            self.assertEqual(row["schema_file"], filename)
            self.assertEqual(row["schema_sha256"], hashlib.sha256(raw).hexdigest())
            self.assertEqual(row["read_supported"], "read" in handlers)
            self.assertEqual(row["list_supported"], "list" in handlers)
            if row["selected"]:
                self.assertIsNone(row["exclusion"])
                self.assertEqual(row["scope_review"], "reviewed_regional")
                self.assertNotIn(name, MODEL_REQUIRED)
                self.assertTrue(Draft7Validator(handlers["list"].get("handlerSchema", {})).is_valid({}))
            elif row["exclusion"] == "schema_parent_model_required":
                self.assertFalse(Draft7Validator(handlers["list"].get("handlerSchema", {})).is_valid({}))

    def test_original_200_view_column_contract_is_byte_independently_frozen(self):
        # Digest of the original v0.4 type/view/column projection. Updating the
        # executable catalog must not silently rewrite migration 003's views.
        projection = [{"type_name": row["type_name"], "view_name": row["view_name"], "columns": row["columns"]}
                      for row in self.catalog["resource_types"][:200]]
        digest = hashlib.sha256(json.dumps(projection, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        self.assertEqual(digest, "6227d8c4eef92be7cc1593469efa613773d52cc53c06cca13e29f6200630b36f")

    def test_every_generated_rule_is_declared_string_metadata_with_bounded_constraints(self):
        types = self.catalog["resource_types"]
        self.assertEqual(len(types), 397)
        self.assertEqual(len({row["view_name"] for row in types}), 397)
        self.assertEqual(len({row["service"] for row in types}), 76)
        for model in types:
            schema = self.schemas[model["type_name"]][2]
            nodes = schema_nodes(schema)
            rules = model["relationships"]
            self.assertLessEqual(len(rules), 256)
            self.assertEqual(len({row["pointer"] for row in rules}), len(rules))
            for rule in rules:
                with self.subTest(type_name=model["type_name"], pointer=rule["pointer"]):
                    self.assertIn(rule["pointer"], nodes)
                    self.assertTrue(string_node(nodes[rule["pointer"]]))
                    self.assertFalse(any(under(rule["pointer"], path)
                                         for path in model["write_only_properties"] + model["sensitive_properties"]))
                    if rule["target_format"] == "template":
                        self.assertTrue(rule.get("identifier_prefix"))
                        self.assertNotIn("arn_service", rule)
                    else:
                        self.assertNotIn("identifier_prefix", rule)

    def test_review_excludes_unreviewed_scopes_and_parent_context_even_with_read_list_handlers(self):
        types = {row["type_name"] for row in self.catalog["resource_types"]}
        for excluded in ("AWS::EC2::IPAM", "AWS::IAM::Role", "AWS::Route53::HostedZone", "AWS::S3::Bucket",
                         "AWS::OpenSearchServerless::AccessPolicy", "AWS::MSK::ClusterPolicy",
                         "AWS::Cognito::UserPoolDomain", "AWS::ElasticLoadBalancingV2::Listener"):
            self.assertNotIn(excluded, types)
        snapshot = next(row for row in self.catalog["resource_types"] if row["type_name"] == "AWS::EC2::Snapshot")
        self.assertEqual(snapshot["arn_template"], "arn:{partition}:ec2:{region}::snapshot/{identifier}")

    def test_field_names_do_not_override_type_specific_schema_meaning(self):
        by_name = {row["type_name"]: row for row in self.catalog["resource_types"]}
        instance = {row["pointer"]: row for row in by_name["AWS::EC2::Instance"]["relationships"]}
        self.assertNotIn("/SecurityGroups/*", instance)
        self.assertEqual(instance["/SecurityGroupIds/*"]["identifier_prefix"], "sg-")
        service = {row["pointer"]: row for row in by_name["AWS::ECS::Service"]["relationships"]}
        for pointer in ("/VpcLatticeConfigurations/*/TargetGroupArn",
                        "/VpcLatticeConfigurations/*/AdvancedConfiguration/AlternateTargetGroupArn"):
            self.assertEqual(service[pointer]["arn_service"], "vpc-lattice")
            self.assertEqual(service[pointer]["target_kind"], "aws_vpc_lattice_target_group")
        self.assertEqual(service["/LoadBalancers/*/TargetGroupArn"]["arn_service"], "elasticloadbalancing")

    def test_every_certificate_arn_site_uses_its_reviewed_service_contract(self):
        sites = [(model["type_name"], rule) for model in self.catalog["resource_types"]
                 for rule in model["relationships"] if rule["pointer"].lower().endswith("certificatearn")]
        self.assertEqual(len(sites), 25)
        dms = [(name, rule) for name, rule in sites if name.startswith("AWS::DMS::")]
        self.assertEqual(len(dms), 14)
        for name, rule in dms:
            with self.subTest(type_name=name, pointer=rule["pointer"]):
                self.assertEqual(rule["target_kind"], "aws_reference")
                self.assertEqual(rule["relation"], "REFERENCES")
                self.assertNotIn("arn_service", rule)
        msk = [rule for name, rule in sites if name == "AWS::MSK::Replicator"]
        self.assertEqual(len(msk), 1)
        self.assertEqual(msk[0]["target_kind"], "aws_secrets_manager_secret")
        self.assertEqual(msk[0]["arn_service"], "secretsmanager")
        self.assertEqual(msk[0]["arn_resource_prefix"], "secret:")
        acm = [rule for _, rule in sites if rule["target_kind"] == "aws_certificate_manager_certificate"]
        self.assertEqual(len(acm), 10)
        self.assertTrue(all(rule["arn_service"] == "acm" and rule["arn_resource_prefix"] == "certificate/" for rule in acm))

    def test_generator_is_deterministic_for_every_selected_descriptor(self):
        sensitive = set(self.catalog["evidence"]["global_sensitive_property_names"])
        for descriptor in self.catalog["resource_types"]:
            schema = self.schemas[descriptor["type_name"]][2]
            self.assertEqual(enrich_rules(copy.deepcopy(descriptor), schema, sensitive), descriptor["relationships"])


if __name__ == "__main__":
    unittest.main()
