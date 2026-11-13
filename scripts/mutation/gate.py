#!/usr/bin/env python3
"""Freeze mutation candidates, run isolated partitions, and enforce a complete 95% gate.

Only a complete aggregate of the four production packages can pass this gate.
Package reports are useful progress evidence, never a whole-project pass.
"""

from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import datetime as dt
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import signal
import subprocess
import time

VERSION = "27.1.0"
TOOL_SHA256 = "f985f265ee3ea3e453aa98b04c52134953911f692f8ce8abf137f3873202a2d0"
DEFAULT_TOOL = Path("/tmp/aws-inventory-test-tools/bin/cargo-mutants")
PACKAGES = ("aws-inventory", "inventory-core", "inventory-engine", "inventory-aws")
PACKAGE_DIRS = {
    "aws-inventory": ".", "inventory-core": "crates/core",
    "inventory-engine": "crates/engine", "inventory-aws": "crates/aws",
    "inventory-sdk-catalog": "crates/sdk-catalog",
}
DEPENDENCIES = {
    "inventory-core": (), "inventory-engine": ("inventory-core",),
    "inventory-aws": ("inventory-engine", "inventory-core"),
    "aws-inventory": ("inventory-core", "inventory-engine", "inventory-aws", "inventory-sdk-catalog"),
}
DEPENDENCY_DATA_DIRS = ("sql", "schemas", "data")
DEPENDENCY_DATA_FILES = {"inventory-sdk-catalog": ("sdk-services.json",)}
# The aggregate, as well as the preferred launcher, owns this acceptance policy.
# An otherwise complete unit-only partition cannot bypass the live-test gate.
REQUIRED_LIVE_TESTS = {
    "aws-inventory": (
        "emulator_cli_native_snapshot_reports_and_graph_delivery",
        "emulator_cli_durable_worker_persists_an_assumed_role_scan",
    ),
    "inventory-core": (),
    "inventory-engine": (
        "live_falkordb_replay_and_parameter_round_trip",
        "live_falkordb_rejected_batch_is_never_acknowledged_or_finalized",
        "live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint",
    ),
    "inventory-aws": (
        "emulator_snapshot_native", "emulator_snapshot_sources",
        "emulator_snapshot_cloudcontrol", "emulator_snapshot_assumedrole",
        "emulator_snapshot_parents",
    ),
}
AWS_FIXTURE_ENVIRONMENT = (
    "INVENTORY_EMULATOR_TEST", "INVENTORY_EMULATOR_SNAPSHOT_PATH",
    "INVENTORY_EMULATOR_SNAPSHOT_SHA256", "INVENTORY_EMULATOR_BASELINE_SUMMARY",
    "AWS_ENDPOINT_URL", "AWS_REGION", "AWS_DEFAULT_REGION",
    "AWS_EC2_METADATA_DISABLED", "AWS_SHARED_CREDENTIALS_FILE", "AWS_CONFIG_FILE",
    "SSL_CERT_FILE",
)
REQUIRED_LIVE_ENVIRONMENT = {
    "aws-inventory": (*AWS_FIXTURE_ENVIRONMENT, "FALKORDB_TEST_URL"),
    "inventory-core": (),
    "inventory-engine": ("FALKORDB_TEST_URL",),
    "inventory-aws": AWS_FIXTURE_ENVIRONMENT,
}
ENVIRONMENT = {
    "PROPTEST_DISABLE_FAILURE_PERSISTENCE": "1",
    "PROPTEST_RNG_SEED": "137043", "CARGO_NET_OFFLINE": "true",
}
MUTANT_FIELDS = {"name", "package", "file", "function", "span", "replacement", "genre"}
IGNORED_DIRS = {"target", "artifacts", "__pycache__", ".git", "mutants.out", "mutants.out.old"}
MAX_JSON_BYTES = 64 * 1024 * 1024
MAX_SOURCE_BYTES = 64 * 1024 * 1024
MAX_LOG_BYTES = 16 * 1024 * 1024
MAX_FILES = 50_000
MAX_CANDIDATES = 100_000
MAX_RUNS = 10_000
THRESHOLD_PERCENT = 95
INFRA_FAILURE = re.compile(
    r"SIGKILL|signal: 9|No space left on device|out of memory|\bOOM(?::\s*| )command not allowed|failed to download|"
    r"failed to get .* as a dependency|failed to resolve|failed to read .*Cargo|"
    r"could not execute process|Too many open files|failed to acquire.*lock|"
    r"linking with .* failed|failed to run custom build command|"
    r"internal compiler error|the compiler unexpectedly panicked", re.I
)
COMPILER_DIAGNOSTIC = re.compile(r"^error\[E\d{4}\]:", re.M)
# Coded rustc diagnostics are not all source-invalid mutations. These identify
# incompatible/corrupt dependencies or a missing standard library/target.
COMPILER_INFRA_FAILURE = re.compile(
    r"^error\[(?:E0514|E0786)\]:|"
    r"^error\[E0463\]:\s+can(?:not|'t) find crate for [`'\"](?:std|core)[`'\"](?:\s|$)",
    re.M,
)
INTERNAL_DEADLINE = re.compile(
    r"fixture CLI exceeded its \d+-second deadline|"
    r"CLI exceeded the \d+-second test deadline|"
    r"snapshot collection exceeded its deadline|"
    r"durable (?:job|store) property exceeded its \d+-second case deadline|"
    r"engine limiter barrier test exceeded its \d+-second virtual deadline|"
    r"engine graph integration test exceeded its \d+-second deadline|"
    r"inventory export property exceeded its 10-second case deadline|"
    r"inventory export streaming test exceeded its 30-second deadline|"
    r"graph retry fixture (?:query start|shutdown) exceeded its 5-second deadline|"
    r"parent SDK wire fixture exceeded its 20-second deadline"
)


class GateError(ValueError):
    """Evidence is incomplete, inconsistent, or outside the pinned contract."""


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def canonical(value) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False).encode()


def object_no_duplicates(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise GateError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def load_json(path: Path):
    if path.stat().st_size > MAX_JSON_BYTES:
        raise GateError(f"JSON exceeds {MAX_JSON_BYTES} bytes: {path}")
    return json.loads(
        path.read_text(), object_pairs_hook=object_no_duplicates,
        parse_constant=lambda value: (_ for _ in ()).throw(GateError(f"nonfinite JSON value: {value}")),
    )


def write_json(path: Path, value, *, exclusive: bool = False) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x" if exclusive else "w") as output:
        output.write(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")


def digest(path: Path, maximum: int | None = None) -> str:
    if maximum is not None and path.stat().st_size > maximum:
        raise GateError(f"file exceeds size bound: {path}")
    result = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def relative_file(base: Path, name: str) -> Path:
    pure = PurePosixPath(name)
    if not isinstance(name, str) or pure.is_absolute() or ".." in pure.parts:
        raise GateError(f"unsafe evidence path: {name}")
    result = base / name
    if not result.resolve().is_relative_to(base.resolve()) or result.is_symlink():
        raise GateError(f"evidence path escapes its directory: {name}")
    return result


def selected_packages(packages) -> list[str]:
    selected = sorted(set(packages))
    if not selected or any(package not in PACKAGES for package in selected):
        raise GateError("select one or more of the four production packages")
    return selected


def files_under(root: Path, directory: Path) -> list[Path]:
    if not directory.exists():
        return []
    files = []
    for parent, directories, names in os.walk(directory):
        for name in directories:
            if name not in IGNORED_DIRS and (Path(parent) / name).is_symlink():
                raise GateError(f"source closure contains a directory symlink: {Path(parent) / name}")
        directories[:] = sorted(name for name in directories if name not in IGNORED_DIRS)
        for name in sorted(names):
            path = Path(parent) / name
            if path.is_symlink():
                raise GateError(f"source closure contains a symlink: {path.relative_to(root)}")
            if path.is_file():
                files.append(path)
            if len(files) > MAX_FILES:
                raise GateError("source closure exceeds file-count bound")
    return files


def closure_files(root: Path, packages) -> dict[str, str]:
    """Bind own tests, dependency production source, Cargo inputs, and shared fixtures."""
    selected = selected_packages(packages)
    paths = {
        root / name for name in (
            "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rust-toolchain",
            ".cargo/config.toml", ".cargo/config", "build.rs",
        ) if (root / name).is_file()
    }
    for package in selected:
        directory = root / PACKAGE_DIRS[package]
        if package == "aws-inventory":
            for name in ("src", "tests", "examples"):
                paths.update(files_under(root, root / name))
        else:
            paths.update(files_under(root, directory))
        for dependency in DEPENDENCIES[package]:
            dependency_dir = root / PACKAGE_DIRS[dependency]
            paths.update(files_under(root, dependency_dir / "src"))
            for name in DEPENDENCY_DATA_DIRS:
                paths.update(files_under(root, dependency_dir / name))
            for name in ("Cargo.toml", "build.rs", *DEPENDENCY_DATA_FILES.get(dependency, ())):
                if (dependency_dir / name).is_file():
                    paths.add(dependency_dir / name)
    # Core has no external fixture reads; adapter/engine/root tests use these roots.
    if any(package != "inventory-core" for package in selected):
        for name in ("examples", "catalog", "sql", "schemas", "data"):
            paths.update(files_under(root, root / name))
    if len(paths) > MAX_FILES:
        raise GateError("source closure exceeds file-count bound")
    return {
        path.relative_to(root).as_posix(): digest(path, MAX_SOURCE_BYTES)
        for path in sorted(paths)
    }


def source_digest(files: dict[str, str]) -> str:
    return hashlib.sha256(canonical(files)).hexdigest()


def identity(candidate: dict) -> dict:
    if not isinstance(candidate, dict) or set(candidate) - MUTANT_FIELDS - {"diff"}:
        raise GateError("unexpected candidate schema")
    if not MUTANT_FIELDS <= set(candidate):
        raise GateError("candidate is missing identity fields")
    result = {key: candidate[key] for key in sorted(MUTANT_FIELDS)}
    if result["package"] not in PACKAGES:
        raise GateError("candidate is outside first-party production packages")
    path = PurePosixPath(result["file"])
    if path.is_absolute() or ".." in path.parts:
        raise GateError("candidate source path escapes the workspace")
    for key in ("name", "file", "replacement", "genre"):
        if not isinstance(result[key], str):
            raise GateError(f"candidate field {key} is not a string")
    return result


def candidate_id(candidate: dict) -> str:
    return hashlib.sha256(canonical(identity(candidate))).hexdigest()


def candidate_map(candidates) -> dict[str, dict]:
    if not isinstance(candidates, list) or len(candidates) > MAX_CANDIDATES:
        raise GateError("candidate list is invalid or exceeds its bound")
    result = {}
    for candidate in candidates:
        key = candidate_id(candidate)
        if key in result:
            raise GateError("duplicate candidate identity")
        result[key] = identity(candidate)
    return result


def mutation_options(packages, include_ignored: bool = False) -> list[str]:
    result = ["--no-config"]
    for package in selected_packages(packages):
        result += ["--package", package]
    result += [
        "--all-features", "--no-shuffle", "--cap-lints", "false",
        "--cargo-arg=--locked", "--cargo-arg=--offline", "--cargo-arg=-j1",
    ]
    if include_ignored:
        result += ["--cargo-test-arg=--", "--cargo-test-arg=--include-ignored"]
    return result


def tool_info(tool: Path) -> dict:
    actual = digest(tool, 32 * 1024 * 1024)
    if actual != TOOL_SHA256:
        raise GateError("cargo-mutants binary differs from the pinned official release")
    version = subprocess.check_output([str(tool), "mutants", "--version"], text=True, timeout=10).strip()
    if version != f"cargo-mutants {VERSION}":
        raise GateError("cargo-mutants version mismatch")
    return {"version": VERSION, "version_output": version, "binary_sha256": actual}


def run_environment(extra: dict | None = None) -> dict:
    environment = os.environ.copy()
    for key in ("RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER", "CLIPPY_ARGS", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        if environment.get(key):
            raise GateError(f"unexpected compiler override: {key}")
    environment.update(ENVIRONMENT)
    if extra:
        environment.update(extra)
    return environment


def enumerate_candidates(root: Path, packages, tool: Path, output: Path, include_ignored=False) -> list[dict]:
    command = [str(tool), "mutants", *mutation_options(packages, include_ignored), "--list", "--json"]
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x") as stdout, output.with_suffix(".log").open("x") as stderr:
        completed = subprocess.run(command, cwd=root, env=run_environment(), stdout=stdout, stderr=stderr, timeout=120)
    if completed.returncode != 0:
        raise GateError(f"candidate enumeration failed: {output.with_suffix('.log')}")
    candidates = load_json(output)
    candidate_map(candidates)
    return candidates


def bind_fixture_context(receipt: Path | None, fixture_kind: str,
                         fixture_manifest: Path | None = None) -> dict | None:
    if receipt is None:
        if fixture_kind != "aws_emulator" or fixture_manifest is not None:
            raise GateError("an explicit fixture kind/manifest requires an outer supervision receipt")
        return None
    if fixture_kind not in {"aws_emulator", "falkordb_native"}:
        raise GateError("unsupported fixture kind")
    outer = load_json(receipt)
    if outer.get("status") != "running" or not isinstance(outer.get("argv"), list):
        raise GateError("fixture wrapper must be actively supervising the mutation command")
    context = {"receipt": str(receipt.resolve()), "fixture_kind": fixture_kind,
               "argv_sha256": hashlib.sha256(canonical(outer["argv"])).hexdigest(),
               "workspace": outer.get("workspace"), "log": outer.get("log")}
    if fixture_kind == "aws_emulator":
        if fixture_manifest is not None:
            raise GateError("AWS fixture snapshots use the existing snapshot context, not a native manifest")
        value = os.environ.get("INVENTORY_EMULATOR_SNAPSHOT_SHA256", "")
        if not re.fullmatch(r"[0-9a-f]{64}", value):
            raise GateError("fixture context has no valid frozen snapshot SHA-256")
        context["snapshot_sha256"] = value
    else:
        if fixture_manifest is None:
            raise GateError("native FalkorDB fixture requires its explicit manifest")
        value = os.environ.get("INVENTORY_MUTATION_FIXTURE_SHA256", "")
        if (not re.fullmatch(r"[0-9a-f]{64}", value)
                or digest(fixture_manifest, MAX_SOURCE_BYTES) != value):
            raise GateError("native fixture manifest does not match INVENTORY_MUTATION_FIXTURE_SHA256")
        context.update(fixture_manifest=str(fixture_manifest.resolve()), fixture_sha256=value)
    return context


def validate_required_tests(names, by_package, packages) -> None:
    def valid_names(values):
        return (isinstance(values, (list, tuple)) and len(values) <= 1000
                and all(isinstance(value, str) and 0 < len(value.encode()) <= 512 for value in values)
                and len(set(values)) == len(values))

    if not valid_names(names) or not isinstance(by_package, dict):
        raise GateError("required-test policy is malformed or exceeds its bounds")
    if not set(by_package) <= set(packages) or not all(valid_names(values) for values in by_package.values()):
        raise GateError("package-specific required tests are invalid or outside the frozen scope")


def required_tests_for(manifest: dict, packages) -> list[str]:
    names = manifest.get("required_tests", [])
    by_package = manifest.get("required_tests_by_package", {})
    validate_required_tests(names, by_package, manifest["packages"])
    return sorted(set(names).union(*(by_package.get(package, []) for package in packages)))


def freeze(root: Path, campaign: Path, packages, tool: Path, required_tests=(), required_environment=(), inputs=(),
           include_ignored=False, fixture_receipt: Path | None = None, fixture_kind="aws_emulator",
           fixture_manifest: Path | None = None, required_tests_by_package: dict | None = None) -> dict:
    packages = selected_packages(packages)
    required_tests_by_package = required_tests_by_package or {}
    validate_required_tests(required_tests, required_tests_by_package, packages)
    info = tool_info(tool)
    source_files = closure_files(root, packages)
    campaign.mkdir(parents=True, exist_ok=False)
    candidates = enumerate_candidates(root, packages, tool, campaign / "candidates.json", include_ignored)
    mapping = candidate_map(candidates)
    if not mapping:
        raise GateError("empty mutation universe cannot establish a score")
    if closure_files(root, packages) != source_files:
        raise GateError("source closure changed while enumerating candidates")
    environment_inputs = {}
    for key in required_environment:
        value = os.environ.get(key)
        if not value:
            raise GateError(f"required test environment variable is unset: {key}")
        environment_inputs[key] = hashlib.sha256(value.encode()).hexdigest()
    input_files = {str(path.resolve()): digest(path, MAX_SOURCE_BYTES) for path in inputs}
    context = bind_fixture_context(fixture_receipt, fixture_kind, fixture_manifest)
    if context is not None and fixture_kind == "falkordb_native":
        input_files[context["fixture_manifest"]] = context["fixture_sha256"]
        environment_inputs["INVENTORY_MUTATION_FIXTURE_SHA256"] = hashlib.sha256(
            context["fixture_sha256"].encode()).hexdigest()
    manifest = {
        "schema_version": 1, "scope": "package-partition", "packages": packages,
        "workspace_origin": str(root), "created_at": now(), "tool": info,
        "environment": ENVIRONMENT, "source_files": source_files,
        "source_digest": source_digest(source_files), "candidate_file": "candidates.json",
        "candidate_file_sha256": digest(campaign / "candidates.json"),
        "candidate_count": len(mapping), "candidate_ids": list(mapping),
        "mutation_options": mutation_options(packages, include_ignored), "test_scope": "mutated-package",
        "baseline_required": True, "score_threshold_percent": THRESHOLD_PERCENT,
        "include_ignored": bool(include_ignored), "baseline_runs_required": 2 if include_ignored else 1,
        "required_tests": list(required_tests), "required_environment_sha256": environment_inputs,
        "required_tests_by_package": {package: list(names)
                                      for package, names in sorted(required_tests_by_package.items())},
        "input_files": input_files,
        "fixture_context": context,
        "source_closure_policy": "Own crate/tests; first-party dependency production source and embedded data; Cargo/toolchain/config and shared fixture roots.",
        "exclusions": [
            "External dependencies", "Generated SDK catalog declarative reexports",
            "Test functions/modules recognized by cargo-mutants",
            "Examples/benchmarks are not mutation targets of cargo-mutants; their source is disclosed separately",
        ],
        "tool_limits": [
            "AST-based mutation does not expand generated/macro code or prove statement coverage",
            "Built-in skip-calls default is with_capacity; no project-specific survivor exclusions",
            "Feature-inactive candidate mutations remain in the universe and can survive",
        ],
    }
    write_json(campaign / "manifest.json", manifest, exclusive=True)
    return manifest


def read_manifest(campaign: Path, root: Path | None = None) -> tuple[dict, dict[str, dict]]:
    manifest = load_json(campaign / "manifest.json")
    if manifest.get("schema_version") != 1 or manifest.get("baseline_required") is not True:
        raise GateError("unsupported manifest or omitted baseline requirement")
    packages = selected_packages(manifest["packages"])
    validate_required_tests(manifest.get("required_tests", []), manifest.get("required_tests_by_package", {}), packages)
    if manifest["mutation_options"] != mutation_options(packages, manifest.get("include_ignored", False)):
        raise GateError("manifest changes the fixed mutation scope or test options")
    expected_baselines = 2 if manifest.get("include_ignored", False) else 1
    if manifest.get("baseline_runs_required", 1) != expected_baselines:
        raise GateError("manifest weakens the required clean-baseline policy")
    if manifest.get("tool", {}).get("binary_sha256") != TOOL_SHA256 or manifest["tool"].get("version") != VERSION:
        raise GateError("manifest uses an unpinned mutation tool")
    if manifest["environment"] != ENVIRONMENT or manifest["test_scope"] != "mutated-package":
        raise GateError("manifest changes the fixed execution environment or package test policy")
    if manifest.get("score_threshold_percent") != THRESHOLD_PERCENT:
        raise GateError("manifest changes the required score")
    if source_digest(manifest["source_files"]) != manifest["source_digest"]:
        raise GateError("source closure digest mismatch")
    candidate_path = relative_file(campaign, manifest["candidate_file"])
    if digest(candidate_path) != manifest["candidate_file_sha256"]:
        raise GateError("candidate file hash mismatch")
    mapping = candidate_map(load_json(candidate_path))
    if list(mapping) != manifest["candidate_ids"] or len(mapping) != manifest["candidate_count"]:
        raise GateError("candidate manifest does not match its immutable full list")
    if {candidate["package"] for candidate in mapping.values()} != set(packages):
        raise GateError("candidate list does not cover every declared package")
    if root is not None and closure_files(root, packages) != manifest["source_files"]:
        raise GateError("current source/test/dependency closure differs from the frozen manifest")
    return manifest, mapping


def verify_external_inputs(manifest: dict) -> dict:
    result = {}
    for name, expected in manifest.get("input_files", {}).items():
        result[name] = digest(Path(name), MAX_SOURCE_BYTES)
        if result[name] != expected:
            raise GateError(f"explicit test input changed: {name}")
    return result


def selected_ids(ids: list[str], shard: str | None) -> list[str]:
    if shard is None:
        return ids
    match = re.fullmatch(r"(\d+)/(\d+)", shard)
    if not match:
        raise GateError("shard must be k/n")
    index, count = map(int, match.groups())
    if count == 0 or count > MAX_CANDIDATES or index >= count:
        raise GateError("shard index or count is outside bounds")
    chunk = (len(ids) + count - 1) // count
    return ids[index * chunk:min((index + 1) * chunk, len(ids))]


def snapshot_workspace(root: Path, destination: Path, campaign: Path) -> None:
    manifest, _ = read_manifest(campaign, root)
    if destination == root or destination.is_relative_to(root):
        raise GateError("snapshot must be a separate disposable directory")
    shutil.copytree(root, destination, ignore=shutil.ignore_patterns(*IGNORED_DIRS))
    # A reused target may contain an interrupted mutant's compiled artifacts.
    # Copying original older mtimes must not make those appear fresh to Cargo.
    # Touch only first-party Rust inputs; external SDK source/cache is untouched.
    for name in manifest["source_files"]:
        if name.endswith(".rs"):
            os.utime(destination / name, None)
    if closure_files(destination, manifest["packages"]) != manifest["source_files"]:
        raise GateError("snapshot does not reproduce the frozen source closure")
    write_json(destination / ".mutation-workspace.json", {
        "manifest_sha256": digest(campaign / "manifest.json"), "origin": str(root),
        "first_party_rust_mtimes_refreshed": True,
    }, exclusive=True)


def run_partition(campaign: Path, workspace: Path, tool: Path, shard: str | None, target: Path | None,
                  timeout: int, build_timeout: int, wall_timeout: int) -> dict:
    manifest = load_json(campaign / "manifest.json")
    if workspace == Path(manifest["workspace_origin"]).resolve():
        raise GateError("in-place mutation of the authoring workspace is forbidden")
    with (workspace / ".mutation-run.lock").open("a+") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise GateError("another mutation process already owns this disposable workspace") from error
        return run_partition_locked(campaign, workspace, tool, shard, target, timeout, build_timeout, wall_timeout)


def run_partition_locked(campaign: Path, workspace: Path, tool: Path, shard: str | None, target: Path | None,
                         timeout: int, build_timeout: int, wall_timeout: int) -> dict:
    manifest, mutations = read_manifest(campaign, workspace)
    if workspace == Path(manifest["workspace_origin"]).resolve():
        raise GateError("in-place mutation of the authoring workspace is forbidden")
    marker = load_json(workspace / ".mutation-workspace.json")
    if marker["manifest_sha256"] != digest(campaign / "manifest.json"):
        raise GateError("disposable workspace marker belongs to another manifest")
    tool_info(tool)
    external_inputs = verify_external_inputs(manifest)
    for key, expected in manifest.get("required_environment_sha256", {}).items():
        actual = os.environ.get(key, "")
        if hashlib.sha256(actual.encode()).hexdigest() != expected:
            raise GateError(f"required test environment differs: {key}")
    if not (1 <= timeout <= 3600 and 1 <= build_timeout <= 3600 and 1 <= wall_timeout <= 604800):
        raise GateError("timeout is outside its explicit bound")
    ids = selected_ids(manifest["candidate_ids"], shard)
    if not ids:
        raise GateError("selected shard contains no candidates")
    run_root = campaign / "runs"
    run_root.mkdir(exist_ok=True)
    run_dir = None
    for number in range(1, MAX_RUNS + 1):
        possible = run_root / f"{number:04d}"
        try:
            possible.mkdir(exist_ok=False)
            run_dir = possible
            break
        except FileExistsError:
            continue
    if run_dir is None:
        raise GateError("run-count bound exceeded")
    target = target or workspace / "target"
    environment = run_environment({"CARGO_TARGET_DIR": str(target)})
    command = [str(tool), "mutants", *manifest["mutation_options"], "--baseline", "run", "--in-place", "--jobs", "1",
               "--timeout", str(timeout), "--build-timeout", str(build_timeout), "--output", str(run_dir)]
    if shard is not None:
        command += ["--shard", shard, "--sharding", "slice"]
    receipt = {
        "schema_version": 1, "manifest_sha256": digest(campaign / "manifest.json"),
        "started_at": now(), "command": command, "cwd": str(workspace),
        "target_directory": str(target), "environment": ENVIRONMENT,
        "source_digest_before": manifest["source_digest"], "candidate_ids": ids,
        "shard": shard, "status": "running", "tool_binary_sha256": TOOL_SHA256,
        "required_environment_sha256": manifest.get("required_environment_sha256", {}),
        "input_files_before": external_inputs,
    }
    write_json(run_dir / "receipt.json", receipt, exclusive=True)
    started = time.monotonic()
    interrupted = False
    preflight_ok = True
    if manifest.get("baseline_runs_required", 1) == 2:
        packages = sorted({mutations[key]["package"] for key in ids})
        preflight = {"scenario": "Baseline", "summary": "Failure", "log_path": "preflight-baseline.log", "phase_results": []}
        with (run_dir / "preflight-baseline.log").open("x") as log:
            for phase, is_build in (("Build", True), ("Test", False)):
                argv = ["cargo", "test"] + (["--no-run"] if is_build else []) + ["--verbose"]
                argv += ["--package=" + package for package in packages]
                argv += ["--all-features", "--locked", "--offline", "-j1"]
                if not is_build and manifest.get("include_ignored", False):
                    argv += ["--", "--include-ignored"]
                before = time.monotonic()
                status = execute_bounded(argv, workspace, environment, log, build_timeout if is_build else timeout)
                preflight["phase_results"].append({"phase": phase, "duration": time.monotonic() - before,
                                                   "process_status": status, "argv": argv})
                if status != "Success":
                    interrupted = status == "Timeout" or (isinstance(status, dict) and "Signalled" in status)
                    preflight_ok = False
                    break
        if preflight_ok:
            preflight["summary"] = "Success"
        receipt["preflight_baseline"] = preflight
    with (run_dir / "runner.log").open("x") as output:
        if preflight_ok:
            status = execute_bounded(command, workspace, environment, output, wall_timeout)
            interrupted = status == "Timeout" or (isinstance(status, dict) and "Signalled" in status)
            code = (0 if status == "Success" else
                    status.get("Failure", 128 + status.get("Signalled", 2)) if isinstance(status, dict) else 130)
        else:
            code = 10
    after = closure_files(workspace, manifest["packages"])
    evidence = {}
    for path in sorted((run_dir / "mutants.out").rglob("*")):
        if path.is_file():
            evidence[path.relative_to(run_dir).as_posix()] = digest(path)
        if len(evidence) > MAX_FILES:
            raise GateError("mutation evidence file-count bound exceeded")
    evidence["runner.log"] = digest(run_dir / "runner.log")
    if (run_dir / "preflight-baseline.log").is_file():
        evidence["preflight-baseline.log"] = digest(run_dir / "preflight-baseline.log")
    receipt.update(status="interrupted" if interrupted else "finished", exit_code=code, finished_at=now(),
                   seconds=round(time.monotonic() - started, 3), source_digest_after=source_digest(after),
                   source_restored=after == manifest["source_files"], evidence_files=evidence)
    receipt["input_files_after"] = verify_external_inputs(manifest)
    write_json(run_dir / "receipt.json", receipt)
    return {"receipt": str(run_dir / "receipt.json"), **{key: receipt[key] for key in ("status", "exit_code", "seconds", "source_restored")}}


def execute_bounded(argv: list[str], workspace: Path, environment: dict, output, timeout: int):
    # The fixture supervisor owns this Python process, while cargo-mutants owns
    # a separate process group. Forward both termination signals so a failed
    # fixture cannot leave its mutation worker alive in that separate group.
    requested_signal = None

    def request_stop(number, _frame):
        nonlocal requested_signal
        if requested_signal is None:
            requested_signal = number

    previous = {number: signal.getsignal(number) for number in (signal.SIGINT, signal.SIGTERM)}
    process = None
    try:
        for number in previous:
            signal.signal(number, request_stop)
        if requested_signal is not None:
            return {"Signalled": requested_signal}
        process = subprocess.Popen(argv, cwd=workspace, env=environment, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        deadline = time.monotonic() + timeout
        while True:
            if requested_signal is not None:
                return {"Signalled": requested_signal}
            code = process.poll()
            if code is not None:
                if code == 0:
                    return "Success"
                return {"Signalled": -code} if code < 0 else {"Failure": code}
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return "Timeout"
            try:
                process.wait(timeout=min(0.25, remaining))
            except subprocess.TimeoutExpired:
                pass
    finally:
        try:
            if process is not None and process.poll() is None:
                try:
                    os.killpg(process.pid, signal.SIGINT)
                    # Keep this drain below mutate.py's ten-second termination
                    # budget. cargo-mutants forwards the interrupt to Cargo's
                    # owned process group and restores the edited source.
                    process.wait(timeout=6)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=2)
                except ProcessLookupError:
                    process.wait(timeout=2)
        finally:
            for number, handler in previous.items():
                signal.signal(number, handler)


def cargo_packages(argv: list) -> set[str]:
    packages = set()
    for position, value in enumerate(argv):
        if value.startswith("--package="):
            packages.add(value.split("=", 1)[1].split("@", 1)[0])
        elif value in ("-p", "--package") and position + 1 < len(argv):
            packages.add(argv[position + 1].split("@", 1)[0])
    return packages


def validate_phase(phase: dict, expected_packages: set[str], build: bool, include_ignored=False) -> str | dict:
    argv = phase.get("argv")
    if not isinstance(argv, list) or not all(isinstance(value, str) for value in argv):
        raise GateError("phase argv is missing")
    if len(argv) < 2 or Path(argv[0]).name != "cargo" or argv[1] != "test":
        raise GateError("mutation phase did not run Cargo tests")
    allowed = {"--verbose", "--no-run", "--all-features", "--locked", "--offline", "-j1"}
    test_args = ["--", "--include-ignored"] if include_ignored and not build else []
    cargo_args = argv[:-2] if test_args and argv[-2:] == test_args else argv
    if test_args and argv[-2:] != test_args:
        raise GateError("ignored integration tests were not enabled")
    if any(value not in allowed and not value.startswith("--package=") for value in cargo_args[2:]):
        raise GateError("Cargo phase changes the fixed test target/feature/filter policy")
    if not {"--all-features", "--locked", "--offline", "-j1"} <= set(argv):
        raise GateError("Cargo phase omits a required feature/lock/offline/jobs argument")
    if ("--no-run" in argv) != build or cargo_packages(argv) != expected_packages:
        raise GateError("Cargo phase has incorrect build/test or package selection")
    return phase.get("process_status")


def ordinary_failure(status) -> bool:
    return (isinstance(status, dict) and set(status) == {"Failure"}
            and isinstance(status["Failure"], int) and not isinstance(status["Failure"], bool)
            and 0 < status["Failure"] <= 255)


def classify_outcome(outcome: dict, log: str, expected_packages: set[str], baseline=False,
                     required_tests=(), include_ignored=False) -> tuple[str, str | None]:
    phases = outcome.get("phase_results")
    if not isinstance(phases, list) or not 1 <= len(phases) <= 2:
        return "unresolved", "missing or unsupported execution phases"
    try:
        if phases[0].get("phase") != "Build":
            return "unresolved", "metadata-only checks do not establish mutation test outcomes"
        build = validate_phase(phases[0], expected_packages, True, include_ignored)
        if build == "Timeout" or any(phase.get("process_status") == "Timeout" for phase in phases):
            return "timeout", "timeout is unresolved, never caught"
        if INTERNAL_DEADLINE.search(log):
            return "timeout", "an internal test-harness deadline elapsed; not credited as caught"
        if INFRA_FAILURE.search(log) or COMPILER_INFRA_FAILURE.search(log):
            return "unresolved", "infrastructure failure appears in the execution log"
        if ordinary_failure(build):
            if (not baseline and len(phases) == 1 and outcome.get("summary") == "Unviable"
                    and COMPILER_DIAGNOSTIC.search(log)):
                return "unviable", None
            return "unresolved", "build failure lacks confirmed compiler-error evidence"
        if build != "Success" or len(phases) != 2 or phases[1].get("phase") != "Test":
            return "unresolved", "no successful build followed by an executed test phase"
        test = validate_phase(phases[1], expected_packages, False, include_ignored)
        if test == "Success":
            if not re.search(r"^test result: ok\. [1-9]\d* passed;", log, re.M):
                return "unresolved", "no evidence that any tests actually executed"
            for name in required_tests:
                if not re.search(r"^test " + re.escape(name) + r" \.\.\. ok$", log, re.M):
                    return "unresolved", f"required test did not report success: {name}"
            expected_summary = "Success" if baseline else "MissedMutant"
            if outcome.get("summary") != expected_summary:
                return "unresolved", "summary conflicts with successful test phases"
            return ("baseline_passed" if baseline else "missed"), None
        if not baseline and ordinary_failure(test) and outcome.get("summary") == "CaughtMutant":
            if re.search(r"^test result: FAILED\. .* [1-9]\d* failed;", log, re.M):
                return "caught", None
            return "unresolved", "test failure has no failing-test result; not credited as caught"
        return "unresolved", "baseline failed or test process did not finish normally"
    except GateError as error:
        return "unresolved", str(error)


def read_log(run_dir: Path, outcome: dict, evidence: dict) -> str:
    relative = "mutants.out/" + outcome.get("log_path", "")
    if relative not in evidence:
        raise GateError("outcome log is missing from the receipt")
    path = relative_file(run_dir, relative)
    if path.stat().st_size > MAX_LOG_BYTES:
        raise GateError("outcome log exceeds inspection bound")
    return path.read_text(errors="replace")


def validate_fixture_context(manifest: dict, *, allow_pending=False) -> dict | None:
    expected = manifest.get("fixture_context")
    if expected is None:
        return None
    kind = expected.get("fixture_kind", "aws_emulator")
    if kind not in {"aws_emulator", "falkordb_native"}:
        raise GateError("unsupported frozen fixture kind")
    details = {"fixture_kind": kind}
    if kind == "falkordb_native":
        value = expected.get("fixture_sha256", "")
        if (not re.fullmatch(r"[0-9a-f]{64}", value)
                or manifest.get("input_files", {}).get(expected.get("fixture_manifest")) != value
                or manifest.get("required_environment_sha256", {}).get("INVENTORY_MUTATION_FIXTURE_SHA256")
                != hashlib.sha256(value.encode()).hexdigest()):
            raise GateError("native fixture is not bound to its explicit manifest and environment")
        details.update(fixture_sha256=value, fixture_manifest=expected["fixture_manifest"])
    elif "snapshot_sha256" in expected:
        details["snapshot_sha256"] = expected["snapshot_sha256"]
    path = Path(expected["receipt"])
    actual = load_json(path)
    if (hashlib.sha256(canonical(actual.get("argv"))).hexdigest() != expected["argv_sha256"]
            or actual.get("workspace") != expected["workspace"] or actual.get("log") != expected["log"]):
        raise GateError("outer fixture receipt does not match the frozen mutation command/context")
    if allow_pending and actual.get("status") == "running":
        return {"status": "pending", "receipt": str(path), **details}
    checks = actual.get("fixture_health_checks")
    if (actual.get("status") != "completed" or actual.get("exit_code") != 0
            or not isinstance(checks, int) or isinstance(checks, bool) or checks < 2):
        raise GateError("outer fixture supervision did not complete healthy; run cannot establish caught mutations")
    log = Path(actual["log"])
    if digest(log, 128 * 1024 * 1024) != actual.get("log_sha256"):
        raise GateError("outer fixture command log hash changed")
    return {"status": "verified", "receipt": str(path), "sha256": digest(path), "health_checks": checks, **details}


def campaign_results(campaign: Path, root: Path, *, allow_pending_fixture=False) -> dict:
    manifest, candidates = read_manifest(campaign, root)
    fixture_context = validate_fixture_context(manifest, allow_pending=allow_pending_fixture)
    attempts = defaultdict(list)
    errors = []
    receipts = sorted((campaign / "runs").glob("*/receipt.json"))
    if len(receipts) > MAX_RUNS:
        raise GateError("run-count bound exceeded")
    for receipt_path in receipts:
        run_dir = receipt_path.parent
        try:
            receipt = load_json(receipt_path)
            if receipt["manifest_sha256"] != digest(campaign / "manifest.json"):
                raise GateError("run belongs to another manifest")
            if receipt.get("status") != "finished" or receipt.get("exit_code") not in (0, 2, 3):
                raise GateError("run did not finish normally; completed outcomes remain preliminary")
            if receipt.get("tool_binary_sha256") != TOOL_SHA256 or receipt.get("environment") != ENVIRONMENT:
                raise GateError("run tool or environment differs from the manifest")
            if (receipt.get("source_digest_before") != manifest["source_digest"]
                    or receipt.get("source_digest_after") != manifest["source_digest"]
                    or receipt.get("source_restored") is not True):
                raise GateError("run source closure was not preserved")
            if receipt.get("required_environment_sha256", {}) != manifest.get("required_environment_sha256", {}):
                raise GateError("run required-test environment does not match the manifest")
            if (receipt.get("input_files_before", {}) != manifest.get("input_files", {})
                    or receipt.get("input_files_after", {}) != manifest.get("input_files", {})):
                raise GateError("run external fixture inputs changed or were not recorded")
            expected_ids = selected_ids(manifest["candidate_ids"], receipt.get("shard"))
            if receipt["candidate_ids"] != expected_ids:
                raise GateError("run candidate selection is not its complete declared shard")
            evidence = receipt.get("evidence_files", {})
            if not evidence or len(evidence) > MAX_FILES:
                raise GateError("run evidence hashes are missing or oversized")
            for name, expected in evidence.items():
                if digest(relative_file(run_dir, name)) != expected:
                    raise GateError(f"run evidence file hash changed: {name}")
            for required in ("mutants.out/mutants.json", "mutants.out/outcomes.json"):
                if required not in evidence:
                    raise GateError(f"missing required run evidence: {required}")
            selected = candidate_map(load_json(run_dir / "mutants.out/mutants.json"))
            if list(selected) != expected_ids:
                raise GateError("tool selected mutants differ from the frozen declared shard")
            results = load_json(run_dir / "mutants.out/outcomes.json")
            if results.get("cargo_mutants_version") != VERSION or not results.get("end_time"):
                raise GateError("outcomes are not a completed pinned-tool run")
            outcomes = results.get("outcomes", [])
            baselines = [value for value in outcomes if value.get("scenario") == "Baseline"]
            if len(baselines) != 1:
                raise GateError("exactly one real baseline is required per run")
            baseline = baselines[0]
            baseline_packages = {candidates[key]["package"] for key in expected_ids}
            if manifest.get("baseline_runs_required", 1) == 2:
                preflight = receipt.get("preflight_baseline")
                if not preflight or "preflight-baseline.log" not in evidence:
                    raise GateError("second required clean baseline is missing")
                path = relative_file(run_dir, "preflight-baseline.log")
                if path.stat().st_size > MAX_LOG_BYTES:
                    raise GateError("preflight baseline log exceeds inspection bound")
                status, detail = classify_outcome(preflight, path.read_text(errors="replace"), baseline_packages,
                    baseline=True, required_tests=required_tests_for(manifest, baseline_packages),
                    include_ignored=manifest.get("include_ignored", False))
                if status != "baseline_passed":
                    raise GateError(f"second clean baseline did not pass: {detail}")
            status, detail = classify_outcome(baseline, read_log(run_dir, baseline, evidence), baseline_packages,
                                              baseline=True, required_tests=required_tests_for(manifest, baseline_packages),
                                              include_ignored=manifest.get("include_ignored", False))
            if status != "baseline_passed":
                raise GateError(f"baseline did not pass with executed tests: {detail}")
            seen = set()
            raw_counts = Counter()
            run_attempts = []
            for outcome in outcomes:
                scenario = outcome.get("scenario")
                if scenario == "Baseline":
                    continue
                if not isinstance(scenario, dict) or set(scenario) != {"Mutant"}:
                    raise GateError("unknown outcome scenario")
                key = candidate_id(scenario["Mutant"])
                if key not in selected or key in seen:
                    raise GateError("unknown or duplicate mutant outcome")
                seen.add(key)
                raw_counts[outcome.get("summary")] += 1
                status, detail = classify_outcome(outcome, read_log(run_dir, outcome, evidence),
                                                  {candidates[key]["package"]},
                                                  required_tests=required_tests_for(manifest, [candidates[key]["package"]]),
                                                  include_ignored=manifest.get("include_ignored", False))
                run_attempts.append((key, {"status": status, "detail": detail, "tool_summary": outcome.get("summary"),
                                          "receipt": str(receipt_path),
                                          "log": str(run_dir / "mutants.out" / outcome["log_path"])}))
            if results.get("total_mutants") != len(seen):
                raise GateError("tool total_mutants disagrees with individual outcomes")
            for field, label in (("caught", "CaughtMutant"), ("missed", "MissedMutant"),
                                 ("timeout", "Timeout"), ("unviable", "Unviable"), ("success", "Success")):
                if results.get(field) != raw_counts[label]:
                    raise GateError(f"tool summary field {field} disagrees with individual outcomes")
            if seen != set(expected_ids):
                errors.append(f"{receipt_path}: run omitted {len(set(expected_ids) - seen)} selected outcomes")
            for key, attempt in run_attempts:
                attempts[key].append(attempt)
        except (GateError, KeyError, OSError, ValueError, TypeError) as error:
            errors.append(f"{receipt_path}: {error}")
    return {"manifest": str(campaign / "manifest.json"), "manifest_sha256": digest(campaign / "manifest.json"),
            "manifest_data": manifest,
            "packages": manifest["packages"], "candidates": candidates, "attempts": dict(attempts),
            "errors": errors, "run_count": len(receipts), "exclusions": manifest.get("exclusions", []),
            "fixture_context": fixture_context}


def summarize(candidates: dict[str, dict], attempts: dict, errors=(), full_scope=False) -> dict:
    counts = Counter()
    rows = []
    package_counts = defaultdict(Counter)
    timeout_attempts = 0
    for key, candidate in candidates.items():
        observed = attempts.get(key, [])
        timeout_attempts += sum(value["status"] == "timeout" for value in observed)
        completed = {value["status"] for value in observed if value["status"] in {"caught", "missed", "unviable"}}
        if len(completed) > 1:
            status = "unresolved"
            detail = "contradictory completed outcomes on the same frozen source; not cherry-picked"
        elif completed:
            status = next(iter(completed))
            detail = None
        elif observed:
            status, detail = "unresolved", "only timeout or infrastructure/incomplete outcomes"
        else:
            status, detail = "untested", "no accepted completed-run outcome"
        counts[status] += 1
        package_counts[candidate["package"]][status] += 1
        rows.append({"id": key, "name": candidate["name"], "package": candidate["package"],
                     "status": status, "detail": detail, "attempts": observed})
    viable = counts["caught"] + counts["missed"]
    score_met = viable > 0 and counts["caught"] * 100 >= viable * THRESHOLD_PERCENT
    complete = not counts["untested"] and not counts["unresolved"] and not errors
    return {
        "schema_version": 1, "observed_at": now(), "gate_passed": bool(full_scope and complete and score_met),
        "whole_first_party_scope": bool(full_scope), "complete": complete,
        "threshold_percent": THRESHOLD_PERCENT, "score_requirement_met": score_met,
        "score_percent": round(100 * counts["caught"] / viable, 6) if viable else None,
        "score_formula": "caught / (caught + missed); compiler-confirmed unviable excluded; unresolved and untested prohibit passing",
        "candidate_count": len(candidates), "viable_completed": viable,
        "counts": {name: counts[name] for name in ("caught", "missed", "unviable", "unresolved", "untested")},
        "timeout_attempts": timeout_attempts, "by_package": {key: dict(value) for key, value in sorted(package_counts.items())},
        "errors": list(errors), "mutants": rows,
        "limits": [
            "Mutation score is not line/branch coverage, live AWS fidelity, or proof of no bugs",
            "A package partition cannot establish whole-first-party completion",
            "Timeouts are not caught; a later consistent completed attempt may resolve that candidate while timeout history remains visible",
        ],
    }


def report_campaign(campaign: Path, root: Path, *, allow_pending_fixture=False) -> dict:
    result = campaign_results(campaign, root, allow_pending_fixture=allow_pending_fixture)
    report = summarize(result["candidates"], result["attempts"], result["errors"], full_scope=False)
    report.update(manifests=[result["manifest"]], exclusions=result["exclusions"], run_count=result["run_count"],
                  fixture_context=result["fixture_context"])
    if result["fixture_context"] and result["fixture_context"]["status"] == "pending":
        report["complete"] = False
    return report


def validate_aws_snapshot_binding(manifest: dict, context: dict) -> None:
    value = context.get("snapshot_sha256", "")
    environment = manifest.get("required_environment_sha256", {})
    if (not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value)
            or environment.get("INVENTORY_EMULATOR_SNAPSHOT_SHA256")
            != hashlib.sha256(value.encode()).hexdigest()):
        raise GateError("final AWS fixture snapshot digest is not bound to its environment")
    # Bind the value and path together, without exposing environment values or
    # depending on an ephemeral fixture still being present after finalization.
    if not any(
        expected == value and hashlib.sha256(name.encode()).hexdigest()
        == environment.get("INVENTORY_EMULATOR_SNAPSHOT_PATH")
        for name, expected in manifest.get("input_files", {}).items()
    ):
        raise GateError("final AWS fixture snapshot path and content are not frozen inputs")


def validate_acceptance_policy(manifest: dict) -> None:
    """Require the complete live contract at the final aggregation boundary."""
    for package in manifest["packages"]:
        required = REQUIRED_LIVE_TESTS[package]
        if not required:
            continue
        if manifest.get("include_ignored") is not True or manifest.get("baseline_runs_required") != 2:
            raise GateError(f"final {package} evidence requires ignored live tests and two clean baselines")
        if not set(required) <= set(required_tests_for(manifest, [package])):
            raise GateError(f"final {package} evidence omits a mandatory live test")
        context = manifest.get("fixture_context")
        if not isinstance(context, dict):
            raise GateError(f"final {package} evidence requires a supervised fixture lifetime")
        kind = context.get("fixture_kind", "aws_emulator")
        if package in {"aws-inventory", "inventory-aws"} and kind != "aws_emulator":
            raise GateError(f"final {package} evidence requires the AWS emulator fixture")
        environment = manifest.get("required_environment_sha256", {})
        for name in REQUIRED_LIVE_ENVIRONMENT[package]:
            value = environment.get(name)
            if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
                raise GateError(f"final {package} fixture environment is not bound: {name}")
        if kind == "aws_emulator":
            validate_aws_snapshot_binding(manifest, context)


def frozen_fixture_identity(context: dict | None) -> tuple[str, str] | None:
    """Identify comparable frozen inputs, independent of receipt paths/reruns."""
    if context is None:
        return None
    if not isinstance(context, dict):
        raise GateError("invalid final fixture context")
    kind = context.get("fixture_kind", "aws_emulator")
    field = {"aws_emulator": "snapshot_sha256", "falkordb_native": "fixture_sha256"}.get(kind)
    if field is None:
        raise GateError("unsupported final fixture kind")
    value = context.get(field)
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
        raise GateError(f"final {kind} fixture identity is missing or invalid")
    return kind, value


def aggregate(root: Path, campaigns: list[Path], tool: Path, universe_output: Path) -> dict:
    tool_info(tool)
    initial = closure_files(root, PACKAGES)
    universe = candidate_map(enumerate_candidates(root, PACKAGES, tool, universe_output))
    if {candidate["package"] for candidate in universe.values()} != set(PACKAGES):
        raise GateError("final enumeration omits a required production package")
    if closure_files(root, PACKAGES) != initial:
        raise GateError("full source changed during final universe enumeration")
    attempts = defaultdict(list)
    fixture_identities = defaultdict(set)
    errors, manifests, exclusions = [], [], []
    represented = set()
    for campaign in campaigns:
        try:
            result = campaign_results(campaign, root)
            validate_acceptance_policy(result["manifest_data"])
            fixture = frozen_fixture_identity(result["fixture_context"])
            for key, candidate in result["candidates"].items():
                if key not in universe or candidate != universe[key]:
                    raise GateError("partition contains a stale or out-of-universe candidate")
                represented.add(key)
            for key, values in result["attempts"].items():
                attempts[key].extend(values)
            errors.extend(result["errors"])
            manifests.append({"path": result["manifest"], "sha256": result["manifest_sha256"]})
            exclusions.extend(result["exclusions"])
            if fixture is not None:
                kind, value = fixture
                fixture_identities[kind].add(value)
        except (GateError, KeyError, OSError, ValueError, TypeError) as error:
            errors.append(f"{campaign}: {error}")
    # AWS-backed and native-engine partitions are deliberately different kinds.
    # Within either kind, every retry/shard must use the same frozen fixture.
    # Preserve all attempts; do not select a favorable fixture for each mutant.
    for kind, values in sorted(fixture_identities.items()):
        if len(values) != 1:
            errors.append(f"multiple frozen {kind} fixture identities across accepted campaigns: {len(values)}")
    if represented != set(universe):
        errors.append(f"{len(set(universe) - represented)} final-universe candidates lack a current closure manifest")
    report = summarize(universe, attempts, errors, full_scope=True)
    report.update(manifests=manifests, exclusions=sorted(set(exclusions)),
                  fixture_identities={kind: sorted(values) for kind, values in sorted(fixture_identities.items())},
                  universe_file=str(universe_output), universe_sha256=digest(universe_output),
                  final_source_digest=source_digest(initial), final_source_files=initial,
                  non_mutated_example_targets=[
                      str(path.relative_to(root)) for package in PACKAGE_DIRS if package != "aws-inventory"
                      for path in sorted((root / PACKAGE_DIRS[package] / "examples").glob("*.rs"))
                  ])
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tool", type=Path, default=DEFAULT_TOOL)
    sub = parser.add_subparsers(dest="operation", required=True)
    freeze_parser = sub.add_parser("freeze", help="Freeze an entire package partition before running any mutations")
    freeze_parser.add_argument("--workspace", type=Path, required=True)
    freeze_parser.add_argument("--campaign", type=Path, required=True)
    freeze_parser.add_argument("--package", action="append", choices=PACKAGES, required=True)
    freeze_parser.add_argument("--required-test", action="append", default=[])
    freeze_parser.add_argument("--required-package-test", action="append", default=[], metavar="PACKAGE=TEST",
                               help="Require a real test only when that package's suite is executed")
    freeze_parser.add_argument("--required-env", action="append", default=[])
    freeze_parser.add_argument("--input", type=Path, action="append", default=[])
    freeze_parser.add_argument("--include-ignored", action="store_true",
                               help="Run ignored integration tests and require two clean baselines")
    freeze_parser.add_argument("--fixture-receipt", type=Path,
                               help="Bind the active outer fixture wrapper receipt; aggregate requires final completion")
    freeze_parser.add_argument("--fixture-kind", choices=("aws_emulator", "falkordb_native"), default="aws_emulator")
    freeze_parser.add_argument("--fixture-manifest", type=Path,
                               help="Explicit native fixture manifest bound by INVENTORY_MUTATION_FIXTURE_SHA256")
    snapshot_parser = sub.add_parser("snapshot")
    snapshot_parser.add_argument("--workspace", type=Path, required=True)
    snapshot_parser.add_argument("--campaign", type=Path, required=True)
    snapshot_parser.add_argument("--destination", type=Path, required=True)
    run_parser = sub.add_parser("run")
    run_parser.add_argument("--workspace", type=Path, required=True)
    run_parser.add_argument("--campaign", type=Path, required=True)
    run_parser.add_argument("--shard")
    run_parser.add_argument("--target-dir", type=Path)
    run_parser.add_argument("--timeout", type=int, default=120)
    run_parser.add_argument("--build-timeout", type=int, default=300)
    run_parser.add_argument("--wall-timeout", type=int, default=86400)
    report_parser = sub.add_parser("report")
    report_parser.add_argument("--workspace", type=Path, required=True)
    report_parser.add_argument("--campaign", type=Path, required=True)
    report_parser.add_argument("--output", type=Path, required=True)
    aggregate_parser = sub.add_parser("aggregate")
    aggregate_parser.add_argument("--workspace", type=Path, required=True)
    aggregate_parser.add_argument("--campaign", type=Path, action="append", required=True)
    aggregate_parser.add_argument("--universe-output", type=Path, required=True)
    aggregate_parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        root = args.workspace.resolve()
        tool = args.tool.resolve()
        if args.operation == "freeze":
            by_package = defaultdict(list)
            for value in args.required_package_test:
                package, separator, name = value.partition("=")
                if not separator or not name:
                    raise GateError("--required-package-test must be PACKAGE=TEST")
                by_package[package].append(name)
            result = freeze(root, args.campaign.resolve(), args.package, tool, args.required_test,
                            args.required_env, args.input, args.include_ignored, args.fixture_receipt,
                            args.fixture_kind, args.fixture_manifest, dict(by_package))
            print(json.dumps({"manifest": str(args.campaign / "manifest.json"), "candidate_count": result["candidate_count"]}))
        elif args.operation == "snapshot":
            snapshot_workspace(root, args.destination.resolve(), args.campaign.resolve())
            print(json.dumps({"snapshot": str(args.destination.resolve())}))
        elif args.operation == "run":
            result = run_partition(args.campaign.resolve(), root, tool, args.shard,
                                   args.target_dir.resolve() if args.target_dir else None,
                                   args.timeout, args.build_timeout, args.wall_timeout)
            print(json.dumps(result, sort_keys=True))
            return 0 if result["exit_code"] in (0, 2, 3) and result["source_restored"] else 1
        else:
            if args.operation == "report":
                result = report_campaign(args.campaign.resolve(), root)
            else:
                result = aggregate(root, [path.resolve() for path in args.campaign], tool, args.universe_output.resolve())
            write_json(args.output, result)
            print(json.dumps({key: value for key, value in result.items() if key not in {"mutants", "final_source_files"}}, sort_keys=True))
            return 0 if result["gate_passed"] else 1
    except (GateError, OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        parser.exit(2, f"mutation gate failed closed: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
