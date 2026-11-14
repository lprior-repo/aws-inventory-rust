# AWS collectors

> Status: Baseline being replaced. It describes the hand-written typed-SDK
> adapters and the Cloud Control path as they work today; ROADMAP M5 replaces
> the adapters service by service with overlay data on the wire engine and M9.5
> deletes Cloud Control and the old `scan` path. See [VISION](VISION.md) and
> [ROADMAP](ROADMAP.md).

The `inventory-aws` crate performs read-only inventory with the official AWS Rust
SDK. It streams pages to the caller's bounded event channel. It does not invoke
the AWS CLI, enable services, create indexes or recorders, change IAM, or retain an account-sized resource vector. Existing roles can be
selected explicitly or through bounded Organizations account discovery.

The pinned AWS API model catalog records service and operation metadata. A
modeled operation does not, by itself, implement a resource collector. The
native coverage is **39 resource/component kinds**: the original 13 compute,
container and data kinds, six VPC networking kinds, five ELBv2 kinds, and
ACTIVE ECS task-definition revisions, plus seven S3/IAM/Route 53/CloudFront
kinds described in `GLOBAL_INVENTORY.md`, SQS queues, SNS topics and
subscriptions, KMS keys and aliases, and Step Functions state machines and
activities. An additional shared Cloud Control
adapter collects 421 schema-backed regional types across 76 service families:
397 frozen root types and 24 parent-dependent types. See
[COLLECTION_PLAN.md](COLLECTION_PLAN.md) and [PARENT_COLLECTIONS.md](PARENT_COLLECTIONS.md).
The provider views added through SQLite schema version **13** expose their 4,309 selected provider columns and architectural
evidence; the current on-disk schema is version **13**, which adds the SQS queue, SNS topic/subscription, KMS key/alias, Step Functions state-machine/activity views and the final parent views. SQL aliases and expanded route rows do not add native resource kinds.
The catalog declares 968 property relationship rules across 255 types, plus 24
runtime `CHILD_OF` workflows; 268 distinct types have at least one of these.
An eligible field must actually be returned for a property rule to emit an edge.
The remaining descriptors
still provide resource observations without a complete graph of their dependencies.
See [CLOUDCONTROL_COVERAGE.md](CLOUDCONTROL_COVERAGE.md) for the exact selection,
mapping constraints and independent executed-evidence matrix.

## Identity and regions

`scan(AwsScanOptions, Arc<ApiExecutor>, Sender<CollectionEvent>)` scans one AWS
credential profile or assumed-role account target. Initial SDK configuration loading has a 30-second timeout
and is interrupted by executor cancellation. A pre-cancelled executor returns
before credential-provider setup, including for queued profiles.
The standard AWS credential provider chain is used; profile
selection is optional. `sts:GetCallerIdentity` derives the account and ARN
partition from the actual caller. The caller can run several profiles through a
shared executor and storage writer, using a distinct scan ID per profile to
avoid scope collisions when several profiles reach the same account.

Explicit regions are deduplicated. If regional native, Cloud Control or automatic source discovery is selected and no
regions were supplied, `ec2:DescribeRegions` discovers enabled regions. Region
discovery failure produces a failed coverage scope; optional supplemental
sources can still run. Bootstrap identity failure is returned to the caller.
There is a hard limit of 64 regions per invocation. Region work is bounded to
four concurrent regions. Services within each region run concurrently, subject
to the shared request executor's admission controls.

Every explicit inventory SDK operation, including STS and region discovery,
runs through `ApiExecutor`. SDK retries are set to one attempt, so the executor
owns application retries and rate accounting. Credential-provider work, such as
standard SSO credential refresh, remains part of the SDK credential provider.
SDK clients are reused within each collector/region across its pages.
The public async scan boundary contains one boxed `Send` future per profile.
Region, service, pagination-scope, and bounded enrichment-group boundaries also
erase their concrete future types. A region assembles at most twelve collector
group futures (ten direct groups, one bounded Cloud Control stream and optional source discovery) before awaiting them. The base EC2 and networking groups share the
same EC2 quota family. This keeps the SDK's large future tree inside this
crate and bounds downstream compiler/type-layout expansion. Service requests
remain typed. These are bounded orchestration allocations, not detached tasks;
they do not establish a throughput result.

## Direct collectors

Each row has the common fields `id`, `account_id`, `region`, `kind`, `name`,
`properties`, `tags`, `source`, and `observed_at`. IDs use returned AWS ARNs or
EC2 ARNs constructed from account, partition, region, resource type, and AWS
resource ID. Where an owner is not reported, provenance records the scanning
account assumption. ELBv2 target-health memberships use an explicitly synthetic
tuple ID rather than an invented AWS ARN. Reported EC2 owner IDs and
resource ARNs take precedence over the caller account, so a RAM-shared subnet,
VPC, or security group can belong to a different account from the scan scope.
`observed_at` is the local
observation time in epoch milliseconds.

| Kind | API calls | Selected properties |
|---|---|---|
| `aws_ec2_instance` | `DescribeInstances` | instance ID/type/state, image, architecture, VPC/subnet, availability zone, private/public IP, security groups, instance profile ARN |
| `aws_vpc` | `DescribeVpcs` | VPC ID, CIDR, state, default flag, DHCP options, tenancy, owner |
| `aws_vpc_subnet` | `DescribeSubnets` | subnet/VPC IDs, CIDR, availability zone, state, available IP count, public-IP launch flag, default-for-AZ flag, owner |
| `aws_vpc_security_group` | `DescribeSecurityGroups` | group ID/name, VPC, description, owner, ingress/egress protocols and port ranges, IPv4/IPv6 ranges, group references, prefix-list references |
| `aws_ec2_network_interface` | `DescribeNetworkInterfaces` | ENI/VPC/subnet IDs, status, private IP, interface type, description, attachment, security groups |
| `aws_ebs_volume` | `DescribeVolumes` | volume ID, size, type/state, encryption, AZ, IOPS, throughput, KMS key, snapshot, instance attachments |
| `aws_dynamodb_table` | `ListTables`, `DescribeTable` | table name/ID/status, item count, byte count, key schema, attribute definitions, billing mode, deletion protection, provisioned throughput, latest stream ARN |
| `aws_lambda_function` | `ListFunctions` | function name, runtime, handler, memory, timeout, role, code size, description, version, modification time, package type, architectures, VPC/subnets/security groups |
| `aws_sqs_queue` | `ListQueues`, `GetQueueAttributes`, `ListQueueTags` | queue ARN/name/URL, FIFO flag, approximate queued and not-visible counts, visibility timeout, creation timestamp, KMS key ID, SQS-managed SSE flag, redrive maximum receive count, tags; a `DELIVERS_TO` edge only when `RedrivePolicy.deadLetterTargetArn` is a parseable queue ARN |
| `aws_sns_topic` | `ListTopics`, `GetTopicAttributes`, `ListTagsForResource` | topic ARN/name, display name, owner, confirmed/pending/deleted subscription counts, topic-policy presence (no body), delivery-policy presence (no body), KMS master key ID, FIFO flag, content-based deduplication, tags |
| `aws_sns_subscription` | `ListSubscriptions` | identity from the subscription ARN, or a deterministic synthetic digest for `PendingConfirmation` rows; topic ARN, protocol, endpoint as returned, owner, pending flag; a `SUBSCRIBES_TO` edge only when `TopicArn` is a well-formed in-scope SNS topic ARN |
| `aws_kms_key` | `ListKeys`, `DescribeKey`, `GetKeyRotationStatus`, `ListResourceTags` | exact key ARN identity and key ID, account, description, key manager/usage/state/spec, origin, enabled and multi-Region flags, creation time, rotation enabled/period and tags for customer-managed keys; no key policy body or key material. A failed describe discards that key; a failed rotation or tag read keeps it with `rotation_collected` or `tags_collected` false |
| `aws_kms_alias` | `ListAliases` | exact alias ARN identity and alias name, target key ID, creation/last-updated times; no tags are returned by `ListAliases`, and the target key remains a property rather than an inferred edge |
| `aws_step_functions_state_machine` | `ListStateMachines`, `DescribeStateMachine`, `ListTagsForResource` | exact state machine ARN identity and name, list and describe types, status, creation time, role ARN, revision, description, logging/tracing/encryption configuration and tags; definitions, variable references and execution data are never stored. A failed describe discards that state machine; a failed tag read keeps it with `tags_collected=false`. `ASSUMES_ROLE` requires an exact in-account IAM role ARN and `ENCRYPTED_BY` an exact in-scope KMS key ARN; logging destinations emit no CloudWatch edge |
| `aws_step_functions_activity` | `ListActivities`, `DescribeActivity`, `ListTagsForResource` | exact activity ARN identity and name, creation time, encryption configuration and tags; no definition or execution data. A failed describe discards that activity; a failed tag read keeps it with `tags_collected=false`. `ENCRYPTED_BY` requires an exact in-scope KMS key ARN |
| `aws_ecs_cluster` | `ListClusters`, `DescribeClusters` with tags | cluster ARN/name/status, registered-container-instance count, running/pending task counts, active-service count, capacity providers |
| `aws_ecs_service` | `ListServices`, `DescribeServices` with tags | service ARN/name/status, cluster, task definition, desired/running/pending counts, launch type, platform version, scheduling strategy, subnets/security groups, primary/alternate target groups and Classic load-balancer references |
| `aws_ecs_task` | `ListTasks`, `DescribeTasks` with tags | task/cluster/task-definition ARNs, desired/last status, launch type, CPU/memory, AZ, group, container instance, stop details, selected container fields |
| `aws_ecs_task_definition` | `ListTaskDefinitions`, `DescribeTaskDefinition` with tags | ACTIVE revisions, full revision ARN, roles, runtime/platform, selected containers and volume configuration; no plaintext environment values |
| `aws_eks_cluster` | `ListClusters`, `DescribeCluster` | cluster name/ARN/status/version, endpoint, platform version, role, VPC/subnets/security groups, endpoint access flags and public CIDRs |
| `aws_eks_node_group` | `ListNodegroups`, `DescribeNodegroup` | nodegroup name/ARN, cluster, status/version/release, instance types, role, subnets, disk size, capacity/AMI type, scaling sizes, ASG names |
| `aws_vpc_route_table` | `DescribeRouteTables` | route-table/VPC IDs, owner, main flag, destination/target/state/origin for each route, explicit associations, propagating VPN gateway IDs |
| `aws_vpc_internet_gateway` | `DescribeInternetGateways` | gateway ID, owner, attached VPC IDs and attachment states |
| `aws_vpc_nat_gateway` | `DescribeNatGateways` | gateway/VPC/subnet IDs, state, connectivity and availability modes, addresses, ENIs/EIP allocation IDs, route-table field, creation/deletion times and failure details |
| `aws_vpc_endpoint` | `DescribeVpcEndpoints` | endpoint type/ID/owner, VPC, service name/region, state, route tables/subnets/security groups/ENIs, DNS configuration, policy text, service-network/resource-configuration ARNs |
| `aws_vpc_network_acl` | `DescribeNetworkAcls` | ACL/VPC IDs, owner/default flag, ordered rule numbers, direction, protocol, allow/deny, CIDRs, ports/ICMP fields and subnet associations |
| `aws_vpc_peering_connection` | `DescribeVpcPeeringConnections` | connection ID/status, expiration, separate requester/accepter VPC IDs, owners, regions, CIDRs and peering options |
| `aws_elbv2_load_balancer` | `DescribeLoadBalancers` | ARN/name, application/network/gateway type, scheme, DNS name, state, IP-address type, VPC/subnets/security groups and Availability Zone address details |
| `aws_elbv2_listener` | `DescribeListeners` | listener/load-balancer ARNs, port/protocol, SSL policy name, ALPN, returned default certificate and default actions with weighted target groups |
| `aws_elbv2_target_group` | `DescribeTargetGroups` | ARN/name, target type, protocol/version/port, VPC, associated load balancers, health-check settings, matcher, IP-address type and control port |
| `aws_elbv2_target_health` | `DescribeTargetHealth` | group/type/target ID, registered port/AZ/QUIC server ID, health-check port, health state/reason/description and administrative override |
| `aws_elbv2_rule` | `DescribeRules` | application listener rules, including default rules; ARN, parent listener, priority/default flag, conditions, actions, weights and host/URL rewrites |

| Native selector | Included collector groups |
|---|---|
| `ec2`, `vpc`, `ebs` | Six base EC2 kinds plus all six networking kinds |
| `network` | Six networking kinds only |
| `dynamodb`, `dynamo` | DynamoDB tables |
| `lambda` | Lambda functions |
| `sqs` | SQS queues with bounded attribute and tag enrichment |
| `sns` | SNS topics and account-wide subscriptions filtered to the scan region |
| `kms` | KMS keys and aliases with per-key rotation and tag enrichment |
| `step-functions`, `stepfunctions`, `sfn` | Step Functions state machines and activities with per-item describe and tag enrichment |
| `ecs` | ECS clusters, services, tasks and ACTIVE task-definition revisions |
| `ecs-task-definitions` | ACTIVE task-definition revisions only |
| `eks` | EKS clusters and managed nodegroups |
| `elbv2`, `elb`, `load-balancing` | Five modern ELBv2 kinds; the `elb` alias does not select Classic ELB |
| `s3` | General-purpose bucket listing, regional metadata sections and tags |
| `iam` | Roles, instance profiles and customer-managed policy metadata |
| `route53` | Hosted zones and record sets |
| `cloudfront` | Distributions and their selected embedded origin/behavior metadata |
| `all` or an empty selector list | All sixteen native collector groups, covering 39 kinds including global inventory |

SNS subscriptions are discovered from the account-wide `ListSubscriptions`
response and retained only when the subscription ARN belongs to the scan
account, region and partition. Confirmed subscriptions use that ARN as identity;
`PendingConfirmation` rows use a deterministic digest identity scoped to the
topic. `SUBSCRIBES_TO` edges target the returned `TopicArn` only when it is a
well-formed in-scope topic ARN; an unobserved target stays an exact-ARN dangling
reference, matching the other native edge rules. Endpoints are stored as AWS
reports them, which for `email`, `email-json` and `sms` protocols can contain
personal contact data. Topic and delivery policy bodies are never stored.

KMS keys are enumerated regionally by `ListKeys` and described with
`DescribeKey`; only customer-managed keys receive `GetKeyRotationStatus` and
`ListResourceTags`. Aliases are enumerated independently by `ListAliases`, keep
the exact alias ARN as identity, and retain the target key ID as a property.
Every returned key and alias ARN must match the scan partition, account and
region. A describe failure is recorded in a child scope and discards that key;
rotation and tag failures are recorded in child scopes and retain the key with
explicit `rotation_collected`/`tags_collected` false values. No key policy body
or key material is requested or stored, and no alias-to-key edge is inferred.

Use `--cloud-control` to add all 421 catalog types, or `--resource-type` to
select exact CloudFormation type names. `--service cloud-control --cloud-control`
selects only that broad source. The `plan` command performs the same selector
expansion offline and exposes the collection stages and request-cost uncertainty.
A child-only selection adds its root prerequisite; multiple selected siblings
share one root enumeration. Selecting a root alone does not enable its children.

The parent workflows cover reviewed children of EKS clusters, ELBv2 load balancers,
API Gateway REST APIs and V2 APIs, Logs log groups, and AppConfig applications.
They obtain their model from a successful sanitized parent `GetResource`, pass
that model on every child list page, and verify it again against child details
before attaching a `CHILD_OF` edge. Each root group admits at most 10,000 child
list models and 100,000 distinct child identifiers across its selected families.
Failed or incomplete prerequisite discovery remains explicit coverage failure;
only a successful exhaustive empty parent list proves an empty child discovery.
These plans use the existing two Cloud Control SDK operations and shared retries.
The exact models, scope identities and exclusions are in
[PARENT_COLLECTIONS.md](PARENT_COLLECTIONS.md).

The native selectors do not mean every AWS service or every resource in a named
service. With an explicit existing aggregator/view or `--discover-sources`, the `config` and
`resource-explorer` selectors can run supplemental discovery without selecting
native groups. Supplemental sources supplied alongside native groups also run.

These are control-plane metadata reads. DynamoDB items, Lambda code, container
logs, EBS contents, and application data are not scanned. DynamoDB item/byte
counts are the values reported by `DescribeTable`, not a freshly counted scan
of the table contents.

EC2, networking, ECS, and EKS tags included in these responses are retained.
DynamoDB, Lambda, and ELBv2 tag APIs are not called; those resources have
`properties.tags_collected = false`. An empty `tags` map on those resources must
not be interpreted as proof that AWS has no tags. Lambda `ListFunctions` omits
state, state reasons, update status/reasons, and runtime-version configuration;
this collector does not call `GetFunction` to enrich those fields. Its mapped
`state` value is therefore normally null. `ListFunctions` can include
environment variables in the transient SDK response, but the mapper excludes
them from persisted resource and graph records. Function code is not fetched.
Other configuration fields absent from the table above are outside this
collector's field contract.

The networking details and edge evidence contract are in
[NETWORKING.md](NETWORKING.md). Route destinations remain distinct from target
references. Repeated routes to one target share an edge whose evidence array
preserves their differences. Main route-table records do not fabricate implicit
subnet associations. Regional NAT gateways can legitimately lack a subnet ID.

The ELBv2 details are in [LOAD_BALANCING.md](LOAD_BALANCING.md). Load-balancer and
target-group discovery are independent, so unattached target groups are
collected. Listeners cover application, network, and gateway types; rule
enumeration is restricted to application listeners. Target-health identities
include target group, target ID, optional port, optional Availability Zone, and
optional QUIC server ID. Arbitrary IP targets receive group-scoped unresolved
references with unknown ownership. Lambda target ARNs retain any qualifier.

ELBv2 listener certificates are the values returned by `DescribeListeners`,
with `certificate_list_complete=false`; the full certificate-list operation is
not called. Authentication action types/presence are retained with
`authentication_details_collected=false`; OIDC client secrets are excluded.
ELBv2 attributes, mutual TLS details, SSL policy contents, WAF associations,
metrics and target-health anomaly detection are outside this field contract.
Classic ELB and application-level routing evaluation remain outside the native
ELBv2 contract. Route 53 hosted zones and record sets are collected separately by
the global adapter described in [GLOBAL_INVENTORY.md](GLOBAL_INVENTORY.md).

DynamoDB descriptions are bounded to 16 concurrent requests per list page.
ECS traverses up to four clusters per cluster page; service descriptions use
batches of at most 10, with at most four such batches in flight. Task
descriptions use at most 100 ARNs. Tasks with desired status `RUNNING` and
`STOPPED` use separate coverage scopes. Stopped-task inventory is limited to
what ECS still retains and returns; it is not historical task storage. EKS
traverses up to four clusters per list page and describes up to eight nodegroups
concurrently per nodegroup page. All of these requests also pass the global and
quota-bucket limits.

Networking runs six independent paginated scopes. ELBv2 runs two independent
roots, traverses up to four load balancers and four target groups concurrently
per respective response page, and up to four listener-rule scopes per active
load-balancer child. `DescribeTargetHealth` has no pagination; each target group
has one bounded response. These limits compose with the shared executor rather
than creating one task per discovered object.

## Relationships

Edges retain source and observation time. Targets can be unresolved because
the corresponding collector is disabled, denied, or not implemented. Such an
edge identifies a known reference without fabricating the target's properties.

Many network references provide only a VPC/subnet/security-group ID and omit
the owner's account. Their constructed target ARN assumes the scanning
account, and the edge explicitly records `target_resource_id`, `target_kind`,
`target_region`, and `owner_resolution = "assumed_scanning_account"`.
Cross-account network resolution is therefore incomplete: a shared resource's
authoritative ARN may differ from that provisional target ARN. Subnet-to-VPC
edges use the subnet's reported owner and mark that evidence separately. Route
table/ACL subnet associations and known internet-gateway references use the
VPC-owner derivation where the reported parent owner establishes it. Peering
participant owners and regions remain separate. No
extra unbounded describe calls or account-wide identity cache are used to hide
this uncertainty.

| Source kind | Relationships |
|---|---|
| EC2 instance | `IN_VPC`, `IN_SUBNET`, `USES_SECURITY_GROUP`, `USES_INSTANCE_PROFILE` |
| Subnet, security group | `IN_VPC` |
| ENI | `IN_VPC`, `IN_SUBNET`, `ATTACHED_TO` instance, `USES_SECURITY_GROUP` |
| EBS volume | `ATTACHED_TO` instance |
| Lambda function | `ASSUMES_ROLE`, `IN_VPC`, `USES_SUBNET`, `USES_SECURITY_GROUP` |
| ECS service | `IN_CLUSTER`, `USES_TASK_DEFINITION`, `USES_SUBNET`, `USES_SECURITY_GROUP`, `USES_TARGET_GROUP`, `USES_CLASSIC_LOAD_BALANCER` |
| ECS task | `IN_CLUSTER`, `USES_TASK_DEFINITION`, `RUNS_ON` container instance; available attachment details add `USES_NETWORK_INTERFACE` and `IN_SUBNET` |
| ECS task definition | `USES_TASK_ROLE`, `USES_EXECUTION_ROLE`, `MOUNTS_EFS`, `MOUNTS_FSX`, `MOUNTS_S3_FILES`; filesystem edges represent declared volume configuration |
| EKS cluster | `ASSUMES_ROLE`, `IN_VPC`, `USES_SUBNET`, `USES_SECURITY_GROUP` |
| EKS nodegroup | `IN_CLUSTER`, `ASSUMES_ROLE`, `USES_SUBNET` |
| Route table | `IN_VPC`, `MAIN_ROUTE_TABLE_FOR`, `ASSOCIATED_WITH_SUBNET`, `ASSOCIATED_WITH_GATEWAY`, `ROUTES_TO`, `PROPAGATES_FROM` |
| Internet gateway | `ATTACHED_TO_VPC` |
| NAT gateway | `IN_VPC`, `IN_SUBNET`, `USES_NETWORK_INTERFACE`, `USES_ELASTIC_IP`; `USES_ROUTE_TABLE` only for an `rtb-` value |
| VPC endpoint | `IN_VPC`, `USES_SUBNET`, `USES_SECURITY_GROUP`, `USES_NETWORK_INTERFACE`, `USES_ROUTE_TABLE`, `TARGETS_SERVICE_NETWORK`, `TARGETS_RESOURCE_CONFIGURATION` |
| Network ACL | `IN_VPC`, `ASSOCIATED_WITH_SUBNET` |
| Peering connection | `REQUESTER_VPC`, `ACCEPTER_VPC` |
| ELBv2 load balancer | `IN_VPC`, `USES_SUBNET`, `USES_SECURITY_GROUP` |
| ELBv2 listener | `LISTENER_OF`, `FORWARDS_TO` |
| ELBv2 rule | `RULE_OF`, `FORWARDS_TO` |
| ELBv2 target group | `TARGET_GROUP_OF`, `IN_VPC`, `HAS_TARGET` membership; the health child scope emits `HAS_TARGET` |
| ELBv2 target-health membership | `TARGETS` typed instance, IP, Lambda, ALB, or explicitly unresolved future-type reference |

IAM roles and instance profiles are independently enumerated when the global
`iam` collector is selected and authorized. Container instances and Auto Scaling
groups are not native inventory kinds; selected Cloud Control types can provide
additional configuration observations. A reference alone does not imply that
its target was collected or that its complete configuration was scanned. Config
aggregator and Resource Explorer adapters emit resource records without
relationship edges. Cloud Control emits relationships from the catalog's
explicit mapping rules, with typed ARN and reviewed short-ID checks.

ECS service-to-target-group relationships retain primary and alternate target
configuration evidence. Classic load-balancer names remain references with an
explicit account assumption. ACTIVE task-definition revisions are independently
enumerated; their role and filesystem edges describe configured references.
A target registration can reference an inventoried instance,
but an IP registration does not automatically identify an ECS task. Reachability,
effective permissions, and a complete request path through the architecture
are not inferred by this inventory.

SQLite schema 2 adds views for the eleven new native kinds. The
`aws_ec2_application_load_balancer`, `aws_ec2_network_load_balancer`, and
`aws_ec2_gateway_load_balancer` views filter the generic ELBv2 load-balancer view
by type. `scan_vpc_routes` and `aws_vpc_route` expand collected route arrays into
snapshot/latest-table rows; routes remain components of route-table resources.
`scan_architecture_edges` joins relationships to observed endpoints by exact
identity and flags missing endpoints. These are selected SQLite views, not full
Steampipe schemas or cross-account identity reconciliation.
Schema 4 added global-service views. Schema 5 adds 197 Cloud Control views to
the original 200, preserving the earlier column contracts and observations.
Portable full-inventory JSON/JSONL export and optional post-scan FalkorDB
delivery consume finalized snapshots; they do not add collector coverage.
See [EXPORT.md](EXPORT.md) and [DELIVERY.md](DELIVERY.md).

## Pagination and bounded memory

The user page size must be 1–1,000. Each API applies its own valid clamp:
EC2 instances/VPCs/subnets/security groups/ENIs use 5–1,000; volumes use 5–500;
route tables use 5–100; internet gateways, NAT gateways, ACLs, and peering use
5–1,000; VPC endpoints use 1–1,000; DynamoDB, ECS, EKS, Config, and Resource
Explorer use 1–100; Lambda uses 1–50; the four paginated ELBv2 operations use
1–400; SQS, SNS and KMS paginated listings use 1–1,000. The unpaginated target-health response accepts at most 10,000 memberships
and explicitly fails a larger response instead of truncating it.
Service pages are held only while they are mapped/enriched and sent to the
bounded channel. Child descriptions buffer at most one bounded list page.

Pages are numbered from one. Empty results with a continuation token continue
pagination. Empty results without a token emit a terminal empty page, providing
positive evidence that the selected scope was enumerated successfully. The six
networking operations normalize an empty next token to terminal, matching the
official EC2 paginator behavior; nonempty tokens always preserve continuation.

The hard scan limit is 10,000 pages per scope, with a caller-selectable lower
cap. Each scope accepts at most 100,000 enumerated items. The remaining item
budget is checked immediately after a list response, before descriptions,
resource mapping, or recursive child traversal. An oversized final response can
be received to detect the limit, but its children are not traversed or enriched.
Continuation tokens are
limited to 8 KiB each; token history is capped at 8 MiB. The collector detects
non-adjacent token cycles, not just immediate repetition. Resource and edge
counts and JSON payload sizes also pass the core model's validation before an
event is emitted.

Networking caps mapped nested components at 4,096 per resource and 65,536 per
page, with at most 4,096 evidence observations per edge. ELBv2 caps each mapped
nested array at 1,024 entries and counts normalized payload bytes as rows
accumulate. Both added modules reject repeated resource identities within a
page before persistence. Their full bounds are recorded in the linked module
documents. SDK decoding precedes these checks, so they do not prove a fixed
process RSS for arbitrary upstream response sizes.

A cap or token cycle produces a failure after the last valid nonterminal page.
It does not replace the continuation token with `None`. A page that would
exceed the item cap is not committed as a successful terminal page. The scan
therefore cannot infer full coverage or deletion from a capped traversal.

## Failure semantics and retry scopes

Throttling, transport failures, service failures, denials, timeout, cancellation,
retry exhaustion, invalid responses, and pagination limits are explicit.
Service errors are classified before the executor retries. Denials and permanent
errors are not retryable. The retry policy and bounded attempts are implemented
in `inventory-engine`; no nested SDK operation retry is enabled.

Native quota keys contain partition, caller account, region, service, and quota
family. EC2 uses a conservative shared read group; DynamoDB uses a shared
control-read group; Lambda uses a shared control-read group; ECS and EKS use
shared service read groups; ELBv2 uses `elasticloadbalancing:read`. This
conservatively combines some independently
throttled operations. It does not discover account-specific quotas or guarantee
any particular requests-per-second rate.

List failures are attached to the exact collector/parent scope. Enrichment
failures have separate description scopes, so surviving resources and other
collectors still persist. An ECS identifier listed but absent from its describe
response is recorded as `describe_missing`. That can result from concurrent
deletion or a service-side describe failure. It is not silently discarded as
proof of absence. Each ELBv2 listener, rule, and target-health enumeration has its
own parent scope. A failed child leaves valid parent observations available;
successful empty children have terminal empty pages. Networking operations have
independent root scopes, so an API denial in one does not imply that another
networking enumeration failed. Storage can classify a scan with any such
failures as partial.

The adapter spawns no detached tasks. Bounded stream combinators own concurrent
futures. A closed consumer channel returns an error and drops outstanding
collector futures. Executor cancellation interrupts inventory API admission,
backoff, and active calls; local clock/metadata mapping is synchronous and
bounded by the active page.

## Existing AWS Config aggregator

`config_aggregator` names an already configured aggregator in the bootstrap
region (the first explicit region, otherwise profile/environment/default region).
The collector only calls `SelectAggregateResourceConfig`; it does not create or
start a recorder, delivery channel, authorization, or aggregator.

The query selects account, region, ARN, resource ID/name/type, configuration,
tags, and capture time. Each resource uses the owning account and region from
its row, which can differ from the account/region hosting the aggregator.
Configuration may be a JSON object or an encoded JSON string; both are parsed.
The original configuration and capture time are retained, with selected familiar
property aliases for recognized resource types. Kind/column normalization is
source-dependent and does not reproduce all fields or relationships of a native
collector. Records without an ARN receive
an explicitly namespaced `aws-config:` identifier, not a fabricated ARN.

Recognized Config types include the six networking kinds and four ELBv2
resource kinds: load balancers, listeners, target groups, and rules. Selected
scalar and string-list aliases are copied only when present. Nested routes,
actions and attachments remain in raw `configuration`; they are not assumed to
have the native mapper's property shape. Consequently, native-style route views
do not expand routes that exist only inside raw Config configuration. Config
normalization creates no relationship edges or target-health memberships.

AWS Config advanced query only covers resources represented by the recorder and
aggregator. AWS documents that it does not query deleted resources or
`ResourceNotRecorded` configuration items. Recorder scope, source-region/account
coverage, aggregation delay, supported resource types, and permissions all
constrain results. A successful query proves traversal of that query's available
results, not complete current AWS account coverage.

## Existing Resource Explorer view

`resource_explorer_view` must be an existing regional view ARN. Its ARN selects
the query region. The collector calls `ListResources` using that view. It uses
at most 100 results per page: AWS's API documentation says setting `MaxResults`
to 1,000 suppresses `NextToken`. It does not use `Search` as a complete export
mechanism.

Resource owner account, resource region, ARN, resource type, service,
`last_reported_at`, property names, and supported tag representations are
preserved. Resource Explorer property documents are not treated as full service
configuration. Known resource types map to the direct collectors' kind names;
other types receive a sanitized `aws_...` kind. A normalized kind identifies the
resource family; it does not establish complete service configuration or native
graph enrichment. Target-health memberships and other operational components
must not be assumed available simply because a source indexes a parent type.
Modern ELB kind normalization also checks the returned ARN shape. Classic ELB,
malformed ARNs and mismatched resource types retain generic fallback kinds
rather than being counted as modern ELBv2 records. Resource Explorer metadata
does not fill the detailed native field columns merely because its kind is
recognized.

The view's filters, index coverage, supported resource types, service-linked
role state, permissions, and index freshness constrain discovery. Views can
include several accounts or regions, so row ownership is kept independently
from the query scope. No index, aggregator index, service-linked role, or view
is created. An existing view with limited visibility is not silently upgraded.

## Verification and live-environment limits

Parser tests cover Config ownership, resource type normalization, object/string
configuration, malformed rows, Resource Explorer ownership/freshness, and
shared-subnet identity/reference evidence. Pagination tests exercise empty pages with
tokens, non-adjacent token repetition, cap checkpoints, terminal empty pages,
and a permission failure after a successful page. These tests use synthetic
data and perform no AWS calls. A budget regression also verifies that a rejected
parent page cannot trigger its child-enrichment step.

Networking fixtures cover routes and blackholes, main/explicit associations,
cross-account owners, aggregated duplicate target evidence, regional NATs,
endpoint references, ACL direction, peering participants, bounds, and failed
continuations. ELBv2 fixtures cover all five kinds, weighted forwarding,
conditions/rewrites, collision-resistant membership identity, unresolved IP
ownership, secret omission, and denied/missing/deleted/empty child outcomes.

Compilation checks generated SDK method compatibility. Compilation and synthetic
tests do not validate a particular account's credentials, permissions, view
configuration, recorder coverage, quotas, or live scan completeness. A live scan
must inspect its persisted failed scopes and source freshness before drawing
coverage conclusions.

The native adapter test command for the pinned toolchain is:

```sh
cargo -Zprofile-hint-mostly-unused test -p inventory-aws --lib --locked -j 1
```

The current executed test counts, commands, results, and build limitations are
recorded in [VERIFICATION.md](VERIFICATION.md) and its referenced artifacts.
The Cargo hint uses the EC2 SDK's upstream `mostly-unused`
declaration to avoid generating unused dependency code. Build success and
synthetic tests are distinct from an AWS throughput benchmark or a live scan.

## Primary AWS references

- [Rust SDK retry behavior](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/retries.html)
- [Rust SDK pagination](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/paginating.html)
- [Rust SDK best practices](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/best-practices.html)
- [EC2 API throttling](https://docs.aws.amazon.com/ec2/latest/devguide/ec2-api-throttling.html)
- [Shared VPC owner and participant responsibilities](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-share-limitations.html)
- [VPC networking API and mapping references](NETWORKING.md)
- [ELBv2 API and mapping references](LOAD_BALANCING.md)
- [ECS API throttling](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/request-throttling.html)
- [Lambda quotas](https://docs.aws.amazon.com/lambda/latest/dg/gettingstarted-limits.html)
- [Lambda ListFunctions response boundaries](https://docs.aws.amazon.com/lambda/latest/api/API_ListFunctions.html)
- [DynamoDB constraints](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Constraints.html)
- [Config advanced queries and limitations](https://docs.aws.amazon.com/config/latest/developerguide/querying-AWS-resources.html)
- [Config aggregate query API](https://docs.aws.amazon.com/config/latest/APIReference/API_SelectAggregateResourceConfig.html)
- [Resource Explorer ListResources API](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_ListResources.html)
- [Resource Explorer supported resource types](https://docs.aws.amazon.com/resource-explorer/latest/userguide/supported-resource-types.html)
- [Rust Cargo mostly-unused hint](https://blog.rust-lang.org/inside-rust/2025/07/15/call-for-testing-hint-mostly-unused/)

## ECS task definitions and service target groups

`TASK_DEFINITIONS.md` specifies ACTIVE-only revision discovery, per-revision child
scopes, definition identity checks, bounded duplicate detection, selected fields
and sensitive-value omissions. ECS service-to-target-group edges include primary
and alternate targets with configuration evidence. A Classic load-balancer name
is retained as an assumed-account reference, not an observed Classic resource.
