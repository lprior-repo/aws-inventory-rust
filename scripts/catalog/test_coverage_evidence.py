"""Negative controls for transport/typed-read capability accounting."""

from __future__ import annotations

import copy
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest

from refresh_cloudcontrol_coverage import apply_verification, canonical_sha256, checked_histogram, persisted_reads


class CoverageEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.catalog = {"resource_types": [{"type_name": "AWS::Example::Thing", "schema_sha256": "a" * 64}]}
        self.matrix = {"resource_types": [
            {"type_name": "AWS::Example::Thing", "schema_sha256": "a" * 64, "selected": True},
            {"type_name": "AWS::Example::Excluded", "schema_sha256": "b" * 64, "selected": False,
             "emulator_verification": {"status": "not_run"}},
        ], "summary": {}, "evidence": {}}
        names = ("coverage:all_catalog_types_listed", "coverage:every_registered_type_read",
                 "sqlite:integrity", "cloud_control:architecture_fixture_metadata",
                 "provenance:embedded_catalog_matches", "provenance:source_unchanged",
                 "provenance:binary_unchanged", "provenance:fixture_unchanged")
        self.summary = {"schema_version": 1, "failed_checks": 0, "passed_checks": len(names),
                        "checks": [{"check": name, "passed": True} for name in names],
                        "cloud_control_resource_types": [{
                            "type_name": "AWS::Example::Thing",
                            "operations": {"ListResources": {"200": 1}, "GetResource": {"200": 1}},
                        }]}
        self.binding = {"summary_file": "artifacts/example/summary.json", "catalog_sha256": "c" * 64,
                        "source_sha256": "d" * 64, "binary_sha256": "e" * 64,
                        "architecture_fixture_sha256": "f" * 64}
        self.summary["execution"] = {
            "catalog_canonical_sha256": canonical_sha256(self.catalog), "catalog_sha256": "c" * 64,
            "source": {"sha256": "d" * 64}, "binary": {"sha256": "e" * 64},
            "architecture_fixture_sha256": "f" * 64,
        }

    def merged(self, counts=None):
        return apply_verification(self.matrix, self.catalog, self.summary, counts or {}, self.binding)

    def test_only_matching_persisted_typed_read_is_a_success(self):
        result = self.merged({"AWS::Example::Thing": 1})
        selected = result["resource_types"][0]["emulator_verification"]
        self.assertEqual(selected["status"], "read_observed")
        self.assertTrue(selected["read_observed"])
        self.assertEqual(result["summary"]["emulator_read_observed"], 1)
        self.assertEqual(result["resource_types"][1]["emulator_verification"], {"status": "not_run"})
        self.assertNotIn("emulator_verification", self.matrix["resource_types"][0])

    def test_http_200_without_a_typed_record_is_explicitly_unverified(self):
        result = self.merged()
        self.assertEqual(result["resource_types"][0]["emulator_verification"]["status"], "response_without_persisted_read")
        self.assertEqual(result["summary"]["emulator_read_observed"], 0)

    def test_list_error_is_attempted_but_not_a_successful_read(self):
        self.summary["cloud_control_resource_types"][0]["operations"] = {
            "ListResources": {"400": 1}, "GetResource": {},
        }
        result = self.merged()
        self.assertEqual(result["resource_types"][0]["emulator_verification"]["status"], "list_error")
        self.assertEqual(result["summary"]["emulator_list_attempted"], 1)
        self.assertEqual(result["summary"]["emulator_read_observed"], 0)

    def test_missing_failed_or_miscounted_baseline_checks_fail_closed(self):
        original = copy.deepcopy(self.summary)
        for mutation in (
            lambda value: value.update(failed_checks=1),
            lambda value: value["checks"][0].update(passed=False),
            lambda value: value["checks"].pop(),
            lambda value: value["checks"][0].update(check="unrelated-check"),
        ):
            self.summary = copy.deepcopy(original)
            mutation(self.summary)
            with self.assertRaisesRegex(ValueError, "complete asserted"):
                self.merged()

    def test_wrong_type_set_or_schema_binding_cannot_reuse_an_old_run(self):
        self.summary["cloud_control_resource_types"][0]["type_name"] = "AWS::Example::Other"
        with self.assertRaisesRegex(ValueError, "type set differs"):
            self.merged()
        self.summary["cloud_control_resource_types"][0]["type_name"] = "AWS::Example::Thing"
        self.matrix["resource_types"][0]["schema_sha256"] = "c" * 64
        with self.assertRaisesRegex(ValueError, "schema binding differs"):
            self.merged()

    def test_extra_persisted_records_cannot_exceed_successful_sdk_responses(self):
        with self.assertRaisesRegex(ValueError, "matching Rust GetResource"):
            self.merged({"AWS::Example::Thing": 2})
        with self.assertRaisesRegex(ValueError, "unexpected provider"):
            self.merged({"AWS::Example::Other": 1})

    def test_changed_mapping_cannot_reuse_matching_type_and_upstream_schema_hash(self):
        self.catalog["resource_types"][0]["relationships"] = [{"pointer": "/ChangedRoleArn"}]
        with self.assertRaisesRegex(ValueError, "differs from executed"):
            self.merged({"AWS::Example::Thing": 1})

    def test_changed_collector_source_binary_or_fixture_cannot_reuse_old_execution(self):
        for key in ("source_sha256", "binary_sha256", "architecture_fixture_sha256", "catalog_sha256"):
            with self.subTest(key=key):
                binding = dict(self.binding)
                binding[key] = "changed"
                with self.assertRaisesRegex(ValueError, "differs from executed"):
                    apply_verification(self.matrix, self.catalog, self.summary, {"AWS::Example::Thing": 1}, binding)

    def test_duplicate_matrix_entries_cannot_count_one_execution_twice(self):
        result = self.merged({"AWS::Example::Thing": 1})
        result["resource_types"].append(copy.deepcopy(result["resource_types"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate capability"):
            apply_verification(result, self.catalog, self.summary, {"AWS::Example::Thing": 1}, self.binding)

    def test_missing_provenance_is_not_equal_to_missing_current_binding(self):
        self.summary["execution"] = {"catalog_canonical_sha256": canonical_sha256(self.catalog)}
        with self.assertRaisesRegex(ValueError, "differs from executed"):
            apply_verification(self.matrix, self.catalog, self.summary, {"AWS::Example::Thing": 1},
                               {"summary_file": "unbound.json"})

    def test_status_histograms_are_bounded_integer_counts(self):
        self.assertEqual(checked_histogram({"200": 2, "400": 1, "None": 1}), {"200": 2, "400": 1, "None": 1})
        for value in ({"200": True}, {"200": 0}, {"200": -1}, {"200": "1"}, {"999": 1}, {"success": 1}):
            with self.subTest(value=value), self.assertRaises(ValueError):
                checked_histogram(value)

    def test_persisted_read_loader_is_scoped_and_rejects_wrong_schema(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "inventory.sqlite"
            connection = sqlite3.connect(path)
            connection.execute("CREATE TABLE scans(scan_id TEXT,status TEXT)")
            connection.execute("CREATE TABLE resource_observations(scan_id TEXT,source TEXT,resource_json TEXT)")
            connection.execute("INSERT INTO scans VALUES ('scan','partial')")
            resource = {"source": "cloud_control", "properties": {
                "cloudformation_type": "AWS::Example::Thing", "schema_sha256": "a" * 64,
                "details_available": True, "observation_detail": "get_resource",
            }}
            connection.execute("INSERT INTO resource_observations VALUES ('scan','cloud_control',?)", (json.dumps(resource),))
            connection.execute("INSERT INTO resource_observations VALUES ('other','cloud_control',?)", (json.dumps(resource),))
            connection.execute("INSERT INTO resource_observations VALUES ('scan','direct',?)", (json.dumps(resource),))
            connection.commit()
            self.assertEqual(persisted_reads(path, "scan", self.catalog), {"AWS::Example::Thing": 1})
            with self.assertRaisesRegex(ValueError, "status differs"):
                persisted_reads(path, "scan", self.catalog, "complete")
            with self.assertRaisesRegex(ValueError, "missing or not terminal"):
                persisted_reads(path, "other", self.catalog)
            resource["properties"]["schema_sha256"] = "b" * 64
            connection.execute("UPDATE resource_observations SET resource_json=? WHERE scan_id='scan' AND source='cloud_control'",
                               (json.dumps(resource),))
            connection.commit()
            with self.assertRaisesRegex(ValueError, "different provider schema"):
                persisted_reads(path, "scan", self.catalog)
            connection.close()


if __name__ == "__main__":
    unittest.main()
