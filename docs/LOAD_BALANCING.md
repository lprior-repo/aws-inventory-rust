# ELBv2 inventory and routing evidence

The `elbv2` service selector collects modern Application, Network, and Gateway
Load Balancers through the official `aws-sdk-elasticloadbalancingv2` client.
`elb` and `load-balancing` are selector aliases. The collector describes five
resource kinds and emits relationships for network placement and configured
routing. These records are configuration observations, not a proof that an
application is reachable or that a particular request traversed a route.

Implementation: `crates/aws/src/load_balancing.rs`. This document describes the
implemented collector; the overall scan, provenance, and SQL contracts are in
`ARCHITECTURE.md`, `COLLECTORS.md`, and `STORAGE.md`.

## Scope and API traversal

| Resource kind | AWS operation | Collection scope and pagination |
|---|---|---|
| `aws_elbv2_load_balancer` | `DescribeLoadBalancers` | Account and region; `Marker` / `NextMarker` |
| `aws_elbv2_listener` | `DescribeListeners` | One scope per load balancer ARN; `Marker` / `NextMarker` |
| `aws_elbv2_rule` | `DescribeRules` | One scope per **application** listener ARN; `Marker` / `NextMarker`; includes the default rule |
| `aws_elbv2_target_group` | `DescribeTargetGroups` | Account and region; `Marker` / `NextMarker`; no load-balancer filter |
| `aws_elbv2_target_health` | `DescribeTargetHealth` | One scope per target group ARN; one unpaginated response |

Load-balancer and target-group discovery are independent roots. Enumerating
target groups without a load-balancer filter includes unattached target groups;
they remain resources even when `load_balancer_arns` is empty. AWS documents that
the unfiltered target-group operation describes all target groups available to
the caller. See [DescribeTargetGroups](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_DescribeTargetGroups.html).

Listeners are collected for all three supported load-balancer types. The AWS
operation returns their default actions and, for HTTPS/TLS, the default
certificate; a separate operation is required for the complete certificate list.
The collector consequently sets `certificate_list_complete=false`. See
[DescribeListeners](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_DescribeListeners.html).

Application listener rules retain both default and conditional rules. Network
and Gateway Load Balancers retain their listener default actions without
calling `DescribeRules`. Unknown future load-balancer type strings remain in the
load-balancer record but do not opt into application-rule traversal. See
[DescribeRules](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_DescribeRules.html).

Every actual request runs through `Context::execute` using the
`elasticloadbalancing:read` quota family, alongside the existing account,
partition, and region dimensions. SDK retries are disabled by the scan's shared
configuration; the executor owns retry admission, pacing, cancellation, and
error classification. No collector task is detached.

## Selected properties

Every resource includes the standard inventory identity, source, observation
time, account, and region fields. `tags_collected=false` means the empty tag map
does not establish that the AWS resource has no tags.

| Kind | Selected properties |
|---|---|
| Load balancer | `load_balancer_arn`, `load_balancer_name`, `type`, `scheme`, `dns_name`, `state`, `state_reason`, `ip_address_type`, `vpc_id`, `subnet_ids`, `security_group_ids`, `availability_zones` |
| Listener | `listener_arn`, `load_balancer_arn`, `port`, `protocol`, `ssl_policy`, `alpn_policy`, `certificates`, `certificate_list_complete`, `default_actions`, `default_target_group_arns` |
| Target group | `target_group_arn`, `target_group_name`, `target_type`, `protocol`, `protocol_version`, `port`, `vpc_id`, `load_balancer_arns`, `health_check_enabled`, `health_check_protocol`, `health_check_port`, `health_check_path`, `health_check_interval_seconds`, `health_check_timeout_seconds`, `healthy_threshold_count`, `unhealthy_threshold_count`, `matcher`, `ip_address_type`, `target_control_port` |
| Target health membership | `target_group_arn`, `target_type`, `target_id`, `target_port`, `availability_zone`, `quic_server_id`, `health_check_port`, `target_health_state`, `target_health_reason`, `target_health_description`, `administrative_override`, `anomaly_detection_requested` |
| Rule | `rule_arn`, `listener_arn`, `priority`, `is_default`, `conditions`, `actions`, `transforms`, `target_group_arns` |

Availability-zone properties include subnet IDs, Outpost IDs, source NAT IPv6
prefixes, and returned load-balancer address/allocation information. Action
properties preserve order, direct target-group references, weighted forwarding
tuples, stickiness, redirects, and fixed responses. Authentication configurations
are represented by presence flags; their details are not copied. In particular,
OIDC client secrets do not enter the normalized inventory.

Rule conditions preserve host, path, header, method, source IP, and query-string
conditions, including the modeled regular-expression values. Rule transforms
retain host-header and URL rewrite patterns and replacements. This is selected
configuration mapping, not a routing-rule evaluator. See the AWS
[Rule](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_Rule.html)
model.

## Relationship semantics

| Origin | Relation | Destination | Evidence |
|---|---|---|---|
| Load balancer | `IN_VPC` | VPC reference | Returned VPC ID |
| Load balancer | `USES_SUBNET` | Subnet reference | Returned Availability Zone subnet ID |
| Load balancer | `USES_SECURITY_GROUP` | Security-group reference | Returned security-group ID |
| Listener | `LISTENER_OF` | Load balancer ARN | Requested parent, checked against the returned parent when present |
| Listener | `FORWARDS_TO` | Target group ARN | A default action of type `forward` |
| Rule | `RULE_OF` | Listener ARN | Requested parent |
| Rule | `FORWARDS_TO` | Target group ARN | A rule action of type `forward` |
| Target group | `TARGET_GROUP_OF` | Load balancer ARN | Returned associated load-balancer ARN |
| Target group | `IN_VPC` | VPC reference | Returned VPC ID |
| Target group | `HAS_TARGET` | Target-health membership | Returned registration from `DescribeTargetHealth` |
| Target-health membership | `TARGETS` | Typed target reference | Returned target ID interpreted using the target group's target type |

Repeated references to the same destination and relation are deduplicated
within a mapped resource. A direct forward ARN and the same ARN in a weighted
forward list therefore produce one edge. Original action arrays and weights
remain in properties, including a configured weight of zero. `FORWARDS_TO`
records configured forwarding and does not assert that the destination is
currently receiving traffic.

Network IDs do not contain ownership. Their references use the shared network
evidence convention: the generated EC2 ARN uses the scanning account, and edge
properties state `owner_resolution=assumed_scanning_account`, with the raw
resource ID, kind, and region. Shared VPC reconciliation may be needed before
treating such a reference as verified ownership.

### Target identity and ownership

A target-health row represents one registration in one target group. Its ID is
the prefix `aws-elbv2-target-health:` followed by an encoded JSON tuple:

```text
[target_group_arn, target_id, target_port, availability_zone, quic_server_id]
```

Null values are preserved. Tuple encoding avoids ambiguous delimiters, while
including the group, port, zone, and QUIC identifier prevents distinct
registrations from collapsing. This ID is explicitly synthetic; it is not
presented as an AWS ARN. AWS defines the type-specific ID, optional port/zone,
and QUIC identifier in
[TargetDescription](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_TargetDescription.html).

| Target type | `TARGETS` reference and ownership interpretation |
|---|---|
| `instance` | EC2 instance ARN expanded from the returned instance ID and scanning context; marked `assumed_scanning_account` rather than authoritative ownership |
| `ip` | Synthetic `aws-elbv2-ip-target:` reference encoded from target group ARN plus literal IP address; `target_kind=ip_address`, `owner_resolution=unknown`, `target_region=null` |
| `lambda` | Returned Lambda ARN preserved exactly, including an alias or version qualifier; `owner_resolution=returned_arn` |
| `alb` | Returned Application Load Balancer ARN preserved exactly; `owner_resolution=returned_arn` |
| Unknown future type | Synthetic group/type/ID reference with `target_kind=unknown` and `owner_resolution=unknown`; raw type and ID remain available |

An IP target is not materialized as an EC2 resource. Its reference is scoped to
the target group so that matching private IP strings in unrelated networks are
not automatically merged. IP edges retain `observation_region` separately from
the unknown target region. AWS explicitly allows IP targets representing peered
networks and on-premises resources, so inferring account ownership from an
address would be unsound. See
[Application Load Balancer target groups](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/load-balancer-target-groups.html).

`target_health_state` is AWS's returned health observation. A null state means
the response did not provide it. An unhealthy or draining target remains a
registered membership; it is not dropped. Health and configuration requests
occur at different instants, so the snapshot is not an atomic account-wide view.

## Bounds and incomplete scopes

| Boundary | Implemented ceiling or behavior |
|---|---|
| Independent root chains | Two |
| Child traversal | Four active load-balancer children, four listener-rule children per active load balancer, four target-health children; executor limits apply across all of them |
| Requested paginated page size | Configured size clamped to 1–400 |
| Pages and enumerated children | Shared configured page cap, maximum 10,000 pages; 100,000 enumerated children per scope |
| Unpaginated target-health response | At most 10,000 memberships accepted; a larger response fails its scope before row mapping |
| Nested arrays | At most 1,024 entries per mapped nested array; excess is an explicit invalid response |
| Normalized payload | Shared 10,000 resource / 100,000 edge / 16 MiB page limits and 1 MiB resource-property limit; mapped payload bytes are also counted as rows accumulate |

AWS's target-health operation exposes no continuation token. Oversized
target-health responses therefore fail explicitly rather than being silently
truncated or represented as complete. See
[DescribeTargetHealth](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_DescribeTargetHealth.html).
SDK response decoding happens before these normalization checks; these bounds
do not establish a fixed process RSS for arbitrary upstream responses.

Access denial, exhausted retries, cancellation, missing identity fields,
malformed targets, or a target group deleted during traversal produce a failure
for the affected scope. An unsuccessful request never produces a successful
empty page. A successful response containing no children produces a terminal
empty page and proves only that specific enumeration was empty. Child failures
do not discard valid parent resources or prevent unrelated child scopes from
being attempted. If a parent enumeration fails, its unseen children cannot be
enumerated; the parent failure records that coverage gap.

The shared paginator also detects repeated tokens and respects page/child caps.
Accepted nonterminal pages retain their continuation token when a later limit
or request fails. The store preserves observations and failed coverage; absence
in a partial snapshot never implies deletion.

## Deliberate omissions and verification

Classic ELB is not collected by this module. It also does not enumerate tags,
load-balancer/target-group/listener attributes, the full certificate list,
mutual TLS configuration, SSL policy contents, WAF associations, DNS records,
authentication configuration details, CloudWatch metrics, or target-health
anomaly detection. The explicit flags in relevant rows distinguish omitted
details from a claim that none exist. Complete architectural reachability would
also require application configuration and network-policy evaluation.

The module has SDK-builder fixtures for all five kinds, duplicate relationships,
weighted routing, rule conditions/rewrites, target identity collisions,
unresolved ownership, secret omission, and bounds. Injected target-health
responses exercise denied, malformed, deleted, successful empty, and healthy
sibling scope outcomes without contacting AWS. Consolidated execution results
belong in `VERIFICATION.md`; no live AWS account was used for these fixtures.
