"""Adversarial checks of accounting/evidence parsing; no Cargo or network calls."""

from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location("mutation_gate", Path(__file__).with_name("gate.py"))
assert SPEC is not None and SPEC.loader is not None
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


def candidate(number: int, package="inventory-core"):
    return {"name": f"src/lib.rs:{number}: replace value", "package": package,
            "file": "crates/core/src/lib.rs", "function": None, "span": {"start": number, "end": number},
            "replacement": "false", "genre": "FnValue"}


def phase(build, status="Success", package="inventory-core", ignored=False):
    argv = ["cargo", "test"] + (["--no-run"] if build else [])
    argv += ["--verbose", f"--package={package}@0.4.0", "--all-features", "--locked", "--offline", "-j1"]
    if ignored and not build:
        argv += ["--", "--include-ignored"]
    return {"phase": "Build" if build else "Test", "process_status": status, "duration": 0.1, "argv": argv}


def outcome(status="caught", ignored=False):
    if status == "unviable":
        return {"summary": "Unviable", "phase_results": [phase(True, {"Failure": 101}, ignored=ignored)]}
    test = "Success" if status in {"baseline", "missed"} else "Timeout" if status == "timeout" else {"Failure": 101}
    summary = {"baseline": "Success", "missed": "MissedMutant", "caught": "CaughtMutant", "timeout": "Timeout"}[status]
    return {"summary": summary, "phase_results": [phase(True, ignored=ignored), phase(False, test, ignored=ignored)]}


PASSED = "test example ... ok\ntest result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.1s\n"
FAILED = "test example ... FAILED\ntest result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.1s\n"


class BoundedProcessSupervision(unittest.TestCase):
    def run_interruption(self, send_sigterm):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            child = root / "child.py"
            child.write_text(
                "import json, os, signal, subprocess, sys, time\n"
                "from pathlib import Path\n"
                "root = Path(sys.argv[1])\n"
                "grandchild = subprocess.Popen([sys.executable, '-c', "
                "\"import signal, sys, time; from pathlib import Path; "
                "signal.signal(signal.SIGINT, lambda *_: (Path(sys.argv[1]).write_text('drained'), sys.exit(0))); "
                "Path(sys.argv[2]).write_text('ready'); time.sleep(60)\", "
                "str(root / 'grandchild-drained'), str(root / 'grandchild-ready')])\n"
                "try:\n"
                " while not (root / 'grandchild-ready').exists(): time.sleep(0.01)\n"
                " (root / 'ready.json').write_text(json.dumps({'pid': os.getpid()}))\n"
                " time.sleep(60)\n"
                "except KeyboardInterrupt:\n"
                " grandchild.wait(timeout=2)\n"
                " (root / 'child-drained').write_text('drained')\n"
            )
            probe = root / "probe.py"
            probe.write_text(
                "import importlib.util, json, os, signal, sys\n"
                "from pathlib import Path\n"
                f"spec = importlib.util.spec_from_file_location('gate', {str(Path(gate.__file__).resolve())!r})\n"
                "gate = importlib.util.module_from_spec(spec); spec.loader.exec_module(gate)\n"
                "root = Path(sys.argv[1]); before = signal.getsignal(signal.SIGTERM)\n"
                "with (root / 'child.log').open('w') as log:\n"
                " result = gate.execute_bounded([sys.executable, str(root / 'child.py'), str(root)], "
                "root, dict(os.environ), log, int(sys.argv[2]))\n"
                "print(json.dumps({'result': result, 'handler_restored': before == signal.getsignal(signal.SIGTERM)}))\n"
            )
            process = subprocess.Popen(
                [sys.executable, str(probe), str(root), "30" if send_sigterm else "1"],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True,
            )
            try:
                deadline = time.monotonic() + 5
                while not (root / "ready.json").exists() and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertTrue((root / "ready.json").is_file(), "owned subprocess did not become ready")
                if send_sigterm:
                    process.send_signal(signal.SIGTERM)
                stdout, stderr = process.communicate(timeout=9)
                self.assertEqual(process.returncode, 0, stderr)
                result = json.loads(stdout)
                self.assertEqual(result["result"], {"Signalled": signal.SIGTERM} if send_sigterm else "Timeout")
                self.assertTrue(result["handler_restored"])
                self.assertTrue((root / "child-drained").is_file())
                self.assertTrue((root / "grandchild-drained").is_file())
            finally:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=2)
                if (root / "ready.json").is_file():
                    pid = json.loads((root / "ready.json").read_text())["pid"]
                    try:
                        os.killpg(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass

    def test_outer_sigterm_interrupts_and_drains_separate_owned_group(self):
        self.run_interruption(True)

    def test_internal_deadline_interrupts_and_drains_separate_owned_group(self):
        self.run_interruption(False)


class EvidenceClassification(unittest.TestCase):
    def classify(self, value, log=FAILED, **kwargs):
        return gate.classify_outcome(value, log, {"inventory-core"}, **kwargs)[0]

    def test_actual_test_failure_after_build_is_caught(self):
        self.assertEqual(self.classify(outcome()), "caught")

    def test_zero_tests_cannot_pass_a_baseline(self):
        self.assertEqual(self.classify(outcome("baseline"), PASSED.replace("5 passed", "0 passed"), baseline=True), "unresolved")

    def test_successful_baseline_requires_both_phases(self):
        value = outcome("baseline")
        value["phase_results"].pop()
        self.assertEqual(self.classify(value, PASSED, baseline=True), "unresolved")

    def test_timeout_is_never_caught_even_with_failure_text(self):
        value = outcome("timeout")
        value["summary"] = "CaughtMutant"
        self.assertEqual(self.classify(value), "timeout")

    def test_internal_harness_deadlines_are_unresolved_even_when_tests_fail(self):
        messages = [
            "fixture CLI exceeded its 60-second deadline",
            "CLI exceeded the 30-second test deadline",
            "snapshot collection exceeded its deadline; cancelled and drained=true",
            "durable job property exceeded its 10-second case deadline",
            "durable store property exceeded its 10-second case deadline",
            "engine limiter barrier test exceeded its 30-second virtual deadline",
            "engine graph integration test exceeded its 10-second deadline",
        ]
        for message in messages:
            with self.subTest(message=message):
                self.assertEqual(self.classify(outcome(), message + "\n" + FAILED), "timeout")

    def test_normal_sdk_timeout_error_assertion_can_still_catch_a_mutant(self):
        log = "SDK error classification differed: TimeoutError, expected retryable\n" + FAILED
        self.assertEqual(self.classify(outcome(), log), "caught")

    def test_parent_wire_fixture_deadline_is_unresolved(self):
        log = "Error: parent SDK wire fixture exceeded its 20-second deadline\n" + FAILED
        self.assertEqual(self.classify(outcome(), log), "timeout")

    def test_inventory_export_harness_deadlines_are_unresolved(self):
        for message in [
            "inventory export property exceeded its 10-second case deadline",
            "inventory export streaming test exceeded its 30-second deadline",
        ]:
            with self.subTest(message=message):
                self.assertEqual(self.classify(outcome(), message + "\n" + FAILED), "timeout")

    def test_graph_retry_fixture_deadlines_are_unresolved(self):
        for message in [
            "graph retry fixture query start exceeded its 5-second deadline",
            "graph retry fixture shutdown exceeded its 5-second deadline",
        ]:
            with self.subTest(message=message):
                self.assertEqual(self.classify(outcome(), message + "\n" + FAILED), "timeout")

    def test_production_graph_and_export_contract_failures_remain_caught(self):
        for message in [
            "assertion failed: matches!(result, Err(GraphError::Deadline))",
            "inventory export assertion failed: exported row count changed",
            "graph retry fixture assertion failed: expected exactly one reconnect",
        ]:
            with self.subTest(message=message):
                self.assertEqual(self.classify(outcome(), message + "\n" + FAILED), "caught")

    def test_sigkill_is_not_an_unviable_mutant(self):
        self.assertEqual(self.classify(outcome("unviable"), "error: could not compile\n(signal: 9, SIGKILL)"), "unresolved")

    def test_redis_memory_exhaustion_is_not_a_caught_mutant(self):
        for text in ["OOM command not allowed when used memory > 'maxmemory'",
                     "Redis error - OOM: command not allowed when used memory > 'maxmemory'"]:
            self.assertEqual(self.classify(outcome(), text + "\n" + FAILED), "unresolved")

    def test_cargo_failure_without_failed_tests_is_not_caught(self):
        self.assertEqual(self.classify(outcome(), "error: failed to load test executable"), "unresolved")

    def test_confirmed_compiler_failure_is_unviable(self):
        self.assertEqual(self.classify(outcome("unviable"), "error[E0308]: mismatched types"), "unviable")

    def test_forged_summary_disagrees_with_process_status(self):
        value = outcome("missed")
        value["summary"] = "CaughtMutant"
        self.assertEqual(self.classify(value, PASSED), "unresolved")

    def test_wrong_package_test_is_not_credited(self):
        value = outcome()
        value["phase_results"][1] = phase(False, {"Failure": 101}, "inventory-engine")
        self.assertEqual(self.classify(value), "unresolved")

    def test_filtered_test_suite_is_rejected(self):
        value = outcome()
        value["phase_results"][1]["argv"].append("only_easy_test")
        self.assertEqual(self.classify(value), "unresolved")

    def test_metadata_check_is_never_mutation_evidence(self):
        value = outcome()
        value["phase_results"][0]["phase"] = "Check"
        self.assertEqual(self.classify(value), "unresolved")

    def test_ignored_integration_policy_must_be_present(self):
        self.assertEqual(self.classify(outcome("baseline"), PASSED, baseline=True, include_ignored=True), "unresolved")
        self.assertEqual(self.classify(outcome("baseline", ignored=True), PASSED,
                                       baseline=True, include_ignored=True), "baseline_passed")

    def test_required_integration_test_cannot_be_ignored(self):
        self.assertEqual(self.classify(outcome("baseline", ignored=True), PASSED,
                                       baseline=True, include_ignored=True, required_tests=["missing"]), "unresolved")
        self.assertEqual(self.classify(outcome("baseline", ignored=True), PASSED,
                                       baseline=True, include_ignored=True, required_tests=["example"]), "baseline_passed")


class ScoreAccounting(unittest.TestCase):
    def matrix(self, caught, missed=0, unviable=0, unresolved=0, untested=0):
        mutations, attempts = {}, {}
        statuses = ["caught"] * caught + ["missed"] * missed + ["unviable"] * unviable + ["timeout"] * unresolved + [None] * untested
        for index, status in enumerate(statuses):
            value = candidate(index)
            key = gate.candidate_id(value)
            mutations[key] = value
            if status:
                attempts[key] = [{"status": status}]
        return mutations, attempts

    def test_exact_95_percent_passes_only_complete_whole_scope(self):
        mutations, attempts = self.matrix(19, missed=1)
        self.assertTrue(gate.summarize(mutations, attempts, full_scope=True)["gate_passed"])
        self.assertFalse(gate.summarize(mutations, attempts, full_scope=False)["gate_passed"])

    def test_below_95_percent_fails_without_rounding_up(self):
        mutations, attempts = self.matrix(18, missed=1)
        result = gate.summarize(mutations, attempts, full_scope=True)
        self.assertFalse(result["gate_passed"])
        self.assertFalse(result["score_requirement_met"])

    def test_untested_candidates_prevent_a_pass(self):
        mutations, attempts = self.matrix(99, untested=1)
        result = gate.summarize(mutations, attempts, full_scope=True)
        self.assertEqual(result["score_percent"], 100)
        self.assertFalse(result["gate_passed"])
        self.assertEqual(result["counts"]["untested"], 1)

    def test_timeouts_prevent_a_pass_without_entering_numerator(self):
        mutations, attempts = self.matrix(99, unresolved=1)
        result = gate.summarize(mutations, attempts, full_scope=True)
        self.assertEqual(result["counts"]["caught"], 99)
        self.assertEqual(result["timeout_attempts"], 1)
        self.assertFalse(result["gate_passed"])

    def test_all_unviable_does_not_create_a_score(self):
        mutations, attempts = self.matrix(0, unviable=20)
        result = gate.summarize(mutations, attempts, full_scope=True)
        self.assertIsNone(result["score_percent"])
        self.assertFalse(result["gate_passed"])

    def test_unviable_is_visible_and_excluded_from_denominator(self):
        mutations, attempts = self.matrix(19, missed=1, unviable=100)
        result = gate.summarize(mutations, attempts, full_scope=True)
        self.assertEqual(result["viable_completed"], 20)
        self.assertEqual(result["counts"]["unviable"], 100)
        self.assertEqual(result["score_percent"], 95)

    def test_duplicate_attempts_cannot_inflate_caught_count(self):
        mutations, attempts = self.matrix(1)
        key = next(iter(mutations))
        attempts[key] *= 10
        self.assertEqual(gate.summarize(mutations, attempts)["counts"]["caught"], 1)

    def test_conflicting_completed_results_are_not_cherry_picked(self):
        mutations, attempts = self.matrix(1)
        key = next(iter(mutations))
        attempts[key].append({"status": "missed"})
        result = gate.summarize(mutations, attempts, full_scope=True)
        self.assertEqual(result["counts"]["unresolved"], 1)
        self.assertFalse(result["gate_passed"])

    def test_later_completed_attempt_can_resolve_timeout_with_history_retained(self):
        mutations, attempts = self.matrix(0, unresolved=1)
        key = next(iter(mutations))
        attempts[key].append({"status": "caught"})
        result = gate.summarize(mutations, attempts, full_scope=True)
        self.assertEqual(result["counts"]["caught"], 1)
        self.assertEqual(result["timeout_attempts"], 1)
        self.assertEqual(len(result["mutants"][0]["attempts"]), 2)


class InputIntegrity(unittest.TestCase):
    def test_candidate_diff_is_not_part_of_tool_outcome_identity(self):
        value = candidate(1)
        self.assertEqual(gate.candidate_id(value), gate.candidate_id({**value, "diff": "context"}))

    def test_duplicate_candidate_ids_fail(self):
        with self.assertRaises(gate.GateError):
            gate.candidate_map([candidate(1), candidate(1)])

    def test_unknown_candidate_field_fails(self):
        with self.assertRaises(gate.GateError):
            gate.candidate_id({**candidate(1), "excluded_survivor": True})

    def test_slices_cover_universe_once_even_with_empty_tail_shard(self):
        ids = [str(i) for i in range(10)]
        shards = [gate.selected_ids(ids, f"{i}/6") for i in range(6)]
        self.assertEqual([item for shard in shards for item in shard], ids)
        self.assertEqual(shards[-1], [])

    def test_invalid_shards_fail(self):
        for shard in ("0/0", "2/2", "-1/5", "0/100001", "0"):
            with self.subTest(shard=shard), self.assertRaises(gate.GateError):
                gate.selected_ids(["a"], shard)

    def test_duplicate_json_keys_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "bad.json"
            path.write_text('{"caught":1,"caught":100}')
            with self.assertRaises(gate.GateError):
                gate.load_json(path)

    def test_traversal_and_symlink_logs_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            with self.assertRaises(gate.GateError):
                gate.relative_file(root, "../elsewhere")
            (root / "outside").symlink_to("/tmp")
            with self.assertRaises(gate.GateError):
                gate.relative_file(root, "outside/file")

    def test_source_and_own_test_changes_change_the_closure(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "crates/core/src").mkdir(parents=True)
            (root / "crates/core/tests").mkdir()
            source = root / "crates/core/src/lib.rs"
            test = root / "crates/core/tests/property.rs"
            source.write_text("fn value() -> bool { true }")
            test.write_text("#[test] fn validates() {}")
            first = gate.closure_files(root, ["inventory-core"])
            test.write_text("#[test] fn changed() {}")
            self.assertNotEqual(first, gate.closure_files(root, ["inventory-core"]))

    def test_unrelated_package_test_edit_does_not_invalidate_core(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "crates/core/src").mkdir(parents=True)
            (root / "crates/aws/tests").mkdir(parents=True)
            (root / "crates/core/src/lib.rs").write_text("fn value() -> bool { true }")
            first = gate.closure_files(root, ["inventory-core"])
            (root / "crates/aws/tests/property.rs").write_text("#[test] fn changes() {}")
            self.assertEqual(first, gate.closure_files(root, ["inventory-core"]))

    def test_dependency_source_edit_invalidates_engine(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "crates/core/src").mkdir(parents=True)
            (root / "crates/engine/src").mkdir(parents=True)
            source = root / "crates/core/src/lib.rs"
            source.write_text("fn value() -> bool { true }")
            first = gate.closure_files(root, ["inventory-engine"])
            source.write_text("fn value() -> bool { false }")
            self.assertNotEqual(first, gate.closure_files(root, ["inventory-engine"]))


class CompleteReceiptValidation(unittest.TestCase):
    """Synthetic file-backed receipts exercise the entire acceptance path."""

    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.root = Path(self.folder.name)
        source = self.root / "crates/core/src/lib.rs"
        source.parent.mkdir(parents=True)
        source.write_text("pub fn value() -> bool { true }\n")
        (self.root / "Cargo.toml").write_text("# synthetic parser fixture; never compiled\n")
        self.campaign = self.root / "artifacts/campaign"
        self.run = self.campaign / "runs/0001"
        self.output = self.run / "mutants.out"
        (self.output / "log").mkdir(parents=True)
        self.mutation = candidate(1)
        self.key = gate.candidate_id(self.mutation)
        gate.write_json(self.campaign / "candidates.json", [{**self.mutation, "diff": "synthetic"}])
        files = gate.closure_files(self.root, ["inventory-core"])
        self.manifest = {
            "schema_version": 1, "baseline_required": True, "packages": ["inventory-core"],
            "workspace_origin": str(self.root), "mutation_options": gate.mutation_options(["inventory-core"]),
            "tool": {"version": gate.VERSION, "binary_sha256": gate.TOOL_SHA256},
            "environment": gate.ENVIRONMENT, "test_scope": "mutated-package", "score_threshold_percent": 95,
            "source_files": files, "source_digest": gate.source_digest(files),
            "candidate_file": "candidates.json", "candidate_file_sha256": gate.digest(self.campaign / "candidates.json"),
            "candidate_count": 1, "candidate_ids": [self.key],
        }
        gate.write_json(self.campaign / "manifest.json", self.manifest)
        self.baseline = {**outcome("baseline"), "scenario": "Baseline", "log_path": "log/baseline.log"}
        self.observed = {**outcome(), "scenario": {"Mutant": self.mutation}, "log_path": "log/mutant.log"}
        self.results = {"cargo_mutants_version": gate.VERSION, "end_time": "2026-11-12T00:00:00Z",
                        "total_mutants": 1, "caught": 1, "missed": 0, "timeout": 0, "unviable": 0, "success": 0,
                        "outcomes": [self.baseline, self.observed]}
        gate.write_json(self.output / "mutants.json", [self.mutation])
        (self.output / "log/baseline.log").write_text(PASSED)
        (self.output / "log/mutant.log").write_text(FAILED)
        self.receipt = {
            "manifest_sha256": gate.digest(self.campaign / "manifest.json"), "status": "finished", "exit_code": 0,
            "tool_binary_sha256": gate.TOOL_SHA256, "environment": gate.ENVIRONMENT,
            "source_digest_before": self.manifest["source_digest"], "source_digest_after": self.manifest["source_digest"],
            "source_restored": True, "candidate_ids": [self.key], "shard": None,
        }
        self.save()

    def tearDown(self):
        self.folder.cleanup()

    def test_disposable_snapshot_refreshes_rust_mtimes_without_changing_source_hashes(self):
        source = self.root / "crates/core/src/lib.rs"
        os.utime(source, (1, 1))
        with tempfile.TemporaryDirectory() as folder:
            destination = Path(folder) / "snapshot"
            gate.snapshot_workspace(self.root, destination, self.campaign)
            copied = destination / "crates/core/src/lib.rs"
            self.assertGreater(copied.stat().st_mtime_ns, source.stat().st_mtime_ns)
            self.assertEqual(copied.read_bytes(), source.read_bytes())
            self.assertEqual(gate.closure_files(destination, ["inventory-core"]), self.manifest["source_files"])
            marker = gate.load_json(destination / ".mutation-workspace.json")
            self.assertTrue(marker["first_party_rust_mtimes_refreshed"])
        self.assertEqual(source.stat().st_mtime_ns, 1_000_000_000)

    def save(self):
        gate.write_json(self.output / "outcomes.json", self.results)
        self.receipt["evidence_files"] = {
            path.relative_to(self.run).as_posix(): gate.digest(path)
            for path in sorted(self.output.rglob("*")) if path.is_file()
        }
        gate.write_json(self.run / "receipt.json", self.receipt)

    def test_valid_complete_receipt_is_accepted_as_package_only(self):
        result = gate.report_campaign(self.campaign, self.root)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["counts"]["caught"], 1)
        self.assertTrue(result["complete"])
        self.assertFalse(result["gate_passed"])

    def test_changed_log_hash_rejects_the_run(self):
        (self.output / "log/mutant.log").write_text("forged caught result")
        result = gate.report_campaign(self.campaign, self.root)
        self.assertTrue(result["errors"])
        self.assertEqual(result["counts"]["untested"], 1)

    def test_missing_baseline_rejects_the_run(self):
        self.results["outcomes"].remove(self.baseline)
        self.save()
        self.assertTrue(gate.report_campaign(self.campaign, self.root)["errors"])

    def test_tool_summary_disagreement_is_not_trusted(self):
        self.results["caught"] = 100
        self.save()
        self.assertTrue(gate.report_campaign(self.campaign, self.root)["errors"])

    def test_unfinished_tool_output_cannot_pass(self):
        self.results["end_time"] = None
        self.save()
        self.assertTrue(gate.report_campaign(self.campaign, self.root)["errors"])

    def test_deleted_candidate_from_tool_selection_is_rejected(self):
        gate.write_json(self.output / "mutants.json", [])
        self.save()
        self.assertTrue(gate.report_campaign(self.campaign, self.root)["errors"])

    def test_unknown_mutant_outcome_is_rejected(self):
        self.observed["scenario"] = {"Mutant": candidate(2)}
        self.save()
        self.assertTrue(gate.report_campaign(self.campaign, self.root)["errors"])

    def test_duplicate_outcome_does_not_inflate_result(self):
        self.results["outcomes"].append(self.observed)
        self.save()
        self.assertTrue(gate.report_campaign(self.campaign, self.root)["errors"])

    def test_unrestored_source_rejects_the_run(self):
        self.receipt["source_restored"] = False
        self.save()
        self.assertTrue(gate.report_campaign(self.campaign, self.root)["errors"])

    def test_final_source_drift_rejects_the_partition(self):
        (self.root / "crates/core/src/lib.rs").write_text("pub fn value() -> bool { false }\n")
        with self.assertRaises(gate.GateError):
            gate.report_campaign(self.campaign, self.root)

    def test_candidate_manifest_tampering_is_rejected(self):
        gate.write_json(self.campaign / "candidates.json", [])
        with self.assertRaises(gate.GateError):
            gate.report_campaign(self.campaign, self.root)

    def test_integration_policy_cannot_reduce_required_baselines(self):
        self.manifest.update(include_ignored=True, baseline_runs_required=1,
                             mutation_options=gate.mutation_options(["inventory-core"], True))
        gate.write_json(self.campaign / "manifest.json", self.manifest)
        with self.assertRaises(gate.GateError):
            gate.read_manifest(self.campaign, self.root)


class RequiredPackageTests(unittest.TestCase):
    def setUp(self):
        self.manifest = {
            "packages": list(gate.PACKAGES), "required_tests": [],
            "required_tests_by_package": {
                "aws-inventory": ["root_fixture"],
                "inventory-aws": ["aws_fixture"],
                "inventory-engine": ["native_graph_fixture"],
            },
        }

    def test_combined_baseline_requires_every_selected_packages_fixture(self):
        self.assertEqual(gate.required_tests_for(self.manifest, gate.PACKAGES),
                         ["aws_fixture", "native_graph_fixture", "root_fixture"])

    def test_core_mutants_do_not_require_other_packages_integration_tests(self):
        self.assertEqual(gate.required_tests_for(self.manifest, ["inventory-core"]), [])

    def test_root_mutants_require_root_integration_without_unrun_aws_suites(self):
        names = gate.required_tests_for(self.manifest, ["aws-inventory"])
        self.assertEqual(names, ["root_fixture"])
        value = outcome("missed", ignored=True)
        for part in value["phase_results"]:
            part["argv"] = [item.replace("inventory-core", "aws-inventory") for item in part["argv"]]
        status, _ = gate.classify_outcome(value, "test root_fixture ... ok\n" + PASSED,
                                         {"aws-inventory"}, required_tests=names, include_ignored=True)
        self.assertEqual(status, "missed")

    def test_required_aws_fixture_missing_from_success_still_fails_closed(self):
        value = outcome("missed", ignored=True)
        for part in value["phase_results"]:
            part["argv"] = [item.replace("inventory-core", "inventory-aws") for item in part["argv"]]
        status, _ = gate.classify_outcome(value, PASSED, {"inventory-aws"},
            required_tests=gate.required_tests_for(self.manifest, ["inventory-aws"]), include_ignored=True)
        self.assertEqual(status, "unresolved")

    def test_required_test_mapping_cannot_name_an_unselected_package(self):
        self.manifest["required_tests_by_package"]["external-dependency"] = ["some_test"]
        with self.assertRaises(gate.GateError):
            gate.required_tests_for(self.manifest, gate.PACKAGES)

    def test_required_test_mapping_rejects_duplicates_and_non_names(self):
        for names in (["test", "test"], [None], "test", [""]):
            with self.subTest(names=names):
                self.manifest["required_tests_by_package"]["inventory-aws"] = names
                with self.assertRaises(gate.GateError):
                    gate.required_tests_for(self.manifest, gate.PACKAGES)


class FixtureSupervision(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.root = Path(self.folder.name)
        self.path = self.root / "mutation-command.json"
        self.log = self.root / "mutation-command.log"
        self.log.write_text("synthetic wrapper output\n")
        self.outer = {"argv": ["python", "run_root.py"], "workspace": "/synthetic/source",
                      "log": str(self.log), "status": "completed", "exit_code": 0,
                      "fixture_health_checks": 4, "log_sha256": gate.digest(self.log)}
        self.manifest = {"fixture_context": {
            "receipt": str(self.path), "argv_sha256": gate.hashlib.sha256(gate.canonical(self.outer["argv"])).hexdigest(),
            "workspace": self.outer["workspace"], "log": self.outer["log"],
        }}
        gate.write_json(self.path, self.outer)

    def tearDown(self):
        self.folder.cleanup()

    def test_healthy_completed_wrapper_is_bound_by_hash(self):
        result = gate.validate_fixture_context(self.manifest)
        self.assertEqual(result["status"], "verified")
        self.assertEqual(result["sha256"], gate.digest(self.path))

    def test_pending_wrapper_only_allowed_for_preliminary_child_report(self):
        self.outer["status"] = "running"
        gate.write_json(self.path, self.outer)
        self.assertEqual(gate.validate_fixture_context(self.manifest, allow_pending=True)["status"], "pending")
        with self.assertRaises(gate.GateError):
            gate.validate_fixture_context(self.manifest)

    def test_fixture_failure_blocks_even_otherwise_caught_mutants(self):
        self.outer.update(status="fixture_unavailable", exit_code=124)
        gate.write_json(self.path, self.outer)
        with self.assertRaises(gate.GateError):
            gate.validate_fixture_context(self.manifest)

    def test_outer_command_mismatch_cannot_reuse_an_unrelated_healthy_receipt(self):
        self.outer["argv"] = ["python", "only_baseline.py"]
        gate.write_json(self.path, self.outer)
        with self.assertRaises(gate.GateError):
            gate.validate_fixture_context(self.manifest)

    def test_missing_health_observations_prevent_acceptance(self):
        self.outer["fixture_health_checks"] = 0
        gate.write_json(self.path, self.outer)
        with self.assertRaises(gate.GateError):
            gate.validate_fixture_context(self.manifest)

    def test_wrapper_log_tampering_is_detected(self):
        self.log.write_text("changed after execution")
        with self.assertRaises(gate.GateError):
            gate.validate_fixture_context(self.manifest)

    def native_binding(self):
        fixture = self.root / "native-fixture.json"
        fixture.write_text('{"kind":"falkordb_native","binary_sha256":"synthetic"}\n')
        value = gate.digest(fixture)
        self.outer["status"] = "running"
        gate.write_json(self.path, self.outer)
        with mock.patch.dict(os.environ, {"INVENTORY_MUTATION_FIXTURE_SHA256": value,
                                         "INVENTORY_EMULATOR_SNAPSHOT_SHA256": ""}):
            context = gate.bind_fixture_context(self.path, "falkordb_native", fixture)
        manifest = {
            "fixture_context": context, "input_files": {str(fixture): value},
            "required_environment_sha256": {
                "INVENTORY_MUTATION_FIXTURE_SHA256": gate.hashlib.sha256(value.encode()).hexdigest(),
            },
        }
        self.outer["status"] = "completed"
        gate.write_json(self.path, self.outer)
        return fixture, manifest

    def test_native_fixture_is_explicitly_tagged_without_an_aws_snapshot(self):
        _, manifest = self.native_binding()
        result = gate.validate_fixture_context(manifest)
        self.assertEqual(result["status"], "verified")
        self.assertEqual(result["fixture_kind"], "falkordb_native")
        self.assertNotIn("snapshot_sha256", result)

    def test_native_context_cannot_omit_the_frozen_input_binding(self):
        _, manifest = self.native_binding()
        manifest["input_files"] = {}
        with self.assertRaises(gate.GateError):
            gate.validate_fixture_context(manifest)

    def test_native_context_cannot_omit_the_required_environment_binding(self):
        _, manifest = self.native_binding()
        manifest["required_environment_sha256"] = {}
        with self.assertRaises(gate.GateError):
            gate.validate_fixture_context(manifest)

    def test_native_freeze_rejects_a_mismatched_manifest_digest(self):
        fixture, _ = self.native_binding()
        self.outer["status"] = "running"
        gate.write_json(self.path, self.outer)
        with mock.patch.dict(os.environ, {"INVENTORY_MUTATION_FIXTURE_SHA256": "0" * 64}):
            with self.assertRaises(gate.GateError):
                gate.bind_fixture_context(self.path, "falkordb_native", fixture)

    def test_native_manifest_requires_an_active_outer_lifetime(self):
        fixture, _ = self.native_binding()
        with self.assertRaises(gate.GateError):
            gate.bind_fixture_context(None, "falkordb_native", fixture)
        with self.assertRaises(gate.GateError):
            gate.bind_fixture_context(self.path, "falkordb_native", fixture)


if __name__ == "__main__":
    unittest.main()
