"""Bounded file fingerprints shared by fixture execution and evidence refresh."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path

MAX_SOURCE_FILES = 4_096
MAX_SOURCE_BYTES = 32 * 1024 * 1024


def file_sha256(path: Path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def canonical_sha256(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def expected_runtime_catalog(root_catalog, parent_catalog):
    """Build the exact additive parent overlay without changing frozen inputs."""
    def named_rows(value, field):
        rows = value.get(field)
        if not isinstance(rows, list) or not rows or len(rows) > 512:
            raise ValueError(f"invalid runtime catalog {field} bounds")
        if any(not isinstance(row, dict) or not isinstance(row.get("type_name"), str)
               or not row["type_name"] for row in rows):
            raise ValueError(f"invalid runtime catalog {field} identity")
        result = {row["type_name"]: row for row in rows}
        if len(result) != len(rows):
            raise ValueError(f"duplicate runtime catalog {field} identity")
        return rows, result

    roots, root_names = named_rows(root_catalog, "resource_types")
    children, child_names = named_rows(parent_catalog, "resource_types")
    plans, by_name = named_rows(parent_catalog, "parent_plans")
    if len(roots) + len(children) > 512 or len(plans) != len(children):
        raise ValueError("invalid root/parent runtime catalog bounds")
    if root_names.keys() & child_names.keys() or by_name.keys() != child_names.keys():
        raise ValueError("overlapping or missing parent workflow identity")
    if (any(plan.get("parent_type") not in root_names
            or root_names[plan["parent_type"]].get("list_requires_model") is not False for plan in plans)
            or any(row.get("list_requires_model") is not True for row in children)):
        raise ValueError("parent runtime catalog requires reviewed root prerequisites")
    if not isinstance(parent_catalog.get("evidence"), dict):
        raise ValueError("parent runtime catalog evidence missing")
    merged = copy.deepcopy(root_catalog)
    merged["resource_types"].extend(
        {**copy.deepcopy(row), "parent_workflow": copy.deepcopy(by_name[row["type_name"]])}
        for row in children)
    merged["parent_evidence"] = copy.deepcopy(parent_catalog["evidence"])
    return merged


def source_fingerprint(root: Path):
    files = set((root / "src").rglob("*.rs"))
    for crate in (root / "crates").iterdir():
        if crate.is_dir():
            files.update((crate / "src").rglob("*.rs"))
            if (crate / "Cargo.toml").is_file():
                files.add(crate / "Cargo.toml")
    files.update(root / name for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"))
    # Runtime include_str!/include_bytes! inputs are executable semantics too.
    # The capability matrix is a reporting-only evidence attachment and remains
    # separately bound, avoiding a self-referential post-run evidence digest.
    files.update((root / "crates/engine/sql").glob("*.sql"))
    files.update(root / name for name in (
        ".cargo/config.toml", "catalog/cloudcontrol-resources.json",
        "catalog/cloudcontrol-parent-resources.json", "catalog/cloudcontrol-v04-types.json",
        "crates/sdk-catalog/sdk-services.json",
    ) if (root / name).is_file())
    if not files or len(files) > MAX_SOURCE_FILES:
        raise ValueError("fixture source fingerprint file budget exceeded")
    size, rows = 0, []
    for path in sorted(files):
        size += path.stat().st_size
        if size > MAX_SOURCE_BYTES:
            raise ValueError("fixture source fingerprint byte budget exceeded")
        rows.append({"file": str(path.relative_to(root)), "sha256": file_sha256(path)})
    return {"sha256": canonical_sha256(rows), "files": rows}
