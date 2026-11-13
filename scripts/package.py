#!/usr/bin/env python3
"""Package source and reviewable evidence, excluding toolchains and local credentials."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
EXCLUDED_DIRECTORIES = {"target", ".git", "__pycache__", ".pytest_cache", "mutants.out.old"}
EXCLUDED_ARTIFACT_TREES = {
    "emulator-seed-preflight", "emulator-v04-first", "emulator-v04-second", "emulator-v04-expanded",
}


def included(path: Path) -> bool:
    relative = path.relative_to(ROOT)
    if any(part in EXCLUDED_DIRECTORIES for part in relative.parts) or path.is_symlink():
        return False
    if path.name == "MANIFEST.sha256" or path.suffix in {".pyc", ".deb", ".so"}:
        return False
    if relative.parts[0] == "artifacts" and len(relative.parts) > 1:
        if relative.parts[1] in EXCLUDED_ARTIFACT_TREES or relative.parts[1].startswith("emulator-oracle-"):
            return False
    if path.name.endswith((".sqlite", ".sqlite-wal", ".sqlite-shm", ".db", ".db-wal", ".db-shm")):
        return relative.as_posix() in {
            "artifacts/emulator-v04/inventory.sqlite",
            "artifacts/emulator-v05/inventory.sqlite",
            "artifacts/emulator-v06/inventory.sqlite",
            "artifacts/emulator-v06-verified/inventory.sqlite",
            "artifacts/emulator-v06-final/inventory.sqlite",
            "artifacts/emulator-v06-complete/inventory.sqlite",
        }
    if path.name in {"credentials", "emulator_credentials", "ca.pem", "server.key", "leaf.key"}:
        return False
    return True


def package(destination: Path) -> dict:
    files = sorted(path for path in ROOT.rglob("*") if path.is_file() and included(path))
    payloads = []
    for path in files:
        before = path.stat()
        data = path.read_bytes()
        after = path.stat()
        if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
            raise RuntimeError(f"file changed during packaging: {path.relative_to(ROOT)}")
        payloads.append((path.relative_to(ROOT).as_posix(), data))
    manifest = "".join(f"{hashlib.sha256(data).hexdigest()}  {name}\n" for name, data in payloads)
    (ROOT / "MANIFEST.sha256").write_text(manifest)
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_suffix(".building.zip")
    with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=7) as archive:
        for name, data in payloads:
            archive.writestr("aws-inventory-rs/" + name, data)
        archive.writestr("aws-inventory-rs/MANIFEST.sha256", manifest)
    with zipfile.ZipFile(temporary) as archive:
        if archive.testzip() is not None:
            raise RuntimeError("ZIP integrity check failed")
        for name, data in payloads:
            if hashlib.sha256(archive.read("aws-inventory-rs/" + name)).digest() != hashlib.sha256(data).digest():
                raise RuntimeError(f"packaged content hash mismatch: {name}")
    temporary.replace(destination)
    with destination.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"path": str(destination.resolve()), "files": len(payloads) + 1,
            "bytes": destination.stat().st_size, "sha256": digest}


if __name__ == "__main__":
    import json
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(package(args.output.resolve()), indent=2, sort_keys=True))
