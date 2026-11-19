#!/usr/bin/env python3
"""Run the full AWS partition inside mutate.py's verified local fixture context.

One disposable workspace and one Cargo target own every candidate. All five
real collector cases are required in both clean baselines and surviving-mutant
runs. An execution receipt is separate from the whole-project 95% quality gate.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess

import gate


REQUIRED_TESTS = gate.REQUIRED_LIVE_TESTS["inventory-aws"]
REQUIRED_ENVIRONMENT = gate.REQUIRED_LIVE_ENVIRONMENT["inventory-aws"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--campaign", type=Path, required=True)
    parser.add_argument("--snapshot", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--tool", type=Path, default=gate.DEFAULT_TOOL)
    parser.add_argument("--timeout", type=int, default=240)
    parser.add_argument("--build-timeout", type=int, default=600)
    parser.add_argument("--wall-timeout", type=int, default=86_400)
    args = parser.parse_args()
    try:
        if os.environ.get("INVENTORY_EMULATOR_TEST") != "1":
            raise gate.GateError("run_aws.py requires mutate.py's explicitly prepared emulator context")
        for name in REQUIRED_ENVIRONMENT:
            if not os.environ.get(name):
                raise gate.GateError(f"required fixture context variable is missing: {name}")
        snapshot_input = Path(os.environ["INVENTORY_EMULATOR_SNAPSHOT_PATH"]).resolve()
        if gate.digest(snapshot_input) != os.environ["INVENTORY_EMULATOR_SNAPSHOT_SHA256"]:
            raise gate.GateError("fixture snapshot hash does not match the checked context")
        expected = gate.load_json(snapshot_input)
        summary_input = Path(os.environ["INVENTORY_EMULATOR_BASELINE_SUMMARY"]).resolve()
        if gate.digest(summary_input) != expected["provenance"]["summary_sha256"]:
            raise gate.GateError("independent fixture summary does not match the frozen snapshot")
        if not (196 <= args.timeout <= 3_600 and 1 <= args.build_timeout <= 3_600
                and 1 <= args.wall_timeout <= 604_800):
            raise gate.GateError("timeouts must be bounded; test deadline must exceed the 195-second collector deadline")

        workspace = args.workspace.resolve()
        campaign = args.campaign.resolve()
        snapshot = args.snapshot.resolve()
        manifest = gate.freeze(
            workspace, campaign, ["inventory-aws"], args.tool.resolve(),
            required_tests=REQUIRED_TESTS,
            required_environment=REQUIRED_ENVIRONMENT,
            inputs=[snapshot_input, summary_input, Path(__file__).resolve(), Path(gate.__file__).resolve()],
            include_ignored=True,
            fixture_receipt=snapshot_input.parent / "mutation-command.json",
        )
        gate.snapshot_workspace(workspace, snapshot, campaign)
        print(json.dumps({
            "status": "frozen", "candidate_count": manifest["candidate_count"],
            "source_digest": manifest["source_digest"],
            "manifest": str(campaign / "manifest.json"), "snapshot": str(snapshot),
            "required_tests": list(REQUIRED_TESTS), "workers": 1,
        }, sort_keys=True), flush=True)
        execution = gate.run_partition(
            campaign, snapshot, args.tool.resolve(), None, args.target_dir.resolve(),
            args.timeout, args.build_timeout, args.wall_timeout,
        )
        report = gate.report_campaign(campaign, workspace, allow_pending_fixture=True)
        gate.write_json(campaign / "report.json", report)
        result = {
            "execution": execution, "candidate_count": report["candidate_count"],
            "counts": report["counts"], "score_percent": report["score_percent"],
            "score_requirement_met": report["score_requirement_met"],
            "whole_first_party_gate_passed": report["gate_passed"],
            "errors": report["errors"], "report": str(campaign / "report.json"),
            "semantics": "This is an AWS-package execution; only the full aggregate can satisfy the project gate.",
        }
        gate.write_json(campaign / "execution-summary.json", result)
        print(json.dumps(result, sort_keys=True), flush=True)
        return 0 if execution["source_restored"] and execution["exit_code"] in (0, 2, 3) and not report["errors"] else 1
    except (gate.GateError, OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        parser.exit(2, f"AWS mutation execution failed closed: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
