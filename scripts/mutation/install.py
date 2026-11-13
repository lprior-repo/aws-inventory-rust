#!/usr/bin/env python3
"""Install the pinned official Linux cargo-mutants release without a Cargo build."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import urllib.request

VERSION = "27.1.0"
URL = (
    "https://github.com/sourcefrog/cargo-mutants/releases/download/v27.1.0/"
    "cargo-mutants-x86_64-unknown-linux-gnu.tar.gz"
)
# Published by the official GitHub releases API for this exact release asset.
ARCHIVE_SHA256 = "dfe6dc37d0342c891d2829b5a695aa57c2d0edecef7e7d0399a30cc6e206411e"
MAX_ARCHIVE_BYTES = 16 * 1024 * 1024
MAX_BINARY_BYTES = 32 * 1024 * 1024


def install(destination: Path) -> dict:
    request = urllib.request.Request(URL, headers={"User-Agent": "aws-inventory-mutation-gate"})
    with urllib.request.urlopen(request, timeout=45) as response:
        archive = response.read(MAX_ARCHIVE_BYTES + 1)
    if len(archive) > MAX_ARCHIVE_BYTES:
        raise ValueError("release asset exceeds the download bound")
    actual = hashlib.sha256(archive).hexdigest()
    if actual != ARCHIVE_SHA256:
        raise ValueError("official release asset SHA-256 mismatch")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as bundle:
        members = [
            member for member in bundle.getmembers()
            if member.isfile() and Path(member.name).name == "cargo-mutants"
        ]
        if len(members) != 1:
            raise ValueError("release asset does not contain exactly one regular executable")
        stream = bundle.extractfile(members[0])
        if stream is None:
            raise ValueError("release executable cannot be read")
        with stream:
            binary = stream.read(MAX_BINARY_BYTES + 1)
    if len(binary) > MAX_BINARY_BYTES:
        raise ValueError("release executable exceeds the extraction bound")
    executable = destination / "bin" / "cargo-mutants"
    executable.parent.mkdir(parents=True, exist_ok=True)
    temporary = executable.with_suffix(".installing")
    temporary.write_bytes(binary)
    temporary.chmod(0o755)
    version = subprocess.check_output(
        [str(temporary), "mutants", "--version"], text=True, timeout=10
    ).strip()
    if version != f"cargo-mutants {VERSION}":
        raise ValueError(f"unexpected executable version: {version}")
    temporary.replace(executable)
    receipt = {
        "package": "cargo-mutants",
        "version": VERSION,
        "version_output": version,
        "installed_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "release_url": f"https://github.com/sourcefrog/cargo-mutants/releases/tag/v{VERSION}",
        "asset_url": URL,
        "asset_sha256": actual,
        "asset_bytes": len(archive),
        "binary": str(executable.resolve()),
        "binary_sha256": hashlib.sha256(binary).hexdigest(),
        "binary_bytes": len(binary),
    }
    (destination / f"cargo-mutants-{VERSION}-install.json").write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    )
    return receipt


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, default=Path("/tmp/aws-inventory-test-tools"))
    args = parser.parse_args()
    try:
        print(json.dumps(install(args.destination.resolve()), indent=2, sort_keys=True))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, f"installation failed: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
