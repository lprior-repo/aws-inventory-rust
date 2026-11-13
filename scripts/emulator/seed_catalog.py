"""Populate Cloud Control's emulator store for every reviewed catalog type.

These are synthetic read-contract fixtures, NOT deployable CloudFormation
templates. The emulator uses an independent generic property store for most
types; passing these tests proves SDK/list/get/mapping plumbing, not provider
implementation fidelity. Native SQS/Logs bridges use their real seed APIs.
"""

import hashlib
import json
import os
import time
from pathlib import Path

from botocore.config import Config

from transport import require_loopback
from assert_cloud_control import golden_fixtures


def _put(properties, pointer, value):
    parts = [part.replace("~1", "/").replace("~0", "~") for part in pointer.strip("/").split("/")]
    if any(part == "*" for part in parts):
        raise ValueError("primary identifier cannot contain wildcard")
    current = properties
    for part in parts[:-1]:
        current = current.setdefault(part, {})
    current[parts[-1]] = value


def seed(session, record=None):
    endpoint = os.environ["AWS_ENDPOINT_URL"]
    require_loopback(endpoint)
    config = Config(retries={"total_max_attempts": 1}, connect_timeout=3, read_timeout=10)
    client = session.client("cloudcontrol", endpoint_url=endpoint, verify=os.environ["AWS_CA_BUNDLE"], config=config)
    logs = session.client("logs", endpoint_url=endpoint, verify=os.environ["AWS_CA_BUNDLE"], config=config)
    descriptors = json.loads((Path(__file__).resolve().parents[2] / "catalog/cloudcontrol-resources.json").read_text())["resource_types"]
    architecture, architecture_sha = golden_fixtures()
    architecture_by_type = {fixture["type_name"]: fixture for fixture in architecture}
    outcomes = []
    pending = []

    def call(name, function):
        if record:
            return record(name, function)
        try:
            return function()
        except Exception:
            return None

    # This bridge is implemented by the Logs service rather than a generic map.
    call("logs:CreateLogGroup", lambda: logs.create_log_group(logGroupName="inventory-emulator-catalog-log", tags={"Fixture": "inventory-emulator"}))
    call("logs:PutDestination", lambda: logs.put_destination(
        destinationName="inventory-emulator-catalog-destination",
        targetArn="arn:aws:kinesis:us-east-1:000000000000:stream/inventory-emulator-fixture",
        roleArn="arn:aws:iam::000000000000:role/inventory-emulator-read-role",
        tags={"Fixture": "inventory-emulator"}))
    for descriptor in descriptors:
        name = descriptor["type_name"]
        if name in {"AWS::SQS::Queue", "AWS::Logs::LogGroup", "AWS::Logs::Destination"}:
            listed = call("cloudcontrol:ListResources:seed-check:" + name,
                          lambda n=name: client.list_resources(TypeName=n, MaxResults=1))
            nonempty = bool(listed and listed.get("ResourceDescriptions"))
            outcomes.append({"type_name": name, "fixture_mode": "native_bridge",
                             "status": "verified_nonempty" if nonempty else "seed_failed"})
            continue
        suffix = hashlib.sha256(name.encode()).hexdigest()[:12]
        identifier = "inventory-emulator-" + suffix
        arn = f"arn:aws:{descriptor['service']}:us-east-1:000000000000:fixture/{identifier}"
        fixture = architecture_by_type.get(name)
        if fixture:
            properties = dict(fixture["provider"])
            # The generic emulator provider chooses names heuristically. This
            # explicit fallback makes a compound fixture identifier repeatable;
            # it does not pretend these fixtures are deployable AWS templates.
            properties.setdefault("Name", fixture["identifier"])
        else:
            properties = {"Name": identifier}
            for pointer in descriptor["primary_identifier"]:
                key = pointer.rsplit("/", 1)[-1].lower()
                _put(properties, pointer, arn if "arn" in key or descriptor.get("identifier_is_arn") else identifier)
            for pointer in descriptor.get("arn_pointers", []):
                if "*" not in pointer:
                    _put(properties, pointer, arn)
        response = call("cloudcontrol:CreateResource:" + name, lambda n=name, p=properties: client.create_resource(
            TypeName=n, DesiredState=json.dumps(p), ClientToken="inventory-emulator-" + suffix))
        outcome = {"type_name": name, "fixture_mode": "generic_emulator_property_store", "status": "seed_failed"}
        if response:
            event = response.get("ProgressEvent", {})
            token = event.get("RequestToken")
            outcome.update(status=event.get("OperationStatus", "unknown"), identifier=event.get("Identifier"),
                           error_code=event.get("ErrorCode"), status_message=event.get("StatusMessage"))
            if token:
                pending.append((name, token, outcome))
        outcomes.append(outcome)
    # Reviewed upstream RESOURCE_OPERATION_SPAN is 3s. Create the bounded set
    # first, then settle it once, instead of sleeping/polling each serial job.
    if pending:
        time.sleep(3.1)
    for name, token, outcome in pending:
        status = call("cloudcontrol:GetResourceRequestStatus:" + name,
                      lambda t=token: client.get_resource_request_status(RequestToken=t))
        if status:
            event = status.get("ProgressEvent", {})
            outcome.update(status=event.get("OperationStatus", "unknown"), identifier=event.get("Identifier"),
                           error_code=event.get("ErrorCode"), status_message=event.get("StatusMessage"))
    return {"status": "complete" if all(row["status"] in {"SUCCESS", "verified_nonempty"} for row in outcomes) else "partial",
            "fixture_semantics": "synthetic generic read contracts; no claim of live provider schema validation",
            "architecture_fixtures": {"fixture_sha256": architecture_sha,
                                      "names": sorted(fixture["name"] for fixture in architecture)},
            "resource_types": outcomes}
