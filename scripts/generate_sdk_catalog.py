#!/usr/bin/env python3
"""Refresh official AWS SDK pins or reproduce the catalog offline.

Network refresh uses AWS's own versions.toml and crates.io's sparse index.
No AWS credentials are loaded and no AWS account API is called. The checked-in
JSON snapshot and original versions.toml are sufficient for offline generation.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
import json
import os
import pathlib
import re
import sys
import time
import tomllib
import urllib.error
import urllib.request
from html.parser import HTMLParser

ROOT = pathlib.Path(__file__).resolve().parents[1]
DESTINATION = ROOT / "crates" / "sdk-catalog"
SNAPSHOT = DESTINATION / "sdk-services.json"
UPSTREAM = DESTINATION / "upstream-versions.toml"
REPOSITORY = "https://github.com/awslabs/aws-sdk-rust"
CATALOG_URL = "https://awslabs.github.io/aws-sdk-rust/"
INDEX_ORIGIN = "https://index.crates.io/"
MAX_SERVICES = 1_024
MAX_RESPONSE_BYTES = 8_000_000
MAX_ATTEMPTS = 3
MAX_WORKERS = 8
NAME_PATTERN = re.compile(r"aws-sdk-([a-z][a-z0-9]*)\Z")
VERSION_PATTERN = re.compile(r"([0-9]+)\.([0-9]+)\.([0-9]+)\Z")
HEX_PATTERN = re.compile(r"[0-9a-f]{64}\Z")


def fetch(url: str) -> bytes:
    last_error = "request not attempted"
    for attempt in range(MAX_ATTEMPTS):
        try:
            request = urllib.request.Request(
                url, headers={"User-Agent": "aws-inventory-sdk-catalog/0.1"}
            )
            with urllib.request.urlopen(request, timeout=30) as response:
                payload = response.read(MAX_RESPONSE_BYTES + 1)
            if len(payload) > MAX_RESPONSE_BYTES:
                raise ValueError(f"response exceeds byte limit: {url}")
            return payload
        except (urllib.error.URLError, TimeoutError) as error:
            last_error = str(error)
            if attempt + 1 < MAX_ATTEMPTS:
                time.sleep(0.25 * (2**attempt))
    raise RuntimeError(f"request failed after {MAX_ATTEMPTS} attempts: {url}: {last_error}")


class CatalogLinks(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.names: set[str] = set()

    def handle_starttag(self, tag: str, attributes: list[tuple[str, str | None]]) -> None:
        if tag != "a":
            return
        for name, value in attributes:
            if name == "href" and value:
                prefix = "https://crates.io/crates/"
                if value.startswith(prefix):
                    crate = value.removeprefix(prefix).rstrip("/")
                    if NAME_PATTERN.fullmatch(crate):
                        self.names.add(crate)


def version_key(version: str) -> tuple[int, int, int] | None:
    match = VERSION_PATTERN.fullmatch(version)
    if match is None:
        return None
    major, minor, patch = match.groups()
    return int(major), int(minor), int(patch)


def registry_record(name: str, upstream: dict[str, str], revision: str) -> dict[str, object]:
    version = upstream["version"]
    index_url = f"{INDEX_ORIGIN}{name[:2]}/{name[2:4]}/{name}"
    records = [json.loads(line) for line in fetch(index_url).splitlines() if line]
    exact = [record for record in records if record["vers"] == version]
    if len(exact) != 1 or exact[0]["yanked"]:
        raise ValueError(f"{name} {version} is missing, duplicated, or yanked in the registry")
    published = exact[0]
    candidates = [
        (version_key(record["vers"]), record["vers"])
        for record in records
        if not record["yanked"] and version_key(record["vers"]) is not None
    ]
    latest = max(candidates)[1]
    feature = name.removeprefix("aws-sdk-")
    return {
        "name": name,
        "feature": feature,
        "version": version,
        "latest_published_version_at_check": latest,
        "origin": f"{REPOSITORY}/tree/{revision}/sdk/{feature}",
        "registry_index_url": index_url,
        "registry_package_url": f"https://crates.io/crates/{name}/{version}",
        "checksum_sha256": published["cksum"],
        "rust_version": published.get("rust_version"),
        "yanked": False,
        "sdk_source_sha256": upstream["source_hash"],
        "model_sha256": upstream.get("model_hash"),
    }


def refresh(revision: str | None) -> dict[str, object]:
    requested_revision = revision if revision is not None else "main"
    if re.fullmatch(r"[a-zA-Z0-9._-]{1,100}", requested_revision) is None:
        raise ValueError("revision must be a Git revision without path separators")
    commit = json.loads(
        fetch(f"https://api.github.com/repos/awslabs/aws-sdk-rust/commits/{requested_revision}")
    )
    revision = commit["sha"]
    if not isinstance(revision, str) or re.fullmatch(r"[0-9a-f]{40}", revision) is None:
        raise ValueError("official source did not resolve to an immutable Git commit")
    versions_url = f"https://raw.githubusercontent.com/awslabs/aws-sdk-rust/{revision}/versions.toml"
    original = fetch(versions_url)
    versions = tomllib.loads(original.decode("utf-8"))
    service_versions = {
        name: value for name, value in versions["crates"].items() if NAME_PATTERN.fullmatch(name)
    }
    if not 1 <= len(service_versions) <= MAX_SERVICES:
        raise ValueError("official service count is outside the supported hard bound")
    links = CatalogLinks()
    links.feed(fetch(CATALOG_URL).decode("utf-8"))
    missing_from_versions = sorted(links.names - service_versions.keys())
    missing_from_catalog = sorted(service_versions.keys() - links.names)
    results: list[dict[str, object]] = []
    failures: list[str] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
        pending = {
            executor.submit(registry_record, name, value, revision): name
            for name, value in sorted(service_versions.items())
        }
        for future in concurrent.futures.as_completed(pending):
            try:
                results.append(future.result())
            except (ValueError, RuntimeError, KeyError, TypeError) as error:
                failures.append(f"{pending[future]}: {error}")
            completed = len(results) + len(failures)
            if completed % 50 == 0 or completed == len(pending):
                print(f"verified registry records: {completed}/{len(pending)}", file=sys.stderr, flush=True)
    if failures:
        raise RuntimeError("catalog refresh failed:\n" + "\n".join(failures[:20]))
    results.sort(key=lambda service: service["name"])
    document: dict[str, object] = {
        "schema_version": 1,
        "checked_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "service_count": len(results),
        "official_catalog_url": CATALOG_URL,
        "official_versions_url": versions_url,
        "official_versions_sha256": hashlib.sha256(original).hexdigest(),
        "requested_revision": requested_revision,
        "official_revision": revision,
        "official_release_tag": versions.get("release", {}).get("tag"),
        "smithy_rs_revision": versions.get("smithy_rs_revision"),
        "catalog_cross_check": {
            "listed_service_count": len(links.names),
            "missing_from_versions": missing_from_versions,
            "missing_from_catalog": missing_from_catalog,
        },
        "services": results,
    }
    validate(document, original)
    DESTINATION.mkdir(parents=True, exist_ok=True)
    write(UPSTREAM, original.decode("utf-8"))
    write(SNAPSHOT, json.dumps(document, indent=2, ensure_ascii=False) + "\n")
    return document


def validate(document: dict[str, object], original: bytes) -> None:
    services = document["services"]
    if not isinstance(services, list) or not 1 <= len(services) <= MAX_SERVICES:
        raise ValueError("invalid service list")
    if document["schema_version"] != 1 or document["service_count"] != len(services):
        raise ValueError("catalog schema or count does not match")
    if hashlib.sha256(original).hexdigest() != document["official_versions_sha256"]:
        raise ValueError("upstream versions.toml checksum does not match the snapshot")
    revision = document["official_revision"]
    if not isinstance(revision, str) or re.fullmatch(r"[0-9a-f]{40}", revision) is None:
        raise ValueError("snapshot must identify the immutable official Git commit")
    expected_versions_url = (
        f"https://raw.githubusercontent.com/awslabs/aws-sdk-rust/{revision}/versions.toml"
    )
    if document["official_versions_url"] != expected_versions_url:
        raise ValueError("unexpected official version manifest origin")
    upstream = tomllib.loads(original.decode("utf-8"))["crates"]
    official_names = {name for name in upstream if NAME_PATTERN.fullmatch(name)}
    names = [service["name"] for service in services]
    if names != sorted(set(names)) or set(names) != official_names:
        raise ValueError("service names must be unique, sorted, and complete for the official source")
    for service in services:
        name = service["name"]
        if NAME_PATTERN.fullmatch(name) is None or version_key(service["version"]) is None:
            raise ValueError(f"invalid crate name or stable version: {name}")
        if service["feature"] != name.removeprefix("aws-sdk-"):
            raise ValueError(f"feature does not match crate name: {name}")
        if service["version"] != upstream[name]["version"] or service["yanked"]:
            raise ValueError(f"version does not match official source or is yanked: {name}")
        if service["sdk_source_sha256"] != upstream[name]["source_hash"]:
            raise ValueError(f"source fingerprint does not match official source: {name}")
        if service["model_sha256"] != upstream[name].get("model_hash"):
            raise ValueError(f"model fingerprint does not match official source: {name}")
        if HEX_PATTERN.fullmatch(service["checksum_sha256"]) is None:
            raise ValueError(f"invalid registry checksum: {name}")
        expected_origin = f"{REPOSITORY}/tree/{document['official_revision']}/sdk/{service['feature']}"
        if service["origin"] != expected_origin:
            raise ValueError(f"unexpected source origin: {name}")


def manifest(document: dict[str, object]) -> str:
    services = document["services"]
    lines = [
        "# Generated by scripts/generate_sdk_catalog.py; exact SDK release pins.",
        "[package]",
        'name = "inventory-sdk-catalog"',
        "version.workspace = true",
        "edition.workspace = true",
        "rust-version.workspace = true",
        "license.workspace = true",
        "publish = false",
        'description = "Pinned catalog of every official AWS service SDK; not collection coverage"',
        "",
        "[features]",
        "default = []",
        "all-services = [",
    ]
    lines.extend(f'    "{service["feature"]}",' for service in services)
    lines.append("]")
    lines.extend(f'{service["feature"]} = ["dep:{service["name"]}"]' for service in services)
    lines.extend(["", "[dependencies]"])
    lines.extend(
        f'{service["name"]} = {{ version = "={service["version"]}", optional = true }}'
        for service in services
    )
    lines.extend([
        "", "[dev-dependencies]", "serde_json.workspace = true",
        'sha2 = "0.10.9"', 'toml = "1.2.1"',
        "", "[lints]", "workspace = true", "",
    ])
    return "\n".join(lines)


def library(document: dict[str, object]) -> str:
    lines = [
        "//! Generated SDK catalog. Availability is not implemented collector coverage.",
        "//! Service crates compile only when their explicit feature is enabled.",
        "#![forbid(unsafe_code)]",
        "#![deny(unused_must_use)]",
        "",
        f'pub const SERVICE_COUNT: usize = {document["service_count"]};',
        'pub const SERVICE_MANIFEST_JSON: &str = include_str!("../sdk-services.json");',
    ]
    for service in document["services"]:
        lines.extend([
            "",
            f'#[cfg(feature = "{service["feature"]}")]',
            f'pub use {service["name"].replace("-", "_")} as {service["feature"]};',
        ])
    return "\n".join(lines) + "\n"


def write(path: pathlib.Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(text, encoding="utf-8")
    temporary.replace(path)


def verify_cache(document: dict[str, object]) -> dict[str, int]:
    cargo_home = pathlib.Path(os.environ.get("CARGO_HOME", pathlib.Path.home() / ".cargo"))
    registries = sorted((cargo_home / "registry" / "cache").glob("*"))
    if not 1 <= len(registries) <= 16:
        raise ValueError("expected 1..=16 Cargo registry cache directories")
    total_bytes = 0
    for service in document["services"]:
        archive = f'{service["name"]}-{service["version"]}.crate'
        matches = [registry / archive for registry in registries if (registry / archive).is_file()]
        if not matches:
            raise ValueError(f"SDK archive missing from Cargo cache: {archive}; run cargo fetch --locked")
        verified = False
        for path in matches:
            with path.open("rb") as reader:
                digest = hashlib.file_digest(reader, "sha256").hexdigest()
            if digest == service["checksum_sha256"]:
                total_bytes += path.stat().st_size
                verified = True
                break
        if not verified:
            raise ValueError(f"SDK archive checksum mismatch: {archive}")
    return {"verified_sdk_archives": document["service_count"], "sdk_archive_bytes": total_bytes}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refresh", action="store_true", help="fetch current official sources and registry records")
    parser.add_argument("--revision", help="pin an official Git commit or release tag during refresh")
    parser.add_argument("--check", action="store_true", help="verify generated files against the snapshot without changing them")
    parser.add_argument("--verify-cache", action="store_true", help="verify all SDK archive checksums after cargo fetch")
    args = parser.parse_args()
    if args.refresh and args.check:
        parser.error("--refresh and --check are mutually exclusive")
    if args.revision and not args.refresh:
        parser.error("--revision requires --refresh")
    document = refresh(args.revision) if args.refresh else json.loads(SNAPSHOT.read_text(encoding="utf-8"))
    validate(document, UPSTREAM.read_bytes())
    outputs = {DESTINATION / "Cargo.toml": manifest(document), DESTINATION / "src" / "lib.rs": library(document)}
    for path, text in outputs.items():
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != text:
                raise ValueError(f"generated file differs: {path.relative_to(ROOT)}")
        else:
            write(path, text)
    newer = [service["name"] for service in document["services"] if service["version"] != service["latest_published_version_at_check"]]
    cache = verify_cache(document) if args.verify_cache else {}
    print(json.dumps({"service_count": document["service_count"], "release": document["official_release_tag"], "newer_registry_versions": newer, "mode": "checked" if args.check else "generated", **cache}))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, KeyError, TypeError) as error:
        print(f"SDK catalog error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
