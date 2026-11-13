#!/usr/bin/env python3
"""Run the complete engine mutation partition with a real local FalkorDB.

Each shard gets its own frozen checkout, target directory, and two clean
baselines. The shared Falkor process outlives every test child. A package report
does not establish the whole-project mutation gate.
"""

from __future__ import annotations

import argparse
from collections import deque
from contextlib import ExitStack
import json
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

import gate

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/emulator"))
from mutate import falkor_server


def request_stop(_signal, _frame):
    raise KeyboardInterrupt


def stop_workers(active):
    for process, _ in active:
        if process.poll() is None:
            process.send_signal(signal.SIGINT)
    deadline = time.monotonic() + 45
    for process, _ in active:
        try:
            process.wait(timeout=max(0.1, deadline - time.monotonic()))
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=10)


def run_workers(args, campaign, snapshots, targets):
    pending = deque(range(args.shards))
    active = []
    results = []
    deadline = time.monotonic() + args.wall_timeout
    with ExitStack() as files:
        try:
            while pending or active:
                if time.monotonic() >= deadline:
                    raise TimeoutError("engine mutation coordinator exceeded its wall deadline")
                while pending and len(active) < args.workers:
                    index = pending.popleft()
                    command = [
                        sys.executable, str(ROOT / "scripts/mutation/gate.py"), "run",
                        "--workspace", str(snapshots[index]), "--campaign", str(campaign),
                        "--target-dir", str(targets[index]), "--shard", f"{index}/{args.shards}",
                        "--timeout", str(args.test_timeout), "--build-timeout", str(args.build_timeout),
                        "--wall-timeout", str(args.wall_timeout),
                    ]
                    log = files.enter_context((campaign / f"shard-{index}.log").open("x"))
                    process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
                    active.append((process, index))
                    print(json.dumps({"phase": "shard_started", "shard": index, "pid": process.pid}), flush=True)
                for process, index in list(active):
                    code = process.poll()
                    if code is not None:
                        result = {"shard": index, "exit_code": code}
                        results.append(result)
                        active.remove((process, index))
                        print(json.dumps({"phase": "shard_finished", **result}), flush=True)
                if active:
                    time.sleep(0.25)
        finally:
            stop_workers(active)
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--campaign", type=Path, required=True)
    parser.add_argument("--scratch", type=Path, required=True)
    parser.add_argument("--redis-server", type=Path, required=True)
    parser.add_argument("--falkor-module", type=Path, required=True)
    parser.add_argument("--seed-target", type=Path)
    parser.add_argument("--shards", type=int, default=4)
    parser.add_argument("--workers", type=int, default=2)
    parser.add_argument("--test-timeout", type=int, default=120)
    parser.add_argument("--build-timeout", type=int, default=600)
    parser.add_argument("--wall-timeout", type=int, default=21_600)
    args = parser.parse_args()
    if not 1 <= args.workers <= args.shards <= 16:
        parser.error("require 1 <= workers <= shards <= 16")
    campaign = args.campaign.resolve()
    scratch = args.scratch.resolve()
    if campaign.exists() or scratch.exists():
        parser.error("campaign and scratch must both be new paths")
    if scratch == ROOT or scratch.is_relative_to(ROOT):
        parser.error("mutation scratch must be outside the authoring repository")
    signal.signal(signal.SIGTERM, request_stop)
    scratch.mkdir(parents=True)
    context = scratch / "server"
    context.mkdir()
    snapshots = [scratch / f"workspace-{index}" for index in range(args.shards)]
    targets = [scratch / f"target-{index}" for index in range(args.shards)]
    with falkor_server(args.redis_server, args.falkor_module, context):
        manifest = gate.freeze(
            ROOT, campaign, ["inventory-engine"], gate.DEFAULT_TOOL,
            required_tests=["live_falkordb_replay_and_parameter_round_trip"],
            required_environment=["FALKORDB_TEST_URL"],
            inputs=[args.redis_server, args.falkor_module, Path(__file__)],
            include_ignored=True,
        )
        print(json.dumps({"phase": "frozen", "candidates": manifest["candidate_count"],
                          "source_digest": manifest["source_digest"]}), flush=True)
        for snapshot, target in zip(snapshots, targets):
            gate.snapshot_workspace(ROOT, snapshot, campaign)
            if args.seed_target:
                shutil.copytree(args.seed_target.resolve(), target)
        results = run_workers(args, campaign, snapshots, targets)
        gate.write_json(campaign / "workers.json", results)
        report = gate.report_campaign(campaign, ROOT)
        gate.write_json(campaign / "report.json", report)
        print(json.dumps({"phase": "engine_report", "complete": report["complete"],
                          "score_percent": report["score_percent"], "counts": report["counts"],
                          "whole_project_gate": False}), flush=True)
    shutil.copy2(context / "falkor-server.log", campaign / "falkor-server.log")
    return 0 if report["complete"] and report["score_requirement_met"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
