"""Assert collected global architecture using authoritative local fixture IDs.

`check(connection, scan_id, fixtures=None)` returns harness-compatible checks.
Only direct-source observations from the specified native scan are inspected;
Cloud Control and later scans cannot satisfy a missing native assertion.

Pass seed_global's `report['fixtures']` as the optional third argument. Without
it, the helper reads the adjacent seed-results.json belonging to that database.
Expected identifiers are never inferred from the output being tested. The
read-only `validate(db_path, scan_id)` wrapper supports saved run evidence.
"""

from __future__ import annotations

from collections import Counter
import json
from pathlib import Path
import sqlite3
from typing import Any

MAX_ROWS = 20_000
MAX_JSON_BYTES = 32 * 1024 * 1024
MAX_SEED_BYTES = 4 * 1024 * 1024
GLOBAL_PREFIXES = ("aws_s3", "aws_iam", "aws_route53", "aws_cloudfront")
OMISSION_SENTINELS = (
    "inventory-fixture-txt-must-not-be-collected",
    "fixture-header-must-not-be-collected",
    "omit-this-origin-path",
)


def _fixture_metadata(connection: sqlite3.Connection) -> dict[str, Any] | None:
    for row in connection.execute("PRAGMA database_list"):
        if row[1] == "main" and row[2]:
            path = Path(row[2]).parent / "seed-results.json"
            if path.is_file() and path.stat().st_size <= MAX_SEED_BYTES:
                data = json.loads(path.read_text())
                return data.get("modules", {}).get("seed_global", {}).get("fixtures")
    return None


def _observations(connection: sqlite3.Connection, table: str, column: str, scan_id: str) -> list[dict[str, Any]]:
    # Table/column names are module constants, never caller-controlled SQL.
    if (table, column) not in {
        ("resource_observations", "resource_json"),
        ("edge_observations", "edge_json"),
    }:
        raise ValueError("unsupported observation table")
    results = []
    byte_count = 0
    for row in connection.execute(
        f"SELECT {column} FROM {table} WHERE scan_id=? AND source='direct' LIMIT ?",
        (scan_id, MAX_ROWS + 1),
    ):
        byte_count += len(row[0].encode("utf-8"))
        if len(results) >= MAX_ROWS or byte_count > MAX_JSON_BYTES:
            raise ValueError("global fixture assertion observation budget exceeded")
        value = json.loads(row[0])
        if not isinstance(value, dict):
            raise ValueError("inventory observation is not an object")
        results.append(value)
    return results


def check(
    connection: sqlite3.Connection,
    scan_id: str,
    fixtures: dict[str, Any] | None = None,
) -> list[dict[str, Any]]:
    """Check one complete native fixture scan without modifying the database."""
    checks: list[dict[str, Any]] = []

    def add(name: str, passed: Any, detail: Any = None) -> None:
        value = {"check": "global:" + name, "passed": bool(passed)}
        if detail is not None:
            value["detail"] = detail
        checks.append(value)

    try:
        fixtures = fixtures if fixtures is not None else _fixture_metadata(connection)
        rows = _observations(connection, "resource_observations", "resource_json", scan_id)
        edges = _observations(connection, "edge_observations", "edge_json", scan_id)
    except (ValueError, OSError, sqlite3.Error) as error:
        add("readable_bounded_evidence", False, type(error).__name__ + ": " + str(error)[:300])
        return checks
    add("authoritative_fixture_metadata", isinstance(fixtures, dict))
    if not isinstance(fixtures, dict):
        return checks

    expected_counts = {"policy_arns": 3, "role_arns": 2, "profile_arns": 2}
    iam = fixtures.get("iam", {})
    s3 = fixtures.get("s3", {})
    dns = fixtures.get("route53", {})
    cdn = fixtures.get("cloudfront", {})
    complete_fixture_ids = (
        all(len(iam.get(key, [])) == count for key, count in expected_counts.items())
        and all(s3.get(key) for key in (
            "rich_bucket", "empty_bucket", "logs_bucket", "replica_bucket", "kms_key_arn",
            "notification_topic_arn", "notification_queue_arn",
        ))
        and bool(iam.get("replication_role_arn"))
        and bool(dns.get("vpc_id"))
        and len(dns.get("zones", [])) == 2
        and len(cdn.get("distributions", [])) == 2
    )
    add("fixture_parent_ids_complete", complete_fixture_ids)
    if not complete_fixture_ids:
        return checks

    status = connection.execute("SELECT status FROM scans WHERE scan_id=?", (scan_id,)).fetchone()
    add("native_scan_complete", status is not None and status[0] == "complete")
    resources = {row.get("id"): row for row in rows}
    edge_keys = {(edge.get("from_id"), edge.get("to_id"), edge.get("relation")) for edge in edges}

    def properties(identifier: str) -> dict[str, Any]:
        value = resources.get(identifier, {}).get("properties", {})
        return value if isinstance(value, dict) else {}

    def edge(source: str, target: str, relation: str) -> bool:
        return (source, target, relation) in edge_keys

    def section(value: dict[str, Any], key: str) -> dict[str, Any]:
        # Unavailable S3 sections are explicitly null. A partial scan must fail
        # the appropriate assertion, not abort validation with AttributeError.
        nested = value.get(key)
        return nested if isinstance(nested, dict) else {}

    rich = "arn:aws:s3:::" + s3["rich_bucket"]
    empty = "arn:aws:s3:::" + s3["empty_bucket"]
    rich_properties = properties(rich)
    empty_properties = properties(empty)
    add("rich_details_complete", rich_properties.get("details_available")
        and rich_properties.get("tags_available") and rich_properties.get("unavailable_sections") == [])
    add("rich_policy_authoritative_private", rich_properties.get("policy_status") == {"configured": True, "is_public": False},
        rich_properties.get("policy_status"))
    add("empty_policy_absent", empty_properties.get("policy_status") == {"configured": False},
        empty_properties.get("policy_status"))
    add("bucket_actual_region", rich_properties.get("bucket_region") == "us-east-1"
        and resources.get(rich, {}).get("region") == "us-east-1")
    add("versioning_enabled", section(rich_properties, "versioning").get("status") == "Enabled")
    add("kms_encryption_identity", any(rule.get("kms_key_id") == s3["kms_key_arn"]
        for rule in section(rich_properties, "encryption").get("rules", [])))
    add("public_block_all_true", all(section(rich_properties, "public_access_block").get(key) is True
        for key in ("block_public_acls", "ignore_public_acls", "block_public_policy", "restrict_public_buckets")))
    for relation, target in (
        ("ASSUMES_ROLE", iam["replication_role_arn"]),
        ("LOGS_TO", "arn:aws:s3:::" + s3["logs_bucket"]),
        ("REPLICATES_TO", "arn:aws:s3:::" + s3["replica_bucket"]),
        ("ENCRYPTED_WITH", s3["kms_key_arn"]),
        ("REPLICA_ENCRYPTED_WITH", s3["kms_key_arn"]),
        ("NOTIFIES_TOPIC", s3["notification_topic_arn"]),
        ("NOTIFIES_QUEUE", s3["notification_queue_arn"]),
    ):
        add("rich_edge_" + relation, edge(rich, target, relation))
    add("rich_website_redirect", any(source == rich and relation == "REDIRECTS_TO_DNS"
        for source, _, relation in edge_keys))
    add("s3_object_and_raw_policy_omitted", rich_properties.get("object_data_collected") is False
        and rich_properties.get("raw_policy_collected") is False)
    add("notification_eventbridge_metadata", section(rich_properties, "notifications").get("eventbridge_enabled") is True)

    for index, arn in enumerate(iam["role_arns"]):
        value = properties(arn)
        add(f"iam_role_detail_{index}", value.get("details_available")
            and value.get("observation_detail") == "get_role" and value.get("trust_policy_values_omitted"))
        add(f"iam_role_boundary_{index}", value.get("permissions_boundary_arn") == iam["policy_arns"][0])
        add(f"iam_role_trust_edge_{index}", any(source == arn and relation == "DECLARES_TRUST"
            for source, _, relation in edge_keys))
    for index, (profile, role) in enumerate(zip(iam["profile_arns"], iam["role_arns"])):
        value = properties(profile)
        add(f"iam_profile_detail_{index}", value.get("details_available")
            and value.get("role_arns") == [role] and edge(profile, role, "HAS_ROLE"))
    for index, arn in enumerate(iam["policy_arns"]):
        value = properties(arn)
        add(f"iam_policy_metadata_{index}", value.get("details_available")
            and value.get("policy_document_collected") is False and value.get("policy_scope") == "Local")

    for zone in dns["zones"]:
        arn = "arn:aws:route53:::hostedzone/" + zone["id"]
        value = properties(arn)
        label = zone["kind"]
        add("zone_detail_" + label, value.get("details_available") and value.get("tags_available"))
        if label == "private":
            add("private_zone_vpc_edge", edge(arn, "arn:aws:ec2:us-east-1:000000000000:vpc/" + dns["vpc_id"], "ASSOCIATED_WITH_VPC"))
        records = [row for row in rows if row.get("kind") == "aws_route53_record"
                   and row.get("properties", {}).get("zone_id") == zone["id"]]
        mx = [row for row in records if row["properties"].get("type") == "MX"]
        txt = [row for row in records if row["properties"].get("type") == "TXT"]
        weighted = [row for row in records if row["properties"].get("set_identifier")]
        add("null_mx_preserved_" + label, len(mx) == 1 and mx[0]["properties"].get("records") == ["0 ."]
            and not any(source == mx[0]["id"] and relation == "DNS_TARGET" for source, _, relation in edge_keys))
        add("txt_omitted_" + label, len(txt) == 1 and txt[0]["properties"].get("records") == []
            and txt[0]["properties"].get("rdata_omitted") and txt[0]["properties"].get("rdata_count") == 1)
        add("weighted_cursor_identity_" + label, len(weighted) == 2
            and {row["properties"]["set_identifier"] for row in weighted} == {"blue", "green"}
            and len({row["id"] for row in weighted}) == 2)

    for index, distribution in enumerate(cdn["distributions"]):
        arn = distribution["arn"]
        value = properties(arn)
        behaviors = value.get("cache_behaviors", [])
        add(f"distribution_tags_{index}", value.get("tags_available"))
        add(f"distribution_origins_{index}", len(value.get("origins", [])) == 2)
        add(f"distribution_behaviors_{index}", section(value, "default_cache_behavior").get("target_in_configuration")
            and len(behaviors) == 1 and behaviors[0].get("target_in_configuration"))
        add(f"distribution_s3_origin_{index}", edge(arn, rich, "ORIGINATES_FROM"))
        add(f"distribution_custom_dns_origin_{index}", any(source == arn and relation == "ORIGINATES_FROM" and target != rich
            for source, target, relation in edge_keys))
    encoded = json.dumps(rows)
    add("sensitive_fixture_values_omitted", all(sentinel not in encoded for sentinel in OMISSION_SENTINELS))
    kind_counts = Counter(row.get("kind") for row in rows)
    global_counts = {kind: count for kind, count in kind_counts.items() if isinstance(kind, str) and kind.startswith(GLOBAL_PREFIXES)}
    add("all_seven_global_kinds", len(global_counts) == 7, global_counts)
    add("all_39_native_kinds", len(kind_counts) == 39, {"distinct_kinds": len(kind_counts)})
    return checks


def validate(db_path: str | Path, scan_id: str, fixtures: dict[str, Any] | None = None) -> list[dict[str, Any]]:
    """Read a saved run; no SQLite write or fixture mutation is performed."""
    path = Path(db_path).resolve()
    with sqlite3.connect(path.as_uri() + "?mode=ro", uri=True) as connection:
        return check(connection, scan_id, fixtures)
