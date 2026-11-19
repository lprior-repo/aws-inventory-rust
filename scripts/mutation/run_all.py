#!/usr/bin/env python3
"""Run every first-party mutation serially inside one verified fixture lifetime.

`launch` owns the emulator/Falkor wrapper, a filesystem stop hook, and the final
strict aggregate. `execute` is its internal child entrypoint and never declares
the whole-project gate passed while the outer fixture receipt remains open.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

import gate


REQUIRED_TESTS = gate.REQUIRED_LIVE_TESTS
REQUIRED_ENVIRONMENT = (*gate.AWS_FIXTURE_ENVIRONMENT, "FALKORDB_TEST_URL")
MAX_LAUNCH_LOG_BYTES = 128 * 1024 * 1024


def stop_owned_wrapper(process: subprocess.Popen, grace=45) -> bool:
    """Allow wrapper -> gate -> cargo-mutants cancellation and owned cleanup."""
    if process.poll() is not None:
        return True
    process.send_signal(signal.SIGINT)
    try:
        process.wait(timeout=grace)
        return True
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=5)
        return False
    except ProcessLookupError:
        process.wait(timeout=5)
        return True


def supervise(command, workspace: Path, environment: dict, log: Path, stop_file: Path,
              timeout: int) -> dict:
    """Bound one owned wrapper and accept stop requests across tool namespaces."""
    requested_signal = None

    def request_stop(number, _frame):
        nonlocal requested_signal
        requested_signal = number

    previous = {number: signal.getsignal(number) for number in (signal.SIGINT, signal.SIGTERM)}
    process = None
    started = time.monotonic()
    result = {"status": "running", "started_at": gate.now(), "stop_file": str(stop_file)}
    try:
        for number in previous:
            signal.signal(number, request_stop)
        with log.open("x") as output:
            process = subprocess.Popen(command, cwd=workspace, env=environment, stdin=subprocess.DEVNULL,
                                       stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
            next_progress = time.monotonic() + 30
            while process.poll() is None:
                if requested_signal is not None or stop_file.exists():
                    result.update(status="stop_requested", requested_signal=requested_signal)
                    break
                if time.monotonic() - started >= timeout:
                    result["status"] = "deadline_exceeded"
                    break
                if log.stat().st_size > MAX_LAUNCH_LOG_BYTES:
                    result["status"] = "log_budget_exceeded"
                    break
                if time.monotonic() >= next_progress:
                    print(json.dumps({"phase": "full_campaign", "elapsed_seconds": round(time.monotonic() - started),
                                      "stop_file": str(stop_file)}), flush=True)
                    next_progress = time.monotonic() + 30
                time.sleep(0.1)
            if result["status"] == "running":
                result.update(status="completed", exit_code=process.returncode)
    finally:
        try:
            if process is not None:
                result["graceful_owned_drain"] = stop_owned_wrapper(process)
                result.setdefault("exit_code", process.returncode)
        finally:
            for number, handler in previous.items():
                signal.signal(number, handler)
            result.update(finished_at=gate.now(), seconds=round(time.monotonic() - started, 3))
            # A fast child can exit between polls, or write during its drain.
            # Final evidence must enforce the same budgets as the active loop.
            result["log_bytes"] = log.stat().st_size if log.exists() else 0
            if result["log_bytes"] > MAX_LAUNCH_LOG_BYTES:
                result["status"] = "log_budget_exceeded"
            elif result["status"] == "completed" and time.monotonic() - started >= timeout:
                result["status"] = "deadline_exceeded"
            elif result["status"] == "completed" and (requested_signal is not None or stop_file.exists()):
                result["status"] = "stop_requested"
    return result


def execute(args) -> int:
    if os.environ.get("INVENTORY_EMULATOR_TEST") != "1":
        raise gate.GateError("execute requires the verified local emulator/Falkor fixture context")
    for name in REQUIRED_ENVIRONMENT:
        if not os.environ.get(name):
            raise gate.GateError(f"required fixture context variable is missing: {name}")
    snapshot_input = Path(os.environ["INVENTORY_EMULATOR_SNAPSHOT_PATH"]).resolve()
    if gate.digest(snapshot_input) != os.environ["INVENTORY_EMULATOR_SNAPSHOT_SHA256"]:
        raise gate.GateError("fixture snapshot hash differs from its verified context")
    fixture = gate.load_json(snapshot_input)
    summary_input = Path(os.environ["INVENTORY_EMULATOR_BASELINE_SUMMARY"]).resolve()
    if gate.digest(summary_input) != fixture["provenance"]["summary_sha256"]:
        raise gate.GateError("independent fixture summary differs from its frozen snapshot")
    if not (196 <= args.timeout <= 3600 and 1 <= args.build_timeout <= 3600
            and 1 <= args.wall_timeout <= 604800):
        raise gate.GateError("bounded test deadline must exceed the195-second collector cancellation deadline")
    workspace, campaign = args.workspace.resolve(), args.campaign.resolve()
    manifest = gate.freeze(
        workspace, campaign, gate.PACKAGES, args.tool.resolve(),
        required_tests_by_package=REQUIRED_TESTS, required_environment=REQUIRED_ENVIRONMENT,
        inputs=[snapshot_input, summary_input, Path(__file__).resolve(),
                Path(gate.__file__).resolve(), workspace / "scripts/emulator/mutate.py"],
        include_ignored=True, fixture_receipt=snapshot_input.parent / "mutation-command.json",
    )
    gate.snapshot_workspace(workspace, args.snapshot.resolve(), campaign)
    print(json.dumps({"status": "frozen", "candidate_count": manifest["candidate_count"],
                      "packages": manifest["packages"], "source_digest": manifest["source_digest"],
                      "required_tests_by_package": REQUIRED_TESTS, "mutation_workers": 1,
                      "cargo_jobs": 1, "manifest": str(campaign / "manifest.json")}, sort_keys=True), flush=True)
    execution = gate.run_partition(campaign, args.snapshot.resolve(), args.tool.resolve(), None,
                                   args.target_dir.resolve(), args.timeout, args.build_timeout, args.wall_timeout)
    report = gate.report_campaign(campaign, workspace, allow_pending_fixture=True)
    gate.write_json(campaign / "pending-report.json", report)
    gate.write_json(campaign / "execution-summary.json", {
        "execution": execution, "candidate_count": report["candidate_count"], "counts": report["counts"],
        "score_percent": report["score_percent"], "errors": report["errors"], "gate_passed": False,
        "semantics": "Fixture lifetime is still open; only the parent's later strict aggregate can pass.",
    })
    return 0 if execution["source_restored"] and execution["exit_code"] in (0, 2, 3) and not report["errors"] else 2


def launch(args) -> int:
    workspace, campaign = args.workspace.resolve(), args.campaign.resolve()
    snapshot, target = args.snapshot.resolve(), args.target_dir.resolve()
    fixture_output = args.fixture_output.resolve()
    stop_file = (args.stop_file or campaign.with_name(campaign.name + ".STOP")).resolve()
    receipt_path = campaign.with_name(campaign.name + "-launcher.json")
    log = campaign.with_name(campaign.name + "-launcher.log")
    for path in (campaign, snapshot, fixture_output, stop_file, receipt_path, log):
        if path.exists():
            raise gate.GateError(f"preserve existing evidence/control paths; choose a new path: {path}")
    if target == workspace or target.is_relative_to(workspace):
        raise gate.GateError("use a dedicated target outside the authoring workspace")
    if not (196 <= args.timeout <= 3600 and 1 <= args.build_timeout <= 3600
            and 1 <= args.wall_timeout <= 600000):
        raise gate.GateError("full-run timeouts exceed their bounds or undercut the collector deadline")
    campaign.parent.mkdir(parents=True, exist_ok=True)
    stop_file.parent.mkdir(parents=True, exist_ok=True)
    command = [
        sys.executable, str(workspace / "scripts/emulator/mutate.py"),
        "--emulator", str(args.emulator.resolve()), "--binary", str(args.binary.resolve()),
        "--branch-probe", str(args.branch_probe.resolve()), "--output", str(fixture_output),
        "--workspace", str(workspace), "--redis-server", str(args.redis_server.resolve()),
        "--falkor-module", str(args.falkor_module.resolve()),
        "--command-deadline", str(args.wall_timeout + 1200), "--",
        sys.executable, str(Path(__file__).resolve()), "execute",
        "--workspace", str(workspace), "--campaign", str(campaign), "--snapshot", str(snapshot),
        "--target-dir", str(target), "--tool", str(args.tool.resolve()),
        "--timeout", str(args.timeout), "--build-timeout", str(args.build_timeout),
        "--wall-timeout", str(args.wall_timeout),
    ]
    environment = gate.run_environment({"CARGO_TARGET_DIR": str(target)})
    receipt = {
        "status": "running", "started_at": gate.now(), "command": command, "workspace": str(workspace),
        "log": str(log), "stop_file": str(stop_file), "target_directory": str(target),
        "environment": {**gate.ENVIRONMENT, "CARGO_TARGET_DIR": str(target)},
        "runner_sha256": gate.digest(Path(__file__).resolve()), "gate_sha256": gate.digest(Path(gate.__file__).resolve()),
        "mutation_workers": 1, "cargo_jobs": 1, "full_packages": list(gate.PACKAGES),
    }
    gate.write_json(receipt_path, receipt, exclusive=True)
    print(json.dumps({"phase": "starting_full_campaign", "receipt": str(receipt_path),
                      "stop_file": str(stop_file), "log": str(log)}), flush=True)
    result = supervise(command, workspace, environment, log, stop_file, args.wall_timeout + 3600)
    receipt.update(result, log_sha256=(gate.digest(log, MAX_LAUNCH_LOG_BYTES)
                                      if log.stat().st_size <= MAX_LAUNCH_LOG_BYTES else None))
    gate.write_json(receipt_path, receipt)
    if result["status"] != "completed" or result["exit_code"] != 0 or stop_file.exists():
        print(json.dumps({"status": "execution_incomplete", "gate_passed": False,
                          "receipt": str(receipt_path), "reason": result["status"]}), flush=True)
        return 2
    # The fixture wrapper is closed now. The aggregate re-enumerates every
    # first-party candidate and verifies the completed outer receipt and hashes.
    previous_target = os.environ.get("CARGO_TARGET_DIR")
    os.environ["CARGO_TARGET_DIR"] = str(target)
    try:
        report = gate.aggregate(workspace, [campaign], args.tool.resolve(), campaign / "final-universe.json")
    finally:
        if previous_target is None:
            os.environ.pop("CARGO_TARGET_DIR", None)
        else:
            os.environ["CARGO_TARGET_DIR"] = previous_target
    gate.write_json(campaign / "final-report.json", report)
    print(json.dumps({"status": "evaluated", "candidate_count": report["candidate_count"],
                      "counts": report["counts"], "score_percent": report["score_percent"],
                      "gate_passed": report["gate_passed"], "errors": report["errors"],
                      "report": str(campaign / "final-report.json")}, sort_keys=True), flush=True)
    return 0 if report["gate_passed"] else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    for name in ("launch", "execute"):
        child = sub.add_parser(name)
        for field in ("workspace", "campaign", "snapshot", "target-dir"):
            child.add_argument("--" + field, type=Path, required=True)
        child.add_argument("--tool", type=Path, default=gate.DEFAULT_TOOL)
        child.add_argument("--timeout", type=int, default=240)
        child.add_argument("--build-timeout", type=int, default=600)
        child.add_argument("--wall-timeout", type=int, default=86400)
        if name == "launch":
            for field in ("fixture-output", "emulator", "binary", "branch-probe", "redis-server", "falkor-module"):
                child.add_argument("--" + field, type=Path, required=True)
            child.add_argument("--stop-file", type=Path)
    args = parser.parse_args()
    try:
        return launch(args) if args.mode == "launch" else execute(args)
    except (gate.GateError, OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        parser.exit(2, f"full mutation execution failed closed: {error}\n")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
