"""Offline evidence-recorder regressions using independent botocore serializers.

Run with the emulator test environment:
  python -m unittest discover -s scripts/emulator -p test_transport.py -v

No credentials are resolved and no network requests are issued.  These tests
verify classification and evidence handling, not AWS operation implementation.
"""

from __future__ import annotations

import contextlib
import io
import json
import re
import tempfile
import unittest
from datetime import datetime, timezone
from email.message import Message
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
from urllib.parse import urlencode, urlsplit

import botocore.session
from botocore.serialize import create_serializer

from transport import Recorder, require_loopback


ROOT = Path(__file__).resolve().parents[2]
SECRET = "CREDENTIAL-AND-CURSOR-CANARY-NOT-FOR-EVIDENCE"


def sample_value(shape, depth=0):
    """Supply only required input fields to botocore's validating serializer."""
    if depth > 12:
        raise ValueError("test model recursion exceeded bound")
    kind = shape.type_name
    if kind == "structure":
        return {name: sample_value(shape.members[name], depth + 1) for name in shape.required_members}
    if kind == "list":
        return [sample_value(shape.member, depth + 1)]
    if kind == "map":
        return {"Fixture": sample_value(shape.value, depth + 1)}
    if kind == "string":
        return shape.enum[0] if shape.enum else "inventory-emulator-fixture"
    if kind in {"integer", "long", "float", "double"}:
        return max(1, shape.metadata.get("min", 0))
    if kind == "boolean":
        return False
    if kind == "timestamp":
        return datetime(2026, 11, 12, tzinfo=timezone.utc)
    if kind == "blob":
        return b"fixture"
    raise ValueError(f"unsupported fixture shape: {kind}")


def sdk_request(session, descriptor, paginated=False):
    """Serialize the operation independently of the recorder's routing logic."""
    model = session.get_service_model(descriptor["boto3_service"])
    operation = model.operation_model(descriptor["api_operation"])
    parameters = sample_value(operation.input_shape) if operation.input_shape else {}
    if paginated:
        members = operation.input_shape.members
        for spelling in descriptor["pagination"]["request_fields"]:
            name = next(name for name in members if name.lower() == spelling.lower())
            parameters[name] = sample_value(members[name])
    request = create_serializer(model.metadata["protocol"]).serialize_to_request(parameters, operation)
    query = request["query_string"]
    query = urlencode(query, doseq=True) if isinstance(query, dict) else query
    path = request["url_path"]
    if query:
        path += ("&" if "?" in path else "?") + query
    body = request["body"]
    if isinstance(body, dict):
        body = urlencode(body, doseq=True).encode()
    elif isinstance(body, str):
        body = body.encode()
    signing = model.metadata.get("signingName", model.metadata["endpointPrefix"])
    headers = Message()
    for name, value in request["headers"].items():
        headers[name] = value
    headers["Authorization"] = (
        f"AWS4-HMAC-SHA256 Credential={SECRET}/20261112/us-east-1/{signing}/aws4_request, "
        "SignedHeaders=host;x-amz-date, Signature=fixture"
    )
    headers["X-Amz-Security-Token"] = SECRET
    return request["method"], path, headers, body, model.metadata["protocol"]


class CatalogWireTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = json.loads((ROOT / "catalog/aws-operations.json").read_text())
        cls.session = botocore.session.get_session()

    def setUp(self):
        self.recorder = Recorder(self.session)

    def test_all_catalog_operations_match_actual_sdk_serialization(self):
        operations = self.catalog["operations"]
        self.assertEqual(len(operations), self.catalog["counts"]["explicit_unique_operations"])
        self.assertEqual(len({row["id"] for row in operations}), len(operations))
        for descriptor in operations:
            with self.subTest(operation=descriptor["id"]):
                method, path, headers, body, protocol = sdk_request(self.session, descriptor)
                classified = self.recorder.classify(method, path, headers, body)
                self.assertEqual(classified["service"], descriptor["boto3_service"])
                self.assertEqual(classified["operation"], descriptor["api_operation"])
                self.assertFalse(classified["pagination_request"])
                self.assertEqual(method, descriptor["wire"]["method"])
                self.assertEqual(protocol, descriptor["wire"]["protocol"])
                pattern = re.escape(descriptor["wire"]["path_template"])
                pattern = re.sub(r"\\\{[^}]+\\\}", "[^/]+", pattern)
                self.assertRegex(urlsplit(path).path, "^" + pattern + "/?$")

    def test_all_catalog_cursor_fields_are_detected_on_the_wire(self):
        cases = [row for row in self.catalog["operations"] if row.get("pagination")]
        self.assertGreater(len(cases), 0)
        for descriptor in cases:
            with self.subTest(operation=descriptor["id"]):
                method, path, headers, body, _ = sdk_request(self.session, descriptor, paginated=True)
                classified = self.recorder.classify(method, path, headers, body)
                self.assertEqual(classified["operation"], descriptor["api_operation"])
                self.assertTrue(classified["pagination_request"])

    def test_credential_provider_boundary_is_classified_separately(self):
        explicit_ids = {row["id"] for row in self.catalog["operations"]}
        for descriptor in self.catalog["credential_provider_boundaries"]:
            with self.subTest(operation=descriptor["id"]):
                self.assertNotIn(descriptor["id"], explicit_ids)
                method, path, headers, body, _ = sdk_request(self.session, descriptor)
                classified = self.recorder.classify(method, path, headers, body)
                self.assertEqual(classified["service"], "sts")
                self.assertEqual(classified["operation"], "AssumeRole")

    def test_secret_headers_and_cursor_values_are_not_retained(self):
        descriptor = next(row for row in self.catalog["operations"] if row["id"] == "cloudcontrol.list_resources")
        method, path, headers, _, _ = sdk_request(self.session, descriptor)
        body = json.dumps({"TypeName": "AWS::SQS::Queue", "NextToken": SECRET, "Credentials": SECRET}).encode()
        classified = self.recorder.classify(method, path, headers, body)
        classified.update(http_status=200, response_bytes=12, response_sha256="fixture-hash")
        self.recorder.append(classified)
        evidence = json.dumps(self.recorder.snapshot())
        self.assertNotIn(SECRET, evidence)
        self.assertNotIn("Authorization", evidence)
        self.assertNotIn("Credentials", evidence)
        self.assertNotIn("body", evidence)
        self.assertEqual(classified["type_name"], "AWS::SQS::Queue")

    def test_request_phase_survives_a_later_command_start(self):
        descriptor = next(row for row in self.catalog["operations"] if row["id"] == "s3.list_buckets")
        method, path, headers, body, _ = sdk_request(self.session, descriptor)
        self.recorder.set_phase("seed:in-flight")
        pending = self.recorder.classify(method, path, headers, body)
        self.recorder.set_phase("rust:next-command")
        pending.update(http_status=200, response_bytes=0, response_sha256="fixture-hash")
        self.recorder.append(pending)
        self.assertEqual(self.recorder.snapshot()[0]["phase"], "seed:in-flight")

    def test_evidence_capacity_refuses_growth_past_bound(self):
        with patch("transport.MAX_REQUESTS", 2):
            self.recorder.append({"operation": "first"})
            self.recorder.append({"operation": "second"})
            with self.assertRaisesRegex(RuntimeError, "capacity"):
                self.recorder.append({"operation": "third"})
        self.assertEqual(len(self.recorder.snapshot()), 2)

    def test_summary_excludes_seed_and_python_probe_operations(self):
        from run import Run

        for phase, operation_id in [
            ("seed:fixture", "dynamodb.describe_table"),
            ("rust:native", "s3.list_buckets"),
            ("probe:assertion", "iam.list_roles"),
            ("rust-probe:branch", "ec2.describe_regions"),
        ]:
            descriptor = next(row for row in self.catalog["operations"] if row["id"] == operation_id)
            method, path, headers, body, _ = sdk_request(self.session, descriptor)
            self.recorder.set_phase(phase)
            entry = self.recorder.classify(method, path, headers, body)
            entry.update(http_status=200, response_bytes=0, response_sha256="fixture-hash")
            self.recorder.append(entry)
        with tempfile.TemporaryDirectory(prefix="inventory-evidence-test-") as directory:
            execution = Run(SimpleNamespace(output=Path(directory), seed_only=True))
            with contextlib.redirect_stdout(io.StringIO()):
                result = execution.summarize(self.recorder, "test version", "https://127.0.0.1:9297")
            self.assertEqual(result["rust_operations_observed"], 2)
            observed = {(row["service"], row["operation"]): row for row in result["operation_evidence"]}
            self.assertEqual(set(observed), {("s3", "ListBuckets"), ("ec2", "DescribeRegions")})
            self.assertEqual(observed[("s3", "ListBuckets")]["phases"], ["rust:native"])
            expected = {row["id"]: row for row in result["expected_operations"]}
            self.assertEqual(expected["s3.list_buckets"]["status"], "collector_success")
            self.assertEqual(expected["ec2.describe_regions"]["status"], "probe_response_unverified")
            self.assertEqual(expected["dynamodb.describe_table"]["status"], "not_exercised")
            self.assertEqual(expected["iam.list_roles"]["status"], "not_exercised")


class SummaryReceiptTests(unittest.TestCase):
    """Wire evidence and a successful typed receipt are independent requirements."""

    @classmethod
    def setUpClass(cls):
        cls.catalog = json.loads((ROOT / "catalog/aws-operations.json").read_text())
        cls.session = botocore.session.get_session()

    def setUp(self):
        self.recorder = Recorder(self.session)

    def record_response(self, operation_id, phase="rust-probe:branch-probes", status=200):
        descriptor = next(row for row in self.catalog["operations"] if row["id"] == operation_id)
        method, path, headers, body, _ = sdk_request(self.session, descriptor)
        self.recorder.set_phase(phase)
        entry = self.recorder.classify(method, path, headers, body)
        entry.update(http_status=status, response_bytes=0, response_sha256="fixture-hash")
        self.recorder.append(entry)

    @staticmethod
    def branch_receipt(*, exit_code=0, assertions=(), explorer=(), name="branch-probes"):
        return {
            "name": name,
            "exit_code": exit_code,
            "result": {
                "status": "partial" if exit_code == 2 else "complete",
                "scope": "explicit_sdk_branch_probes",
                "production_branch_reached": False,
                "assertions": list(assertions),
                "resource_explorer": list(explorer),
            },
        }

    def summarize(self, commands=()):
        from run import Run

        with tempfile.TemporaryDirectory(prefix="inventory-receipt-test-") as directory:
            execution = Run(SimpleNamespace(output=Path(directory), seed_only=True))
            execution.commands.extend(commands)
            with contextlib.redirect_stdout(io.StringIO()):
                result = execution.summarize(self.recorder, "test version", "https://127.0.0.1:9297")
        return result, {row["id"]: row for row in result["expected_operations"]}

    def test_basic_receipt_verifies_only_the_reported_operation(self):
        self.record_response("ec2.describe_regions")
        self.record_response("s3.head_bucket")
        assertion = {
            "operation": "ec2:DescribeRegions", "contains_fixture_region": True, "region_count": 1,
        }
        for exit_code in (0, 2):
            with self.subTest(exit_code=exit_code):
                _, expected = self.summarize([self.branch_receipt(exit_code=exit_code, assertions=[assertion])])
                self.assertEqual(expected["ec2.describe_regions"]["status"], "explicit_sdk_probe_success")
                self.assertEqual(expected["ec2.describe_regions"]["phases"], ["rust-probe:branch-probes"])
                self.assertEqual(expected["s3.head_bucket"]["status"], "probe_response_unverified")

    def test_explorer_receipt_requires_typed_and_semantic_success(self):
        self.record_response("resourceexplorer2.get_default_view")
        probe = {
            "operation": "resource-explorer-2:GetDefaultView",
            "sdk_decoded": True,
            "assertions_passed": True,
            "response": {"matches_fixture": True},
        }
        for exit_code in (0, 2):
            with self.subTest(exit_code=exit_code):
                _, expected = self.summarize([self.branch_receipt(exit_code=exit_code, explorer=[probe])])
                self.assertEqual(expected["resourceexplorer2.get_default_view"]["status"], "explicit_sdk_probe_success")

    def test_failed_exit_cannot_verify_otherwise_successful_receipts(self):
        self.record_response("ec2.describe_regions")
        self.record_response("resourceexplorer2.get_default_view")
        assertion = {"operation": "ec2:DescribeRegions", "contains_fixture_region": True, "region_count": 1}
        probe = {
            "operation": "resource-explorer-2:GetDefaultView", "sdk_decoded": True, "assertions_passed": True,
        }
        for exit_code in (1, -9, None):
            with self.subTest(exit_code=exit_code):
                command = self.branch_receipt(exit_code=exit_code, assertions=[assertion], explorer=[probe])
                _, expected = self.summarize([command])
                for operation_id in ("ec2.describe_regions", "resourceexplorer2.get_default_view"):
                    self.assertEqual(expected[operation_id]["status"], "probe_response_unverified")

    def test_missing_or_invalid_json_cannot_verify_a_response(self):
        self.record_response("ec2.describe_regions")
        self.record_response("resourceexplorer2.get_default_view")
        records = [
            {"name": "branch-probes", "exit_code": 0},
            {"name": "branch-probes", "exit_code": 0, "result": {}},
            {"name": "branch-probes", "exit_code": 0, "result": {"invalid_json": True}},
            {"name": "branch-probes", "exit_code": 0, "result": None},
            {"name": "branch-probes", "exit_code": 0, "result": []},
            {"name": "branch-probes", "timeout_seconds": 90},
        ]
        for command in records:
            with self.subTest(command=command):
                _, expected = self.summarize([command])
                for operation_id in ("ec2.describe_regions", "resourceexplorer2.get_default_view"):
                    self.assertEqual(expected[operation_id]["status"], "probe_response_unverified")

    def test_unrelated_command_cannot_supply_a_branch_receipt(self):
        self.record_response("ec2.describe_regions")
        self.record_response("resourceexplorer2.get_default_view")
        command = self.branch_receipt(
            name="native",
            assertions=[{"operation": "ec2:DescribeRegions", "contains_fixture_region": True, "region_count": 1}],
            explorer=[{
                "operation": "resource-explorer-2:GetDefaultView", "sdk_decoded": True, "assertions_passed": True,
            }],
        )
        _, expected = self.summarize([command])
        self.assertEqual(expected["ec2.describe_regions"]["status"], "probe_response_unverified")
        self.assertEqual(expected["resourceexplorer2.get_default_view"]["status"], "probe_response_unverified")

    def test_incomplete_explorer_receipt_is_unverified(self):
        self.record_response("resourceexplorer2.get_default_view")
        for fields in ({}, {"sdk_decoded": True}, {"assertions_passed": True}):
            with self.subTest(fields=fields):
                probe = {"operation": "resource-explorer-2:GetDefaultView", **fields}
                _, expected = self.summarize([self.branch_receipt(explorer=[probe])])
                self.assertEqual(expected["resourceexplorer2.get_default_view"]["status"], "probe_response_unverified")

    def test_failed_semantic_assertions_remain_failures_after_http_success(self):
        self.record_response("resourceexplorer2.get_default_view", phase="rust:sources")
        self.record_response("resourceexplorer2.get_default_view")
        probe = {
            "operation": "resource-explorer-2:GetDefaultView", "sdk_decoded": True, "assertions_passed": False,
            "response": {"matches_fixture": False},
        }
        _, expected = self.summarize([self.branch_receipt(exit_code=2, explorer=[probe])])
        row = expected["resourceexplorer2.get_default_view"]
        self.assertEqual(row["status"], "explicit_sdk_probe_assertion_failed")
        self.assertEqual(row["attempts"], 2)
        self.assertEqual(row["http_statuses"], {"200": 2})

    def test_service_error_is_not_mislabeled_as_a_decode_failure(self):
        self.record_response("resourceexplorer2.get_view", status=403)
        probe = {
            "operation": "resource-explorer-2:GetView", "sdk_decoded": False,
            "sdk_error_kind": "service_error", "http_status": 403,
            "error_chain": ["service error", "AccessDeniedException: fixture denied"],
        }
        _, expected = self.summarize([self.branch_receipt(exit_code=2, explorer=[probe])])
        row = expected["resourceexplorer2.get_view"]
        self.assertEqual(row["status"], "explicit_sdk_probe_error")
        self.assertEqual(row["http_statuses"], {"403": 1})
        self.assertEqual(row["sdk_decode_error_chain"], probe["error_chain"])

    def test_decode_failure_overrides_production_and_probe_http_success(self):
        self.record_response("resourceexplorer2.get_index", phase="rust:discovery")
        self.record_response("resourceexplorer2.get_index")
        for kind in ("success_response_rejected", "response_decode_failure"):
            with self.subTest(sdk_error_kind=kind):
                probe = {
                    "operation": "resource-explorer-2:GetIndex", "sdk_decoded": False,
                    "sdk_error_kind": kind, "http_status": 200,
                    "error_chain": ["response rejected", "expected JSON string, got number"],
                }
                _, expected = self.summarize([self.branch_receipt(exit_code=2, explorer=[probe])])
                row = expected["resourceexplorer2.get_index"]
                self.assertEqual(row["status"], "sdk_decode_failure")
                self.assertEqual(row["attempts"], 2)
                self.assertEqual(row["http_statuses"], {"200": 2})
                self.assertEqual(row["sdk_decode_error_chain"], probe["error_chain"])

    def test_success_receipt_does_not_turn_python_requests_into_rust_coverage(self):
        self.record_response("ec2.describe_regions", phase="seed:fixture")
        self.record_response("ec2.describe_regions", phase="probe:assertion")
        command = self.branch_receipt(assertions=[{
            "operation": "ec2:DescribeRegions", "contains_fixture_region": True, "region_count": 1,
        }])
        result, expected = self.summarize([command])
        self.assertEqual(result["rust_operations_observed"], 0)
        self.assertEqual(expected["ec2.describe_regions"]["status"], "not_exercised")
        self.assertEqual(expected["ec2.describe_regions"]["attempts"], 0)


class EndpointGuardTests(unittest.TestCase):
    def test_loopback_ipv4_and_ipv6_are_accepted(self):
        for endpoint in ["https://127.0.0.1:9297", "https://[::1]:9297/"]:
            with self.subTest(endpoint=endpoint):
                self.assertEqual(require_loopback(endpoint).port, 9297)

    def test_ambiguous_or_external_endpoints_are_rejected(self):
        for endpoint in [
            "https://aws.amazon.com:443", "https://169.254.169.254:80",
            "http://127.0.0.1:9297", "https://localhost:9297",
            "https://127.0.0.1", "https://127.0.0.1:0", "https://127.0.0.1:65536",
            "https://user:password@127.0.0.1:9297", "https://127.0.0.1:9297/path",
            "https://127.0.0.1:9297?query=1", "https://127.0.0.1:9297#fragment",
        ]:
            with self.subTest(endpoint=endpoint), self.assertRaises(ValueError):
                require_loopback(endpoint)


if __name__ == "__main__":
    unittest.main()
