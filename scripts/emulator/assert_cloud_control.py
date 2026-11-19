"""Independent Cloud Control architecture assertions for the local fixture run.

Expected relationships come from authored semantic fixtures, not the generated
descriptor rules or the output under test. The emulator's generic provider
store verifies the Rust SDK/list/get/sanitize/map/SQLite path only; it does not
implement the behavior of these production AWS resource providers.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sqlite3

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "crates/aws/tests/fixtures/cloudcontrol-architecture.json"
MAX_ROWS = 20_000
MAX_JSON_BYTES = 16 * 1024 * 1024
GOLDEN_NAMES = frozenset({
    "sagemaker_model_role_and_vpc",
    "rds_cluster_multiple_property_evidence",
    "firehose_schema_refs_preserve_bucket_arn",
    "volume_snapshot_uses_accountless_arn",
    "snapshot_self_identity_uses_accountless_arn",
    "compound_attachment_is_not_a_fabricated_arn",
    "dax_role_topic_and_security_group",
    "apprunner_ingress_links_network_and_service",
    "security_group_names_are_not_ids_even_when_they_resemble_ids",
    "ecs_lattice_and_elb_target_groups_have_distinct_services",
    "dms_endpoint_certificates_are_exact_dms_references",
    "dms_data_provider_certificate_is_not_acm",
    "msk_oauth_ca_certificate_is_a_secrets_manager_reference",
})


def golden_fixtures():
    raw = FIXTURES.read_bytes()
    if len(raw) > 2 * 1024 * 1024:
        raise ValueError("architecture fixture byte budget exceeded")
    document = json.loads(raw)
    if document.get("schema_version") != 1:
        raise ValueError("unknown architecture fixture version")
    # Only the local scanning account changes. Cross-account and cross-region
    # reference ARNs remain literal independently authored expectations.
    fixtures = json.loads(json.dumps([row for row in document["fixtures"]
                                     if row["name"] in GOLDEN_NAMES]).replace("111111111111", "000000000000"))
    if {row["name"] for row in fixtures} != GOLDEN_NAMES or len(fixtures) != 13:
        raise ValueError("architecture fixture selection is incomplete or duplicated")
    if len({row["type_name"] for row in fixtures}) != len(fixtures):
        raise ValueError("architecture fixtures must seed one resource per selected type")
    return fixtures, hashlib.sha256(raw).hexdigest()


def expected_id(fixture):
    if fixture["expected_resource_id"]:
        return fixture["expected_resource_id"]
    if not fixture["opaque_identity"]:
        raise ValueError("fixture lacks an independently defined identity")
    digest = hashlib.sha256(fixture["identifier"].encode()).hexdigest()
    return f"awscc:aws:us-east-1:000000000000:{fixture['type_name']}:{digest}"


def _observations(connection, table, column, scan_id):
    if (table, column) not in {
        ("resource_observations", "resource_json"), ("edge_observations", "edge_json")
    }:
        raise ValueError("unexpected observation table")
    values, byte_count = [], 0
    for row in connection.execute(
        f"SELECT {column} FROM {table} WHERE scan_id=? AND source='cloud_control' LIMIT ?",
        (scan_id, MAX_ROWS + 1),
    ):
        byte_count += len(row[0].encode())
        if len(values) >= MAX_ROWS or byte_count > MAX_JSON_BYTES:
            raise ValueError("Cloud Control assertion evidence exceeds its bound")
        values.append(json.loads(row[0]))
    return values


def check(connection: sqlite3.Connection, scan_id: str, seed_report: dict | None = None):
    checks = []

    def add(name, passed, detail=None):
        check = {"check": "cloud_control:" + name, "passed": bool(passed)}
        if detail is not None:
            check["detail"] = detail
        checks.append(check)

    try:
        fixtures, digest = golden_fixtures()
        rows = _observations(connection, "resource_observations", "resource_json", scan_id)
        edges = _observations(connection, "edge_observations", "edge_json", scan_id)
    except (ValueError, OSError, sqlite3.Error) as error:
        add("bounded_architecture_evidence", False, str(error)[:300])
        return checks
    metadata = (seed_report or {}).get("architecture_fixtures", {})
    add("architecture_fixture_metadata", metadata.get("fixture_sha256") == digest
        and set(metadata.get("names", [])) == GOLDEN_NAMES
        and len(metadata.get("names", [])) == len(GOLDEN_NAMES))
    for fixture in fixtures:
        name, type_name = fixture["name"], fixture["type_name"]
        identifier = expected_id(fixture)
        observed = [row for row in rows if row.get("properties", {}).get("cloudformation_type") == type_name]
        add(name + ":one_typed_read", len(observed) == 1)
        if len(observed) != 1:
            continue
        resource = observed[0]
        properties = resource.get("properties", {})
        add(name + ":exact_identity", resource.get("id") == identifier)
        add(name + ":completed_detail", resource.get("source") == "cloud_control"
            and properties.get("details_available") is True
            and properties.get("observation_detail") == "get_resource")
        add(name + ":redaction", "HIDDEN" not in json.dumps(resource))
        add(name + ":valid_observation_time", type(resource.get("observed_at")) is int
            and resource["observed_at"] > 0)
        actual_edges = [edge for edge in edges if edge.get("from_id") == identifier]
        actual_keys = {(edge.get("relation"), edge.get("to_id")) for edge in actual_edges}
        expected_keys = {(edge["relation"], edge["target"]) for edge in fixture["edges"]}
        add(name + ":exact_edge_set", actual_keys == expected_keys and len(actual_edges) == len(expected_keys),
            {"expected": len(expected_keys), "observed": len(actual_edges)})
        for expected in fixture["edges"]:
            matching = [edge for edge in actual_edges
                        if (edge.get("relation"), edge.get("to_id")) == (expected["relation"], expected["target"])]
            label = name + ":edge:" + expected["relation"] + ":" + expected["target"]
            if len(matching) != 1:
                add(label, False)
                continue
            edge = matching[0]
            evidence = edge.get("properties", {})
            paths = evidence.get("property_paths", [])
            add(label, evidence.get("target_kind") == expected["kind"]
                and evidence.get("owner_resolution") == expected["owner"]
                and evidence.get("cloudformation_type") == type_name
                and set(paths) == set(expected["paths"]) and len(paths) == len(expected["paths"])
                and edge.get("source") == "cloud_control"
                and edge.get("observed_at") == resource.get("observed_at"))
        if fixture["opaque_identity"]:
            add(name + ":compound_identity_opaque", properties.get("identity_resolution") == "namespaced_provider_identifier")
        if name == "snapshot_self_identity_uses_accountless_arn":
            add(name + ":accountless_identity", properties.get("identity_resolution") == "canonical_template_without_account_component")
    return checks
