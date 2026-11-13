"""Negative controls for the independent parent collection oracle."""
from __future__ import annotations

import copy
import hashlib
import json
import sqlite3
import unittest

import assert_parents


def identity(row, identifier):
    return row["expected_resource_id"] or (
        "awscc:aws:us-east-1:000000000000:" + row["type_name"] + ":" + hashlib.sha256(identifier.encode()).hexdigest())


class ParentOracleTests(unittest.TestCase):
    def setUp(self):
        fixture, digest = assert_parents.load_fixtures()
        self.fixture = fixture
        self.connection = sqlite3.connect(":memory:")
        self.connection.executescript("""
            CREATE TABLE resource_observations(scan_id TEXT,source TEXT,resource_json TEXT);
            CREATE TABLE edge_observations(scan_id TEXT,source TEXT,edge_json TEXT);
            CREATE TABLE scope_checkpoints(scan_id TEXT,collector TEXT,parent_id TEXT,last_page INTEGER,complete INTEGER,failed INTEGER);
            CREATE TABLE collection_failures(scan_id TEXT);
        """)
        self.seed = {"fixture_sha256": digest, "status": "complete", "resources": []}
        self.seed["seed_source_sha256"] = hashlib.sha256((assert_parents.ROOT / "scripts/emulator/seed_parents.py").read_bytes()).hexdigest()
        self.seed["parent_catalog_sha256"] = hashlib.sha256((assert_parents.ROOT / "catalog/cloudcontrol-parent-resources.json").read_bytes()).hexdigest()
        self.parents = {}
        for parent in fixture["parents"]:
            identifier = "returned-fixture-" + parent["key"]
            self.seed["resources"].append({"role": "parent", "name": parent["key"], "type_name": parent["type_name"], "identifier": identifier, "status": "SUCCESS"})
            self.parents[parent["key"]] = (parent, identity(parent, identifier))
            self.add_resource({"id": identity(parent, identifier), "kind": parent["expected_kind"],
                               "properties": {"cloudformation_type": parent["type_name"], "provider_properties": parent["provider"]}})
        for child in fixture["children"] + fixture["pagination_children"]:
            identifier = "returned-fixture-" + child["name"]
            self.seed["resources"].append({"role": "child", "name": child["name"], "type_name": child["type_name"], "identifier": identifier, "status": "SUCCESS"})
            parent, parent_id = self.parents[child["parent_key"]]
            child_id = identity(child, identifier)
            self.add_resource({"id": child_id, "kind": child["expected_kind"], "account_id": "000000000000", "region": "us-east-1", "source": "cloud_control", "observed_at": 10,
                "properties": {"cloudformation_type": child["type_name"], "provider_properties": child["expected_provider"],
                    "details_available": True, "observation_detail": "get_resource", "resource_identifier": identifier,
                    "list_resource_model": child["expected_model"], "parent_type": parent["type_name"], "parent_resource_id": parent_id}})
            self.connection.execute("INSERT INTO edge_observations VALUES('scan','cloud_control',?)", (json.dumps({
                "from_id": child_id, "to_id": parent_id, "relation": "CHILD_OF", "source": "cloud_control", "observed_at": 10,
                "properties": {"target_kind": parent["expected_kind"], "cloudformation_type": child["type_name"], "parent_type": parent["type_name"],
                    "resource_model": child["expected_model"], "evidence": "reviewed_parent_resource_model_and_matching_read_properties",
                    "owner_resolution": "observed_parent_resource_identity"}}),))
            for edge in child["expected_relationships"]:
                self.connection.execute("INSERT INTO edge_observations VALUES('scan','cloud_control',?)", (json.dumps({
                    "from_id": child_id, "to_id": edge["target"], "relation": edge["relation"], "source": "cloud_control", "observed_at": 10,
                    "properties": {"target_kind": edge["kind"], "cloudformation_type": child["type_name"],
                        "property_paths": edge["paths"], "owner_resolution": edge["owner_resolution"]}}),))
        for type_name in {row["type_name"] for row in fixture["children"]}:
            self.connection.execute("INSERT INTO scope_checkpoints VALUES('scan','cloud_control_parent_discovery',?,1,1,0)", (type_name,))
        for side in ("a", "b"):
            model = json.dumps({"ClusterName": "inventory-parent-eks-" + side}, sort_keys=True, separators=(",", ":"))
            scope = "AWS::EKS::Addon:" + hashlib.sha256(model.encode()).hexdigest()
            self.connection.execute("INSERT INTO scope_checkpoints VALUES('scan','cloud_control_list',?,2,1,0)", (scope,))

    def tearDown(self):
        self.connection.close()

    def add_resource(self, resource):
        self.connection.execute("INSERT INTO resource_observations VALUES('scan','cloud_control',?)", (json.dumps(resource),))

    def results(self):
        return assert_parents.check(self.connection, "scan", self.seed)

    def failures(self):
        return [row["check"] for row in self.results() if not row["passed"]]

    def mutate_child(self, update):
        for rowid, raw in self.connection.execute("SELECT rowid,resource_json FROM resource_observations"):
            value = json.loads(raw)
            if "list_resource_model" in value.get("properties", {}):
                update(value)
                self.connection.execute("UPDATE resource_observations SET resource_json=? WHERE rowid=?", (json.dumps(value), rowid))
                return
        self.fail("no fixture child found")

    def test_literal_complete_snapshot_passes(self):
        checks = self.results()
        self.assertGreater(len(checks), 1_000)
        self.assertEqual(self.failures(), [])

    def test_cross_parent_edge_swap_fails(self):
        rowid, raw = self.connection.execute("SELECT rowid,edge_json FROM edge_observations LIMIT 1").fetchone()
        edge = json.loads(raw)
        edge["to_id"] = self.parents["eks-b"][1]
        self.connection.execute("UPDATE edge_observations SET edge_json=? WHERE rowid=?", (json.dumps(edge), rowid))
        self.assertIn("parents:no_cross_parent_edges", self.failures())

    def test_missing_child_observation_fails(self):
        self.connection.execute("DELETE FROM resource_observations WHERE rowid=(SELECT max(rowid) FROM resource_observations)")
        self.assertIn("parents:exact_child_count", self.failures())

    def test_duplicate_child_observation_fails(self):
        raw = self.connection.execute("SELECT resource_json FROM resource_observations ORDER BY rowid DESC LIMIT 1").fetchone()[0]
        self.connection.execute("INSERT INTO resource_observations VALUES('scan','cloud_control',?)", (raw,))
        self.assertIn("parents:exact_child_count", self.failures())

    def test_changed_model_and_provider_properties_fail(self):
        self.mutate_child(lambda row: row["properties"]["provider_properties"].update(ClusterName="wrong-parent"))
        self.assertTrue(any(check.endswith(":exact_read") for check in self.failures()))
        self.mutate_child(lambda row: row["properties"].update(list_resource_model={"ClusterName": "wrong-parent"}))
        self.assertTrue(any(check.endswith(":exact_parent_provenance") for check in self.failures()))

    def test_wrong_child_kind_or_detail_availability_fails(self):
        self.mutate_child(lambda row: row.update(kind="aws_wrong_kind"))
        self.assertTrue(any(check.endswith(":exact_read") for check in self.failures()))
        self.mutate_child(lambda row: row["properties"].update(details_available=False))
        self.assertTrue(any(check.endswith(":exact_read") for check in self.failures()))

    def test_unredacted_secret_fails(self):
        self.mutate_child(lambda row: row["properties"]["provider_properties"].update(ConfigurationValues="HIDDEN-secret"))
        self.assertIn("parents:no_sensitive_fixture_values", self.failures())

    def test_incomplete_discovery_or_failure_record_fails(self):
        self.connection.execute("UPDATE scope_checkpoints SET failed=1 WHERE collector='cloud_control_parent_discovery'")
        self.connection.execute("INSERT INTO collection_failures VALUES('scan')")
        self.assertIn("parents:all_twenty_one_parent_discoveries_complete", self.failures())
        self.assertIn("parents:no_collection_failures", self.failures())

    def test_single_page_receipt_does_not_claim_pagination(self):
        self.connection.execute("UPDATE scope_checkpoints SET last_page=1 WHERE collector='cloud_control_list'")
        self.assertIn("parents:addon_a:actual_emulator_pagination", self.failures())
        self.assertIn("parents:addon_b:actual_emulator_pagination", self.failures())

    def test_changed_or_duplicate_seed_receipt_fails(self):
        self.seed["fixture_sha256"] = "wrong"
        self.seed["resources"][-1] = copy.deepcopy(self.seed["resources"][-2])
        self.assertIn("parents:fixture_hash", self.failures())
        self.assertIn("parents:seed_receipts_unique", self.failures())

    def test_missing_native_role_architecture_edge_fails(self):
        for rowid, raw in self.connection.execute("SELECT rowid,edge_json FROM edge_observations"):
            edge = json.loads(raw)
            if edge["relation"] == "ASSUMES_ROLE" and edge["to_id"].endswith("role/fixture-node"):
                self.connection.execute("DELETE FROM edge_observations WHERE rowid=?", (rowid,))
                break
        self.assertTrue(any(check.endswith(":exact_architecture_edge_set") for check in self.failures()))


if __name__ == "__main__":
    unittest.main()
