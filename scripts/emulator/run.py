#!/usr/bin/env python3
"""Run the shipped Rust collector against a fresh, signed AWS Emulator process.

Dependencies are test-only (requirements.txt). All AWS environment settings are
replaced inside this child process; all writes seed an ephemeral loopback server.
Run from the project root; use --help for paths. No live AWS account is contacted.
"""

from __future__ import annotations

import argparse
import importlib
import json
import os
import signal
import sqlite3
import subprocess
import tempfile
import time
import urllib.parse
from collections import Counter
from pathlib import Path

import boto3
from botocore.config import Config

from transport import Recorder, RecordingProxy, require_loopback
from fixture_provenance import canonical_sha256, expected_runtime_catalog, file_sha256, source_fingerprint

ROOT = Path(__file__).resolve().parents[2]
PINNED_COMMIT = "f515581569b98a9dcbd2e686c12f912fdce51bba"
PINNED_VERSION = "1.0.336"


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True, default=str) + "\n")


def type_operation_coverage(entries, descriptors, phase):
    """Keep root/parent read evidence bound to the command that produced it."""
    result = []
    for descriptor in descriptors:
        rows = [row for row in entries if row.get("type_name") == descriptor["type_name"]
                and row.get("phase") == phase and row.get("service") == "cloudcontrol"]
        result.append({"type_name": descriptor["type_name"],
                       "operations": {operation: dict(Counter(str(row["http_status"]) for row in rows
                                                             if row["operation"] == operation))
                                      for operation in ("ListResources", "GetResource")}})
    return result


class Run:
    def __init__(self, args):
        self.args = args
        self.output = args.output.resolve()
        self.output.mkdir(parents=True, exist_ok=True)
        self.checks = []
        self.commands = []
        self.seed_calls = []
        self.seed_reports = {}
        self.execution = None
        self.start = time.monotonic()

    def check(self, name, passed, detail=None):
        self.checks.append({"check": name, "passed": bool(passed), "detail": detail})

    def record(self, name, call):
        try:
            result = call()
            status = result.get("ResponseMetadata", {}).get("HTTPStatusCode") if isinstance(result, dict) else None
            self.seed_calls.append({"name": name, "status": "success", "http_status": status})
            return result
        except Exception as error:
            response = getattr(error, "response", {})
            code = response.get("Error", {}).get("Code", type(error).__name__)
            message = response.get("Error", {}).get("Message", str(error))[:1200]
            self.seed_calls.append({"name": name, "status": "error", "error_code": code, "message": message})
            return None

    def cli(self, name, arguments, recorder, timeout=300, executable=None, phase_prefix="rust:"):
        recorder.set_phase(phase_prefix + name)
        started = time.monotonic()
        try:
            binary = executable or self.args.binary
            result = subprocess.run([str(binary.resolve()), *arguments], cwd=ROOT,
                                    capture_output=True, text=True, timeout=timeout, check=False)
            if len(result.stdout) > 8 * 1024 * 1024 or len(result.stderr) > 2 * 1024 * 1024:
                raise RuntimeError("CLI evidence exceeded byte budget")
            try:
                value = json.loads(result.stdout)
            except ValueError:
                value = {"invalid_json": True, "stdout_prefix": result.stdout[:4096]}
            record = {"name": name, "arguments": arguments, "exit_code": result.returncode,
                      "seconds": round(time.monotonic() - started, 3), "result": value,
                      "stderr": result.stderr[:65536], "binary_sha256": file_sha256(binary.resolve())}
            self.commands.append(record)
            write_json(self.output / (name + ".json"), record)
            print(json.dumps({"phase": name, "exit_code": result.returncode,
                              "status": value.get("status"), "seconds": record["seconds"]}), flush=True)
            self.check(name + ":structured_output", not value.get("invalid_json", False))
            self.check(name + ":no_panic", "panicked at" not in result.stderr)
            return record
        except subprocess.TimeoutExpired:
            record = {"name": name, "timeout_seconds": timeout}
            self.commands.append(record)
            self.check(name + ":bounded_runtime", False, record)
            return record

    def seed(self, session, recorder):
        for module in ("seed_global", "seed_regional", "seed_discovery", "seed_catalog", "seed_parents"):
            recorder.set_phase("seed:" + module)
            try:
                self.seed_reports[module] = importlib.import_module(module).seed(session, self.record)
            except Exception as error:
                self.seed_reports[module] = {"module_error": type(error).__name__, "message": str(error)[:2000]}
        write_json(self.output / "seed-results.json", {"modules": self.seed_reports, "calls": self.seed_calls})
        print(json.dumps({"phase": "seed", "calls": len(self.seed_calls),
                          "errors": sum(call["status"] == "error" for call in self.seed_calls)}), flush=True)

    def collect(self, recorder):
        catalog_path = ROOT / "catalog/cloudcontrol-resources.json"
        fixture_path = ROOT / "crates/aws/tests/fixtures/cloudcontrol-architecture.json"
        catalog = json.loads(catalog_path.read_text())
        self.execution = {
            "catalog_sha256": file_sha256(catalog_path),
            "catalog_canonical_sha256": canonical_sha256(catalog),
            "source": source_fingerprint(ROOT),
            "binary": {"path": str(self.args.binary.resolve()), "sha256": file_sha256(self.args.binary.resolve())},
            "architecture_fixture_sha256": file_sha256(fixture_path),
            "parent_fixture_sha256": file_sha256(ROOT / "crates/aws/tests/fixtures/cloudcontrol-parents.json"),
        }
        embedded = self.cli("catalog", ["catalog"], recorder)
        parent_catalog = json.loads((ROOT / "catalog/cloudcontrol-parent-resources.json").read_text())
        expected_catalog = expected_runtime_catalog(catalog, parent_catalog)
        self.execution["parent_catalog_sha256"] = file_sha256(ROOT / "catalog/cloudcontrol-parent-resources.json")
        self.execution["runtime_catalog_canonical_sha256"] = canonical_sha256(expected_catalog)
        self.execution["collection_catalog_scope"] = "explicit frozen 397 root types; parent workflows verified separately"
        catalog_matches = (embedded.get("exit_code") == 0 and embedded.get("result", {}).get("cloud_control") == expected_catalog
                           and embedded.get("binary_sha256") == self.execution["binary"]["sha256"])
        self.check("provenance:embedded_catalog_matches", catalog_matches)
        if not catalog_matches:
            raise RuntimeError("compiled collector catalog differs from the selected fixture catalog")
        self.execution["catalog_command"] = {"file": "catalog.json", "sha256": file_sha256(self.output / "catalog.json")}
        db = self.output / "inventory.sqlite"
        if db.exists():
            raise RuntimeError("output inventory already exists; use a new output directory")
        common = ["--db", str(db), "--region", "us-east-1", "--attempts", "2", "--max-pages", "128",
                  "--page-size", "1", "--concurrency", "32", "--per-bucket-concurrency", "4", "--rps", "100"]
        native = self.cli("native", ["scan", *common, "--service", "all"], recorder)
        self.check("native:terminal_scan", native.get("exit_code") in (0, 2))
        discovery = self.cli("discovery", ["discover", "--region", "us-east-1", "--organization", "--hierarchy",
                             "--config-aggregators", "--resource-explorer", "--page-size", "1", "--max-pages", "100"], recorder)
        self.check("discovery:terminal_report", discovery.get("exit_code") in (0, 2))
        self.cli("sources", ["scan", *common, "--service", "config,resource-explorer", "--discover-sources"], recorder)
        self.cli("cloud-control", ["scan", *common, "--service", "cloud-control", "--cloud-control",
                                  "--resource-type", ",".join(row["type_name"] for row in catalog["resource_types"])], recorder)
        self.cli("parents", ["scan", *common, "--service", "cloud-control", "--cloud-control",
                             "--resource-type", ",".join(row["type_name"] for row in parent_catalog["resource_types"])], recorder)
        role_arn = self.seed_reports.get("seed_discovery", {}).get("fixtures", {}).get(
            "assume_role_arn", "arn:aws:iam::000000000000:role/inventory-emulator-read-role")
        self.cli("assumed-role", ["scan", *common, "--service", "iam", "--role-arn",
                                  role_arn], recorder)
        rich_bucket = self.seed_reports.get("seed_global", {}).get("fixtures", {}).get("s3", {}).get("rich_bucket")
        if rich_bucket and self.args.branch_probe.is_file():
            os.environ["AWS_EMULATOR_BUCKET"] = rich_bucket
            os.environ["AWS_EMULATOR_VIEW_ARN"] = self.seed_reports.get("seed_discovery", {}).get("fixtures", {}).get("resource_explorer_view_arn", "")
            branch = self.cli("branch-probes", [], recorder, executable=self.args.branch_probe, phase_prefix="rust-probe:")
            self.check("branch-probes:bounded_execution", branch.get("exit_code") in (0, 2))
        else:
            self.check("branch-probes:available", False, "rich bucket fixture and compiled emulator_branches example required")
        if db.exists():
            self.inspect_database(db)
            with sqlite3.connect(db) as connection:
                scan_id = connection.execute("SELECT scan_id FROM scans ORDER BY started_at,scan_id LIMIT 1").fetchone()[0]
            self.cli("architecture", ["architecture", "--db", str(db), "--scan-id", scan_id, "--limit", "1000"], recorder)
            self.cli("report", ["report", "--db", str(db), "--scan-id", scan_id], recorder)
            exported = self.cli("graph-export", ["graph-export", "--db", str(db), "--scan-id", scan_id,
                                "--graph", "inventory_emulator", "--output", str(self.output / "native-graph.jsonl")], recorder)
            self.check("graph:complete_native_snapshot_exported", exported.get("exit_code") == 0)

    def inspect_database(self, db):
        with sqlite3.connect(db) as connection:
            connection.row_factory = sqlite3.Row
            rows = lambda sql: [dict(row) for row in connection.execute(sql)]
            data = {
                "integrity": connection.execute("PRAGMA integrity_check").fetchone()[0],
                "scans": rows("SELECT * FROM scans ORDER BY started_at,scan_id"),
                "resources": rows("SELECT kind,source,count(*) AS count FROM resource_observations GROUP BY kind,source ORDER BY kind,source"),
                "relationships": rows("SELECT relation,count(*) AS count FROM edge_observations GROUP BY relation ORDER BY relation"),
                "failures": rows("SELECT c.collector,c.parent_id,f.category,f.message FROM collection_failures f JOIN scope_checkpoints c USING(scan_id,scope_key) ORDER BY c.collector,c.parent_id"),
                "scopes": rows("SELECT collector,count(*) AS scopes,sum(complete) AS complete,sum(failed) AS failed,max(last_page) AS max_page FROM scope_checkpoints GROUP BY collector ORDER BY collector"),
                "resource_samples": rows("SELECT kind,id,source,properties FROM latest_resources ORDER BY kind,id LIMIT 500"),
            }
            for row in data["resource_samples"]:
                row["properties"] = json.loads(row["properties"])
            native_scan = connection.execute("SELECT scan_id FROM scans ORDER BY started_at,scan_id LIMIT 1").fetchone()
            if native_scan:
                from assert_global import check as check_global
                global_checks = check_global(connection, native_scan[0], self.seed_reports.get("seed_global", {}).get("fixtures"))
                self.checks.extend(global_checks)
                write_json(self.output / "global-assertions.json", global_checks)
                from assert_regional import check as check_regional
                regional_checks = check_regional(connection, native_scan[0], self.seed_reports.get("seed_regional", {}).get("fixtures", {}))
                self.checks.extend(regional_checks)
                write_json(self.output / "regional-assertions.json", regional_checks)
            cloud_control = next((command for command in self.commands if command.get("name") == "cloud-control"), {})
            cloud_scans = cloud_control.get("result", {}).get("scans", [])
            if len(cloud_scans) == 1 and cloud_control.get("exit_code") in (0, 2):
                from assert_cloud_control import check as check_cloud_control
                cloud_checks = check_cloud_control(connection, cloud_scans[0]["scan_id"],
                                                    self.seed_reports.get("seed_catalog", {}))
                self.checks.extend(cloud_checks)
                write_json(self.output / "cloud-control-assertions.json", cloud_checks)
            else:
                self.check("cloud_control:terminal_scan_for_architecture", False)
            parents = next((command for command in self.commands if command.get("name") == "parents"), {})
            parent_scans = parents.get("result", {}).get("scans", [])
            if len(parent_scans) == 1 and parents.get("exit_code") in (0, 2):
                from assert_parents import check as check_parents
                parent_checks = check_parents(connection, parent_scans[0]["scan_id"], self.seed_reports.get("seed_parents", {}))
                self.checks.extend(parent_checks)
                write_json(self.output / "parent-assertions.json", parent_checks)
            else:
                self.check("parents:terminal_scan_for_architecture", False)
        write_json(self.output / "database-evidence.json", data)
        self.check("sqlite:integrity", data["integrity"] == "ok")
        self.check("sqlite:all_scans_finalized", all(row["status"] != "running" for row in data["scans"]))
        self.check("sqlite:nonempty_resources", bool(data["resources"]))
        self.check("sqlite:nonempty_relationships", bool(data["relationships"]))
        self.check("pagination:noninitial_pages", any(row["max_page"] > 1 for row in data["scopes"]))

    def summarize(self, recorder, version, endpoint):
        if self.execution is not None:
            self.check("provenance:source_unchanged", source_fingerprint(ROOT) == self.execution["source"])
            self.check("provenance:binary_unchanged", file_sha256(self.args.binary.resolve()) == self.execution["binary"]["sha256"])
            self.check("provenance:fixture_unchanged",
                       file_sha256(ROOT / "crates/aws/tests/fixtures/cloudcontrol-architecture.json")
                       == self.execution["architecture_fixture_sha256"])
            self.check("provenance:parent_fixture_unchanged",
                       file_sha256(ROOT / "crates/aws/tests/fixtures/cloudcontrol-parents.json")
                       == self.execution["parent_fixture_sha256"])
            command = next((row for row in self.commands if row.get("name") == "cloud-control"), {})
            scans = command.get("result", {}).get("scans", [])
            if len(scans) == 1 and (self.output / "cloud-control.json").is_file():
                self.execution["cloud_control_command"] = {
                    "file": "cloud-control.json", "sha256": file_sha256(self.output / "cloud-control.json"),
                    "scan_id": scans[0]["scan_id"], "binary_sha256": command.get("binary_sha256"),
                }
            parent_command = next((row for row in self.commands if row.get("name") == "parents"), {})
            parent_scans = parent_command.get("result", {}).get("scans", [])
            if len(parent_scans) == 1 and (self.output / "parents.json").is_file():
                self.execution["parent_command"] = {
                    "file": "parents.json", "sha256": file_sha256(self.output / "parents.json"),
                    "scan_id": parent_scans[0]["scan_id"], "binary_sha256": parent_command.get("binary_sha256"),
                }
            scan_commands = {}
            for name in ("native", "sources", "cloud-control", "parents", "assumed-role"):
                command = next((row for row in self.commands if row.get("name") == name), {})
                scans = command.get("result", {}).get("scans", [])
                path = self.output / (name + ".json")
                if (len(scans) == 1 and command.get("exit_code") in (0, 2) and path.is_file()
                        and command.get("binary_sha256") == self.execution["binary"]["sha256"]):
                    scan_commands[name] = {
                        "file": path.name, "sha256": file_sha256(path),
                        "scan_id": scans[0]["scan_id"], "binary_sha256": command["binary_sha256"],
                    }
            self.execution["scan_commands"] = scan_commands
            self.check("provenance:scan_commands_bound", len(scan_commands) == 5)
        entries = recorder.snapshot()
        with (self.output / "requests.jsonl").open("w") as stream:
            for entry in entries:
                stream.write(json.dumps(entry, sort_keys=True) + "\n")
        grouped = {}
        for row in entries:
            if not row["phase"].startswith(("rust:", "rust-probe:")):
                continue
            key = (row["service"], row["operation"])
            group = grouped.setdefault(key, {"service": key[0], "operation": key[1], "attempts": 0,
                                             "http_statuses": Counter(), "phases": set(), "pagination_requests": 0})
            group["attempts"] += 1
            group["http_statuses"][str(row["http_status"])] += 1
            group["phases"].add(row["phase"])
            group["pagination_requests"] += int(row["pagination_request"])
        coverage = []
        for key in sorted(grouped):
            row = grouped[key]
            row["phases"] = sorted(row["phases"])
            coverage.append(row)
        matrix = json.loads((ROOT / "catalog/aws-operations.json").read_text())
        typed_failures = {}
        verified_probes = set()
        failed_probe_assertions = set()
        for command in self.commands:
            if command.get("name") != "branch-probes" or command.get("exit_code") not in (0, 2):
                continue
            result = command.get("result", {})
            if not isinstance(result, dict) or result.get("invalid_json"):
                continue
            for assertion in result.get("assertions", []):
                verified_probes.add(assertion["operation"])
            for probe in result.get("resource_explorer", []):
                if probe.get("sdk_decoded") is False:
                    typed_failures[probe["operation"]] = probe
                elif probe.get("sdk_decoded") is True and probe.get("assertions_passed") is True:
                    verified_probes.add(probe["operation"])
                elif probe.get("sdk_decoded") is True and probe.get("assertions_passed") is False:
                    failed_probe_assertions.add(probe["operation"])
        expected = []
        for operation in matrix["operations"] + matrix["credential_provider_boundaries"]:
            rows = [row for row in entries if row["service"] == operation["boto3_service"]
                    and row["operation"] == operation["api_operation"]
                    and row["phase"].startswith(("rust:", "rust-probe:"))]
            collector_success = any(row["http_status"] < 300 and row["phase"].startswith("rust:") for row in rows)
            typed_key = operation["boto3_service"] + ":" + operation["api_operation"]
            probe_http_success = any(row["http_status"] < 300 and row["phase"].startswith("rust-probe:") for row in rows)
            probe_success = probe_http_success and typed_key in verified_probes
            status = ("collector_success" if collector_success else "explicit_sdk_probe_success" if probe_success
                      else "emulator_not_implemented" if any(row["http_status"] == 501 for row in rows)
                      else "probe_response_unverified" if probe_http_success
                      else "error_response" if rows else "not_exercised")
            if typed_key in typed_failures:
                status = ("sdk_decode_failure" if typed_failures[typed_key].get("sdk_error_kind") in
                          {"success_response_rejected", "response_decode_failure"} else "explicit_sdk_probe_error")
            if typed_key in failed_probe_assertions:
                status = "explicit_sdk_probe_assertion_failed"
            expected.append({"id": operation["id"], "service": operation["boto3_service"],
                             "operation": operation["api_operation"], "status": status,
                             "attempts": len(rows), "phases": sorted({row["phase"] for row in rows}),
                             "http_statuses": dict(Counter(str(row["http_status"]) for row in rows)),
                             "sdk_decode_error_chain": typed_failures.get(typed_key, {}).get("error_chain"),
                             "credential_provider_boundary": not operation["explicit_sdk_send"]})
        catalog = json.loads((ROOT / "catalog/cloudcontrol-resources.json").read_text())["resource_types"]
        type_coverage = type_operation_coverage(entries, catalog, "rust:cloud-control")
        parent_catalog = json.loads((ROOT / "catalog/cloudcontrol-parent-resources.json").read_text())["resource_types"]
        parent_coverage = type_operation_coverage(entries, parent_catalog, "rust:parents")
        if not self.args.seed_only:
            self.check("coverage:every_operation_attempted", all(row["attempts"] for row in expected),
                       [row["id"] for row in expected if not row["attempts"]])
            self.check("coverage:all_catalog_types_listed", all(row["operations"]["ListResources"] for row in type_coverage))
            gaps = json.loads((Path(__file__).parent / "known-gaps.json").read_text())
            unexpected = [row for row in expected if row["status"] not in {"collector_success", "explicit_sdk_probe_success"}
                          and gaps["operations"].get(row["id"]) != row["status"]]
            self.check("coverage:no_unexplained_operation_failures", not unexpected,
                       [{"id": row["id"], "status": row["status"]} for row in unexpected])
            missing_reads = [row["type_name"] for row in type_coverage if row["type_name"] not in gaps["cloud_control_unregistered_types"]
                             and not row["operations"]["GetResource"].get("200")]
            self.check("coverage:every_registered_type_read", not missing_reads, missing_reads)
            self.check("parents:all_types_listed", all(row["operations"]["ListResources"] for row in parent_coverage))
            self.check("parents:all_types_read", all(row["operations"]["GetResource"].get("200") for row in parent_coverage))
        successful_statuses = {"collector_success", "explicit_sdk_probe_success"}
        successful_operations = sum(row["status"] in successful_statuses for row in expected)
        readable_types = sum(bool(row["operations"]["GetResource"].get("200")) for row in type_coverage)
        readable_parent_types = sum(bool(row["operations"]["GetResource"].get("200")) for row in parent_coverage)
        report = {
            "schema_version": 1, "execution": self.execution,
            "emulator": {"version_output": version.strip(), "source_commit": PINNED_COMMIT,
                "endpoint": endpoint, "signature_authentication": True, "tls_verified_both_hops": True,
                "telemetry": False, "region": "us-east-1", "account": "000000000000",
                "idle_timeout_seconds": getattr(self.args, "emulator_idle_timeout", 600)},
            "duration_seconds": round(time.monotonic() - self.start, 3),
            "requests": len(entries), "rust_operations_observed": len(coverage), "operation_evidence": coverage,
            "expected_operations": expected, "operation_outcomes": dict(Counter(row["status"] for row in expected)),
            "operation_attempt_coverage_complete": all(row["attempts"] for row in expected),
            "implemented_operation_boundaries": len(expected),
            "successful_operation_boundaries": successful_operations,
            "unsuccessful_operation_boundaries": len(expected) - successful_operations,
            "emulator_compatibility": "complete_for_exercised_matrix" if successful_operations == len(expected)
                and readable_types == len(type_coverage) and readable_parent_types == len(parent_coverage) else "partial",
            "cloud_control_resource_types": type_coverage,
            "cloud_control_types_with_successful_reads": readable_types,
            "parent_cloud_control_resource_types": parent_coverage,
            "parent_cloud_control_types_with_successful_reads": readable_parent_types,
            "combined_cloud_control_types_with_successful_reads": readable_types + readable_parent_types,
            "checks": self.checks, "passed_checks": sum(check["passed"] for check in self.checks),
            "failed_checks": sum(not check["passed"] for check in self.checks),
            "limits": ["Emulator behavior does not prove live AWS fidelity, IAM authorization, quota correctness, or regional isolation.",
                       "Fixture mutations and Python probes do not count as Rust collector operation coverage.",
                       "A 200 response proves transport/SDK decoding only where result assertions also pass; error responses are reported separately."],
        }
        write_json(self.output / "summary.json", report)
        print(json.dumps({"evidence": str(self.output), "rust_operations_observed": len(coverage),
                          "checks_passed": report["passed_checks"], "checks_failed": report["failed_checks"]}), flush=True)
        return report


def execute(args, after_verified_run=None, before_collection=None):
    run = Run(args)
    version = subprocess.check_output([str(args.emulator.resolve()), "--version"], text=True, timeout=30)
    if PINNED_VERSION not in version or PINNED_COMMIT not in version:
        raise RuntimeError("emulator binary does not match the reviewed version/commit")
    # Remove inherited AWS credentials, endpoints and credential-provider controls.
    for key in list(os.environ):
        if key.startswith("AWS_"):
            os.environ.pop(key)
    os.environ.update(AWS_EC2_METADATA_DISABLED="true", AWS_REGION="us-east-1", AWS_DEFAULT_REGION="us-east-1",
                      AWS_DISABLE_HOST_PREFIX_INJECTION="true", EMULATOR_TELEMETRY="off", NO_PROXY="127.0.0.1,::1",
                      RUST_LOG="warn,aws_emulator_server=info,aws_emulator_router=info,emulator::audit=off")
    server_log = (run.output / "server.jsonl").open("w")
    process = subprocess.Popen([str(args.emulator.resolve()), "--port", "0", "--bind", "127.0.0.1",
                                "--auth", "on", "--telemetry", "off", "--idle-timeout",
                                str(getattr(args, "emulator_idle_timeout", 600))],
                               stdout=server_log, stderr=server_log)
    try:
        endpoint = None
        for _ in range(150):
            if process.poll() is not None:
                raise RuntimeError("emulator exited before readiness; inspect server.jsonl")
            for line in (run.output / "server.jsonl").read_text().splitlines():
                try:
                    endpoint = json.loads(line).get("fields", {}).get("endpoint_url") or endpoint
                except ValueError:
                    pass
            if endpoint:
                break
            time.sleep(0.2)
        if not endpoint:
            raise RuntimeError("emulator did not become ready within 30 seconds")
        parsed = require_loopback(endpoint)
        directory = Path.home() / ".aws/emulator" / str(parsed.port)
        os.environ.update(AWS_SHARED_CREDENTIALS_FILE=str(directory / "emulator_credentials"),
                          AWS_CONFIG_FILE=str(directory / "config"))
        session = boto3.Session(region_name="us-east-1")
        recorder = Recorder(session._session)
        with tempfile.TemporaryDirectory(prefix="inventory-emulator-tls-") as temp:
            with RecordingProxy(endpoint, directory / "ca.pem", temp, recorder) as proxy:
                # The Rust SDK uses SSL_CERT_FILE; Boto3 uses AWS_CA_BUNDLE.
                os.environ.update(AWS_ENDPOINT_URL=proxy.endpoint, AWS_CA_BUNDLE=str(proxy.cert), SSL_CERT_FILE=str(proxy.cert))
                config_copy = Path(temp) / "config"
                config_copy.write_text("[default]\nregion = us-east-1\nendpoint_url = " + proxy.endpoint + "\nca_bundle = " + str(proxy.cert) + "\n")
                os.environ["AWS_CONFIG_FILE"] = str(config_copy)
                session = boto3.Session(region_name="us-east-1")
                sts = session.client("sts", endpoint_url=proxy.endpoint, verify=str(proxy.cert),
                                     config=Config(retries={"total_max_attempts": 1}, connect_timeout=5, read_timeout=10))
                identity = sts.get_caller_identity()
                if identity.get("Account") != "000000000000":
                    raise RuntimeError("test account identity did not match emulator")
                fixture_credentials = session.get_credentials().get_frozen_credentials()
                if not fixture_credentials.access_key.startswith("EMULATOR"):
                    raise RuntimeError("credentials did not originate from the pinned emulator")
                os.environ["INVENTORY_EMULATOR_ACCESS_KEY_ID"] = fixture_credentials.access_key
                os.environ["INVENTORY_EMULATOR_SECRET_ACCESS_KEY"] = fixture_credentials.secret_key
                run.check("endpoint:emulator_identity", True)
                run.seed(session, recorder)
                if before_collection is not None:
                    before_collection(run, session, recorder)
                if not args.seed_only:
                    run.collect(recorder)
                report = run.summarize(recorder, version, endpoint)
            if after_verified_run is not None:
                if args.seed_only or report["failed_checks"]:
                    raise RuntimeError("additional checks require a complete, verified fixture baseline")
                for name in ("AWS_EMULATOR_BUCKET", "AWS_EMULATOR_VIEW_ARN",
                             "INVENTORY_EMULATOR_ACCESS_KEY_ID", "INVENTORY_EMULATOR_SECRET_ACCESS_KEY"):
                    os.environ.pop(name, None)
                # Long mutation campaigns reuse the seeded, read-only fixture
                # server directly. The bounded baseline recorder is already
                # finalized and never mislabels later mutation traffic as
                # coverage of the ordinary collector binary.
                os.environ.update(AWS_ENDPOINT_URL=endpoint, AWS_CA_BUNDLE=str(directory / "ca.pem"),
                                  SSL_CERT_FILE=str(directory / "ca.pem"))
                config_copy.write_text("[default]\nregion = us-east-1\nendpoint_url = " + endpoint
                                       + "\nca_bundle = " + str(directory / "ca.pem") + "\n")
                result = after_verified_run(run, report, endpoint)
                if result:
                    return result
        return 1 if report["failed_checks"] else 0
    finally:
        os.environ.pop("INVENTORY_EMULATOR_ACCESS_KEY_ID", None)
        os.environ.pop("INVENTORY_EMULATOR_SECRET_ACCESS_KEY", None)
        process.send_signal(signal.SIGTERM) if process.poll() is None else None
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        server_log.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--emulator", type=Path, required=True)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/aws-inventory")
    parser.add_argument("--branch-probe", type=Path, default=ROOT / "target/debug/examples/emulator_branches")
    parser.add_argument("--output", type=Path, required=True, help="new directory for JSON evidence and SQLite data")
    parser.add_argument("--seed-only", action="store_true")
    raise SystemExit(execute(parser.parse_args()))
