"""Negative controls for the lexical source and execution provenance audit."""

import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import reconcile_aws_operations as audit


class CallsiteAuditTests(unittest.TestCase):
    def test_comments_strings_chars_and_lifetimes_do_not_create_calls(self):
        source = r'''
use aws_sdk_ec2::Client;
// client.fake().send()
/* outer /* nested client.fake().send() */ comment */
fn collect<'a>(client: &'a Client) {
    let text = "client.fake().send()";
    let raw = r###"client.fake().send() /* not a comment */"###;
    let quote = '\'';
    client.describe_vpcs().set_next_token(Some(format!("{}", text))).send().await;
}
'''
        rows, excluded = audit.calls_in_source("sample.rs", source)
        self.assertEqual(excluded, 0)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["id"], "ec2.describe_vpcs")
        self.assertEqual(rows[0]["function"], "collect")
        self.assertEqual(rows[0]["send_line"], 9)

    def test_qualified_client_assignment_and_parameter_resolve_service(self):
        source = '''
fn one() { let sdk = aws_sdk_config::Client::new(config); sdk.get_status().send(); }
fn two(sdk: &aws_sdk_resourceexplorer2::Client) { sdk.get_status().send(); }
'''
        rows, _ = audit.calls_in_source("sample.rs", source)
        self.assertEqual([row["id"] for row in rows], ["config.get_status", "resourceexplorer2.get_status"])

    def test_changed_sdk_namespace_is_not_inferred_from_old_method_names(self):
        source = "use aws_sdk_eks::Client; fn list(client: &Client) {client.list_clusters().send();}"
        rows, _ = audit.calls_in_source("ecs.rs", source)
        self.assertEqual(rows[0]["id"], "eks.list_clusters")

    def test_channel_sends_are_excluded_and_opaque_empty_sends_fail_closed(self):
        source = "fn emit(sender: Sender) {sender.send(event).await;}"
        self.assertEqual(audit.calls_in_source("sample.rs", source), ([], 1))
        with self.assertRaisesRegex(ValueError, "unrecognized empty send"):
            audit.calls_in_source("sample.rs", "fn collect() {builder.send().await;}")

    def test_unrecognized_client_import_cannot_borrow_an_operation_identity(self):
        with self.assertRaisesRegex(ValueError, "uniquely resolve"):
            audit.calls_in_source("sample.rs", "fn collect(client: OtherClient) {client.describe_vpcs().send();}")

    def test_lexical_bounds_and_incomplete_source_fail_closed(self):
        for value in ['fn x() { "unterminated', 'fn x() { /* unterminated', 'fn x() { r##"unterminated']:
            with self.subTest(value=value), self.assertRaises(ValueError):
                audit.calls_in_source("sample.rs", value)
        with patch.object(audit, "MAX_TOKENS", 2), self.assertRaisesRegex(ValueError, "token budget"):
            audit.calls_in_source("sample.rs", "fn collect() {}")

    def test_missing_function_brace_before_proptest_reports_exact_opening(self):
        source = "\nfn cancelled() {\n    Ok(())\n\nproptest! { fn generated() {} }\n"
        with self.assertRaisesRegex(ValueError, r"parent_cases\.rs:2:16: unclosed Rust delimiter '\{'; expected '\}'"):
            audit.calls_in_source("parent_cases.rs", source)
        complete = source.replace("    Ok(())\n", "    Ok(())\n}\n")
        self.assertEqual(audit.calls_in_source("parent_cases.rs", complete), ([], 0))

    def test_mismatched_closing_delimiter_names_both_source_locations(self):
        source = "fn collect() {\n    let values = [1, 2);\n}\n"
        with self.assertRaisesRegex(ValueError, r"sample\.rs:2:23: unbalanced Rust delimiters: '\)' closes '\[' opened at 2:18; expected '\]'\Z"):
            audit.calls_in_source("sample.rs", source)

    def test_unexpected_closing_delimiter_has_its_path_and_location(self):
        with self.assertRaisesRegex(ValueError, r"sample\.rs:2:1: unexpected closing Rust delimiter '\}'\Z"):
            audit.calls_in_source("sample.rs", "fn collect() {}\n}\n")

    def test_unterminated_literals_and_comments_report_original_path_and_line(self):
        for start, message in [('"', "Rust string"), ('r##"', "Rust raw string"), ('/*', "Rust block comment")]:
            with self.subTest(start=start), self.assertRaisesRegex(ValueError, r"sample\.rs:2:5: unterminated " + message):
                audit.calls_in_source("sample.rs", "fn collect() {\n    " + start + "unfinished\n}")

    def test_client_resolution_failure_includes_callsite_context(self):
        source = "\nfn collect(client: OtherClient) {\n    client.describe_vpcs().send();\n}"
        with self.assertRaisesRegex(ValueError, r"unknown_client\.rs:3:\d+: cannot uniquely resolve"):
            audit.calls_in_source("unknown_client.rs", source)

    def test_parent_model_argument_does_not_create_a_new_sdk_operation(self):
        source = '''
use aws_sdk_cloudcontrol::Client;
async fn collect(client: &Client) {
    client.list_resources().type_name(&request.type_name)
        .set_resource_model(request.resource_model.clone())
        .set_next_token(request.token.clone()).send().await;
}
'''
        rows, excluded = audit.calls_in_source("parents.rs", source)
        self.assertEqual(excluded, 0)
        self.assertEqual([row["id"] for row in rows], ["cloudcontrol.list_resources"])
        self.assertEqual(rows[0]["function"], "collect")

    def test_proptest_raw_byte_c_strings_and_character_braces_remain_masked(self):
        source = r'''
use aws_sdk_cloudcontrol::Client;
proptest! {
    fn property(regex in r#"[{}\"\\]{1,64}"#) {
        let bytes = br###"client.fake().send() } /*"###;
        let text = cr##"client.fake().send() }"##;
        let chars = [b'}', '{', '\'', '\\'];
        client.list_resources().send();
    }
}
'''
        rows, excluded = audit.calls_in_source("property.rs", source)
        self.assertEqual(excluded, 0)
        self.assertEqual([row["id"] for row in rows], ["cloudcontrol.list_resources"])

    def test_source_inventory_keeps_test_module_sdk_calls_in_scope(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / "crates/aws/src"
            directory.mkdir(parents=True)
            (directory / "lib.rs").write_text("use aws_sdk_ec2::Client; fn run() {client.describe_vpcs().send();}")
            (directory / "tests.rs").write_text("use aws_sdk_ec2::Client; #[test] fn example() {client.describe_subnets().send();}")
            sources, calls, excluded = audit.source_inventory(root)
            self.assertEqual(len(sources), 2)
            self.assertEqual({row["id"] for row in calls}, {"ec2.describe_vpcs", "ec2.describe_subnets"})
            self.assertEqual(excluded, 0)


class ExecutionBindingTests(unittest.TestCase):
    def setUp(self):
        self.report = {"source": {"sha256": "a" * 64, "files": []},
                       "cloud_control_provider_dimension": {"catalog_sha256": "b" * 64}}
        self.catalog = {"resource_types": [{"type_name": "AWS::Example::Thing"}]}
        self.matrix = {"operations": [{"id": "example.list", "boto3_service": "example",
                                       "api_operation": "List", "explicit_sdk_send": True}],
                       "credential_provider_boundaries": []}
        checks = ["provenance:embedded_catalog_matches", "provenance:source_unchanged",
                  "provenance:binary_unchanged", "provenance:fixture_unchanged",
                  "coverage:every_operation_attempted", "coverage:no_unexplained_operation_failures"]
        self.summary = {
            "schema_version": 1, "failed_checks": 0, "passed_checks": len(checks),
            "checks": [{"check": name, "passed": True} for name in checks],
            "execution": {"source": copy.deepcopy(self.report["source"]),
                          "catalog_sha256": "b" * 64,
                          "catalog_canonical_sha256": audit.canonical_sha256(self.catalog)},
            "expected_operations": [{"id": "example.list", "service": "example", "operation": "List",
                                     "credential_provider_boundary": False, "status": "sdk_decode_failure"}],
        }

    def validate(self):
        return audit.validate_summary(self.report, self.summary, self.catalog, self.matrix)

    def test_decoding_failure_remains_distinct_from_success(self):
        _, rows = self.validate()
        self.assertEqual(rows[0]["status"], "sdk_decode_failure")

    def test_changed_source_or_descriptor_cannot_reuse_an_old_run(self):
        self.report["source"]["sha256"] = "c" * 64
        with self.assertRaisesRegex(ValueError, "different source or catalog"):
            self.validate()
        self.report["source"]["sha256"] = "a" * 64
        self.catalog["resource_types"][0]["relationships"] = [{"relation": "CHANGED"}]
        with self.assertRaisesRegex(ValueError, "different source or catalog"):
            self.validate()

    def test_provider_boundary_cannot_be_relabelled_as_an_explicit_sdk_send(self):
        self.summary["expected_operations"][0]["credential_provider_boundary"] = True
        with self.assertRaisesRegex(ValueError, "provider classification"):
            self.validate()

    def test_duplicate_or_missing_operation_identity_fails_closed(self):
        self.summary["expected_operations"].append(copy.deepcopy(self.summary["expected_operations"][0]))
        with self.assertRaisesRegex(ValueError, "identity set"):
            self.validate()

    def test_failed_or_missing_provenance_checks_cannot_bind_execution(self):
        self.summary["checks"][0]["passed"] = False
        with self.assertRaisesRegex(ValueError, "complete asserted"):
            self.validate()

    def test_parent_and_merged_catalog_digests_are_required_for_new_evidence(self):
        dimension = self.report["cloud_control_provider_dimension"]
        dimension.update(parent_catalog_sha256="c" * 64, runtime_catalog_canonical_sha256="d" * 64)
        for parent, merged in [(None, None), ("e" * 64, "d" * 64), ("c" * 64, "e" * 64)]:
            self.summary["execution"].update(parent_catalog_sha256=parent, runtime_catalog_canonical_sha256=merged)
            with self.subTest(parent=parent, merged=merged), self.assertRaisesRegex(ValueError, "different parent or merged"):
                self.validate()
        self.summary["execution"].update(parent_catalog_sha256="c" * 64, runtime_catalog_canonical_sha256="d" * 64)
        _, rows = self.validate()
        self.assertEqual(rows[0]["status"], "sdk_decode_failure")


class CatalogAndOutputTests(unittest.TestCase):
    def setUp(self):
        self.root = {"schema_version": 1, "resource_types": [
            {"type_name": "AWS::EKS::Cluster", "list_requires_model": False}],
            "evidence": {"frozen": "root evidence"}}
        self.parents = {"schema_version": 1, "resource_types": [
            {"type_name": "AWS::EKS::Addon", "list_requires_model": True}],
            "parent_plans": [{"type_name": "AWS::EKS::Addon", "parent_type": "AWS::EKS::Cluster",
                              "bindings": [{"source_pointer": "/Name", "model_property": "ClusterName"}]}],
            "evidence": {"frozen": "parent evidence"}}

    def test_runtime_catalog_is_exact_additive_overlay_without_mutating_inputs(self):
        original, parents = copy.deepcopy(self.root), copy.deepcopy(self.parents)
        merged = audit.expected_runtime_catalog(self.root, self.parents)
        self.assertEqual(merged, {**original, "resource_types": [original["resource_types"][0],
            {**parents["resource_types"][0], "parent_workflow": parents["parent_plans"][0]}],
            "parent_evidence": parents["evidence"]})
        self.assertEqual((self.root, self.parents), (original, parents))
        merged["resource_types"][1]["parent_workflow"]["bindings"].clear()
        self.assertEqual(self.parents, parents)

    def test_duplicate_overlapping_missing_and_nested_parent_plans_fail_closed(self):
        for change in ["duplicate", "overlap", "missing", "nested", "no_model"]:
            parent = copy.deepcopy(self.parents)
            if change == "duplicate":
                parent["parent_plans"].append(copy.deepcopy(parent["parent_plans"][0]))
                parent["resource_types"].append(copy.deepcopy(parent["resource_types"][0]))
            elif change == "overlap":
                parent["resource_types"][0]["type_name"] = "AWS::EKS::Cluster"
                parent["parent_plans"][0]["type_name"] = "AWS::EKS::Cluster"
            elif change == "missing":
                parent["parent_plans"].clear()
            elif change == "nested":
                parent["parent_plans"][0]["parent_type"] = "AWS::EKS::Addon"
            else:
                parent["resource_types"][0]["list_requires_model"] = False
            with self.subTest(change=change), self.assertRaises(ValueError):
                audit.expected_runtime_catalog(self.root, parent)

    def test_parent_catalog_requires_frozen_root_file_digest(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / "catalog"
            directory.mkdir()
            root_file = directory / "cloudcontrol-resources.json"
            root_file.write_text(json.dumps(self.root))
            parent_file = directory / "cloudcontrol-parent-resources.json"
            self.parents["evidence"]["root_catalog_sha256"] = "0" * 64
            parent_file.write_text(json.dumps(self.parents))
            with self.assertRaisesRegex(ValueError, "frozen root catalog"):
                audit.catalogs(root)
            self.parents["evidence"]["root_catalog_sha256"] = audit.file_sha256(root_file)
            parent_file.write_text(json.dumps(self.parents))
            _root, root_sha, _parents, parent_sha, merged = audit.catalogs(root)
            self.assertEqual((root_sha, parent_sha), (audit.file_sha256(root_file), audit.file_sha256(parent_file)))
            self.assertEqual(len(merged["resource_types"]), 2)

    def test_historical_matrix_and_v04_v05_receipts_cannot_be_replaced(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ["catalog/aws-operations.json", "artifacts/aws-operation-source-reconciliation-v04.json",
                         "artifacts/aws-operation-source-reconciliation-v05.json"]:
                with self.subTest(name=name), self.assertRaisesRegex(ValueError, "historical evidence"):
                    audit.output_path(root, root / name)
            current = root / "artifacts/aws-operation-source-reconciliation-v06.json"
            self.assertEqual(audit.output_path(root, current), current)
            with self.assertRaises(ValueError):
                audit.output_path(root, root.parent / "outside.json")


class RequestLedgerTests(unittest.TestCase):
    def evidence(self, expected_count):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "requests.jsonl"
            entries = [
                {"service": "example", "operation": "List", "http_status": 200, "phase": "seed:catalog"},
                {"service": "example", "operation": "List", "http_status": 200, "phase": "rust:cloud-control"},
            ]
            path.write_text("".join(json.dumps(row) + "\n" for row in entries))
            return audit.request_evidence(path, [{"service": "example", "operation": "List",
                "attempts": expected_count, "http_statuses": {"200": expected_count},
                "phases": ["rust:cloud-control"]}])

    def test_seed_traffic_is_not_counted_as_rust_execution(self):
        result = self.evidence(1)
        self.assertEqual(result["records"], 2)

    def test_http_summary_cannot_credit_requests_absent_from_the_rust_ledger(self):
        with self.assertRaisesRegex(ValueError, "disagrees with the Rust request ledger"):
            self.evidence(2)


if __name__ == "__main__":
    unittest.main()
