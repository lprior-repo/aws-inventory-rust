#!/usr/bin/env python3
"""Bind the capability matrix to observed Rust traffic and persisted SDK reads.

No AWS calls are made. Python seeding and an HTTP 200 without a matching typed
Rust resource observation cannot establish a successful Cloud Control read.
"""

from __future__ import annotations

import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import sqlite3
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/emulator"))
from fixture_provenance import canonical_sha256, expected_runtime_catalog, file_sha256, source_fingerprint

CATALOG = ROOT / "catalog/cloudcontrol-resources.json"
PARENT_CATALOG = ROOT / "catalog/cloudcontrol-parent-resources.json"
MATRIX = ROOT / "catalog/cloudcontrol-capabilities.json"
MAX_DOCUMENT_BYTES = 32 * 1024 * 1024
MAX_OBSERVATIONS = 100_000


def read_json(path):
    with path.open("rb") as stream:
        raw = stream.read(MAX_DOCUMENT_BYTES + 1)
    if len(raw) > MAX_DOCUMENT_BYTES:
        raise ValueError("coverage evidence document exceeds byte budget")
    return json.loads(raw), hashlib.sha256(raw).hexdigest()


def relative(path):
    return str(path.resolve().relative_to(ROOT))


def checked_histogram(value):
    if not isinstance(value, dict) or len(value) > 600:
        raise ValueError("invalid operation status histogram")
    for code, count in value.items():
        valid_code = code == "None" or code.isascii() and code.isdigit() and 100 <= int(code) <= 599
        if not valid_code or type(count) is not int or count <= 0:
            raise ValueError("invalid operation status histogram entry")
    return dict(value)


def apply_verification(matrix, catalog, summary, read_counts, binding):
    """Pure evidence merge, deliberately separate from artifact/database I/O."""
    types = {row["type_name"]: row for row in catalog["resource_types"]}
    if not types or len(types) != len(catalog["resource_types"]) or len(types) > 4_096:
        raise ValueError("invalid executable catalog type set")
    checks = summary.get("checks", [])
    required = {"coverage:all_catalog_types_listed", "coverage:every_registered_type_read",
                "sqlite:integrity", "cloud_control:architecture_fixture_metadata",
                "provenance:embedded_catalog_matches", "provenance:source_unchanged",
                "provenance:binary_unchanged", "provenance:fixture_unchanged"}
    if (summary.get("schema_version") != 1 or summary.get("failed_checks") != 0
            or not checks or any(row.get("passed") is not True for row in checks)
            or summary.get("passed_checks") != len(checks)
            or not required <= {row.get("check") for row in checks}):
        raise ValueError("coverage refresh requires a complete asserted emulator run")
    executed = summary.get("execution") or {}
    digests = [binding.get(key) for key in (
        "catalog_sha256", "source_sha256", "binary_sha256", "architecture_fixture_sha256"
    )]
    if (any(not isinstance(value, str) or len(value) != 64 or any(char not in "0123456789abcdef" for char in value)
            for value in digests)
            or executed.get("catalog_canonical_sha256") != canonical_sha256(catalog)
            or executed.get("catalog_sha256") != binding.get("catalog_sha256")
            or executed.get("source", {}).get("sha256") != binding.get("source_sha256")
            or executed.get("binary", {}).get("sha256") != binding.get("binary_sha256")
            or executed.get("architecture_fixture_sha256") != binding.get("architecture_fixture_sha256")):
        raise ValueError("current catalog/source/binary/fixture differs from executed evidence")
    observed = summary.get("cloud_control_resource_types", [])
    if len(observed) != len(types) or {row["type_name"] for row in observed} != set(types):
        raise ValueError("emulator type set differs from the executable catalog")
    if set(read_counts) - set(types):
        raise ValueError("persisted reads include an unexpected provider type")
    result = copy.deepcopy(matrix)
    entries = result["resource_types"]
    if (not entries or len(entries) > 4_096
            or len({row["type_name"] for row in entries}) != len(entries)
            or any(type(row["selected"]) is not bool for row in entries)):
        raise ValueError("invalid or duplicate capability matrix type names")
    if {row["type_name"] for row in result["resource_types"] if row["selected"]} != set(types):
        raise ValueError("capability selection differs from the executable catalog")
    by_name = {row["type_name"]: row for row in result["resource_types"]}
    for row in observed:
        name = row["type_name"]
        listing = checked_histogram(row["operations"]["ListResources"])
        getting = checked_histogram(row["operations"]["GetResource"])
        count = read_counts.get(name, 0)
        if type(count) is not int or count < 0:
            raise ValueError("invalid persisted read count")
        if count > getting.get("200", 0):
            raise ValueError("persisted provider read lacks matching Rust GetResource success")
        entry = by_name[name]
        if entry["schema_sha256"] != types[name]["schema_sha256"]:
            raise ValueError("capability schema binding differs from the executable catalog")
        status = ("read_observed" if count else
                  "response_without_persisted_read" if getting.get("200") else
                  "get_error" if getting else
                  "listed_without_read" if listing.get("200") else
                  "list_error" if listing else "not_run")
        entry["emulator_verification"] = {
            "status": status, "summary_file": binding["summary_file"],
            "list_attempted": bool(listing), "get_attempted": bool(getting),
            "read_observed": count > 0, "persisted_read_count": count,
            "list_http_statuses": listing, "get_http_statuses": getting,
        }
    selected = [row for row in result["resource_types"] if row["selected"]]
    result["summary"].update(
        emulator_list_attempted=sum(row["emulator_verification"]["list_attempted"] for row in selected),
        emulator_read_observed=sum(row["emulator_verification"]["read_observed"] for row in selected),
        emulator_outcomes=dict(sorted(Counter(row["emulator_verification"]["status"] for row in selected).items())),
    )
    result["evidence"]["emulator"] = binding
    return result


def persisted_reads(database, scan_id, catalog, expected_status=None):
    types = {row["type_name"]: row for row in catalog["resource_types"]}
    connection = sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True)
    counts, byte_count = Counter(), 0
    try:
        if connection.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
            raise ValueError("coverage database integrity check failed")
        status = connection.execute("SELECT status FROM scans WHERE scan_id=?", (scan_id,)).fetchone()
        if status is None or status[0] not in ("complete", "partial"):
            raise ValueError("coverage scan is missing or not terminal")
        if expected_status is not None and status[0] != expected_status:
            raise ValueError("persisted coverage scan status differs from the command receipt")
        rows = connection.execute(
            "SELECT resource_json FROM resource_observations WHERE scan_id=? AND source='cloud_control' LIMIT ?",
            (scan_id, MAX_OBSERVATIONS + 1),
        )
        for index, (raw,) in enumerate(rows):
            byte_count += len(raw.encode())
            if index >= MAX_OBSERVATIONS or byte_count > MAX_DOCUMENT_BYTES:
                raise ValueError("persisted coverage observation budget exceeded")
            value = json.loads(raw)
            properties = value.get("properties", {})
            name = properties.get("cloudformation_type")
            if value.get("source") != "cloud_control" or name not in types:
                raise ValueError("unexpected persisted Cloud Control source or type")
            if properties.get("schema_sha256") != types[name]["schema_sha256"]:
                raise ValueError("persisted observation was built from a different provider schema")
            if properties.get("details_available") is True and properties.get("observation_detail") == "get_resource":
                counts[name] += 1
    finally:
        connection.close()
    return dict(counts)


def write_rollup(matrix, path):
    summary = matrix["summary"]
    lines = [
        "# Cloud Control coverage evidence",
        "",
        "Generated from the pinned capability matrix. Schema support, reviewed collector selection,",
        "relationship rules, and observed emulator reads are separate evidence dimensions.",
        "",
        "| Measure | Count |",
        "| --- | ---: |",
        f"| Retained provider schemas | {summary['schema_count']:,} |",
        f"| Schemas declaring read and list handlers | {summary['read_and_list_count']:,} |",
        f"| Selected executable regional descriptors | {summary['selected_count']:,} |",
        f"| Declared relationship rules | {summary['relationship_rule_count']:,} |",
        f"| Selected types with relationship rules | {summary['types_with_relationships']:,} |",
        f"| Types with observed Rust ListResources requests | {summary['emulator_list_attempted']:,} |",
        f"| Types with matching Rust GetResource and persisted detailed observations | {summary['emulator_read_observed']:,} |",
        "",
        "## Exclusions",
        "",
        "| Reason | Count |",
        "| --- | ---: |",
    ]
    lines.extend(f"| {reason} | {count:,} |" for reason, count in sorted(summary["exclusions"].items()))
    emulator = matrix["evidence"].get("emulator")
    lines.extend(["", "## Executed evidence", ""])
    if emulator:
        lines.extend([
            f"- Summary: [{emulator['summary_file']}](../{emulator['summary_file']}).",
            f"- Summary SHA-256: `{emulator['summary_sha256']}`.",
            f"- Cloud Control scan receipt: [{emulator['command_file']}](../{emulator['command_file']}).",
            f"- Database: [{emulator['database_file']}](../{emulator['database_file']}).",
            "- Python fixture creation is excluded from operation coverage. A successful HTTP response",
            "  alone does not establish a successful typed, stored read.",
            "",
            "| Observed per-type outcome | Count |",
            "| --- | ---: |",
        ])
        lines.extend(f"| {name} | {count:,} |" for name, count in sorted(summary["emulator_outcomes"].items()))
        failed = [row["type_name"] for row in matrix["resource_types"]
                  if row["selected"] and not row["emulator_verification"]["read_observed"]]
        if failed:
            lines.extend(["", "Selected types without an observed detailed read:", ""])
            lines.extend(f"- `{name}`." for name in failed)
    else:
        lines.append("The expanded catalog has not yet been bound to a completed emulator run.")
    lines.extend([
        "",
        "## Full matrix and interpretation",
        "",
        "- [All provider capability entries](../catalog/cloudcontrol-capabilities.json).",
        "- [Executable descriptors and property mappings](../catalog/cloudcontrol-resources.json).",
        "- [Collection and relationship design](CLOUDCONTROL_COVERAGE.md).",
        "",
        "The pinned emulator uses a generic property store for most types. These observations",
        "exercise SDK decoding, collection, normalization and storage; they do not certify the",
        "production AWS provider's behavior, IAM permissions, regional availability or quotas.",
        "",
    ])
    path.write_text("\n".join(lines))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--emulator-summary", type=Path)
    args = parser.parse_args()
    matrix, _ = read_json(MATRIX)
    catalog, catalog_sha = read_json(CATALOG)
    if matrix["evidence"]["catalog_sha256"] != catalog_sha:
        raise ValueError("capability matrix does not match the current executable catalog")
    if args.emulator_summary:
        parent_catalog, parent_catalog_sha = read_json(PARENT_CATALOG)
        runtime_catalog = expected_runtime_catalog(catalog, parent_catalog)
        runtime_catalog_sha = canonical_sha256(runtime_catalog)
        summary_path = args.emulator_summary.resolve()
        summary, summary_sha = read_json(summary_path)
        command_path = summary_path.parent / "cloud-control.json"
        command, command_sha = read_json(command_path)
        scans = command.get("result", {}).get("scans", [])
        if command.get("exit_code") not in (0, 2) or len(scans) != 1:
            raise ValueError("coverage requires one completed Rust Cloud Control command")
        executed = summary.get("execution") or {}
        if (executed.get("parent_catalog_sha256") != parent_catalog_sha
                or executed.get("runtime_catalog_canonical_sha256") != runtime_catalog_sha):
            raise ValueError("executed parent/runtime catalog differs from current descriptors")
        source = source_fingerprint(ROOT)
        if executed.get("source") != source:
            raise ValueError("collector source changed since the asserted emulator run")
        binary = executed.get("binary", {})
        binary_path = Path(binary.get("path", ""))
        if not binary_path.is_file() or file_sha256(binary_path) != binary.get("sha256"):
            raise ValueError("executed collector binary is unavailable or changed")
        expected_command = executed.get("cloud_control_command", {})
        if (expected_command.get("file") != "cloud-control.json" or expected_command.get("sha256") != command_sha
                or expected_command.get("scan_id") != scans[0]["scan_id"]
                or expected_command.get("binary_sha256") != binary["sha256"]
                or command.get("binary_sha256") != binary["sha256"]):
            raise ValueError("summary is not bound to this Cloud Control command and scan")
        catalog_command_path = summary_path.parent / "catalog.json"
        catalog_command, catalog_command_sha = read_json(catalog_command_path)
        expected_catalog = executed.get("catalog_command", {})
        if (expected_catalog.get("file") != "catalog.json" or expected_catalog.get("sha256") != catalog_command_sha
                or catalog_command.get("exit_code") != 0 or catalog_command.get("binary_sha256") != binary["sha256"]
                or catalog_command.get("result", {}).get("cloud_control") != runtime_catalog):
            raise ValueError("executed binary catalog differs from the current descriptor catalog")
        fixture_sha = file_sha256(ROOT / "crates/aws/tests/fixtures/cloudcontrol-architecture.json")
        database = summary_path.parent / "inventory.sqlite"
        counts = persisted_reads(database, scans[0]["scan_id"], catalog, scans[0]["status"])
        matrix = apply_verification(matrix, catalog, summary, counts, {
            "summary_file": relative(summary_path), "summary_sha256": summary_sha,
            "command_file": relative(command_path), "command_sha256": command_sha,
            "database_file": relative(database), "scan_id": scans[0]["scan_id"],
            "catalog_sha256": catalog_sha, "emulator_source_commit": summary["emulator"]["source_commit"],
            "source_sha256": source["sha256"], "binary_sha256": binary["sha256"],
            "architecture_fixture_sha256": fixture_sha,
            "parent_catalog_sha256": parent_catalog_sha,
            "runtime_catalog_canonical_sha256": runtime_catalog_sha,
            "catalog_command_file": relative(catalog_command_path), "catalog_command_sha256": catalog_command_sha,
            "semantics": "Rust request ledger plus typed Cloud Control GetResource observations in the asserted scan",
        })
        temporary = MATRIX.with_suffix(".json.tmp")
        temporary.write_text(json.dumps(matrix, indent=2) + "\n")
        temporary.replace(MATRIX)
    write_rollup(matrix, ROOT / "docs/CLOUDCONTROL_EVIDENCE.md")
    print(json.dumps(matrix["summary"], sort_keys=True))


if __name__ == "__main__":
    main()
