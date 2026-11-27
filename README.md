# AWS Inventory in Rust

> Status: Baseline being replaced. The opening states where the product is
> going; every section after it documents the baseline collector as it works
> today. The plan for what replaces it is in [docs/e2e/](docs/e2e/README.md).

This project is becoming a CMDB of every AWS resource in the accounts it is
pointed at. It will read AWS through the services' own endpoint APIs with our
own wire engine, driven by the pinned official AWS API models, and keep the
inventory current by repeated fast sweeps. Resources are stored as binary
records in SQLite, projected into FalkorDB for graph traversal, and shown as a
DAG in a native Makepad application that reads both databases in process. The
first version runs on one laptop with about 10 profiles, holds current state
without history, and derives owner, team and application from tags.
The plan, build steps and decisions are in [docs/e2e/](docs/e2e/README.md).

**What works today** is the baseline collector that the plan replaces. The
`catalog` and `coverage` commands report 39 native resource kinds collected by
hand-written typed SDK adapters behind 16 service selectors, 421 reviewed Cloud
Control resource types (431 distinct kinds together), 968 property relationship
rules, and SQLite schema 13. The baseline also has a FalkorDB snapshot
projection, a rate limiter that is the single retry owner, fenced durable scan
jobs, a `serve` mode with a cadence scheduler, an emulator harness
(`inventory-harness`), a workload generator (`cmdb-bench`), a bounded export
read model (`cmdb-view`) and a Makepad viewer (`cmdb-ui`) that renders a
synthetic fixture. Cloud Control collection is not the way forward: it gets no
new work and is deleted in ROADMAP M9.5 (decisions V01, V04).

The new architecture has landed its foundations: `aws-wire` (SigV4 signing, the
pooled HTTP transport, and request and response handling) and the binary record
module in `inventory-core`. Not built yet: the universal `sweep` command and its
planner (ROADMAP M3.6), the binary record store with single-writer commits (M3.2
to M3.4), and in-process SQLite and FalkorDB reads for the UI (M6.2 and M6.3).

All product code is Rust. SQLx's bundled SQLite engine is native SQLite C and
FalkorDB is an external database. Legacy Python development tooling has been
replaced by the Rust `inventory-harness` CLI; it provides fixture seeding,
request recording, installation, the mutation gate, and catalog generation.

**The sections below document the baseline.** For the local emulator stack run
`scripts/local-stack.sh help`; for the baseline build-and-verify handoff see
[START_HERE.md](START_HERE.md).

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
  all` combines the native adapters with universal listing in one scan.
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
command describes the implemented
resource fields and relationships. `complete` means all attempted, selected
scopes completed; it does not mean every AWS service was scanned.

Config and Resource Explorer use the same canonical kinds for the supported
network and ELBv2 resources. Their detail remains source-dependent: Explorer
provides discovery metadata, Config retains raw configuration with selected
scalar aliases, and neither currently supplies native-equivalent relationships.
Config's raw nested route arrays are retained but are not expanded by the
normalized routes panel.

## Test the actual AWS calls locally

AWS Emulator testing describes the pinned full source
checkout, verified AWS binary, populated service fixtures and operation-by-operation
evidence. The call matrix enumerates **89 explicit SDK
operations in 19 services** (79 in the frozen matrix plus 10 reviewed KMS and
Step Functions additions; 2 of the 89 are Cloud Control) **plus the SDK's
AssumeRole boundary**, separately from the 421 Cloud Control resource types and
437 available SDK crates.
The current source/run reconciliation
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

Property tests exercise generated retry histories,
pagination, resource identities, SQLx transactions, durable job leases and the
actual CLI. Guarded integration tests compare the compiled collectors and CLI
with independently checked AWS Emulator snapshots and a real local FalkorDB.

The mutation gate requires a complete, frozen inventory
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
sets the retry configuration's explicit origin as well as its value. Normal CLI configuration
loaders already supplied that origin.

## Reviewed describe skips

`inventory-harness probe` accepts repeatable
`--compare-pair service/Parent/Child` selectors with `--compared-at <review date>`,
alongside its catalog, loopback HTTPS endpoint, CA, region and report options.
`--service` cannot be combined with `--compare-pair`.
It compares every selected list page and bound describe response, retaining
absent/null/value distinctions. A describe-only modeled member, any value
difference, or zero seeded resources produces no skip.

The TSV result is written to `--report`. The probe first publishes a TOML
fragment to `<report>.skip.toml` and prints it. The fragment groups operation
tables by service and records `skip`, the service and list/describe pair, a
SHA-256 fingerprint of the comparison inputs, compared selectors, resource
count and review date. Existing receipts and symlinked output paths are
refused without overwriting the receipt or writing a report. Review and merge
the operation tables for each service into its existing reviewed policy,
preserving all other fields. The probe never edits catalog sources or enables
skips automatically.

Overlay checking rejects receipts for the wrong pair. Catalog compilation also
rejects stale fingerprints and skips that would lose child emissions or
discovery work. An additional parent emission can preserve a child kind if it
comes from the compared rows and preserves the child's metadata.

## Build

Install Rust 1.100 stable with rustup (`rustup toolchain install 1.100.0`) and a
C toolchain for bundled SQLite. Run commands from the repository root so
`.cargo/config.toml` applies the SQLite allocation bounds. Dependencies are
locked.

Supported targets are Linux x86_64, Linux aarch64 and macOS aarch64 (Apple
silicon). `inventory-harness install` selects the emulator asset for the host
and returns an unsupported-platform error for any other target. The FalkorDB
module has no macOS build upstream, so `install-graph` reports the same error on
macOS; SQLite-only work does not need it.

The pinned AWS API models are the `.local/aws-models` git submodule. Fetch it
before compiling the catalog:

```bash
git submodule update --init --depth 1 .local/aws-models
scripts/bootstrap-catalog.sh
```

`scripts/bootstrap-catalog.sh` regenerates `catalog/aws-api.catalog` and the
other generated catalog outputs from those models and verifies them against
`MANIFEST.sha256`. They are not checked in; builds fail with a pointer to the
script when they are missing. Large pinned inputs are stored with Git LFS
(`git lfs install` once per clone).

```bash
# Fetch the locked workspace dependencies.
cargo fetch --locked

# Local ingestion, SQLite SQL, and FalkorDB export; no AWS clients compiled.
cargo build --locked

# Include the implemented native AWS collectors.
cargo build --locked --features aws --jobs 1
```

Selected collectors compile only the SDK crates they use. All-service SDK
inventory generation was removed in M1.7; the pinned API model catalog is
the source for service and operation metadata.

The canonical maintainer gate is Moon 2.2.4 and needs no Python:

```bash
moon ci
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

`catalog-tool iam-policy --write` generates the callable-plan action bundle
(`catalog/iam-read-only-policy.json`, an array of managed-policy documents),
`deploy/read-only-role.yaml`, and `deploy/read-only-policy-gaps.txt`; `--check`
compares all three byte for byte without writing. Each document has at most
6,144 non-whitespace characters. Generation caps attachments at the default
quota of 20 per role; AWS's adjustable maximum is 25 (verified 2026-11-24).
The generated template states its actual attachment count. With no mode flag,
stdout is only the JSON policy bundle. Writes stage all artifacts before
publishing, and attempt every rollback on a later I/O error, reporting all
restoration failures. Abrupt process/machine failure during replacements is not
a multi-file transaction.

The gap report's `shared` section uses explicit operation overrides or model
signing-name/operation defaults; non-callable operations whose IAM action is not
derivable by that lookup are not listed. DENIED (must-deny) overlaps appear first
as findings, separately from deferred operations. IAM grants actions rather than
operation IDs, so an action needed by a callable operation can also permit an
excluded operation. The derived `broad` section lists granted actions associated
with multiple operation IDs by existing aliases or callable mappings, with their
callable owners; for example, `apigateway:GET` can permit other GET requests.
These sections describe the current catalog lookup, not verified real-account
authorization coverage.

Deploy the template through S3, not `--template-body`: CloudFormation permits
only 51,200 bytes inline but 1 MB with an S3 `--template-url`
([AWS quotas](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/cloudformation-limits.html),
[CreateStack API](https://docs.aws.amazon.com/AWSCloudFormation/latest/APIReference/API_CreateStack.html),
verified 2026-11-24). Tests bound the generated template to a conservative
1,000,000 bytes, including the maximum 20-policy attachment set.
Set `AWS_REGION` and `TEMPLATE_BUCKET` for an existing private bucket in the
stack's Region. Create a mode-0600 `role-parameters.json` containing
CloudFormation `ParameterKey`/`ParameterValue` entries for your actual
`TrustedPrincipalArn` and `ExternalId` (and optionally `RoleName`).
The uploading/deploying identity must have access to the S3 object:

```bash
aws s3 cp deploy/read-only-role.yaml \
  "s3://${TEMPLATE_BUCKET}/aws-inventory/read-only-role.yaml" --region "$AWS_REGION"
aws cloudformation create-stack --stack-name aws-inventory-read-only \
  --region "$AWS_REGION" \
  --template-url "https://${TEMPLATE_BUCKET}.s3.${AWS_REGION}.amazonaws.com/aws-inventory/read-only-role.yaml" \
  --capabilities CAPABILITY_NAMED_IAM --parameters file://role-parameters.json
```

`catalog-tool iam-reference --date YYYY-MM-DD` downloads AWS's machine-readable
Service Authorization Reference into sorted `catalog/iam-actions.tsv`, for
services used by callable plans. This requires `curl` and network access;
normal policy generation and checks are offline. Exceptions to the model's
signing-name/operation-name convention are reviewed explicitly in
`catalog/overlay/iam-actions.toml`, one scalar action per callable operation.
Non-read-only reference actions fail closed, naming the operation.
Reviewed reference omissions are pinned as `unconfirmed` in
`catalog/overlay/iam-reference-exceptions.tsv`, with the signing name and
reference date. These operations receive their `<signing name>:<Operation>`
grant only if the operation also has a catalog read-verb prefix; write-named
omissions are refused. The generated gap document lists the unconfirmed count
and names. Generation refuses stale, newly published, or no-longer-callable exceptions.
Attempted actions, including unconfirmed names and aliases, must all be in
the policy. Neither reference membership nor emulator tests prove real-account
IAM minimality or dependent resource-provider permissions.

Sweep callers may set `SweepControl.attempts` to an `Arc<AttemptedOperations>`
created from the invocation's runtime catalog. After the sweep, `iter()` exposes
the distinct service/operation pairs recorded immediately before signing,
including retries and failures. Recording allocates only at table construction;
an unknown operation fails the sweep rather than disappearing from the receipt.

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
See delivery, portable export and
graph retry for formats, bounds and recovery contracts.

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

Continuous collection via `serve`:

```bash
# Serve mode: sweep profiles on an interval, idling in between.
# Runs until stopped with Ctrl+C.
cargo run --locked --features aws -- serve \
  --database inventory.sqlite \
  --profile audit \
  --interval 15m

# Multi-profile serve: sweeps each profile once per interval.
# Stalest profiles run first within a sweep; if a sweep overruns the interval,
# the next starts immediately. Sweeps never overlap.
cargo run --locked --features aws -- serve \
  --database inventory.sqlite \
  --profile development,production,staging \
  --region us-east-1 \
  --interval 30m \
  --concurrency 32
```

The `serve` command reads the catalog once, opens the store once, and then
repeats: for each configured profile, run a full sweep, then sleep until the
next interval. Within a sweep, scopes that have not been swept in the longest
time run first. The command runs until stopped with Ctrl+C; it exits with code 0
on clean stop, 2 if any sweep pass had failures.

```bash
# With custom endpoint and CA bundle
cargo run --locked --features aws -- serve \
  --database inventory.sqlite \
  --profile audit \
  --interval 10m \
  --endpoint https://emulator.example.com \
  --ca-bundle /etc/ssl/emulator-ca.pem
```

Organizations listing must complete before automatic account fan-out starts.
Inactive or missing accounts are not silently accepted by an explicit filter.
`--max-accounts` defaults to 128 and is capped at 1,024. An optional
`--external-id-env VARIABLE_NAME` reads its value only at execution; the value is
never persisted in a queue job.

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
files are separate databases.

Diff reports observation changes, including source changes. It does not infer
deletion from a missing record. Graph paths follow observed configuration
references; they do not prove network reachability or IAM authorization.

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
types share `elasticloadbalancing:read`. Other applications consume the same AWS account quotas.

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

For schema 14 PostgreSQL inventories, use the separate `sql` command:

```bash
aws-inventory sql --database "$INVENTORY_READER_URL" --limit 100 \
  "SELECT kind, count(*) FROM resources GROUP BY kind ORDER BY kind"
```

Use a dedicated reader login with `USAGE` on `inventory` and `SELECT` on
its tables. The command refuses database owners, administrators, roles with
inventory write privileges, broad built-in read roles, and logins that can
assume such roles; it does not downgrade a writer URL. Reader provisioning
is an administrative operation, not something this command performs.

Statements run on a dedicated connection in a read-only transaction with a
five-second timeout and the `inventory` search path. Success is followed by
bounded rollback and connection shutdown. Ordinary query failures disconnect
without draining pending rows. If the transport watchdog expires, a separate
authenticated reader connection terminates the timed-out backend and waits for
its exit; termination failures are reported rather than claiming cancellation.
Exact quoted and unquoted advisory-lock, backend-control, `set_config`,
notification and logical-message function names are refused. SQL-text execution
through `query_to_xml*`, `ts_stat`, `ts_rewrite`, and the documented
`tablefunc`/`dblink` entry points is also refused. Harmless columns such as
`pg_advisory_note` remain queryable.
Unicode-escaped quoted identifiers (`U&"..."`) are also refused; ordinary quoted
identifiers and Unicode string literals remain supported.
Connections require `standard_conforming_strings = on`.
Defaults cap results at 10,000 rows, 256 columns, 100,000 cells and 8 MiB of
column names and binary cell payloads; escaped table output is also capped at 8 MiB,
with every rendered cell charged before allocating its heap text.
The byte bound is checked after a cell is received; it is not a memory bound
for one very large cell.
`--limit` can only lower the row cap. Tables preserve SQL nulls, show bytes as
hex (up to 32 bytes), and format timestamps in UTC. Unsupported types must be
cast to `text`; this includes PostgreSQL `numeric`, arrays and UUIDs.
Only columns actually materialized by the catalog are available; use
`information_schema.columns` to inspect them rather than assuming a full
Steampipe schema, including `aws_ec2_vpc.cidr_block`.
The lexer is a guard, not a sandbox for arbitrary administrator-defined
functions or untrusted extensions. Reader privileges and the read-only
transaction remain necessary. The SQL-text entry points are documented in
PostgreSQL 18's [XML mapping functions](https://www.postgresql.org/docs/18/functions-xml.html#FUNCTIONS-XML-MAPPING),
[text search functions](https://www.postgresql.org/docs/18/textsearch-features.html),
[tablefunc](https://www.postgresql.org/docs/18/tablefunc.html) and
[dblink](https://www.postgresql.org/docs/18/dblink.html).

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
tables are preserved.

## Retry primitives and workflow engines

The historical SDK source audit covered 437 pinned service crates and counted
19,617 operations, 3,779 paginators and 407 waiters. Those results remain
historical evidence under ignored `artifacts/`; M1.7 removed the derived SDK
operation inventory. The pinned AWS API model catalog is the source of service
and operation metadata. Typed service adapters remain for collection and
differential proof, but are not a second metadata inventory.

This version keeps **one retry owner**: the shared executor, with each explicit
service SDK request configured for one attempt. Every retry re-enters admission.
Enabling a second five-attempt SDK loop inside the outer five-attempt loop can
produce 25 attempts and invalidate configured accounting. The audited public
SDK hooks did not provide a proven drop-in asynchronous quota wait before
signing and before the SDK's attempt timeout.

The local SQLx queue supplies durable scan submission, worker-crash recovery,
fenced ownership, cancellation and explicit retry without an additional server.
Restate and Temporal remain options for ownership and timers across multiple
machines. Neither removes quota coordination or idempotent persistence work.

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
```

The plan for what replaces the baseline is in [docs/e2e/](docs/e2e/README.md).

### Sweep benchmark

`cmdb-bench sweep --collector <binary> --profile thin|full|proxy --runs 3`
starts the harness's private emulator and graph server, seeds once, and creates a
fresh PostgreSQL database for each cold/warm pair. Set `INVENTORY_POSTGRES_URL`
to a harness administrative connection. Installed emulator/graph paths can be
set through `INVENTORY_EMULATOR_BINARY`, `INVENTORY_REDIS_SERVER`, and
`INVENTORY_FALKOR_MODULE`, as with the harness.

Use `--out <file>` for raw TSV, `--work <empty-dir>` to retain collector logs,
`--catalog <file>` to select the compiled catalog, and repeated
`--service <recipe>` to restrict an explicitly labelled smoke workload.
Record `--server container|native|managed`, `--volume named|bind|native-disk`,
and `--build release|debug`; unstated values are printed, not inferred.
The table includes actual PostgreSQL settings, measured HTTP protocol and Git
state. Failed collectors, dirty/unavailable commits, fsync-off servers and
non-Linux hosts are not publishable, with the reason recorded. A failed
collector's table is printed but never written to `--out`. WAL is server-wide:
unrelated writes can contribute. Resource and writer rates are per second,
rounded down; CPU is milliseconds and decode share is parts per million.
Zero-denominator rows are `unavailable`, not zero; comparison refuses them.

For the PostgreSQL cold-write comparison, leave `INVENTORY_POSTGRES_COPY_MIN_ROWS`
unset for the default `UNNEST` path, or export it as `1` to force staging COPY
for every nonempty changed batch. Other positive values set the minimum changed
row count; warm unchanged rows do not qualify. Use the same release collector,
catalog, server settings and three runs per profile (`thin` and `full`) for both
settings. Compare P1, P8 and P9, check P2 for a warm regression, and watch P11
for growth from the session-local tables. The separate direct-COPY measurement
requires its own writer mode; no current setting enables it.

`--profile proxy` selects thin seeds with 50 ms latency and 20 requests per second
per account/region/service. `--proxy latency_ms=50,rps=20` also works with thin/full
and overrides those defaults. Every cold and warm collection gets fresh buckets
and the same fixed fault seed. Raw 100-ms counter recordings accompany collector
logs. Proxy extra rows measure the cold phase; warm extras are discarded.
Steady throughput and ramp-up count admitted responses, excluding throttles.
Ramp-up compares aggregate admitted responses against one scope's quota, not
proof that all scopes saturated. Short runs may not reach the 30-second
throttle-share window or the quota threshold.

Collector logs and seed reports are read with a 16 MiB bound; each seed recipe
has a 1 MiB bound. SIGINT/SIGTERM cancel the collector and drain owned resources.

`cmdb-bench pipeline "$BENCH_POSTGRES_URL" <compiled-catalog> 2 3 2` runs
initial collection, a tag update, and duplicate replay, checking stored rows after
each phase. Use a disposable PostgreSQL database with no `inventory` schema.
The pipeline refuses an existing inventory rather than resuming or replacing it.

`cmdb-bench open --catalog <file>` measures catalog decode, fresh schema creation,
reopening, and insert preparation (P11). `cmdb-bench compare <before> <after>`
checks every median against the larger of 5% and twice the baseline spread.
`--band <whole-percent>` overrides this band; `--allow-different-server` allows
different profile/server, operating-system, build and protocol headers with an
explicit non-comparability warning. Compare exits 0 for no regression, 1 for a
regression and 2 for a refused comparison. Collector failures exit nonzero.


## CMDB product direction

The inventory CLI stores swept resources in SQLite or PostgreSQL and projects
their topology into FalkorDB. The bounded `cmdb-view` read model supplies
tables, details, counts, tags, freshness and edge pages directly from either
inventory database. PostgreSQL reads use pinned repeatable-read, read-only
snapshots, so pagination and details stay consistent while a sweep writes.
Resource pages keep a continuation when the 10 MiB byte budget truncates a page.
A single row larger than that budget, or a resource missing its kind mapping or
record, returns an explicit error in both backends. Search treats wildcard
characters literally and folds ASCII case only in both backends. PostgreSQL
uses the `C` collation for this search, independently of the server locale.

The Makepad viewer reads a PostgreSQL inventory database written by
`aws-inventory sweep`. It accepts only a PostgreSQL URL; a file path is rejected.
Give it a read-only role and a URL with no password; the password comes from
`PGPASSFILE` or `~/.pgpass`, which SQLx reads when it parses the URL. The viewer takes
its URL only as a flag, so that URL is visible to other local users in the process list
(`ps`). Keep the sweep writer URL in its own variable and never give it to the viewer.

```bash
cmdb-ui --database "$INVENTORY_VIEWER_URL"
```

To show the FalkorDB topology, use an operator-owned FalkorDB server on port `16379` (the
`scripts/local-stack.sh` default, not Redis's `6379`) and pass it with the graph name
the sweep wrote:

```bash
cmdb-ui --database "$INVENTORY_VIEWER_URL" \
  --graph-url redis://127.0.0.1:16379 --graph-name aws_inventory
```

The graph is optional; without it the viewer reports the graph offline. The bootstrap
privileges (`CONNECT` and `CREATE` for the first sweep, no pre-created `inventory`
schema), the runtime paths and the sweep exit codes are in
[crates/cmdb-ui/README.md](crates/cmdb-ui/README.md). The walkthrough does not start, stop
or signal the FalkorDB server. PostgreSQL URLs are
redacted in configuration diagnostics. `cmdb-bench` supplies a deterministic workload
generator and an independent oracle.

The implementation plan is in [docs/e2e/](docs/e2e/README.md). The obsolete
HTTP read API (`cmdb-api`) has been removed; the viewer uses in-process readers.

Upstream projects informed the design; this is an independent implementation.
No upstream documentation prose,
logos, or complete Steampipe schemas are bundled as a claimed port.

## License

MIT. See [LICENSE](LICENSE). The project license was deliberately changed from
Apache-2.0 to MIT for this development tree by the project owner; third-party
dependencies keep their own licenses: the official
AWS SDK for Rust crates are Apache-2.0.
