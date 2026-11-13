#!/usr/bin/env python3
"""Reconcile current Rust SDK call sites without rewriting historical evidence.

This is a bounded lexical audit, not Rust type checking or execution evidence.
Unrecognized call shapes fail closed and require review. No Cargo or AWS calls.
"""

from __future__ import annotations

import argparse
from collections import Counter
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/emulator"))
from fixture_provenance import canonical_sha256, expected_runtime_catalog, file_sha256, source_fingerprint

MAX_FILES = 4_096
MAX_FILE_BYTES = 8 * 1024 * 1024
MAX_SOURCE_BYTES = 32 * 1024 * 1024
MAX_TOKENS = 500_000
MAX_CALLS = 4_096
MAX_REQUESTS = 100_000
TOKEN = re.compile(r"::|[A-Za-z_][A-Za-z_0-9]*|[^\s]")
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z_0-9]*\Z")
CHARACTER = re.compile(r"'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]{1,6}\}|.)|[^\\'\n])'")
RAW_STRING = re.compile(r'(?:br|cr|r)(#{0,255})"')
ADDITIONS_NAME = "catalog/aws-operation-additions.json"
INVENTORY_NAME = "catalog/aws-sdk-operations.json"
MAX_ADDITIONS = 512
ADDITION_STATUSES = {"unexercised", "pending_execution"}
SOURCE_PREFIX = "crates/aws/src/"
HEX64 = re.compile(r"[0-9a-f]{64}\Z")


class RustLexicalError(ValueError):
    """A bounded lexical failure with its original source character offset."""

    def __init__(self, message, offset):
        super().__init__(message)
        self.offset = offset


def location(source, offset):
    return source.count("\n", 0, offset) + 1, offset - source.rfind("\n", 0, offset)


def source_error(path, source, offset, message):
    line, column = location(source, offset)
    return ValueError(f"{path}:{line}:{column}: {message}")


def read_bytes(path: Path, limit=MAX_FILE_BYTES):
    with path.open("rb") as stream:
        value = stream.read(limit + 1)
    if len(value) > limit:
        raise ValueError(f"input byte budget exceeded: {path}")
    return value


def read_json(path: Path):
    raw = read_bytes(path)
    return json.loads(raw), hashlib.sha256(raw).hexdigest()


def mask_noncode(text: str):
    """Keep offsets/newlines while removing strings, chars and nested comments."""
    output, position = list(text), 0
    while position < len(text):
        start, end = position, None
        if text.startswith("//", position):
            found = text.find("\n", position)
            end = len(text) if found < 0 else found
        elif text.startswith("/*", position):
            depth, cursor = 1, position + 2
            while cursor < len(text) and depth:
                if text.startswith("/*", cursor):
                    depth += 1
                    if depth > 64:
                        raise RustLexicalError("Rust comment nesting exceeds audit bound", cursor)
                    cursor += 2
                elif text.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            if depth:
                raise RustLexicalError("unterminated Rust block comment", position)
            end = cursor
        else:
            raw = RAW_STRING.match(text, position)
            if raw and (position == 0 or not (text[position - 1].isalnum() or text[position - 1] == "_")):
                ending = '"' + raw.group(1)
                found = text.find(ending, raw.end())
                if found < 0:
                    raise RustLexicalError("unterminated Rust raw string", position)
                end = found + len(ending)
            elif text[position] == '"':
                cursor = position + 1
                while cursor < len(text):
                    if text[cursor] == "\\":
                        cursor += 2
                    elif text[cursor] == '"':
                        end = cursor + 1
                        break
                    else:
                        cursor += 1
                if end is None:
                    raise RustLexicalError("unterminated Rust string", position)
            elif text[position] == "'":
                char = CHARACTER.match(text, position)
                if char:
                    end = char.end()
        if end is None:
            position += 1
            continue
        for index in range(start, end):
            if text[index] != "\n":
                output[index] = " "
        position = end
    return "".join(output)


def tokens_and_pairs(text):
    tokens, stack, pairs = [], [], {}
    matching = {")": "(", "]": "[", "}": "{"}
    closing = {opening: end for end, opening in matching.items()}
    for match in TOKEN.finditer(text):
        if len(tokens) >= MAX_TOKENS:
            raise RustLexicalError("Rust token budget exceeded", match.start())
        index, value = len(tokens), match.group()
        tokens.append((value, match.start()))
        if value in ("(", "[", "{"):
            stack.append(index)
        elif value in matching:
            if not stack:
                raise RustLexicalError(f"unexpected closing Rust delimiter {value!r}", match.start())
            if tokens[stack[-1]][0] != matching[value]:
                opened, offset = tokens[stack[-1]]
                line, column = location(text, offset)
                raise RustLexicalError(
                    f"unbalanced Rust delimiters: {value!r} closes {opened!r} opened at "
                    f"{line}:{column}; expected {closing[opened]!r}", match.start())
            opening = stack.pop()
            pairs[opening] = index
            pairs[index] = opening
    if stack:
        opened, offset = tokens[stack[-1]]
        raise RustLexicalError(f"unclosed Rust delimiter {opened!r}; expected {closing[opened]!r} before end of file", offset)
    return tokens, pairs


def function_at(masked, offset):
    matches = list(re.finditer(r"\bfn\s+([A-Za-z_][A-Za-z_0-9]*)\b", masked[:offset]))
    if not matches:
        raise ValueError("SDK send is not inside a recognized function")
    return matches[-1].group(1), matches[-1].start()


def receiver_service(masked, receiver, function_start, offset):
    scope = masked[function_start:offset]
    binding = re.compile(r"\b" + re.escape(receiver) +
                         r"\s*(?::\s*&?\s*aws_sdk_([a-z0-9_]+)::Client\b|"
                         r"=\s*aws_sdk_([a-z0-9_]+)::Client\s*::)")
    matches = list(binding.finditer(scope))
    if matches:
        return matches[-1].group(1) or matches[-1].group(2)
    imports = set()
    for match in re.finditer(r"\buse\s+aws_sdk_([a-z0-9_]+)::\s*([^;]+);", masked):
        body = re.sub(r"\s+", "", match.group(2))
        if body == "Client" or body.startswith(("{Client,", "{Client}")):
            imports.add(match.group(1))
    if len(imports) != 1:
        raise ValueError(f"cannot uniquely resolve SDK client receiver {receiver}")
    return next(iter(imports))


def calls_in_source(path, source):
    try:
        masked = mask_noncode(source)
        tokens, pairs = tokens_and_pairs(masked)
    except RustLexicalError as error:
        raise source_error(path, source, error.offset, str(error)) from error
    calls, nonempty_sends = [], 0
    for index in range(max(0, len(tokens) - 3)):
        if [row[0] for row in tokens[index:index + 3]] != [".", "send", "("]:
            continue
        if pairs[index + 2] != index + 3:
            nonempty_sends += 1
            continue
        cursor, methods = index - 1, []
        while cursor >= 0 and tokens[cursor][0] == ")":
            opening = pairs[cursor]
            if opening < 2 or tokens[opening - 2][0] != "." or not IDENTIFIER.fullmatch(tokens[opening - 1][0]):
                break
            methods.append(tokens[opening - 1])
            cursor = opening - 3
        if not methods or cursor < 0 or not IDENTIFIER.fullmatch(tokens[cursor][0]):
            raise source_error(path, source, tokens[index][1], "unrecognized empty send call chain")
        operation, operation_offset = methods[-1]
        receiver = tokens[cursor][0]
        try:
            function, function_start = function_at(masked, tokens[index][1])
            service = receiver_service(masked, receiver, function_start, tokens[index][1])
        except ValueError as error:
            raise source_error(path, source, tokens[index][1], str(error)) from error
        calls.append({"id": f"{service}.{operation}", "path": path, "function": function,
                      "sdk_service": service, "sdk_operation": operation, "receiver": receiver,
                      "operation_line": source.count("\n", 0, operation_offset) + 1,
                      "send_line": source.count("\n", 0, tokens[index + 1][1]) + 1})
        if len(calls) > MAX_CALLS:
            raise ValueError("SDK send call budget exceeded")
    return calls, nonempty_sends


def source_inventory(root):
    sources, total = {}, 0
    for path in (root / "crates/aws/src").rglob("*.rs"):
        if len(sources) >= MAX_FILES or path.is_symlink():
            raise ValueError("unsupported AWS source inventory")
        raw = read_bytes(path)
        total += len(raw)
        if total > MAX_SOURCE_BYTES:
            raise ValueError("AWS source byte budget exceeded")
        sources[str(path.relative_to(root))] = raw.decode()
    if not sources:
        raise ValueError("empty AWS source inventory")
    calls, nonempty = [], 0
    for path, source in sorted(sources.items()):
        found, skipped = calls_in_source(path, source)
        calls.extend(found)
        nonempty += skipped
        if len(calls) > MAX_CALLS:
            raise ValueError("SDK call budget exceeded")
    return sources, calls, nonempty


def registry_file(crate, version, relative):
    base = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))) / "registry/src"
    roots = list(base.iterdir())
    if len(roots) > 32:
        raise ValueError("Cargo registry source root budget exceeded")
    matches = [path / f"{crate}-{version}" / relative for path in roots
               if (path / f"{crate}-{version}" / relative).is_file()]
    if len(matches) != 1:
        raise ValueError(f"expected one cached pinned SDK source: {crate} {version} {relative}")
    return matches[0]



def source_path(path):
    return (isinstance(path, str) and path.startswith(SOURCE_PREFIX)
            and all(part not in ("", ".", "..") for part in path.split("/")))


def hex64(value):
    return isinstance(value, str) and HEX64.fullmatch(value) is not None


def addition_sites(row, ident):
    values = row.get("source_calls")
    if not isinstance(values, list) or not values:
        raise ValueError("invalid reviewed additions document")
    sites, seen = [], set()
    for site in values:
        path, function = site.get("path"), site.get("function")
        if not source_path(path) or not isinstance(function, str) or not function:
            raise ValueError("invalid reviewed additions document")
        lines = [site.get(name) for name in ("builder_line", "operation_line", "send_line")]
        if any(type(line) is not int or line < 1 for line in lines):
            raise ValueError(f"invalid reviewed callsite lines for {ident}")
        if lines != sorted(lines):
            raise ValueError(f"invalid reviewed callsite lines for {ident}")
        if (path, function) in seen:
            raise ValueError("invalid reviewed additions document")
        seen.add((path, function))
        sites.append(site)
    return sites


def addition_sources(root, row, sites):
    hashes = row.get("source_sha256")
    paths = {site["path"] for site in sites}
    if not isinstance(hashes, dict) or set(hashes) != paths:
        raise ValueError("invalid reviewed additions document")
    for path, digest in hashes.items():
        if not source_path(path) or not hex64(digest):
            raise ValueError("invalid reviewed additions document")
        current = hashlib.sha256(read_bytes(root / path)).hexdigest()
        if current != digest:
            raise ValueError(f"reviewed addition source changed since review: {path}")
    return dict(hashes)


def addition_wire(row, ident, service, module):
    wire = row.get("wire_source")
    if not isinstance(wire, dict):
        raise ValueError("invalid reviewed additions document")
    expected = f"src/operation/{module}.rs"
    if (wire.get("service") != service or wire.get("rust_operation_module") != expected
            or not hex64(wire.get("rust_operation_module_sha256"))
            or not hex64(wire.get("operation_model_sha256"))):
        raise ValueError(f"pinned SDK operation module changed: {ident}")
    return wire["rust_operation_module_sha256"]


def addition_status(row, ident):
    coverage = row.get("coverage")
    if not isinstance(coverage, dict):
        raise ValueError("invalid reviewed additions document")
    if coverage.get("status") not in ADDITION_STATUSES:
        raise ValueError(f"reviewed addition {ident} must not claim an emulator outcome")
    if (coverage.get("execution_receipt") is not None
            or not isinstance(coverage.get("reason"), str) or not coverage["reason"]):
        raise ValueError(f"reviewed addition {ident} must not claim an emulator outcome")
    return coverage["status"]


def inventory_index(inventory):
    index = {}
    for entry in inventory.get("crates", []):
        name = entry.get("name", "")
        service = name.removeprefix("aws-sdk-")
        if not name.startswith("aws-sdk-") or not service:
            raise ValueError("invalid SDK operation inventory")
        operations = {row["module"]: row["api_operation"] for row in entry.get("operations", [])}
        index[(name, entry.get("version", ""))] = (service, operations)
    return index


def load_additions(root, matrix, matrix_sha):
    path = root / ADDITIONS_NAME
    if not path.is_file():
        return None
    document, digest = read_json(path)
    if document.get("schema_version") != 1:
        raise ValueError("invalid reviewed additions document")
    binds = document.get("binds") or {}
    if (binds.get("historical_matrix") or {}).get("sha256") != matrix_sha:
        raise ValueError("reviewed additions bind a different historical operation matrix")
    inventory, inventory_sha = read_json(root / INVENTORY_NAME)
    binding = binds.get("operation_inventory") or {}
    if binding.get("path") != INVENTORY_NAME or binding.get("sha256") != inventory_sha:
        raise ValueError("reviewed additions bind a different SDK operation inventory")
    reviewed_at = (document.get("review") or {}).get("reviewed_at_utc")
    if not isinstance(reviewed_at, str) or not reviewed_at:
        raise ValueError("invalid reviewed additions document")
    rows = document.get("additions")
    if not isinstance(rows, list) or not rows or len(rows) > MAX_ADDITIONS:
        raise ValueError("invalid reviewed additions document")
    matrix_ids = {row["id"] for row in matrix.get("operations", [])}
    index, seen, keys = inventory_index(inventory), set(), set()
    for row in rows:
        ident, service = row.get("id"), row.get("sdk_service")
        crate, version = row.get("sdk_crate"), row.get("sdk_version")
        module, api = row.get("sdk_operation"), row.get("api_operation")
        boto = row.get("boto3_service")
        if (row.get("explicit_sdk_send") is not True or not isinstance(boto, str) or not boto
                or ident != f"{service}.{module}"):
            raise ValueError(f"reviewed addition {ident} disagrees with its crate/module")
        if ident in matrix_ids:
            raise ValueError(f"reviewed addition {ident} reuses a historical operation identity")
        if "emulator_coverage" in row:
            raise ValueError(f"reviewed addition {ident} must not claim an emulator outcome")
        sites = addition_sites(row, ident)
        addition_sources(root, row, sites)
        addition_wire(row, ident, service, module)
        addition_status(row, ident)
        resolved = index.get((crate, version))
        if resolved is None or resolved[0] != service or resolved[1].get(module) != api:
            raise ValueError(f"reviewed addition {ident} is absent from {INVENTORY_NAME}")
        key = (crate, version, module)
        if ident in seen or key in keys:
            raise ValueError("invalid reviewed additions document")
        seen.add(ident)
        keys.add(key)
    return {"path": ADDITIONS_NAME, "sha256": digest, "reviewed_at_utc": reviewed_at,
            "matrix_sha256": matrix_sha, "inventory_sha256": inventory_sha,
            "additions": rows}


def addition_expected(additions):
    if not additions:
        return []
    return [{**site, "id": row["id"], "origin": "reviewed_addition"}
            for row in additions["additions"] for site in row["source_calls"]]


def addition_dependency_evidence(root, additions):
    if not additions:
        return []
    lock = tomllib.loads(read_bytes(root / "Cargo.lock").decode())
    versions = {}
    for package in lock["package"]:
        versions.setdefault(package["name"], set()).add(package["version"])
    rows = []
    for operation in additions["additions"]:
        crate, version = operation["sdk_crate"], operation["sdk_version"]
        if versions.get(crate) != {version}:
            raise ValueError(f"pinned SDK version changed: {crate}")
        expected = operation["wire_source"]
        digest = hashlib.sha256(read_bytes(registry_file(
            crate, version, expected["rust_operation_module"]))).hexdigest()
        if digest != expected["rust_operation_module_sha256"]:
            raise ValueError(f"pinned SDK operation module changed: {operation['id']}")
        rows.append({"id": operation["id"], "sdk_crate": crate, "sdk_version": version,
                     "api_operation": operation["api_operation"], "module_sha256": digest})
    return rows


def check_additions(root):
    matrix, matrix_sha = read_json(root / "catalog/aws-operations.json")
    additions = load_additions(root, matrix, matrix_sha)
    if additions:
        paths = sorted({site["path"] for row in additions["additions"]
                        for site in row["source_calls"]})
        calls = []
        for path in paths:
            found, _skipped = calls_in_source(root / path, read_bytes(root / path).decode())
            calls.extend(found)
        for row in additions["additions"]:
            declared = Counter((site["path"], site["function"])
                               for site in row["source_calls"])
            current = Counter((str(call["path"].relative_to(root)), call["function"])
                              for call in calls if call["id"] == row["id"])
            if declared != current:
                raise ValueError(f"reviewed addition {row['id']} callsites changed; "
                                 f"re-review {ADDITIONS_NAME}")
    addition_dependency_evidence(root, additions)
    rows = additions["additions"] if additions else []
    return {"additions": len(rows), "callsites": sum(len(row["source_calls"]) for row in rows),
            "status": "applied" if additions else "absent"}


def dependency_evidence(root, matrix):
    lock = tomllib.loads(read_bytes(root / "Cargo.lock").decode())
    versions = {}
    for package in lock["package"]:
        versions.setdefault(package["name"], set()).add(package["version"])
    rows = []
    for operation in matrix["operations"]:
        crate, version = operation["sdk_crate"], operation["sdk_version"]
        if versions.get(crate) != {version}:
            raise ValueError(f"pinned SDK version changed: {crate}")
        expected = operation["wire_source"]
        path = registry_file(crate, version, expected["rust_operation_module"])
        digest = hashlib.sha256(read_bytes(path)).hexdigest()
        if digest != expected["rust_operation_module_sha256"]:
            raise ValueError(f"pinned SDK operation module changed: {operation['id']}")
        rows.append({"id": operation["id"], "sdk_crate": crate, "sdk_version": version,
                     "api_operation": operation["api_operation"], "module_sha256": digest})
    provider = matrix["credential_provider_boundaries"][0]["provider_source"]
    if versions.get(provider["crate"]) != {provider["version"]}:
        raise ValueError("pinned credential provider version changed")
    digest = hashlib.sha256(read_bytes(registry_file(provider["crate"], provider["version"], provider["path"]))).hexdigest()
    if digest != provider["sha256"]:
        raise ValueError("pinned credential provider source changed")
    return rows, {**provider, "current_sha256": digest}


def key(row):
    return row["path"], row["function"], row["id"]


def catalogs(root):
    catalog, catalog_sha = read_json(root / "catalog/cloudcontrol-resources.json")
    parents, parent_sha = read_json(root / "catalog/cloudcontrol-parent-resources.json")
    if parents.get("evidence", {}).get("root_catalog_sha256") != catalog_sha:
        raise ValueError("parent catalog is not bound to the current frozen root catalog")
    return catalog, catalog_sha, parents, parent_sha, expected_runtime_catalog(catalog, parents)


def reconcile(root):
    before = source_fingerprint(root)
    matrix_path = root / "catalog/aws-operations.json"
    previous_path = root / "artifacts/aws-operation-source-reconciliation-v04.json"
    matrix, matrix_sha = read_json(matrix_path)
    previous, previous_sha = read_json(previous_path)
    if previous["historical_matrix"]["sha256"] != matrix_sha:
        raise ValueError("historical operation matrix changed since the preserved v0.4 audit")
    operations = {row["id"]: row for row in matrix["operations"]}
    if len(operations) != len(matrix["operations"]) or len(operations) > MAX_CALLS:
        raise ValueError("duplicate or excessive historical operation IDs")
    additions = load_additions(root, matrix, matrix_sha)
    expected = [{**site, "id": operation["id"], "origin": "historical_matrix"}
                for operation in matrix["operations"] for site in operation["source_calls"]]
    expected += addition_expected(additions)
    sources, calls, nonempty = source_inventory(root)
    closure_files = {row["file"]: row["sha256"] for row in before["files"]}
    if any(closure_files.get(path) != hashlib.sha256(text.encode()).hexdigest()
           for path, text in sources.items()):
        raise ValueError("AWS source changed while the audit snapshot was being read")
    expected_counts, actual_counts = Counter(map(key, expected)), Counter(map(key, calls))
    missing, added = expected_counts - actual_counts, actual_counts - expected_counts
    if missing or added:
        raise ValueError(f"SDK operation boundary changes require review: missing={dict(missing)} added={dict(added)}")
    expected_by_key = {key(row): row for row in expected}
    if len(expected_by_key) != len(expected):
        raise ValueError("historical callsites are ambiguous within one function")
    for call in calls:
        old = expected_by_key[key(call)]
        reviewed = old.get("origin") == "reviewed_addition"
        if additions is not None:
            call["origin"] = old["origin"]
        if reviewed:
            call["reviewed_builder_line"] = old["builder_line"]
            call["reviewed_operation_line"] = old["operation_line"]
            call["reviewed_send_line"] = old["send_line"]
        else:
            call["historical_operation_line"] = old["operation_line"]
            call["historical_send_line"] = old["send_line"]
            call["positions_changed"] = ((call["operation_line"], call["send_line"])
                                         != (old["operation_line"], old["send_line"]))
    provider_sites = []
    for path, source in sources.items():
        masked = mask_noncode(source)
        for match in re.finditer(r"\bAssumeRoleProvider\s*::\s*builder\s*\(", masked):
            function, _ = function_at(masked, match.start())
            provider_sites.append({"path": path, "function": function,
                                   "provider_builder_line": source.count("\n", 0, match.start()) + 1})
    providers = matrix["credential_provider_boundaries"]
    if len(providers) != 1 or providers[0]["id"] != "sts.assume_role" or providers[0]["explicit_sdk_send"] is not False:
        raise ValueError("unexpected historical credential-provider boundaries")
    old_sites = providers[0]["source_calls"]
    if Counter((row["path"], row["function"]) for row in provider_sites) != Counter((row["path"], row["function"]) for row in old_sites):
        raise ValueError("credential-provider boundary changed")
    modules, provider_dependency = dependency_evidence(root, matrix)
    addition_modules = addition_dependency_evidence(root, additions)
    catalog, catalog_sha, parents, parent_sha, merged = catalogs(root)
    descriptors = merged["resource_types"]
    if not descriptors or len(descriptors) > MAX_CALLS or len({row["type_name"] for row in descriptors}) != len(descriptors):
        raise ValueError("invalid Cloud Control descriptor set")
    old_hashes = matrix["extraction"]["source_sha256"]
    previous_hashes = previous["all_current_aws_source_sha256"]
    source_rows = [{"path": path, "sha256": hashlib.sha256(source.encode()).hexdigest(),
                    "historical_sha256": old_hashes.get(path), "v04_sha256": previous_hashes.get(path)}
                   for path, source in sorted(sources.items())]
    for row in source_rows:
        row["changed_since_historical_matrix"] = row["historical_sha256"] != row["sha256"]
        row["changed_since_v04_reconciliation"] = row["v04_sha256"] != row["sha256"]
    report = {
        "schema_version": 1, "observed_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "status": "source_boundaries_reconciled", "sdk_operation_boundaries_added": 0,
        "sdk_operation_boundaries_removed": 0,
        "historical_matrix": {"path": str(matrix_path.relative_to(root)), "sha256": matrix_sha, "left_unchanged": True},
        "historical_reconciliation": {"path": str(previous_path.relative_to(root)), "sha256": previous_sha, "left_unchanged": True},
        "summary": {"explicit_unique_operations": len(operations), "explicit_sdk_send_callsites": len(calls),
                    "sdk_services": len({row["sdk_service"] for row in calls}),
                    "credential_provider_operation_boundaries": len(provider_sites),
                    "callsite_positions_changed": sum(row.get("positions_changed") is True for row in calls),
                    "nonempty_channel_send_sites_excluded": nonempty,
                    "aws_source_files": len(source_rows)},
        "method": "Mask comments/string/character literals; enumerate every empty send under crates/aws/src; resolve each direct receiver/method chain and SDK client namespace; require the exact historical path/function/operation multiset. Verify pinned operation/provider module hashes against Cargo.lock and the local registry source.",
        "callsites": sorted(calls, key=lambda row: (row["path"], row["send_line"])),
        "credential_provider": {"id": "sts.assume_role", "explicit_sdk_send": False,
                                "source_calls": provider_sites, "pinned_dependency": provider_dependency},
        "pinned_sdk_operation_modules": modules, "aws_source_files": source_rows,
        "source": before,
        "cloud_control_provider_dimension": {
            "historical_type_count": matrix["cloud_control_provider_dimension"]["resource_type_count"],
            "current_type_count": len(descriptors), "catalog_sha256": catalog_sha,
            "root_type_count": len(catalog["resource_types"]),
            "parent_dependent_type_count": len(parents["resource_types"]),
            "parent_catalog_sha256": parent_sha,
            "runtime_catalog_canonical_sha256": canonical_sha256(merged),
            "relationship_rule_count": sum(len(row.get("relationships", [])) for row in descriptors),
            "parent_relationship_rule_count": len(parents["parent_plans"]),
            "operation_ids": ["cloudcontrol.list_resources", "cloudcontrol.get_resource"],
            "meaning": "Descriptor/relationship breadth is separate from SDK operation boundaries and executed provider coverage.",
        },
        "emulator_evidence": {"status": "pending", "current_run_bound": False,
                              "meaning": "No endpoint or provider execution result is claimed by this source-only receipt."},
        "limits": ["Lexical audit, not a Rust AST/type proof or branch coverage measurement.",
                   "Unrecognized call shapes and changed boundaries fail closed; new operations require an updated reviewed matrix.",
                   "No live AWS calls, Cargo compilation, mutation campaign, or throughput measurement was performed.",
                   "Default credential-chain network providers remain outside the explicit collector operation count."],
    }
    if additions is not None:
        add_operations = [{"id": operation["id"], "sdk_crate": operation["sdk_crate"],
                           "sdk_version": operation["sdk_version"],
                           "api_operation": operation["api_operation"],
                           "status": operation["coverage"]["status"],
                           "execution_status": "unexercised",
                           "callsites": len(operation["source_calls"])}
                          for operation in additions["additions"]]
        report["reviewed_additions"] = {
            "path": additions["path"], "sha256": additions["sha256"],
            "bound_historical_matrix_sha256": matrix_sha,
            "operation_boundaries": len(additions["additions"]),
            "callsites": len(addition_expected(additions)),
            "execution_status": "unexercised",
            "operations": add_operations,
            "pinned_sdk_operation_modules": addition_modules,
        }
        report["summary"]["reviewed_additive_operations"] = len(additions["additions"])
        report["summary"]["reviewed_additive_callsites"] = len(addition_expected(additions))
        report["method"] += (" Reviewed additive callsites come only from "
                             f"{ADDITIONS_NAME}, are verified against pinned SDK operation "
                             "modules, and are never emulator outcomes.")
    return report


def validate_summary(report, summary, catalog, matrix):
    checks = summary.get("checks", [])
    required = {"provenance:embedded_catalog_matches", "provenance:source_unchanged",
                "provenance:binary_unchanged", "provenance:fixture_unchanged",
                "coverage:every_operation_attempted", "coverage:no_unexplained_operation_failures"}
    if (summary.get("schema_version") != 1 or summary.get("failed_checks") != 0
            or not checks or any(row.get("passed") is not True for row in checks)
            or summary.get("passed_checks") != len(checks)
            or not required <= {row.get("check") for row in checks}):
        raise ValueError("emulator binding requires a complete asserted run")
    executed = summary.get("execution") or {}
    if (executed.get("source") != report["source"]
            or executed.get("catalog_sha256") != report["cloud_control_provider_dimension"]["catalog_sha256"]
            or executed.get("catalog_canonical_sha256") != canonical_sha256(catalog)):
        raise ValueError("emulator evidence belongs to a different source or catalog")
    dimension = report["cloud_control_provider_dimension"]
    if dimension.get("parent_catalog_sha256") is not None:
        if (executed.get("parent_catalog_sha256") != dimension["parent_catalog_sha256"]
                or executed.get("runtime_catalog_canonical_sha256")
                != dimension["runtime_catalog_canonical_sha256"]):
            raise ValueError("emulator evidence belongs to a different parent or merged runtime catalog")
    expected = {row["id"]: row for row in matrix["operations"] + matrix["credential_provider_boundaries"]}
    rows = summary.get("expected_operations", [])
    if len(rows) != len(expected) or {row["id"] for row in rows} != set(expected):
        raise ValueError("emulator operation identity set differs from reconciled boundaries")
    for row in rows:
        old = expected[row["id"]]
        if (row.get("service") != old["boto3_service"] or row.get("operation") != old["api_operation"]
                or row.get("credential_provider_boundary") is not (not old["explicit_sdk_send"])):
            raise ValueError("emulator operation identity/provider classification differs")
    return executed, rows


def request_evidence(path, rows):
    expected = {(row["service"], row["operation"]): row for row in rows}
    counts, phases = {}, {}
    size, records = 0, 0
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while True:
            raw = stream.readline(65_537)
            if not raw:
                break
            size += len(raw)
            records += 1
            if len(raw) > 65_536 or size > MAX_SOURCE_BYTES or records > MAX_REQUESTS:
                raise ValueError("request-ledger audit bound exceeded")
            digest.update(raw)
            row = json.loads(raw)
            if not str(row.get("phase", "")).startswith(("rust:", "rust-probe:")):
                continue
            identity = (row.get("service"), row.get("operation"))
            if identity not in expected:
                raise ValueError("unexpected Rust operation in the request ledger")
            counts.setdefault(identity, Counter())[str(row.get("http_status"))] += 1
            phases.setdefault(identity, set()).add(row["phase"])
    for identity, row in expected.items():
        histogram = counts.get(identity, Counter())
        if (type(row.get("attempts")) is not int or row["attempts"] != sum(histogram.values())
                or row.get("http_statuses") != dict(histogram)
                or row.get("phases") != sorted(phases.get(identity, set()))):
            raise ValueError("operation summary disagrees with the Rust request ledger")
    return {"sha256": digest.hexdigest(), "records": records, "bytes": size}


def bind_emulator(report, root, summary_path):
    summary_path = summary_path.resolve()
    summary_path.relative_to(root)
    summary, summary_sha = read_json(summary_path)
    matrix, _ = read_json(root / "catalog/aws-operations.json")
    catalog, _catalog_sha, _parents, _parent_sha, merged = catalogs(root)
    executed, rows = validate_summary(report, summary, catalog, matrix)
    binary = executed.get("binary", {})
    if not Path(binary.get("path", "")).is_file() or file_sha256(Path(binary["path"])) != binary.get("sha256"):
        raise ValueError("executed collector binary is absent or changed")
    fixture = root / "crates/aws/tests/fixtures/cloudcontrol-architecture.json"
    if file_sha256(fixture) != executed.get("architecture_fixture_sha256"):
        raise ValueError("architecture fixture changed after emulator execution")
    artifacts = {str(summary_path.relative_to(root)): summary_sha}
    command_records = {}
    for key_name, filename in [("catalog_command", "catalog.json"), ("cloud_control_command", "cloud-control.json")]:
        path = summary_path.parent / filename
        command, digest = read_json(path)
        binding = executed.get(key_name, {})
        if (binding.get("file") != filename or binding.get("sha256") != digest
                or command.get("binary_sha256") != binary["sha256"]):
            raise ValueError("emulator command receipt is not bound to this run/binary")
        artifacts[str(path.relative_to(root))] = digest
        command_records[key_name] = command
    if (command_records["catalog_command"].get("exit_code") != 0
            or command_records["catalog_command"].get("result", {}).get("cloud_control") != merged):
        raise ValueError("executed binary catalog differs from current descriptor data")
    command = command_records["cloud_control_command"]
    scans = command.get("result", {}).get("scans", [])
    if (command.get("exit_code") not in (0, 2) or len(scans) != 1
            or scans[0].get("scan_id") != executed["cloud_control_command"].get("scan_id")
            or executed["cloud_control_command"].get("binary_sha256") != binary["sha256"]):
        raise ValueError("Cloud Control command/scan binding differs")
    ledger = summary_path.parent / "requests.jsonl"
    ledger_evidence = request_evidence(ledger, rows)
    artifacts[str(ledger.relative_to(root))] = ledger_evidence["sha256"]
    return {"status": "current_execution_receipts_bound", "current_run_bound": True,
            "artifact_sha256": artifacts, "execution": executed,
            "checks_passed": summary["passed_checks"], "checks_failed": summary["failed_checks"],
            "operation_results": rows, "operation_outcomes": dict(sorted(Counter(row["status"] for row in rows).items())),
            "request_ledger": ledger_evidence,
            "meaning": "Recorded statuses are retained without promoting probe-only, unsupported, decode-failure or HTTP-only evidence. Per-type typed read coverage belongs to the separately asserted capability matrix; no live AWS claim."}


def output_path(root, requested):
    output = requested.resolve()
    output.relative_to(root)
    protected = {
        root / "catalog/aws-operations.json",
        root / ADDITIONS_NAME,
        root / "artifacts/aws-operation-source-reconciliation-v04.json",
        root / "artifacts/aws-operation-source-reconciliation-v05.json",
    }
    if output in protected:
        raise ValueError("historical evidence cannot be replaced")
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/aws-operation-source-reconciliation-v06.json")
    parser.add_argument("--emulator-summary", type=Path)
    parser.add_argument("--check-additions", action="store_true")
    args = parser.parse_args()
    if args.check_additions:
        print(json.dumps(check_additions(ROOT), sort_keys=True))
        return
    report = reconcile(ROOT)
    if args.emulator_summary:
        report["emulator_evidence"] = bind_emulator(report, ROOT, args.emulator_summary)
    report["auditor"] = {"path": str(Path(__file__).relative_to(ROOT)), "sha256": file_sha256(Path(__file__)),
                         "fixture_provenance_sha256": file_sha256(ROOT / "scripts/emulator/fixture_provenance.py")}
    # Re-read source closure after the audit to reject concurrent authoring edits.
    if report["source"] != source_fingerprint(ROOT):
        raise ValueError("source changed while operation reconciliation was running")
    output = output_path(ROOT, args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_name(output.name + ".partial")
    with temporary.open("x") as stream:
        stream.write(json.dumps(report, indent=2, sort_keys=True) + "\n")
    temporary.replace(output)
    print(json.dumps({"output": str(output.relative_to(ROOT)), "status": report["status"], **report["summary"]}, sort_keys=True))


if __name__ == "__main__":
    main()
