"""Adversarial checks for the Cloud Control architecture evidence gate."""

from __future__ import annotations

import json
import sqlite3
import unittest
from unittest.mock import patch

import assert_cloud_control as oracle


class CloudControlOracleTests(unittest.TestCase):
    def setUp(self):
        self.connection = sqlite3.connect(":memory:")
        self.addCleanup(self.connection.close)
        self.connection.execute("CREATE TABLE resource_observations(scan_id TEXT,source TEXT,resource_json TEXT)")
        self.connection.execute("CREATE TABLE edge_observations(scan_id TEXT,source TEXT,edge_json TEXT)")
        self.fixtures, digest = oracle.golden_fixtures()
        self.report = {"architecture_fixtures": {"fixture_sha256": digest,
                                                "names": sorted(oracle.GOLDEN_NAMES)}}
        for fixture in self.fixtures:
            identifier = oracle.expected_id(fixture)
            resolution = ("namespaced_provider_identifier" if fixture["opaque_identity"] else
                          "canonical_template_without_account_component"
                          if fixture["name"] == "snapshot_self_identity_uses_accountless_arn" else "returned_resource_arn")
            row = {"id": identifier, "source": "cloud_control", "observed_at": 1234,
                   "properties": {"cloudformation_type": fixture["type_name"], "details_available": True,
                                  "observation_detail": "get_resource", "identity_resolution": resolution}}
            self.connection.execute("INSERT INTO resource_observations VALUES ('cloud','cloud_control',?)",
                                    (json.dumps(row),))
            for expected in fixture["edges"]:
                row = {"from_id": identifier, "to_id": expected["target"], "relation": expected["relation"],
                       "source": "cloud_control", "observed_at": 1234,
                       "properties": {"cloudformation_type": fixture["type_name"],
                                      "target_kind": expected["kind"], "owner_resolution": expected["owner"],
                                      "property_paths": expected["paths"]}}
                self.connection.execute("INSERT INTO edge_observations VALUES ('cloud','cloud_control',?)",
                                        (json.dumps(row),))

    def results(self, scan_id="cloud"):
        return oracle.check(self.connection, scan_id, self.report)

    def mutate_first_edge(self, change):
        row_id, raw = self.connection.execute("SELECT rowid,edge_json FROM edge_observations LIMIT 1").fetchone()
        value = json.loads(raw)
        change(value)
        self.connection.execute("UPDATE edge_observations SET edge_json=? WHERE rowid=?", (json.dumps(value), row_id))

    def test_authored_fixture_selection_and_complete_positive_evidence(self):
        self.assertEqual(len(self.fixtures), 13)
        self.assertEqual(sum(len(row["edges"]) for row in self.fixtures), 30)
        checks = self.results()
        self.assertEqual(len(checks), 111)
        self.assertTrue(all(row["passed"] for row in checks), [row for row in checks if not row["passed"]])

    def test_missing_edge_cannot_be_satisfied_by_resource_presence(self):
        self.connection.execute("DELETE FROM edge_observations WHERE rowid=(SELECT min(rowid) FROM edge_observations)")
        self.assertTrue(any(not row["passed"] and row["check"].endswith(":exact_edge_set") for row in self.results()))

    def test_wrong_target_kind_or_owner_or_property_path_is_rejected(self):
        changes = (
            lambda edge: edge["properties"].update(target_kind="aws_reference"),
            lambda edge: edge["properties"].update(owner_resolution="assumed_scanning_account"),
            lambda edge: edge["properties"].update(property_paths=["/NotTheObservedProperty"]),
            lambda edge: edge.update(observed_at=1235),
        )
        original = self.connection.execute("SELECT rowid,edge_json FROM edge_observations LIMIT 1").fetchone()
        for change in changes:
            with self.subTest(change=change):
                self.connection.execute("UPDATE edge_observations SET edge_json=? WHERE rowid=?", (original[1], original[0]))
                self.mutate_first_edge(change)
                self.assertTrue(any(not row["passed"] and ":edge:" in row["check"] for row in self.results()))

    def test_other_scan_or_source_cannot_supply_missing_evidence(self):
        self.assertTrue(any(not row["passed"] for row in self.results("another-scan")))
        self.connection.execute("UPDATE resource_observations SET source='direct'")
        self.connection.execute("UPDATE edge_observations SET source='direct'")
        self.assertTrue(any(not row["passed"] for row in self.results()))

    def test_duplicate_read_or_duplicate_edge_is_not_a_complete_exact_snapshot(self):
        self.connection.execute("INSERT INTO resource_observations SELECT * FROM resource_observations LIMIT 1")
        self.assertTrue(any(not row["passed"] and row["check"].endswith(":one_typed_read") for row in self.results()))
        self.connection.execute("DELETE FROM resource_observations WHERE rowid=(SELECT max(rowid) FROM resource_observations)")
        self.connection.execute("INSERT INTO edge_observations SELECT * FROM edge_observations LIMIT 1")
        self.assertTrue(any(not row["passed"] and row["check"].endswith(":exact_edge_set") for row in self.results()))

    def test_redaction_or_fixture_binding_failure_remains_a_failure(self):
        row_id, raw = self.connection.execute("SELECT rowid,resource_json FROM resource_observations LIMIT 1").fetchone()
        value = json.loads(raw)
        value["properties"]["environment"] = {"value": "HIDDEN"}
        self.connection.execute("UPDATE resource_observations SET resource_json=? WHERE rowid=?", (json.dumps(value), row_id))
        self.report["architecture_fixtures"]["fixture_sha256"] = "0" * 64
        failed = [row["check"] for row in self.results() if not row["passed"]]
        self.assertIn("cloud_control:architecture_fixture_metadata", failed)
        self.assertTrue(any(name.endswith(":redaction") for name in failed))

    def test_observation_budget_is_an_explicit_failure(self):
        with patch.object(oracle, "MAX_ROWS", 1):
            self.assertEqual(self.results(), [{"check": "cloud_control:bounded_architecture_evidence", "passed": False,
                                              "detail": "Cloud Control assertion evidence exceeds its bound"}])


if __name__ == "__main__":
    unittest.main()
