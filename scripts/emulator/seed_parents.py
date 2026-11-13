"""Seed independent parent-model read contracts through public emulator APIs.

The 63 semantic fixtures cover 21 types under two parents; 98 extra EKS addons
force the pinned emulator's 100-item default list window to paginate. Generic
property-store fixtures are explicitly not deployable CloudFormation templates.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import time

from botocore.config import Config

from assert_parents import load_fixtures
from transport import require_loopback

MAX_CALLS = 600


def seed(session, record=None):
    endpoint = os.environ["AWS_ENDPOINT_URL"]
    require_loopback(endpoint)
    fixture, fixture_sha = load_fixtures()
    if session.region_name != fixture["region"]:
        raise ValueError("parent fixtures require the pinned us-east-1 test region")
    config = Config(retries={"total_max_attempts": 1}, connect_timeout=3, read_timeout=10)
    cloud = session.client("cloudcontrol", endpoint_url=endpoint, verify=os.environ["AWS_CA_BUNDLE"], config=config)
    logs = session.client("logs", endpoint_url=endpoint, verify=os.environ["AWS_CA_BUNDLE"], config=config)
    outcomes, pending, failures = [], [], []
    calls = 0

    def call(operation, function):
        nonlocal calls
        if calls >= MAX_CALLS:
            raise ValueError("parent fixture API budget exceeded")
        calls += 1
        try:
            return record(operation, function) if record else function()
        except Exception as error:
            response = getattr(error, "response", {})
            failures.append({"operation": operation, "code": response.get("Error", {}).get("Code", type(error).__name__)})
            return None

    records = [("parent", row["key"], row) for row in fixture["parents"]]
    records += [("child", row["name"], row) for row in fixture["children"] + fixture["pagination_children"]]
    for role, key, row in records:
        type_name = row["type_name"]
        outcome = {"role": role, "name": key, "type_name": type_name, "status": "seed_failed",
                   "fixture_mode": row.get("fixture_mode", "synthetic_generic_read_contract")}
        outcomes.append(outcome)
        if row.get("fixture_mode") == "native_logs_bridge":
            name = row["provider"]["LogGroupName"]
            created = call("logs:CreateLogGroup:parent-fixture:" + key,
                           lambda n=name: logs.create_log_group(logGroupName=n))
            if created is not None:
                read = call("cloudcontrol:GetResource:parent-fixture-native:" + key,
                            lambda n=name: cloud.get_resource(TypeName="AWS::Logs::LogGroup", Identifier=n))
                if read and read.get("ResourceDescription", {}).get("Identifier") == name:
                    outcome.update(status="SUCCESS", identifier=read["ResourceDescription"]["Identifier"])
            continue
        token = "inventory-parent-" + hashlib.sha256((type_name + ":" + key).encode()).hexdigest()
        created = call("cloudcontrol:CreateResource:parent-fixture:" + key,
                       lambda row=row, token=token: cloud.create_resource(
                           TypeName=row["type_name"], DesiredState=json.dumps(row["provider"]), ClientToken=token))
        if created:
            event = created.get("ProgressEvent", {})
            outcome.update(status=event.get("OperationStatus", "unknown"), identifier=event.get("Identifier"),
                           error_code=event.get("ErrorCode"))
            if event.get("RequestToken"):
                pending.append((key, event["RequestToken"], outcome))
    # Pinned RESOURCE_OPERATION_SPAN is three seconds; one bounded batch settle
    # avoids a separate wait per resource. All APIs remain normal signed calls.
    if pending:
        time.sleep(3.1)
    for key, token, outcome in pending:
        status = call("cloudcontrol:GetResourceRequestStatus:parent-fixture:" + key,
                      lambda token=token: cloud.get_resource_request_status(RequestToken=token))
        if status:
            event = status.get("ProgressEvent", {})
            outcome.update(status=event.get("OperationStatus", "unknown"), identifier=event.get("Identifier"),
                           error_code=event.get("ErrorCode"))
    root = Path(__file__).resolve().parents[2]
    return {"status": "complete" if all(row["status"] == "SUCCESS" and row.get("identifier") for row in outcomes) else "partial",
            "fixture_semantics": fixture["fixture_semantics"], "fixture_sha256": fixture_sha,
            "seed_source_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "parent_catalog_sha256": hashlib.sha256((root / "catalog/cloudcontrol-parent-resources.json").read_bytes()).hexdigest(),
            "expected": {"parents": 12, "semantic_children": 63, "pagination_children": 98, "total_children": 161, "child_types": 21},
            "calls": calls, "failures": failures, "resources": outcomes}
