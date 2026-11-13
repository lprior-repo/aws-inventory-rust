#!/usr/bin/env python3
"""Freeze independently checked emulator observations for Cargo integration tests.

Only the collector's observation timestamp is removed. Provider timestamps,
resource properties, duplicate observations, edges, page boundaries, and failure
details remain part of the oracle. A newly seeded process needs a new snapshot.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sqlite3

from fixture_provenance import canonical_sha256, expected_runtime_catalog, file_sha256, source_fingerprint

ROOT = Path(__file__).resolve().parents[2]
MAX_DOCUMENT_BYTES = 16 * 1024 * 1024
MAX_ROWS = 100_000
CASES = (("native", "native"), ("sources", "sources"),
         ("cloudcontrol", "cloud-control"), ("assumedrole", "assumed-role"),
         ("parents", "parents"))
REQUIRED_CHECKS = {
    "endpoint:emulator_identity", "native:terminal_scan", "global:all_39_native_kinds",
    "sqlite:integrity", "sqlite:all_scans_finalized", "coverage:every_operation_attempted",
    "coverage:no_unexplained_operation_failures", "coverage:every_registered_type_read",
    "provenance:embedded_catalog_matches", "provenance:source_unchanged",
    "provenance:binary_unchanged", "provenance:fixture_unchanged",
    "provenance:parent_fixture_unchanged", "provenance:scan_commands_bound",
    "parents:fixture_hash", "parents:seed_source_hash", "parents:parent_catalog_hash",
    "parents:seed_complete", "parents:exact_child_count", "parents:exact_parent_edge_count",
    "parents:all_twenty_four_parent_discoveries_complete", "parents:no_collection_failures",
}


def read_json(path: Path):
    with path.open("rb") as stream:
        data = stream.read(MAX_DOCUMENT_BYTES + 1)
    if len(data) > MAX_DOCUMENT_BYTES:
        raise ValueError(f"JSON document exceeds its byte budget: {path.name}")
    return json.loads(data), hashlib.sha256(data).hexdigest()


def canonical(value) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def bound_command(evidence: Path, name: str, binding: dict, binary_sha: str) -> dict:
    command, command_sha = read_json(evidence / (name + ".json"))
    if (not isinstance(binding, dict) or binding.get("file") != name + ".json"
            or binding.get("sha256") != command_sha):
        raise ValueError(f"snapshot command {name} is not bound to the asserted baseline")
    if command.get("name") != name or command.get("binary_sha256") != binary_sha:
        raise ValueError(f"snapshot command {name} belongs to a different command or binary")
    return command


def execution_binding(summary: dict, evidence: Path, binary: Path):
    execution = summary.get("execution")
    if not isinstance(execution, dict):
        raise ValueError("snapshot baseline lacks execution provenance")
    binary_sha = file_sha256(binary)
    if execution.get("binary", {}).get("sha256") != binary_sha:
        raise ValueError("snapshot binary differs from the asserted baseline binary")
    if execution.get("source") != source_fingerprint(ROOT):
        raise ValueError("snapshot baseline belongs to different collector source")
    catalog, catalog_sha = read_json(ROOT / "catalog/cloudcontrol-resources.json")
    parents, parents_sha = read_json(ROOT / "catalog/cloudcontrol-parent-resources.json")
    runtime = expected_runtime_catalog(catalog, parents)
    expected = {
        "catalog_sha256": catalog_sha,
        "catalog_canonical_sha256": canonical_sha256(catalog),
        "parent_catalog_sha256": parents_sha,
        "runtime_catalog_canonical_sha256": canonical_sha256(runtime),
        "architecture_fixture_sha256": file_sha256(ROOT / "crates/aws/tests/fixtures/cloudcontrol-architecture.json"),
        "parent_fixture_sha256": file_sha256(ROOT / "crates/aws/tests/fixtures/cloudcontrol-parents.json"),
    }
    if any(execution.get(key) != value for key, value in expected.items()):
        raise ValueError("snapshot baseline belongs to different catalogs or architecture fixtures")
    command = bound_command(evidence, "catalog", execution.get("catalog_command"), binary_sha)
    if (command.get("arguments") != ["catalog"] or command.get("exit_code") != 0
            or command.get("result", {}).get("cloud_control") != runtime):
        raise ValueError("snapshot baseline binary does not report the exact merged runtime catalog")
    bindings = execution.get("scan_commands")
    if not isinstance(bindings, dict) or set(bindings) != {command for _, command in CASES}:
        raise ValueError("snapshot baseline must bind exactly the five collector commands")
    for field, name in (("cloud_control_command", "cloud-control"), ("parent_command", "parents")):
        if execution.get(field) != bindings[name]:
            raise ValueError("snapshot baseline command bindings disagree")
    return execution, binary_sha, catalog, parents


def case_arguments(evidence: Path, role: str, catalog: dict, parents: dict) -> dict:
    common = ["scan", "--db", str(evidence / "inventory.sqlite"), "--region", "us-east-1",
              "--attempts", "2", "--max-pages", "128", "--page-size", "1", "--concurrency", "32",
              "--per-bucket-concurrency", "4", "--rps", "100"]
    return {
        "native": common + ["--service", "all"],
        "sources": common + ["--service", "config,resource-explorer", "--discover-sources"],
        "cloud-control": common + ["--service", "cloud-control", "--cloud-control", "--resource-type",
                                   ",".join(row["type_name"] for row in catalog["resource_types"])],
        "parents": common + ["--service", "cloud-control", "--cloud-control", "--resource-type",
                             ",".join(row["type_name"] for row in parents["resource_types"])],
        "assumed-role": common + ["--service", "iam", "--role-arn", role],
    }


def observations(connection, table, column, scan_id):
    # Table/column names are private constants, never user-provided SQL.
    records = connection.execute(
        f"SELECT {column} FROM {table} WHERE scan_id=? LIMIT ?", (scan_id, MAX_ROWS + 1)
    ).fetchall()
    if len(records) > MAX_ROWS:
        raise ValueError("snapshot observation row budget exceeded")
    values = []
    for row in records:
        value = json.loads(row[column])
        observed_at = value.pop("observed_at")
        if type(observed_at) is not int or observed_at <= 0:
            raise ValueError("baseline contains an invalid observation timestamp")
        values.append(value)
    return sorted(values, key=canonical)


def freeze(evidence: Path, destination: Path, binary: Path) -> dict:
    evidence = evidence.resolve()
    summary, summary_sha = read_json(evidence / "summary.json")
    if summary.get("failed_checks") != 0 or summary.get("passed_checks", 0) < 1_000:
        raise ValueError("snapshot requires the complete independently asserted fixture baseline")
    checks = summary.get("checks", [])
    if not checks or any(check.get("passed") is not True for check in checks):
        raise ValueError("snapshot baseline contains missing or unsuccessful assertions")
    if len(checks) != summary["passed_checks"]:
        raise ValueError("snapshot baseline assertion accounting is inconsistent")
    if not REQUIRED_CHECKS <= {check.get("check") for check in checks}:
        raise ValueError("snapshot baseline lacks required independent or provenance assertions")
    emulator = summary.get("emulator", {})
    if (emulator.get("account"), emulator.get("region"), emulator.get("source_commit")) != (
        "000000000000", "us-east-1", "f515581569b98a9dcbd2e686c12f912fdce51bba"
    ) or "1.0.336" not in emulator.get("version_output", ""):
        raise ValueError("snapshot baseline emulator identity does not match the pinned fixture")
    execution, binary_sha, catalog, parents = execution_binding(summary, evidence, binary)
    seed, _ = read_json(evidence / "seed-results.json")
    role = seed["modules"]["seed_discovery"]["fixtures"]["assume_role_arn"]
    if not role.startswith("arn:aws:iam::000000000000:role/"):
        raise ValueError("snapshot assume-role fixture belongs to a different account")
    arguments = case_arguments(evidence, role, catalog, parents)
    cases = []
    case_scans = {}
    database = evidence / "inventory.sqlite"
    connection = sqlite3.connect(database.as_uri() + "?mode=ro", uri=True)
    connection.row_factory = sqlite3.Row
    try:
        connection.execute("BEGIN")
        if connection.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
            raise ValueError("snapshot database failed integrity checking")
        for case_name, command_name in CASES:
            binding = execution["scan_commands"][command_name]
            command = bound_command(evidence, command_name, binding, binary_sha)
            if command.get("arguments") != arguments[command_name] or binding.get("binary_sha256") != binary_sha:
                raise ValueError(f"snapshot case {case_name} has different collection arguments or binary")
            result = command.get("result", {})
            scans = result.get("scans", [])
            if len(scans) != 1 or type(command.get("exit_code")) is not int or command["exit_code"] not in (0, 2):
                raise ValueError(f"snapshot case {case_name} has no single terminal scan")
            scan = scans[0]
            status = scan["status"]
            if status not in ("complete", "partial"):
                raise ValueError(f"snapshot case {case_name} is not terminal")
            if (result.get("status") != status or result.get("write_error") is not None
                    or scan.get("error") is not None or command["exit_code"] != (0 if status == "complete" else 2)):
                raise ValueError(f"snapshot case {case_name} has inconsistent terminal command evidence")
            scan_id = scan["scan_id"]
            if (not isinstance(scan_id, str) or not scan_id or binding.get("scan_id") != scan_id
                    or scan_id in case_scans):
                raise ValueError("snapshot cases require five distinct baseline-bound scan identities")
            case_scans[scan_id] = status
            actual = connection.execute("SELECT status FROM scans WHERE scan_id=?", (scan_id,)).fetchone()
            if actual is None or actual[0] != status:
                raise ValueError(f"snapshot case {case_name} disagrees with persisted scan state")
            scopes = [dict(row) for row in connection.execute(
                "SELECT scope_key,account_id,region,collector,parent_id,source,last_page,"
                "next_token IS NOT NULL AS has_next_token,complete,failed "
                "FROM scope_checkpoints WHERE scan_id=? LIMIT ?", (scan_id, MAX_ROWS + 1))]
            failures = [dict(row) for row in connection.execute(
                "SELECT scope_key,category,message FROM collection_failures "
                "WHERE scan_id=? LIMIT ?", (scan_id, MAX_ROWS + 1))]
            if len(scopes) > MAX_ROWS or len(failures) > MAX_ROWS:
                raise ValueError("snapshot scope/failure row budget exceeded")
            for scope in scopes:
                for key in ("has_next_token", "complete", "failed"):
                    if scope[key] not in (0, 1):
                        raise ValueError("snapshot contains a non-boolean scope flag")
                    scope[key] = bool(scope[key])
            cases.append({
                "name": case_name, "status": status,
                "resources": observations(connection, "resource_observations", "resource_json", scan_id),
                "edges": observations(connection, "edge_observations", "edge_json", scan_id),
                "scopes": sorted(scopes, key=canonical),
                "failures": sorted(failures, key=canonical),
            })
            if not cases[-1]["resources"] or not cases[-1]["scopes"]:
                raise ValueError(f"snapshot case {case_name} lacks populated finalized evidence")
        persisted = connection.execute("SELECT scan_id,status FROM scans LIMIT ?", (len(CASES) + 1,)).fetchall()
        if len(persisted) != len(CASES) or {row[0]: row[1] for row in persisted} != case_scans:
            raise ValueError("snapshot database must contain exactly the five asserted scan identities and states")
    finally:
        connection.close()
    # Recheck every source/binary/catalog/fixture input after copying the stable
    # SQLite read transaction; do not publish an oracle across concurrent edits.
    execution_binding(summary, evidence, binary)
    snapshot = {
        "schema_version": 1, "account_id": "000000000000", "region": "us-east-1",
        "assume_role_arn": role,
        "provenance": {
            "emulator_version": "1.0.336",
            "emulator_commit": emulator["source_commit"], "summary_sha256": summary_sha,
            "baseline_binary_sha256": binary_sha,
            "normalization": "remove only the top-level observed_at field from resource and edge observations",
        },
        "cases": cases,
    }
    encoded = (json.dumps(snapshot, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode()
    if len(encoded) > MAX_DOCUMENT_BYTES:
        raise ValueError("snapshot output byte budget exceeded")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("xb") as stream:
        stream.write(encoded)
    return {
        "snapshot": str(destination.resolve()), "snapshot_sha256": hashlib.sha256(encoded).hexdigest(),
        "summary": str((evidence / "summary.json").resolve()), "summary_sha256": summary_sha,
        "cases": [{"name": case["name"], "status": case["status"],
                   **{key: len(case[key]) for key in ("resources", "edges", "scopes", "failures")}}
                  for case in cases],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(freeze(args.evidence, args.output, args.binary), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
