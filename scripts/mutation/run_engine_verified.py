#!/usr/bin/env python3
"""Supervise a frozen engine mutation campaign with a real native FalkorDB.

The outer process owns the server and checks PONG throughout the child command.
The gate binds the native fixture manifest and final supervision receipt. Create
CONTEXT/STOP to request an owned, bounded drain without a tool-level interrupt.
Only a complete whole-project aggregate can establish the overall 95% gate.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import time
from urllib.parse import urlsplit

import gate

ROOT = Path(__file__).resolve().parents[2]

MAX_LOG_BYTES = 128 * 1024 * 1024
HEALTH_INTERVAL_SECONDS = 5
GROUP_DRAIN_SECONDS = 55
FORCED_DRAIN_SECONDS = 10


def request_stop(_signal, _frame):
    raise KeyboardInterrupt


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def endpoint_port(url: str) -> int:
    parts = urlsplit(url)
    if (parts.scheme != "redis" or parts.hostname != "127.0.0.1"
            or parts.username is not None or parts.password is not None
            or parts.path not in ("", "/") or parts.query or parts.fragment
            or parts.port is None or not 1 <= parts.port <= 65_535):
        raise ValueError("native fixture endpoint must be a plain loopback Redis URL")
    return parts.port


def probe_health(url: str) -> None:
    with socket.create_connection(("127.0.0.1", endpoint_port(url)), timeout=3) as connection:
        connection.sendall(b"*1\r\n$4\r\nPING\r\n")
        received = bytearray()
        for _ in range(7):
            if len(received) == 7:
                break
            chunk = connection.recv(7 - len(received))
            if not chunk:
                break
            received.extend(chunk)
        if received != b"+PONG\r\n":
            raise RuntimeError("native fixture did not return exactly PONG")


def endpoint_closed(url: str) -> bool:
    try:
        with socket.create_connection(("127.0.0.1", endpoint_port(url)), timeout=1):
            return False
    except ConnectionRefusedError:
        return True


def stop_child(process: subprocess.Popen | None) -> None:
    if process is None:
        return

    def drained():
        process.poll()
        try:
            os.killpg(process.pid, 0)
            return False
        except ProcessLookupError:
            return True

    # A reaped group leader can still have live descendants. Signal the owned
    # process group even when the leader already returned.
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        process.wait(timeout=FORCED_DRAIN_SECONDS)
        return
    deadline = time.monotonic() + GROUP_DRAIN_SECONDS
    while time.monotonic() < deadline:
        if drained():
            process.wait(timeout=FORCED_DRAIN_SECONDS)
            return
        time.sleep(0.1)
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    deadline = time.monotonic() + FORCED_DRAIN_SECONDS
    while time.monotonic() < deadline:
        if drained():
            process.wait(timeout=FORCED_DRAIN_SECONDS)
            return
        time.sleep(0.1)
    raise RuntimeError("owned child process group did not drain within its bound")


class FixtureUnavailable(RuntimeError):
    pass


def supervise(command: list[str], workspace: Path, context: Path, url: str, deadline: int) -> dict:
    """Own one real child process group and persist its health/termination proof.

    Successful child completion is recorded as command_completed until the
    outer caller has also closed its owned native server context.
    """
    log = context / "fixture-command.log"
    receipt_path = context / "fixture-command.json"
    health_path = context / "health.jsonl"
    receipt = {"argv": command, "workspace": str(workspace), "log": str(log),
               "status": "running", "fixture_kind": "falkordb_native",
               "deadline_seconds": deadline, "stop_request": str(context / "STOP")}
    gate.write_json(receipt_path, receipt, exclusive=True)
    started = time.monotonic()
    checks = 0
    process = None
    with log.open("xb") as output, health_path.open("x") as health:
        def check(phase):
            nonlocal checks
            event = {"phase": phase, "elapsed_seconds": round(time.monotonic() - started, 3)}
            try:
                probe_health(url)
            except (OSError, ValueError, RuntimeError) as error:
                event.update(passed=False, error_type=type(error).__name__)
                health.write(json.dumps(event) + "\n")
                health.flush()
                raise FixtureUnavailable(type(error).__name__) from error
            checks += 1
            event["passed"] = True
            health.write(json.dumps(event) + "\n")
            health.flush()

        try:
            check("before")
            process = subprocess.Popen(command, cwd=workspace, stdin=subprocess.DEVNULL,
                                       stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
            next_health = time.monotonic() + HEALTH_INTERVAL_SECONDS
            next_progress = time.monotonic() + 30
            while process.poll() is None:
                now = time.monotonic()
                if (context / "STOP").exists():
                    receipt["status"] = "interrupted"
                    break
                if now - started > deadline:
                    receipt["status"] = "timeout"
                    break
                if log.stat().st_size > MAX_LOG_BYTES:
                    receipt["status"] = "log_budget_exceeded"
                    break
                if now >= next_health:
                    check("periodic")
                    next_health = now + HEALTH_INTERVAL_SECONDS
                if now >= next_progress:
                    print(json.dumps({"phase": "native_engine", "elapsed_seconds": round(now - started),
                                      "health_checks": checks}), flush=True)
                    next_progress = now + 30
                time.sleep(0.25)
            if receipt["status"] == "running":
                # A short-lived child can exit before the polling loop sees
                # its final output or wall time. Apply the same bounds here.
                if log.stat().st_size > MAX_LOG_BYTES:
                    receipt["status"] = "log_budget_exceeded"
                elif time.monotonic() - started > deadline:
                    receipt["status"] = "timeout"
                elif (context / "STOP").exists():
                    receipt["status"] = "interrupted"
                else:
                    check("after")
                    if time.monotonic() - started > deadline:
                        receipt["status"] = "timeout"
                    else:
                        receipt.update(status="command_completed", exit_code=process.returncode)
        except FixtureUnavailable as error:
            receipt.update(status="fixture_unavailable", fixture_error_type=str(error))
        except (OSError, ValueError) as error:
            receipt.update(status="supervisor_failed", supervisor_error_type=type(error).__name__)
        except KeyboardInterrupt:
            receipt["status"] = "interrupted"
        finally:
            try:
                stop_child(process)
            except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
                receipt.update(status="drain_failed", drain_error_type=type(error).__name__)
            receipt["child_exit_code"] = None if process is None else process.returncode
    # Descendants can finish writes during the owned termination/drain phase.
    if log.stat().st_size > MAX_LOG_BYTES:
        receipt["status"] = "log_budget_exceeded"
    if receipt["status"] != "command_completed":
        receipt["exit_code"] = 130 if receipt["status"] == "interrupted" else 124
    receipt.update(elapsed_seconds=round(time.monotonic() - started, 3), fixture_health_checks=checks,
                   log_sha256=sha256(log), health_log=str(health_path), health_log_sha256=sha256(health_path))
    gate.write_json(receipt_path, receipt)
    return receipt


def copy_quiescent_target(source: Path, destination: Path) -> None:
    with (source / "debug/.cargo-lock").open("rb") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        shutil.copytree(source, destination)


def run_frozen(args, context: Path) -> int:
    import run_engine

    signal.signal(signal.SIGTERM, request_stop)
    campaign = args.campaign.resolve()
    scratch = args.scratch.resolve()
    manifest = gate.freeze(
        ROOT, campaign, ["inventory-engine"], gate.DEFAULT_TOOL,
        required_tests=gate.REQUIRED_LIVE_TESTS["inventory-engine"],
        required_environment=gate.REQUIRED_LIVE_ENVIRONMENT["inventory-engine"], include_ignored=True,
        fixture_receipt=context / "fixture-command.json", fixture_kind="falkordb_native",
        fixture_manifest=context / "fixture-manifest.json",
        inputs=[args.redis_server, args.falkor_module, Path(__file__), Path(run_engine.__file__), Path(gate.__file__),
                ROOT / "scripts/emulator/mutate.py"],
    )
    print(json.dumps({"phase": "frozen", "candidates": manifest["candidate_count"],
                      "source_digest": manifest["source_digest"]}), flush=True)
    snapshots = [scratch / f"workspace-{index}" for index in range(args.shards)]
    targets = [scratch / f"target-{index}" for index in range(args.shards)]
    for snapshot, target in zip(snapshots, targets):
        gate.snapshot_workspace(ROOT, snapshot, campaign)
        if args.seed_target:
            copy_quiescent_target(args.seed_target.resolve(), target)
    results = run_engine.run_workers(args, campaign, snapshots, targets)
    gate.write_json(campaign / "workers.json", results)
    gate.write_json(campaign / "pending-report.json",
                    gate.report_campaign(campaign, ROOT, allow_pending_fixture=True))
    return 0 if all(result["exit_code"] == 0 for result in results) else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--campaign", type=Path, required=True)
    parser.add_argument("--scratch", type=Path, required=True)
    parser.add_argument("--redis-server", type=Path, required=True)
    parser.add_argument("--falkor-module", type=Path, required=True)
    parser.add_argument("--seed-target", type=Path)
    parser.add_argument("--shards", type=int, default=4)
    parser.add_argument("--workers", type=int, default=1)
    parser.add_argument("--test-timeout", type=int, default=120)
    parser.add_argument("--build-timeout", type=int, default=600)
    parser.add_argument("--wall-timeout", type=int, default=21_600)
    parser.add_argument("--internal-run", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if not 1 <= args.workers <= args.shards <= 16:
        parser.error("require 1 <= workers <= shards <= 16")
    campaign = args.campaign.resolve()
    scratch = args.scratch.resolve()
    context = campaign.with_name(campaign.name + "-context")
    if args.internal_run:
        return run_frozen(args, context)
    if any(path.exists() for path in (campaign, scratch, context)):
        parser.error("campaign, scratch and context must all be new paths")
    if scratch == ROOT or scratch.is_relative_to(ROOT):
        parser.error("mutation scratch must be outside the authoring repository")
    signal.signal(signal.SIGTERM, request_stop)
    scratch.mkdir(parents=True)
    context.mkdir(parents=True)
    fixture = {
        "schema_version": 1, "fixture_kind": "falkordb_native",
        "redis_sha256": sha256(args.redis_server), "module_sha256": sha256(args.falkor_module),
        "test_sources": {name: sha256(ROOT / name) for name in
                         ("crates/engine/tests/graph.rs", "crates/engine/tests/graph_contracts.rs")},
        "graph_lifetime": "Each required test creates and deletes an isolated synthetic graph.",
        "server_memory_limit_bytes": 134_217_728,
    }
    fixture_path = context / "fixture-manifest.json"
    gate.write_json(fixture_path, fixture, exclusive=True)
    os.environ["INVENTORY_MUTATION_FIXTURE_SHA256"] = sha256(fixture_path)
    command = [sys.executable, str(Path(__file__).resolve()), *sys.argv[1:], "--internal-run"]
    sys.path.insert(0, str(ROOT / "scripts/emulator"))
    from mutate import falkor_server

    with falkor_server(args.redis_server, args.falkor_module, context):
        url = os.environ["FALKORDB_TEST_URL"]
        receipt = supervise(command, ROOT, context, url, args.wall_timeout)
    try:
        closed = endpoint_closed(url)
    except OSError as error:
        closed = False
        receipt["cleanup_error_type"] = type(error).__name__
    receipt["owned_server_context_closed"] = closed
    if receipt["status"] == "command_completed":
        receipt["status"] = "completed" if closed else "cleanup_failed"
    if not closed:
        receipt["exit_code"] = 124
    gate.write_json(context / "fixture-command.json", receipt)
    if receipt["status"] != "completed" or receipt["exit_code"] != 0:
        print(json.dumps({"phase": "native_engine", "status": receipt["status"],
                          "exit_code": receipt["exit_code"], "receipt": str(context / "fixture-command.json")}), flush=True)
        return receipt["exit_code"]
    report = gate.report_campaign(campaign, ROOT)
    gate.write_json(campaign / "report.json", report)
    print(json.dumps({"phase": "engine_report", "complete": report["complete"],
                      "score_percent": report["score_percent"], "counts": report["counts"],
                      "whole_project_gate": False}), flush=True)
    return 0 if report["complete"] and report["score_requirement_met"] else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130) from None
