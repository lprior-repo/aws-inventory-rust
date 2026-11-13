# AWS Inventory in Rust

**Local stack:** [setup and commands](docs/LOCAL_SETUP.md) ·
[current results](docs/DEPENDENCY_UPDATE.md) ·
[black-hat and Rust review](docs/LOCAL_REVIEW.md).

**Continuing development elsewhere? Start with [START_HERE.md](START_HERE.md)**
for build commands, measured results, remaining work and the
portable handoff boundaries.

**Where this is going:** [the vision spec](docs/VISION.md) defines the product:
a CMDB of every AWS resource, read through the services' own endpoint APIs by
our own wire engine, kept current by fast repeated sweeps, stored in SQL and a
relationship graph, and shown as a Makepad DAG. [The roadmap](docs/ROADMAP.md)
is the build plan. Cloud Control collection, described below, is being
replaced and will be deleted (decision V04).

A working foundation for AWS architecture inventory: official Rust SDK
collectors, bounded request scheduling, SQLx with local SQLite, familiar SQL
resource views, and replayable FalkorDB snapshot projections.

The application and collectors are Rust. SQLx's bundled SQLite engine is native
SQLite C; FalkorDB is an external database. SDK/resource-catalog refresh utilities
use Python as development tooling, outside collection and the deployed application.
The SDK source auditor and SQLite view generator are Rust.
The optional AWS Emulator harness has a Rust implementation (`inventory-harness`)
for installing the pinned emulator, freezing snapshots, seeding, assertions and
live end-to-end tests; request recording and the mutation gate remain Python
until their ports land.

## Current scope

- **421 schema-backed Cloud Control resource types across 76 service families** through one shared Rust
  list/read adapter, with bounded enrichment, per-resource failures, schema-based
  redaction, source provenance, and generated SQLite views. These are configured
  collection capabilities, not a claim of live-account coverage in every region.
  The selection includes 397 root types and 24 parent-scoped types. Selecting a
  child automatically collects its required root and checks ownership before
  deriving a `CHILD_OF` relationship.
- A complete **1,881-schema capability matrix** distinguishes selected types,
  missing handlers, parent-model dependencies and unreviewed scope. The offline
  `coverage` command exposes exclusions and separately bound emulator evidence.
  SQLite schema 13 exposes 4,309 provider columns across the 421 Cloud Control
  views, plus native views for SQS, SNS, KMS and Step Functions. The catalog includes 968
  property relationship rules across 255 types plus 24 explicit parent workflows,
  covering 268 types with at least one relationship rule. A configured rule is
  distinct from an observed relationship.
- **Universal listable coverage.** `--resource-type all` attempts every one of
  the 1,291 list-capable types through the shared Cloud Control adapter, not just
  the reviewed 397. Against the pinned emulator: 1,291 attempted, 999 listed,
  292 provider-unsupported, 0 failed, with 410 parent-scoped and 180 handler-less
  types reported as explicit skips. The scan JSON carries the full partition and
  names every skip. `--service all,cloud-control --cloud-control --resource-type
  all` combines the native adapters with universal listing in one scan. See
  [universal coverage](docs/UNIVERSAL_COVERAGE.md).
- **39 direct resource and component kinds:** EC2 instances, VPCs, subnets, security groups,
  network interfaces, EBS volumes, DynamoDB tables, Lambda functions, ECS
  clusters/services/tasks and ACTIVE task-definition revisions, EKS clusters/nodegroups, route tables, internet and
  NAT gateways, VPC endpoints, network ACLs, peering connections, and ELBv2
  load balancers, listeners, rules, target groups and target-health memberships;
  S3 buckets, IAM roles/instance profiles/customer-managed policy metadata,
  Route 53 zones/records, CloudFront distributions, SQS queues, SNS topics
  and subscriptions, KMS keys and aliases, and Step Functions state machines
  and activities.
- Optional collection through an **existing AWS Config aggregator** and an
  **existing Resource Explorer view**, selected explicitly or through conservative
  automatic source discovery, including additional resource kinds they return.
- Organizations account/hierarchy discovery and optional collection through an
  existing role in selected ACTIVE accounts, with bounded targets and explicit
  credential-assumption failures. Global services run once per target scan.
- A **SQLx durable scan queue** with transactional claims, attempt history,
  renewable fenced leases, cancellation and recovery after worker crashes.
- **437 official AWS service SDK crates**, pinned to the November 12, 2026 SDK
  release, available as optional features in a generated catalog. Package
  checksums and source/model fingerprints are retained.
- An offline `plan` command exposes APIs, dependencies, quota groups, known
  request-count formulas, architecture evidence, and execution bounds before
  credentials are loaded.
- SQLx transactions atomically persist each page, its resource observations,
  edges, and continuation checkpoint. Identical page replay is idempotent.
- Query-only SQLite views with Steampipe-style names and selected columns,
  snapshot-specific route expansion, an `architecture` report command, streaming
  snapshot diff, bounded graph neighborhoods/paths and evidence-based findings.
- Graph export in bounded batches, idempotent node/edge identities, and SQLx
  delivery acknowledgements keyed by destination, graph, scan, and sequence.
- Streaming **JSON and JSONL inventory export**, including raw source/page
  provenance, scope outcomes and failures. Collection can automatically publish
  files and deliver to FalkorDB; the `deliver` command retries an existing
  snapshot without repeating AWS calls.
- Bounded FalkorDB retry with reconnects, idempotent replay after ambiguous
  network failures, cancellation drainage and receipts preserving completed
  artifacts when a later stage fails.
- Account/region/quota-family pacing, retries, timeouts, cancellation,
  adaptive throttle feedback, fractional exact-bucket ceilings, and shared
  cooldowns from server retry metadata.

SDK availability is distinct from collector coverage. This is not a complete
port of awsmap, Cartography, or every Steampipe AWS table. The CLI `catalog`
command and [collector contract](docs/COLLECTORS.md) describe the implemented
resource fields and relationships. `complete` means all attempted, selected
scopes completed; it does not mean every AWS service was scanned.

Config and Resource Explorer use the same canonical kinds for the supported
network and ELBv2 resources. Their detail remains source-dependent: Explorer
provides discovery metadata, Config retains raw configuration with selected
scalar aliases, and neither currently supplies native-equivalent relationships.
Config's raw nested route arrays are retained but are not expanded by the
normalized routes panel.

See [verification evidence](docs/VERIFICATION.md) for the actual build, test,
benchmark, and live-environment boundaries of this delivery.

## Test the actual AWS calls locally

[AWS Emulator testing](docs/EMULATOR_TESTING.md) describes the pinned full source
checkout, verified AWS binary, populated service fixtures and operation-by-operation
evidence. [The historical call matrix](docs/AWS_CALL_MATRIX.md) enumerates **72 explicit AWS
operations plus the SDK's AssumeRole boundary**, separately from the 421 Cloud
Control resource types and 437 available SDK crates.
The [current source/run reconciliation](docs/AWS_OPERATION_RECONCILIATION.md)
binds operation identities to a specific source tree and executed evidence.
The parent workflows reuse Cloud Control's two existing operation identities;
their type, ownership and pagination checks remain separate evidence dimensions.

The harness verifies TLS on both connections, uses emulator-generated credentials,
and keeps fixture mutations separate from Rust collector traffic. It checks
native resource identities, properties, relationships and pagination in SQLite.
Unimplemented endpoints and SDK decoding failures remain visible in its report.
The pinned emulator currently has Organizations and Resource Explorer limitations;
HTTP 200 alone does not establish a valid SDK response.

## Property tests and mutation testing

[Property tests](docs/PROPERTY_TESTING.md) exercise generated retry histories,
pagination, resource identities, SQLx transactions, durable job leases and the
actual CLI. Guarded integration tests compare the compiled collectors and CLI
with independently checked AWS Emulator snapshots and a real local FalkorDB.

The [mutation gate](docs/MUTATION_TESTING.md) requires a complete, frozen inventory
of production candidates across the core, engine, AWS and CLI packages. It
requires **at least 95% of viable mutations caught**, and rejects incomplete
runs, timeouts and infrastructure failures. The historical v0.4 core campaign caught
**56 of 56** candidates. The other initial campaigns were stopped as diagnostic
runs; their survivor follow-ups need a fresh full run. **The whole-project
95% requirement has not passed**. Test counts and a configured threshold are not
substitutes for that result.

The retry configuration property also found and fixed a real boundary defect:
a manually constructed shared SDK configuration could lose the one-attempt
setting when converted to a DynamoDB client. The public AssumeRole helper now
sets the retry configuration's explicit origin as well as its value. See the
[regression and scope](docs/AWS_PROPERTY_TESTS.md); normal CLI configuration
loaders already supplied that origin.

## Build

Install Rust with rustup and a C toolchain for bundled SQLite. Run commands from
the repository root so `.cargo/config.toml` applies the SQLite allocation bounds.
The toolchain is pinned in `rust-toolchain.toml`; dependencies are locked.
The repository enables the pinned nightly's selective `profile-hint-mostly-unused`
support so large SDK crates can defer code generation for unused APIs. This
resolved the EC2 build's memory failure in the delivery environment. The setting
respects upstream crate hints rather than applying the flag to every dependency.

```bash
# Fetch the locked SDK catalog and all other workspace dependencies.
cargo fetch --locked
python scripts/generate_sdk_catalog.py --check --verify-cache

# Local ingestion, SQLite SQL, and FalkorDB export; no AWS clients compiled.
cargo build --locked

# Include the implemented native AWS collectors.
cargo build --locked --features aws --jobs 1

# Audit every downloaded SDK source without compiling every service SDK.
cargo run -p inventory-sdk-catalog --example audit --locked -- \
  --catalog crates/sdk-catalog/sdk-services.json \
  --registry-src /PATH/TO/registry/src/index.crates.io-1949cf8c6b5b557f \
  --output sdk-capabilities.json
```

Normal builds do not enable all 437 SDK features. Full-catalog compilation is
an explicit, separate operation with a substantial resource cost. Selected
collectors compile the SDK crates they use.

The full maintainer checks also exercise the Python schema and fixture tools.
Their dependencies are separate from the deployed Rust application:

```bash
python3 -m venv /tmp/aws-inventory-checks
/tmp/aws-inventory-checks/bin/python -m pip install -r scripts/requirements.txt
INVENTORY_PYTHON=/tmp/aws-inventory-checks/bin/python bash scripts/verify.sh
```

Live emulator/FalkorDB tests require the explicit local fixture setup described
below. Ordinary tests leave those integration cases ignored unless the guarded
environment is prepared.

## Try it locally

The demo is synthetic and makes no AWS requests. Use a fresh database path.

```bash
cargo run --locked -- demo --db demo.sqlite
cargo run --locked -- report --db demo.sqlite --scan-id synthetic-demo-001
cargo run --locked -- query --db demo.sqlite \
  'SELECT instance_id, instance_type, vpc_id, region FROM aws_ec2_instance'
cargo run --locked -- graph-export --db demo.sqlite \
  --scan-id synthetic-demo-001 --output demo-graph.jsonl
```

The fixture contains seven resources and four relationships. It is a functional
pipeline demonstration, not a performance result or a real AWS account snapshot.

For a larger network and load-balancer fixture, follow
[architecture queries](docs/ARCHITECTURE_QUERIES.md). It exercises the new
snapshot report, route targets, target health, and unresolved references.

## Collect AWS configuration

Use an existing AWS credential profile or the standard SDK credential provider
chain. The scanner does not persist credentials from that credential provider
chain. These examples perform inventory
reads when you run them with your own credentials.

```bash
# Inspect the 421-type breadth plan and direct architecture collectors offline.
cargo run --locked -- plan --cloud-control --region us-east-1,us-west-2

# Explain selected types, exclusions and separately recorded emulator evidence.
cargo run --locked -- coverage --service SageMaker --limit 100

# Execute that plan using an existing profile.
cargo run --locked --features aws -- scan \
  --db inventory.sqlite --profile audit --region us-east-1,us-west-2 \
  --service all --cloud-control

# Collect, write portable JSONL and project the finished snapshots into FalkorDB.
# Set FALKORDB_URL in the worker environment before running this command.
cargo run --locked --features aws -- scan \
  --db inventory.sqlite --profile audit --region us-east-1 \
  --service all --cloud-control --discover-sources \
  --output-dir outputs --falkor --provenance all-observations --allow-partial

# Limit Cloud Control to a specific catalog type.
cargo run --locked --features aws -- scan \
  --db inventory.sqlite --profile audit --region us-east-1 \
  --service cloud-control --cloud-control --resource-type AWS::RDS::DBInstance

# Explicit regions are useful for a first bounded collection.
cargo run --locked --features aws -- scan \
  --db inventory.sqlite --profile audit \
  --region us-east-1,us-west-2 --service ec2,dynamodb,lambda,ecs,eks,elbv2

# Multiple profiles share one request executor and one SQLite writer.
# Each profile receives its own immutable scan ID.
cargo run --locked --features aws -- scan \
  --db inventory.sqlite --profile development,production \
  --region us-east-1,us-west-2

# Explicitly reuse supplemental sources that already exist.
cargo run --locked --features aws -- scan \
  --db inventory.sqlite --profile audit --region us-east-1 \
  --config-aggregator organization-inventory \
  --resource-explorer-view arn:aws:resource-explorer-2:us-east-1:123456789012:view/all-resources/VIEW_ID
```

Omitting regions discovers enabled regions with `DescribeRegions` when regional
collection or automatic source discovery is selected. A global-only scan does
not need regional discovery. Explicit source names/ARNs take precedence over
automatic source discovery. All infrastructure and cross-account roles must
already exist.

`--allow-partial` includes finalized partial snapshots while preserving their
actual collection status and failures. Without it, requested delivery skips
partial snapshots. Delivery is independently retryable from local SQLite:

```bash
aws-inventory export --db inventory.sqlite --scan-id SCAN_ID \
  --output inventory.json --format json --provenance all-observations --allow-partial
aws-inventory deliver --db inventory.sqlite --scan-id SCAN_ID \
  --output-dir outputs --falkor --allow-partial
```

The `deliver` command performs no AWS calls. It publishes a new attempt directory
and resumes graph acknowledgements from `outputs/projection.sqlite`. A failed
graph delivery retains the already written inventory artifact and its receipt.
See [delivery](docs/DELIVERY.md), [portable export](docs/EXPORT.md) and
[graph retry](docs/GRAPH_DELIVERY.md) for formats, bounds and recovery contracts.

```bash
# Inspect existing organization and supplemental sources without scanning them.
aws-inventory discover --profile management --organization --hierarchy \
  --config-aggregators --resource-explorer --region us-east-1

# Plan and execute through an existing role in selected ACTIVE accounts.
aws-inventory plan --profile management --organization-role InventoryReader \
  --account-id 123456789012,210987654321 --region us-east-1,us-west-2 \
  --cloud-control --discover-sources
aws-inventory scan --db inventory.sqlite --profile management \
  --organization-role InventoryReader --account-id 123456789012,210987654321 \
  --region us-east-1,us-west-2 --cloud-control --discover-sources

# Global configuration only, with the normal credential chain.
aws-inventory scan --db inventory.sqlite --service s3,iam,route53,cloudfront
```

Organizations listing must complete before automatic account fan-out starts.
Inactive or missing accounts are not silently accepted by an explicit filter.
`--max-accounts` defaults to 128 and is capped at 1,024. An optional
`--external-id-env VARIABLE_NAME` reads its value only at execution; the value is
never persisted in a queue job. See [discovery](docs/DISCOVERY.md) and
[global inventory](docs/GLOBAL_INVENTORY.md) for selection and coverage contracts.

`ec2` includes the six network collectors as well as the original EC2 resources.
Use `--service network` for only routes, gateways, endpoints, ACLs and peering,
or `--service elbv2` for load-balancer topology. `elb` and `load-balancing` are
aliases for `elbv2`; these do not collect Classic Load Balancers.
`ecs` also includes ACTIVE task-definition revisions; use
`--service ecs-task-definitions` to select only those definitions.
Cloud Control is opt-in through `--cloud-control`. Its first catalog contains
regional types that have list/read handlers and accept an empty discovery
model; parent-filtered and global types require separate planning. The official
schema snapshot establishes eligibility in us-east-1, not universal support
or permissions in every selected region. Runtime failures remain explicit.

The scan prints per-profile scan IDs, resource/edge counts, status, and API
counters. Exit status `0` means a successful command, `1` is a command error,
and `2` means a reported partial or cancelled scan. Inspect `report` after
collection: a denial or unfinished page chain cannot become an empty success.

## Durable jobs and offline analysis

```bash
# Queue validated scan intent without resolving credentials.
aws-inventory job submit --queue jobs.sqlite --job-id nightly-example \
  --db inventory.sqlite --profile audit --region us-east-1 \
  --service all --cloud-control --discover-sources
aws-inventory job run --queue jobs.sqlite --limit 1
aws-inventory job show --queue jobs.sqlite --job-id nightly-example

# Compare observations and explore the declared architecture in finalized scans.
aws-inventory diff --db inventory.sqlite --before SCAN_BEFORE --after SCAN_AFTER
aws-inventory neighbors --db inventory.sqlite --scan-id SCAN_ID \
  --resource-id RESOURCE_ARN --max-depth 2
aws-inventory path --db inventory.sqlite --scan-id SCAN_ID \
  --from RESOURCE_ARN --to TARGET_ARN --direction outgoing --max-depth 8
aws-inventory findings --db inventory.sqlite --scan-id SCAN_ID
```

Leased jobs recover after worker expiry by starting a new scan attempt. API
retries still belong to the request executor; normal partial/failed jobs require
an explicit `job retry`. Pagination checkpoints are durable evidence, but a
fresh job attempt does not resume arbitrary provider cursors. Queue and inventory
files are separate databases. See [durable jobs](docs/DURABLE_JOBS.md).

Diff reports observation changes, including source changes. It does not infer
deletion from a missing record. Graph paths follow observed configuration
references; they do not prove network reachability or IAM authorization.
See [analysis semantics](docs/ANALYSIS.md) for bounds and truncation behavior.

## Set verified rate ceilings

The default CLI admits at most 128 active requests globally, four per quota
bucket, and five starts per second per bucket. Retries count as starts.
`--concurrency`, `--per-bucket-concurrency`, `--rps`, and `--attempts` change
these bounds; they are not AWS quota discovery.

For heterogeneous ceilings, pass `--rate-policy policy.json`. Keys match the
adapter's complete quota key exactly. Unmatched buckets use the CLI defaults.

```json
{
  "aws:123456789012:us-east-1:ec2:read": {
    "requests_per_second": 15,
    "concurrency": 8
  },
  "aws:123456789012:us-east-1:lambda:control_read": {
    "requests_per_second": 10,
    "concurrency": 4
  },
  "aws:123456789012:us-east-1:elasticloadbalancing:read": {
    "requests": 1,
    "period_ms": 2000,
    "concurrency": 2
  }
}
```

These are example budgets with headroom, not guarantees about your account.
The period form expresses rates below one request per second; it spaces starts
and does not create a burst allowance. Both shapes reject unknown fields.
All EC2 and network operations share the EC2 read bucket; all five ELBv2 request
types share `elasticloadbalancing:read`. Other applications consume the same AWS account quotas. See
[scheduler design](docs/SCHEDULER.md) before increasing ceilings.

Standard `Retry-After`, AWS `x-amz-retry-after`, and Lambda's typed retry delay
extend the affected bucket's cooldown. Malformed or excessive server delays
close that bucket instead of dispatching earlier than the server requested.
Retries still consume the same admission budget. This is a process-local policy;
multiple scanner processes require separate quota coordination.

## Query the inventory

The service views expose the freshest last-known observation across scans;
source priority only breaks equal observation timestamps, so a newer Config
observation is selected over an older direct one. Use `scan_resources` or
`scan_edges` with an explicit `scan_id` for a particular priority-first
snapshot. `source` and `observed_at` remain visible on every selected row.
`latest_resource_sources` exposes the independently retained source records.
Cloud Control views use the `aws_cc_*` namespace and select their own source
records, retaining top-level provider columns and `provider_properties` JSON.
They remain inspectable even when a fresher observation from another source is
selected.

```sql
SELECT account_id, region, kind, count(*) AS resources
FROM aws_resource
GROUP BY account_id, region, kind
ORDER BY account_id, region, kind;

SELECT function_name, runtime, memory_size, region
FROM aws_lambda_function;

SELECT table_name, table_status, billing_mode, account_id, region
FROM aws_dynamodb_table;

SELECT account_id, region, collector, coverage_status
FROM scan_coverage
WHERE scan_id = 'YOUR_SCAN_ID';

SELECT from_id, relation, to_id
FROM scan_edges
WHERE scan_id = 'YOUR_SCAN_ID';

SELECT family, revision, task_role_arn, execution_role_arn
FROM aws_ecs_task_definition;

SELECT cloudformation_type, account_id, region, count(*) AS resources
FROM aws_cloudcontrol_resource
GROUP BY cloudformation_type, account_id, region;
```

Pass one statement to `query`. It runs on a separate read-only/query-only
connection with statement, result, and native execution bounds. SQLite syntax
and JSON functions apply; PostgreSQL-specific Steampipe SQL is not automatically
compatible. Absent columns are not silently fabricated.

### Architecture report

```bash
cargo run --locked -- architecture --db inventory.sqlite \
  --scan-id YOUR_SCAN_ID --limit 100
```

The finalized-snapshot report contains resource and coverage counts,
relationship counts, internet-facing load balancers and observed listeners,
routes, target-health counts, unresolved endpoints, and collection failures.
Each panel has a row limit and an explicit `truncated` flag. A route target or
load-balancer scheme is configuration evidence, not a reachability verdict.
Unknown endpoints mean they were not observed in the same scan.

Opening an older database upgrades it transactionally to schema version 13,
including when opened by query/report commands. Observation and checkpoint
tables are preserved. [Storage design](docs/STORAGE.md) describes the upgrade;
[networking](docs/NETWORKING.md) and [load balancing](docs/LOAD_BALANCING.md)
specify exactly which fields and relationships are collected.

## Retry primitives and workflow engines

The SDK source audit covers all 437 pinned service crates: 19,617 operations,
3,779 paginators, and 407 waiters. AWS already supplies retry classifiers,
backoff, timeouts, and duplicate-token controls. A service SDK does not supply
this scanner's account/region quota coordination, durable page commits, or
resource-to-architecture mapping.

This version keeps **one retry owner**: the shared executor, with each explicit
service SDK request configured for one attempt. Every retry re-enters admission.
Enabling a second five-attempt SDK loop inside the outer five-attempt loop can
produce 25 attempts and invalidate the configured accounting. The audited public
SDK hooks do not provide a proven drop-in asynchronous quota wait before signing
and before the SDK's attempt timeout. The source-backed decision is in
[SDK retry audit](docs/SDK_RETRY_AUDIT.md).

The local SQLx queue supplies durable scan submission, worker-crash recovery,
fenced ownership, cancellation and explicit retry without an additional server.
Restate and Temporal remain options for ownership and timers across multiple
machines. Neither removes quota coordination or idempotent persistence work.
The comparison and current behavior are in
[retries and workflows](docs/RETRY_AND_WORKFLOWS.md).

HTTP 501 is terminal. A modeled-error-free SDK rejection of an HTTP success
response is reported as a response-decoding/protocol problem without copying the
response body into a diagnostic. This distinction was verified against the
emulator's malformed Resource Explorer timestamps.

## Export to FalkorDB

```bash
cargo run --locked -- graph-export --db inventory.sqlite \
  --scan-id YOUR_SCAN_ID --graph aws_inventory --output graph.jsonl

export FALKORDB_URL='redis://127.0.0.1:6379'
cargo run --locked -- graph-push --input graph.jsonl \
  --checkpoint graph-delivery.sqlite
```

Graph export rejects running scans. Partial/cancelled snapshots require the
CLI's explicit `--allow-partial` flag and retain their actual status. Export
writes an adjacent `.partial` file and installs it without overwriting an existing
destination after successful flush/sync.
On an ordinary failure or cooperative cancellation, export drains its writer
and removes its own partial file. Abrupt process termination or failed cleanup
can leave a partial behind. An existing destination or another invocation's
partial is never removed; inspect such a leftover before retrying that path.

Only apply trusted graph plans generated by this tool: a plan contains graph
write queries. Reusing the same delivery checkpoint skips acknowledged batches.
If the destination database was reset or its graph deleted, use a fresh
checkpoint to rebuild it. A delivery checkpoint cannot detect every external
mutation of the graph.

Every node has a snapshot-specific key; historical snapshots coexist. Resource
nodes use `kind`, and relationships use a stable `RELATES_TO` type plus `kind`.
Referenced targets that were not collected become `unresolved=true`
placeholders. This is a portable architecture graph, not Cartography's complete
Neo4j label/relationship schema.

```cypher
MATCH (a:Resource)-[e:RELATES_TO]->(b:Resource)
WHERE a.scan_id = 'YOUR_SCAN_ID' AND e.kind = 'IN_VPC'
RETURN a.kind, a.id, b.id;
```

## Inspect and extend

```bash
cargo run --locked -- catalog
cargo run --locked -- sdk-catalog --service ec2
cargo check -p inventory-sdk-catalog --features sts
```

Start with [architecture](docs/ARCHITECTURE.md), [storage](docs/STORAGE.md),
[collectors](docs/COLLECTORS.md), [scheduler](docs/SCHEDULER.md), and
[SDK catalog](docs/SDK_CATALOG.md). [The expansion plan](docs/ROADMAP.md)
separates breadth work, production hardening, and performance validation.
The [collection plan](docs/COLLECTION_PLAN.md) specifies the shared 421-type
adapter, schema evidence, architecture relationships, and query generation.
[Parent collection design](docs/PARENT_COLLECTIONS.md) explains prerequisite
discovery, exact resource models, ownership checks and bounded child traversal.
The [coverage matrix](docs/CLOUDCONTROL_COVERAGE.md) explains the full registry
partition, excluded types, relationship inference and observed emulator evidence.

## CMDB product direction

The current tool is a local inventory CLI with SQLite, a process-local scheduler,
persistent jobs and a replayable FalkorDB projection. The development tree now
also contains the first continuous-CMDB slice: freshness views and an
account/region/collector coverage registry, a deterministic workload generator
with an independent oracle (`cmdb-bench`), a bounded read model (`cmdb-view`), a
read-only loopback query API (`cmdb-api`), and a Makepad viewer (`cmdb-ui`) with
a filterable table and a real account→region→kind→resource DAG. Missing pieces
are explicit: server-side auth and a write API, event ingestion,
tombstones/retention, per-collector enumeration epochs, distributed quota
coordination, PostgreSQL evaluation, durable fault/soak campaigns and live AWS
calibration. The [performance contract](docs/CMDB_PERFORMANCE_CONTRACT.md) defines
the target; the [CMDB architecture](docs/CMDB_ARCHITECTURE.md) records component
status and the [release baseline](docs/CMDB_BASELINE_2026-11-13.md) records
preliminary local storage and mock-scheduler measurements only. It does not
establish fleet capacity or competitive superiority.

Upstream projects informed the design; this is an independent implementation.
See [sources and licensing](docs/SOURCES.md). No upstream documentation prose,
logos, or complete Steampipe schemas are bundled as a claimed port.

## License

MIT. See [LICENSE](LICENSE). The project license was deliberately changed from
Apache-2.0 to MIT for this development tree by the project owner; third-party
dependencies keep their own licenses: the official
AWS SDK for Rust crates are Apache-2.0, SQLx and FalkorDB are covered in
[sources and licensing](docs/SOURCES.md).
