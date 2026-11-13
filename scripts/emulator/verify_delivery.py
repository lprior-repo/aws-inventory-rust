#!/usr/bin/env python3
"""Check every seeded snapshot's JSON export and real FalkorDB projection.

Run inside scripts/emulator/mutate.py's owned fixture lifetime. This command
performs no AWS collection. Every intended projected property comes independently
from the normalized SQLite payloads; FalkorDB supplies the actual graph. Property
and label cardinalities also reject extra fields or labels that fixed-column
queries alone would miss. Nested JSON is compared structurally, not by spacing.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import uuid

MAX_FIXTURE_ROWS = 10_000
MAX_ARTIFACT_BYTES = 67_108_864
MAX_RECORD_BYTES = 2_359_296


def command(arguments: list[str], accepted: tuple[int, ...] = (0,)):
    # Bound decoded output before loading it into memory. subprocess.run owns
    # and waits for its child, including after its explicit timeout.
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        result = subprocess.run(arguments, stdin=subprocess.DEVNULL, stdout=stdout,
                                stderr=stderr, timeout=90, check=False)
        if stdout.tell() > 4_194_304 or stderr.tell() > 65_536:
            raise RuntimeError("verification command exceeded its output budget")
        stdout.seek(0)
        stderr.seek(0)
        if result.returncode not in accepted:
            detail = stderr.read(4096).decode("utf-8", errors="replace")
            raise RuntimeError(f"verification command exit {result.returncode}: {detail}")
        return json.loads(stdout.read(4_194_305))


def canonical(value) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(",", ":"), allow_nan=False)


def strict_json(value: str):
    # A second duplicate key must not conceal a changed first value. NaN and
    # Infinity are not JSON and cannot occur in a valid serde_json projection.
    def object_pairs(pairs):
        result = {}
        for key, item in pairs:
            if key in result:
                raise ValueError("duplicate JSON property")
            result[key] = item
        return result

    def invalid_constant(_value):
        raise ValueError("non-finite JSON number")

    def finite_float(value):
        result = float(value)
        if not math.isfinite(result):
            raise ValueError("overflowed JSON number")
        return result

    try:
        return json.loads(value, object_pairs_hook=object_pairs,
                          parse_constant=invalid_constant, parse_float=finite_float)
    except (ValueError, RecursionError) as error:
        raise RuntimeError("invalid projected JSON content") from error


def normalize_json_columns(rows, columns):
    # Preserve SQL NULL as absence; the JSON string "null" remains a present
    # property. This distinction matters for unresolved nodes and Value::Null.
    normalized = [list(row) for row in rows]
    for row in normalized:
        for column in columns:
            value = row[column]
            if value is not None:
                if type(value) is not str:
                    raise RuntimeError("projected JSON property is not a string")
                row[column] = canonical(strict_json(value))
    return normalized


def decode_graph_rows(response, columns: int, maximum: int = MAX_FIXTURE_ROWS):
    if (not isinstance(response, list) or len(response) != 3
            or not isinstance(response[0], list) or len(response[0]) != columns
            or not all(isinstance(item, str) for item in response[0])
            or not isinstance(response[1], list) or len(response[1]) > maximum
            or not isinstance(response[2], list)
            or not all(isinstance(item, str) for item in response[2])):
        raise RuntimeError("invalid or oversized noncompact graph response")
    for row in response[1]:
        if (not isinstance(row, list) or len(row) != columns
                or any(type(cell) not in (str, int, type(None)) for cell in row)
                or any(isinstance(cell, str) and len(cell.encode("utf-8")) > MAX_RECORD_BYTES
                       for cell in row)):
            raise RuntimeError("invalid noncompact graph scalar row")
    return response[1]


def graph_rows(redis_cli: Path, port: int, graph: str, query: str, columns: int):
    response = command([str(redis_cli), "-h", "127.0.0.1", "-p", str(port), "--json",
                        "GRAPH.RO_QUERY", graph, query])
    return decode_graph_rows(response, columns)


def bounded_rows(connection, query, scan_id):
    for index, row in enumerate(connection.execute(
            query + " LIMIT ?", (scan_id, MAX_FIXTURE_ROWS + 1))):
        if index >= MAX_FIXTURE_ROWS:
            raise RuntimeError("seeded SQL result exceeds fixture row bound")
        yield row


def expected(connection: sqlite3.Connection, scan_id: str) -> dict:
    # The Rust exporter reads these immutable normalized payloads. Do not derive
    # the expected projection from its generated Cypher plan or live graph.
    resources = {}
    payload_bytes = 0
    for identifier, payload in bounded_rows(connection,
            "SELECT id,resource_json FROM scan_resources WHERE scan_id=? ORDER BY id", scan_id):
        payload_bytes = projection_budget(payload, payload_bytes)
        resource = strict_json(payload)
        if resource["id"] != identifier or identifier in resources:
            raise RuntimeError("normalized resource identity differs from its SQL key")
        resources[identifier] = resource
    resource_ids = set(resources)
    edges = []
    for row in bounded_rows(connection,
            "SELECT from_id,to_id,relation,source,edge_json FROM scan_edges WHERE scan_id=? "
            "ORDER BY from_id,to_id,relation,source", scan_id):
        payload_bytes = projection_budget(row[4], payload_bytes)
        edges.append(row)
    graph_ids = resource_ids | {value for row in edges for value in row[:2]}
    if len(graph_ids) > MAX_FIXTURE_ROWS:
        raise RuntimeError("seeded graph exceeds fixture node bound")
    def count(table):
        # Fixed table names below are verification code, never user SQL.
        return connection.execute(f"SELECT count(*) FROM {table} WHERE scan_id=?", (scan_id,)).fetchone()[0]
    observations = {"resource": count("resource_observations"),
                    "edge": count("edge_observations"),
                    "scope": count("scope_checkpoints"),
                    "failure": count("collection_failures")}
    if any(value > MAX_FIXTURE_ROWS for value in observations.values()):
        raise RuntimeError("seeded observations exceed fixture row bound")
    state = connection.execute(
        "SELECT status,started_at,finished_at FROM scans WHERE scan_id=?", (scan_id,)).fetchone()
    if state is None or state[0] not in ("complete", "partial", "cancelled") or state[2] is None:
        raise RuntimeError("expected a finalized snapshot")
    completed, failed = connection.execute(
        "SELECT count(*) FILTER(WHERE complete=1 AND failed=0), "
        "count(*) FILTER(WHERE failed=1) FROM scope_checkpoints WHERE scan_id=?", (scan_id,)).fetchone()
    return {"counts": {"resources": len(resource_ids), "edges": len(edges),
                       "nodes": len(graph_ids), "unresolved": len(graph_ids - resource_ids)},
            "scan": {"scan_id": scan_id, "status": state[0], "started_at": state[1],
                     "finished_at": state[2], "resources": len(resource_ids), "edges": len(edges),
                     "completed_scopes": completed, "failed_scopes": failed},
            "observations": observations,
            "node_rows": [expected_node(scan_id, identifier, resources.get(identifier))
                          for identifier in sorted(graph_ids)],
            "edge_rows": [expected_edge(scan_id, row) for row in edges]}


def projection_budget(payload, current_bytes):
    size = len(payload.encode("utf-8"))
    if size > MAX_RECORD_BYTES or current_bytes + size > MAX_ARTIFACT_BYTES:
        raise RuntimeError("normalized projection exceeds fixture byte bound")
    return current_bytes + size


def expected_node(scan_id, identifier, resource):
    identity = [identifier, scan_id, "observed" if resource is not None else "unresolved",
                f"{scan_id}::{identifier}"]
    if resource is None:
        # Unresolved endpoints have exactly key/id/scan_id/unresolved and one label.
        return [*identity, *([None] * 8), 4, 1]
    return [*identity, resource["kind"], resource["account_id"], resource["region"],
            "" if resource["name"] is None else resource["name"],
            canonical(resource["properties"]), canonical(resource["tags"]),
            resource["source"], resource["observed_at"], 12, 1]


def expected_edge(scan_id, row):
    edge = strict_json(row[4])
    if tuple(edge[key] for key in ("from_id", "to_id", "relation", "source")) != tuple(row[:4]):
        raise RuntimeError("normalized edge identity differs from its SQL key")
    return [row[0], row[1], "RELATES_TO", row[2], row[3], scan_id, scan_id,
            edge["observed_at"], canonical(edge["properties"]), 4,
            f"{scan_id}::{row[0]}", f"{scan_id}::{row[1]}"]


def scope_fields(row):
    return dict(zip(("account_id", "region", "collector", "parent_id", "source"), row, strict=True))


def expected_records(connection, scan_id):
    # Build provenance from the structured scope columns, independently of the
    # exporter's scope_key JSON decoder. Raw stored payloads retain all fields.
    for tag, table, payload, ordering in (
            ("resource", "resource_observations", "resource_json", "r.resource_id"),
            ("edge", "edge_observations", "edge_json", "r.from_id,r.to_id,r.relation")):
        query = (f"SELECT r.{payload},r.page_number,c.account_id,c.region,c.collector,c.parent_id,c.source "
                 f"FROM {table} r JOIN scope_checkpoints c USING(scan_id,scope_key) "
                 f"WHERE r.scan_id=? ORDER BY {ordering},r.scope_key,r.page_number")
        for row in bounded_rows(connection, query, scan_id):
            yield {"record": tag, "data": {tag: json.loads(row[0]),
                   "observation": {"scope": scope_fields(row[2:]), "page_number": row[1]}}}
    for row in bounded_rows(connection,
            "SELECT last_page,complete,failed,account_id,region,collector,parent_id,source "
            "FROM scope_checkpoints WHERE scan_id=? ORDER BY scope_key", scan_id):
        yield {"record": "scope", "data": {"scope": scope_fields(row[3:]),
               "last_page": row[0], "complete": bool(row[1]), "failed": bool(row[2])}}
    for row in bounded_rows(connection,
            "SELECT f.category,f.message,c.account_id,c.region,c.collector,c.parent_id,c.source "
            "FROM collection_failures f JOIN scope_checkpoints c USING(scan_id,scope_key) "
            "WHERE f.scan_id=? ORDER BY f.scope_key,f.category,f.message", scan_id):
        yield {"record": "failure", "data": {"scope": scope_fields(row[2:]),
               "category": row[0], "message": row[1]}}


def verify_inventory(receipt: dict, model: dict, connection):
    inventory = receipt["outputs"]["inventory"]
    path = Path(inventory["path"])
    if not path.is_file() or path.stat().st_size > MAX_ARTIFACT_BYTES:
        raise RuntimeError("seeded inventory file missing or exceeds fixture bound")
    counts = {"resources": model["observations"]["resource"],
              "edges": model["observations"]["edge"], "scopes": model["observations"]["scope"],
              "failures": model["observations"]["failure"],
              "records": sum(model["observations"].values())}
    expected_summary = {"scan_id": model["scan"]["scan_id"], "status": model["scan"]["status"],
                        "format": "jsonl", "provenance": "all_observations", **counts}
    if canonical({key: inventory["summary"].get(key) for key in expected_summary}) != canonical(expected_summary):
        raise RuntimeError("inventory receipt identity, provenance or counts differ from SQLite")
    header = {"record": "header", "data": {"artifact": "aws-inventory.snapshot", "version": 1,
              "provenance": "all_observations", "scan": model["scan"]}}
    hasher = hashlib.sha256()
    byte_count = 0
    record_count = 0
    with path.open("rb") as stream:
        def read_record():
            nonlocal byte_count
            line = stream.readline(MAX_RECORD_BYTES + 1)
            if not line or not line.endswith(b"\n") or len(line) > MAX_RECORD_BYTES:
                raise RuntimeError("export record exceeds record bound")
            byte_count += len(line)
            if byte_count > MAX_ARTIFACT_BYTES:
                raise RuntimeError("export exceeds fixture byte bound")
            hasher.update(line)
            return json.loads(line)
        if canonical(read_record()) != canonical(header):
            raise RuntimeError("inventory header identity, status or counts differ from SQLite")
        for record_count, wanted in enumerate(expected_records(connection, model["scan"]["scan_id"]), 1):
            if canonical(read_record()) != canonical(wanted):
                raise RuntimeError(f"inventory data record {record_count} differs from SQL payload/provenance")
        if record_count != counts["records"]:
            raise RuntimeError("SQL provenance join does not cover every stored observation")
        if canonical(read_record()) != canonical({"record": "summary", "data": counts}):
            raise RuntimeError("inventory footer counts differ from SQLite")
        if stream.read(1):
            raise RuntimeError("unexpected data after inventory footer")
    digest = hasher.hexdigest()
    if (digest != inventory["summary"]["sha256"] or byte_count != inventory["summary"]["bytes"]
            or path.stat().st_size != byte_count):
        raise RuntimeError("published inventory bytes differ from its receipt")
    return {"bytes": byte_count, "sha256": digest, "counts": counts,
            "exact_sql_payload_and_provenance_match": True}


def validate_receipt(receipt, scan_id, status):
    wanted = {"version": 1, "scan_id": scan_id, "status": status,
              "collection_status": status, "delivery_status": "complete"}
    if (not isinstance(receipt, dict)
            or canonical({key: receipt.get(key) for key in wanted}) != canonical(wanted)
            or any(key in receipt for key in ("error", "receipt_error", "cleanup_error"))):
        raise RuntimeError("delivery receipt failed or changed the collection identity/outcome")


def verify_receipt_file(receipt, output):
    directory = Path(receipt["directory"]).resolve()
    if not directory.is_relative_to(output.resolve()):
        raise RuntimeError("delivery receipt points outside its owned output directory")
    for path in (receipt["outputs"]["inventory"]["path"], receipt["outputs"]["falkor"]["plan"]):
        if Path(path).resolve().parent != directory:
            raise RuntimeError("delivery artifact points outside its receipt directory")
    path = directory / "delivery.json"
    if path.resolve().parent != directory:
        raise RuntimeError("persisted delivery receipt points outside its owned directory")
    with path.open("rb") as stream:
        contents = stream.read(65_537)
    if len(contents) > 65_536 or canonical(json.loads(contents)) != canonical(receipt):
        raise RuntimeError("persisted delivery receipt differs from command output")
    return {"bytes": len(contents), "sha256": hashlib.sha256(contents).hexdigest()}


def verify_graph(redis_cli, port, graph, model):
    limit = MAX_FIXTURE_ROWS + 1
    nodes = graph_rows(redis_cli, port, graph,
        "MATCH (r:Resource) RETURN r.id,r.scan_id,"
        "CASE WHEN r.unresolved=true THEN 'unresolved' "
        "WHEN r.unresolved=false THEN 'observed' ELSE 'invalid' END,r.key,"
        "r.kind,r.account_id,r.region,r.name,r.properties_json,r.tags_json,"
        "r.source,r.observed_at,size(keys(r)),size(labels(r)) "
        f"ORDER BY r.id LIMIT {limit}", 14)
    edges = graph_rows(redis_cli, port, graph,
        "MATCH (a)-[e]->(b) RETURN a.id,b.id,type(e),e.kind,e.source,a.scan_id,b.scan_id,"
        "e.observed_at,e.properties_json,size(keys(e)),a.key,b.key "
        f"ORDER BY a.id,b.id,e.kind,e.source LIMIT {limit}", 12)
    marker = graph_rows(redis_cli, port, graph,
        "MATCH (s:InventoryScan) RETURN s.id,s.state,s.resources,s.edges,"
        "size(keys(s)),size(labels(s)) "
        f"LIMIT {limit}", 6)
    total = graph_rows(redis_cli, port, graph, "MATCH (n) RETURN count(n)", 1)
    wanted_marker = [[model["scan"]["scan_id"], model["scan"]["status"],
                      model["counts"]["resources"], model["counts"]["edges"], 4, 1]]
    nodes = normalize_json_columns(nodes, (8, 9))
    edges = normalize_json_columns(edges, (8,))
    if canonical(nodes) != canonical(model["node_rows"]):
        raise RuntimeError("FalkorDB node IDs, snapshot keys, labels or projected properties differ from SQLite")
    if canonical(edges) != canonical(model["edge_rows"]):
        raise RuntimeError("FalkorDB edge endpoints, relations, provenance or projected properties differ from SQLite")
    if canonical(marker) != canonical(wanted_marker) or canonical(total) != canonical([[len(nodes) + 1]]):
        raise RuntimeError("FalkorDB scan marker or complete graph node count differs from SQLite")
    return {**model["counts"], "state": model["scan"]["status"],
            "node_rows_sha256": hashlib.sha256(canonical(nodes).encode()).hexdigest(),
            "edge_rows_sha256": hashlib.sha256(canonical(edges).encode()).hexdigest(),
            "exact_sql_identity_and_provenance_match": True,
            "exact_sql_projected_properties_match": True,
            "exact_property_and_label_cardinality_match": True}


def verify_push(receipt, model, graph, replay):
    exported = receipt["outputs"]["falkor"]["export"]
    wanted = {"graph": graph, "scan_id": model["scan"]["scan_id"],
              "resources": model["counts"]["resources"], "edges": model["counts"]["edges"]}
    if canonical({key: exported.get(key) for key in wanted}) != canonical(wanted):
        raise RuntimeError("graph plan receipt differs from the SQL snapshot")
    batches = exported["batches"]
    pushed = receipt["outputs"]["falkor"]["push"]
    if type(batches) is not int or not 3 <= batches <= 1_000_000:
        raise RuntimeError("graph plan batch count outside its bound")
    if any(type(pushed.get(key)) is not int or pushed[key] < 0
           for key in ("applied_batches", "skipped_batches", "attempts", "retries", "reconnects")):
        raise RuntimeError("invalid graph push receipt counters")
    if replay:
        if pushed != {"applied_batches": 0, "skipped_batches": batches,
                      "attempts": 0, "retries": 0, "reconnects": 0}:
            raise RuntimeError("checkpoint resume resent or failed to skip acknowledged graph batches")
    elif (pushed["applied_batches"] != batches or pushed["skipped_batches"] != 0
          or pushed["attempts"] < batches):
        raise RuntimeError("first delivery did not apply every graph batch")
    return pushed


def cleanup_graph(redis_cli, port, graph):
    prefix = [str(redis_cli), "-h", "127.0.0.1", "-p", str(port), "--json"]
    exists = command([*prefix, "EXISTS", graph])
    if type(exists) is not int or exists not in (0, 1):
        raise RuntimeError("invalid graph cleanup existence response")
    if exists:
        removed = command([*prefix, "GRAPH.DELETE", graph])
        if removed != "OK":
            raise RuntimeError("graph deletion was not acknowledged")
        remaining = command([*prefix, "EXISTS", graph])
        if type(remaining) is not int or remaining != 0:
            raise RuntimeError("owned graph remains after deletion")


def verify_case(args, connection, scan_id: str, status: str, port: int, output: Path, case=None):
    if case is None:
        case = {"scan_id": scan_id, "status": status, "passed": False}
    graph = "inventory_delivery_" + uuid.uuid4().hex
    argv = [str(args.binary), "deliver", "--db", str(args.database), "--scan-id", scan_id,
            "--output-dir", str(output), "--provenance", "all-observations", "--falkor",
            "--falkor-url-env", "FALKORDB_TEST_URL", "--graph", graph, "--allow-partial"]
    primary = None
    cleanup = None
    try:
        model = expected(connection, scan_id)
        receipt = command(argv, (0,) if status == "complete" else (2,))
        validate_receipt(receipt, scan_id, status)
        receipt_file = verify_receipt_file(receipt, output)
        artifact = verify_inventory(receipt, model, connection)
        first_push = verify_push(receipt, model, graph, False)
        observed = verify_graph(args.redis_cli, port, graph, model)
        replay = command(argv, (0,) if status == "complete" else (2,))
        validate_receipt(replay, scan_id, status)
        replay_file = verify_receipt_file(replay, output)
        replay_artifact = verify_inventory(replay, model, connection)
        pushed = verify_push(replay, model, graph, True)
        if replay["outputs"]["falkor"]["export"] != receipt["outputs"]["falkor"]["export"]:
            raise RuntimeError("checkpoint resume changed the immutable graph plan receipt")
        if verify_graph(args.redis_cli, port, graph, model) != observed:
            raise RuntimeError("checkpoint resume changed graph topology")
        if replay_artifact != artifact:
            raise RuntimeError("checkpoint resume changed the deterministic inventory artifact")
        case.update(expected=model["counts"], observations=model["observations"], graph=observed,
                    artifact=artifact, first_push=first_push, resume_push=pushed,
                    first_receipt=receipt_file, resume_receipt=replay_file)
    except BaseException as error:
        primary = error
        case["error"] = str(error)[:4096]
    finally:
        try:
            cleanup_graph(args.redis_cli, port, graph)
            case["cleanup_complete"] = True
        except BaseException as error:
            cleanup = error
            case["cleanup_complete"] = False
            case["cleanup_error"] = str(error)[:4096]
    if primary is not None:
        if cleanup is not None and isinstance(primary, Exception):
            raise RuntimeError(f"{primary}; graph cleanup also failed: {cleanup}") from primary
        raise primary
    if cleanup is not None:
        raise cleanup
    case["passed"] = True
    return case


def main():
    # Pure verifier tests do not need the AWS fixture owner's boto3 dependency.
    from mutate import execute_command, fixture_health

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--redis-cli", type=Path, required=True)
    parser.add_argument("--database", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--run-workspace-tests", action="store_true",
                        help="run all ordinary and explicitly enabled live Rust tests inside this fixture")
    args = parser.parse_args()
    fixture_health()
    url = os.environ["FALKORDB_TEST_URL"]
    prefix = "redis://127.0.0.1:"
    if not url.startswith(prefix) or not 1 <= int(url[len(prefix):]) <= 65535:
        raise RuntimeError("local fixture URL required")
    port = int(url[len(prefix):])
    report = {"passed": False, "cases": [], "semantics": "seeded local emulator snapshots and real local FalkorDB; no live AWS account or throughput claim"}
    try:
        if args.run_workspace_tests:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            tests = execute_command(
                ["cargo", "test", "--workspace", "--exclude", "inventory-sdk-catalog",
                 "--all-features", "--locked", "--offline", "-j1", "--",
                 "--include-ignored", "--test-threads=1"],
                Path(__file__).resolve().parents[2], args.output.parent, "workspace-live", 900)
            report["tests"] = tests
            if tests.get("status") != "completed" or tests.get("exit_code") != 0:
                raise RuntimeError("full Rust workspace live suite did not pass")
        with tempfile.TemporaryDirectory(prefix="inventory-delivery-check-") as directory:
            with sqlite3.connect(f"file:{args.database.resolve()}?mode=ro", uri=True) as connection:
                scans = list(connection.execute("SELECT scan_id,status FROM scans ORDER BY started_at,scan_id LIMIT 6"))
                if len(scans) != 5 or any(status not in ("complete", "partial", "cancelled") for _, status in scans):
                    raise RuntimeError("expected exactly five finalized fixture scans")
                for scan_id, status in scans:
                    case = {"scan_id": scan_id, "status": status, "passed": False}
                    report["cases"].append(case)
                    verify_case(args, connection, scan_id, status, port, Path(directory), case)
        fixture_health()
        report["passed"] = True
    except Exception as error:
        report["error"] = str(error)[:4096]
        raise
    finally:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"passed": True, "cases": len(report["cases"]), "receipt": str(args.output)}))


if __name__ == "__main__":
    main()
