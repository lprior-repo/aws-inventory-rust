"""Synthetic parser fixtures: exercise final acceptance, never run Cargo or AWS."""

from __future__ import annotations

import copy
import contextlib
import hashlib
import importlib.util
import io
import os
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location("acceptance_gate", Path(__file__).with_name("gate.py"))
assert SPEC is not None and SPEC.loader is not None
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)

# Literal public acceptance contract: do not derive this oracle from gate.py.
LIVE_TESTS = {
    "aws-inventory": [
        "native::emulator_cli_native_snapshot_reports_and_graph_delivery",
        "worker::emulator_cli_durable_worker_persists_an_assumed_role_scan",
    ],
    "inventory-core": [],
    "inventory-engine": [
        "live_falkordb_replay_and_parameter_round_trip",
        "live_falkordb_rejected_batch_is_never_acknowledged_or_finalized::live_falkordb_rejected_batch_is_never_acknowledged_or_finalized",
        "live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint::live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint",
    ],
    "inventory-aws": [
        "emulator_snapshot_native", "emulator_snapshot_sources",
        "emulator_snapshot_cloudcontrol", "emulator_snapshot_assumedrole",
        "emulator_snapshot_parents",
    ],
}


def sha_text(value):
    return hashlib.sha256(value.encode()).hexdigest()


def phase(packages, build, ignored, status="Success"):
    argv = ["cargo", "test", "--verbose"] + (["--no-run"] if build else [])
    argv += [f"--package={package}@0.5.0" for package in packages]
    argv += ["--all-features", "--locked", "--offline", "-j1"]
    if ignored and not build:
        argv += ["--", "--include-ignored"]
    return {"phase": "Build" if build else "Test", "process_status": status, "argv": argv}


class ReceiptFixture:
    """A partition of four fake candidates with complete integrity metadata."""

    def __init__(self, root, live=True, packages=gate.PACKAGES,
                 fixture_kind="aws_emulator", fixture_value="fixture snapshot"):
        self.root = root
        self.campaign = root / "artifacts/campaign"
        self.run = self.campaign / "runs/0001"
        self.output = self.run / "mutants.out"
        (self.output / "log").mkdir(parents=True)
        self.universe = []
        for number, package in enumerate(LIVE_TESTS, 1):
            relative = ("src/main.rs" if package == "aws-inventory"
                        else gate.PACKAGE_DIRS[package] + "/src/lib.rs")
            source = root / relative
            source.parent.mkdir(parents=True)
            source.write_text("// synthetic integrity fixture; never compiled\n")
            self.universe.append({"name": f"synthetic {package}", "package": package,
                                  "file": relative, "function": None, "span": {"start": number},
                                  "replacement": "false", "genre": "FnValue"})
        self.candidates = [candidate for candidate in self.universe if candidate["package"] in packages]
        gate.write_json(self.campaign / "candidates.json", self.candidates)
        files = gate.closure_files(root, packages)
        self.manifest = {
            "schema_version": 1, "baseline_required": True, "packages": list(packages),
            "workspace_origin": str(root), "mutation_options": gate.mutation_options(packages, live),
            "tool": {"version": gate.VERSION, "binary_sha256": gate.TOOL_SHA256},
            "environment": gate.ENVIRONMENT, "test_scope": "mutated-package", "score_threshold_percent": 95,
            "source_files": files, "source_digest": gate.source_digest(files),
            "candidate_file": "candidates.json", "candidate_file_sha256": gate.digest(self.campaign / "candidates.json"),
            "candidate_count": len(self.candidates), "candidate_ids": list(gate.candidate_map(self.candidates)),
            "include_ignored": live, "baseline_runs_required": 2 if live else 1,
            "required_tests_by_package": {package: copy.deepcopy(LIVE_TESTS[package]) for package in packages} if live else {},
            "required_environment_sha256": {}, "input_files": {}, "fixture_context": None,
        }
        if live:
            self.bind_fixture(fixture_kind, fixture_value)
        names = [name for package in packages for name in LIVE_TESTS[package]] if live else ["ordinary_test"]
        passed = "".join(f"test {name} ... ok\n" for name in names)
        passed += f"test result: ok. {len(names)} passed; 0 failed; 0 ignored;\n"
        (self.output / "log/baseline.log").write_text(passed)
        baseline = {"scenario": "Baseline", "summary": "Success", "log_path": "log/baseline.log",
                    "phase_results": [phase(packages, True, live), phase(packages, False, live)]}
        outcomes = [baseline]
        for number, candidate in enumerate(self.candidates):
            name = f"log/mutant-{number}.log"
            (self.output / name).write_text("test assertion ... FAILED\ntest result: FAILED. 0 passed; 1 failed;\n")
            outcomes.append({"scenario": {"Mutant": candidate}, "summary": "CaughtMutant", "log_path": name,
                             "phase_results": [phase([candidate["package"]], True, live),
                                               phase([candidate["package"]], False, live, {"Failure": 101})]})
        gate.write_json(self.output / "mutants.json", self.candidates)
        gate.write_json(self.output / "outcomes.json", {
            "cargo_mutants_version": gate.VERSION, "end_time": "2026-11-13T00:00:00Z",
            "total_mutants": len(self.candidates), "caught": len(self.candidates),
            "missed": 0, "timeout": 0, "unviable": 0, "success": 0,
            "outcomes": outcomes,
        })
        self.receipt = {
            "status": "finished", "exit_code": 2, "tool_binary_sha256": gate.TOOL_SHA256,
            "environment": gate.ENVIRONMENT, "source_digest_before": self.manifest["source_digest"],
            "source_digest_after": self.manifest["source_digest"], "source_restored": True,
            "candidate_ids": self.manifest["candidate_ids"], "shard": None,
        }
        if live:
            (self.run / "preflight-baseline.log").write_text(passed)
            self.receipt["preflight_baseline"] = {**baseline, "log_path": "preflight-baseline.log"}
        self.save()

    def bind_fixture(self, kind, payload):
        log = self.root / "artifacts/fixture.log"
        log.write_text("synthetic healthy fixture receipt\n")
        receipt = self.root / "artifacts/fixture.json"
        outer = {"status": "completed", "exit_code": 0, "fixture_health_checks": 2,
                 "argv": ["synthetic fixture process"], "workspace": str(self.root),
                 "log": str(log), "log_sha256": gate.digest(log)}
        gate.write_json(receipt, outer)
        context = {"fixture_kind": kind, "receipt": str(receipt),
                   "argv_sha256": hashlib.sha256(gate.canonical(outer["argv"])).hexdigest(),
                   "workspace": str(self.root), "log": str(log)}
        artifact = self.root / ("artifacts/snapshot.json" if kind == "aws_emulator" else "artifacts/native-fixture.json")
        gate.write_json(artifact, {"synthetic": payload})
        value = gate.digest(artifact)
        environment = {"FALKORDB_TEST_URL": "synthetic local endpoint"}
        if kind == "aws_emulator":
            environment.update({name: "synthetic fixture value" for name in gate.AWS_FIXTURE_ENVIRONMENT})
            environment.update(INVENTORY_EMULATOR_SNAPSHOT_PATH=str(artifact),
                               INVENTORY_EMULATOR_SNAPSHOT_SHA256=value)
            context["snapshot_sha256"] = value
        else:
            environment["INVENTORY_MUTATION_FIXTURE_SHA256"] = value
            context.update(fixture_manifest=str(artifact), fixture_sha256=value)
        self.manifest["required_environment_sha256"] = {key: sha_text(text) for key, text in environment.items()}
        self.manifest["input_files"] = {str(artifact): value}
        self.manifest["fixture_context"] = context

    def save(self):
        gate.write_json(self.campaign / "manifest.json", self.manifest)
        self.receipt.update(manifest_sha256=gate.digest(self.campaign / "manifest.json"),
                            required_environment_sha256=self.manifest["required_environment_sha256"],
                            input_files_before=self.manifest["input_files"], input_files_after=self.manifest["input_files"])
        evidence = {path.relative_to(self.run).as_posix(): gate.digest(path)
                    for path in self.output.rglob("*") if path.is_file()}
        if (self.run / "preflight-baseline.log").exists():
            evidence["preflight-baseline.log"] = gate.digest(self.run / "preflight-baseline.log")
        self.receipt["evidence_files"] = evidence
        gate.write_json(self.run / "receipt.json", self.receipt)

    def aggregate(self, campaigns=None):
        def enumerate_candidates(_root, _packages, _tool, output):
            gate.write_json(output, self.universe)
            return self.universe

        with mock.patch.object(gate, "tool_info"), mock.patch.object(gate, "enumerate_candidates", enumerate_candidates):
            return gate.aggregate(self.root, campaigns or [self.campaign], self.root / "synthetic-tool",
                                  self.campaign / "final-universe.json")


class FinalAcceptance(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def test_fixed_policy_contains_all_ten_named_live_tests(self):
        self.assertEqual({package: list(names) for package, names in gate.REQUIRED_LIVE_TESTS.items()}, LIVE_TESTS)
        self.assertEqual(sum(len(names) for names in LIVE_TESTS.values()), 10)

    def test_complete_live_receipts_can_pass_the_real_aggregate_parser(self):
        fixture = ReceiptFixture(self.root)
        report = fixture.aggregate()
        self.assertTrue(report["gate_passed"], report["errors"])
        self.assertEqual(report["candidate_count"], 4)
        self.assertEqual(report["counts"]["caught"], 4)

    def test_repeated_same_fixture_preserves_attempts_without_inflating_score(self):
        first = ReceiptFixture(self.root / "first")
        second = ReceiptFixture(self.root / "second")
        report = first.aggregate([first.campaign, second.campaign])
        self.assertTrue(report["gate_passed"], report["errors"])
        self.assertEqual(report["candidate_count"], 4)
        self.assertEqual(report["counts"]["caught"], 4)
        self.assertEqual(report["fixture_identities"], {
            "aws_emulator": [first.manifest["fixture_context"]["snapshot_sha256"]],
        })
        self.assertTrue(all(len(row["attempts"]) == 2 for row in report["mutants"]))

    def test_mixed_aws_snapshots_cannot_pass_even_when_all_mutants_are_caught(self):
        first = ReceiptFixture(self.root / "first")
        second = ReceiptFixture(self.root / "second", fixture_value="different frozen snapshot")
        # The legacy omitted kind still denotes AWS and must not evade comparison.
        del second.manifest["fixture_context"]["fixture_kind"]
        second.save()
        report = first.aggregate([first.campaign, second.campaign])
        self.assertFalse(report["gate_passed"])
        self.assertFalse(report["complete"])
        self.assertEqual(report["candidate_count"], 4)
        self.assertEqual(report["counts"]["caught"], 4)
        self.assertTrue(all(len(row["attempts"]) == 2 for row in report["mutants"]))
        self.assertEqual(len(report["fixture_identities"]["aws_emulator"]), 2)
        self.assertEqual(len(report["errors"]), 1, report["errors"])
        self.assertIn("multiple frozen aws_emulator fixture identities", report["errors"][0])

    def test_mixed_native_fixtures_cannot_pass_with_valid_aws_partitions(self):
        aws = ReceiptFixture(self.root / "aws", packages=["aws-inventory", "inventory-core", "inventory-aws"])
        first = ReceiptFixture(self.root / "native-first", packages=["inventory-engine"],
                               fixture_kind="falkordb_native", fixture_value="first native fixture")
        second = ReceiptFixture(self.root / "native-second", packages=["inventory-engine"],
                                fixture_kind="falkordb_native", fixture_value="second native fixture")
        report = aws.aggregate([aws.campaign, first.campaign, second.campaign])
        self.assertFalse(report["gate_passed"])
        self.assertFalse(report["complete"])
        self.assertEqual(report["counts"]["caught"], 4)
        self.assertEqual(len(report["fixture_identities"]["aws_emulator"]), 1)
        self.assertEqual(len(report["fixture_identities"]["falkordb_native"]), 2)
        self.assertEqual(len(report["errors"]), 1, report["errors"])
        self.assertIn("multiple frozen falkordb_native fixture identities", report["errors"][0])

    def test_separate_aws_native_and_fixture_free_core_partitions_are_supported(self):
        aws = ReceiptFixture(self.root / "aws", packages=["aws-inventory", "inventory-aws"])
        native = ReceiptFixture(self.root / "native", packages=["inventory-engine"],
                                fixture_kind="falkordb_native", fixture_value="native fixture")
        core = ReceiptFixture(self.root / "core", live=False, packages=["inventory-core"])
        report = aws.aggregate([aws.campaign, native.campaign, core.campaign])
        self.assertTrue(report["gate_passed"], report["errors"])
        self.assertEqual(report["counts"]["caught"], 4)
        self.assertEqual(report["fixture_identities"], {
            "aws_emulator": [aws.manifest["fixture_context"]["snapshot_sha256"]],
            "falkordb_native": [native.manifest["fixture_context"]["fixture_sha256"]],
        })

    def test_unit_only_complete_receipts_cannot_pass_whole_project(self):
        fixture = ReceiptFixture(self.root, live=False)
        # These are legitimate partition diagnostics, but cannot prove final acceptance.
        self.assertTrue(gate.report_campaign(fixture.campaign, self.root)["complete"])
        report = fixture.aggregate()
        self.assertFalse(report["gate_passed"])
        self.assertTrue(any("ignored live tests" in error for error in report["errors"]))

    def test_omitting_any_one_of_ten_tests_blocks_acceptance(self):
        fixture = ReceiptFixture(self.root)
        for package, names in LIVE_TESTS.items():
            for name in names:
                manifest = copy.deepcopy(fixture.manifest)
                manifest["required_tests_by_package"][package].remove(name)
                with self.subTest(name=name), self.assertRaisesRegex(gate.GateError, "mandatory live test"):
                    gate.validate_acceptance_policy(manifest)

    def test_omitted_test_rejected_through_full_artifact_parser(self):
        fixture = ReceiptFixture(self.root)
        fixture.manifest["required_tests_by_package"]["inventory-engine"].pop()
        fixture.save()
        report = fixture.aggregate()
        self.assertFalse(report["gate_passed"])
        self.assertTrue(any("mandatory live test" in error for error in report["errors"]))

    def test_unbound_fixture_lifetime_cannot_establish_acceptance(self):
        fixture = ReceiptFixture(self.root)
        fixture.manifest["fixture_context"] = None
        fixture.save()
        report = fixture.aggregate()
        self.assertFalse(report["gate_passed"])
        self.assertTrue(any("supervised fixture lifetime" in error for error in report["errors"]))

    def test_native_falkor_fixture_cannot_substitute_for_aws(self):
        fixture = ReceiptFixture(self.root)
        fixture.manifest["fixture_context"]["fixture_kind"] = "falkordb_native"
        with self.assertRaisesRegex(gate.GateError, "AWS emulator fixture"):
            gate.validate_acceptance_policy(fixture.manifest)

    def test_core_partition_does_not_require_unrelated_live_tests(self):
        gate.validate_acceptance_policy({"packages": ["inventory-core"]})

    def test_native_engine_partition_can_satisfy_its_own_live_policy(self):
        gate.validate_acceptance_policy({
            "packages": ["inventory-engine"], "include_ignored": True, "baseline_runs_required": 2,
            "required_tests_by_package": {"inventory-engine": LIVE_TESTS["inventory-engine"]},
            "fixture_context": {"fixture_kind": "falkordb_native"},
            "required_environment_sha256": {"FALKORDB_TEST_URL": sha_text("synthetic")},
        })

    def test_each_required_environment_value_must_be_bound(self):
        fixture = ReceiptFixture(self.root)
        for name in fixture.manifest["required_environment_sha256"]:
            manifest = copy.deepcopy(fixture.manifest)
            del manifest["required_environment_sha256"][name]
            with self.subTest(name=name), self.assertRaises(gate.GateError):
                gate.validate_acceptance_policy(manifest)

    def test_snapshot_hash_must_match_frozen_environment_value(self):
        fixture = ReceiptFixture(self.root)
        fixture.manifest["fixture_context"]["snapshot_sha256"] = "f" * 64
        with self.assertRaisesRegex(gate.GateError, "digest is not bound"):
            gate.validate_acceptance_policy(fixture.manifest)

    def test_snapshot_content_without_matching_path_binding_is_insufficient(self):
        fixture = ReceiptFixture(self.root)
        value = fixture.manifest["fixture_context"]["snapshot_sha256"]
        fixture.manifest["input_files"] = {"/different/snapshot.json": value}
        with self.assertRaisesRegex(gate.GateError, "path and content"):
            gate.validate_acceptance_policy(fixture.manifest)

    def test_snapshot_path_without_matching_content_binding_is_insufficient(self):
        fixture = ReceiptFixture(self.root)
        fixture.manifest["input_files"] = {name: "f" * 64 for name in fixture.manifest["input_files"]}
        with self.assertRaisesRegex(gate.GateError, "path and content"):
            gate.validate_acceptance_policy(fixture.manifest)

    def test_single_baseline_cannot_satisfy_final_acceptance(self):
        fixture = ReceiptFixture(self.root)
        fixture.manifest["baseline_runs_required"] = 1
        with self.assertRaisesRegex(gate.GateError, "two clean baselines"):
            gate.validate_acceptance_policy(fixture.manifest)


class EmbeddedDataIntegrity(unittest.TestCase):
    def check_change(self, selected, relative):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            path = root / relative
            path.parent.mkdir(parents=True)
            path.write_text("original embedded data\n")
            before = gate.closure_files(root, [selected])
            self.assertIn(relative, before)
            path.write_text("changed embedded data\n")
            self.assertNotEqual(before, gate.closure_files(root, [selected]))

    def test_aws_partition_binds_dependency_migrations(self):
        self.check_change("inventory-aws", "crates/engine/sql/005_cloudcontrol_expansion.sql")

    def test_root_partition_binds_dependency_migrations(self):
        self.check_change("aws-inventory", "crates/engine/sql/jobs.sql")

    def test_root_partition_binds_embedded_sdk_service_manifest(self):
        self.check_change("aws-inventory", "crates/sdk-catalog/sdk-services.json")


class CompilerEvidence(unittest.TestCase):
    def classify(self, log):
        outcome = {"summary": "Unviable", "phase_results": [phase(["inventory-core"], True, False, {"Failure": 101})]}
        return gate.classify_outcome(outcome, log, {"inventory-core"})[0]

    def test_linker_failure_is_unresolved(self):
        self.assertEqual(self.classify('error: linking with `cc` failed: exit status: 1\n'), "unresolved")

    def test_build_script_failure_is_unresolved(self):
        self.assertEqual(self.classify('error: failed to run custom build command for `dependency`\n'), "unresolved")

    def test_rustc_ice_is_unresolved_even_with_an_error_code(self):
        self.assertEqual(self.classify('error[E0308]: mismatched types\nerror: internal compiler error\n'), "unresolved")

    def test_dependency_compiler_version_mismatch_is_unresolved(self):
        log = 'error[E0514]: found crate `dependency` compiled by an incompatible version of rustc\n'
        self.assertEqual(self.classify(log), "unresolved")
        self.assertEqual(self.classify('error[E0308]: mismatched types\n' + log), "unresolved")

    def test_invalid_crate_metadata_is_unresolved(self):
        self.assertEqual(self.classify('error[E0786]: found invalid metadata files for crate `dependency`\n'),
                         "unresolved")

    def test_missing_standard_library_or_core_is_unresolved(self):
        for message in ["can't find crate for `std`", "can't find crate for `core`", "cannot find crate for 'std'"]:
            with self.subTest(message=message):
                self.assertEqual(self.classify(f"error[E0463]: {message}\n"), "unresolved")

    def test_missing_application_crate_can_still_be_source_unviability(self):
        for name in ["mutated_dependency", "core_extra", "std_extra"]:
            with self.subTest(name=name):
                self.assertEqual(self.classify(f"error[E0463]: can't find crate for `{name}`\n"), "unviable")

    def test_cargo_summary_alone_does_not_prove_unviability(self):
        self.assertEqual(self.classify('error: could not compile `inventory-core`\n'), "unresolved")

    def test_explicit_rustc_type_diagnostic_remains_unviable(self):
        self.assertEqual(self.classify('error[E0308]: mismatched types\n'), "unviable")


class PackageRunnerPolicy(unittest.TestCase):
    class FreezeReached(Exception):
        """Stop before any Cargo operation or mutation snapshot is created."""

    def load_runner(self, name):
        spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + ".py"))
        self.assertIsNotNone(spec)
        self.assertIsNotNone(spec.loader)
        module = importlib.util.module_from_spec(spec)
        with mock.patch.dict(sys.modules, {"gate": gate}):
            spec.loader.exec_module(module)
        return module

    def runner_inputs(self, root):
        summary = root / "summary.json"
        gate.write_json(summary, {"synthetic": "summary"})
        snapshot = root / "snapshot.json"
        gate.write_json(snapshot, {"provenance": {"summary_sha256": gate.digest(summary)}})
        environment = {name: "synthetic" for name in gate.AWS_FIXTURE_ENVIRONMENT}
        environment.update(INVENTORY_EMULATOR_TEST="1", INVENTORY_EMULATOR_SNAPSHOT_PATH=str(snapshot),
                           INVENTORY_EMULATOR_SNAPSHOT_SHA256=gate.digest(snapshot),
                           INVENTORY_EMULATOR_BASELINE_SUMMARY=str(summary), FALKORDB_TEST_URL="synthetic")
        argv = ["synthetic-runner", "--workspace", str(root), "--campaign", str(root / "campaign"),
                "--snapshot", str(root / "disposable"), "--target-dir", str(root / "target")]
        return environment, argv, summary, snapshot

    def test_root_and_aws_runners_freeze_the_full_shared_acceptance_policy(self):
        for name, package in [("run_root", "aws-inventory"), ("run_aws", "inventory-aws")]:
            with self.subTest(runner=name), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                module = self.load_runner(name)
                environment, argv, summary, snapshot = self.runner_inputs(root)
                with mock.patch.dict(os.environ, environment), mock.patch.object(sys, "argv", argv), \
                        mock.patch.object(gate, "freeze", side_effect=self.FreezeReached) as freeze:
                    with self.assertRaises(self.FreezeReached):
                        module.main()
                keywords = freeze.call_args.kwargs
                self.assertEqual(list(keywords["required_tests"]), LIVE_TESTS[package])
                self.assertEqual(keywords["required_environment"], gate.REQUIRED_LIVE_ENVIRONMENT[package])
                self.assertTrue(keywords["include_ignored"])
                self.assertTrue({summary, snapshot, Path(gate.__file__).resolve()} <= set(keywords["inputs"]))

    def test_root_and_aws_runners_reject_changed_summary_before_freezing(self):
        for name in ["run_root", "run_aws"]:
            with self.subTest(runner=name), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                module = self.load_runner(name)
                environment, argv, summary, _snapshot = self.runner_inputs(root)
                gate.write_json(summary, {"synthetic": "changed summary"})
                with mock.patch.dict(os.environ, environment), mock.patch.object(sys, "argv", argv), \
                        mock.patch.object(gate, "freeze") as freeze, contextlib.redirect_stderr(io.StringIO()):
                    with self.assertRaises(SystemExit) as caught:
                        module.main()
                self.assertEqual(caught.exception.code, 2)
                freeze.assert_not_called()

    def test_verified_engine_runner_requires_all_three_native_cases(self):
        module = self.load_runner("run_engine_verified")
        args = SimpleNamespace(campaign=Path("/synthetic/campaign"), scratch=Path("/synthetic/disposable"),
                               redis_server=Path("/synthetic/redis-server"), falkor_module=Path("/synthetic/falkor.so"))
        # The legacy worker module imports emulator fixture dependencies. The
        # test stops at freeze and must not require boto3 or start that worker.
        worker = SimpleNamespace(__file__=str(Path(__file__).with_name("run_engine.py")))
        with mock.patch.dict(sys.modules, {"run_engine": worker}), mock.patch.object(module.signal, "signal"), \
                mock.patch.object(gate, "freeze", side_effect=self.FreezeReached) as freeze:
            with self.assertRaises(self.FreezeReached):
                module.run_frozen(args, Path("/synthetic/fixture"))
        self.assertEqual(list(freeze.call_args.kwargs["required_tests"]), LIVE_TESTS["inventory-engine"])
        self.assertEqual(freeze.call_args.kwargs["fixture_kind"], "falkordb_native")
        self.assertIn(Path(gate.__file__), freeze.call_args.kwargs["inputs"])


if __name__ == "__main__":
    unittest.main()
