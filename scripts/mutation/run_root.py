#!/usr/bin/env python3
"""Freeze and run the complete root partition inside mutate.py's fixture context.

Exit status describes execution/evidence validity. The separate whole-project
aggregate remains the only command that can assert the 95% quality gate.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path

import gate


REQUIRED_TESTS = gate.REQUIRED_LIVE_TESTS["aws-inventory"]
REQUIRED_ENVIRONMENT = gate.REQUIRED_LIVE_ENVIRONMENT["aws-inventory"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--campaign", type=Path, required=True)
    parser.add_argument("--snapshot", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--tool", type=Path, default=gate.DEFAULT_TOOL)
    parser.add_argument("--timeout", type=int, default=120)
    parser.add_argument("--build-timeout", type=int, default=300)
    parser.add_argument("--wall-timeout", type=int, default=86400)
    args = parser.parse_args()
    try:
        if os.environ.get("INVENTORY_EMULATOR_TEST") != "1":
            raise gate.GateError("run_root.py requires mutate.py's explicitly prepared emulator context")
        for name in REQUIRED_ENVIRONMENT:
            if not os.environ.get(name):
                raise gate.GateError(f"required fixture context variable is missing: {name}")
        snapshot_input = Path(os.environ["INVENTORY_EMULATOR_SNAPSHOT_PATH"]).resolve()
        if gate.digest(snapshot_input) != os.environ["INVENTORY_EMULATOR_SNAPSHOT_SHA256"]:
            raise gate.GateError("fixture snapshot hash does not match the checked context")
        fixture = gate.load_json(snapshot_input)
        summary_input = Path(os.environ["INVENTORY_EMULATOR_BASELINE_SUMMARY"]).resolve()
        if gate.digest(summary_input) != fixture["provenance"]["summary_sha256"]:
            raise gate.GateError("independent fixture summary does not match the frozen snapshot")
        workspace = args.workspace.resolve()
        campaign = args.campaign.resolve()
        snapshot = args.snapshot.resolve()
        manifest = gate.freeze(
            workspace, campaign, ["aws-inventory"], args.tool.resolve(),
            required_tests=REQUIRED_TESTS, required_environment=REQUIRED_ENVIRONMENT,
            inputs=[snapshot_input, summary_input, Path(__file__).resolve(), Path(gate.__file__).resolve()],
            include_ignored=True,
            fixture_receipt=snapshot_input.parent / "mutation-command.json",
        )
        gate.snapshot_workspace(workspace, snapshot, campaign)
        print(json.dumps({"status": "frozen", "candidate_count": manifest["candidate_count"],
                          "source_digest": manifest["source_digest"],
                          "manifest": str(campaign / "manifest.json"), "snapshot": str(snapshot)},
                         sort_keys=True), flush=True)
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
            "semantics": "This is a complete root-package execution, not the whole-project mutation gate.",
        }
        gate.write_json(campaign / "execution-summary.json", result)
        print(json.dumps(result, sort_keys=True), flush=True)
        return 0 if execution["source_restored"] and execution["exit_code"] in (0, 2, 3) and not report["errors"] else 1
    except (gate.GateError, OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(2, f"root mutation execution failed closed: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
