"""Independent exact parent ownership, properties and pagination assertions."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sqlite3

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "crates/aws/tests/fixtures/cloudcontrol-parents.json"
MAX_ROWS = 20_000
MAX_BYTES = 16 * 1024 * 1024
NEW_PARENT_TYPES = {
    "AWS::AppConfig::ExperimentDefinition",
    "AWS::EKS::AccessEntry",
    "AWS::EKS::PodIdentityAssociation",
}


def load_fixtures():
    raw = FIXTURES.read_bytes()
    if len(raw) > 1_048_576:
        raise ValueError("parent fixture byte bound exceeded")
    document = json.loads(raw)
    if document.get("schema_version") != 1 or (document.get("account_id"), document.get("region")) != ("000000000000", "us-east-1"):
        raise ValueError("unrecognized parent fixture version or account/region")
    parents, children, pages = (document.get(key, []) for key in ("parents", "children", "pagination_children"))
    if (len(parents), len(children), len(pages)) != (12, 72, 98):
        raise ValueError("parent fixture families or pagination records are incomplete")
    if len({row["key"] for row in parents}) != 12 or len({row["name"] for row in children + pages}) != 170:
        raise ValueError("parent fixture names are duplicated")
    if len({row["type_name"] for row in children}) != 24:
        raise ValueError("parent fixture child-type coverage is incomplete")
    parent_keys = {row["key"] for row in parents}
    if any(row["parent_key"] not in parent_keys for row in children + pages):
        raise ValueError("parent fixture child has no literal parent")
    if any(row["type_name"] != "AWS::EKS::Addon" or row["parent_key"] != "eks-a" for row in pages):
        raise ValueError("pagination fixtures no longer exercise the reviewed EKS parent")
    return document, hashlib.sha256(raw).hexdigest()


def normalized_id(row, seed):
    if row.get("expected_resource_id"):
        return row["expected_resource_id"]
    identifier = seed.get("identifier")
    if not isinstance(identifier, str) or not identifier or len(identifier.encode()) > 1_024:
        raise ValueError("fixture receipt lacks a bounded returned identifier")
    return "awscc:aws:us-east-1:000000000000:" + row["type_name"] + ":" + hashlib.sha256(identifier.encode()).hexdigest()


def read_rows(connection, table, column, scan_id):
    if (table, column) not in {("resource_observations", "resource_json"), ("edge_observations", "edge_json")}:
        raise ValueError("unexpected parent assertion table")
    rows, size = [], 0
    for (raw,) in connection.execute(f"SELECT {column} FROM {table} WHERE scan_id=? AND source='cloud_control' LIMIT ?", (scan_id, MAX_ROWS + 1)):
        size += len(raw.encode())
        if len(rows) >= MAX_ROWS or size > MAX_BYTES:
            raise ValueError("parent assertion evidence bound exceeded")
        rows.append(json.loads(raw))
    return rows


def check(connection: sqlite3.Connection, scan_id: str, seed_report: dict | None = None):
    checks = []

    def add(name, passed, detail=None):
        row = {"check": "parents:" + name, "passed": bool(passed)}
        if detail is not None:
            row["detail"] = detail
        checks.append(row)

    try:
        fixture, fixture_sha = load_fixtures()
        resources = read_rows(connection, "resource_observations", "resource_json", scan_id)
        edges = read_rows(connection, "edge_observations", "edge_json", scan_id)
    except (ValueError, OSError, sqlite3.Error) as error:
        add("bounded_evidence", False, str(error)[:300])
        return checks
    seed_report = seed_report or {}
    add("fixture_hash", seed_report.get("fixture_sha256") == fixture_sha)
    add("seed_source_hash", seed_report.get("seed_source_sha256") == hashlib.sha256((ROOT / "scripts/emulator/seed_parents.py").read_bytes()).hexdigest())
    add("parent_catalog_hash", seed_report.get("parent_catalog_sha256") == hashlib.sha256((ROOT / "catalog/cloudcontrol-parent-resources.json").read_bytes()).hexdigest())
    add("seed_complete", seed_report.get("status") == "complete")
    seed_rows = seed_report.get("resources", [])
    add("seed_receipt_count", len(seed_rows) == 182)
    seeds = {(row.get("role"), row.get("name")): row for row in seed_rows}
    add("seed_receipts_unique", len(seeds) == len(seed_rows))
    parents = {row["key"]: row for row in fixture["parents"]}
    children = fixture["children"] + fixture["pagination_children"]
    child_types = {row["type_name"] for row in children}
    discovery_types = child_types | NEW_PARENT_TYPES
    allowed_types = discovery_types | {row["type_name"] for row in fixture["parents"]}
    add("no_unselected_resource_types", all(row.get("properties", {}).get("cloudformation_type") in allowed_types for row in resources))
    observed_children = [row for row in resources if row.get("properties", {}).get("cloudformation_type") in child_types]
    parent_edges = [row for row in edges if row.get("relation") == "CHILD_OF"]
    add("exact_child_count", len(observed_children) == 170, {"expected": 170, "observed": len(observed_children)})
    add("exact_parent_edge_count", len(parent_edges) == 170, {"expected": 170, "observed": len(parent_edges)})
    add("no_sensitive_fixture_values", "HIDDEN" not in json.dumps(resources + edges))
    expected_pairs = set()
    expected_child_ids = set()
    for child in children:
        name = child["name"]
        seed = seeds.get(("child", name), {})
        parent = parents[child["parent_key"]]
        parent_seed = seeds.get(("parent", child["parent_key"]), {})
        add(name + ":seed_success", seed.get("status") == "SUCCESS" and parent_seed.get("status") == "SUCCESS"
            and seed.get("type_name") == child["type_name"] and parent_seed.get("type_name") == parent["type_name"])
        try:
            identity, parent_id = normalized_id(child, seed), normalized_id(parent, parent_seed)
        except ValueError as error:
            add(name + ":identity_receipt", False, str(error))
            continue
        expected_pairs.add((identity, parent_id))
        expected_child_ids.add(identity)
        matches = [row for row in observed_children if row.get("id") == identity]
        add(name + ":one_exact_child", len(matches) == 1)
        if len(matches) != 1:
            continue
        resource = matches[0]
        properties = resource.get("properties", {})
        add(name + ":exact_read", resource.get("kind") == child["expected_kind"]
            and resource.get("account_id") == fixture["account_id"] and resource.get("region") == fixture["region"]
            and resource.get("source") == "cloud_control" and properties.get("details_available") is True
            and properties.get("observation_detail") == "get_resource"
            and properties.get("resource_identifier") == seed.get("identifier")
            and properties.get("provider_properties") == child["expected_provider"])
        add(name + ":exact_parent_provenance", properties.get("parent_resource_id") == parent_id
            and properties.get("parent_type") == parent["type_name"]
            and properties.get("list_resource_model") == child["expected_model"])
        actual_edges = [edge for edge in parent_edges if edge.get("from_id") == identity]
        add(name + ":one_parent_edge", len(actual_edges) == 1)
        if len(actual_edges) == 1:
            edge, evidence = actual_edges[0], actual_edges[0].get("properties", {})
            add(name + ":parent_edge_semantics", edge.get("to_id") == parent_id
                and edge.get("source") == "cloud_control" and edge.get("observed_at") == resource.get("observed_at")
                and evidence.get("target_kind") == parent["expected_kind"]
                and evidence.get("cloudformation_type") == child["type_name"]
                and evidence.get("parent_type") == parent["type_name"]
                and evidence.get("resource_model") == child["expected_model"]
                and evidence.get("evidence") == "reviewed_parent_resource_model_and_matching_read_properties"
                and evidence.get("owner_resolution") == "observed_parent_resource_identity")
        add(name + ":parent_observed", any(row.get("id") == parent_id and row.get("kind") == parent["expected_kind"] for row in resources))
        architecture = [edge for edge in edges if edge.get("from_id") == identity and edge.get("relation") != "CHILD_OF"]
        expected_architecture = child["expected_relationships"]
        add(name + ":exact_architecture_edge_set", len(architecture) == len(expected_architecture)
            and {(edge.get("relation"), edge.get("to_id")) for edge in architecture}
            == {(edge["relation"], edge["target"]) for edge in expected_architecture})
        for expected in expected_architecture:
            matches = [edge for edge in architecture if (edge.get("relation"), edge.get("to_id")) == (expected["relation"], expected["target"])]
            add(name + ":architecture_evidence:" + expected["relation"], len(matches) == 1
                and matches[0].get("source") == "cloud_control" and matches[0].get("observed_at") == resource.get("observed_at")
                and matches[0].get("properties", {}).get("target_kind") == expected["kind"]
                and matches[0].get("properties", {}).get("owner_resolution") == expected["owner_resolution"]
                and matches[0].get("properties", {}).get("cloudformation_type") == child["type_name"]
                and matches[0].get("properties", {}).get("property_paths") == expected["paths"])
    add("no_missing_or_extra_child_identities", {row.get("id") for row in observed_children} == expected_child_ids and len(expected_child_ids) == 170)
    add("no_cross_parent_edges", {(row.get("from_id"), row.get("to_id")) for row in parent_edges} == expected_pairs and len(expected_pairs) == 170)
    try:
        discovery = connection.execute("SELECT parent_id,complete,failed FROM scope_checkpoints WHERE scan_id=? AND collector='cloud_control_parent_discovery'", (scan_id,)).fetchall()
        add("all_twenty_four_parent_discoveries_complete", len(discovery) == 24
            and {row[0] for row in discovery} == discovery_types and all((row[1], row[2]) == (1, 0) for row in discovery))
        failures = connection.execute("SELECT count(*) FROM collection_failures WHERE scan_id=?", (scan_id,)).fetchone()[0]
        add("no_collection_failures", failures == 0, {"observed": failures})
        for side in ("a", "b"):
            model = {"ClusterName": "inventory-parent-eks-" + side}
            encoded = json.dumps(model, sort_keys=True, separators=(",", ":"))
            parent_scope = "AWS::EKS::Addon:" + hashlib.sha256(encoded.encode()).hexdigest()
            scopes = connection.execute("SELECT last_page,complete,failed FROM scope_checkpoints WHERE scan_id=? AND collector='cloud_control_list' AND parent_id=?", (scan_id, parent_scope)).fetchall()
            add("addon_" + side + ":actual_emulator_pagination", len(scopes) == 1 and scopes[0][0] >= 2 and tuple(scopes[0][1:]) == (1, 0))
    except sqlite3.Error as error:
        add("scope_evidence", False, str(error)[:300])
    return checks
