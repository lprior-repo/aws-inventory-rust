"""Exercise the all-scope coordinator without Cargo, SDK calls, or real fixtures."""

from __future__ import annotations

import argparse
import contextlib
import io
import os
from pathlib import Path
import signal
import sys
import tempfile
import threading
import time
import unittest
from unittest import mock

import run_all


class OwnedFixtureSupervision(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.root = Path(self.folder.name)
        self.script = self.root / "fixture.py"
        self.script.write_text(
            "import signal, sys, time\nfrom pathlib import Path\n"
            "root = Path(sys.argv[1])\n"
            "try:\n"
            " (root / 'ready').write_text('ready')\n"
            " time.sleep(60)\n"
            "except KeyboardInterrupt:\n"
            " (root / 'owned-cleanup').write_text('completed')\n"
        )

    def tearDown(self):
        self.folder.cleanup()

    def test_filesystem_stop_reaches_owned_wrapper_cleanup(self):
        stop = self.root / "STOP"

        def request_stop():
            deadline = time.monotonic() + 3
            while not (self.root / "ready").exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            stop.write_text("requested by a separate controller")

        thread = threading.Thread(target=request_stop)
        before = signal.getsignal(signal.SIGTERM)
        thread.start()
        try:
            result = run_all.supervise([sys.executable, str(self.script), str(self.root)], self.root,
                                       dict(os.environ), self.root / "wrapper.log", stop, 5)
        finally:
            thread.join(timeout=4)
        self.assertFalse(thread.is_alive())
        self.assertEqual(result["status"], "stop_requested")
        self.assertTrue(result["graceful_owned_drain"])
        self.assertTrue((self.root / "owned-cleanup").exists())
        self.assertEqual(signal.getsignal(signal.SIGTERM), before)

    def test_wall_deadline_still_drains_owned_wrapper(self):
        result = run_all.supervise([sys.executable, str(self.script), str(self.root)], self.root,
                                   dict(os.environ), self.root / "wrapper.log", self.root / "STOP", 0.3)
        self.assertEqual(result["status"], "deadline_exceeded")
        self.assertTrue(result["graceful_owned_drain"])
        self.assertTrue((self.root / "owned-cleanup").exists())

    def test_normal_exit_keeps_its_actual_status(self):
        result = run_all.supervise([sys.executable, "-c", "raise SystemExit(7)"], self.root,
                                   dict(os.environ), self.root / "wrapper.log", self.root / "STOP", 5)
        self.assertEqual(result["status"], "completed")
        self.assertEqual(result["exit_code"], 7)

    def test_fast_exit_cannot_bypass_the_output_limit(self):
        with mock.patch.object(run_all, "MAX_LAUNCH_LOG_BYTES", 100):
            result = run_all.supervise([sys.executable, "-c", "print('x' * 1000)"], self.root,
                dict(os.environ), self.root / "wrapper.log", self.root / "STOP", 5)
        self.assertEqual(result["status"], "log_budget_exceeded")
        self.assertGreater(result["log_bytes"], 100)

    def test_exit_between_polls_cannot_bypass_the_wall_deadline(self):
        result = run_all.supervise([sys.executable, "-c", "import time; time.sleep(0.02)"], self.root,
            dict(os.environ), self.root / "wrapper.log", self.root / "STOP", 0.01)
        self.assertEqual(result["status"], "deadline_exceeded")


class WholeGateOrdering(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.root = Path(self.folder.name)
        self.workspace = self.root / "authoring"
        self.workspace.mkdir()
        self.args = argparse.Namespace(
            workspace=self.workspace, campaign=self.workspace / "artifacts/all",
            snapshot=self.root / "snapshot", target_dir=self.root / "target",
            fixture_output=self.workspace / "artifacts/fixture", stop_file=None,
            tool=Path(run_all.gate.DEFAULT_TOOL), timeout=240, build_timeout=600, wall_timeout=86400,
            emulator=self.root / "emulator", binary=self.root / "collector", branch_probe=self.root / "probe",
            redis_server=self.root / "redis-server", falkor_module=self.root / "module.so",
        )

    def tearDown(self):
        self.folder.cleanup()

    def fake_supervision(self, status="completed", code=0):
        def run(_command, _workspace, _environment, log, _stop_file, _timeout):
            log.write_text("synthetic lifecycle evidence, no mutation execution\n")
            return {"status": status, "exit_code": code, "graceful_owned_drain": True}
        return run

    def test_final_aggregate_only_runs_after_closed_successful_outer_lifetime(self):
        calls = []

        def aggregate(workspace, campaigns, _tool, output):
            receipt = run_all.gate.load_json(self.args.campaign.with_name("all-launcher.json"))
            self.assertEqual(receipt["status"], "completed")
            self.assertEqual(receipt["exit_code"], 0)
            self.assertEqual(workspace, self.workspace)
            self.assertEqual(campaigns, [self.args.campaign])
            self.assertEqual(output, self.args.campaign / "final-universe.json")
            calls.append("aggregate")
            return {"candidate_count": 2535, "counts": {}, "score_percent": 94,
                    "gate_passed": False, "errors": []}

        with mock.patch.object(run_all, "supervise", self.fake_supervision()), \
             mock.patch.object(run_all.gate, "aggregate", aggregate), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(run_all.launch(self.args), 1)
        self.assertEqual(calls, ["aggregate"])
        final = run_all.gate.load_json(self.args.campaign / "final-report.json")
        self.assertFalse(final["gate_passed"])

    def test_stopped_run_never_starts_final_enumeration_or_claims_a_score(self):
        with mock.patch.object(run_all, "supervise", self.fake_supervision("stop_requested", 130)), \
             mock.patch.object(run_all.gate, "aggregate") as aggregate, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(run_all.launch(self.args), 2)
        aggregate.assert_not_called()
        self.assertFalse((self.args.campaign / "final-report.json").exists())

    def test_failed_fixture_lifetime_never_reaches_final_aggregate(self):
        with mock.patch.object(run_all, "supervise", self.fake_supervision("completed", 124)), \
             mock.patch.object(run_all.gate, "aggregate") as aggregate, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(run_all.launch(self.args), 2)
        aggregate.assert_not_called()

    def test_preexisting_stop_request_prevents_process_start(self):
        stop = self.args.campaign.with_name("all.STOP")
        stop.parent.mkdir(parents=True)
        stop.write_text("stop")
        with mock.patch.object(run_all, "supervise") as supervise:
            with self.assertRaises(run_all.gate.GateError):
                run_all.launch(self.args)
        supervise.assert_not_called()

    def test_authoring_target_cannot_be_reused_for_mutations(self):
        self.args.target_dir = self.workspace / "target"
        with mock.patch.object(run_all, "supervise") as supervise:
            with self.assertRaises(run_all.gate.GateError):
                run_all.launch(self.args)
        supervise.assert_not_called()


if __name__ == "__main__":
    unittest.main()
