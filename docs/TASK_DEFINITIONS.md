# ECS task definitions and service load-balancer references

> Status: Baseline being replaced. The `ecs` and `task_definitions` adapters
> described here are replaced by overlay data on the wire engine in ROADMAP
> M5.5 and deleted in the same change. See [VISION](VISION.md) and
> [ROADMAP](ROADMAP.md).

The native ECS inventory includes `aws_ecs_task_definition` resources and service
relationships to target groups and Classic load-balancer names. The `ecs`
selector includes clusters, services, tasks, and task definitions. The
`ecs-task-definitions` selector runs task-definition discovery independently of
cluster enumeration. Implementation is in `crates/aws/src/ecs.rs` and
`crates/aws/src/task_definitions.rs`; both use the existing ECS SDK dependency.

## Discovery and revision identity

Task-definition discovery calls `ListTaskDefinitions` with an explicit
`status=ACTIVE`, `sort=ASC`, and no family filter. It describes every returned
revision within the configured bounds; it does not collapse a family to its
latest revision. AWS documents a page size of 1–100 and requires the same status
filter throughout pagination. See
[ListTaskDefinitions](https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_ListTaskDefinitions.html).

Each `DescribeTaskDefinition` request uses the returned full revision ARN and
requests `include=TAGS`. The resource ID is that complete ARN, and its display
name is `family:revision`. A description must return the same requested ARN, with
matching family and revision fields. Resource account and region come from the
returned ARN. Tags returned with the description are retained. See
[DescribeTaskDefinition](https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_DescribeTaskDefinition.html).

`INACTIVE` and `DELETE_IN_PROGRESS` revisions are not separately enumerated.
An existing service or task may still reference an inactive revision; its
`USES_TASK_DEFINITION` edge remains available, even if the target was not
observed. This is a coverage limitation, not evidence that the definition was
deleted. If a revision changes status between listing and description, the
description's actual status is retained alongside
`discovery_status_filter = "ACTIVE"`.

The two operations are read-only. They require access to
`ecs:ListTaskDefinitions` and `ecs:DescribeTaskDefinition`, with the requested
tag visibility. The collector does not call secret-retrieval, object-download,
container-registry, IAM, or filesystem APIs to enrich references.

## Coverage scopes and bounded work

| Scope | Parent | Meaning of a terminal page |
| --- | --- | --- |
| `aws_ecs_task_definition_list` | `status=ACTIVE` | The ACTIVE revision listing exhausted its continuation token |
| `aws_ecs_task_definition` | Full task-definition revision ARN | That revision was successfully described, mapped, and validated |

The list scope carries empty resource/edge arrays: resources stream from their
individual description scopes. A denial, malformed response, exhausted retry
budget, or other failed description produces an explicit failure in that
revision's scope. Successful sibling descriptions continue. Therefore a complete
list scope can coexist with failed description scopes, and the scan remains
partial. A failed list cannot become a successful empty inventory.

The collector reuses one ECS client per account/region. Every API attempt enters
the shared account/region/ECS `read` bucket through `ApiExecutor`, including
retries. SDK retries remain set to one attempt. Task-definition discovery shares
the ECS quota family with cluster, service, and task collection. It adds no
independent retry engine or detached tasks.

Work is bounded by the existing paginator and enrichment helper:

- A requested list page contains 1–100 revision ARNs, and at most four
  descriptions are in flight for that page.
- The configured `max_pages` applies to listing, within the 10,000-page hard
  ceiling. A list scope can enumerate at most 100,000 revisions.
- Empty result pages with continuation tokens continue. Repeated tokens,
  malformed tokens, and page/child limits retain explicit incomplete coverage.
  An unconsumed token is not changed to a terminal page to hide a limit.
- A bounded revision-identity ledger rejects repeated ARNs across pages before
  the same description scope could be replayed with different observations.
  It stores only ARN strings, with at most 100,000 entries and 8 MiB of combined
  ARN text; no SDK definitions or mapped resource payloads accumulate across
  list pages. Cursor history is separately bounded by the shared paginator.
- One mapped definition permits at most 4,096 list/map components in aggregate,
  counting containers, volumes, nested references, tags, and retained option
  names. Core validation also limits resource/edge properties to 1 MiB each and
  aggregate event pages to 16 MiB.
- The existing bounded event channel provides backpressure. Cancellation,
  attempt deadlines, and server cooldowns are handled by the shared executor;
  channel closure remains a fatal producer error.

These are application bounds, not claims about AWS throughput or snapshot
isolation. Changes made in AWS while pagination is in progress can affect what
the service returns.

## Selected task-definition properties

| Area | Persisted fields |
| --- | --- |
| Identity and status | Full ARN, family, revision, actual status, ACTIVE discovery filter, registration/deregistration/deletion-request times in checked epoch milliseconds, registering principal |
| Runtime | Task CPU/memory strings, network/PID/IPC modes, compatibility lists, runtime CPU architecture and OS family, ephemeral-storage size, fault-injection flag |
| IAM | Task-role and execution-role ARNs |
| Placement | Required attribute names, placement constraint type/expression, inference-accelerator device name/type |
| Containers | Name/image, essential flag, CPU/memory/reservation, user, privileged/read-only flags, start/stop timeouts, version consistency |
| Container network | Named port mappings, container/host ports and ranges, protocols, links, hostname, network-disable flag, DNS settings and explicit extra hosts |
| Dependencies and mounts | Container dependency conditions, mount source/path/read-only flag, volumes-from source/read-only flag, resource requirement type/value |
| Environment references | Environment variable names, environment-file type/location, secret name/source references, repository-credential reference, credential-spec references |
| Logs | Driver, option names, `awslogs-group`, `awslogs-region`, `awslogs-stream-prefix`, and log-secret references |
| Presence and names | Command/entry-point/health-command presence, health timing settings, Docker label keys, FireLens type/option keys, proxy type/container/property names |
| Host and Docker volumes | Volume name, launch-configuration flag, host path, Docker driver/scope/autoprovision, driver-option keys and label keys |
| EFS | Filesystem ID, root directory, transit-encryption setting/port, access-point ID and IAM authorization setting |
| FSx for Windows | Filesystem ID, root directory, authorization domain and credentials-parameter reference |
| S3 Files | Returned filesystem/access-point ARNs, root directory and transit-encryption port |

The property names follow the mapper's explicit snake-case JSON contract. This
is selected architecture metadata, not a complete serialized SDK response or a
task definition that can be submitted back to ECS.

### Handling values that may contain credentials

`DescribeTaskDefinition` can deliver plaintext environment values and command
strings in the transient SDK response. The mapper does **not retain or persist**
environment values, command strings, entry-point strings, or health-check
commands. It records names or presence instead. The flags
`environment_values_collected`, `secret_values_collected`, and
`command_values_collected` make that omission explicit.

Arbitrary log-option, FireLens-option, Docker label, Docker driver-option,
Docker-volume label, and proxy-property values are also omitted. Log metadata
uses only the three named `awslogs-*` values listed above. Secret references,
environment-file locations, credential-parameter references, and registry
credential references remain configuration references; their contents are
never fetched. Tags and other selected ordinary metadata are retained as
returned, so this is a field-selection policy rather than a universal scanner
for credentials placed in unrelated metadata fields.

## Service load-balancer configuration

ECS service properties retain a `load_balancers` array with target-group ARN,
Classic load-balancer name, container name, and container port. Advanced
blue/green configuration retains its alternate target-group ARN, production/test
listener-rule references, and deployment-role ARN. AWS distinguishes these
fields in the [ECS LoadBalancer schema](https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_LoadBalancer.html).

The deduplicated `target_group_arns` array contains both primary and alternate
target groups. `classic_load_balancer_names` is also deduplicated. A repeated
target group produces one `USES_TARGET_GROUP` edge whose evidence array preserves
the load-balancer configuration index, primary/alternate role, container name,
and container port. Advanced listener-rule and deployment-role references remain
properties; this implementation does not emit separate edges for them.

Classic load-balancer names produce `USES_CLASSIC_LOAD_BALANCER` references with
target kind `aws_ec2_classic_load_balancer`. These are references only: the
collector does not describe Classic ELB resources. The modern `elbv2` collector
can resolve returned target-group ARNs when selected and authorized. A configured
service/target-group association is not proof of a currently registered target,
healthy target, or reachable application.

## Relationships and ownership

| Source | Relation | Target and evidence |
| --- | --- | --- |
| ECS service | `USES_TASK_DEFINITION` | Existing full revision reference |
| ECS service | `USES_TARGET_GROUP` | Primary or alternate AWS-returned target-group ARN; container/port and configuration-role evidence |
| ECS service | `USES_CLASSIC_LOAD_BALANCER` | Name-based Classic ELB ARN under the scanning account/region; explicitly assumed owner |
| Task definition | `USES_TASK_ROLE`, `USES_EXECUTION_ROLE` | AWS-returned IAM role ARN, preserving its account and global-region form |
| Task definition | `MOUNTS_EFS`, `MOUNTS_FSX` | Filesystem-ID reference expanded under the scanning account/region, with explicit owner assumption |
| Task definition | `MOUNTS_S3_FILES` | AWS-returned S3 Files filesystem ARN, preserving account and region |

The `MOUNTS_*` relations mean a volume is **declared in the task definition**.
Their evidence contains `configuration = "declared_volume"`, the volume index,
and volume name. They do not prove that a running task mounted the filesystem,
that every container uses that volume, or that authorization/network access
will succeed. Container `mount_points` remain available for that association.
S3 Files uses its full ARN as specified in
[Configuring S3 Files for ECS](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/s3files-volumes.html).

Returned ARNs receive `owner_resolution = "aws_returned_arn"`. Constructed
Classic/EFS/FSx references receive
`owner_resolution = "assumed_scanning_account"`, plus the raw target ID/name,
target kind, target account, and target region. Cross-account EFS/FSx ownership
cannot be determined from the filesystem ID alone and is not silently treated
as authoritative. Access-point, secret, and environment-file references remain
properties rather than invented observed resources.

Repeated references are merged by `(from_id, to_id, relation)` before event
emission, with at most 1,024 evidence entries per edge. Existing service subnet
and security-group references use the same merge helper. A task definition's
`awsvpc` mode alone creates no VPC/subnet relationship: those placements belong
to the service or running-task configuration.

## Synthetic verification

The module tests use SDK response builders and injected list/describe futures;
they require no credentials. Cases cover revision/owner identity, status changes
between list and describe, duplicate edge evidence, sensitive-value omission,
malformed references and timestamps, component/identity limits, empty list pages,
description denial, malformed description isolation, list denial, pagination
limits, duplicate revisions across pages, and malformed continuation tokens.
Native compilation and the consolidated test suite are required before claiming
the extension has passed verification. These tests do not establish live AWS
permissions, coverage, request rates, or service behavior under load.
