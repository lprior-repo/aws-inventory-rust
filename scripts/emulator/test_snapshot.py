"""File-backed synthetic snapshot provenance controls; no AWS or Cargo is run."""

from __future__ import annotations

import copy
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest import mock

import snapshot


CASE_NAMES = {
    "native": "native", "sources": "sources", "cloudcontrol": "cloud-control",
    "assumedrole": "assumed-role", "parents": "parents",
}
CHECK_NAMES = [
    "endpoint:emulator_identity", "native:terminal_scan", "global:all_39_native_kinds",
    "sqlite:integrity", "sqlite:all_scans_finalized", "coverage:every_operation_attempted",
    "coverage:no_unexplained_operation_failures", "coverage:every_registered_type_read",
    "provenance:embedded_catalog_matches", "provenance:source_unchanged",
    "provenance:binary_unchanged", "provenance:fixture_unchanged",
    "provenance:parent_fixture_unchanged", "provenance:scan_commands_bound",
    "parents:fixture_hash", "parents:seed_source_hash", "parents:parent_catalog_hash",
    "parents:seed_complete", "parents:exact_child_count", "parents:exact_parent_edge_count",
    "parents:all_twenty_four_parent_discoveries_complete", "parents:no_collection_failures",
]


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    return snapshot.file_sha256(path)


class SnapshotBinding(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.evidence = self.root / "evidence"
        self.evidence.mkdir()
        self.destination = self.root / "frozen.json"
        self.binary = self.root / "collector"
        self.binary.write_bytes(b"synthetic collector identity; never executable\n")
        self.binary_sha = snapshot.file_sha256(self.binary)
        for name in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src/main.rs", "crates/aws/src/lib.rs"]:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("synthetic frozen source\n")
        self.catalog = {"resource_types": [{"type_name": "AWS::EKS::Cluster", "list_requires_model": False}],
                        "evidence": {"synthetic": "roots"}}
        self.parents = {
            "resource_types": [{"type_name": "AWS::EKS::Addon", "list_requires_model": True}],
            "parent_plans": [{"type_name": "AWS::EKS::Addon", "parent_type": "AWS::EKS::Cluster",
                              "bindings": [{"source_pointer": "/Name", "model_property": "ClusterName"}]}],
            "evidence": {"synthetic": "parents"},
        }
        self.runtime = {"resource_types": [copy.deepcopy(self.catalog["resource_types"][0]),
            {**copy.deepcopy(self.parents["resource_types"][0]),
             "parent_workflow": copy.deepcopy(self.parents["parent_plans"][0])}],
            "evidence": {"synthetic": "roots"}, "parent_evidence": {"synthetic": "parents"}}
        self.role = "arn:aws:iam::000000000000:role/synthetic-role"
        write_json(self.evidence / "seed-results.json", {
            "modules": {"seed_discovery": {"fixtures": {"assume_role_arn": self.role}}}})
        self.execution = {
            "binary": {"path": str(self.binary), "sha256": self.binary_sha},
            "catalog_sha256": write_json(self.root / "catalog/cloudcontrol-resources.json", self.catalog),
            "catalog_canonical_sha256": snapshot.canonical_sha256(self.catalog),
            "parent_catalog_sha256": write_json(self.root / "catalog/cloudcontrol-parent-resources.json", self.parents),
            "runtime_catalog_canonical_sha256": snapshot.canonical_sha256(self.runtime),
            "architecture_fixture_sha256": write_json(
                self.root / "crates/aws/tests/fixtures/cloudcontrol-architecture.json", {"fixture": "root"}),
            "parent_fixture_sha256": write_json(
                self.root / "crates/aws/tests/fixtures/cloudcontrol-parents.json", {"fixture": "parent"}),
            "source": snapshot.source_fingerprint(self.root), "scan_commands": {},
        }
        catalog_sha = write_json(self.evidence / "catalog.json", {
            "name": "catalog", "arguments": ["catalog"], "exit_code": 0,
            "binary_sha256": self.binary_sha, "result": {"cloud_control": self.runtime}})
        self.execution["catalog_command"] = {"file": "catalog.json", "sha256": catalog_sha}
        common = ["scan", "--db", str(self.evidence / "inventory.sqlite"), "--region", "us-east-1",
                  "--attempts", "2", "--max-pages", "128", "--page-size", "1", "--concurrency", "32",
                  "--per-bucket-concurrency", "4", "--rps", "100"]
        selections = {
            "native": ["--service", "all"],
            "sources": ["--service", "config,resource-explorer", "--discover-sources"],
            "cloud-control": ["--service", "cloud-control", "--cloud-control", "--resource-type", "AWS::EKS::Cluster"],
            "parents": ["--service", "cloud-control", "--cloud-control", "--resource-type", "AWS::EKS::Addon"],
            "assumed-role": ["--service", "iam", "--role-arn", self.role],
        }
        self.commands = {}
        with sqlite3.connect(self.evidence / "inventory.sqlite") as connection:
            connection.executescript("""
                CREATE TABLE scans(scan_id TEXT PRIMARY KEY,status TEXT);
                CREATE TABLE resource_observations(scan_id TEXT,resource_json TEXT);
                CREATE TABLE edge_observations(scan_id TEXT,edge_json TEXT);
                CREATE TABLE scope_checkpoints(scan_id TEXT,scope_key TEXT,account_id TEXT,region TEXT,
                    collector TEXT,parent_id TEXT,source TEXT,last_page INTEGER,next_token TEXT,complete INTEGER,failed INTEGER);
                CREATE TABLE collection_failures(scan_id TEXT,scope_key TEXT,category TEXT,message TEXT);
            """)
            for name in CASE_NAMES.values():
                scan_id = "synthetic-" + name
                self.commands[name] = {"name": name, "arguments": common + selections[name], "exit_code": 0,
                    "binary_sha256": self.binary_sha, "result": {"status": "complete", "write_error": None,
                    "scans": [{"scan_id": scan_id, "status": "complete", "error": None}]}}
                self.bind_command(name)
                connection.execute("INSERT INTO scans VALUES (?, 'complete')", (scan_id,))
                connection.execute("INSERT INTO resource_observations VALUES (?,?)", (scan_id, json.dumps({
                    "id": name, "observed_at": 123, "properties": {"observed_at": "provider-time", "array": [2, 1]}})))
                connection.execute("INSERT INTO edge_observations VALUES (?,?)", (scan_id, json.dumps({
                    "from_id": name, "to_id": "parent", "observed_at": 124, "properties": {"kept": True}})))
                connection.execute("INSERT INTO scope_checkpoints VALUES (?,?,?,?,?,?,?,?,?,?,?)", (
                    scan_id, "scope-" + name, "000000000000", "us-east-1", name, None, "direct", 2, None, 1, 0))
        checks = [{"check": name, "passed": True} for name in CHECK_NAMES]
        checks += [{"check": f"synthetic:{number}", "passed": True} for number in range(1000 - len(checks))]
        self.summary = {"failed_checks": 0, "passed_checks": len(checks), "checks": checks,
            "execution": self.execution, "emulator": {"account": "000000000000", "region": "us-east-1",
                "source_commit": "f515581569b98a9dcbd2e686c12f912fdce51bba", "version_output": "aws-emulator 1.0.336"}}
        self.save_summary()
        patcher = mock.patch.object(snapshot, "ROOT", self.root)
        patcher.start()
        self.addCleanup(patcher.stop)

    def bind_command(self, name):
        command = self.commands[name]
        binding = {"file": name + ".json", "sha256": write_json(self.evidence / (name + ".json"), command),
                   "scan_id": command["result"]["scans"][0]["scan_id"], "binary_sha256": command["binary_sha256"]}
        self.execution["scan_commands"][name] = binding
        aliases = {"cloud-control": "cloud_control_command", "parents": "parent_command"}
        if name in aliases:
            self.execution[aliases[name]] = copy.deepcopy(binding)

    def save_summary(self):
        write_json(self.evidence / "summary.json", self.summary)

    def freeze(self, binary=None):
        return snapshot.freeze(self.evidence, self.destination, binary or self.binary)

    def rejected(self, message, binary=None):
        with self.assertRaisesRegex(ValueError, message):
            self.freeze(binary)
        self.assertFalse(self.destination.exists(), "invalid evidence must not produce an oracle")

    def test_exact_five_bound_cases_preserve_payload_and_only_remove_local_time(self):
        receipt = self.freeze()
        result = json.loads(self.destination.read_text())
        self.assertEqual(result["schema_version"], 1)
        self.assertEqual({case["name"] for case in result["cases"]}, set(CASE_NAMES))
        self.assertEqual(len(result["cases"]), 5)
        self.assertEqual(result["provenance"]["baseline_binary_sha256"], self.binary_sha)
        self.assertEqual(receipt["snapshot_sha256"], snapshot.file_sha256(self.destination))
        for case in result["cases"]:
            self.assertEqual(case["resources"], [{"id": CASE_NAMES[case["name"]],
                "properties": {"observed_at": "provider-time", "array": [2, 1]}}])
            self.assertEqual(case["edges"], [{"from_id": CASE_NAMES[case["name"]], "to_id": "parent",
                                              "properties": {"kept": True}}])
            self.assertEqual(case["scopes"][0]["last_page"], 2)
            self.assertIs(case["scopes"][0]["complete"], True)

    def test_unrelated_binary_cannot_mint_a_new_oracle_from_valid_evidence(self):
        other = self.root / "unrelated-binary"
        other.write_bytes(b"different collector")
        self.rejected("binary differs", other)

    def test_every_case_must_use_the_same_binary_even_with_rehashed_receipts(self):
        for name in CASE_NAMES.values():
            with self.subTest(name=name):
                self.commands[name]["binary_sha256"] = "f" * 64
                self.bind_command(name)
                self.save_summary()
                self.rejected("different command or binary")
                self.commands[name]["binary_sha256"] = self.binary_sha
                self.bind_command(name)

    def test_changed_current_source_cannot_reuse_prior_assertions(self):
        (self.root / "src/main.rs").write_text("changed collector\n")
        self.rejected("different collector source")

    def test_both_architecture_fixtures_are_frozen_inputs(self):
        for name in ["cloudcontrol-architecture.json", "cloudcontrol-parents.json"]:
            path = self.root / "crates/aws/tests/fixtures" / name
            original = path.read_bytes()
            with self.subTest(name=name):
                path.write_text("changed fixture\n")
                self.rejected("different catalogs or architecture fixtures")
                path.write_bytes(original)

    def test_parent_and_merged_catalog_digests_cannot_be_substituted(self):
        for field in ["parent_catalog_sha256", "runtime_catalog_canonical_sha256"]:
            original = self.execution[field]
            with self.subTest(field=field):
                self.execution[field] = "f" * 64
                self.save_summary()
                self.rejected("different catalogs or architecture fixtures")
                self.execution[field] = original

    def test_catalog_response_must_contain_the_exact_parent_overlay(self):
        command = json.loads((self.evidence / "catalog.json").read_text())
        command["result"]["cloud_control"]["resource_types"].pop()
        self.execution["catalog_command"]["sha256"] = write_json(self.evidence / "catalog.json", command)
        self.save_summary()
        self.rejected("exact merged runtime catalog")

    def test_missing_parent_assertion_cannot_hide_behind_a_large_check_count(self):
        for check in self.summary["checks"]:
            if check["check"] == "parents:exact_child_count":
                check["check"] = "unrelated passing assertion"
        self.save_summary()
        self.rejected("required independent or provenance assertions")

    def test_changed_command_receipt_must_match_the_original_summary_digest(self):
        self.commands["sources"]["result"]["scans"][0]["status"] = "partial"
        write_json(self.evidence / "sources.json", self.commands["sources"])
        self.rejected("not bound to the asserted baseline")

    def test_missing_or_extra_case_binding_is_rejected(self):
        bindings = copy.deepcopy(self.execution["scan_commands"])
        for change in ["missing", "extra"]:
            with self.subTest(change=change):
                self.execution["scan_commands"] = copy.deepcopy(bindings)
                if change == "missing":
                    self.execution["scan_commands"].pop("parents")
                else:
                    self.execution["scan_commands"]["unrelated"] = copy.deepcopy(bindings["native"])
                self.save_summary()
                self.rejected("exactly the five collector commands")

    def test_duplicate_scan_cannot_be_relabelled_as_a_second_case(self):
        self.commands["sources"]["result"]["scans"][0]["scan_id"] = "synthetic-native"
        self.bind_command("sources")
        self.save_summary()
        self.rejected("five distinct baseline-bound scan identities")

    def test_extra_persisted_scan_cannot_hide_outside_the_five_command_receipts(self):
        with sqlite3.connect(self.evidence / "inventory.sqlite") as connection:
            connection.execute("INSERT INTO scans VALUES ('unrelated-scan','complete')")
        self.rejected("exactly the five asserted scan identities and states")

    def test_case_selection_cannot_change_even_when_receipt_hash_is_updated(self):
        self.commands["parents"]["arguments"][-1] = "AWS::EKS::Cluster"
        self.bind_command("parents")
        self.save_summary()
        self.rejected("different collection arguments or binary")

    def test_stale_parent_command_alias_cannot_disagree_with_the_bound_five_cases(self):
        self.execution["parent_command"]["scan_id"] = "unrelated-parent-scan"
        self.save_summary()
        self.rejected("command bindings disagree")

    def test_failed_or_inconsistent_command_cannot_serve_as_a_terminal_baseline(self):
        original = copy.deepcopy(self.commands["parents"])
        changes = [
            lambda value: value.update(exit_code=1),
            lambda value: value.update(exit_code=False),
            lambda value: value["result"].update(status="partial"),
            lambda value: value["result"].update(write_error="synthetic write failure"),
            lambda value: value["result"]["scans"][0].update(error="synthetic scan failure"),
        ]
        for index, change in enumerate(changes):
            with self.subTest(index=index):
                self.commands["parents"] = copy.deepcopy(original)
                change(self.commands["parents"])
                self.bind_command("parents")
                self.save_summary()
                self.rejected("terminal")

    def test_expected_partial_case_preserves_failed_scopes_and_failures(self):
        command = self.commands["sources"]
        command["exit_code"] = 2
        command["result"]["status"] = "partial"
        command["result"]["scans"][0]["status"] = "partial"
        self.bind_command("sources")
        self.save_summary()
        with sqlite3.connect(self.evidence / "inventory.sqlite") as connection:
            connection.execute("UPDATE scans SET status='partial' WHERE scan_id='synthetic-sources'")
            connection.execute("UPDATE scope_checkpoints SET complete=0,failed=1 WHERE scan_id='synthetic-sources'")
            connection.execute("INSERT INTO collection_failures VALUES (?,?,?,?)", (
                "synthetic-sources", "scope-sources", "access_denied", "literal expected provider error"))
        self.freeze()
        result = json.loads(self.destination.read_text())
        case = next(value for value in result["cases"] if value["name"] == "sources")
        self.assertEqual(case["status"], "partial")
        self.assertEqual(case["failures"], [{"scope_key": "scope-sources", "category": "access_denied",
                                            "message": "literal expected provider error"}])
        self.assertIs(case["scopes"][0]["failed"], True)
        self.assertIs(case["scopes"][0]["complete"], False)

    def test_missing_persisted_scan_or_mismatched_state_cannot_be_frozen(self):
        with sqlite3.connect(self.evidence / "inventory.sqlite") as connection:
            connection.execute("UPDATE scans SET status='partial' WHERE scan_id='synthetic-parents'")
        self.rejected("disagrees with persisted scan state")

    def test_source_change_during_read_does_not_publish_a_snapshot(self):
        original = snapshot.observations

        def changed(*args):
            result = original(*args)
            (self.root / "src/main.rs").write_text("changed while freezing\n")
            return result

        with mock.patch.object(snapshot, "observations", changed):
            self.rejected("different collector source")


if __name__ == "__main__":
    unittest.main()
