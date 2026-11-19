"""Receipt regressions using real children and an injected health boundary."""

import errno
import hashlib
import json
import os
from pathlib import Path
import signal
import ssl
import subprocess
import sys
import tempfile
import threading
import time
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch
from urllib.error import URLError

sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutate


class OracleCallbackTests(unittest.TestCase):
    def invoke(self, statuses):
        with tempfile.TemporaryDirectory(prefix="mutation-oracle-callback-") as directory:
            root = Path(directory)
            output = root / "new-fixture"
            command = ["synthetic-mutation-worker", "--synthetic-argument"]
            arguments = ["mutate.py", "--emulator", str(root / "emulator"),
                         "--binary", str(root / "collector"), "--branch-probe", str(root / "probe"),
                         "--output", str(output), "--workspace", str(root),
                         "--command-deadline", "1800", "--", *command]
            snapshot = {"snapshot": str(output / "snapshot.json"), "snapshot_sha256": "a" * 64,
                        "summary": str(output / "summary.json")}

            def checked_fixture(args, *, after_verified_run, before_collection):
                self.assertIs(before_collection, mutate.freeze_config_recorder)
                args.output.mkdir()
                return after_verified_run(SimpleNamespace(output=args.output), {}, "https://127.0.0.1:1")

            with patch.object(sys, "argv", arguments), patch.dict(os.environ), \
                    patch.object(mutate.run, "execute", side_effect=checked_fixture), \
                    patch.object(mutate, "freeze", return_value=snapshot) as freeze, \
                    patch.object(mutate, "execute_command", side_effect=[{"exit_code": code} for code in statuses]) as execute, \
                    patch.object(mutate.subprocess, "Popen", side_effect=AssertionError("callback test must not run Cargo or services")):
                status = mutate.main()
                freeze.assert_called_once_with(output, output / "snapshot.json", root / "collector")
            return status, [call.args for call in execute.call_args_list], root, output, command

    def test_both_oracle_baselines_keep_workspace_features_and_snapshot_only_target(self):
        status, calls, root, output, command = self.invoke([0, 0, 0])
        expected = ["cargo", "test", "--workspace", "--exclude", "inventory-sdk-catalog",
                    "--all-features", "--test", "emulator_snapshot", "--locked", "--offline",
                    "-j1", "--", "--ignored", "--test-threads=1"]
        self.assertEqual(status, 0)
        self.assertEqual(calls, [
            (expected, root, output, "oracle-baseline-1", 600),
            (expected, root, output, "oracle-baseline-2", 600),
            (command, root, output, "mutation-command", 1800),
        ])

    def test_either_failed_oracle_stops_before_the_mutation_command(self):
        for statuses in ([124], [0, 101]):
            with self.subTest(statuses=statuses):
                status, calls, _root, _output, _command = self.invoke(statuses)
                self.assertEqual(status, statuses[-1])
                self.assertEqual([call[3] for call in calls],
                                 [f"oracle-baseline-{index}" for index in range(1, len(statuses) + 1)])
                self.assertTrue(all(call[4] == 600 for call in calls))


class CommandReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="mutation-receipt-tests-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def ready(self, path):
        deadline = time.monotonic() + 3
        while not path.exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        self.assertTrue(path.exists(), "real child failed to reach readiness")

    def check_receipt(self, receipt):
        self.assertEqual(json.loads((self.root / "command.json").read_text()), receipt)
        self.assertEqual(receipt["log_sha256"], hashlib.sha256((self.root / "command.log").read_bytes()).hexdigest())

    def test_keyboard_interrupt_records_130_and_drains_child(self):
        pid, stopped = self.root / "pid", self.root / "stopped"
        source = """
import os, signal, sys, time
from pathlib import Path
def stop(*_):
    Path(sys.argv[2]).touch()
    raise SystemExit(0)
signal.signal(signal.SIGTERM, stop)
Path(sys.argv[1]).write_text(str(os.getpid()))
time.sleep(30)
"""
        def interrupt():
            self.ready(pid)
            raise KeyboardInterrupt
        with patch.object(mutate, "fixture_health", side_effect=interrupt):
            receipt = mutate.execute_command([sys.executable, "-c", source, str(pid), str(stopped)], self.root, self.root, "command", 5)
        self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "interrupted")
        self.assertEqual(receipt["exit_code"], 130)
        self.assertTrue(stopped.exists())
        with self.assertRaises(ProcessLookupError):
            os.killpg(int(pid.read_text()), 0)

    def test_fast_child_above_log_cap_is_not_completed(self):
        with patch.object(mutate, "fixture_health", return_value=None), patch.object(mutate, "MAX_LOG_BYTES", 32):
            receipt = mutate.execute_command([sys.executable, "-c", "print('x' * 33)"], self.root, self.root, "command", 5)
        self.check_receipt(receipt)
        self.assertEqual(receipt["status"], "log_budget_exceeded")
        self.assertEqual(receipt["exit_code"], 124)

    def test_failed_final_health_overrides_zero_exit(self):
        ready = self.root / "ready"
        calls = 0
        def health():
            nonlocal calls
            calls += 1
            if calls == 1:
                self.ready(ready)
            else:
                raise RuntimeError("synthetic fixture unavailable")
        source = "from pathlib import Path; import sys,time; Path(sys.argv[1]).touch(); time.sleep(0.05)"
        with patch.object(mutate, "fixture_health", side_effect=health):
            receipt = mutate.execute_command([sys.executable, "-c", source, str(ready)], self.root, self.root, "command", 5)
        self.check_receipt(receipt)
        self.assertEqual(calls, 2)
        self.assertEqual(receipt["status"], "fixture_unavailable")
        self.assertEqual(receipt["exit_code"], 124)

    def test_health_failure_closes_nested_receipt_and_drains_separate_session(self):
        for ignore_term in (False, True):
            with self.subTest(ignore_term=ignore_term):
                root = self.root / str(ignore_term)
                root.mkdir()
                leaf = root / "leaf.py"
                leaf.write_text(
                    "import json, os, signal, sys, time\nfrom pathlib import Path\n"
                    "root = Path(sys.argv[1])\n"
                    "def stop(*_):\n (root / 'stopped').touch()\n raise SystemExit(0)\n"
                    f"signal.signal(signal.SIGTERM, signal.SIG_IGN if {ignore_term!r} else stop)\n"
                    "(root / 'leaf.json').write_text(json.dumps({'pid': os.getpid()}))\n"
                    "time.sleep(60)\n"
                )
                worker = root / "worker.py"
                worker.write_text(
                    "import json, signal, sys\nfrom pathlib import Path\n"
                    f"sys.path.insert(0, {str(Path(mutate.__file__).parent)!r})\n"
                    "import mutate\nroot = Path(sys.argv[1])\n"
                    "before = signal.getsignal(signal.SIGTERM)\n"
                    "mutate.fixture_health = lambda: None\n"
                    "receipt = mutate.execute_command([sys.executable, str(root / 'leaf.py'), str(root)], "
                    "root, root, 'nested', 60)\n"
                    "(root / 'restored.json').write_text(json.dumps({'restored': before == signal.getsignal(signal.SIGTERM)}))\n"
                    "raise SystemExit(receipt['exit_code'])\n"
                )

                def failed_health():
                    self.ready(root / "leaf.json")
                    raise URLError(ConnectionRefusedError(errno.ECONNREFUSED, "SECRET-URL-CANARY"))

                try:
                    with patch.dict(os.environ, {mutate.DRAIN_ENVIRONMENT: "35"}), \
                            patch.object(mutate, "fixture_health", side_effect=failed_health):
                        receipt = mutate.execute_command([sys.executable, str(worker), str(root)],
                                                         root, root, "outer", 60)
                    nested = json.loads((root / "nested.json").read_text())
                    self.assertEqual(receipt["status"], "fixture_unavailable")
                    self.assertEqual(nested["status"], "interrupted")
                    self.assertEqual(nested["exit_code"], 143)
                    self.assertEqual(nested["interrupt_signal"], signal.SIGTERM)
                    self.assertEqual((receipt["cleanup_budget_seconds"], nested["cleanup_budget_seconds"]), (35, 8))
                    self.assertTrue(receipt["cleanup"]["drained"])
                    self.assertFalse(receipt["cleanup"]["forced"])
                    self.assertEqual(nested["cleanup"], {"drained": True, "forced": ignore_term})
                    self.assertTrue(json.loads((root / "restored.json").read_text())["restored"])
                    self.assertEqual(receipt["fixture_error"]["reason_type"], "ConnectionRefusedError")
                    self.assertEqual(receipt["fixture_error"]["errno"], errno.ECONNREFUSED)
                    self.assertNotIn("SECRET-URL-CANARY", json.dumps(receipt))
                    if not ignore_term:
                        self.assertTrue((root / "stopped").is_file())
                    with self.assertRaises(ProcessLookupError):
                        os.killpg(json.loads((root / "leaf.json").read_text())["pid"], 0)
                finally:
                    if (root / "leaf.json").exists():
                        try:
                            os.killpg(json.loads((root / "leaf.json").read_text())["pid"], signal.SIGKILL)
                        except ProcessLookupError:
                            pass

    def test_sigterm_during_final_health_cannot_become_completed(self):
        probe = self.root / "final-health.py"
        child = ("import json,os,sys,time; from pathlib import Path; root=Path(sys.argv[1]); "
                 "(root/'child.json').write_text(json.dumps({'pid':os.getpid()})); "
                 "exec(\"while not (root/'health-seen').exists(): time.sleep(0.01)\")")
        probe.write_text(
            "import json, os, signal, sys, time\nfrom pathlib import Path\n"
            f"sys.path.insert(0, {str(Path(mutate.__file__).parent)!r})\nimport mutate\n"
            "root = Path(sys.argv[1]); calls = 0\n"
            "before = {number: signal.getsignal(number) for number in (signal.SIGINT, signal.SIGTERM)}\n"
            "def health():\n global calls\n calls += 1\n"
            " if calls == 1:\n"
            "  deadline = time.monotonic() + 3\n"
            "  while not (root/'child.json').exists() and time.monotonic() < deadline: time.sleep(0.01)\n"
            "  (root/'health-seen').touch()\n"
            " else: os.kill(os.getpid(), signal.SIGTERM)\n"
            "mutate.fixture_health = health\n"
            f"receipt = mutate.execute_command([sys.executable, '-c', {child!r}, str(root)], root, root, 'final', 10)\n"
            "(root/'probe.json').write_text(json.dumps({'calls': calls, 'restored': "
            "all(signal.getsignal(number) == handler for number,handler in before.items())}))\n"
        )
        process = subprocess.Popen([sys.executable, str(probe), str(self.root)],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
        try:
            stdout, stderr = process.communicate(timeout=15)
            self.assertEqual(process.returncode, 0, stdout + stderr)
            receipt = json.loads((self.root / "final.json").read_text())
            self.assertEqual((receipt["status"], receipt["exit_code"]), ("interrupted", 143))
            self.assertEqual(receipt["fixture_health_checks"], 1)
            self.assertEqual(json.loads((self.root / "probe.json").read_text()), {"calls": 2, "restored": True})
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=3)
            if (self.root / "child.json").exists():
                try:
                    os.killpg(json.loads((self.root / "child.json").read_text())["pid"], signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def test_group_descendant_is_stopped_after_the_leader_already_exited(self):
        leader = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"], process_group=0)
        ready, stopped = self.root / "group-ready", self.root / "group-stopped"
        child = ("import signal,sys,time; from pathlib import Path; "
                 "signal.signal(signal.SIGTERM, lambda *_: (Path(sys.argv[2]).touch(), sys.exit(0))); "
                 "Path(sys.argv[1]).touch(); time.sleep(60)")
        descendant = subprocess.Popen([sys.executable, "-c", child, str(ready), str(stopped)],
                                      process_group=leader.pid)
        reaper = threading.Thread(target=descendant.wait)
        reaper.start()
        try:
            self.ready(ready)
            leader.terminate()
            leader.wait(timeout=3)
            self.assertIsNotNone(leader.returncode)
            self.assertEqual(mutate.stop(leader, 6), {"drained": True, "forced": False})
            reaper.join(timeout=3)
            self.assertFalse(reaper.is_alive())
            self.assertTrue(stopped.exists())
            with self.assertRaises(ProcessLookupError):
                os.killpg(leader.pid, 0)
        finally:
            for process in (leader, descendant):
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=3)
            reaper.join(timeout=3)

    def test_exhausted_or_malformed_nested_budget_records_failure_before_spawn(self):
        for number, value in enumerate(("0", "-1", "bad", "36")):
            with self.subTest(value=value), patch.dict(os.environ, {mutate.DRAIN_ENVIRONMENT: value}), \
                    patch.object(mutate.subprocess, "Popen") as spawn:
                receipt = mutate.execute_command(["synthetic-not-executed"], self.root, self.root,
                                                 f"bad-budget-{number}", 5)
                spawn.assert_not_called()
                self.assertEqual(receipt["status"], "supervisor_error")
                self.assertEqual(receipt["supervisor_error"]["phase"], "budget")
                self.assertNotEqual(receipt["exit_code"], 0)

    def test_spawn_failure_is_terminal_and_restores_signal_handlers(self):
        before = {number: signal.getsignal(number) for number in (signal.SIGINT, signal.SIGTERM)}
        receipt = mutate.execute_command([str(self.root / "nonexistent-executable")], self.root, self.root, "spawn", 5)
        self.assertEqual(receipt["status"], "supervisor_error")
        self.assertEqual(receipt["supervisor_error"], {"phase": "spawn", "type": "FileNotFoundError", "errno": errno.ENOENT})
        self.assertEqual(json.loads((self.root / "spawn.json").read_text()), receipt)
        self.assertEqual({number: signal.getsignal(number) for number in before}, before)

    def test_health_diagnostics_preserve_stage_and_tls_code_without_exception_text(self):
        failure = ssl.SSLCertVerificationError(1, "https://user:SECRET-URL-CANARY@host")
        failure.verify_code = 62
        with patch.dict(os.environ, {"AWS_ENDPOINT_URL": "https://127.0.0.1:1", "SSL_CERT_FILE": "synthetic"}), \
                patch.object(mutate.ssl, "create_default_context"), \
                patch.object(mutate.urllib.request, "urlopen", side_effect=URLError(failure)) as request:
            with self.assertRaises(mutate.FixtureHealthError) as observed:
                mutate.fixture_health()
        details = mutate.error_details(observed.exception)
        self.assertEqual(details["stage"], "aws_tls_health")
        self.assertEqual(details["type"], "URLError")
        self.assertEqual(details["reason_type"], "SSLCertVerificationError")
        self.assertEqual(details["verify_code"], 62)
        self.assertEqual(request.call_args.kwargs["timeout"], 3)
        self.assertNotIn("SECRET-URL-CANARY", json.dumps(details))

    def test_native_cleanup_exception_preserves_the_primary_error(self):
        primary = RuntimeError("primary fixture failure")
        reservation, connection = Mock(), Mock()
        reservation.getsockname.return_value = ("127.0.0.1", 12345)
        connection.recv.return_value = b"+PONG\r\n"
        from contextlib import nullcontext
        with patch.object(mutate.socket, "socket", return_value=nullcontext(reservation)), \
                patch.object(mutate.socket, "create_connection", return_value=nullcontext(connection)), \
                patch.object(mutate.subprocess, "Popen") as spawn, \
                patch.object(mutate, "stop", side_effect=OSError("SECRET-URL-CANARY")):
            spawn.return_value.poll.return_value = None
            with self.assertRaises(RuntimeError) as observed:
                with mutate.falkor_server(self.root / "server", self.root / "module", self.root):
                    raise primary
        self.assertIs(observed.exception, primary)
        self.assertTrue(any("cleanup raised OSError" in note for note in primary.__notes__))
        self.assertNotIn("SECRET-URL-CANARY", str(primary.__notes__))


if __name__ == "__main__":
    unittest.main()
