# Implementation contract

> Status: Baseline being replaced. This is the implementation contract of the
> baseline collector (typed SDK adapters, Cloud Control, JSON properties,
> SQLite schema 13); it binds changes to that code until the roadmap deletes it
> and is not the contract for new work. See [VISION](docs/VISION.md) and
> [ROADMAP](docs/ROADMAP.md).

New work follows VISION section 4 (design rules) and section 10 (how work
lands), and the "Rules for every package" in the roadmap. Where they differ
from this file they win: for example no JSON on the data path, no new typed SDK
service crates, and services described by data instead of adapters. The first
paragraph below, no `unsafe` and the listed lint bans, holds for both.

All application Rust forbids unsafe; no unwrap/expect/panic/todo/unimplemented,
unchecked indexing, lossy as casts, or silently ignored fallible results.

## Workspace

- `inventory-core`: synchronous model, identity, validation only.
- `inventory-engine`: rate/retry executor, SQLite store, graph projection.
- `inventory-aws`: official AWS Rust SDK collectors, no SDK nested retries.
- `inventory-sdk-catalog`, `sdk-operations`: all pinned official service SDK
  dependencies, provenance and the operation inventory. Deleted in ROADMAP M1.7.
- `inventory-harness`: emulator installation, seeding, assertions and audits;
  never linked into the collector.
- `catalog-tool`: catalog maintenance and source packaging.
- `cmdb-bench`, `cmdb-view`, `cmdb-ui`: workload generator and oracle, read
  model over one JSON export, Makepad viewer over a synthetic fixture.
- root binary `aws-inventory`: CLI, fixtures, reports and composition.

## Shared model (`inventory_core`)

`Source` is `Direct`, `CloudControl`, `AwsConfig`, or `ResourceExplorer`.
`Resource` fields: `id: String` (canonical ARN when available), `account_id: String`,
`region: String`, `kind: String` (Steampipe-style singular table name),
`name: Option<String>`, `properties: serde_json::Value`,
`tags: BTreeMap<String,String>`, `source: Source`, `observed_at: i64` (epoch ms).
`Edge` fields: `from_id: String`, `to_id: String`, `relation: String`,
`source: Source`, `observed_at: i64`, `properties: serde_json::Value`.
`CollectionScope` fields: `account_id`, `region`, `collector`, `parent_id: Option<String>`, `source`.
`CollectionPage` fields: `scope`, `page_number: u32`, `next_token: Option<String>`,
`resources: Vec<Resource>`, `edges: Vec<Edge>`.
`CollectionFailure` fields: `scope`, `category: String`, `message: String`.
`ScanStatus`: `Running`, `Complete`, `Partial`, `Cancelled`.
`ScanSummary`: scan_id String, resources u64, edges u64, completed_scopes u64, failed_scopes u64.

IDs are actual full ARNs where possible. If a target is not observed, graph edges
may point to an explicitly unresolved placeholder; no invented configuration.

## Rate executor (`inventory_engine::limiter`)

`ApiError` variants include Throttled(String), Transient(String), Denied(String),
Permanent(String), Timeout, Cancelled, Exhausted(String), InvalidConfig(String),
RetryAfter { kind:RetryKind, message:String, retry_after:RetryAfter },
and RetryAfterRejected(String).
Expose `ApiExecutor::new(config: ExecutorConfig) -> Result<Self,ApiError>`.
Expose `async execute<T,F,Fut>(&self, bucket: &str, operation: F) -> Result<T,ApiError>`
where `F: FnMut() -> Fut`, `Fut: Future<Output=Result<T,ApiError>>`.
Expose `cancel(&self)`, `is_cancelled(&self) -> bool`, `async cancelled(&self)`,
and `stats(&self) -> ExecutorStats`.
`with_bucket_policies(config, BTreeMap<String,BucketLimits>)` installs immutable
exact-key overrides. `BucketLimits` contains RPS and concurrency ceilings.
`with_rate_policies(config, BTreeMap<String,BucketPolicy>)` accepts either that
integer shape or `PeriodLimits { requests:u32, period_ms:u64, concurrency:usize }`.
Starts use conservative integer-nanosecond spacing; period policies do not imply
burst credits. Valid server hints extend a monotonic shared bucket deadline.
Invalid/excessive hints close that bucket and wake its waiters; cancellation
and completion release owned permits and active-operation guards.
`ExecutorConfig`: global_concurrency usize, per_bucket_concurrency usize,
requests_per_second u32, max_attempts u32, attempt_timeout_ms u64,
max_backoff_ms u64. Validate against hard bounds.
Bucket key groups account+region+service quota family, NOT every operation by default.
The rate budget gates EVERY attempt. Never hold global permits while rate waiting/backoff.
SDK operation retry count must be 1. Include deterministic time tests and bounded cleanup.

## Store (`inventory_engine::store`)

Async `Store` owns SQLx SQLite writer and reader pools, each with one connection.
All the following operations are async and return the indicated result:

- `open(path: impl AsRef<Path>) -> Result<Self,StoreError>`.
- `begin_scan(&self, scan_id: &str, started_at: i64) -> Result<(),StoreError>`.
- `apply_page(&self, scan_id: &str, page: CollectionPage) -> Result<(),StoreError>`.
- `record_failure(&self, scan_id: &str, failure: &CollectionFailure) -> Result<(),StoreError>`.
- `finish_scan(&self, scan_id: &str, status: ScanStatus, finished_at:i64) -> Result<ScanSummary,StoreError>`.
- `query(&self, sql:&str, row_limit:usize) -> Result<serde_json::Value,StoreError>`.
- `resources(&self, scan_id:&str) -> Result<Vec<Resource>,StoreError>` (bounded).
- `edges(&self, scan_id:&str) -> Result<Vec<Edge>,StoreError>` (bounded).
- `summary(&self, scan_id:&str) -> Result<ScanSummary,StoreError>`.
- `scan_status(&self, scan_id:&str) -> Result<ScanStatus,StoreError>`.
- `checkpoint(&self, scan_id:&str, scope:&CollectionScope) -> Result<Option<Checkpoint>,StoreError>`.
- `close(self)` consumes the store and drains tracked preparation and pools.

`resource_stream` and `edge_stream` return borrowed boxed Send streams of typed
results in deterministic order. Graph projection uses these instead of complete
snapshot vectors. `visit_resources` is an async visitor convenience.
Graph export checkpoint read/write methods are also async.

The page is owned so bounded preparation can move into a tracked worker without
duplicating its payload. The preparation permit remains held through its write
transaction. SQL query execution uses a physical read-only/query-only reader,
single-statement validation and native/result bounds.
Queries that hit SQLite's native `SQLITE_TOOBIG` bound return
`StoreError::Limit("SQLite value bytes")`.
Page data, graph edges and checkpoint commit in ONE transaction. Replays idempotent.
Provenance per source retained. Partial scans never infer deletion.
WAL; synchronous FULL; busy timeout; foreign keys; prepared statements; SQL views.
Schema initialization/upgrades run within one `BEGIN IMMEDIATE` transaction.
Older supported versions upgrade additively to the current version, 13
(`inventory_engine::store::SCHEMA_VERSION`); unsupported newer versions fail.
Do not claim full Steampipe SQL/schema compatibility, merely familiar names.

## AWS adapter (`inventory_aws`)

`AwsScanOptions { profile:Option<String>, regions:Vec<String>,
services:Vec<String>, config_aggregator:Option<String>, resource_explorer_view:Option<String>,
max_pages:u32, page_size:i32 }`.
`async scan(options: AwsScanOptions, executor: Arc<ApiExecutor>,
sender: tokio::sync::mpsc::Sender<CollectionEvent>) -> Result<(),AwsScanError>`.
`CollectionEvent` in core: `Page(CollectionPage)` or `Failure(CollectionFailure)`.
Cover named service resource types with official SDKs. Shared credentials/client pool.
Regions can be explicit or discovered when empty. STS identity derives actual account.
AWS source objects aren't generally serde Serialize; map selected typed fields explicitly.
Native list pagination bounded by max_pages <= hard ceiling; reject repeated token loops;
continue empty pages with token; errors emit explicit coverage failure. No fake resources.
Config/Explorer are optional read-only sources; never auto-enable paid services.
Native selections: ec2 (including network), network, dynamodb, lambda, ecs,
ecs-task-definitions, eks, elbv2, s3, iam, route53, cloudfront, sqs, sns, kms,
step-functions. Thirty-nine native resource/component kinds are described by
`catalog`.
ELBv2 target-health membership is distinct from the referenced target resource.
All ELBv2 operations share the `elasticloadbalancing:read` quota family.

## Architecture report

The CLI `architecture` command reads a finalized scan and preserves its actual
complete/partial/cancelled status. Eight panels use exact snapshot identities,
at most 1,000 rows per panel, and a limit-plus-one probe for explicit truncation.
The SQLite query byte/work bounds remain in force. Route JSON and target fields
are retained; no public access or effective reachability is inferred.

## Graph boundary

`graph::export_snapshot(&Store, scan_id, graph, output).await` reads the persisted
status, rejects running scans, and creates a bounded plan without replacing an
existing destination. The CLI requires explicit partial export.

`graph::push_file(input, checkpoint, url).await` validates the complete plan,
applies idempotent batches and records SQLx acknowledgements only after server
success. The checkpoint identity includes destination and batch-content digests.
The projection is rebuildable from SQLite; it is not a cross-database transaction.
