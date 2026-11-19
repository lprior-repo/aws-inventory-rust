"""Create bounded regional fixtures through AWS Emulator's signed public APIs.

The caller supplies a boto3 Session and optionally ``record(name, callable)``.
Only HTTPS on a literal loopback address is accepted. Credentials, complete SDK
responses and function code are never retained in the report. This module does
not alter emulator stores or replace unsupported operations with invented data.

The fixtures populate inputs for the 39 native resource kinds, including SQS
queues, SNS topics and subscriptions, KMS keys and aliases, and Step Functions
state machines and activities. Most parent/child
lists receive two entries; one RunInstances request creates six instances so the
collector's minimum EC2 page size of five can page. Fixture API successes do not
certify subsequent collector or data-plane success.
"""

from __future__ import annotations

import io
import json
import os
import time
import uuid
import zipfile
from typing import Any, Callable

from transport import require_loopback


MAX_CALLS = 512
MAX_POLLS = 12
POLL_SECONDS = 0.1
ACCOUNT = "000000000000"


def _at(value: Any, *path: str | int) -> Any:
    for part in path:
        if isinstance(part, int) and isinstance(value, list) and 0 <= part < len(value):
            value = value[part]
        elif isinstance(part, str) and isinstance(value, dict):
            value = value.get(part)
        else:
            return None
    return value


class _Seed:
    def __init__(self, session: Any, record: Callable[..., Any] | None):
        self.endpoint = os.environ.get("AWS_ENDPOINT_URL", "").rstrip("/")
        require_loopback(self.endpoint)  # Before constructing any credential provider.
        from botocore.config import Config

        self.session = session
        self.record = record
        self.region = session.region_name or "us-east-1"
        self.prefix = "inventory-" + uuid.uuid4().hex[:8]
        self.config = Config(
            connect_timeout=3,
            read_timeout=10,
            retries={"mode": "standard", "total_max_attempts": 1},
        )
        self.clients: dict[str, Any] = {}
        self.fixtures: dict[str, Any] = {}
        self.calls: list[dict[str, Any]] = []
        self.skipped: list[dict[str, Any]] = []
        self.count = 0

    def client(self, service: str) -> Any:
        if service not in self.clients:
            self.clients[service] = self.session.client(
                service,
                region_name=self.region,
                endpoint_url=self.endpoint,
                verify=os.environ.get("AWS_CA_BUNDLE") or True,
                config=self.config,
            )
        return self.clients[service]

    def call(self, service: str, method: str, label: str = "", **arguments: Any) -> Any:
        if self.count >= MAX_CALLS:
            raise RuntimeError("regional fixture API-call budget exceeded")
        self.count += 1
        sdk = self.client(service)
        operation = f"{service}:{sdk.meta.method_to_api_mapping[method]}"
        entry: dict[str, Any] = {"operation": operation, "fixture": label}
        invoked = False
        captured = None

        def invoke() -> Any:
            nonlocal invoked, captured
            if invoked:
                raise RuntimeError("fixture recorder invoked the same operation twice")
            invoked = True
            try:
                captured = getattr(sdk, method)(**arguments)
            except Exception as error:
                response = getattr(error, "response", {})
                entry.update(
                    status="error",
                    error_code=_at(response, "Error", "Code") or type(error).__name__,
                    http_status=_at(response, "ResponseMetadata", "HTTPStatusCode"),
                )
                raise
            entry.update(
                status="ok",
                http_status=_at(captured, "ResponseMetadata", "HTTPStatusCode"),
            )
            failures = _at(captured, "failures")
            if isinstance(failures, list) and failures:
                entry["status"] = "returned_failures"
                entry["failure_reasons"] = [str(item.get("reason", "unknown"))[:256] for item in failures[:16] if isinstance(item, dict)]
            return captured

        try:
            if self.record is None:
                invoke()
            else:
                self.record(operation, invoke)
        except Exception as error:
            entry.setdefault("status", "error")
            entry.setdefault("error_code", type(error).__name__)
        if not invoked:
            raise RuntimeError("fixture recorder did not invoke the operation")
        self.calls.append(entry)
        return captured

    def skip(self, operation: str, reason: str) -> None:
        self.skipped.append({"operation": operation, "reason": reason})

    def wait(self, label: str, read: Callable[[], Any], ready: Callable[[Any], bool]) -> bool:
        for attempt in range(MAX_POLLS):
            response = read()
            if response is None:
                return False
            if ready(response):
                return True
            if attempt + 1 < MAX_POLLS:
                time.sleep(POLL_SECONDS)
        self.skip(label, "readiness was not observed within the bounded polling budget")
        return False

    def tags(self, resource: str, name: str) -> list[dict[str, Any]]:
        return [{"ResourceType": resource, "Tags": [{"Key": "Name", "Value": name}, {"Key": "purpose", "Value": "inventory-emulator-fixture"}]}]

    def report(self) -> dict[str, Any]:
        expected = {
            ("ec2", "vpc_ids"): 2, ("ec2", "subnet_ids"): 2,
            ("ec2", "security_group_ids"): 2, ("ec2", "network_interface_ids"): 2,
            ("ec2", "volume_ids"): 2, ("ec2", "instance_ids"): 6,
            ("ec2", "route_table_ids"): 2, ("ec2", "internet_gateway_ids"): 2,
            ("ec2", "nat_gateway_ids"): 2, ("ec2", "vpc_endpoint_ids"): 2,
            ("ec2", "network_acl_ids"): 2, ("ec2", "vpc_peering_connection_ids"): 1,
            ("lambda", "function_arns"): 2, ("dynamodb", "table_names"): 2,
            ("ecs", "clusters"): 2, ("ecs", "task_definition_arns"): 4,
            ("eks", "clusters"): 2, ("elbv2", "load_balancers"): 2,
            ("elbv2", "target_group_arns"): 2,
            ("sqs", "queue_arns"): 1,
            ("sns", "topic_arns"): 1, ("sns", "subscription_arns"): 1,
            ("kms", "key_arns"): 2, ("kms", "alias_arns"): 1,
            ("step_functions", "state_machine_arns"): 2,
            ("step_functions", "activity_arns"): 2,
        }
        missing = []
        for path, minimum in expected.items():
            observed = len(_at(self.fixtures, *path) or [])
            if observed < minimum:
                missing.append({"fixture": ".".join(path), "observed": observed, "expected_minimum": minimum})
        for service, collection, children in (
            ("ecs", "clusters", {"service_arns": 2, "task_arns": 2}),
            ("eks", "clusters", {"nodegroups": 2}),
            ("elbv2", "load_balancers", {"listeners": 2, "rule_arns": 4}),
        ):
            for index, parent in enumerate(_at(self.fixtures, service, collection) or []):
                for child, minimum in children.items():
                    observed = len(parent.get(child, []))
                    if observed < minimum:
                        missing.append({"fixture": f"{service}.{collection}.{index}.{child}", "observed": observed, "expected_minimum": minimum})
        partial = bool(self.skipped or missing) or any(row["status"] != "ok" for row in self.calls)
        return {
            "status": "partial" if partial else "complete",
            "endpoint": self.endpoint,
            "region": self.region,
            "prefix": self.prefix,
            "fixtures": self.fixtures,
            "calls": self.calls,
            "skipped": self.skipped,
            "missing_fixtures": missing,
            "limits": {"api_calls": MAX_CALLS, "polls_per_resource": MAX_POLLS},
            "semantics": "Control-plane fixtures created through actual emulator APIs; no direct store injection, data-plane assertion, or live AWS request.",
        }


def seed(session: Any, record: Callable[..., Any] | None = None) -> dict[str, Any]:
    context = _Seed(session, record)
    identity = context.call("sts", "get_caller_identity", "identity")
    if _at(identity, "Account") != ACCOUNT:
        context.skip("regional fixture writes", "emulator account identity was not verified")
        return context.report()
    network = _network(context)
    roles = _roles(context)
    functions = _functions(context, roles, network)
    _tables(context)
    _kubernetes(context, roles, network)
    _load_balancers(context, network)
    _containers(context, roles, network)
    _messaging(context, functions)
    _keys(context)
    _workflows(context, roles)
    return context.report()


def _network(ctx: _Seed) -> dict[str, Any]:
    result: dict[str, Any] = {
        "vpc_ids": [], "vpc_cidr_blocks": [], "subnet_ids": [], "security_group_ids": [],
        "network_interface_ids": [], "volume_ids": [], "instance_ids": [],
        "route_table_ids": [], "internet_gateway_ids": [], "nat_gateway_ids": [],
        "vpc_endpoint_ids": [], "network_acl_ids": [], "vpc_peering_connection_ids": [],
        "nat_gateway_bindings": [],
    }
    ctx.fixtures["ec2"] = result
    for index in range(2):
        response = ctx.call("ec2", "create_vpc", f"vpc-{index}", CidrBlock=f"10.{83 + index}.0.0/16", TagSpecifications=ctx.tags("vpc", f"{ctx.prefix}-vpc-{index}"))
        vpc = _at(response, "Vpc", "VpcId")
        if vpc:
            result["vpc_ids"].append(vpc)
            result["vpc_cidr_blocks"].append(f"10.{83 + index}.0.0/16")
    if not result["vpc_ids"]:
        ctx.skip("EC2 network children", "no VPC was created")
        return result
    vpc = result["vpc_ids"][0]
    subnet_prefix = ".".join(result["vpc_cidr_blocks"][0].split(".")[:2])
    primary_gateway = None
    for index, attached_vpc in enumerate(result["vpc_ids"]):
        name = f"{ctx.prefix}-gateway-{index}"
        gateway = ctx.call("ec2", "create_internet_gateway", name, TagSpecifications=ctx.tags("internet-gateway", name))
        gateway_id = _at(gateway, "InternetGateway", "InternetGatewayId")
        if gateway_id:
            result["internet_gateway_ids"].append(gateway_id)
            attached = ctx.call("ec2", "attach_internet_gateway", name, InternetGatewayId=gateway_id, VpcId=attached_vpc)
            if index == 0 and attached is not None:
                primary_gateway = gateway_id
    for index, zone in enumerate(("a", "b")):
        name = f"{ctx.prefix}-{zone}"
        subnet = ctx.call("ec2", "create_subnet", name, VpcId=vpc, CidrBlock=f"{subnet_prefix}.{index + 1}.0/24", AvailabilityZone=ctx.region + zone, TagSpecifications=ctx.tags("subnet", name))
        subnet_id = _at(subnet, "Subnet", "SubnetId")
        if subnet_id:
            result["subnet_ids"].append(subnet_id)
        group = ctx.call("ec2", "create_security_group", name, VpcId=vpc, GroupName=name, Description="Local inventory fixture", TagSpecifications=ctx.tags("security-group", name))
        group_id = _at(group, "GroupId")
        if group_id:
            result["security_group_ids"].append(group_id)
            ctx.call("ec2", "authorize_security_group_ingress", name, GroupId=group_id, IpPermissions=[{"IpProtocol": "tcp", "FromPort": 80, "ToPort": 80, "IpRanges": [{"CidrIp": "0.0.0.0/0", "Description": "Emulated HTTP fixture"}]}])
        if subnet_id:
            arguments: dict[str, Any] = {"SubnetId": subnet_id, "Description": name, "TagSpecifications": ctx.tags("network-interface", name)}
            if group_id:
                arguments["Groups"] = [group_id]
            interface = ctx.call("ec2", "create_network_interface", name, **arguments)
            if interface_id := _at(interface, "NetworkInterface", "NetworkInterfaceId"):
                result["network_interface_ids"].append(interface_id)
        volume = ctx.call("ec2", "create_volume", name, AvailabilityZone=ctx.region + zone, Size=1, VolumeType="gp3", TagSpecifications=ctx.tags("volume", name))
        if volume_id := _at(volume, "VolumeId"):
            result["volume_ids"].append(volume_id)
        route = ctx.call("ec2", "create_route_table", name, VpcId=vpc, TagSpecifications=ctx.tags("route-table", name))
        route_id = _at(route, "RouteTable", "RouteTableId")
        if route_id:
            result["route_table_ids"].append(route_id)
            if subnet_id:
                ctx.call("ec2", "associate_route_table", name, RouteTableId=route_id, SubnetId=subnet_id)
        if route_id and primary_gateway:
            ctx.call("ec2", "create_route", name, RouteTableId=route_id, DestinationCidrBlock="0.0.0.0/0", GatewayId=primary_gateway)
        acl = ctx.call("ec2", "create_network_acl", name, VpcId=vpc, TagSpecifications=ctx.tags("network-acl", name))
        if acl_id := _at(acl, "NetworkAcl", "NetworkAclId"):
            result["network_acl_ids"].append(acl_id)
            ctx.call("ec2", "create_network_acl_entry", name, NetworkAclId=acl_id, RuleNumber=100, Protocol="6", RuleAction="allow", Egress=False, CidrBlock="0.0.0.0/0", PortRange={"From": 80, "To": 80})
        if subnet_id:
            allocation = ctx.call("ec2", "allocate_address", name, Domain="vpc")
            if allocation_id := _at(allocation, "AllocationId"):
                nat = ctx.call("ec2", "create_nat_gateway", name, SubnetId=subnet_id, AllocationId=allocation_id, ConnectivityType="public", TagSpecifications=ctx.tags("natgateway", name))
                if nat_id := _at(nat, "NatGateway", "NatGatewayId"):
                    result["nat_gateway_ids"].append(nat_id)
                    result["nat_gateway_bindings"].append({
                        "id": nat_id, "subnet_id": subnet_id, "allocation_id": allocation_id,
                        "network_interface_ids": [
                            address["NetworkInterfaceId"]
                            for address in (_at(nat, "NatGateway", "NatGatewayAddresses") or [])
                            if address.get("NetworkInterfaceId")
                        ],
                    })
        endpoint = ctx.call("ec2", "create_vpc_endpoint", name, VpcId=vpc, VpcEndpointType="Gateway", ServiceName=f"com.amazonaws.{ctx.region}.{'s3' if index == 0 else 'dynamodb'}", RouteTableIds=[route_id] if route_id else [], TagSpecifications=ctx.tags("vpc-endpoint", name))
        if endpoint_id := _at(endpoint, "VpcEndpoint", "VpcEndpointId"):
            result["vpc_endpoint_ids"].append(endpoint_id)
    if len(result["vpc_ids"]) == 2:
        peering = ctx.call("ec2", "create_vpc_peering_connection", "peering", VpcId=result["vpc_ids"][0], PeerVpcId=result["vpc_ids"][1], TagSpecifications=ctx.tags("vpc-peering-connection", ctx.prefix))
        if peering_id := _at(peering, "VpcPeeringConnection", "VpcPeeringConnectionId"):
            result["vpc_peering_connection_ids"].append(peering_id)
            # The pinned emulator returns 501 here. Preserve that real failure;
            # the created pending-acceptance connection is still a valid fixture.
            ctx.call("ec2", "accept_vpc_peering_connection", "peering", VpcPeeringConnectionId=peering_id)
    _instances(ctx, result)
    return result


def _instances(ctx: _Seed, network: dict[str, Any]) -> None:
    images = ctx.call("ec2", "describe_images", "source-ami", Owners=["amazon"], MaxResults=5, Filters=[{"Name": "architecture", "Values": ["x86_64"]}, {"Name": "state", "Values": ["available"]}])
    image_id = _at(images, "Images", 0, "ImageId")
    if not image_id and network["volume_ids"]:
        snapshot = ctx.call("ec2", "create_snapshot", "source-ami", VolumeId=network["volume_ids"][0], Description="Local inventory fixture AMI")
        snapshot_id = _at(snapshot, "SnapshotId")
        if snapshot_id:
            ctx.wait("EC2 snapshot readiness", lambda: ctx.call("ec2", "describe_snapshots", "source-ami", SnapshotIds=[snapshot_id]), lambda response: _at(response, "Snapshots", 0, "State") == "completed")
            image = ctx.call("ec2", "register_image", "source-ami", Name=ctx.prefix + "-image", Architecture="x86_64", VirtualizationType="hvm", RootDeviceName="/dev/sda1", BlockDeviceMappings=[{"DeviceName": "/dev/sda1", "Ebs": {"SnapshotId": snapshot_id, "DeleteOnTermination": True}}])
            image_id = _at(image, "ImageId")
    if not image_id or not network["subnet_ids"]:
        ctx.skip("EC2 RunInstances", "no observed AMI or created subnet was available")
        return
    network["image_id"] = image_id
    arguments: dict[str, Any] = {
        "ImageId": image_id, "MinCount": 6, "MaxCount": 6, "InstanceType": "t3.micro",
        "SubnetId": network["subnet_ids"][0], "TagSpecifications": ctx.tags("instance", ctx.prefix),
        "MetadataOptions": {"HttpTokens": "required", "HttpEndpoint": "enabled"},
    }
    if network["security_group_ids"]:
        arguments["SecurityGroupIds"] = [network["security_group_ids"][0]]
    response = ctx.call("ec2", "run_instances", "six-instances", **arguments)
    for instance in (_at(response, "Instances") or [])[:6]:
        if instance.get("InstanceId"):
            network["instance_ids"].append(instance["InstanceId"])
    if network["instance_ids"] and network["volume_ids"]:
        ctx.wait("EC2 instance readiness", lambda: ctx.call("ec2", "describe_instances", "six-instances", InstanceIds=network["instance_ids"]), lambda answer: _at(answer, "Reservations", 0, "Instances", 0, "State", "Name") == "running")
        ctx.call("ec2", "attach_volume", "attached-volume", VolumeId=network["volume_ids"][0], InstanceId=network["instance_ids"][0], Device="/dev/sdf")


def _roles(ctx: _Seed) -> dict[str, str]:
    roles = {}
    definitions = {
        "lambda": (["lambda.amazonaws.com"], ["service-role/AWSLambdaVPCAccessExecutionRole"]),
        "eks_cluster": (["eks.amazonaws.com"], ["AmazonEKSClusterPolicy"]),
        "eks_node": (["ec2.amazonaws.com"], ["AmazonEKSWorkerNodePolicy", "AmazonEC2ContainerRegistryReadOnly"]),
        "ecs": (["ecs-tasks.amazonaws.com"], ["service-role/AmazonECSTaskExecutionRolePolicy"]),
        "states": (["states.amazonaws.com"], []),
    }
    for label, (services, policies) in definitions.items():
        name = ctx.prefix + "-" + label
        trust = {"Version": "2012-10-17", "Statement": [{"Effect": "Allow", "Principal": {"Service": services}, "Action": "sts:AssumeRole"}]}
        response = ctx.call("iam", "create_role", label, RoleName=name, AssumeRolePolicyDocument=json.dumps(trust), Description="Local regional inventory fixture role")
        if role := _at(response, "Role", "Arn"):
            roles[label] = role
            for policy in policies:
                ctx.call("iam", "attach_role_policy", label, RoleName=name, PolicyArn="arn:aws:iam::aws:policy/" + policy)
    ctx.fixtures["roles"] = roles
    return roles


def _functions(ctx: _Seed, roles: dict[str, str], network: dict[str, Any]) -> list[str]:
    functions = []
    ctx.fixtures["lambda"] = {"function_arns": functions}
    if "lambda" not in roles:
        ctx.skip("Lambda functions", "execution role was not created")
        return functions
    archive = io.BytesIO()
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as package:
        package.writestr("handler.py", "def handler(event, context):\n    return {'inventory_fixture': True}\n")
    for index in range(2):
        arguments: dict[str, Any] = {
            "FunctionName": f"{ctx.prefix}-function-{index}", "Runtime": "python3.12",
            "Role": roles["lambda"], "Handler": "handler.handler", "Code": {"ZipFile": archive.getvalue()},
            "Timeout": 3, "MemorySize": 128, "Tags": {"purpose": "inventory-emulator-fixture"},
        }
        if network["subnet_ids"] and network["security_group_ids"]:
            arguments["VpcConfig"] = {"SubnetIds": network["subnet_ids"], "SecurityGroupIds": network["security_group_ids"]}
        response = ctx.call("lambda", "create_function", f"function-{index}", **arguments)
        if arn := _at(response, "FunctionArn"):
            functions.append(arn)
    return functions


def _messaging(ctx: _Seed, function_arns: list[str]) -> None:
    queues: list[str] = []
    queue_names: list[str] = []
    queue_arns: list[str] = []
    topics: list[str] = []
    subscriptions: list[str] = []
    ctx.fixtures["sqs"] = {"queue_urls": queues, "queue_names": queue_names, "queue_arns": queue_arns}
    ctx.fixtures["sns"] = {"topic_arns": topics, "subscription_arns": subscriptions}
    queue_name = f"{ctx.prefix}-orders"
    created = ctx.call("sqs", "create_queue", "queue", QueueName=queue_name, tags={"purpose": "inventory-emulator-fixture"})
    if queue_url := _at(created, "QueueUrl"):
        queues.append(queue_url)
        queue_names.append(queue_name)
        queue_arns.append(f"arn:aws:sqs:{ctx.region}:{ACCOUNT}:{queue_name}")
    if not function_arns:
        ctx.skip("SNS topic and subscription", "a Lambda endpoint is required")
        return
    topic_name = f"{ctx.prefix}-orders"
    topic = ctx.call("sns", "create_topic", "topic", Name=topic_name, Tags=[{"Key": "purpose", "Value": "inventory-emulator-fixture"}, {"Key": "Name", "Value": topic_name}])
    if topic_arn := _at(topic, "TopicArn"):
        topics.append(topic_arn)
        subscribed = ctx.call("sns", "subscribe", "subscription", TopicArn=topic_arn, Protocol="lambda", Endpoint=function_arns[0], ReturnSubscriptionArn=True)
        if subscription_arn := _at(subscribed, "SubscriptionArn"):
            subscriptions.append(subscription_arn)


def _keys(ctx: _Seed) -> None:
    key_arns: list[str] = []
    key_ids: list[str] = []
    alias_arns: list[str] = []
    alias_names: list[str] = []
    alias_targets: list[str] = []
    ctx.fixtures["kms"] = {
        "key_arns": key_arns, "key_ids": key_ids,
        "alias_arns": alias_arns, "alias_names": alias_names, "alias_targets": alias_targets,
    }
    for index in range(2):
        response = ctx.call(
            "kms", "create_key", f"key-{index}",
            Description=f"{ctx.prefix} inventory key {index}",
            Tags=[{"TagKey": "purpose", "TagValue": "inventory-emulator-fixture"}],
        )
        key_id = _at(response, "KeyMetadata", "KeyId")
        key_arn = _at(response, "KeyMetadata", "Arn")
        if key_id and key_arn:
            key_ids.append(key_id)
            key_arns.append(key_arn)
    if not key_ids:
        ctx.skip("KMS alias", "a created key is required")
        return
    alias_name = f"alias/{ctx.prefix}-orders"
    ctx.call("kms", "create_alias", "alias", AliasName=alias_name, TargetKeyId=key_ids[0])
    listed = ctx.call("kms", "list_aliases", "alias-readback", KeyId=key_ids[0])
    for item in _at(listed, "Aliases") or []:
        if item.get("AliasName") != alias_name or not item.get("AliasArn"):
            continue
        alias_arns.append(item["AliasArn"])
        alias_names.append(alias_name)
        alias_targets.append(item.get("TargetKeyId") or key_ids[0])
        break


def _workflows(ctx: _Seed, roles: dict[str, str]) -> None:
    state_machine_arns: list[str] = []
    state_machine_names: list[str] = []
    activity_arns: list[str] = []
    activity_names: list[str] = []
    ctx.fixtures["step_functions"] = {
        "state_machine_arns": state_machine_arns,
        "state_machine_names": state_machine_names,
        "activity_arns": activity_arns,
        "activity_names": activity_names,
    }
    if "states" not in roles:
        ctx.skip("Step Functions fixtures", "state machine execution role was not created")
        return
    definition = json.dumps({"StartAt": "Done", "States": {"Done": {"Type": "Succeed"}}})
    tags = [{"key": "purpose", "value": "inventory-emulator-fixture"}]
    for index in range(2):
        name = f"{ctx.prefix}-workflow-{index}"
        response = ctx.call(
            "stepfunctions", "create_state_machine", f"state-machine-{index}",
            name=name, definition=definition, roleArn=roles["states"],
            type="STANDARD", tags=tags,
        )
        if arn := _at(response, "stateMachineArn"):
            state_machine_arns.append(arn)
            state_machine_names.append(name)
    for index in range(2):
        name = f"{ctx.prefix}-activity-{index}"
        response = ctx.call(
            "stepfunctions", "create_activity", f"activity-{index}",
            name=name, tags=tags,
        )
        if arn := _at(response, "activityArn"):
            activity_arns.append(arn)
            activity_names.append(name)


def _tables(ctx: _Seed) -> None:
    tables = []
    ctx.fixtures["dynamodb"] = {"table_names": tables}
    for index in range(2):
        name = f"{ctx.prefix}-table-{index}"
        response = ctx.call("dynamodb", "create_table", name, TableName=name, BillingMode="PAY_PER_REQUEST", AttributeDefinitions=[{"AttributeName": "id", "AttributeType": "S"}], KeySchema=[{"AttributeName": "id", "KeyType": "HASH"}], Tags=[{"Key": "purpose", "Value": "inventory-emulator-fixture"}])
        if actual := _at(response, "TableDescription", "TableName"):
            tables.append(actual)


def _kubernetes(ctx: _Seed, roles: dict[str, str], network: dict[str, Any]) -> None:
    clusters = []
    ctx.fixtures["eks"] = {"clusters": clusters}
    if len(network["subnet_ids"]) < 2 or "eks_cluster" not in roles or "eks_node" not in roles:
        ctx.skip("EKS clusters and nodegroups", "two subnets and both EKS roles are required")
        return
    for index in range(2):
        name = f"{ctx.prefix}-eks-{index}"
        response = ctx.call("eks", "create_cluster", name, name=name, roleArn=roles["eks_cluster"], resourcesVpcConfig={"subnetIds": network["subnet_ids"], "securityGroupIds": network["security_group_ids"], "endpointPublicAccess": False, "endpointPrivateAccess": True}, tags={"purpose": "inventory-emulator-fixture"})
        actual = _at(response, "cluster", "name")
        if not actual:
            continue
        cluster = {"name": actual, "arn": _at(response, "cluster", "arn"), "nodegroups": [], "nodegroup_arns": {}}
        clusters.append(cluster)
        ctx.wait("EKS cluster readiness", lambda: ctx.call("eks", "describe_cluster", actual, name=actual), lambda answer: _at(answer, "cluster", "status") == "ACTIVE")
        for ordinal in range(2):
            group = f"{ctx.prefix}-nodes-{ordinal}"
            created = ctx.call("eks", "create_nodegroup", group, clusterName=actual, nodegroupName=group, nodeRole=roles["eks_node"], subnets=network["subnet_ids"], scalingConfig={"minSize": 0, "maxSize": 1, "desiredSize": 0}, tags={"purpose": "inventory-emulator-fixture"})
            if nodegroup := _at(created, "nodegroup", "nodegroupName"):
                cluster["nodegroups"].append(nodegroup)
                if nodegroup_arn := _at(created, "nodegroup", "nodegroupArn"):
                    cluster["nodegroup_arns"][nodegroup] = nodegroup_arn


def _load_balancers(ctx: _Seed, network: dict[str, Any]) -> None:
    groups = []
    balancers = []
    ctx.fixtures["elbv2"] = {"load_balancers": balancers, "target_group_arns": groups}
    if not network["vpc_ids"] or len(network["subnet_ids"]) < 2:
        ctx.skip("ELBv2 fixtures", "a VPC and two subnets are required")
        return
    for index in range(2):
        name = f"{ctx.prefix}-lb-{index}"
        group = ctx.call("elbv2", "create_target_group", name, Name=f"{ctx.prefix}-tg-{index}", Protocol="HTTP", Port=80, VpcId=network["vpc_ids"][0], TargetType="instance", HealthCheckPath="/")
        group_arn = _at(group, "TargetGroups", 0, "TargetGroupArn")
        if group_arn:
            groups.append(group_arn)
            if network["instance_ids"]:
                ctx.call("elbv2", "register_targets", name, TargetGroupArn=group_arn, Targets=[{"Id": instance, "Port": 80} for instance in network["instance_ids"][:2]])
        response = ctx.call("elbv2", "create_load_balancer", name, Name=name, Type="application", Scheme="internet-facing", Subnets=network["subnet_ids"], SecurityGroups=network["security_group_ids"])
        arn = _at(response, "LoadBalancers", 0, "LoadBalancerArn")
        if not arn:
            continue
        item = {
            "arn": arn, "target_group_arn": group_arn, "listeners": [], "rule_arns": [],
            "listener_ports": {}, "rule_listeners": {}, "rule_priorities": {},
        }
        balancers.append(item)
        ctx.wait("ELBv2 load balancer readiness", lambda: ctx.call("elbv2", "describe_load_balancers", name, LoadBalancerArns=[arn]), lambda answer: _at(answer, "LoadBalancers", 0, "State", "Code") == "active")
        if not group_arn:
            ctx.skip("ELBv2 listener children", "target group was not created")
            continue
        for port in (80, 8080):
            response = ctx.call("elbv2", "create_listener", f"{name}-{port}", LoadBalancerArn=arn, Protocol="HTTP", Port=port, DefaultActions=[{"Type": "forward", "TargetGroupArn": group_arn}])
            listener = _at(response, "Listeners", 0, "ListenerArn")
            if not listener:
                continue
            item["listeners"].append(listener)
            item["listener_ports"][listener] = port
            for priority in (10, 20):
                rule = ctx.call("elbv2", "create_rule", f"{name}-{port}-{priority}", ListenerArn=listener, Priority=priority, Conditions=[{"Field": "path-pattern", "Values": [f"/inventory/{priority}/*"]}], Actions=[{"Type": "forward", "TargetGroupArn": group_arn}])
                if rule_arn := _at(rule, "Rules", 0, "RuleArn"):
                    item["rule_arns"].append(rule_arn)
                    item["rule_listeners"][rule_arn] = listener
                    item["rule_priorities"][rule_arn] = priority


def _containers(ctx: _Seed, roles: dict[str, str], network: dict[str, Any]) -> None:
    clusters = []
    definitions = []
    ctx.fixtures["ecs"] = {"clusters": clusters, "task_definition_arns": definitions}
    for index in range(2):
        name = f"{ctx.prefix}-ecs-{index}"
        created = ctx.call("ecs", "create_cluster", name, clusterName=name, tags=[{"key": "purpose", "value": "inventory-emulator-fixture"}])
        arn = _at(created, "cluster", "clusterArn")
        if not arn:
            continue
        cluster = {
            "arn": arn, "service_arns": [], "task_arns": [],
            "service_task_definitions": {}, "task_definitions": {},
        }
        clusters.append(cluster)
        for ordinal in range(2):
            family = f"{ctx.prefix}-task-{index}-{ordinal}"
            arguments: dict[str, Any] = {
                "family": family, "networkMode": "awsvpc", "requiresCompatibilities": ["FARGATE"],
                "cpu": "256", "memory": "512",
                "containerDefinitions": [{"name": "web", "image": "public.ecr.aws/nginx/nginx:latest", "essential": True, "memory": 128, "portMappings": [{"containerPort": 80, "protocol": "tcp"}]}],
            }
            if "ecs" in roles:
                arguments.update(taskRoleArn=roles["ecs"], executionRoleArn=roles["ecs"])
            definition = ctx.call("ecs", "register_task_definition", family, **arguments)
            revision = _at(definition, "taskDefinition", "taskDefinitionArn")
            if not revision:
                continue
            definitions.append(revision)
            if not network["subnet_ids"]:
                ctx.skip("ECS service and task", "no awsvpc subnet was created")
                continue
            networking = {"awsvpcConfiguration": {"subnets": network["subnet_ids"], "securityGroups": network["security_group_ids"], "assignPublicIp": "ENABLED"}}
            service = ctx.call("ecs", "create_service", family, cluster=arn, serviceName=f"{ctx.prefix}-service-{ordinal}", taskDefinition=revision, desiredCount=0, launchType="FARGATE", networkConfiguration=networking)
            if service_arn := _at(service, "service", "serviceArn"):
                cluster["service_arns"].append(service_arn)
                cluster["service_task_definitions"][service_arn] = revision
            # No command or entrypoint is submitted. Inventory needs task metadata;
            # it does not need a fixture to execute a program inside the emulator.
            tasks = ctx.call("ecs", "run_task", family, cluster=arn, taskDefinition=revision, count=1, launchType="FARGATE", networkConfiguration=networking)
            for task in (_at(tasks, "tasks") or [])[:1]:
                if task.get("taskArn"):
                    cluster["task_arns"].append(task["taskArn"])
                    cluster["task_definitions"][task["taskArn"]] = revision
