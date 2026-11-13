# Expansion and acceptance plan

The first section records delivered behavior. The stages that follow describe
the remaining validation and expansion work required for broad AWS coverage.

## Delivered foundation and broad collection path

- Rust observation model, bounded validation, and provenance.
- Official SDK adapters for **32 native resource/component kinds**: 13 original
  compute/container/data kinds, six VPC networking kinds, five ELBv2 kinds,
  ACTIVE ECS task-definition revisions, and seven S3/IAM/Route 53/CloudFront kinds.
- Route tables, Internet/NAT gateways, VPC endpoints, NACLs and peering, with
  separate route destinations/targets, association state, aggregated route
  evidence, and explicit ownership assumptions.
- Application/Network/Gateway Load Balancers, listeners, target groups,
  target-health memberships and application listener rules, with weighted
  forwarding, conditions/rewrites and typed target references.
- Existing Config aggregator and Resource Explorer view ingestion, including
  recognized networking/ELBv2 kind aliases while preserving source-specific raw
  data and field limitations.
- A shared Cloud Control list/read adapter for 397 schema-backed regional types
  across 76 service families, with 4,104 selected provider columns and 945 explicit
  relationship rules across 244 types.
- An offline collection planner, a full 1,881-schema capability/exclusion matrix,
  and a Rust generator for 397 source-specific SQL views. Eligibility, implemented
  mappings and observed emulator outcomes remain separate evidence categories.
- All 437 official service SDKs pinned in an optional dependency catalog, with
  a reproducible Rust source audit of retry, pagination and waiter capabilities.
- Per-attempt pacing/retry/cancellation, exact-bucket policies, fractional-period
  admission limits and service-provided retry directives.
- SQLx page transactions, immutable observations, scope checkpoints and SQLite
  schema **5**, including Cloud Control provenance, 397 provider-column views,
  task-definition fields, load-balancer aliases, route rows and architecture joins.
  The original 200 provider-column definitions and migration remain preserved;
  schema 5 adds 197 views transactionally.
- Organizations account/hierarchy discovery, existing-role account fan-out and
  conservative automatic source discovery.
- SQLx durable job leases/attempt history/cancellation and local crash recovery.
- Streaming snapshot diffs, bounded graph exploration and evidence-based findings.
- Streaming full-inventory JSON/JSONL export with normalized or all-observations
  provenance, coverage/failures, deterministic ordering, bounds and atomic
  no-clobber publication.
- Bounded FalkorDB export, immutable plan validation, retries for recognized
  idempotent writes and durable replay acknowledgements.
- Optional scan-to-file/graph delivery and offline `deliver` recovery, retaining
  completed artifacts and separate collection/delivery status.
- Synthetic correctness tests, CLI examples and local measurement harness.

See `VERIFICATION.md` for evidence, including any remaining build/environment
limitations. Having source code for an adapter does not establish live-account
coverage. `COLLECTORS.md`, `NETWORKING.md` and `LOAD_BALANCING.md` specify delivered
fields, operations, relationships and omissions. SQL aliases and route-row
views are not additional native resource kinds. Config/Explorer availability
does not establish native-equivalent configuration or graph enrichment.
The frozen 200-type synthetic event fixture still exercises the original
v0.3/v0.4 subset, not every current catalog type. See `COLLECTION_FIXTURES.md`,
`CLOUDCONTROL_COVERAGE.md`, `EXPORT.md` and `GRAPH_DELIVERY.md` for these boundaries.

## Stage 1: Validate the first operational scope

Choose two accounts, two regions, and deliberately different credential roles.
Use a resource fixture with native resources, a shared VPC/subnet, a denied
service, one empty service, multiple pages, and optional Config/Explorer data.
Include explicit and main route-table associations, multiple destinations to
one target, a blackhole route, cross-account peering, an unattached target group,
weighted forwarding, and repeated target IDs registered with different ports.
Collect a documented scope and compare identities/counts to authoritative APIs.

Acceptance criteria:

- Every selected root/parent enumeration ends complete or explicitly failed.
- Exercise the implemented service-provided retry delays and period policies
  below one request per second; verify cooldown and recovery against the
  selected service's actual limits.
- Denials, expired credentials, missing describes and deletions during collection
  produce observable partial status.
- Config ownership and source capture time remain distinct from aggregator scope.
- Cross-account network references resolve correctly or remain explicitly uncertain.
- VPC-owner subnet/Internet-gateway references join to the reported owner;
  arbitrary IP targets remain unresolved and do not become fabricated EC2 nodes.
- A denied listener/rule/target-health child preserves valid parent observations
  and marks the child scope incomplete; a successful empty child is distinguishable.
- Ctrl-C during configuration loading, admission, a request, normalization and
  persistence drains bounded work and leaves honest status.
- Resuming an exported graph after an ambiguous server response does not duplicate
  nodes/edges; incomplete projection remains marked loading.

Add tested IAM policy examples for the actual collection operations. Do not
substitute broad administrator credentials for a verified read-only contract.

## Stage 2: Complete architectural links and expand service coverage

The first network and modern load-balancer expansion is delivered. The next work
should connect those observations to their consuming workloads and fill the
remaining control-plane gaps:

| Group | Remaining native detail, scope or validation beyond catalog configuration | Useful relationships / acceptance focus |
|---|---|---|
| Networking | Referenced EIPs, egress-only/VPN/transit gateways and attachments, transit/VPN routing, prefix-list contents, endpoint services and additional network policy | Resolve existing references; retain route provenance and separate configuration from evaluated reachability |
| Traffic entry | ELBv2 tags, attributes, complete certificate lists, mutual TLS and selected authentication metadata; Classic ELB, additional CloudFront configuration, API Gateway | Add safe configuration enrichment and remaining entry points without treating omitted fields as absent |
| Naming | Cloud Map, Route 53 health checks/resolver detail beyond the existing catalog | DNS name to endpoint/service |
| Data | S3 account-level controls/additional bucket metadata, RDS/Aurora, ElastiCache, Redshift, EFS, OpenSearch; selected DynamoDB child/related configuration | Workload dependency, encryption key, subnet placement; keep content reads outside inventory |
| Identity | IAM users/groups, inline policy and policy-version metadata, deeper identity references | Assumed role, resource access evidence, trust relationships |
| Containers | Container instances, richer capacity-provider/host mappings and operational ECR/Auto Scaling details beyond Cloud Control configuration | Service to configured target group, task definition, image and host; ASG-to-target-group links |
| Events | Runtime/operational detail and validated producer-consumer relationships beyond the selected SQS, SNS, EventBridge, Lambda event-source and Step Functions provider models | Producer/consumer and invocation chains |
| Organization | Live validation of Accounts/OUs and existing-role assumption; explicit cross-host scheduling if needed | Account coverage, credential refresh and role-assumption failures |

ECS service-to-target-group edges and ACTIVE task definitions now connect
configured workloads to load-balancing and role/filesystem references. Cloud
Control adds regional configuration breadth across the groups above. Additional native
operational detail, further global types, required-parent types and live validation
remain separate work. Matching an IP registration to a workload still requires
explicit network identity evidence.

Tag enrichment is still missing for DynamoDB, Lambda and ELBv2. Full listener
certificates, authentication details, load-balancer/target-group attributes,
metrics and anomaly detection are outside the delivered ELBv2 contract. Any
authentication enrichment should select the needed metadata and continue to
exclude secrets. Kubernetes objects inside EKS require a separate authenticated
Kubernetes API collector and coverage model.

Do not equate a public IP with proven internet reachability or an attached role
with effective permission. Those analyses need the relevant network and policy
resources plus explicit evaluation semantics.

## Stage 3: Extend the reviewed resource manifest

The Cloud Control catalog already drives one shared adapter and generated SQL
views for 397 types. For additional native families and parent/global discovery,
extend that pattern with:

- SDK crate/operation, endpoint scope and accepted partitions.
- Root operation and required parent scopes.
- Pagination input/output tokens, page-size range, max pages/items.
- ID/ARN and actual owner extraction.
- Selected SQL columns with nullability, type and enrichment requirements.
- Tags policy and excluded sensitive fields.
- Batch describe limits and missing-response behavior.
- Quota-group identity, documented defaults and account override strategy.
- Relationship targets, ownership evidence and resolution rules.
- IAM actions and deterministic response fixtures.

The existing Rust view generator covers provider models. Add typed native
adapter generation only where SDK-specific glue remains repetitive. Keep service exceptions explicit. Validate generated
code against the pinned SDK before raising the declared coverage level.

A useful coverage scale is: dependency available; enumeration implemented;
selected fields implemented; relationships implemented; synthetic tests passed;
live fixture validated. Do not collapse these levels into a single support tick.
Keep the generated operation catalog and SQLite view definitions consistent;
adding an SDK crate or a convenience view does not increase the native-kind count.

## Stage 4: Selective enrichment and durable resume

The offline `plan` command now describes selected operations, dependencies,
quota groups, request-count formulas and bounds. It does not yet deduplicate
network calls across sources or resume a native enumeration after a crash.

Use Config and Explorer as discovery inputs with explicit source-scope metadata.
Continue normalizing only recognized source types and fields, retaining the
original configuration and its capture/report time. Source support and property shape can
differ from native APIs, and a parent resource type does not imply availability
of operational components such as target-health memberships.
Schedule native enrichment where identity/fields are missing or stale. Record
why a call was skipped, and do not let an incomplete index suppress native
enumeration without a documented user-selected coverage policy.

Durable whole-scan jobs already provide ownership, bounded leases, attempt
budgets and terminal outcomes. A separate future increment can add per-collector
work items and an enumeration epoch. Resume tokens only when the collector
declares them valid for the same parameters and epoch. Expired/invalid tokens
restart an enumeration without overwriting previous evidence.

For distributed collection, assign exclusive account/region/quota ownership or
coordinate a shared limiter. Independent local token buckets are insufficient.

## Stage 5: Storage scale and graph semantics

The current store already uses bounded multi-row inserts, with local baseline
and candidate measurements retained in the delivery. Shared-host measurements
have not established a stable throughput improvement from batching. Next measure
representative resource payloads and retained history before adding materialized current-state
tables, incremental relationship indexes, compact raw payload storage or history
retention. Compare throughput, tail latency, write amplification, RSS and restart
behavior, not just a synthetic rows/second number.

The schema 2 architecture view already identifies endpoints observed by exact
resource ID and flags unresolved references. Add uniquely resolvable network
aliases with explicit ambiguity rules and retain resolution provenance; do not
silently rewrite an assumed owner as an established owner.
Introduce relationship-specific FalkorDB labels/types or a Cartography
compatibility projection only with a versioned schema and query migration plan.
Retain the generic normalized projection and its explicit unresolved-reference
semantics. Use all-observations inventory export when alternative source/page
evidence must be transferred; the graph does not include every raw observation.

Streaming diffs already compare finalized snapshots without claiming deletion.
An automatic deletion policy requires complete matching enumeration scopes, stable
identity, a staleness policy and explicit partial-scan handling. Deleted,
unobserved, inaccessible and unsupported are different states.

## Stage 6: Release gates

Required before a production release:

- Reproducible builds and release checksums for supported targets.
- Dependency advisory/license review and recorded exceptions.
- Multi-account/region/partition live fixture tests.
- Stress, memory, fault injection, cancellation and crash/replay tests.
- Database migration compatibility and backup/restore exercise.
- Credential-provider, IAM and partial-coverage documentation.
- Metrics export, latency/queue saturation visibility and operational runbook.

SQLx/SQLite and SDK dependency availability are solved pieces of the foundation.
Reliable broad coverage remains the largest engineering and maintenance task.
