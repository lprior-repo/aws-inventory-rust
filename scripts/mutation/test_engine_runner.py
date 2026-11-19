#!/usr/bin/env python3
"""Real child-process and loopback-protocol checks for the native supervisor."""

from __future__ import annotations

from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import signal
import socketserver
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import run_engine_verified as runner


PING = b"*1\r\n$4\r\nPING\r\n"


@contextmanager
def pong_server(responses=(b"+PONG\r\n",)):
    """Return bounded real RESP replies; never substitute a mocked health call."""
    observed = []
    errors = []

    class Handler(socketserver.BaseRequestHandler):
        def handle(self):
            self.request.settimeout(1)
            request = bytearray()
            try:
                while len(request) < len(PING):
                    fragment = self.request.recv(len(PING) - len(request))
                    if not fragment:
                        break
                    request.extend(fragment)
                if not request:
                    return
                observed.append(bytes(request))
                if request != PING:
                    errors.append("unexpected health command")
                    return
                response = responses[min(len(observed) - 1, len(responses) - 1)]
                fragments = response if isinstance(response, tuple) else (response,)
                for fragment in fragments:
                    self.request.sendall(fragment)
                    if len(fragments) > 1:
                        time.sleep(0.001)
            except OSError as error:
                errors.append(type(error).__name__)

    with socketserver.TCPServer(("127.0.0.1", 0), Handler) as server:
        thread = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.01})
        thread.start()
        try:
            yield f"redis://127.0.0.1:{server.server_address[1]}", observed
        finally:
            server.shutdown()
            thread.join(timeout=2)
            if thread.is_alive():
                raise AssertionError("synthetic health server did not stop")
            if errors:
                raise AssertionError(f"synthetic health server errors: {errors}")


def command(source: str, *args: Path) -> list[str]:
    return [sys.executable, "-u", "-c", source, *map(str, args)]


def wait_for(path: Path, seconds=3):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if path.is_file():
            return
        time.sleep(0.01)
    raise AssertionError("owned test child did not reach its readiness checkpoint")


def assert_group_absent(test: unittest.TestCase, pid: int):
    with test.assertRaises(ProcessLookupError):
        os.killpg(pid, 0)


class EngineSupervisorTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="engine-supervisor-tests-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.context = self.root / "context"
        self.context.mkdir()
        self.patches = patch.multiple(
            runner, GROUP_DRAIN_SECONDS=1, FORCED_DRAIN_SECONDS=1,
        )
        self.patches.start()
        self.addCleanup(self.patches.stop)

    def supervise(self, source, url, *args, deadline=5):
        return runner.supervise(command(source, *args), self.root, self.context, url, deadline)

    def check_receipt(self, receipt):
        self.assertEqual(json.loads((self.context / "fixture-command.json").read_text()), receipt)
        for filename, key in (("fixture-command.log", "log_sha256"), ("health.jsonl", "health_log_sha256")):
            self.assertEqual(hashlib.sha256((self.context / filename).read_bytes()).hexdigest(), receipt[key])
        return [json.loads(line) for line in (self.context / "health.jsonl").read_text().splitlines()]

    def test_endpoint_contract_rejects_remote_credentials_and_extra_components(self):
        self.assertEqual(runner.endpoint_port("redis://127.0.0.1:6379"), 6379)
        self.assertEqual(runner.endpoint_port("redis://127.0.0.1:1/"), 1)
        for endpoint in (
            "redis://localhost:6379", "redis://[::1]:6379", "redis://example.com:6379",
            "rediss://127.0.0.1:6379", "redis://127.0.0.1", "redis://127.0.0.1:0",
            "redis://127.0.0.1:65536", "redis://user@127.0.0.1:6379",
            "redis://:secret@127.0.0.1:6379", "redis://127.0.0.1:6379/0",
            "redis://127.0.0.1:6379?x=1", "redis://127.0.0.1:6379#fragment",
        ):
            with self.subTest(endpoint=endpoint), self.assertRaises(ValueError):
                runner.endpoint_port(endpoint)

    def test_fragmented_pong_and_server_closure_use_real_sockets(self):
        with pong_server(((b"+", b"PO", b"NG", b"\r", b"\n"),)) as (url, observed):
            runner.probe_health(url)
            self.assertEqual(observed, [PING])
            self.assertFalse(runner.endpoint_closed(url))
        self.assertTrue(runner.endpoint_closed(url))

    def test_success_is_pending_outer_cleanup_and_hashes_exact_logs(self):
        with pong_server() as (url, _):
            receipt = self.supervise("print('owned child completed')", url)
        events = self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "command_completed")
        self.assertEqual(receipt["exit_code"], 0)
        self.assertEqual(receipt["child_exit_code"], 0)
        self.assertGreaterEqual(receipt["fixture_health_checks"], 2)
        self.assertNotIn("owned_server_context_closed", receipt)
        self.assertEqual([event["phase"] for event in events], ["before", "after"])
        self.assertTrue(all(event["passed"] for event in events))
        self.assertEqual((self.context / "fixture-command.log").read_text(), "owned child completed\n")

    def test_nonzero_child_exit_is_preserved(self):
        with pong_server() as (url, _):
            receipt = self.supervise("raise SystemExit(7)", url)
        self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "command_completed")
        self.assertEqual(receipt["exit_code"], 7)
        self.assertEqual(receipt["child_exit_code"], 7)

    def test_failed_initial_health_prevents_child_side_effects(self):
        marker = self.root / "child-ran"
        with pong_server((b"-DOWN\r\n",)) as (url, _):
            receipt = self.supervise("from pathlib import Path; import sys; Path(sys.argv[1]).touch()", url, marker)
        events = self.check_receipt(receipt)
        self.assertFalse(marker.exists())
        self.assertEqual(receipt["status"], "fixture_unavailable")
        self.assertEqual(receipt["exit_code"], 124)
        self.assertIsNone(receipt["child_exit_code"])
        self.assertEqual(receipt["fixture_health_checks"], 0)
        self.assertEqual(len(events), 1)
        self.assertFalse(events[0]["passed"])

    def test_periodic_health_failure_stops_the_real_owned_child(self):
        ready = self.root / "child-pid"
        terminated = self.root / "terminated"
        source = """
import os, signal, sys, time
from pathlib import Path
def stop(*_):
    Path(sys.argv[2]).write_text('terminated')
    raise SystemExit(0)
signal.signal(signal.SIGTERM, stop)
Path(sys.argv[1]).write_text(str(os.getpid()))
time.sleep(30)
"""
        with pong_server((b"+PONG\r\n", b"-DOWN\r\n")) as (url, _), patch.object(runner, "HEALTH_INTERVAL_SECONDS", 0.02):
            receipt = self.supervise(source, url, ready, terminated)
        events = self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "fixture_unavailable")
        self.assertEqual(receipt["exit_code"], 124)
        self.assertEqual(terminated.read_text(), "terminated")
        self.assertEqual(receipt["fixture_health_checks"], 1)
        self.assertEqual(events[-1]["phase"], "periodic")
        self.assertFalse(events[-1]["passed"])
        assert_group_absent(self, int(ready.read_text()))

    def test_failed_final_health_overrides_a_zero_exit(self):
        with pong_server((b"+PONG\r\n", b"-DOWN\r\n")) as (url, _), patch.object(runner, "HEALTH_INTERVAL_SECONDS", 60):
            receipt = self.supervise("raise SystemExit(0)", url)
        events = self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "fixture_unavailable")
        self.assertEqual(receipt["exit_code"], 124)
        self.assertEqual(receipt["child_exit_code"], 0)
        self.assertEqual(events[-1]["phase"], "after")
        self.assertFalse(events[-1]["passed"])

    def test_filesystem_stop_drains_without_claiming_success(self):
        ready = self.root / "child-pid"
        source = """
import os, signal, sys, time
from pathlib import Path
signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
Path(sys.argv[1]).write_text(str(os.getpid()))
Path(sys.argv[2]).touch()
time.sleep(30)
"""
        with pong_server() as (url, _):
            receipt = self.supervise(source, url, ready, self.context / "STOP")
        self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "interrupted")
        self.assertEqual(receipt["exit_code"], 130)
        assert_group_absent(self, int(ready.read_text()))

    def test_deadline_stops_the_real_child(self):
        with pong_server() as (url, _):
            receipt = self.supervise("import time; time.sleep(30)", url, deadline=0.05)
        self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "timeout")
        self.assertEqual(receipt["exit_code"], 124)
        self.assertLess(receipt["elapsed_seconds"], 4)

    def test_fast_child_cannot_bypass_log_cap_by_exiting_between_polls(self):
        with pong_server() as (url, _), patch.object(runner, "MAX_LOG_BYTES", 32):
            receipt = self.supervise("import sys; sys.stdout.write('x' * 33); sys.stdout.flush()", url)
        self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "log_budget_exceeded")
        self.assertEqual(receipt["exit_code"], 124)
        self.assertEqual(receipt["child_exit_code"], 0)

    def test_log_growth_during_drain_cannot_bypass_budget(self):
        source = """
import signal, sys, time
from pathlib import Path
def stop(*_):
    sys.stdout.write('x' * 65)
    sys.stdout.flush()
    raise SystemExit(0)
signal.signal(signal.SIGTERM, stop)
Path(sys.argv[1]).touch()
time.sleep(30)
"""
        with pong_server() as (url, _), patch.object(runner, "MAX_LOG_BYTES", 64):
            receipt = self.supervise(source, url, self.context / "STOP")
        self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "log_budget_exceeded")
        self.assertEqual(receipt["exit_code"], 124)
        self.assertGreater((self.context / "fixture-command.log").stat().st_size, 64)

    def test_stop_child_drains_a_real_group_with_a_grandchild(self):
        ready = self.root / "group-ready.json"
        source = """
import json, os, signal, subprocess, sys, time
from pathlib import Path
child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'])
def stop(*_):
    child.wait(timeout=2)
    raise SystemExit(0)
signal.signal(signal.SIGTERM, stop)
Path(sys.argv[1]).write_text(json.dumps([os.getpid(), child.pid]))
time.sleep(30)
"""
        process = subprocess.Popen(command(source, ready), cwd=self.root, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        try:
            wait_for(ready)
            leader, child = json.loads(ready.read_text())
            self.assertEqual(leader, process.pid)
            self.assertEqual(os.getpgid(child), leader)
            runner.stop_child(process)
            self.assertEqual(process.returncode, 0)
            assert_group_absent(self, leader)
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=2)


if __name__ == "__main__":
    unittest.main()
