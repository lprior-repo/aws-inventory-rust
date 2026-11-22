# SQLx inventory storage

> Status: Kept and extended. SQLite in WAL mode with one writer and atomic,
> idempotent page commits is kept (V05); schema 13, its JSON property blobs,
> observation history and upgrade path are baseline: ROADMAP M3 replaces them
> with binary records and typed columns in a generated schema 14 created fresh
> (V24), hash reconciliation (M3.4) and no history (V22), and M9.5 deletes SQL
> files 001 to 013. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

`inventory-engine::store::Store` uses SQLx 0.9 with the SQLite and Tokio runtime
features. Ingestion, migration, checkpoints, reporting queries and export streams
all use SQLx. There is no rusqlite backend or nominal SQLx wrapper around another
database library. SQLx runs SQLite calls on its connection worker threads.
The lockfile resolves libsqlite3-sys 0.39.1 with bundled SQLite 3.54.1. SQLx is
pinned to upstream commit `10462fa4414c6c7f1ed1d39bb31678e3b726878c`, which adds
support for this patched SQLite family. This retains the native allocation
limits and includes the [WAL page-size security fix](https://rustsec.org/advisories/RUSTSEC-2026-0459.html).
The `sqlite_dependency` integration tests check the library actually linked to
the store and all four required compile-time limits.

## Connections and lifecycle

`Store::open(path).await` opens one writer connection, verifies the compiled SQLite
limits, and initializes or upgrades to schema version 13 in an immediate transaction. It then opens
one separate reader connection. File databases use WAL, `synchronous=FULL`, foreign
keys, a five second busy timeout, disabled trusted schema, and file backed temporary
storage. Each connection has an 8 MiB page cache target, a command queue of eight,
and a row channel of one. Pool acquisition has a five second deadline. Migrations
007–012 add freshness views, the account/coverage registry, and native
SQS/SNS/KMS/Step Functions views; all are additive and preserve existing
observations and user objects.

The reader opens a real file with SQLite's read-only flag and also enables
`PRAGMA query_only=ON`. It cannot create a missing database. Test-only `:memory:`
databases use unique named shared memory databases so both connections see the
same database; their reader uses `query_only`. A caller supplies a filesystem path
or `:memory:`. SQLite URI filenames are rejected to keep connection options under
application control.

The public API is asynchronous, takes `&self`, and keeps both pools private.
`apply_page(scan_id, page).await` takes ownership of the page. Preparation validates
and serializes it on a tracked `spawn_blocking` task, admitted by a semaphore with
one permit held through the page commit. Concurrent callers cannot accumulate prepared
pages behind the writer. A dropped caller cannot accumulate untracked preparation workers.
`close(self).await` consumes the Store, closes admission, drains preparation, and
closes both pools. Callers should finish or cancel their producers and consume or
drop export streams before closing the Store.

### Transactional upgrades

Schema version 0 initializes the original tables, architecture views and source
expansion. Version 1 receives `002_architecture.sql` before the version-3
migration; version 2 receives the version-3 migration. Versions 1–3 then receive
the additive version-4 global-service views. Versions 1–4 then receive the
version-5 Cloud Control expansion. Versions 1–5 then receive the additive
version-6 parent-type views. Versions 1–6 then receive the version-7 freshness
view rebuild. Version 7 is unchanged on reopen.
A nonzero foreign SQLite application ID is rejected before inventory migration.
This prevents the separate SQLx job queue from being treated as an inventory
file. Unsupported versions fail explicitly. One `BEGIN IMMEDIATE`
transaction contains version inspection, migrations, and `PRAGMA user_version`.
If a view name collides or another migration statement fails, the complete upgrade
rolls back. Existing observations, scope/page records, fingerprints and checkpoints
are preserved. Reopening an upgraded file is idempotent.

Version 3 rebuilds the three source-constrained tables to admit `cloud_control`
and changes stored Direct priority from 3 to 4. Resource/edge JSON, page
fingerprints, scope keys and tokens are copied unchanged. Cloud Control receives
priority 3; Config and Explorer keep 2 and 1. The migration retains all views and
triggers, plus indexes on the rebuilt tables, before recreating them. The managed
ECS-service view is replaced with its expanded definition. ECS task-definition,
Cloud Control base and 200 provider-column views are added.

Version 4 adds ten native global-service and derived views. Version 5 adds
197 source-specific Cloud Control views through
`005_cloudcontrol_expansion.sql`, bringing that version's root catalog to 397 types,
76 service families and 4,104 selected provider columns. Version 6 adds 21
parent-dependent views through `006_cloudcontrol_parents.sql`; version 13 adds
the final three parent views through `013_cloudcontrol_parent_additions.sql`.
The combined catalog is **421 types and 4,309 selected provider columns**, still
across 76 families. These additive migrations do not rebuild
observations or replace the original 200 provider-column definitions. The
frozen v0.3/v0.4 type list and migration, and the 397-type v0.5 root catalog,
remain available for compatibility checks. A caller-owned object occupying an added view name causes rollback;
the migration does not silently replace it or mark a partial upgrade complete.

Version 7 rebuilds only the `latest_resources` and `latest_edges` views; no
table or observation row is copied. Both views select the greatest `observed_at`
first, use `source_priority` only to break equal timestamps, and then order by
descending `scan_id`, ascending `scope_key` and descending `page_number`.
`scan_resources`, `scan_edges` and `latest_resource_sources` keep their previous
priority-first or per-source definitions, so snapshot exports and provenance
audits do not change. The rebuild shares the immediate migration transaction, so
a failure restores the version-6 views and version number.

The private initialization connection disables foreign keys before beginning
the transaction, follows SQLite's create/copy/drop/rename procedure, runs
`foreign_key_check` before commit, and re-enables enforcement before publishing
the Store. No writable-schema edits or legacy ALTER behavior are used. Schema
capture is bounded to 1,024 objects, 4 MiB aggregate SQL and 64 KiB per definition.
Unknown added/removed/reordered columns on the three managed tables are rejected
before any rebuild, avoiding silent loss of application-added values. Custom
views, indexes, triggers and separate tables are supported. An error closes the unpublished writer pool; no connection with disabled foreign
keys becomes available to an inventory caller. The documented procedure is at
[SQLite ALTER TABLE](https://sqlite.org/lang_altertable.html#otheralter).

Every Store open performs this version check. A query, report or graph export can
therefore upgrade a supported version-1 through version-5 file even though its later data reads use a separate
read-only connection. Migration tests exercise preserved data/checkpoints,
reopening, collision rollback and rejection of newer versions.

## Atomic pages and replay

A page transaction contains all of the following:

1. The collection scope and page identity.
2. Every resource observation, retaining its source and observation timestamp.
3. Every relationship observation, including unresolved targets.
4. The next page token and completion state.

Resource and edge insert statements contain at most 128 rows. Singleton chunks
retain static prepared SQL; chunks of 2–128 rows use SQLx `QueryBuilder` with
bound values. A resource batch has at most 1,536 parameters;
an edge batch has at most 1,280. Empty collections issue no insert statement.
These SQL batches all remain inside the original collection-page transaction;
a failure in any later batch rolls back earlier batches and the checkpoint.

The transaction commits only after every record and the checkpoint have been
written. A duplicate resource or edge identity within a page causes the entire
transaction to roll back. `BEGIN IMMEDIATE` serializes writers before validating
the persisted page sequence.

Page numbers start at one and must be contiguous within a scope. A nonempty next
token means the scope remains incomplete. `None` marks the terminal page, including
a successful empty scope. Repeated continuation tokens are rejected. An accepted
terminal scope cannot receive another new page.

The identity `(scan_id, scope_key, page_number)` has a SHA-256 fingerprint of the
validated, serialized page. Replaying the exact page is a no-op, including after
scan finalization. Reusing its identity with different content returns a conflict.
The scope key is structured JSON, so separators in account, collector, or parent
identifiers cannot collide. All persisted values use SQL bind parameters.

Parent-dependent Cloud Control list scopes include a digest of the canonical
request model; child read scopes additionally include the provider identifier
digest. Different parents therefore have separate page sequences and tokens.
A separate `cloud_control_parent_discovery` checkpoint records prerequisite
completeness. Failed prerequisite discovery cannot masquerade as an empty
successful child scan, and a later child list/read failure does not erase
successful siblings. The adapter also rejects duplicate parent models before
another list can restart the same scope. See [PARENT_COLLECTIONS.md](PARENT_COLLECTIONS.md).

Cancellation before commit drops the SQLx transaction and queues its rollback.
Cancellation racing with a successful commit may leave a committed page whose
acknowledgement was not observed. The caller can safely replay the same page to
resolve that ambiguity. No claim of exactly-once delivery across a process failure
is required for this protocol.

## Coverage and scan outcomes

`record_failure` persists category and message alongside the scope and marks it
failed. Duplicate identical failures are idempotent. Failure state remains visible
even if a later page terminates the scope.

`finish_scan(..., Complete, ...)` changes the effective outcome to `Partial` when
any recorded scope is incomplete or failed, or when no scopes were recorded. A
successful empty scope therefore requires a terminal empty page. `Cancelled` and
`Partial` remain explicit outcomes. A finished scan cannot be reopened, extended,
or finalized differently. Identical finalization is idempotent.

Coverage is only as complete as the scopes the collector emits. The store cannot
infer the existence of a region or service that the collector never reports.
`scan_coverage` exposes scope state, source, scan outcome and scan timestamps;
`collection_failures` retains detailed failures. A resource account or region may
differ from its collection scope, for example an AWS Config aggregate or a
RAM-shared VPC. Resource and edge source must match their scope's source.

## Provenance and selection

Every observation remains in `resource_observations` or `edge_observations`.
Current selection (`latest_resources`, `latest_edges` and the service views built
on them) is freshness-first: the greatest `observed_at` wins, regardless of
source. `source_priority` breaks only equal timestamps, ordered direct service
APIs (4), Cloud Control (3), AWS Config (2), Resource Explorer (1). Remaining
ties are deterministic: descending `scan_id`, ascending `scope_key`, then
descending `page_number`; arrival order never decides. Authority is therefore
not absolute, and a newer AWS Config observation outranks an older direct
observation.

Snapshot selection (`scan_resources`, `scan_edges`) keeps the original
priority-first order so per-scan summaries and exports remain reproducible:
`source_priority DESC, observed_at DESC, scope_key, page_number DESC`. Use those
views when a snapshot must prefer the most authoritative source.

| View | Meaning |
| --- | --- |
| `scan_resources` | One priority-first resource per identity in one scan; filter by `scan_id`. |
| `scan_edges` | One priority-first relationship per identity in one scan. |
| `latest_resource_sources` | The newest observation of each resource from each source. |
| `latest_resources` | The freshest observation of each resource across scans; priority breaks equal timestamps. |
| `latest_edges` | The freshest observation of each relationship across scans; priority breaks equal timestamps. |
| `scan_coverage` | Completed, failed or incomplete collection scopes with scan outcome. |

`latest_resource_sources` is unchanged and retains every source's newest
observation, so source comparison and timestamp audits remain possible. The
selected `source`, `observed_at` and `scan_id` remain visible, so provenance is
never lost to the freshness choice. Query `latest_resource_sources` when
comparing sources or timestamps.

An absent observation never creates a deletion, even after a nominally complete
scan. Partial permissions, eventual consistency and changed scope selection make
absence insufficient evidence. Historical resources and edges therefore remain
in the latest views until an explicit deletion/retention policy is implemented.
There is currently no automatic tombstone or retention job.

## SQL interface and limits

`query(sql, row_limit).await` returns a JSON array of row objects. It accepts one
`SELECT` or `WITH` statement, with optional leading/trailing comments and one
trailing semicolon. A small lexer understands quoted strings and identifiers,
line comments and block comments before enforcing the single statement rule.
SQLx's `AssertSqlSafe` marks this intentionally user-authored SQL path after
validation. Migration also uses it for quoted schema-object identifiers and
bounded CREATE definitions already persisted in SQLite. Other internal values
are SQL bind parameters; generated view names and JSON paths are validated
before the static migration file is produced.

A `WITH` statement containing a write still fails on the read-only/query-only
connection. Direct DDL, DML, PRAGMA, transaction control, ATTACH and VACUUM are
rejected by the prefix check. Extension loading is not enabled, and no application
functions with filesystem side effects are registered. This is a local inventory
query interface, not a multi-tenant SQL sandbox or row-level authorization system.

A native SQLite progress handler permits at most 20,000 callbacks at an interval
of approximately 1,000 VM instructions and checks a five second elapsed deadline.
Returning false interrupts the native operation. This also bounds native work
when the async query future is dropped. Every reader checkout removes any stale
handler before reuse; a new query installs its own budget. The deadline is checked
at VM progress points, not as a hard real-time preemption of arbitrary native code.

| Bound | Limit / behavior |
| --- | --- |
| User SQL bytes | 65,536 |
| Returned rows | Caller limit, from 1 through 10,000 |
| Columns | 256 |
| Returned cells | 100,000 |
| Serialized JSON result | At most 8 MiB, with conservative structural accounting |
| Page JSON | 16 MiB, including all resources and edges |
| Individual stored JSON record | 2 MiB |
| Resources / edges per page | 10,000 / 100,000 |
| Continuation token | 8,192 bytes, according to the shared core model |
| Failure message | 16,384 bytes |
| Bounded vector export | 100,000 resources or 250,000 edges, and 64 MiB serialized records |
| Resource/edge stream | 10,000,000 records per resource or edge stream, one buffered row |
| Portable inventory file | 10,000,000 total data records and 1 TiB output; lower caller budgets supported |

A row/cell/output/export bound returns an explicit error rather than a truncated
success. Duplicate output column names require aliases. BLOB results require an
explicit `hex()` projection; invalid UTF-8 and nonfinite numbers are rejected.
SQLite functions may themselves return SQL NULL on a native size limit, following
the function's semantics; the API does not reinterpret a legitimate NULL as an
application error.

`.cargo/config.toml` defines the bundled SQLite `MAX_LENGTH=8388608`,
`MAX_SQL_LENGTH=65536`, `MAX_COLUMN=256` and `MAX_EXPR_DEPTH=128` compilation limits.
Startup checks `sqlite_compileoption_used()` for these exact values and refuses a
build that omits them. These guard large native values before the Rust result
budget can inspect them. File backed temporary storage, bounded row channels and
result limits reduce memory pressure; they are not a fixed bound on the complete
process's resident memory or the database's disk growth.
The reviewed catalog exceeds `MAX_LENGTH`, so it is stored in `metadata` as a
`reviewed_catalog` head row plus ordered `reviewed_catalog.NN` continuation rows of
at most 4 MiB each; readers concatenate them and verify the catalog digest.

## SQL views

The migration provides familiar table names and selected typed configuration
columns for EC2 instances, network interfaces and EBS volumes; VPCs, subnets and
security groups; DynamoDB tables; Lambda functions; ECS clusters, services and
tasks and task definitions; and EKS clusters and node groups. Each includes the common identity,
source, timestamp, JSON properties and tags. `aws_resource` covers all normalized
resource kinds. These names are convenient for SQL users; the schema is a subset
and does not implement Steampipe's complete schema or PostgreSQL dialect.

Version 2 adds six network views (route tables, internet/NAT gateways, endpoints,
ACLs and peering) and five ELBv2 views (load balancers, listeners, rules, target
groups and target-health memberships). Three `aws_ec2_*_load_balancer` aliases
filter application, network and gateway load balancers. These aliases and the
derived `aws_vpc_route` view do not increase the count of directly collected kinds.

Version 3 adds `aws_ecs_task_definition`, target-group columns on
`aws_ecs_service`, and `aws_cloudcontrol_resource`. The 200 `aws_cc_*` views
select Cloud Control observations from `latest_resource_sources`, exposing
their top-level provider fields through SQLite JSON extraction. Provider fields
that collide with common columns use explicit catalog aliases, such as
`provider_tags` for `/Tags` and `configuration_properties` for `/Properties`.
Arrays and objects remain JSON; absent or redacted fields yield SQL NULL.
Source-specific views do not silently replace Cloud Control data with a direct
collector's different schema.
Version 4 adds the S3, IAM, Route 53 and CloudFront native views and derived
origin, role-membership and zone-VPC rows. Version 5 adds 197 further `aws_cc_*`
root views. Version 6 adds the first 21 parent-type views and 169 selected columns; version 13 adds the final three parent-type views and 36 columns. All
**421 current provider views** retain source-specific data, with **4,309 selected
top-level provider columns** in total. The Rust `resource_views` example
regenerates migrations 003, 005, 006 and 013 from the separate catalog selections;
`--check` verifies the exact catalog-to-SQL result. The parent view definitions
retain the common identity/source fields and sanitized provider values just as
the root views do. Parent provenance and the model used to list the child remain
in the resource's normalized JSON properties; the corresponding `CHILD_OF` edge
is persisted through the ordinary transactional edge path.

`scan_vpc_routes` expands route arrays from the selected route-table observation
inside each scan. It preserves IPv4/IPv6/prefix-list destinations, all mapped target
fields, route state/origin and raw route JSON. `aws_vpc_route` selects the preferred
last-known route-table observation across scans; it must not be confused with a
fresh complete snapshot.

`scan_architecture_edges` joins both endpoints on scan ID and resource identity.
Its `source_observed` and `target_observed` flags distinguish reference-only
endpoints from collected nodes. An older scan cannot supply a missing endpoint.
The CLI `architecture` command reads these snapshot views and exposes an explicit
per-panel truncation flag; underlying SQL errors and resource bounds remain errors.

## Streaming, portable export and graph checkpoints

`resource_stream(scan_id)` and `edge_stream(scan_id)` return boxed Send streams of
owned model records. Ordering is respectively resource ID and
`(from_id, to_id, relation)`. A live stream holds the single reader connection;
consume streams sequentially and avoid nested Store queries from a stream
consumer. Competing reads wait up to the pool acquisition deadline. The separate
writer can continue during a file-backed WAL read. They avoid loading the inventory into a vector and
allow an asynchronous consumer to await file or network output. `visit_resources`
is a convenience wrapper for lightweight synchronous callbacks.
Large or blocking consumer work belongs in the consumer's own async/blocking shell.

`resources` and `edges` are bounded vector conveniences for small fixtures and
reports. Large exports should use the streams. Per-record JSON decoding remains a
bounded synchronous operation in the stream consumer. Decoded export and query
rows consume Tokio's cooperative task budget so an always-ready producer does not
allow a ready-only consumer to monopolize a runtime worker indefinitely. This is
scheduling containment, not a measured latency guarantee.

The `inventory-engine::export` module writes complete finalized snapshots as
versioned JSON or JSONL without using the interactive query API's 10,000-row
limit. Normalized mode uses the preferred resource/edge streams. All-observations
mode streams raw observation rows and includes their original collection scope
and page number. Both include coverage, failures, source provenance, scan status
and accurate emitted counts. They retain structured properties/tags rather than
encoding those values as strings. Complete snapshots are required by default;
finalized partial/cancelled snapshots require explicit opt-in, and running
snapshots are always rejected.

The exporter writes one bounded record at a time in deterministic order, hashes
every output byte, drains its owned writer on cancellation, and publishes by
atomic no-clobber installation. Its default/maximum total budget is ten million
data records and 1 TiB; it never returns a truncated success. A focused
10,001-resource regression exercises the path beyond the SQL query row ceiling.
This portable artifact has identity `aws-inventory.snapshot`, version 1. It is
distinct from collection-event ingest input and executable graph-plan JSONL,
and it omits opaque cursor/page fingerprints, jobs and graph ACKs. It is not a
complete database backup. See [EXPORT.md](EXPORT.md) for the exact format,
cleanup/publication boundary and current verification evidence.

Graph checkpoints are keyed by `(scan_id, target, snapshot_id)`, require a finalized
scan, and store resource/edge counts plus completion. Counts and completion may
only move forwards. Identical checkpoint replay is safe. Unresolved graph targets
may add placeholders, so graph counts need not equal observed inventory counts.
The graph writer must use idempotent writes before advancing this checkpoint;
SQLite and a remote graph do not share one distributed transaction.

Actual graph delivery uses a separate SQLx projection log. It binds the complete
validated plan to destination/graph/scan and commits each batch ACK only after
server confirmation. Bounded retries replay only recognized idempotent generated
queries; permanent failures and exhausted attempts preserve committed progress
for an explicit later resume. The portable export does not itself contact
FalkorDB. `scan --output-dir ... --falkor` and `deliver` coordinate both stages
and preserve completed files even when graph delivery fails. See
[GRAPH_DELIVERY.md](GRAPH_DELIVERY.md) and [DELIVERY.md](DELIVERY.md).

## Verification

The store integration tests cover page replay and conflicting replay; all-or-nothing
rollback; pagination and token cycles; source precedence and deterministic ties;
partial scan retention; coverage outcomes; bound parameters containing SQL quotes;
query mutation/multiple-statement denial; native query interruption and reuse;
file reopen and WAL persistence; streaming callback failure; and monotonic graph
checkpoints. A unit test cancels ingestion after SQLite's update hook observes the
first uncommitted resource and verifies that the entire page and checkpoint roll
back before a fresh page can succeed.

```sh
cargo test -p inventory-engine --test store
cargo test -p inventory-engine --lib store::cancellation_tests
```

## Measured performance and its limits

The release benchmark ingests 10,000 synthetic resources and 10,000 edges per
case, with one or 100 resources per page transaction. Database files use the
workspace overlay filesystem, WAL and FULL synchronization. The timed window
includes fixture creation, preparation, ingestion, finalization and final counts;
connection setup and shutdown are outside that window.

The first comparison suggested a large gain for 100-record pages. The paired
repeat did not reproduce it: the original binary measured 6,371 resource rows/s
and the final binary 4,999 resource rows/s for that case, with one edge written
per resource. Singleton pages measured 468 versus 440 resource rows/s. These are
observations from this environment, not a stable throughput improvement. The
root AWS build was compiling the feature-unified SDK graph during the paired
runs, contributing CPU and I/O pressure on the shared host. This was not an
isolated benchmark.

The bounded batch design is retained because it reduces insert command handoffs
while preserving the tested transaction contract. Its real throughput benefit
has not been established by these variable measurements. No AWS throughput,
latency percentile, or fixed RSS claim follows from this benchmark.

All four runs are retained under `artifacts/`, summarized in
`benchmark-comparison.json` and `VERIFICATION.md`. `benchmark-environment.json`
records compiler/host context, source hashes and distinct binary identities.
The baseline was an ordinary byte copy with a separate inode, not a symbolic or
hard link that could be overwritten by the final build. `store-batching.patch`
records the source difference so the baseline can be reconstructed in an
isolated project copy without shipping a temporary executable.

## Primary API references

- [SQLx 0.9 SQLite connection options](https://docs.rs/sqlx/0.9.0/sqlx/sqlite/struct.SqliteConnectOptions.html)
- [Safe native progress handler API](https://docs.rs/sqlx/0.9.0/sqlx/sqlite/struct.LockedSqliteHandle.html#method.set_progress_handler)
- [Explicit dynamic SQL assertion](https://docs.rs/sqlx-core/0.9.0/sqlx_core/sql_str/struct.AssertSqlSafe.html)
- [SQLite limits and their meaning](https://sqlite.org/limits.html)
- [SQLite read-only and memory connection flags](https://sqlite.org/c3ref/open.html)
