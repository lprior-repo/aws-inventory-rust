"""Independent SQL/content negative controls; no CLI, server or Cargo runs."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import sqlite3
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import verify_delivery as verifier

SCAN = "fixture-scan"
RESOURCE = "arn:aws:ec2:us-east-1:111111111111:instance/i-1"
REFERENCE = "arn:aws:ec2:us-east-1:111111111111:vpc/vpc-ref"


class DeliveryFixture(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name)
        self.connection = sqlite3.connect(":memory:")
        self.addCleanup(self.connection.close)
        self.connection.executescript("""
            CREATE TABLE scans(scan_id TEXT,status TEXT,started_at INTEGER,finished_at INTEGER);
            CREATE TABLE scan_resources(scan_id TEXT,id TEXT,resource_json TEXT);
            CREATE TABLE scan_edges(scan_id TEXT,from_id TEXT,to_id TEXT,relation TEXT,source TEXT,edge_json TEXT);
            CREATE TABLE scope_checkpoints(scan_id TEXT,scope_key TEXT,last_page INTEGER,
                complete INTEGER,failed INTEGER,account_id TEXT,region TEXT,collector TEXT,parent_id TEXT,source TEXT);
            CREATE TABLE resource_observations(scan_id TEXT,scope_key TEXT,page_number INTEGER,
                resource_id TEXT,resource_json TEXT);
            CREATE TABLE edge_observations(scan_id TEXT,scope_key TEXT,page_number INTEGER,
                from_id TEXT,to_id TEXT,relation TEXT,edge_json TEXT);
            CREATE TABLE collection_failures(scan_id TEXT,scope_key TEXT,category TEXT,message TEXT);
        """)
        self.connection.executemany("INSERT INTO scans VALUES (?,?,?,?)", [
            (SCAN, "partial", 100, 150), ("another-scan", "complete", 200, 250)])
        self.data_records = []
        self.scopes = []
        resources, edges = [], []
        for key, source, collector, page, complete, failed in [
            ("a-direct", "direct", "ec2", 2, True, False),
            ("b-config", "aws_config", "config", 1, False, True),
        ]:
            scope = {"account_id": "111111111111", "region": "us-east-1",
                     "collector": collector, "parent_id": None, "source": source}
            self.scopes.append(scope)
            self.connection.execute("INSERT INTO scope_checkpoints VALUES (?,?,?,?,?,?,?,?,?,?)",
                (SCAN, key, page, int(complete), int(failed), "111111111111", "us-east-1", collector, None, source))
            resource = {"id": RESOURCE, "kind": "aws_ec2_instance", "account_id": "111111111111",
                        "region": "us-east-1", "name": 'quoted "fixture"', "source": source,
                        "observed_at": 120, "properties": {"evidence": source}, "tags": {"role": "test"}}
            edge = {"from_id": RESOURCE, "to_id": REFERENCE, "relation": "IN_VPC",
                    "source": source, "observed_at": 120, "properties": {"evidence": source}}
            self.connection.execute("INSERT INTO resource_observations VALUES (?,?,?,?,?)",
                (SCAN, key, page, RESOURCE, json.dumps(resource)))
            self.connection.execute("INSERT INTO edge_observations VALUES (?,?,?,?,?,?,?)",
                (SCAN, key, page, RESOURCE, REFERENCE, "IN_VPC", json.dumps(edge)))
            observation = {"scope": scope, "page_number": page}
            resources.append({"record": "resource", "data": {"resource": resource, "observation": observation}})
            edges.append({"record": "edge", "data": {"edge": edge, "observation": observation}})
        self.connection.executemany("INSERT INTO scan_resources VALUES (?,?,?)", [
            (SCAN, RESOURCE, json.dumps(resources[0]["data"]["resource"])),
            ("another-scan", "must-not-leak", "{}")])
        self.connection.execute("INSERT INTO scan_edges VALUES (?,?,?,?,?,?)",
            (SCAN, RESOURCE, REFERENCE, "IN_VPC", "direct", json.dumps(edges[0]["data"]["edge"])))
        self.connection.execute("INSERT INTO collection_failures VALUES (?,?,?,?)",
                                (SCAN, "b-config", "AccessDenied", "fixture denial"))
        self.data_records = resources + edges + [
            {"record": "scope", "data": {"scope": self.scopes[0], "last_page": 2, "complete": True, "failed": False}},
            {"record": "scope", "data": {"scope": self.scopes[1], "last_page": 1, "complete": False, "failed": True}},
            {"record": "failure", "data": {"scope": self.scopes[1], "category": "AccessDenied", "message": "fixture denial"}},
        ]
        # The expected model is authored here, not returned by verifier.expected.
        self.model = {
            "counts": {"resources": 1, "edges": 1, "nodes": 2, "unresolved": 1},
            "scan": {"scan_id": SCAN, "status": "partial", "started_at": 100, "finished_at": 150,
                     "resources": 1, "edges": 1, "completed_scopes": 1, "failed_scopes": 1},
            "observations": {"resource": 2, "edge": 2, "scope": 2, "failure": 1},
            "node_rows": [[RESOURCE, SCAN, "observed", f"{SCAN}::{RESOURCE}",
                           "aws_ec2_instance", "111111111111", "us-east-1", 'quoted "fixture"',
                           '{"evidence":"direct"}', '{"role":"test"}', "direct", 120, 12, 1],
                          [REFERENCE, SCAN, "unresolved", f"{SCAN}::{REFERENCE}",
                           None, None, None, None, None, None, None, None, 4, 1]],
            "edge_rows": [[RESOURCE, REFERENCE, "RELATES_TO", "IN_VPC", "direct", SCAN, SCAN,
                           120, '{"evidence":"direct"}', 4, f"{SCAN}::{RESOURCE}", f"{SCAN}::{REFERENCE}"]],
        }
        self.counts = {"resources": 2, "edges": 2, "scopes": 2, "failures": 1, "records": 7}
        self.header = {"record": "header", "data": {"artifact": "aws-inventory.snapshot", "version": 1,
                       "provenance": "all_observations", "scan": self.model["scan"]}}
        self.footer = {"record": "summary", "data": self.counts}

    def records(self):
        return copy.deepcopy([self.header, *self.data_records, self.footer])

    def publish(self, records=None):
        content = "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in
                          (self.records() if records is None else records)).encode()
        path = self.path / "inventory.jsonl"
        path.write_bytes(content)
        summary = {"scan_id": SCAN, "status": "partial", "format": "jsonl",
                   "provenance": "all_observations", **self.counts,
                   "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()}
        return {"version": 1, "scan_id": SCAN, "status": "partial", "collection_status": "partial",
                "delivery_status": "complete", "directory": str(self.path),
                "outputs": {"inventory": {"path": str(path), "summary": summary},
                            "falkor": {"plan": str(self.path / "graph.jsonl"),
                                "export": {"graph": "test_graph", "scan_id": SCAN, "resources": 1, "edges": 1, "batches": 5},
                                "push": {"applied_batches": 5, "skipped_batches": 0, "attempts": 5,
                                         "retries": 0, "reconnects": 0}}}}

    def graph_responses(self):
        return copy.deepcopy([self.model["node_rows"], self.model["edge_rows"],
                              [[SCAN, "partial", 1, 1, 4, 1]], [[3]]])


class InventoryContentTests(DeliveryFixture):
    def test_independent_sql_model_and_full_records_match_authored_expectations(self):
        self.assertEqual(verifier.expected(self.connection, SCAN), self.model)
        self.assertEqual(list(verifier.expected_records(self.connection, SCAN)), self.data_records)
        result = verifier.verify_inventory(self.publish(), self.model, self.connection)
        self.assertTrue(result["exact_sql_payload_and_provenance_match"])
        self.assertEqual(result["counts"], self.counts)

    def test_same_count_payload_or_scope_page_changes_fail_with_recomputed_file_hash(self):
        changes = [
            lambda rows: rows[1]["data"]["resource"]["properties"].update(evidence="wrong"),
            lambda rows: rows[1]["data"]["observation"].update(page_number=1),
            lambda rows: rows[1]["data"]["observation"]["scope"].update(account_id="222222222222"),
            lambda rows: rows[3]["data"]["edge"].update(source="aws_config"),
            lambda rows: rows[5]["data"].update(failed=True),
            lambda rows: rows[7]["data"].update(message="a different denial"),
        ]
        for change in changes:
            with self.subTest(change=change):
                records = self.records()
                change(records)
                with self.assertRaisesRegex(RuntimeError, "SQL payload/provenance"):
                    verifier.verify_inventory(self.publish(records), self.model, self.connection)

    def test_header_provenance_identity_status_version_and_footer_are_not_count_only(self):
        changes = [
            lambda rows: rows[0]["data"].update(provenance="normalized"),
            lambda rows: rows[0]["data"]["scan"].update(scan_id="another-scan"),
            lambda rows: rows[0]["data"]["scan"].update(status="complete"),
            lambda rows: rows[0]["data"].update(version=2),
            lambda rows: rows[-1]["data"].update(records=8),
            lambda rows: rows.reverse(),
        ]
        for change in changes:
            with self.subTest(change=change):
                records = self.records()
                change(records)
                with self.assertRaises(RuntimeError):
                    verifier.verify_inventory(self.publish(records), self.model, self.connection)

    def test_extra_footer_or_trailing_data_cannot_hide_behind_valid_counts(self):
        records = self.records() + [copy.deepcopy(self.footer)]
        with self.assertRaisesRegex(RuntimeError, "unexpected data after"):
            verifier.verify_inventory(self.publish(records), self.model, self.connection)

    def test_receipt_hash_bytes_or_provenance_must_match_artifact_and_sql(self):
        for field, value in [("sha256", "0" * 64), ("bytes", 0), ("provenance", "normalized")]:
            with self.subTest(field=field):
                receipt = self.publish()
                receipt["outputs"]["inventory"]["summary"][field] = value
                with self.assertRaises(RuntimeError):
                    verifier.verify_inventory(receipt, self.model, self.connection)

    def test_missing_scope_join_cannot_silently_drop_a_stored_observation(self):
        receipt = self.publish()
        self.connection.execute("DELETE FROM scope_checkpoints WHERE scope_key='b-config'")
        with self.assertRaises(RuntimeError):
            verifier.verify_inventory(receipt, self.model, self.connection)


class GraphIdentityTests(DeliveryFixture):
    def check_graph(self, responses):
        with patch.object(verifier, "graph_rows", side_effect=responses):
            return verifier.verify_graph(Path("redis-cli"), 1234, "test_graph", self.model)

    def test_exact_graph_identity_is_accepted(self):
        result = self.check_graph(self.graph_responses())
        self.assertTrue(result["exact_sql_identity_and_provenance_match"])
        self.assertTrue(result["exact_sql_projected_properties_match"])
        self.assertTrue(result["exact_property_and_label_cardinality_match"])
        self.assertEqual(result["nodes"], 2)

    def test_same_count_wrong_node_identity_scope_key_or_resolution_fails(self):
        for column, value in [(0, "wrong-resource"), (1, "another-scan"), (2, "unresolved"), (3, "wrong-key")]:
            with self.subTest(column=column):
                responses = self.graph_responses()
                responses[0][0][column] = value
                with self.assertRaisesRegex(RuntimeError, "node IDs"):
                    self.check_graph(responses)

    def test_every_observed_node_property_is_checked(self):
        for column, value in [
                (4, "aws_ec2_volume"), (5, "222222222222"), (6, "eu-west-1"),
                (7, "same count, wrong name"), (8, '{"evidence":"wrong"}'),
                (9, '{"role":"changed"}'), (10, "aws_config"), (11, 121)]:
            with self.subTest(column=column):
                responses = self.graph_responses()
                responses[0][0][column] = value
                with self.assertRaisesRegex(RuntimeError, "node IDs.*projected properties"):
                    self.check_graph(responses)

    def test_missing_observed_fields_cannot_be_hidden_by_extra_property(self):
        # Keep the reported property count unchanged to model an extra unknown
        # field replacing any one expected property.
        for column in range(4, 12):
            with self.subTest(column=column):
                responses = self.graph_responses()
                responses[0][0][column] = None
                with self.assertRaisesRegex(RuntimeError, "projected properties"):
                    self.check_graph(responses)

    def test_unresolved_endpoints_cannot_inherit_observed_resource_properties(self):
        for column, value in [(4, "guessed-kind"), (5, "111111111111"), (6, "us-east-1"),
                              (7, ""), (8, "null"), (9, "{}"), (10, "direct"), (11, 120)]:
            with self.subTest(column=column):
                responses = self.graph_responses()
                responses[0][1][column] = value
                with self.assertRaisesRegex(RuntimeError, "projected properties"):
                    self.check_graph(responses)

    def test_extra_or_missing_properties_and_extra_labels_are_rejected(self):
        for row, column, value in [(0, 12, 11), (0, 12, 13), (0, 13, 2),
                                    (1, 12, 3), (1, 12, 5), (1, 13, 2)]:
            with self.subTest(row=row, column=column, value=value):
                responses = self.graph_responses()
                responses[0][row][column] = value
                with self.assertRaisesRegex(RuntimeError, "node IDs"):
                    self.check_graph(responses)

    def test_nested_json_is_compared_structurally_and_ordered_arrays_stay_ordered(self):
        content = {"alpha": [True, None, {"quoted": 'snowman ☃ / " \\ \n'}],
                   "zeta": {"integer": 9007199254740993, "fraction": 0.25}}
        self.model["node_rows"][0][8] = verifier.canonical(content)
        responses = self.graph_responses()
        responses[0][0][8] = json.dumps({"zeta": content["zeta"], "alpha": content["alpha"]},
                                      ensure_ascii=True, indent=2)
        self.assertTrue(self.check_graph(responses)["exact_sql_projected_properties_match"])
        for change in [lambda value: value["alpha"].reverse(),
                       lambda value: value["zeta"].update(integer=9007199254740992),
                       lambda value: value["zeta"].update(integer=9007199254740993.0),
                       lambda value: value["alpha"].__setitem__(0, 1)]:
            with self.subTest(change=change):
                wrong = copy.deepcopy(content)
                change(wrong)
                responses[0][0][8] = json.dumps(wrong)
                with self.assertRaisesRegex(RuntimeError, "projected properties"):
                    self.check_graph(responses)

    def test_json_null_string_is_distinct_from_absent_property(self):
        self.model["node_rows"][0][8] = "null"
        self.assertTrue(self.check_graph(self.graph_responses())["exact_sql_projected_properties_match"])
        for value in [None, '"null"', "{}"]:
            with self.subTest(value=value):
                responses = self.graph_responses()
                responses[0][0][8] = value
                with self.assertRaisesRegex(RuntimeError, "projected properties"):
                    self.check_graph(responses)

    def test_invalid_duplicate_or_non_finite_json_cannot_hide_in_properties(self):
        for value in ['{"evidence":"wrong","evidence":"direct"}',
                      '{"outer":{"a":0,"a":1}}', "NaN", "Infinity", "1e9999", "{", 120]:
            with self.subTest(value=value):
                responses = self.graph_responses()
                responses[0][0][8] = value
                with self.assertRaises(RuntimeError):
                    self.check_graph(responses)

    def test_same_count_wrong_edge_endpoint_type_relation_or_source_fails(self):
        for column, value in [(0, REFERENCE), (1, RESOURCE), (2, "OTHER"), (3, "WRONG"),
                              (4, "aws_config"), (5, "another-scan"), (6, "another-scan")]:
            with self.subTest(column=column):
                responses = self.graph_responses()
                responses[1][0][column] = value
                with self.assertRaisesRegex(RuntimeError, "edge endpoints"):
                    self.check_graph(responses)

    def test_all_edge_properties_and_endpoint_keys_are_checked(self):
        for column, value in [(7, 121), (7, None), (8, '{"evidence":"wrong"}'),
                              (8, None), (9, 3), (9, 5), (10, "wrong-from-key"),
                              (11, "wrong-to-key")]:
            with self.subTest(column=column, value=value):
                responses = self.graph_responses()
                responses[1][0][column] = value
                with self.assertRaisesRegex(RuntimeError, "edge endpoints.*projected properties"):
                    self.check_graph(responses)

    def test_edge_property_json_formatting_does_not_change_semantics(self):
        responses = self.graph_responses()
        responses[1][0][8] = '{  "evidence" : "direct" }'
        self.assertTrue(self.check_graph(responses)["exact_sql_projected_properties_match"])

    def test_all_marker_properties_and_labels_are_checked(self):
        for column, value in [(0, "wrong-scan"), (1, "complete"), (2, 2), (3, 2),
                              (4, 3), (4, 5), (5, 2)]:
            with self.subTest(column=column, value=value):
                responses = self.graph_responses()
                responses[2][0][column] = value
                with self.assertRaisesRegex(RuntimeError, "scan marker"):
                    self.check_graph(responses)

    def test_ids_with_quotes_delimiters_and_unicode_are_data_not_query_text(self):
        identifier = "arn:aws:example:::resource/name::'quoted'/%☃/../target"
        self.model["node_rows"][0][0] = identifier
        self.model["node_rows"][0][3] = f"{SCAN}::{identifier}"
        self.model["edge_rows"][0][0] = identifier
        self.model["edge_rows"][0][10] = f"{SCAN}::{identifier}"
        responses = self.graph_responses()
        with patch.object(verifier, "graph_rows", side_effect=responses) as query:
            self.assertTrue(verifier.verify_graph(Path("redis-cli"), 1234, "test_graph",
                                                 self.model)["exact_sql_projected_properties_match"])
            for call in query.call_args_list:
                self.assertNotIn(identifier, call.args[3])
                self.assertNotIn(SCAN, call.args[3])

    def test_wrong_marker_or_extra_unlabelled_node_is_rejected(self):
        for index, value in [(2, [[SCAN, "complete", 1, 1, 4, 1]]), (3, [[4]])]:
            responses = self.graph_responses()
            responses[index] = value
            with self.assertRaisesRegex(RuntimeError, "scan marker"):
                self.check_graph(responses)

    def test_compact_tagged_cells_and_malformed_noncompact_responses_fail(self):
        good = [["count(r)"], [[2]], ["Query internal execution time: 0.1 milliseconds"]]
        self.assertEqual(verifier.decode_graph_rows(good, 1), [[2]])
        for response in [
            [[[1, "count(r)"]], [[[3, 2]]], []],
            [["count(r)"], [[[3, 2]]], []],
            [["count(r)"], [[True]], []],
            [["count(r)"], [[2, 3]], []],
            [["count(r)"], [[2]], "not statistics"],
        ]:
            with self.subTest(response=response), self.assertRaises(RuntimeError):
                verifier.decode_graph_rows(response, 1)
        self.assertEqual(verifier.decode_graph_rows([["r.name"], [[None]], []], 1), [[None]])
        with self.assertRaises(RuntimeError):
            verifier.decode_graph_rows(good, 1, maximum=0)

    def test_graph_decoder_bounds_full_json_properties_without_rejecting_valid_large_values(self):
        value = "x" * 65_537
        self.assertEqual(verifier.decode_graph_rows([["r.properties_json"], [[value]], []], 1), [[value]])
        value = "x" * (verifier.MAX_RECORD_BYTES + 1)
        with self.assertRaisesRegex(RuntimeError, "scalar row"):
            verifier.decode_graph_rows([["r.properties_json"], [[value]], []], 1)


class NormalizedProjectionTests(DeliveryFixture):
    def test_projection_payload_budget_counts_utf8_bytes_and_total(self):
        with patch.object(verifier, "MAX_RECORD_BYTES", 4), \
             patch.object(verifier, "MAX_ARTIFACT_BYTES", 6):
            self.assertEqual(verifier.projection_budget("☃", 1), 4)
            self.assertEqual(verifier.projection_budget("a", 5), 6)
            with self.assertRaisesRegex(RuntimeError, "byte bound"):
                verifier.projection_budget("☃☃", 0)
            with self.assertRaisesRegex(RuntimeError, "byte bound"):
                verifier.projection_budget("☃", 4)

    def test_combined_normalized_resource_and_edge_bytes_are_bounded(self):
        resource_bytes = len(self.connection.execute(
            "SELECT resource_json FROM scan_resources WHERE scan_id=?", (SCAN,)).fetchone()[0].encode())
        with patch.object(verifier, "MAX_ARTIFACT_BYTES", resource_bytes):
            with self.assertRaisesRegex(RuntimeError, "byte bound"):
                verifier.expected(self.connection, SCAN)

    def test_missing_name_projects_as_empty_string_and_json_null_stays_present(self):
        row = copy.deepcopy(self.data_records[0]["data"]["resource"])
        row.update(name=None, properties=None, tags={})
        self.connection.execute("UPDATE scan_resources SET resource_json=? WHERE scan_id=?",
                                (json.dumps(row), SCAN))
        wanted = copy.deepcopy(self.model)
        wanted["node_rows"][0][7:10] = ["", "null", "{}"]
        self.assertEqual(verifier.expected(self.connection, SCAN), wanted)

    def test_normalized_selection_does_not_use_the_last_or_lowest_priority_observation(self):
        self.connection.execute("UPDATE resource_observations SET resource_json='{}'")
        self.connection.execute("UPDATE edge_observations SET edge_json='{}'")
        self.assertEqual(verifier.expected(self.connection, SCAN), self.model)

    def test_normalized_resource_identity_must_agree_with_sql_key(self):
        row = copy.deepcopy(self.data_records[0]["data"]["resource"])
        row["id"] = "wrong-resource"
        self.connection.execute("UPDATE scan_resources SET resource_json=? WHERE scan_id=?",
                                (json.dumps(row), SCAN))
        with self.assertRaisesRegex(RuntimeError, "resource identity"):
            verifier.expected(self.connection, SCAN)

    def test_normalized_edge_identity_and_source_must_agree_with_sql_key(self):
        for field in ("from_id", "to_id", "relation", "source"):
            with self.subTest(field=field):
                row = copy.deepcopy(self.data_records[2]["data"]["edge"])
                row[field] = "wrong"
                self.connection.execute("UPDATE scan_edges SET edge_json=? WHERE scan_id=?",
                                        (json.dumps(row), SCAN))
                with self.assertRaisesRegex(RuntimeError, "edge identity"):
                    verifier.expected(self.connection, SCAN)

    def test_duplicate_resource_identity_does_not_disappear_in_expected_map(self):
        self.connection.execute("INSERT INTO scan_resources SELECT * FROM scan_resources WHERE scan_id=?", (SCAN,))
        with self.assertRaisesRegex(RuntimeError, "resource identity"):
            verifier.expected(self.connection, SCAN)

    def test_real_resource_resolves_a_referenced_endpoint_without_losing_its_properties(self):
        resource = {"id": REFERENCE, "kind": "aws_ec2_vpc", "account_id": "111111111111",
                    "region": "us-east-1", "name": None, "source": "cloud_control",
                    "observed_at": 119, "properties": {"cidr": "10.0.0.0/16"}, "tags": {}}
        self.connection.execute("INSERT INTO scan_resources VALUES (?,?,?)", (SCAN, REFERENCE, json.dumps(resource)))
        wanted = copy.deepcopy(self.model)
        wanted["counts"].update(resources=2, unresolved=0)
        wanted["scan"]["resources"] = 2
        wanted["node_rows"][1] = [REFERENCE, SCAN, "observed", f"{SCAN}::{REFERENCE}",
                                  "aws_ec2_vpc", "111111111111", "us-east-1", "",
                                  '{"cidr":"10.0.0.0/16"}', "{}", "cloud_control", 119, 12, 1]
        self.assertEqual(verifier.expected(self.connection, SCAN), wanted)


class ReceiptAndCleanupTests(DeliveryFixture):
    def test_failed_replay_receipt_is_rejected_even_with_zero_new_attempts(self):
        receipt = self.publish()
        receipt["outputs"]["falkor"]["push"] = {"applied_batches": 0, "skipped_batches": 5,
            "attempts": 0, "retries": 0, "reconnects": 0}
        for change in [lambda row: row.update(delivery_status="failed"),
                       lambda row: row.update(collection_status="complete"),
                       lambda row: row.update(scan_id="another-scan"),
                       lambda row: row.update(cleanup_error="failed cleanup")]:
            failed = copy.deepcopy(receipt)
            change(failed)
            with self.assertRaisesRegex(RuntimeError, "delivery receipt failed"):
                verifier.validate_receipt(failed, SCAN, "partial")
        verifier.validate_receipt(receipt, SCAN, "partial")
        verifier.verify_push(receipt, self.model, "test_graph", replay=True)
        for field in ["attempts", "retries", "reconnects", "applied_batches"]:
            failed = copy.deepcopy(receipt)
            failed["outputs"]["falkor"]["push"][field] = 1
            with self.assertRaisesRegex(RuntimeError, "checkpoint resume"):
                verifier.verify_push(failed, self.model, "test_graph", replay=True)

    def test_persisted_receipt_cannot_disagree_with_command_output(self):
        receipt = self.publish()
        persisted = copy.deepcopy(receipt)
        persisted["delivery_status"] = "failed"
        (self.path / "delivery.json").write_text(json.dumps(persisted))
        with self.assertRaisesRegex(RuntimeError, "persisted delivery receipt differs"):
            verifier.verify_receipt_file(receipt, self.path)
        (self.path / "delivery.json").write_text(json.dumps(receipt))
        self.assertGreater(verifier.verify_receipt_file(receipt, self.path)["bytes"], 0)

    def test_receipt_and_artifact_paths_cannot_escape_owned_output(self):
        with tempfile.TemporaryDirectory() as outside:
            for field in ("directory", "inventory", "plan"):
                with self.subTest(field=field):
                    receipt = self.publish()
                    if field == "directory":
                        receipt["directory"] = outside
                    elif field == "inventory":
                        receipt["outputs"]["inventory"]["path"] = str(Path(outside) / "inventory.jsonl")
                    else:
                        receipt["outputs"]["falkor"]["plan"] = str(Path(outside) / "graph.jsonl")
                    with self.assertRaisesRegex(RuntimeError, "outside"):
                        verifier.verify_receipt_file(receipt, self.path)

    def test_symlinked_receipt_cannot_read_outside_its_directory(self):
        receipt = self.publish()
        with tempfile.TemporaryDirectory() as outside:
            target = Path(outside) / "delivery.json"
            target.write_text(json.dumps(receipt))
            (self.path / "delivery.json").symlink_to(target)
            with self.assertRaisesRegex(RuntimeError, "persisted delivery receipt points outside"):
                verifier.verify_receipt_file(receipt, self.path)

    def test_symlinked_artifact_cannot_alias_an_external_file(self):
        receipt = self.publish()
        with tempfile.TemporaryDirectory() as outside:
            target = Path(outside) / "graph.jsonl"
            target.write_text("unrelated file")
            (self.path / "graph.jsonl").symlink_to(target)
            with self.assertRaisesRegex(RuntimeError, "artifact points outside"):
                verifier.verify_receipt_file(receipt, self.path)

    def test_native_ok_cleanup_requires_the_key_to_be_absent_afterward(self):
        with patch.object(verifier, "command", side_effect=[1, "OK", 0]) as command:
            verifier.cleanup_graph(Path("redis-cli"), 1234, "test_graph")
            self.assertEqual(command.call_count, 3)
        for replies in [[1, "OK", 1], [1, "OK", False], [1, "Graph removed", 0], [False]]:
            with self.subTest(replies=replies), patch.object(verifier, "command", side_effect=replies):
                with self.assertRaises(RuntimeError):
                    verifier.cleanup_graph(Path("redis-cli"), 1234, "test_graph")

    def test_primary_failure_is_retained_when_cleanup_also_fails(self):
        args = SimpleNamespace(binary=Path("inventory"), database=Path("unused.sqlite"), redis_cli=Path("redis-cli"))
        case = {"scan_id": SCAN, "status": "partial", "passed": False}
        with patch.object(verifier, "command", side_effect=RuntimeError("primary delivery failure")), \
             patch.object(verifier, "cleanup_graph", side_effect=RuntimeError("cleanup failure")):
            with self.assertRaisesRegex(RuntimeError, "primary delivery failure.*cleanup failure") as caught:
                verifier.verify_case(args, self.connection, SCAN, "partial", 1234, self.path, case)
        self.assertEqual(str(caught.exception.__cause__), "primary delivery failure")
        self.assertEqual(case["error"], "primary delivery failure")
        self.assertEqual(case["cleanup_error"], "cleanup failure")
        self.assertFalse(case["cleanup_complete"])
        self.assertFalse(case["passed"])


if __name__ == "__main__":
    unittest.main()
