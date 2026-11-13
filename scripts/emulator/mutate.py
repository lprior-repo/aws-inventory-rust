#!/usr/bin/env python3
"""Keep checked local AWS/Falkor fixtures alive while running a mutation command.

The emulator, fixture seeding, two unmodified Cargo snapshot runs, and the
supplied command share one process/network lifetime. Fixture creation is local
and synthetic. No user credentials or cloud accounts are used.

Example (check the integration oracle before starting mutations):
  python scripts/emulator/mutate.py --emulator /path/aws-emulator \
    --binary /path/unmodified/aws-inventory \
    --branch-probe /path/unmodified/emulator_branches --output /tmp/oracle-run

Append -- and a mutation-gate command to run it after both baselines pass.
Use --redis-server and --falkor-module together when including the ignored
FalkorDB test. The wrapper never changes the requested candidate selection.
"""

from __future__ import annotations

import argparse
from contextlib import contextmanager, nullcontext
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import ssl
import subprocess
import tempfile
import time
import urllib.request

import run
from snapshot import freeze

MAX_COMMAND_ARGUMENTS = 128
MAX_COMMAND_ARGUMENT_BYTES = 32_768
MAX_LOG_BYTES = 128 * 1024 * 1024
COMMAND_DRAIN_SECONDS = 35
FORCED_DRAIN_SECONDS = 5
CHILD_DRAIN_RESERVE_SECONDS = 27
DRAIN_ENVIRONMENT = "INVENTORY_COMMAND_DRAIN_SECONDS"


class CommandInterrupted(BaseException):
    """Interrupt a blocking health probe without bypassing owned cleanup."""


class FixtureHealthError(RuntimeError):
    def __init__(self, stage):
        super().__init__(stage)
        self.stage = stage


def error_details(error):
    """Record bounded diagnostic categories, never raw URLs or exception text."""
    details = {"type": type(error).__name__[:96]}
    if isinstance(error, FixtureHealthError):
        details["stage"] = error.stage
        error = error.__cause__
        details["type"] = type(error).__name__[:96]
    reason = getattr(error, "reason", None)
    if isinstance(reason, BaseException):
        details["reason_type"] = type(reason).__name__[:96]
    else:
        reason = error
    for field in ("errno", "verify_code"):
        value = getattr(reason, field, None)
        if isinstance(value, int) and not isinstance(value, bool):
            details[field] = value
    code = getattr(error, "code", None)
    if isinstance(code, int) and 100 <= code <= 599:
        details["http_status"] = code
    return details


@contextmanager
def command_signals():
    previous = {number: signal.getsignal(number) for number in (signal.SIGINT, signal.SIGTERM)}
    state = {"signal": None, "interruptible": False}

    def cancel(number, _frame):
        if state["signal"] is None:
            state["signal"] = number
        if state["interruptible"]:
            state["interruptible"] = False
            raise CommandInterrupted()

    try:
        for number in previous:
            signal.signal(number, cancel)
        yield state
    finally:
        for number, handler in previous.items():
            signal.signal(number, handler)


def command_drain_budget():
    value = os.environ.get(DRAIN_ENVIRONMENT, str(COMMAND_DRAIN_SECONDS))
    if not value.isascii() or not value.isdigit() or len(value) > 2:
        raise ValueError("invalid command drain budget")
    budget = int(value)
    if not FORCED_DRAIN_SECONDS < budget <= COMMAND_DRAIN_SECONDS:
        raise ValueError("command drain budget is invalid or nested supervision is exhausted")
    return budget


def freeze_config_recorder(fixture, session, recorder):
    """Keep provider timestamps exact by stopping only the synthetic recorder.

    The pinned emulator's background recorder observes SQS again during later
    reads. A campaign must not count that moving fixture as a killed mutant.
    Wait for all seeded records, stop this fixture's recorder, and independently
    confirm stable query results before the ordinary collector creates its oracle.
    """
    from botocore.config import Config
    from seed_discovery import CONFIG_AGGREGATOR, PREFIX
    recorder.set_phase("seed:freeze-config")
    client = session.client("config", config=Config(
        retries={"total_max_attempts": 1}, connect_timeout=5, read_timeout=10))

    def records():
        response = client.select_aggregate_resource_config(
            ConfigurationAggregatorName=CONFIG_AGGREGATOR, MaxResults=100,
            Expression="SELECT accountId, awsRegion, arn, resourceId, resourceName, resourceType, configuration, tags, configurationItemCaptureTime")
        if response.get("NextToken"):
            raise RuntimeError("mutation fixture exceeded its expected Config row budget")
        return sorted((json.loads(row) for row in response.get("Results", [])), key=lambda row: row["resourceId"])

    rows = []
    for _ in range(50):
        rows = records()
        if len(rows) >= 11:
            break
        time.sleep(0.2)
    if len(rows) < 11:
        raise RuntimeError(f"Config fixture captured fewer than eleven seeded records: {len(rows)} rows")
    name = f"{PREFIX}-recorder"
    stopped = fixture.record("mutation_fixture:StopConfigurationRecorder", lambda:
        client.stop_configuration_recorder(ConfigurationRecorderName=name))
    if stopped is None:
        raise RuntimeError("could not freeze the synthetic Config recorder")
    statuses = client.describe_configuration_recorder_status(ConfigurationRecorderNames=[name])
    status = statuses.get("ConfigurationRecordersStatus", [])
    stopped = len(status) == 1 and status[0].get("recording") is False
    fixture.check("fixture:config_recorder_stopped", stopped)
    first = records()
    time.sleep(0.3)
    second = records()
    stable = len(first) >= 11 and first == second
    fixture.check("fixture:config_snapshot_stable", stable)
    run.write_json(fixture.output / "config-freeze.json", {
        "recorder": name, "stopped": stopped, "stable": stable, "resources": len(second),
        "configuration_sha256": hashlib.sha256(json.dumps(second, sort_keys=True).encode()).hexdigest(),
        "normalization": "none; provider capture timestamps remain exact",
    })
    run.write_json(fixture.output / "seed-results.json", {"modules": fixture.seed_reports, "calls": fixture.seed_calls})
    if not stopped or not stable:
        raise RuntimeError("synthetic Config recorder did not produce a stable frozen snapshot")


def stop(process: subprocess.Popen, budget=COMMAND_DRAIN_SECONDS) -> dict:
    """Drain the owned group, including members left after its leader exits."""
    started = time.monotonic()

    def drained():
        process.poll()  # Reap the direct child before testing group existence.
        try:
            os.killpg(process.pid, 0)
            return False
        except ProcessLookupError:
            return True

    def wait_until(deadline):
        while time.monotonic() < deadline:
            if drained():
                return True
            time.sleep(min(0.05, max(0, deadline - time.monotonic())))
        return drained()

    if drained():
        return {"drained": True, "forced": False}
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    if wait_until(started + budget - FORCED_DRAIN_SECONDS):
        return {"drained": True, "forced": False}
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    return {"drained": wait_until(started + budget), "forced": True}


def fixture_health() -> None:
    try:
        endpoint = os.environ["AWS_ENDPOINT_URL"]
        if not endpoint.startswith("https://127.0.0.1:"):
            raise RuntimeError("fixture health endpoint is not loopback")
        context = ssl.create_default_context(cafile=os.environ["SSL_CERT_FILE"])
        with urllib.request.urlopen(endpoint + "/__admin/health", context=context, timeout=3) as response:
            if response.status != 200:
                raise RuntimeError("AWS Emulator health check did not return success")
            response.read(4_097)
    except (OSError, ValueError, RuntimeError, KeyError) as error:
        raise FixtureHealthError("aws_tls_health") from error
    if "FALKORDB_TEST_URL" in os.environ:
        try:
            address = os.environ["FALKORDB_TEST_URL"]
            prefix = "redis://127.0.0.1:"
            if not address.startswith(prefix):
                raise RuntimeError("FalkorDB fixture health endpoint is not loopback")
            port = int(address[len(prefix):])
            if not 1 <= port <= 65_535:
                raise RuntimeError("FalkorDB fixture health port is invalid")
            with socket.create_connection(("127.0.0.1", port), timeout=3) as connection:
                connection.sendall(b"*1\r\n$4\r\nPING\r\n")
                if connection.recv(64) != b"+PONG\r\n":
                    raise RuntimeError("FalkorDB fixture did not return PONG")
        except (OSError, ValueError, RuntimeError) as error:
            raise FixtureHealthError("falkordb_ping") from error


def execute_command(arguments, workspace: Path, output: Path, label: str, deadline: int) -> dict:
    if not arguments or len(arguments) > MAX_COMMAND_ARGUMENTS:
        raise ValueError("command argument count is outside its bound")
    if sum(len(os.fsencode(argument)) for argument in arguments) > MAX_COMMAND_ARGUMENT_BYTES:
        raise ValueError("command arguments exceed their byte budget")
    log = output / f"{label}.log"
    receipt = {"argv": arguments, "workspace": str(workspace), "deadline_seconds": deadline,
               "status": "running", "log": str(log)}
    run.write_json(output / f"{label}.json", receipt)
    started = time.monotonic()
    next_progress, next_health, health_checks = started + 30, started, 0
    process, budget, phase = None, COMMAND_DRAIN_SECONDS, "budget"

    def check_health(phase):
        nonlocal health_checks
        probe_started = time.monotonic()
        try:
            fixture_health()
        except (OSError, ValueError, RuntimeError) as error:
            details = error_details(error)
            details.update(phase=phase, elapsed_seconds=round(time.monotonic() - probe_started, 3))
            receipt.update(status="fixture_unavailable", fixture_error_type=details["type"],
                           fixture_error=details)
            return False
        health_checks += 1
        return True

    with command_signals() as cancellation:
        try:
            with log.open("xb") as stream:
                budget = command_drain_budget()
                # A nested supervisor must finish before this owner's SIGKILL.
                # The 35-second outer budget gives 30 seconds of grace; the
                # nested command gets 8 seconds total, leaving 22 seconds for
                # cancellation detection and terminal receipt publication.
                environment = os.environ.copy()
                child_budget = max(0, budget - CHILD_DRAIN_RESERVE_SECONDS)
                environment[DRAIN_ENVIRONMENT] = str(child_budget)
                receipt.update(cleanup_budget_seconds=budget, child_cleanup_budget_seconds=child_budget)
                if cancellation["signal"] is not None:
                    raise CommandInterrupted()
                phase = "spawn"
                process = subprocess.Popen(arguments, cwd=workspace, env=environment,
                                           stdin=subprocess.DEVNULL, stdout=stream,
                                           stderr=subprocess.STDOUT, start_new_session=True)
                receipt.update(pid=process.pid, process_group=process.pid)
                run.write_json(output / f"{label}.json", receipt)
                phase = "execute"
                cancellation["interruptible"] = True
                if cancellation["signal"] is not None:
                    raise CommandInterrupted()
                while process.poll() is None:
                    now = time.monotonic()
                    if now - started >= deadline:
                        receipt["status"] = "timeout"
                        break
                    if log.stat().st_size > MAX_LOG_BYTES:
                        receipt["status"] = "log_budget_exceeded"
                        break
                    if now >= next_health:
                        if not check_health("periodic"):
                            break
                        next_health = time.monotonic() + 5
                    if now >= next_progress:
                        print(json.dumps({"phase": label, "elapsed_seconds": round(now - started)}), flush=True)
                        next_progress = now + 30
                    time.sleep(0.25)
                # A short-lived child may exit between polls. Final health and
                # bounds still apply, and cancellation cannot become success.
                if receipt["status"] == "running" and time.monotonic() - started >= deadline:
                    receipt["status"] = "timeout"
                if receipt["status"] == "running" and log.stat().st_size > MAX_LOG_BYTES:
                    receipt["status"] = "log_budget_exceeded"
                if receipt["status"] == "running" and check_health("final"):
                    receipt.update(status="completed", exit_code=process.returncode)
        except (CommandInterrupted, KeyboardInterrupt):
            if cancellation["signal"] is None:
                cancellation["signal"] = signal.SIGINT
        except Exception as error:
            receipt.update(status="supervisor_error", supervisor_error={"phase": phase, **error_details(error)})
        finally:
            # Further signals record cancellation but cannot interrupt cleanup or
            # prevent the child/receipt from reaching a terminal state.
            cancellation["interruptible"] = False
            if process is not None:
                try:
                    receipt["cleanup"] = stop(process, budget)
                except Exception as error:
                    receipt["cleanup"] = {"drained": False, "error": error_details(error)}
                receipt["child_exit_code"] = process.returncode
                if not receipt["cleanup"]["drained"] and receipt["status"] in {"running", "completed"}:
                    receipt["status"] = "cleanup_failed"
            if cancellation["signal"] is not None:
                receipt["interrupt_signal"] = cancellation["signal"]
                if receipt["status"] in {"running", "completed"}:
                    receipt["status"] = "interrupted"
            # Cleanup output and final health belong to the same bounded run.
            if receipt["status"] == "completed":
                if time.monotonic() - started >= deadline:
                    receipt["status"] = "timeout"
                elif log.stat().st_size > MAX_LOG_BYTES:
                    receipt["status"] = "log_budget_exceeded"
            receipt["elapsed_seconds"] = round(time.monotonic() - started, 3)
            receipt["fixture_health_checks"] = health_checks
            if receipt["status"] != "completed":
                receipt["exit_code"] = (128 + receipt["interrupt_signal"]
                                        if receipt["status"] == "interrupted" else 124)
            with log.open("rb") as stream:
                receipt["log_sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
            run.write_json(output / f"{label}.json", receipt)
    print(json.dumps({"phase": label, "status": receipt["status"], "exit_code": receipt["exit_code"],
                      "elapsed_seconds": receipt["elapsed_seconds"]}), flush=True)
    return receipt


@contextmanager
def falkor_server(binary: Path, module: Path, output: Path):
    with socket.socket() as reserved:
        reserved.bind(("127.0.0.1", 0))
        port = reserved.getsockname()[1]
    with tempfile.TemporaryDirectory(prefix="inventory-mutation-falkor-") as temporary:
        with (output / "falkor-server.log").open("xb") as log:
            process = subprocess.Popen([
                str(binary.resolve()), "--bind", "127.0.0.1", "--port", str(port),
                "--protected-mode", "yes", "--save", "", "--appendonly", "no",
                "--dir", temporary, "--loglevel", "warning", "--maxmemory", "134217728",
                "--loadmodule", str(module.resolve()),
            ], stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            primary_error = None
            try:
                ready = False
                for _ in range(100):
                    if process.poll() is not None:
                        raise RuntimeError("local FalkorDB exited before readiness")
                    try:
                        with socket.create_connection(("127.0.0.1", port), timeout=0.2) as connection:
                            connection.sendall(b"*1\r\n$4\r\nPING\r\n")
                            ready = connection.recv(64) == b"+PONG\r\n"
                    except OSError:
                        pass
                    if ready:
                        break
                    time.sleep(0.1)
                if not ready:
                    raise RuntimeError("local FalkorDB did not become ready within its deadline")
                previous = os.environ.get("FALKORDB_TEST_URL")
                os.environ["FALKORDB_TEST_URL"] = f"redis://127.0.0.1:{port}"
                try:
                    yield
                finally:
                    if previous is None:
                        os.environ.pop("FALKORDB_TEST_URL", None)
                    else:
                        os.environ["FALKORDB_TEST_URL"] = previous
            except BaseException as error:
                primary_error = error
                raise
            finally:
                try:
                    cleanup = stop(process)
                except Exception as error:
                    if primary_error is None:
                        raise RuntimeError("owned FalkorDB cleanup failed") from error
                    primary_error.add_note("owned FalkorDB cleanup raised " + type(error).__name__[:96])
                    cleanup = {"drained": False}
                if not cleanup["drained"]:
                    message = "owned FalkorDB process group did not drain within its cleanup budget"
                    if primary_error is not None:
                        primary_error.add_note(message)
                    else:
                        raise RuntimeError(message)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--emulator", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True, help="verified, unmodified collector binary")
    parser.add_argument("--branch-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new evidence directory")
    parser.add_argument("--workspace", type=Path, default=run.ROOT, help="unmodified Cargo checkout to baseline")
    parser.add_argument("--redis-server", type=Path)
    parser.add_argument("--falkor-module", type=Path)
    parser.add_argument("--command-deadline", type=int, default=86_400)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    args.seed_only = False
    args.output = args.output.resolve()
    args.workspace = args.workspace.resolve()
    if args.output.exists():
        parser.error("output must be a new directory; existing evidence is never overwritten")
    if bool(args.redis_server) != bool(args.falkor_module):
        parser.error("--redis-server and --falkor-module must be supplied together")
    if not 1 <= args.command_deadline <= 604_800:
        parser.error("command deadline must be between one second and seven days")
    # Long stretches of unit-test kills may make no AWS calls. The parent still
    # owns final cleanup, so an idle server cannot disappear midway through a
    # valid campaign and make network failures look like detected mutations.
    args.emulator_idle_timeout = args.command_deadline + 1_800
    command = args.command[1:] if args.command[:1] == ["--"] else args.command

    def checked_campaign(fixture, report, endpoint):
        receipt = freeze(fixture.output, fixture.output / "snapshot.json", args.binary)
        run.write_json(fixture.output / "snapshot-receipt.json", receipt)
        os.environ.update({
            "INVENTORY_EMULATOR_TEST": "1",
            "INVENTORY_EMULATOR_SNAPSHOT_PATH": receipt["snapshot"],
            "INVENTORY_EMULATOR_SNAPSHOT_SHA256": receipt["snapshot_sha256"],
            "INVENTORY_EMULATOR_BASELINE_SUMMARY": receipt["summary"],
            # Mutants must not add persisted counterexamples to the frozen tree
            # and thereby change later candidates' test inputs.
            "PROPTEST_DISABLE_FAILURE_PERSISTENCE": "1",
        })
        # Match ordinary validation's package/feature roots so selecting the
        # oracle does not discard shared features and rebuild the service SDKs.
        # The target selector still runs only the AWS snapshot integration test.
        baseline = ["cargo", "test", "--workspace", "--exclude", "inventory-sdk-catalog",
                    "--all-features", "--test", "emulator_snapshot",
                    "--locked", "--offline", "-j1", "--", "--ignored", "--test-threads=1"]
        server = (falkor_server(args.redis_server, args.falkor_module, fixture.output)
                  if args.redis_server else nullcontext())
        with server:
            for index in (1, 2):
                result = execute_command(baseline, args.workspace, fixture.output,
                                         f"oracle-baseline-{index}", 600)
                if result["exit_code"]:
                    return result["exit_code"]
            if not command:
                return 0
            result = execute_command(command, args.workspace, fixture.output, "mutation-command",
                                     args.command_deadline)
            return result["exit_code"]

    return run.execute(args, after_verified_run=checked_campaign, before_collection=freeze_config_recorder)


if __name__ == "__main__":
    raise SystemExit(main())
