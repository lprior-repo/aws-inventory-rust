#!/usr/bin/env python3
"""Reproduce the pinned native Redis/FalkorDB fixture without a system install."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
import platform
from pathlib import Path, PurePosixPath
import posixpath
import shutil
import signal
import stat
import subprocess
import tarfile
import tempfile
import urllib.request

REDIS_VERSION = "8.10.3"
REDIS_URL = f"https://download.redis.io/releases/redis-{REDIS_VERSION}.tar.gz"
REDIS_SHA256 = "edde778f6bdee7581e42fc139ca9e07e2822212092f7501f99989656e3629d02"
FALKOR_VERSION = "6.0.4"
FALKOR_URL = (
    f"https://github.com/FalkorDB/FalkorDB/releases/download/v{FALKOR_VERSION}/falkordb-x64.so"
)
FALKOR_SHA256 = "0e71f1b4add13ab1831dc68dc03fdb715e0e813e23216163cc5ab37b0ae3f1f2"
MAX_DOWNLOAD_BYTES = 128 * 1024 * 1024
MAX_EXTRACT_BYTES = 256 * 1024 * 1024
MAX_ARCHIVE_MEMBERS = 20_000
# Include a bounded allowance for tar headers, padding and extended metadata.
MAX_TAR_BYTES = MAX_EXTRACT_BYTES + MAX_ARCHIVE_MEMBERS * 1024 + 10_240
REDIS_ROOT = f"redis-{REDIS_VERSION}"
EXPECTED_SYMLINKS = {
    f"{REDIS_ROOT}/modules/redisearch/src/.claude/skills": "../.skills",
    f"{REDIS_ROOT}/modules/redisearch/src/CLAUDE.md": "AGENTS.md",
}


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def regular_file(path: Path, maximum: int) -> bool:
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return False
    if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > maximum:
        raise ValueError(f"fixture path is not a bounded regular file: {path.name}")
    return True


def pinned_download(path: Path, expected: str) -> bool:
    if not regular_file(path, MAX_DOWNLOAD_BYTES):
        return False
    if sha256(path) != expected:
        raise ValueError(f"existing pinned download has a different digest: {path.name}")
    return True


def download(url: str, destination: Path, expected: str) -> None:
    if pinned_download(destination, expected):
        return
    request = urllib.request.Request(url, headers={"User-Agent": "aws-inventory-fixtures"})
    owned = tempfile.NamedTemporaryFile(mode="wb", prefix=destination.name + ".downloading-",
                                       dir=destination.parent, delete=False)
    temporary = Path(owned.name)
    try:
        with owned as output, urllib.request.urlopen(request, timeout=180) as response:
            total = 0
            for _ in range(MAX_DOWNLOAD_BYTES // (1024 * 1024) + 1):
                chunk = response.read(1024 * 1024)
                if not chunk:
                    break
                total += len(chunk)
                if total > MAX_DOWNLOAD_BYTES:
                    raise ValueError("fixture download exceeds byte bound")
                output.write(chunk)
        if sha256(temporary) != expected:
            raise ValueError(f"pinned download digest mismatch: {destination.name}")
        try:
            # Atomic no-clobber publication. A concurrent matching installer may
            # win; a different existing file must never be overwritten.
            os.link(temporary, destination)
        except FileExistsError:
            if not pinned_download(destination, expected):
                raise ValueError("concurrent pinned download disappeared")
    finally:
        # This unique name was created by this invocation. Never delete a
        # pre-existing '.downloading' file owned by somebody else.
        temporary.unlink(missing_ok=True)


class BoundedTarReader:
    """Cap decompressed bytes before tarfile parses extended headers."""

    def __init__(self, stream):
        self.stream = stream
        self.total = 0

    def read(self, size):
        if type(size) is not int or size < 0:
            raise ValueError("unbounded tar read requested")
        value = self.stream.read(min(size, MAX_TAR_BYTES - self.total + 1))
        self.total += len(value)
        if self.total > MAX_TAR_BYTES:
            raise ValueError("Redis archive exceeds decompressed byte bound")
        return value


def check_member(member, seen, total):
    path = PurePosixPath(member.name)
    if (path.is_absolute() or ".." in path.parts or not path.parts
            or path.parts[0] != REDIS_ROOT or str(path) in seen
            or any(str(parent) in EXPECTED_SYMLINKS for parent in path.parents)):
        raise ValueError("unexpected Redis archive member")
    if member.issym():
        target = PurePosixPath(posixpath.normpath(posixpath.join(str(path.parent), member.linkname)))
        if (EXPECTED_SYMLINKS.get(str(path)) != member.linkname or target.is_absolute()
                or not target.parts or target.parts[0] != REDIS_ROOT):
            raise ValueError("unexpected Redis archive link")
    elif not (member.isfile() or member.isdir()):
        raise ValueError("unexpected Redis archive member type")
    if member.size < 0 or total + member.size > MAX_EXTRACT_BYTES:
        raise ValueError("Redis archive exceeds extraction byte bound")
    seen.add(str(path))
    return total + member.size


def extract_redis(archive: Path, destination: Path) -> Path:
    source = destination / REDIS_ROOT
    try:
        source.mkdir()
    except FileExistsError as error:
        # Covers existing directories, files and dangling symlinks alike.
        raise ValueError("Redis source destination already exists; use a fresh directory") from error
    try:
        # Streaming avoids getmembers() allocating unbounded metadata before a
        # count check. The gzip wrapper also bounds PAX headers before parsing.
        with gzip.open(archive, "rb") as decompressed:
            with tarfile.open(fileobj=BoundedTarReader(decompressed), mode="r|") as bundle:
                seen = set()
                total = 0
                for index, member in enumerate(bundle):
                    if index >= MAX_ARCHIVE_MEMBERS:
                        raise ValueError("Redis archive exceeds extraction member bound")
                    total = check_member(member, seen, total)
                    bundle.extract(member, destination, filter="data")
        if not seen:
            raise ValueError("Redis archive is empty")
    except BaseException as primary:
        try:
            shutil.rmtree(source)
        except OSError as cleanup:
            raise RuntimeError(f"Redis extraction failed and owned source cleanup failed: {cleanup}") from primary
        raise
    return source


def cached_installation(destination: Path) -> dict | None:
    receipt_path = destination / "installation.json"
    if not regular_file(receipt_path, 65_536):
        return None
    receipt = json.loads(receipt_path.read_text())
    source = destination / REDIS_ROOT
    expected = {"schema_version": 1, "redis_version": REDIS_VERSION,
                "falkordb_version": FALKOR_VERSION, "redis_url": REDIS_URL,
                "redis_archive_sha256": REDIS_SHA256, "module_url": FALKOR_URL,
                "module_sha256": FALKOR_SHA256,
                "redis_server": str(source / "src/redis-server"),
                "redis_cli": str(source / "src/redis-cli"),
                "falkor_module": str(destination / "falkordb.so")}
    if not isinstance(receipt, dict) or any(receipt.get(key) != value for key, value in expected.items()):
        raise ValueError("existing installation receipt differs from the requested pins or directory")
    for path, digest in ((destination / f"{REDIS_ROOT}.tar.gz", REDIS_SHA256),
                         (destination / "falkordb.so", FALKOR_SHA256)):
        if not pinned_download(path, digest):
            raise ValueError("existing installation is missing a pinned download")
    for key in ("redis_server", "redis_cli"):
        binary = Path(receipt[key])
        if (not binary.resolve().is_relative_to(destination)
                or not regular_file(binary, MAX_DOWNLOAD_BYTES)
                or sha256(binary) != receipt.get(key + "_sha256")):
            raise ValueError("existing installation binary differs from its receipt")
    return receipt


def compile_redis(command: list[str]) -> None:
    # make owns compiler children. Killing only make on timeout can leave those
    # children running after the installer reports failure.
    process = subprocess.Popen(command, start_new_session=True)
    try:
        code = process.wait(timeout=1_800)
    except BaseException as primary:
        try:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass  # The complete group already exited; still reap make.
            process.wait(timeout=10)
        except Exception as cleanup:
            raise RuntimeError(f"Redis compilation failed and build cleanup failed: {cleanup}") from primary
        raise
    if code != 0:
        raise subprocess.CalledProcessError(code, command)


def install(destination: Path, jobs: int) -> dict:
    if platform.system() != "Linux" or platform.machine() not in ("x86_64", "amd64"):
        raise ValueError("the pinned FalkorDB module requires Linux x86_64")
    if type(jobs) is not int or not 1 <= jobs <= 8:
        raise ValueError("build jobs must be 1..8")
    destination = destination.resolve()
    destination.mkdir(parents=True, exist_ok=True)
    cached = cached_installation(destination)
    if cached is not None:
        # Redis 8 requires execute permission even though dlopen reads a module.
        # cached_installation has already verified this exact file and digest.
        for executable in (destination / "falkordb.so",
                           destination / REDIS_ROOT / "src/redis-server",
                           destination / REDIS_ROOT / "src/redis-cli"):
            executable.chmod(0o755)
        return cached
    if os.path.lexists(destination / REDIS_ROOT):
        raise ValueError("Redis source destination already exists; use a fresh directory")
    archive = destination / f"redis-{REDIS_VERSION}.tar.gz"
    module = destination / "falkordb.so"
    download(REDIS_URL, archive, REDIS_SHA256)
    download(FALKOR_URL, module, FALKOR_SHA256)
    module.chmod(0o755)
    source = extract_redis(archive, destination)
    command = ["make", "-C", str(source), f"-j{jobs}", "MALLOC=libc", "OPT=-O2",
               "redis-server", "redis-cli"]
    compile_redis(command)
    server, client = source / "src/redis-server", source / "src/redis-cli"
    versions = {name: subprocess.check_output([str(binary), "--version"], text=True, timeout=10).strip()
                for name, binary in (("server", server), ("client", client))}
    if f"v={REDIS_VERSION} " not in versions["server"] or versions["client"] != f"redis-cli {REDIS_VERSION}":
        raise ValueError("compiled Redis version differs from the pin")
    receipt = {
        "schema_version": 1, "redis_version": REDIS_VERSION, "falkordb_version": FALKOR_VERSION,
        "redis_url": REDIS_URL, "redis_archive_sha256": sha256(archive),
        "redis_checksum_source": "https://github.com/redis/redis-hashes/blob/master/README",
        "module_url": FALKOR_URL, "module_sha256": sha256(module),
        "module_checksum_source": f"https://api.github.com/repos/FalkorDB/FalkorDB/releases/tags/v{FALKOR_VERSION}",
        "build_command": command, "versions": versions,
        "redis_server": str(server), "redis_cli": str(client), "falkor_module": str(module),
        "redis_server_sha256": sha256(server), "redis_cli_sha256": sha256(client),
        "graph_runtime_tested": False,
    }
    with (destination / "installation.json").open("x") as output:
        output.write(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    return receipt


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--jobs", type=int, default=2)
    arguments = parser.parse_args()
    print(json.dumps(install(arguments.destination.resolve(), arguments.jobs), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
