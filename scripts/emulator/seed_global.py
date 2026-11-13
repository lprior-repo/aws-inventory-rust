"""Seed nonempty global inventory through signed, loopback-only AWS APIs.

Public interface: seed(session, record=None) -> JSON-serializable report.
The optional record("service:Operation", callable) invokes the callable once and
returns its result or None. Both the harness and this module retain operation
outcomes, never credentials, signed headers, or complete SDK response bodies.

Reviewed against aws/aws-emulator f515581569b98a9dcbd2e686c12f912fdce51bba:
S3 bucket subresource handlers, Route53 zone/record handlers, IAM creation,
and core/router/tests/all/cloudfront_oac_s3_origin_edge.rs. No data-plane
request is made to any origin, notification consumer, or public DNS name.
"""

from __future__ import annotations

import json
import os
import time
from typing import Any, Callable
from uuid import uuid4

from transport import require_loopback


ACCOUNT = "000000000000"
PREFIX = "inventory-emulator-global"
# The upstream emulator includes this AWS-managed policy; its own cross-service
# CloudFront tests use the same identifier. This is public configuration, not a
# fabricated account-owned resource ARN.
CACHING_DISABLED = "4135ea2d-6df8-44a3-9df3-4b5a84be39ad"


def seed(session: Any, record: Callable[..., Any] | None = None) -> dict[str, Any]:
    """Create bounded local fixtures; preserve partial setup failures explicitly."""
    from botocore.config import Config

    endpoint = os.environ.get("AWS_ENDPOINT_URL", "")
    require_loopback(endpoint)
    region = session.region_name or "us-east-1"
    if region != "us-east-1":
        raise ValueError("Global emulator fixtures require the harness region us-east-1")
    suffix = uuid4().hex[:10]
    prefix = f"{PREFIX}-{suffix}"
    report: dict[str, Any] = {
        "endpoint": endpoint,
        "region": region,
        "fixture_prefix": prefix,
        "fixtures": {"s3": {}, "iam": {}, "route53": {}, "cloudfront": {}},
        "calls": [],
        "skipped": [],
        "branch_probes": [{
            "operation": "s3:HeadBucket",
            "fixture_path": "fixtures.s3.rich_bucket",
            "reason": "Collector fallback runs only when ListBuckets omits BucketRegion.",
            "coverage_rule": "Label a direct SDK probe separately; never rewrite ListBuckets responses.",
        }],
        "limits": [
            "Fixture writes and Python calls are not Rust collector coverage.",
            "A configured policy is private; no public access is granted to the rich bucket.",
            "TXT and origin custom-header values are synthetic omission sentinels, not secrets.",
            "No Lambda notification target is created by this global fixture; SNS/SQS targets and EventBridge metadata are seeded.",
            "A missing/unsupported setup API is reported per call; dependent fixture steps are skipped explicitly.",
        ],
    }
    fixtures = report["fixtures"]
    config = Config(
        connect_timeout=3,
        read_timeout=10,
        retries={"mode": "standard", "total_max_attempts": 1},
        s3={"addressing_style": "path"},
    )
    clients: dict[str, Any] = {}

    def client(service: str) -> Any:
        if service not in clients:
            clients[service] = session.client(
                service, endpoint_url=endpoint, region_name=region, config=config,
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
                raise ValueError("Recorder must invoke each fixture API exactly once")
            invoked = True
            start = time.monotonic()
            try:
                response = getattr(sdk, method)(**arguments)
            except Exception as error:
                detail = getattr(error, "response", {})
                report["calls"].append({
                    "operation": name, "status": "error",
                    "error_code": detail.get("Error", {}).get("Code", type(error).__name__),
                    "http_status": detail.get("ResponseMetadata", {}).get("HTTPStatusCode"),
                    "duration_ms": round((time.monotonic() - start) * 1000, 2),
                })
                raise
            captured["response"] = response
            report["calls"].append({
                "operation": name, "status": "ok",
                "http_status": response.get("ResponseMetadata", {}).get("HTTPStatusCode"),
                "duration_ms": round((time.monotonic() - start) * 1000, 2),
            })
            return response

        try:
            if record is None:
                invoke()
            else:
                record(name, invoke)
        except Exception:
            # Independent fixture groups continue after a recorded emulator gap.
            pass
        if not invoked:
            report["calls"].append({
                "operation": name, "status": "error", "error_code": "RecorderDidNotInvokeCall",
            })
        return captured.get("response")

    def skip(step: str, parent: str) -> None:
        report["skipped"].append({"step": step, "missing_parent": parent})

    def tags(name: str) -> list[dict[str, str]]:
        return [{"Key": "Name", "Value": name}, {"Key": "Fixture", "Value": "aws-inventory-emulator"}]

    identity = call("sts", "get_caller_identity")
    if identity is None or identity.get("Account") != ACCOUNT:
        report["status"] = "refused_identity"
        report["reason"] = "Loopback endpoint did not report emulator account 000000000000"
        return report
    fixtures["account_id"] = ACCOUNT

    # Customer-managed metadata, an unattached policy, two role-policy pages, and
    # role/profile edges. No access keys, policy versions, or inline policies.
    policy_arns = []
    for index in range(3):
        result = call(
            "iam", "create_policy", PolicyName=f"{prefix}-policy-{index}",
            Path="/inventory-emulator/", Description="Local inventory fixture",
            PolicyDocument=json.dumps({
                "Version": "2012-10-17", "Statement": [{
                    "Effect": "Allow", "Action": ["s3:ListAllMyBuckets"], "Resource": "*",
                }],
            }), Tags=tags(f"global-policy-{index}"),
        )
        arn = (result or {}).get("Policy", {}).get("Arn")
        if arn:
            policy_arns.append(arn)
    fixtures["iam"]["policy_arns"] = policy_arns
    role_arns = []
    profile_arns = []
    replication_role = None
    for index in range(2):
        role_name = f"{prefix}-role-{index}"
        services = ["ec2.amazonaws.com", "s3.amazonaws.com"] if index == 0 else ["lambda.amazonaws.com"]
        arguments = {
            "RoleName": role_name, "Path": "/inventory-emulator/", "MaxSessionDuration": 3600,
            "Description": "Local inventory architecture fixture", "Tags": tags(f"global-role-{index}"),
            "AssumeRolePolicyDocument": json.dumps({
                "Version": "2012-10-17", "Statement": [{
                    "Effect": "Allow", "Principal": {"Service": services},
                    "Action": "sts:AssumeRole", "Condition": {"StringEquals": {"aws:SourceAccount": ACCOUNT}},
                }],
            }),
        }
        if policy_arns:
            arguments["PermissionsBoundary"] = policy_arns[0]
        result = call("iam", "create_role", **arguments)
        arn = (result or {}).get("Role", {}).get("Arn")
        if arn:
            role_arns.append(arn)
            if index == 0:
                replication_role = arn
            for policy_arn in policy_arns[:2]:
                call("iam", "attach_role_policy", RoleName=role_name, PolicyArn=policy_arn)
        profile_name = f"{prefix}-profile-{index}"
        profile = call(
            "iam", "create_instance_profile", InstanceProfileName=profile_name,
            Path="/inventory-emulator/", Tags=tags(f"global-profile-{index}"),
        )
        profile_arn = (profile or {}).get("InstanceProfile", {}).get("Arn")
        if profile_arn:
            profile_arns.append(profile_arn)
            if arn:
                call("iam", "add_role_to_instance_profile", InstanceProfileName=profile_name, RoleName=role_name)
            else:
                skip(f"add_role_to_instance_profile:{profile_name}", role_name)
    fixtures["iam"].update(role_arns=role_arns, profile_arns=profile_arns, replication_role_arn=replication_role)

    # Four buckets distinguish absent state from rich configuration and provide
    # real same-region logging/replication destinations without storing objects.
    buckets: dict[str, str] = {}
    for kind in ("rich", "empty", "logs", "replica"):
        name = f"{prefix}-{kind}"
        if call("s3", "create_bucket", Bucket=name) is not None:
            buckets[kind] = name
            fixtures["s3"][f"{kind}_bucket"] = name
    rich = buckets.get("rich")
    if rich:
        owner = {"Bucket": rich, "ExpectedBucketOwner": ACCOUNT}
        call("s3", "put_bucket_versioning", **owner, VersioningConfiguration={"Status": "Enabled"})
        call("s3", "put_bucket_tagging", **owner, Tagging={"TagSet": tags("global-rich-bucket")})
        call("s3", "put_public_access_block", **owner, PublicAccessBlockConfiguration={
            "BlockPublicAcls": True, "IgnorePublicAcls": True,
            "BlockPublicPolicy": True, "RestrictPublicBuckets": True,
        })
        call("s3", "put_bucket_policy", **owner, Policy=json.dumps({
            "Version": "2012-10-17", "Statement": [{
                "Sid": "PrivateAccountMetadata", "Effect": "Allow",
                "Principal": {"AWS": f"arn:aws:iam::{ACCOUNT}:root"},
                "Action": "s3:GetBucketLocation", "Resource": f"arn:aws:s3:::{rich}",
            }],
        }))
        call("s3", "put_bucket_ownership_controls", **owner, OwnershipControls={
            "Rules": [{"ObjectOwnership": "BucketOwnerEnforced"}],
        })
        call("s3", "put_bucket_website", **owner, WebsiteConfiguration={
            "RedirectAllRequestsTo": {"HostName": "inventory-fixture.invalid", "Protocol": "https"},
        })
        key = call("kms", "create_key", Description="Local inventory encryption fixture", Tags=[
            {"TagKey": "Fixture", "TagValue": "aws-inventory-emulator"},
        ])
        key_arn = (key or {}).get("KeyMetadata", {}).get("Arn")
        if key_arn:
            fixtures["s3"]["kms_key_arn"] = key_arn
        encryption = {"SSEAlgorithm": "aws:kms", "KMSMasterKeyID": key_arn} if key_arn else {"SSEAlgorithm": "AES256"}
        call("s3", "put_bucket_encryption", **owner, ServerSideEncryptionConfiguration={
            "Rules": [{"ApplyServerSideEncryptionByDefault": encryption, "BucketKeyEnabled": bool(key_arn)}],
        })
        if buckets.get("logs"):
            call("s3", "put_bucket_logging", **owner, BucketLoggingStatus={
                "LoggingEnabled": {"TargetBucket": buckets["logs"], "TargetPrefix": "inventory-fixture/"},
            })
        else:
            skip("s3:PutBucketLogging", "logs bucket")
        replica = buckets.get("replica")
        if replica and replication_role:
            call("s3", "put_bucket_versioning", Bucket=replica, ExpectedBucketOwner=ACCOUNT,
                 VersioningConfiguration={"Status": "Enabled"})
            destination: dict[str, Any] = {"Bucket": f"arn:aws:s3:::{replica}", "StorageClass": "STANDARD"}
            rule: dict[str, Any] = {
                "ID": "inventory-replication", "Priority": 1, "Status": "Enabled", "Filter": {"Prefix": ""},
                "DeleteMarkerReplication": {"Status": "Disabled"}, "Destination": destination,
            }
            if key_arn:
                destination["EncryptionConfiguration"] = {"ReplicaKmsKeyID": key_arn}
                rule["SourceSelectionCriteria"] = {"SseKmsEncryptedObjects": {"Status": "Enabled"}}
            call("s3", "put_bucket_replication", **owner, ReplicationConfiguration={
                "Role": replication_role, "Rules": [rule],
            })
        else:
            skip("s3:PutBucketReplication", "replica bucket and replication role")

        # Destinations are actual resources returned by local SNS/SQS, with
        # resource policies permitting S3's validation/test notification.
        source_arn = f"arn:aws:s3:::{rich}"
        topic = call("sns", "create_topic", Name=f"{prefix}-notifications", Tags=tags("global-notifications"))
        topic_arn = (topic or {}).get("TopicArn")
        queue = call("sqs", "create_queue", QueueName=f"{prefix}-notifications", tags={"Fixture": "aws-inventory-emulator"})
        queue_url = (queue or {}).get("QueueUrl")
        queue_arn = None
        if queue_url:
            attributes = call("sqs", "get_queue_attributes", QueueUrl=queue_url, AttributeNames=["QueueArn"])
            queue_arn = (attributes or {}).get("Attributes", {}).get("QueueArn")
        notifications: dict[str, Any] = {"EventBridgeConfiguration": {}}
        for service, arn, action, label in (("sns", topic_arn, "sns:Publish", "topic"), ("sqs", queue_arn, "sqs:SendMessage", "queue")):
            if not arn:
                skip(f"s3 notification {label}", f"{service} destination ARN")
                continue
            policy = json.dumps({"Version": "2012-10-17", "Statement": [{
                "Sid": "AllowLocalBucketNotification", "Effect": "Allow", "Principal": {"Service": "s3.amazonaws.com"},
                "Action": action, "Resource": arn,
                "Condition": {"ArnEquals": {"aws:SourceArn": source_arn}, "StringEquals": {"aws:SourceAccount": ACCOUNT}},
            }]})
            if service == "sns":
                policy_result = call("sns", "set_topic_attributes", TopicArn=arn, AttributeName="Policy", AttributeValue=policy)
                if policy_result is not None:
                    notifications["TopicConfigurations"] = [{"Id": "inventory-topic", "TopicArn": arn,
                        "Events": ["s3:ObjectCreated:*"], "Filter": {"Key": {"FilterRules": [{"Name": "prefix", "Value": "topic/"}]}}}]
                    fixtures["s3"]["notification_topic_arn"] = arn
            else:
                policy_result = call("sqs", "set_queue_attributes", QueueUrl=queue_url, Attributes={"Policy": policy})
                if policy_result is not None:
                    notifications["QueueConfigurations"] = [{"Id": "inventory-queue", "QueueArn": arn,
                        "Events": ["s3:ObjectRemoved:*"], "Filter": {"Key": {"FilterRules": [{"Name": "prefix", "Value": "queue/"}]}}}]
                    fixtures["s3"]["notification_queue_arn"] = arn
        call("s3", "put_bucket_notification_configuration", **owner, NotificationConfiguration=notifications)
    else:
        skip("rich S3 configuration", "rich bucket")

    # Route53 uses real locally created VPC identity for its private-zone edge.
    # Public records exercise composite routing identifiers and omission rules.
    vpc = call("ec2", "create_vpc", CidrBlock="10.121.0.0/16", TagSpecifications=[
        {"ResourceType": "vpc", "Tags": tags("global-private-zone-vpc")},
    ])
    vpc_id = (vpc or {}).get("Vpc", {}).get("VpcId")
    fixtures["route53"]["vpc_id"] = vpc_id
    zones = []
    for kind in ("public", "private"):
        name = f"{kind}.{prefix}.example."
        args: dict[str, Any] = {
            "Name": name, "CallerReference": f"{prefix}-{kind}",
            "HostedZoneConfig": {"Comment": "Local inventory fixture", "PrivateZone": kind == "private"},
        }
        if kind == "private":
            if not vpc_id:
                skip("route53 private zone", "VPC")
                continue
            args["VPC"] = {"VPCRegion": region, "VPCId": vpc_id}
        created = call("route53", "create_hosted_zone", **args)
        zone_id = (created or {}).get("HostedZone", {}).get("Id")
        if not zone_id:
            continue
        bare_id = zone_id.removeprefix("/hostedzone/")
        zones.append({"kind": kind, "id": bare_id, "name": name})
        call("route53", "change_tags_for_resource", ResourceType="hostedzone", ResourceId=bare_id,
             AddTags=tags(f"global-{kind}-zone"))
        records = [
            {"Name": f"www.{name}", "Type": "A", "TTL": 60, "ResourceRecords": [{"Value": "192.0.2.10"}]},
            {"Name": f"v6.{name}", "Type": "AAAA", "TTL": 60, "ResourceRecords": [{"Value": "2001:db8::10"}]},
            {"Name": f"alias.{name}", "Type": "A", "AliasTarget": {
                "DNSName": f"www.{name}", "HostedZoneId": bare_id, "EvaluateTargetHealth": False,
            }},
            {"Name": f"weighted.{name}", "Type": "A", "SetIdentifier": "blue", "Weight": 50,
             "TTL": 60, "ResourceRecords": [{"Value": "192.0.2.11"}]},
            {"Name": f"weighted.{name}", "Type": "A", "SetIdentifier": "green", "Weight": 50,
             "TTL": 60, "ResourceRecords": [{"Value": "192.0.2.12"}]},
            {"Name": f"mail.{name}", "Type": "MX", "TTL": 60, "ResourceRecords": [{"Value": "0 ."}]},
            {"Name": f"text.{name}", "Type": "TXT", "TTL": 60,
             "ResourceRecords": [{"Value": '"inventory-fixture-txt-must-not-be-collected"'}]},
        ]
        call("route53", "change_resource_record_sets", HostedZoneId=bare_id, ChangeBatch={
            "Comment": "Architecture and composite-cursor fixtures",
            "Changes": [{"Action": "UPSERT", "ResourceRecordSet": row} for row in records],
        })
    fixtures["route53"]["zones"] = zones

    # These origin hostnames are architecture metadata only. We create two
    # disabled distributions and never invoke the emulator edge/data plane.
    distributions = []
    if rich:
        for index in range(2):
            s3_origin = {"Id": "bucket-origin", "DomainName": f"{rich}.s3.{region}.amazonaws.com",
                         "S3OriginConfig": {"OriginAccessIdentity": ""}, "OriginPath": "/omit-this-origin-path"}
            custom_origin = {"Id": "api-origin", "DomainName": "origin.inventory-fixture.invalid",
                             "CustomOriginConfig": {"HTTPPort": 80, "HTTPSPort": 443,
                                "OriginProtocolPolicy": "https-only", "OriginSslProtocols": {"Quantity": 1, "Items": ["TLSv1.2"]}},
                             "CustomHeaders": {"Quantity": 1, "Items": [{
                                 "HeaderName": "X-Inventory-Fixture", "HeaderValue": "fixture-header-must-not-be-collected",
                             }]}}
            behavior = {"TargetOriginId": "bucket-origin", "ViewerProtocolPolicy": "redirect-to-https",
                        "CachePolicyId": CACHING_DISABLED, "Compress": True,
                        "TrustedSigners": {"Enabled": False, "Quantity": 0},
                        "TrustedKeyGroups": {"Enabled": False, "Quantity": 0},
                        "AllowedMethods": {"Quantity": 2, "Items": ["GET", "HEAD"],
                                           "CachedMethods": {"Quantity": 2, "Items": ["GET", "HEAD"]}}}
            ordered = dict(behavior, PathPattern="/api/*", TargetOriginId="api-origin")
            configuration = {"CallerReference": f"{prefix}-distribution-{index}", "Enabled": False,
                             "Comment": "Local inventory architecture fixture", "Aliases": {"Quantity": 0},
                             "Origins": {"Quantity": 2, "Items": [s3_origin, custom_origin]},
                             "DefaultCacheBehavior": behavior, "CacheBehaviors": {"Quantity": 1, "Items": [ordered]},
                             "ViewerCertificate": {"CloudFrontDefaultCertificate": True},
                             "Restrictions": {"GeoRestriction": {"RestrictionType": "none", "Quantity": 0}},
                             "HttpVersion": "http2", "IsIPV6Enabled": True, "PriceClass": "PriceClass_100"}
            created = call("cloudfront", "create_distribution", DistributionConfig=configuration)
            distribution = (created or {}).get("Distribution", {})
            if distribution.get("Id") and distribution.get("ARN"):
                distributions.append({"id": distribution["Id"], "arn": distribution["ARN"],
                                      "domain_name": distribution.get("DomainName")})
                call("cloudfront", "tag_resource", Resource=distribution["ARN"], Tags={"Items": tags(f"global-distribution-{index}")})
    else:
        skip("cloudfront distributions", "rich S3 origin bucket")
    fixtures["cloudfront"]["distributions"] = distributions
    report["status"] = "complete" if not report["skipped"] and all(row["status"] == "ok" for row in report["calls"]) else "partial"
    report["call_count"] = len(report["calls"])
    report["error_count"] = sum(row["status"] != "ok" for row in report["calls"])
    return report
