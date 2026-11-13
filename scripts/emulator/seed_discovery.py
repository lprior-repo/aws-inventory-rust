"""Seed discovery and supplemental collectors against a local AWS Emulator.

``seed(session, record=None)`` takes a boto3 Session.  The optional recorder has
the signature ``record("service:Operation", callable)`` and must invoke the
callable exactly once.  The return value is a JSON-serializable report; no
credentials or complete SDK responses are retained in it.

Every client uses the explicit AWS_ENDPOINT_URL, which must be HTTPS on a
literal loopback address and a nonzero port.  An STS identity check precedes
all fixture writes.  These are test fixture mutations, never scan behavior.
Restart the emulator to remove the fixtures and obtain a clean test run.

Source reviewed: aws/aws-emulator f515581569b98a9dcbd2e686c12f912fdce51bba.
Organizations ListAccounts/ListChildren are unimplemented at that revision;
the seed preserves these as known gaps instead of fabricating account data.
"""

from __future__ import annotations

import ipaddress
import json
import os
import time
from typing import Any, Callable
from urllib.parse import urlsplit


PREFIX = "inventory-emulator"
ACCOUNT = "000000000000"
CONFIG_AGGREGATOR = f"{PREFIX}-aggregator"
INVENTORY_ROLE = f"{PREFIX}-read-role"
CONFIG_ROLE = f"{PREFIX}-config-role"
MAX_POLLS = 40
POLL_SECONDS = 0.25


def _endpoint(value: str | None) -> str:
    """Fail closed before constructing clients or looking up credentials."""
    parsed = urlsplit(value or "")
    try:
        loopback = ipaddress.ip_address(parsed.hostname or "").is_loopback
        port = parsed.port
    except ValueError as error:
        raise ValueError("AWS_ENDPOINT_URL must use a literal loopback IP") from error
    if (
        parsed.scheme != "https"
        or not loopback
        or port is None
        or port == 0
        or parsed.username is not None
        or parsed.password is not None
        or parsed.path not in ("", "/")
        or parsed.query
        or parsed.fragment
    ):
        raise ValueError("AWS_ENDPOINT_URL must be https://<loopback-ip>:<port>")
    return (value or "").rstrip("/")


def seed(session: Any, record: Callable[..., Any] | None = None) -> dict[str, Any]:
    """Create bounded fixtures and return independently recorded call outcomes."""
    endpoint = _endpoint(os.environ.get("AWS_ENDPOINT_URL"))
    from botocore.config import Config

    region = session.region_name or os.environ.get("AWS_REGION", "us-east-1")
    report: dict[str, Any] = {
        "endpoint": endpoint,
        "region": region,
        "fixtures": {},
        "calls": [],
        "known_emulator_gaps": [
            "Organizations ListAccounts and ListChildren return NotImplemented at source f5155815.",
            "Organizations CreateAccount is not implemented; no member accounts are fabricated.",
            "Resource Explorer indexes SQS/SNS and its own index/view resources only.",
            "The Config recorder auto-observes S3 buckets, SQS queues, and IAM roles only; global IAM recording must be enabled explicitly.",
            "Cloud Control native bridges cover Logs LogGroup/Destination and SQS Queue; other types use its own registry.",
            "Config PutResourceConfig fixtures are stored configuration records, not native DynamoDB resources.",
            "Multi-account isolation, production IAM semantics, and real AWS throttling are outside this test's claims.",
        ],
    }
    fixtures = report["fixtures"]
    client_config = Config(
        connect_timeout=3,
        read_timeout=10,
        retries={"mode": "standard", "total_max_attempts": 1},
        s3={"addressing_style": "path"},
    )
    clients: dict[str, Any] = {}

    def client(service: str) -> Any:
        if service not in clients:
            clients[service] = session.client(
                service,
                endpoint_url=endpoint,
                region_name=region,
                config=client_config,
                verify=os.environ.get("AWS_CA_BUNDLE") or True,
            )
        return clients[service]

    def call(service: str, method: str, **arguments: Any) -> dict[str, Any] | None:
        sdk = client(service)
        operation = sdk.meta.method_to_api_mapping[method]
        name = f"{service}:{operation}"
        captured: dict[str, Any] = {}
        invoked = False

        def invoke() -> dict[str, Any]:
            nonlocal invoked
            if invoked:
                raise ValueError("The recorder must invoke each fixture call exactly once")
            invoked = True
            started = time.monotonic()
            try:
                response = getattr(sdk, method)(**arguments)
            except Exception as error:
                detail = getattr(error, "response", {})
                metadata = detail.get("ResponseMetadata", {})
                report["calls"].append({
                    "operation": name,
                    "status": "error",
                    "error_code": detail.get("Error", {}).get("Code", type(error).__name__),
                    "http_status": metadata.get("HTTPStatusCode"),
                    "duration_ms": round((time.monotonic() - started) * 1000, 2),
                })
                raise
            captured["response"] = response
            report["calls"].append({
                "operation": name,
                "status": "ok",
                "http_status": response.get("ResponseMetadata", {}).get("HTTPStatusCode"),
                "duration_ms": round((time.monotonic() - started) * 1000, 2),
            })
            return response

        try:
            if record is None:
                invoke()
            else:
                record(name, invoke)
        except Exception:
            # Keep independent fixture groups running after an emulator gap.
            pass
        if not invoked:
            report["calls"].append({
                "operation": name, "status": "error", "error_code": "RecorderDidNotInvokeCall",
            })
        return captured.get("response")

    identity = call("sts", "get_caller_identity")
    if identity is None or identity.get("Account") != ACCOUNT:
        report["status"] = "refused_identity"
        report["reason"] = "The loopback emulator did not report account 000000000000."
        return report
    fixtures["account_id"] = ACCOUNT

    # A same-account role exercises the SDK's refreshable AssumeRole provider.
    # No temporary credential returned by STS is ever copied to this report.
    role = call(
        "iam", "create_role", RoleName=INVENTORY_ROLE,
        AssumeRolePolicyDocument=json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow", "Principal": {"AWS": identity["Arn"]},
                "Action": "sts:AssumeRole",
            }],
        }),
    )
    if role is not None:
        fixtures["assume_role_arn"] = role["Role"]["Arn"]
        read_actions = [
            f"{service}:{prefix}*"
            for service in [
                "ec2", "dynamodb", "lambda", "ecs", "eks", "elasticloadbalancing",
                "s3", "iam", "route53", "cloudfront", "organizations", "config",
                "resource-explorer-2",
            ]
            for prefix in ["List", "Get", "Describe"]
        ]
        read_actions.extend([
            "sts:GetCallerIdentity", "config:SelectAggregateResourceConfig",
            "resource-explorer-2:Search", "cloudformation:ListResources",
            "cloudformation:GetResource",
        ])
        call(
            "iam", "put_role_policy", RoleName=INVENTORY_ROLE,
            PolicyName=f"{PREFIX}-read-policy",
            PolicyDocument=json.dumps({
                "Version": "2012-10-17", "Statement": [{
                    "Effect": "Allow", "Action": read_actions, "Resource": "*",
                }],
            }),
        )

    # The mounted Organizations service already owns a management organization.
    roots = call("organizations", "list_roots", MaxResults=20)
    if roots and roots.get("Roots"):
        root_id = roots["Roots"][0]["Id"]
        fixtures["organization_root_id"] = root_id
        fixtures["organizational_unit_ids"] = []
        for name in ("Architecture", "Platform"):
            created = call(
                "organizations", "create_organizational_unit",
                ParentId=root_id, Name=f"{PREFIX}-{name}",
            )
            if created and created.get("OrganizationalUnit"):
                fixtures["organizational_unit_ids"].append(created["OrganizationalUnit"]["Id"])

    # One eligible aggregator avoids ambiguous automatic source selection.
    aggregator = call(
        "config", "put_configuration_aggregator",
        ConfigurationAggregatorName=CONFIG_AGGREGATOR,
        AccountAggregationSources=[{"AccountIds": [ACCOUNT], "AwsRegions": [region]}],
    )
    if aggregator is not None:
        fixtures["config_aggregator"] = CONFIG_AGGREGATOR
    config_role = call(
        "iam", "create_role", RoleName=CONFIG_ROLE,
        AssumeRolePolicyDocument=json.dumps({
            "Version": "2012-10-17", "Statement": [{
                "Effect": "Allow", "Principal": {"Service": "config.amazonaws.com"},
                "Action": "sts:AssumeRole",
            }],
        }),
    )
    bucket = f"{PREFIX}-config-delivery"
    bucket_arguments: dict[str, Any] = {"Bucket": bucket}
    if region != "us-east-1":
        bucket_arguments["CreateBucketConfiguration"] = {"LocationConstraint": region}
    bucket_created = call("s3", "create_bucket", **bucket_arguments)
    if bucket_created is not None:
        bucket_arn = f"arn:aws:s3:::{bucket}"
        bucket_policy = {
            "Version": "2012-10-17", "Statement": [
                {
                    "Effect": "Allow", "Principal": {"Service": "config.amazonaws.com"},
                    "Action": action, "Resource": resource,
                    "Condition": {"StringEquals": {"AWS:SourceAccount": ACCOUNT}},
                }
                for action, resource in [
                    ("s3:GetBucketAcl", bucket_arn), ("s3:ListBucket", bucket_arn),
                    ("s3:PutObject", f"{bucket_arn}/*"),
                ]
            ],
        }
        call("s3", "put_bucket_policy", Bucket=bucket, Policy=json.dumps(bucket_policy))
    if config_role is not None:
        recorder_name = f"{PREFIX}-recorder"
        call(
            "config", "put_configuration_recorder",
            ConfigurationRecorder={
                "name": recorder_name, "roleARN": config_role["Role"]["Arn"],
                "recordingGroup": {"allSupported": True},
            },
        )
        call(
            "config", "put_delivery_channel",
            DeliveryChannel={"name": f"{PREFIX}-delivery", "s3BucketName": bucket},
        )
        started = call(
            "config", "start_configuration_recorder", ConfigurationRecorderName=recorder_name,
        )
        if started is not None:
            fixtures["config_recorded_resources"] = []
            for suffix in ("one", "two"):
                table_name = f"{PREFIX}-config-table-{suffix}"
                written = call(
                    "config", "put_resource_config", ResourceType="AWS::DynamoDB::Table",
                    SchemaVersionId="00000000-0000-0000-0000-000000000001",
                    ResourceId=table_name, ResourceName=table_name,
                    Configuration=json.dumps({
                        "tableName": table_name,
                        "tableArn": f"arn:aws:dynamodb:{region}:{ACCOUNT}:table/{table_name}",
                        "tableStatus": "ACTIVE", "billingModeSummary": {"billingMode": "PAY_PER_REQUEST"},
                    }),
                    Tags={"Fixture": PREFIX},
                )
                if written is not None:
                    fixtures["config_recorded_resources"].append(table_name)

    # Both actual owner bridges are seeded, then an explicit default view is set.
    call("sqs", "create_queue", QueueName=f"{PREFIX}-explorer-queue", tags={"Fixture": PREFIX})
    call("sns", "create_topic", Name=f"{PREFIX}-explorer-topic", Tags=[{"Key": "Fixture", "Value": PREFIX}])
    call("resource-explorer-2", "create_index", ClientToken=f"{PREFIX}-index", Tags={"Fixture": PREFIX})
    index = None
    for attempt in range(MAX_POLLS):
        index = call("resource-explorer-2", "get_index")
        if index is None or index.get("State") == "ACTIVE":
            break
        if attempt + 1 < MAX_POLLS:
            time.sleep(POLL_SECONDS)
    if index and index.get("State") == "ACTIVE":
        fixtures["resource_explorer_index_arn"] = index["Arn"]
        view = call(
            "resource-explorer-2", "create_view", ViewName=f"{PREFIX}-view",
            ClientToken=f"{PREFIX}-view", IncludedProperties=[{"Name": "tags"}],
            Tags={"Fixture": PREFIX},
        )
        if view and view.get("View"):
            view_arn = view["View"]["ViewArn"]
            associated = call("resource-explorer-2", "associate_default_view", ViewArn=view_arn)
            if associated is not None:
                fixtures["resource_explorer_view_arn"] = view_arn
    else:
        report["calls"].append({
            "operation": "resource-explorer-2:AwaitActiveIndex", "status": "incomplete",
            "error_code": "IndexNotActiveWithinPollBound",
        })

    # SQS is a native Cloud Control bridge, so both native and generic inventories
    # can observe it.  Two queues ensure MaxResults=1 produces a real cursor.
    fixtures["cloud_control"] = []
    for suffix in ("one", "two"):
        created = call(
            "cloudcontrol", "create_resource", TypeName="AWS::SQS::Queue",
            ClientToken=f"{PREFIX}-cloudcontrol-{suffix}",
            DesiredState=json.dumps({"QueueName": f"{PREFIX}-cloudcontrol-{suffix}"}),
        )
        progress = (created or {}).get("ProgressEvent", {})
        for attempt in range(MAX_POLLS):
            if progress.get("OperationStatus") in ("SUCCESS", "FAILED", "CANCEL_COMPLETE"):
                break
            token = progress.get("RequestToken")
            if not token:
                break
            if attempt:
                time.sleep(POLL_SECONDS)
            polled = call("cloudcontrol", "get_resource_request_status", RequestToken=token)
            if polled is None:
                break
            progress = polled.get("ProgressEvent", {})
        if progress.get("OperationStatus") == "SUCCESS" and progress.get("Identifier"):
            fixtures["cloud_control"].append({
                "type_name": "AWS::SQS::Queue", "identifier": progress["Identifier"],
            })
        else:
            report["calls"].append({
                "operation": "cloudcontrol:AwaitCreateResource", "status": "incomplete",
                "error_code": progress.get("ErrorCode", "CreateDidNotSucceedWithinPollBound"),
                "operation_status": progress.get("OperationStatus"),
            })

    failures = sum(item["status"] != "ok" for item in report["calls"])
    report["status"] = "ready" if failures == 0 else "partial"
    report["failed_calls"] = failures
    return report


if __name__ == "__main__":
    # The endpoint check also runs before Session construction in standalone use.
    _endpoint(os.environ.get("AWS_ENDPOINT_URL"))
    os.environ["AWS_EC2_METADATA_DISABLED"] = "true"
    import boto3

    print(json.dumps(seed(boto3.Session()), indent=2))
