# Track C steps: SQL and the store

> Status: Plan, 2026-11-15, revised for V53 (PostgreSQL is the only store).
> Step-by-step breakdown of the packages in [C_SQL.md](../C_SQL.md). C5 is the
> store interface, C6 builds the PostgreSQL store and then deletes the old
> store code (C6.12), and C1 to C4 are built on it. Steps C5.2 and C5.4 were
> removed by V53; see "Removed by V53" in [BUILD_PLAN.md](../BUILD_PLAN.md).
> A step's own Status line, where it has one, says what exists in the tree.

Conventions: as in [STEPS_A.md](STEPS_A.md). The store is PostgreSQL in every
step.

Facts this track relies on, read from the tree and the machine:

- `sqlx` is pinned to a git revision (`Cargo.toml` line 26). `Cargo.lock` has
  no `sqlx-postgres`. No PostgreSQL code exists.
- The current store is `BinaryStore` in `crates/engine/src/inventory.rs`: a
  struct with two connection pools and about 17 async methods. Its queries are
  in `crates/engine/src/inventory/*.rs`, one statement per row. It is what
  `tests/sweep_e2e` passes on today (2 passed).
- `cmdb-view` does not depend on `inventory-engine`. It opens the database
  itself (`crates/cmdb-view/src/database.rs`) with its own queries.
- A kind table's typed columns are named `f_<field id>` today
  (`inventory_core::inventory::Column::new`), not by member name.
- This machine has the `docker` client but no daemon, and PostgreSQL 18.6
  installed natively with binaries in `/usr/bin`. `initdb`
  refuses to run as root. Defaults seen: `max_connections` 100,
  `synchronous_commit` on.

## C5 Store interface

**Why a trait.** The sweep calls the store from three files (`execution.rs`,
`execution/draining.rs`, `execution/page.rs`) through eight methods, and the
projection runner through seven functions that take `&BinaryStore`. Naming
those calls in a trait is the smallest change that lets `PgStore` be built and
tested piece by piece: `BinaryStore` implements the trait by delegation,
`PgStore` is tested against the same conformance suite, the switch is one line
where the store is constructed (C6.10), and C6.12 is then file removal. The
alternatives are one large rewrite of the sweep and the store together, or a
branch at every call site.

**The trait is kept after C6.12.** VISION 5.4 asks for "one interface with one
implementation", and the sweep's unit tests use a fake store through it (D1.3,
B3.2). Its cost is a generic parameter; there is no dynamic dispatch.

### C5.1 One trait, and a conformance suite

- **Goal:** Put the calls the sweep and the projection make on `BinaryStore`
  behind one trait, and turn the existing store tests into the suite `PgStore`
  must pass.
- **Depends on:** A0.2 (a passing gate). Otherwise nothing.
- **Files:** `crates/engine/src/inventory.rs` (280 lines),
  `crates/engine/src/inventory/store_api.rs` (new),
  `crates/engine/tests/inventory_store.rs`,
  `crates/engine/tests/inventory_store/behavior.rs` and its directory,
  `crates/engine/tests/inventory_store/support.rs`,
  `crates/engine/tests/inventory_store/conformance.rs` (new).
- **Change:**
  1. `pub trait InventoryStore: Send + Sync` with these methods, each
     returning `impl Future<Output = Result<_, StoreError>> + Send`:
     `validate_catalog`, `seed_works`, `claim`, `commit_page`, `finish_scope`,
     `resume_sweep`, `sweep_summary`, `drain_projection`, `ack_projection`,
     `projection_state`, `projection_resources`, `projection_edges`,
     `projection_resource`, `projection_edges_for`, and `fn registry(&self) ->
     &SchemaRegistry`. Signatures are exactly those of the inherent methods on
     `BinaryStore` today. `begin_projection_snapshot` becomes an associated
     type: `type Snapshot: ProjectionSnapshotRead`, with a trait
     `ProjectionSnapshotRead` (new) carrying the read methods of
     `ProjectionSnapshot` (which owns a database transaction today).
  2. `impl InventoryStore for BinaryStore` by delegation to the inherent
     methods. Nothing else in `crates/engine/src/inventory/` changes.
  3. Tests: `conformance.rs` holds `pub async fn run<S, F>(make: F)` over the
     trait with every behavior case: seed and claim, lease expiry, stale lease
     refused, page commit with observations, references resolved late,
     ambiguous reference not drawn, children seeded, continuation digest
     repeats, scope completion and deletion after two complete enumerations,
     partial deletes nothing, parent deletion removes children, projection
     outbox drain and ack, sweep summary. Move the bodies out of
     `behavior.rs`; `inventory_store.rs` calls `run` with the factory in
     `support.rs`. This one move is the proof that the suite describes
     today's behavior.
- **Tests:** the suite itself, unchanged in what it asserts.
- **Budget:** none: no behavior change.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_store`.
- **Done when:** every existing store behavior test runs through the trait.
- **Size:** about 250 added, 150 moved. One change.
- **Watch out:** native `async fn` in a trait is not `dyn`-safe and gives no
  `Send` bound; that is why the methods are written as `fn ... -> impl Future +
  Send`. Do not add the `async-trait` crate. `inventory.rs` is at 280 lines;
  the trait goes in its own file.

The SQLite implementation exposes this seam through
`inventory::InventoryStore` and `inventory::ProjectionSnapshotRead`.
`inventory_store/conformance.rs` runs backend-neutral scenarios from a fresh
factory per scenario; the SQLite factory prepares sweep manifests before
returning the store. SQLite schema assertions, injected SQL failures, reopen
checks, manifest discovery, explicit rebuild requests and unobserved-scope
reports remain backend-local. They do not expand the portable store contract.

### C5.3 The sweep and the projection use the trait

- **Goal:** No code outside `crates/engine/src/inventory/` names
  `BinaryStore` except where a store is constructed.
- **Depends on:** C5.1, D1.2.
- **Files:** `crates/sweep/src/execution.rs`, `execution/draining.rs`,
  `execution/page.rs` (every `&BinaryStore` parameter),
  `crates/engine/src/inventory/projection/runner.rs` (`run_projection` and the
  six helpers that take `&BinaryStore`), `src/sweep_command/enabled.rs`.
- **Change:** `RuntimeContext<'a, 'catalog, S: InventoryStore>` with `store:
  &'a S`; `run<S: InventoryStore>(catalog, store: &S, ...)`;
  `run_projection<S: InventoryStore>(store: &S, ...)`.
- **Tests:** existing tests compile and pass; `grep -rn "BinaryStore"
  crates/sweep/src` prints nothing.
- **Budget:** none can be recorded yet: the benchmark's first row (B0.2) comes
  after this step. The change is generic code, monomorphized; there is no
  dynamic dispatch on the page path.
- **Prove it:** `cargo test --locked --features sweep --test sweep_e2e -j1 --
  --ignored --test-threads 1` gives 2 passed; the grep.
- **Done when:** the grep is empty.
- **Size:** about 80 added, 120 changed. One change.
- **Watch out:** this touches the same files as B3 and B5.1. BUILD_PLAN orders
  them: D1.2, D1.3, C5.3, then the others one at a time.

## C6 PostgreSQL store

### C6.1 The `postgres` feature of `sqlx`

- **Goal:** Compile PostgreSQL support, and know what it costs the build.
- **Depends on:** D1.1 (both edit the root `Cargo.toml`; land D1.1 first).
- **Files:** `Cargo.toml` (line 26), `Cargo.lock`.
- **Change:** add `"postgres"` to the workspace `sqlx` features. Regenerate
  `Cargo.lock` once without `--locked` and commit it. Record, before and
  after: `cargo tree -e normal -p inventory-engine | wc -l`, the new crates by
  name, clean build time of `inventory-engine` in debug with `-j1`, and peak
  compiler memory (`/usr/bin/time -v`).
- **Tests:** `crates/engine/tests/postgres_dependency.rs` (new) asserting
  through `cargo metadata` that `sqlx-postgres` comes from the same git
  revision as `sqlx-core`.
- **Prove it:** `cargo check --workspace --all-targets --all-features --locked
  -j1` passes after the lock is committed.
- **Done when:** the workspace builds and the four numbers are recorded.
- **Size:** two lines of TOML and the lock. One change.
- **Watch out:** the pin is a git revision of a fork, not a release; whether
  its `postgres` feature builds on `nightly-2026-11-11` is not verified. There
  is no other store to fall back on, so this is checked in wave 0. If it does
  not build, stop and report; do not move the pin in this step. No TLS feature
  is added here: local servers are reached over loopback. Remote PostgreSQL
  (H4.1) needs a `sqlx` TLS feature, and its exact name at this revision is
  not verified.

### C6.2 Run PostgreSQL locally: Podman, Docker or a native server

- **Goal:** This is the step that gets PostgreSQL running on a developer's
  machine, in tests and in CI. The owner confirmed three ways: Podman, Docker,
  or a native server. All three satisfy one contract, the URL in
  `INVENTORY_POSTGRES_URL`; a CI service container is a fourth use of the
  same contract. Tests never share a database.
- **Depends on:** nothing.
- **Files:** `deploy/local/compose.yaml` (new), `deploy/local/README.md`
  (new), `crates/harness/src/postgres.rs` (new),
  `crates/harness/src/postgres/database.rs` (new),
  `crates/harness/src/postgres/runtime.rs` (new),
  `crates/harness/src/postgres/tests.rs` (new), `crates/harness/src/lib.rs`,
  `crates/harness/src/cli.rs`, `crates/harness/src/main.rs`,
  `.github/workflows/ci.yml`, `moon.yml`.
- **Change:**
  1. **Compose file.** One service `postgres` from the official image, pinned
     as `docker.io/library/postgres:18-trixie@sha256:<digest>` (PostgreSQL
     18.7; the digest is resolved and written by the worker). A
     named volume `inventory-pgdata-18` for `/var/lib/postgresql`, never a
     bind mount. Port published on loopback only: `127.0.0.1:54329:5432`.
     `shm_size: 2gb`. Database `inventory`, user `inventory`. A health check
     with `pg_isready`. Server settings passed on the command line and written
     down, because container defaults are small (`shared_buffers` defaults to
     128MB): `shared_buffers=1GB`, `effective_cache_size=3GB`, `work_mem=16MB`,
     `maintenance_work_mem=256MB`, `max_wal_size=4GB`,
     `checkpoint_timeout=15min`, `max_connections=200`,
     `max_locks_per_transaction=256`, `synchronous_commit=on`, `fsync=on`,
     `full_page_writes=on`. These are starting values; B4.4 measures the ones
     that matter and B5.2 records what the server actually reports.
  2. **URL contract.** The harness, `tests/sweep_e2e`, the engine's ignored
     tests and `cmdb-bench` read `INVENTORY_POSTGRES_URL`, an administrative
     URL to an existing server (for the compose file:
     `postgres://inventory:<password>@127.0.0.1:54329/inventory`). When it is
     unset, a test that needs PostgreSQL fails with a message naming the
     variable and the two commands in item 4; it is not skipped silently.
  3. **A database per test.** `pub struct TestDatabase { name: String, url:
     String, admin_url: String }` with `pub async fn create(admin_url: &str)
     -> Result<Self, HarnessError>`, `pub fn url(&self) -> &str`, `pub async
     fn finish(self)`. The name is `inv_t_<8 hex digits of the Unix time>_<pid
     in hex>_<8 hex digits from RandomState>`: at most 40 bytes, well under
     PostgreSQL's 63, and only `[a-z0-9_]`. `create` runs `CREATE DATABASE`;
     `finish` runs `DROP DATABASE ... WITH (FORCE)`. `Drop` does the same on a
     helper thread with its own runtime, so a failed or panicking test still
     cleans up. `pub async fn prune(admin_url, older_than_seconds)` drops any
     `inv_t_` database whose embedded time is older than the bound (default
     3,600), for runs that were killed.
  4. **Convenience commands.** `inventory-harness postgres up [--runtime
     docker|podman]`, `postgres down [--volumes]`, `postgres url`, `postgres
     prune`. `up` picks `podman` if `podman info` succeeds, else `docker` if
     `docker info` succeeds, and runs
     `<runtime> compose -f deploy/local/compose.yaml up -d --wait`, and prints
     the URL. With no working runtime it prints the native instructions of
     item 6 and exits 2.
  5. **CI.** `.github/workflows/ci.yml` (job on `ubuntu-24.04`) gains a
     `services.postgres` entry with the same image digest, the same settings,
     a health check, and `INVENTORY_POSTGRES_URL` in the job environment.
     `moon.yml` gains an opt-in task `live-postgres-component` beside
     `live-falkordb-component`, which requires the variable and runs the
     ignored PostgreSQL tests.
  6. **No container runtime.** Any server works through the same variable.
     `deploy/local/README.md` gives the native commands verified on this
     machine: as an unprivileged user (here `runuser -u postgres --`), `initdb
     -D <dir> --auth=trust -U inv`, then `pg_ctl -D <dir> -o "-c
     listen_addresses='' -c unix_socket_directories=<dir>" -w start`, and the
     URL `postgres://inv@localhost/postgres?host=<dir>`. It also lists the
     settings of item 1 as `-c` options, so a native benchmark server is
     configured like the container.
- **Tests:** ignored tests that need the variable: create, connect, finish,
  and the database is gone; two `TestDatabase`s at once have different names;
  dropping without `finish` removes the database; `prune` removes an old one
  and leaves a fresh one; name length and character set (not ignored, pure).
  Runtime detection is tested with stub executables on `PATH`.
- **Prove it:** on this machine (no daemon): start a native server as above,
  export the URL, `cargo test -p inventory-harness --locked postgres --
  --ignored`. On a machine with Docker or Podman: `inventory-harness postgres
  up` then the same test command.
- **Done when:** the tests pass against a native server here and against the
  container on a machine that has a runtime; CI has the service.
- **Size:** about 380 added Rust, about 60 lines of YAML and a README. Two
  changes: (1) compose file, URL contract and `TestDatabase`; (2) commands
  and CI.
- **Watch out:** this machine has the `docker` client but no daemon, so the
  container path cannot be proven here; say so in the report. `initdb` and
  `postgres` refuse to run as root, and this machine and most CI containers
  run as root: the native path must use an unprivileged user. Docker on macOS
  runs in a virtual machine and is slower on disk, more so with a bind mount:
  the volume is named, and published numbers come from Linux (BUILD_PLAN
  section 3). `CREATE DATABASE` needs the `CREATEDB` privilege; say so in the
  error. `DROP DATABASE ... WITH (FORCE)` needs PostgreSQL 13 or later. The
  password in the compose file is a local default bound to loopback; never
  reuse it.

### C6.3 The PostgreSQL schema generator

- **Goal:** Create schema 14 on PostgreSQL from the catalog's `KindSchema`s,
  as a new module.
- **Depends on:** C6.1, C6.2, C5.1.
- **Files:** `crates/engine/src/inventory/postgres/schema.rs` (new),
  `inventory/postgres/schema/shared.rs` (new),
  `inventory/postgres/schema/kinds.rs` (new),
  `crates/engine/src/inventory/schema.rs` (visibility of `encode_schema` and
  `decode_schema` only), `crates/engine/tests/inventory_schema14_postgres.rs`
  (new).
- **Change:**
  1. The generator is its own module; `crates/engine/src/inventory/schema.rs`
     is not split (that was C5.2, removed). It reuses `encode_schema` and
     `decode_schema` from `schema.rs` by making them `pub(super)`; C6.12 moves
     them when it deletes that file.
  2. Everything lives in the PostgreSQL schema `inventory` (`CREATE SCHEMA
     inventory`), so external clients see one namespace.
  3. Shared tables, translated from the 16 statements of
     `schema.rs::create_shared_tables`: `TEXT` to `text`, `INTEGER` to
     `bigint`, `REAL` to `double precision`, `BLOB` to `bytea`; no `WITHOUT
     ROWID`; the same primary keys and the same foreign keys with `ON DELETE
     CASCADE`. The trigger `delete_resource_children` is not created (C6.8
     deletes children explicitly).
  4. Kind tables: `account text NOT NULL, region text NOT NULL, id text NOT
     NULL, content_hash bytea NOT NULL, first_seen bigint NOT NULL,
     last_changed bigint NOT NULL, record bytea NOT NULL, nested jsonb`, then
     one column per `Column`: `ScalarType::Bool` to `boolean`, `Integer` to
     `bigint`, `Real` to `double precision`, `Text` to `text`, `Blob` to
     `bytea`, `Timestamp` to `timestamptz`. Primary key `(account, region,
     id)`.
  5. **Timestamps are `timestamptz`, one column, not a seconds-and-nanos
     pair.** Reasons: `psql` and desktop clients can filter, sort and index it
     without arithmetic; it is 8 bytes instead of 16; and it costs one column
     of the 1,600 instead of two. The price is precision: the column holds
     microseconds. The record keeps the full value, and the schema document
     says so. Reals need no `__nonfinite` companion either: NaN and the
     infinities are valid `double precision` values.
  6. Counters: `commit/references_tail.rs::enqueue` keeps
     `projection_revision` and `projection_pending` as rows of `metadata` and
     updates them on every enqueue. Use a sequence
     `inventory.projection_revision_seq` instead, so several writers do not
     update one hot row.
  7. Version: `metadata` row `schema_version` = 14. `initialize` refuses a
     database whose `inventory` schema already exists.
  8. DDL runs in transactions of at most 200 kind tables each.
- **Tests:** against a `TestDatabase`: schema created for the committed
  catalog; the registry loaded back equals the catalog's; a second
  `initialize` is refused; every `ScalarType` round-trips one value through an
  insert and a typed select, including NaN, a timestamp with nanoseconds
  (stored truncated to microseconds) and a 1 MiB blob.
- **Budget:** P11: time to create and to reopen, table and column counts, and
  `pg_database_size` of the empty schema, for today's catalog and again after
  A1.7.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_schema14_postgres -- --ignored`.
- **Done when:** the schema exists on PostgreSQL and the P11 row is recorded.
- **Size:** about 380 added. One change.
- **Watch out:** this is link 1 of the schema chain in BUILD_PLAN; every later
  schema change edits these files, one at a time. One `CREATE TABLE` with a
  primary key and wide columns takes locks on up to four relations (table,
  index, TOAST table, TOAST index). The lock table holds about
  `max_locks_per_transaction` times `max_connections` entries (64 times 100 by
  default, 256 times 200 with the compose settings), so creating about 2,200
  kind tables in one transaction can fail with "out of shared memory" on a
  default server; that is why the DDL is batched (computed, not run).
  Identifiers longer than 63 bytes are truncated silently: A2.3 and C1.3 keep
  table and column names within 63.
- **Scale (V76):** every large table has the account as the leading key
  column, and the generator can emit a table partitioned by account hash
  from the same definition. Partitioning stays off until S1.8 measures it,
  but no key or index may make it impossible later.

### C6.4 `PgStore`: connect, create, open

- **Goal:** The PostgreSQL implementation type with its pool and registry.
- **Depends on:** C6.3.
- **Files:** `crates/engine/src/inventory/postgres.rs` (new),
  `inventory/postgres/connection.rs` (new),
  `inventory/postgres/url.rs` (new), `crates/engine/src/inventory.rs` (module
  and re-exports).
- **Change:**
  1. `pub struct PgStore { pub(crate) pool: PgPool, pub(crate) registry:
     SchemaRegistry, pub(crate) config: PgWriteConfig }`. `PgStore::create(url:
     &DatabaseUrl, registry, config)` and `PgStore::open(url, config)`.
  2. `pub struct DatabaseUrl(String)` with `parse` (must start with
     `postgres://` or `postgresql://`) and a `Debug` and `Display` that replace
     the password with `***`.
  3. Pool: `PgPoolOptions` with `max_connections` from the config (default 8),
     `acquire_timeout` 5 seconds, and an `after_connect` hook running `SET
     search_path = inventory` and `SET application_name = 'aws-inventory'`.
  4. `open` reads `schema_version` and refuses anything but 14 with
     `StoreError::SchemaVersion`, loads the registry, and checks the catalog
     digest as `BinaryStore::validate_catalog` does.
  5. No store-selecting enum is added. The only place that ever needs both
     stores is one `match` in the CLI between C6.10 and C6.12.
- **Tests:** create then open; open on an empty database refused; wrong schema
  version refused; catalog digest mismatch refused; the URL never appears in
  an error message with its password.
- **Prove it:** `cargo test -p inventory-engine --locked postgres -- --ignored`.
- **Done when:** a `PgStore` can be created and reopened in a `TestDatabase`.
- **Size:** about 240 added. One change.
- **Watch out:** `sqlx::Error` prints connection details: wrap connection
  errors so the password is never logged.

### C6.5 The work queue with leases

- **Goal:** `seed_works`, `claim`, `finish_scope`, `resume_sweep` and the
  active-sweep check on PostgreSQL.
- **Depends on:** C6.4.
- **Files:** `crates/engine/src/inventory/postgres/work.rs` (new),
  `inventory/postgres/lease.rs` (new),
  `crates/engine/src/inventory/work/transitions.rs` (visibility only:
  `encode_scope`, `decode_scope`, `encode_resource`, `decode_resource`,
  `put_text` and `Cursor` become `pub(in crate::inventory)`).
- **Change:**
  1. `seed_works`: one statement, `INSERT INTO work_queue(sweep_id, scope_key,
     token, bindings, state, generation, lease_until) SELECT $1, k, NULL, b, 0,
     0, 0 FROM UNNEST($2::bytea[], $3::bytea[]) AS t(k, b) ON CONFLICT DO
     NOTHING`, in batches of 5,000.
  2. `claim`: one statement with `FOR UPDATE SKIP LOCKED` (the batch form is
     B3.1): select the first claimable row ordered by `scope_key`, update it to
     `state=1, generation=generation+1, lease_until=$now+$lease`, `RETURNING`
     the lease fields.
  3. `validate_lease` inside a transaction: `SELECT generation, state,
     lease_until FROM work_queue WHERE sweep_id=$1 AND scope_key=$2 FOR
     UPDATE`; stale when the generation differs, the state is not 1 or the
     lease has passed.
  4. Active sweep: one active sweep identity in the queue. Enforce it with
     `SELECT pg_advisory_xact_lock(14)` before the check inside `seed_works`,
     so two processes cannot both seed.
  5. `finish_scope` and `resume_sweep`: the statements of `work.rs` and
     `work/transitions.rs` with `$n` parameters and `ON CONFLICT (...) DO UPDATE SET ... = EXCLUDED....`.
- **Tests:** the work-queue cases of the conformance suite with a PostgreSQL
  factory (a `TestDatabase` per case); plus two connections claiming at once
  never return the same scope; a second process seeding another sweep is
  refused with `ActiveSweepConflict`.
- **Budget:** P9 claims per second on an empty-page workload, recorded.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_store -- --ignored postgres_work`.
- **Done when:** the work-queue conformance cases pass on PostgreSQL.
- **Size:** about 330 added. One change.
- **Watch out:** scope keys are `bytea`; `ORDER BY scope_key` is bytewise,
  which the kill matrix relies on. `generation` is `bigint`; keep the overflow
  check.
- **Scale (V76):** nothing here may assume the whole sweep is seeded up
  front. Seeding takes a set of accounts and regions, so S1.3 can feed it a
  window at a time. Claims already come from the store under a lease, so
  more than one collector process can share one database; keep that true,
  because it is how the sweep grows past one machine.

### C6.6 Page commit, set-based

- **Goal:** `commit_page` on PostgreSQL, atomic per page, writing only rows
  whose content hash changed, in a handful of statements per page instead of
  several per row.
- **Depends on:** C6.5.
- **Files:** `crates/engine/src/inventory/postgres/commit.rs` (new),
  `postgres/commit/resources.rs` (new), `postgres/commit/kind_rows.rs` (new),
  `postgres/commit/arrays.rs` (new), `postgres/commit/queue.rs` (new),
  `crates/engine/src/inventory/commit/references_tail.rs` (visibility of
  `reference_scope`, `reference_path`, `source_key` only).
- **Change:** one transaction per page:
  1. `validate_lease` (C6.5).
  2. Load what is known for the page's keys in one query: `SELECT id,
     content_hash, first_seen FROM resources WHERE account=$1 AND region=$2
     AND kind=$3 AND id = ANY($4::text[])`.
  3. In Rust, sort the page's observations into unchanged, changed and new by
     comparing hashes. Unchanged rows produce no statement at all.
  4. Changed and new rows, `resources`: `INSERT ... SELECT * FROM UNNEST(...)
     ON CONFLICT (account, region, kind, id) DO UPDATE SET name=EXCLUDED.name,
     content_hash=EXCLUDED.content_hash, last_changed=EXCLUDED.last_changed
     WHERE resources.content_hash IS DISTINCT FROM EXCLUDED.content_hash`.
  5. Changed and new rows, the kind table: the same form with one array
     parameter per column, built in `arrays.rs` by walking each record once
     per column path (the logic of `typed.rs::bind_column`, written again for
     arrays): `Vec<Option<bool>>`, `Vec<Option<i64>>`, `Vec<Option<f64>>`,
     `Vec<Option<String>>`, `Vec<Option<Vec<u8>>>`, and timestamps as
     `Vec<Option<i64>>` microseconds converted in SQL with `timestamptz
     'epoch' + u.v * interval '1 microsecond'`.
  6. Changed rows: delete their outgoing `edges`, `tags` and
     `unresolved_refs` with one `DELETE ... USING UNNEST(...)` per table.
  7. Insert the page's unresolved references, tags and child work with
     `UNNEST` inserts and `ON CONFLICT DO NOTHING`.
  8. Enqueue changed and new keys into `projection_outbox` with
     `nextval('projection_revision_seq')`, one statement.
  9. Record the continuation digest and update the queue row (next token and
     state), as `record_continuation` and `persist_page_state` do in
     `work/transitions.rs`.
  10. `COMMIT`. Reference resolution and scope completion are C6.8; until then
      the "done" path returns `StoreError::Invalid("completion not
      implemented")`.
- **Tests:** the page-commit conformance cases; plus: an unchanged page issues
  exactly three statements (lease, load, queue update), counted by a counting
  wrapper in tests; a page of 10,000 rows commits; a record value that does
  not match its column type is refused; a failure in step 7 leaves no row from
  steps 4 to 6.
- **Budget:** P9 writer rows per second and P1 on `thin`, first recorded once
  C6.10 can run a sweep; this row is the "before" for C6.7 and B4.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_store -- --ignored postgres_commit`.
- **Done when:** the cases pass.
- **Size:** about 480 added. Two changes: (1) steps 1 to 5 and 9 to 10,
  (2) steps 6 to 8.
- **Watch out:** a page has one kind (one plan), but do not assume it: group
  by kind. PostgreSQL allows 65,535 bind parameters; one array per column
  keeps the count at the column count. `IS DISTINCT FROM` matters only when
  another writer changed the row between steps 2 and 4. Atomicity is what the
  kill matrix (B4.3) proves: nothing here may run outside the transaction.
  At this point the store still writes `resource_scopes` for every
  observation, as `commit.rs::persist_observation` does; B4.5 removes that.

### C6.7 COPY for the cold path

- **Goal:** Load new rows with `COPY ... FROM STDIN (FORMAT binary)` into a
  staging table and apply them with one set-based upsert, and measure it
  against C6.6's `UNNEST` form.
- **Depends on:** C6.6, C6.8, B5.2.
- **Status:** PostgreSQL defaults to UNNEST. `INVENTORY_POSTGRES_COPY_MIN_ROWS=N`
  selects staging COPY; adding `INVENTORY_POSTGRES_COPY_DIRECT=1` selects
  direct COPY for pages whose keys are all absent. Direct without the threshold
  is rejected, and the measurement itself is NOT yet recorded.
- **Files:** `crates/engine/src/inventory/postgres/copy.rs` (new),
  `postgres/copy/binary.rs` (new), `postgres/commit/kind_rows.rs`,
  `postgres/commit/resources.rs`.
- **Change:**
  1. `binary.rs`: a writer for PostgreSQL's binary COPY format: the 19-byte
     header, per tuple a 16-bit field count, per field a 32-bit length (or -1
     for NULL) and the value: `text` as UTF-8, `bigint` and `double precision`
     as 8 bytes big-endian, `boolean` as 1 byte, `bytea` raw, `timestamptz` as
     a 64-bit count of microseconds since 2000-01-01, `jsonb` as the byte 1
     followed by JSON text; then the 16-bit trailer -1.
  2. Staging: per connection and kind, `CREATE TEMPORARY TABLE IF NOT EXISTS
     stg_<table> (LIKE <table>) ON COMMIT DELETE ROWS` (and one for
     `resources`). B4.4 measures `UNLOGGED` shared staging against this.
  3. When a page (or a group of pages, B4.1) has at least `copy_min_rows`
     changed or new rows (default 64, measured), write them with
     `copy_in_raw` into staging, then `INSERT INTO <table> SELECT * FROM
     stg_<table> ON CONFLICT (...) DO UPDATE SET ... WHERE <table>.content_hash
     IS DISTINCT FROM EXCLUDED.content_hash`. Below the threshold keep the
     `UNNEST` form.
  4. When C6.6's key lookup found none of the page's keys and the changed-row
     threshold is met, COPY directly into the resource and kind tables. A 23505
     unique violation rolls back only the direct-attempt savepoint, preserving
     earlier outer-transaction work, then retries this page through staging.
     Other errors propagate.
- **Tests:** binary writer against the server for every column type and NULL;
  a 100,000-row COPY; staging is empty after commit; the direct variant's
  fallback on a forced unique violation; the conformance suite passes with
  `copy_min_rows` set to 1.
- **Budget:** P1, P8 and P9 on `thin` and `full` cold sweeps: `UNNEST` (the
  C6.6 row) against staging COPY against direct COPY. P2 must not change (a
  warm sweep has nothing to copy).
- **Prove it:** conformance with COPY forced; `cmdb-bench sweep` three times
  with the three settings.
- **Done when:** the three rows are recorded and the default follows them.
- **Size:** about 420 added. Two changes: (1) the binary writer and staging,
  (2) the direct variant and thresholds.
- **Watch out:** `copy_in_raw` on a `PgConnection` exists in upstream `sqlx`;
  it is not verified at this pin. Temporary tables per connection and kind add
  catalog rows; with about 2,200 kinds and a pool of 16 that is many. If
  `pg_class` growth shows in P11, switch to one shared `UNLOGGED` staging table
  per kind keyed by a writer id. No integer may be converted with `as`; use
  `to_be_bytes` on checked values.

### C6.8 References, edges, completion and deletion

- **Goal:** The rest of the write path on PostgreSQL: late-resolving
  references, ambiguous matches, scope completion, deletion after two complete
  enumerations, and deletion of children with their parent.
- **Depends on:** C6.6.
- **Files:** `crates/engine/src/inventory/postgres/references.rs` (new),
  `postgres/completion.rs` (new), `postgres/commit.rs`.
- **Change:**
  1. Resolve the page's references in sets, one statement per target kind and
     target column: join the page's `(target account, target region, value)`
     tuples from `UNNEST` to the kind table on the target column, group by
     reference, keep groups with exactly one match, insert those edges with
     `ON CONFLICT DO NOTHING RETURNING` and enqueue both ends. Two or more
     matches draw no edge (the ambiguous case).
  2. Inbound reconciliation (what `inventory/references.rs::reconcile_inbound`
     does today):
     when a changed row's target-column value changes, retract edges that
     pointed at the old value and resolve waiting `unresolved_refs` against
     the new one, per column in one statement each.
  3. Completion: `complete_scope` as SQL sets: mark resources of the scope not
     seen in this sweep (`missing_complete = missing_complete + 1`), then
     delete those with no scope left below 2, walking children with `WITH
     RECURSIVE` over `resource_parents` as `completion.rs::delete_tree` does,
     deleting kind rows per table and then `resources`, and enqueueing each
     deleted key with `deleted = true`.
  4. The trigger `delete_resource_children` of `schema.rs` is not carried
     over: the recursive
     delete in step 3 covers it, and the foreign keys cascade the rest.
- **Tests:** the reference, completion and deletion conformance cases,
  including: late target; ambiguous target; target value changes; deletion
  only after the second complete enumeration; partial deletes nothing;
  three-level parent chain deleted from the top.
- **Budget:** the reference statements per page must not grow with rows per
  page; recorded with P1 on `thin` once C6.10 runs.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_store -- --ignored postgres`.
- **Done when:** the whole conformance suite passes on PostgreSQL.
- **Size:** about 450 added. Two changes: (1) references, (2) completion.
- **Watch out:** the lookups join on typed target columns; without an index
  they scan the kind table per page. C1.5 adds the indexes; until then note
  the scan in the row. B4.5 rewrites how "seen" is recorded; write step 3 so
  the seen set is one function.

### C6.9 The projection outbox and projection reads

- **Goal:** `drain_projection`, `ack_projection`, `projection_state`, the
  paged projection reads and the snapshot on PostgreSQL, so `run_projection`
  works unchanged.
- **Depends on:** C6.8.
- **Files:** `crates/engine/src/inventory/postgres/outbox.rs` (new),
  `postgres/projection_read.rs` (new).
- **Change:**
  1. Drain: `SELECT resource_key, revision, deleted, full_rebuild FROM
     projection_outbox ORDER BY full_rebuild DESC, revision, resource_key
     LIMIT $1`.
  2. Ack: `DELETE ... WHERE resource_key=$1 AND revision=$2` (a newer revision
     of the same key stays queued). Pending count: `SELECT count(*)` bounded
     by `MAX_PROJECTION_ROWS` (100,000); when the outbox would pass that
     bound, replace it with one full-rebuild marker as `enqueue_rebuild` does.
  3. Snapshot: `type Snapshot` is a `REPEATABLE READ READ ONLY` transaction;
     the paged reads use keyset pagination on `(account, region, kind, id)` as
     `projection_read.rs` does.
- **Tests:** the outbox and projection conformance cases; a re-enqueued key
  survives the ack of its older revision; overflow turns into a rebuild
  marker; a snapshot does not see a commit made after it began.
- **Budget:** P15 (projection keys per second and rebuild time), first
  recorded here.
- **Prove it:** conformance; `cargo test -p inventory-engine --locked --test
  inventory_projection_live -- --ignored` with the PostgreSQL URL and a
  FalkorDB URL.
- **Done when:** a store filled through `PgStore` projects into FalkorDB.
- **Size:** about 300 added. One change.
- **Watch out:** the sequence gives gaps and is not transactional; the
  projection must not assume consecutive revisions (`runner.rs` orders by
  revision and acks by exact value, which is fine).

### C6.10 The sweep runs on PostgreSQL, and the baseline scenario passes

- **Goal:** `aws-inventory sweep --database postgres://...` works, and the
  existing `sweep_e2e` scenario passes against PostgreSQL. This is the
  condition for C6.12.
- **Depends on:** C6.9, C5.3, C6.2.
- **Files:** `src/cli/sweep.rs` (`database: PathBuf` becomes `String`),
  `src/sweep_command/enabled.rs` (`open_store`, `collect`),
  `tests/sweep_e2e/stack.rs`, `tests/sweep_e2e/state.rs` (283 lines),
  `tests/sweep_e2e/state/postgres.rs` (new),
  `tests/sweep_e2e/scenario/arguments.rs`, `tests/sweep_e2e.rs`.
- **Change:**
  1. `collect` branches once on the `--database` value: a `postgres://` URL
     opens or creates a `PgStore` and calls the generic `run`; anything else
     is a file path and takes today's `BinaryStore` path, unchanged. C6.12
     removes that second arm.
  2. `PgStore::open` if the `inventory` schema exists, else `create`;
     `--resume` on a missing schema is refused.
  3. `Stack` gains a `TestDatabase` (C6.2) created beside the emulator and the
     graph server, and `Environment` gains `database_url`.
  4. `state.rs` reads the database file with its own queries
     (`readonly_pool`). The assertions stay; the reads go behind a two-arm
     enum, with the PostgreSQL arm in `state/postgres.rs`.
  5. The suite runs against the target named by `INVENTORY_E2E_STORE`:
     `postgres` (the default) or `file` (today's `BinaryStore` path).
     `scripts/e2e-baseline.sh` runs it for both until C6.12.
- **Tests:** the two existing tests
  (`production_sweep_persists_and_projects_real_seeded_inventory`,
  `controlled_tls_403_preserves_existing_inventory`) on PostgreSQL: seeded
  resources stored, `IN_VPC` edge, projection, zero-write re-sweep, resume
  after a kill, no deletion on a denied sweep.
- **Budget:** P1, P2, P8, P9 on `thin`: the first row of the benchmark table
  (B0.2).
- **Prove it:** `INVENTORY_E2E_STORE=postgres cargo test --locked --features
  sweep --test sweep_e2e -j1 -- --ignored --test-threads 1` gives 2 passed;
  the same with `file` still gives 2 passed.
- **Done when:** both commands pass. This is X10 for the four current
  operations.
- **Size:** about 300 added, 100 changed. One change.
- **Watch out:** a database URL can carry a password; clap must not echo it in
  errors, and the summary line must not print it. `state.rs` is near the size
  limit; split before adding.

### C6.11 `cmdb-view` and the UI read PostgreSQL

- **Goal:** Tables, details, counts and edge pages in the UI come from
  PostgreSQL through `cmdb_view::Database`.
- **Depends on:** C6.10.
- **Files:** `crates/cmdb-view/src/database.rs` (267 lines),
  `crates/cmdb-view/src/database/postgres.rs` (new),
  `database/postgres/queries.rs` (new), `database/postgres/snapshot.rs` (new),
  `crates/cmdb-view/src/error.rs`, `crates/cmdb-view/Cargo.toml`
  (dev-dependency on the harness for `TestDatabase`),
  `crates/cmdb-view/tests/database/support.rs`,
  `crates/cmdb-ui/src/worker.rs` (`WorkerConfig.database` becomes
  `Option<String>`), `crates/cmdb-ui/src/worker/lifecycle.rs`,
  `crates/cmdb-ui/src/worker/tests.rs`.
- **Change:**
  1. Write each query of `database/queries.rs` and `database/snapshot.rs` for
     PostgreSQL in the new files: `$n` placeholders; the snapshot is a
     `REPEATABLE READ READ ONLY` transaction; `detail` reads `record` from the
     kind table named in `kinds`; `search` uses `ILIKE` with the pattern
     escaped, matching the semantics the tests pin. Pool of 4, `SET
     search_path = inventory` and `SET default_transaction_read_only = on` in
     `after_connect`.
  2. `Database::open(target: &str)`: a `postgres://` URL uses the new reader;
     a path uses today's reader in `database/queries.rs` and
     `database/snapshot.rs`, untouched, until C6.12 deletes it. Public types
     (`ResourceFilter`, `StoredResource`, `PageCursor`, `PageRequest`,
     `ResourcePage`, `DatabaseCounts`, `EdgeCursor`, `EdgePageRequest`,
     `EdgePage`, `LodSummary`, `MAX_PAGE_LIMIT`) do not change.
  3. The UI's `--database` accepts a URL. A URL is never shown with its
     password.
- **Tests:** the `cmdb-view` database tests (`tests/database/cursors.rs`,
  `snapshots.rs`) run against a PostgreSQL database filled through `PgStore`;
  worker tests run `Refresh` and `Expand` against it.
- **Budget:** P12: first page of 500 rows and the counts query at 250,000
  resources, through `cmdb-bench measure`.
- **Prove it:** `cargo test -p cmdb-view -p cmdb-ui --locked -- --ignored
  postgres`; `cmdb-bench measure postgres://...`.
- **Done when:** the UI's worker returns the swept rows from PostgreSQL and
  the P12 row is recorded.
- **Size:** about 330 added. One change.
- **Watch out:** `cmdb-view` must not gain a dependency on
  `inventory-engine` for this (it reads the documented tables directly, as an
  external client would); the test fixtures may, as a dev-dependency.
  `count(*)` on PostgreSQL scans; if the counts query is slow at 250,000 rows,
  read counts per kind from a small summary the store maintains, as its own
  step. `crates/cmdb-view/src` is under the under-60-line function gate.

### C6.12 Delete the old store code

- **Goal:** Remove the SQLite schema 14 store: every reader and writer of that
  database file, in code, tests, manifests and documents. After this step
  `PgStore` is the only store of the sweep pipeline.
- **Ordering, said once here:** the code listed below is left untouched, and
  `tests/sweep_e2e` keeps passing on it (2 passed), until the same scenario
  passes on PostgreSQL (C6.10). Only then does this step land.
- **Depends on:** C6.10 (the baseline scenario passes on PostgreSQL), C6.11
  (the UI reads PostgreSQL), D4.1 (the benchmark generator writes PostgreSQL).
- **Files, deleted outright:**
  - `crates/engine/src/inventory/connection.rs`, `schema.rs`, `typed.rs`,
    `commit.rs`, `commit/queue.rs`, `commit/references_tail.rs`,
    `completion.rs`, `references.rs`, `projection_outbox.rs`.
  - `crates/engine/tests/inventory_schema14.rs`.
  - `crates/cmdb-view/src/database/queries.rs`, `database/snapshot.rs` (the
    SQLite reader), and the SQLite fixture builders in
    `crates/cmdb-view/tests/database/support.rs`.
  - `crates/cmdb-bench/src/main.rs`: `connect_writer`, `ensure_fresh`,
    `execute_schema`, `encode_record`, `insert_resource`, `insert_detail`,
    `insert_summary` and the SQLite `generate` (after B5.2's split they are in
    `src/generate.rs`); the SQLite cases in `crates/cmdb-bench/src/tests.rs`.
  - `tests/sweep_e2e/state.rs`: `readonly_pool` and the file arm of every
    read.
- **Files, reduced:**
  - `crates/engine/src/inventory.rs`: delete `BinaryStore` (its two
    `SqlitePool` fields and its `impl`); rename `StoreError::Sqlite(#[from]
    sqlx::Error)` to `StoreError::Database` with the text "storage operation
    failed: {0}"; keep `StoreError`, `SchemaRegistry`, `PageCommit`, `Tag` and
    the trait re-exports.
  - `crates/engine/src/inventory/work.rs`: delete `seed`, `seed_works`,
    `claim`, `commit_page`, `finish_scope`; keep `SweepId`, `WorkSpec`,
    `WorkLease`, `ChangeStats`, `SweepSummary`, `CommitResult`,
    `PendingProjection`, `ProjectionAck`.
  - `crates/engine/src/inventory/work/transitions.rs`: delete the SQL
    functions; move `encode_scope`, `decode_scope`, `encode_resource`,
    `decode_resource`, `put_resource`, `decode_resource_from`, `put_text`,
    `Cursor`, `scope_bytes`, `resource_bytes` to `inventory/codec.rs` (new).
  - `crates/engine/src/inventory/validation.rs`: delete
    `validate_active_sweep`; `validate_work` and `validate_page` take
    `&SchemaRegistry` instead of `&BinaryStore`.
  - `crates/engine/src/inventory/projection_read.rs`: delete the SQL functions
    and `ProjectionSnapshot` (it holds a `Transaction<'static, Sqlite>`); keep
    `ProjectionEdgeCursor`, `ProjectionEdgeHeader`, `ProjectionEdgePage`,
    `ProjectionResource`, `ProjectionState`.
  - Move, before deleting their files: `encode_schema`, `decode_schema`,
    `decode_column`, `scalar_code`, `decode_scalar`, `SchemaCursor` from
    `schema.rs` to `inventory/postgres/schema/definition.rs` (new);
    `reference_scope`, `reference_path`, `put_component`, `source_key`,
    `text_field`, `edge_source` from `commit/references_tail.rs` to
    `inventory/reference_codec.rs` (new).
  - `crates/engine/tests/inventory_store/support.rs`: the SQLite factory goes;
    `inventory_store.rs` runs the conformance suite with the PostgreSQL
    factory only. `inventory_projection_live.rs` loses its SQLite setup.
  - `crates/cmdb-view/src/database.rs`: the path arm of `Database::open` and
    every `SqlitePool` use.
  - `crates/cmdb-ui/src/worker/tests.rs`: the SQLite fixtures (the test
    `given_unconfigured_graph_when_user_refreshes_then_sqlite_counts_stay_usable`
    is renamed and runs on PostgreSQL).
  - `src/sweep_command/enabled.rs`: the file-path arm of `collect` and
    `open_store`'s `std::fs::metadata` check; `src/cli/sweep.rs`:
    `--database` is documented as a URL.
  - `tests/sweep_e2e.rs`, `scenario/arguments.rs`: `INVENTORY_E2E_STORE` and
    its `file` branch; `scripts/e2e-baseline.sh` runs once.
  - `inventory_core::inventory::ScalarType::physical_columns` (2 for real and
    timestamp) describes the SQLite column layout only: delete it; C1.2 sets
    the budget.
- **Cargo:** move the `sqlite` feature of `sqlx` out of the workspace
  declaration. `[workspace.dependencies] sqlx` keeps `runtime-tokio` only;
  each crate adds what it uses: `inventory-engine` `["sqlite", "postgres"]`
  (its `store` module), `inventory-harness` `["sqlite", "postgres"]`, the root
  package's dev-dependency `["sqlite", "postgres"]`, `cmdb-view` and
  `cmdb-bench` `["postgres"]`.
- **Documents, brought in line in the same change** (the files, read from the
  tree; what each says is rewritten for PostgreSQL, not quoted here):
  - `docs/VISION.md`: decision rows V05, V32 and V33; the architecture diagram
    and the component table of section 5; sections 5.4 and 5.8; section 8;
    section 9; the O4 row of section 12.
  - `START_HERE.md`, `CONTRACT.md`, `README.md`, `NEXT_AGENT_PROMPT.txt`: the
    parts that describe the sweep's store. Their parts about the `scan` path
    wait for K2.1 and K4.1.
  - `docs/STORAGE.md`, `docs/README.md` (its index rows for the store),
    `docs/ROADMAP.md` (the status entry for the first end-to-end slice and the
    rows M3.2, M3.3 and M6.2), `docs/VERIFICATION.md`,
    `docs/ARCHITECTURE.md`, `docs/CMDB_ARCHITECTURE.md`,
    `docs/CMDB_PERFORMANCE_CONTRACT.md`.
  - There is no `docs/SCHEMA.md` in this tree. If one exists by the time this
    step lands, it is included.
  - `docs/LOCAL_SETUP.md` gains a pointer to `deploy/local/README.md` (C6.2)
    for running PostgreSQL.
- **What cannot go yet, and why.** The old `scan` pipeline still uses SQLite
  until K2.1 deletes it: `crates/engine/src/store/` (the schema 13 store, 17
  entries), `crates/engine/src/jobs.rs` and `jobs/`,
  `crates/engine/src/graph.rs` and `graph/`, `crates/engine/src/export*`, the
  SQL files 001 to 013, the `scan`, `query`, `ingest`, `demo`, `export`,
  `deliver`, `job` and analysis commands in `src/`, `tests/cli_properties*`
  and `tests/emulator_cli*`, the harness's `snapshot/`, `audit/persisted.rs`
  and `assertions/cloud_control.rs`, and the engine tests `store.rs`,
  `migrations.rs`, `jobs.rs`, `scan_lifecycle.rs`, `sqlite_dependency.rs` and
  their kin. Therefore these also stay: the `sqlite` feature of `sqlx` (on the
  crates above), `libsqlite3-sys` in `Cargo.lock`, `LIBSQLITE3_FLAGS` with
  `SQLITE_MAX_COLUMN=256` in `.cargo/config.toml`,
  `crates/engine/src/store/sql_policy.rs`, and the documents whose subject is
  the `scan` path (`docs/ANALYSIS.md`, `docs/EXPORT.md`, `docs/DELIVERY.md`,
  `docs/DURABLE_JOBS.md`, `docs/COLLECTION_PLAN.md` and the like).
- **Tests:** the conformance suite, the e2e suite, the `cmdb-view` and worker
  tests, all on PostgreSQL. A root test reads the files under
  `crates/engine/src/inventory`, `crates/sweep/src`, `crates/cmdb-view/src`,
  `crates/cmdb-bench/src` and `crates/cmdb-ui/src` and fails if any contains
  `Sqlite` or `sqlite`.
- **Prove it:**
  - `grep -rIl "Sqlite\|sqlite" crates/engine/src/inventory crates/sweep/src
    crates/cmdb-view crates/cmdb-bench crates/cmdb-ui` prints nothing.
  - `cargo tree -p cmdb-view -e normal -i sqlx-sqlite` and `cargo tree -p
    cmdb-bench -e normal -i sqlx-sqlite` report that the package does not
    match (expected; how `cargo tree -p` reports per-crate features under
    resolver 3 is not verified, and `cmdb-bench` depends on the engine after
    D4.1, in which case the path shown must go through `inventory-engine`
    only).
  - `cargo tree -p inventory-engine -e normal -i sqlx-sqlite` still shows it,
    reached from `inventory-engine` itself, until K2.1.
  - `cargo test --locked --features sweep --test sweep_e2e -j1 -- --ignored
    --test-threads 1` gives 2 passed on PostgreSQL.
- **Done when:** the greps and the suite hold, and the Rust lines deleted are
  stated (expected: about 2,300 in `crates/engine/src/inventory`, about 500 in
  `cmdb-view`, about 200 in `cmdb-bench`, plus tests).
- **Size:** about 120 added (moves and signature changes), about 3,500
  deleted, plus the documents. One change, reviewed as a deletion.
- **Watch out:** after this step there is no second store to fall back on. Do
  not land it in the same change as anything else. D3.1 later makes
  `cmdb-view` depend on `inventory-engine`, which brings `sqlx-sqlite` back
  into its tree until K2.1. Databases written by the deleted code are not
  migrated (V24): they are re-collected.

## C1 Typed columns

### C1.1 Column names and every scalar member, in the catalog

- **Goal:** The catalog carries, for each kind, one named column per scalar
  member of its item shape, nested structures flattened.
- **Depends on:** C6.12 (so that `Column` changes once), A0.3.
- **Files:** `crates/catalog-tool/src/runtime_catalog/schemas.rs` (82 lines),
  `runtime_catalog/schemas/columns.rs` (new),
  `runtime_catalog/schemas/column_name.rs` (new),
  `crates/catalog-tool/src/names/words.rs` (reuse),
  `crates/core/src/inventory/schema.rs` (`Column`),
  `crates/core/src/inventory/identifiers.rs` (new `ColumnName`),
  `crates/core/src/catalog/runtime/decode/schemas.rs`,
  `crates/engine/src/inventory/postgres/schema/definition.rs`,
  `docs/API_CATALOG.md`.
- **Change:**
  1. `columns.rs`: for a kind's item shape, walk structure members
     depth-first. A member whose target is boolean, byte, short, integer,
     long, float, double, string, enum, intEnum, blob or timestamp yields a
     column with the path of field IDs and the scalar code used today (0 to
     5). A structure member recurses. A list, map, document or union member
     yields no column (C2 handles them). Stop at path length 8 (the existing
     path bound) and when a shape ID repeats on the path.
  2. `column_name.rs`: split each member name into words with the rule of
     `names/words.rs` (so `VpcId` is `vpc_id`, `CidrBlock` is `cidr_block`,
     `DBInstanceArn` is `db_instance_arn`), lower-case, and join the path's
     names with `_`. `SqlName`'s constructor cannot be used as is: it requires
     at least two words (`names/identifier.rs`), and `arn` or `state` are one
     word. Add `ColumnName` in core with the rule `[a-z][a-z0-9_]*`, at most
     63 bytes (PostgreSQL's identifier limit), not a fixed column (`account`,
     `region`, `id`, `content_hash`, `first_seen`, `last_changed`, `record`,
     `nested`).
  3. Catalog column map gains field 3, the name. `Column::new(path, scalar)`
     becomes `Column::named(name: ColumnName, path, scalar)`;
     `Column::selected_scalar` keeps the name `value`. `KindSchema::new`
     rejects duplicate names as it rejects duplicate paths.
  4. `definition.rs` stores the name (length byte and bytes) in
     `kind_schemas.definition`.
  5. The display-name path and relation target paths are ordinary members of
     this set now; `text_columns` is deleted. The reader's check that the name
     path is one of the columns stays.
- **Tests:** `AWS::EC2::VPC` gets `cidr_block text`, `is_default boolean`,
  `vpc_id text`; a nested member (`State.Name`) gets `state_name`; a timestamp
  member gets scalar code 5; a recursive shape terminates; a member under a
  list yields nothing; the reader rejects a column with no name, with an
  invalid name, and two columns with one name.
- **Budget:** P16: catalog bytes and compile time before and after (the
  column lists are the largest addition to the catalog so far). P11: catalog
  load time.
- **Prove it:** `cargo test -p catalog-tool -p inventory-core --locked`;
  `models compile --write` then `--check`; the e2e suite on a fresh database.
- **Done when:** `SELECT cidr_block FROM inventory.aws_ec2_vpc` can be written
  against a freshly swept database.
- **Size:** about 380 added, 60 deleted. Two changes: (1) core `Column` and
  reader, (2) the compiler.
- **Watch out:** schema chain link 3 starts here. Name collisions and the
  column budget are the next two steps; until they land, this step must fail
  the compile on a collision or an over-wide kind instead of emitting it.

### C1.2 The column budget fits PostgreSQL

- **Goal:** PostgreSQL allows 1,600 columns per table. Core's check allows
  2,000 physical columns, counting a real or a timestamp as 2
  (`ScalarType::physical_columns`; `crates/core/src/inventory/schema.rs` line
  114). Bring the budget down to
  what the store can create, cut deterministically, and report the cut.
- **Depends on:** C1.1.
- **Files:** `crates/core/src/inventory/schema.rs`
  (`validate_column_budget`, `canonical_columns`),
  `crates/core/src/catalog/runtime/decode/metadata.rs` (`MAX_COLUMNS`),
  `crates/catalog-tool/src/runtime_catalog/schemas/budget.rs` (new),
  `runtime_catalog/schemas/report.rs` (new), `catalog/aws-columns.report`
  (new, generated).
- **Change:**
  1. Arithmetic: a kind table has 8 fixed columns (`account`, `region`, `id`,
     `content_hash`, `first_seen`, `last_changed`, `record`, `nested`). Every
     typed column is one physical column on PostgreSQL, timestamps included
     (C6.3 chose `timestamptz`). So at most 1,592 typed columns fit. Had the
     seconds-and-nanos pair been kept, a timestamp would count 2 and the
     budget would depend on the mix; that is one of the reasons it was not.
  2. `pub const MAX_KIND_COLUMNS: usize = 1_592` in core.
     `validate_column_budget` becomes `columns.len() <= MAX_KIND_COLUMNS`; the
     fold that starts at 7 and adds `physical_columns()` is deleted with
     `physical_columns` (C6.12). The reader's `MAX_COLUMNS` (4,096) becomes
     the same constant.
  3. The cut is made once, in the compiler (`budget.rs`), not in the store:
     always keep the display-name column and every relation target column;
     then add columns in order of path length (shallowest first), ties by
     field-ID path, until the limit. The rest stay in the record and in
     `nested` where applicable.
  4. Report, per kind with any cut: `<kind> scalar_members=<n> kept=<n>
     first_cut=<column>`; totals: kinds, columns, kinds cut.
- **Tests:** a kind with 1,700 scalar members keeps 1,592, shallowest first;
  pinned columns are kept even when deep; the same shape always gives the same
  selection; the reader rejects 1,593 columns; a table with 1,592 typed
  columns can be created on PostgreSQL.
- **Budget:** P11: table and column counts and create time, before and after.
- **Prove it:** `cargo test -p inventory-core -p catalog-tool --locked`; the
  report after `models compile --write`; the create test with `-- --ignored`.
- **Done when:** the report exists and the store creates every kind table of
  the catalog.
- **Size:** about 200 added, 30 deleted. One change.
- **Watch out:** the column count is not the only limit. A PostgreSQL heap
  row must fit in an 8 kB page after TOAST, and fixed-width values (`bigint`,
  `double precision`, `timestamptz`, 8 bytes each) cannot be moved out of
  line: a row with more than about 1,000 non-null 8-byte columns fails with
  "row is too big" (computed from the page size, not run). The create test
  must also insert one row with every column non-null. If a real kind hits
  this, lower `MAX_KIND_COLUMNS` for fixed-width columns and report it.
- **Review fix (2026-11-18):** two additions. (1) Column names must be
  stable across catalog versions: persist the mapping from field ID to
  column name in the store's registry when a column is first created, and
  reuse it; a name is never recomputed for an existing column, so a new
  sibling member cannot rename an old column through collision handling.
  (2) Budgets in bytes, not only counts: a bound on the bytes of one row
  (PostgreSQL stores fixed-width columns inline, so many non-null columns
  can exceed a page whatever the column count) and a bound on bytes held in
  flight per page and per commit group. A kind over the row bound keeps its
  shallowest members and reports the cut, as for the column bound.

### C1.3 Name collisions after flattening

- **Goal:** `State.Name` and a sibling `StateName` both flatten to
  `state_name`. Resolve every collision the same way every time.
- **Depends on:** C1.1.
- **Files:** `crates/catalog-tool/src/runtime_catalog/schemas/column_name.rs`,
  `schemas/collisions.rs` (new), `catalog/aws-columns.report`.
- **Change:**
  1. Within one kind, group columns by flattened name. In a group of more than
     one, the shallowest path keeps the name (ties by field-ID path order);
     each other gets `<name>_<k>` with k from 2 in path order. If that name is
     taken, k increases.
  2. A name equal to a fixed column gets the suffix `_field` (`id` becomes
     `id_field`). A name that is a reserved word in PostgreSQL gets `_field`
     too.
  3. A name over 63 bytes is cut to 54 bytes plus `_` plus 8 hex digits of the
     BLAKE3 hash of the full flattened name (the A2.3 rule), then checked for
     collisions again.
  4. Every renamed column is listed in the report as `renamed <kind> <path>
     <final name> <reason>`.
- **Tests:** two-way and three-way collisions; collision between a renamed
  name and an existing one; fixed-column clash; reserved word; a 200-byte
  flattened name; determinism under shuffled member order.
- **Prove it:** `cargo test -p catalog-tool --locked schemas`; `models compile
  --check`.
- **Done when:** the compile no longer fails on a collision and the report
  lists every rename.
- **Size:** about 200 added. One change.
- **Watch out:** adding a member to the model later can change which column
  wins a name. To keep published names stable, order ties by field ID (IDs are
  append-only), never by member name.

### C1.4 The writer fills the columns, and a filter test for every kind

- **Goal:** The PostgreSQL writer fills every column from the record, and a
  test filters on a scalar column of every kind.
- **Depends on:** C1.2, C1.3, C6.6.
- **Files:** `crates/engine/src/inventory/postgres/commit/arrays.rs`,
  `inventory/postgres/extract.rs` (new),
  `crates/engine/tests/inventory_typed_columns.rs` (new).
- **Change:**
  1. `extract.rs`: one function that walks a column path through nested maps
     of a record and returns the typed value, or NULL when an intermediate
     member is absent or is not a map. `arrays.rs` (C6.6) and the COPY writer
     (C6.7) both use it.
  2. Test generator: for each kind in the committed catalog, build a record
     with a deterministic value for each column from its type and path, commit
     it, and run `SELECT count(*) FROM <table> WHERE <column> = <value>` for
     one text column and, when present, one integer and one timestamp column;
     expect 1.
- **Tests:** the generated test; a record whose value has the wrong type for
  its column is refused; an absent nested parent gives NULL.
- **Budget:** P9 writer rows per second and P1 on `thin`, before (today's few
  columns) and after (every scalar column). This is the cost of typed columns;
  record it.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_typed_columns -- --ignored`.
- **Done when:** every kind passes and the cost rows exist.
- **Size:** about 240 added. One change.
- **Watch out:** a timestamp column holds microseconds; compare at that
  precision in the test and say so.

### C1.5 Indexes on reference targets, and the open-time measurement

- **Goal:** Reference resolution looks rows up by a typed column with no
  index. Add the indexes the lookups need, and record the size and open-time
  numbers C_SQL.md asks for.
- **Depends on:** C1.4, A1.7 for the full-catalog numbers.
- **Files:** `crates/engine/src/inventory/postgres/schema/kinds.rs`,
  `crates/cmdb-bench/src/open.rs`.
- **Change:**
  1. For every relation target column of a kind, `CREATE INDEX
     <table>_<column>_ref ON <table>(account, region, <column>)`. Names over
     63 bytes use the A2.3 shortening.
  2. Record with `cmdb-bench open` for the full catalog: table count, column
     count, empty database size, time to create, time to open (connect and
     load the registry), time to prepare one insert per table.
  3. Decision rule from C_SQL.md: if opening exceeds 100 ms, take the fallback
     of VISION 5.4 (kinds share one table and only fields the overlay marks
     `queryable` get columns). The fallback is a new package; this step
     records the number and raises the flag.
- **Tests:** the index exists and `EXPLAIN` shows an index scan for the
  reference query when the table has 10,000 rows and statistics.
- **Budget:** P11 as above; P1 on `thin` with relations, before and after the
  indexes (they speed lookups and slow inserts; record both effects).
- **Prove it:** `cmdb-bench open`; the benchmark rows.
- **Done when:** the numbers are in the table and the 100 ms rule has been
  applied to them in writing.
- **Size:** about 100 added. One change.
- **Watch out:** on PostgreSQL "open" is a connection plus a registry load,
  not a schema parse, so the 100 ms rule (written before V51) may never
  bind; `psql` responsiveness with about 2,200 tables (`\dt`) is the
  practical check and is recorded too.

### C1.6 A catalog update adds columns in place

- **Goal:** Opening a database with a newer catalog adds the new kinds and
  columns instead of failing with `CatalogMismatch`.
- **Depends on:** C1.4, B4.5 (last link of the schema chain before it).
- **Files:** `crates/engine/src/inventory/postgres/upgrade.rs` (new),
  `inventory/store_api.rs`, `src/sweep_command/enabled.rs` (`open_store`),
  `src/cli/sweep.rs`.
- **Change:**
  1. `upgrade_catalog(new_registry)`: refuse while a sweep is active. For each
     kind: absent table, create it; present table, `ALTER TABLE ... ADD
     COLUMN` for each column the stored schema lacks. A column whose type
     changed, and a column that is no longer selected, are left in place and
     listed in the result. Replace `catalog_operations` and `kind_schemas`,
     then the catalog digest; DDL in batches of 200 tables with the digest
     last.
  2. Backfill: a new column is NULL for existing rows, and an unchanged
     resource is never rewritten. After the DDL, fill each added column from
     the stored `record`, in batches of 1,000 rows per transaction, without
     touching `content_hash`, `first_seen` or `last_changed`.
  3. `aws-inventory sweep --upgrade-catalog` opts in. Without the flag a
     digest mismatch is still an error.
- **Tests:** add one member: the column appears, existing rows keep their
  hash and timestamps, and the column is filled from the record; add a kind:
  its table appears; a type change is reported, not applied; upgrade during an
  active sweep is refused; a kill in the middle of the backfill is resumed on
  the next upgrade.
- **Budget:** upgrade time per 100,000 rows per added column, recorded.
- **Prove it:** `cargo test -p inventory-engine --locked upgrade -- --ignored`.
- **Done when:** the C1 exit evidence holds: column added in place, existing
  rows untouched.
- **Size:** about 340 added. Two changes: (1) DDL and digest, (2) backfill.
- **Watch out:** V24 says old databases are thrown away; this step is for
  catalog updates within schema 14, not a migration from any earlier
  database. `ALTER TABLE ADD COLUMN` of a nullable column with no default does
  not rewrite the table on PostgreSQL.
- **Review fix (2026-11-18):** "digest last" is not a protocol. The
  upgrade is driven by C1.7's journal, lock, resumable cursors and atomic
  activation; this step supplies the DDL and backfill that C1.7 sequences.
  A type change or a removed member never alters a column in place: the old
  column is retired in the registry and kept, and a changed type gets a new
  column that is backfilled.

### C1.7 A migration journal, a lock, and atomic activation

- **Goal:** A catalog upgrade that is killed at any point resumes or rolls
  back to a consistent store (review finding 8).
- **Depends on:** C1.6.
- **Files:** `crates/engine/src/inventory/postgres/migrate.rs` (new),
  `inventory/postgres/migrate/journal.rs` (new),
  `inventory/postgres/schema/shared.rs`,
  `crates/engine/tests/inventory_migrate.rs` (new).
- **Change:**
  1. Table `catalog_migrations(id, from_digest, to_digest, state, step,
     cursor, started, finished)` with states `planned`, `ddl`, `backfill`,
     `activating`, `done`, `abandoned`.
  2. One migration at a time: a PostgreSQL advisory lock held for the whole
     upgrade; a sweep refuses to start while a migration is not `done`.
  3. DDL is additive only and idempotent (`ADD COLUMN IF NOT EXISTS`,
     `CREATE TABLE IF NOT EXISTS`), in bounded batches, each batch recorded
     in `step`.
  4. Backfill is resumable: per table, a key cursor stored in `cursor`,
     bounded rows and bounded bytes per transaction.
  5. Activation is one transaction: replace `catalog_operations` and
     `kind_schemas`, set the digest, mark the journal `done`. Until then the
     old catalog stays active and the store still opens with it.
  6. A journal entry that is not `done` is resumed by the next
     `--upgrade-catalog`, or abandoned explicitly; abandoning leaves the
     added columns, unused, and the old digest active.
- **Tests:** kill the upgrade at each state and in the middle of a DDL batch
  and of a backfill batch: the next run resumes and ends with the same
  database as an uninterrupted upgrade; two upgrades at once: one waits or
  is refused; a sweep during a migration is refused; a database crash
  during activation leaves either the old or the new catalog active, never
  a mix.
- **Budget:** upgrade time per 100,000 rows per added column; lock wait.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_migrate -- --ignored`.
- **Done when:** every kill point resumes to the same result.
- **Size:** about 360 added. Two changes: (1) journal and lock, (2)
  resumable backfill and activation.
- **Watch out:** V24 still holds for the first release (old databases are
  re-collected); this is for upgrades after it. Creating many tables in one
  transaction can exhaust the lock table; keep the batch bound.

## C2 Nested fields

### C2.1 Measure the cost of a JSONB column for nested fields

- **Goal:** V40 forbids JSON on the hot path unless a measurement shows it is
  acceptable. Measure it before building on it.
- **Depends on:** C1.4, B5.2.
- **Files:** `crates/core/src/record/json_text.rs` (new),
  `crates/engine/src/inventory/postgres/commit/arrays.rs`,
  `crates/sweep/src/lib.rs` (a measurement switch).
- **Change:**
  1. `json_text.rs`: `pub fn nested(record: MapRef<'_>, names: &impl
     FieldNames, out: &mut String) -> Result<(), RecordError>` writes one JSON
     object holding only the record's top-level members that are lists or
     maps, and structures that contain them, with member names converted by
     the C1.1 word rule. It writes text directly from the CBOR view: no
     `serde_json`, no intermediate tree, bounded by `MAX_RECORD_BYTES` times 2.
     Blobs are base64, timestamps RFC 3339, non-finite floats `null`.
  2. Behind `SweepConfig.nested: NestedMode { Off, Json }` (new), the writer
     fills the `nested jsonb` column.
  3. Measure on `thin` and `full`, cold: P1, P6, P8 and P9 with `Off` and with
     `Json`.
- **Tests:** a record with a list of structures, a map, a nested list, a blob
  and a non-finite float renders to JSON that `SELECT nested->'...'` reads
  back; scalar-only record gives NULL, not `{}`; output is deterministic.
- **Budget:** the rows are the evidence.
- **Prove it:** `cmdb-bench sweep --profile thin` twice (flag off and on).
- **Done when:** the two rows are recorded.
- **Size:** about 280 added. One change.
- **Watch out:** `inventory-core` with default features off must not gain
  `serde_json` (ROADMAP M3.5 checks `cargo tree`). Producing JSON at commit is
  exactly what V03 and V40 guard against; this step exists to put a number on
  it, and its code is removed if C2.2 decides against it.

### C2.2 Decide and finish nested fields

- **Goal:** Apply the decision rule to C2.1's rows and make the chosen
  behavior the default.
- **Depends on:** C2.1.
- **Files:** `crates/sweep/src/lib.rs`, `src/cli/sweep.rs`.
- **Change:**
  1. Proposed rule, to be confirmed by the coordinator with the rows in hand:
     keep `Json` as the default if cold wall time (P1) rises by no more than
     10% and writer rows per second (P9) falls by no more than 15% on `full`.
     Otherwise the default is `Off`, the column stays NULL, and `--nested
     json` turns it on.
  2. A GIN index on `nested` is added only if a later measurement asks for it;
     not here.
- **Tests:** a query on a list field (a security group's rules:
  `nested->'ip_permissions'`) returns the seeded value; with `Off` the column
  is NULL and the typed columns are unaffected.
- **Budget:** P1 and P9 as in the rule.
- **Prove it:** `aws-inventory sql --database $URL "select
  jsonb_array_length(nested->'ip_permissions') from aws_ec2_security_group
  limit 1"` after an e2e run.
- **Done when:** O13 is settled with the rows; the coordinator records it in
  DECISIONS.
- **Size:** about 60 added. One change.
- **Watch out:** the record blob stays CBOR and stays the source for the UI's
  detail view either way.

## C3 The `sql` command

### C3.1 The statement policy

- **Goal:** Reuse the read-only statement policy of
  `crates/engine/src/store/sql_policy.rs` for schema 14, in a module the CLI
  and `cmdb-view` can call.
- **Depends on:** C5.1.
- **Files:** `crates/engine/src/store/sql_policy.rs` (155 lines; the schema 13
  `query` command keeps using it until K2.1), `crates/engine/src/sql.rs`
  (new), `crates/engine/src/sql/policy.rs` (new),
  `crates/engine/src/sql/bounds.rs` (new), `crates/engine/src/lib.rs`.
- **Change:**
  1. Move the lexer (`SqlLexState`, `lex_normal`, `close_quote`,
     `validate_query_prefix`) into `sql/policy.rs` and make
     `store/sql_policy.rs` call it, so there is one copy.
  2. `pub fn validate(sql: &str) -> Result<(), SqlError>` for PostgreSQL:
     non-empty, at most `MAX_SQL_BYTES`, no NUL, exactly one statement (no
     second statement after a `;` outside quotes and comments), first keyword
     one of `SELECT`, `WITH`, `VALUES`, `EXPLAIN`, `TABLE`, `SHOW`. The lexer
     gains dollar-quoted strings and `E'...'` escapes when looking for `;`;
     the old caller keeps its current behavior through a flag.
  3. `bounds.rs`: `pub struct Bounds { pub rows: usize, pub bytes: usize, pub
     cells: usize, pub columns: usize, pub millis: u64 }` with the old
     constants as defaults (`MAX_QUERY_ROWS`, `MAX_QUERY_BYTES`,
     `MAX_QUERY_CELLS`, `MAX_QUERY_COLUMNS`), and a `Budget` that the row loop
     charges, returning the distinct error `SqlError::ResultTooLarge { bound:
     &'static str }`.
- **Tests:** the existing `crates/engine/tests/query_lexical_contract.rs`
  cases pass through the moved code; new cases for dollar quoting, `E''`
  strings, a `;` inside each quote form, two statements, a comment hiding a
  second statement, and each first keyword.
- **Prove it:** `cargo test -p inventory-engine --locked sql
  query_lexical_contract`.
- **Done when:** one lexer serves the old command and the new one.
- **Size:** about 220 added, 120 moved. One change.
- **Watch out:** the lexer is a guard, not the security boundary. The
  boundary is the read-only transaction and the reader role (C3.2, C4.1).

### C3.2 `aws-inventory sql`

- **Goal:** `aws-inventory sql --database <URL> "<statement>"` returns typed
  rows from PostgreSQL, read-only, bounded, while a sweep writes.
- **Depends on:** C3.1, C6.4, C1.4.
- **Files:** `src/cli.rs` (new `Sql` subcommand; the old `Query` stays for
  schema 13 until K2.1), `src/sql_command.rs` (new),
  `src/sql_command/table.rs` (new), `crates/engine/src/sql/run.rs` (new),
  `crates/engine/src/sql/value.rs` (new).
- **Change:**
  1. A dedicated connection; `BEGIN TRANSACTION READ ONLY`; `SET LOCAL
     statement_timeout` from `Bounds::millis`; `SET LOCAL search_path =
     inventory`; run the statement with the extended protocol (`sqlx::query`),
     which the server limits to one command; stream rows and charge the
     budget; `ROLLBACK`.
  2. `value.rs`: `pub enum SqlValue { Null, Bool(bool), Int(i64), Real(f64),
     Text(String), Bytes(Vec<u8>), Timestamp { seconds: i64, micros: u32 },
     Json(String) }`. Decoding by type name: `bool`, `int2`, `int4`, `int8`,
     `float4`, `float8`, `text`, `varchar`, `name`, `bytea`; `timestamptz`
     from its binary form (microseconds since 2000-01-01); `jsonb` from its
     binary form (version byte and text). Any other type returns
     `SqlError::UnsupportedType { column, type_name }` telling the user to
     cast to `text`.
  3. Output: an aligned text table by default, bytes as hex with a length
     cap, timestamps as RFC 3339 in UTC. `--limit <rows>` lowers the row
     bound.
- **Tests:** typed values of every `SqlValue` case; `INSERT`, `UPDATE`,
  `DELETE`, `CREATE`, a writable CTE and `COPY ... TO PROGRAM` are refused (by
  the lexer or by the server; assert each is refused, whichever does it); a
  result over the row bound ends with `ResultTooLarge`; a `pg_sleep(60)` ends
  at the time bound; a query sees committed pages while a sweep is running
  (start a sweep through the proxy with latency, query during it).
- **Budget:** P12: latency of `SELECT count(*) FROM resources` and of a
  filtered typed-column query at 250,000 resources. P1 of a sweep running at
  the same time must not rise beyond the band (readers must not block the
  writer).
- **Prove it:** after an e2e run: `aws-inventory sql --database $URL "select
  cidr_block from aws_ec2_vpc order by 1"` prints the seeded blocks.
- **Done when:** X7's CLI part holds on the thin slice.
- **Size:** about 400 added. Two changes: (1) engine `sql` module, (2) CLI and
  table output.
- **Watch out:** the compose file's user owns the database, and for a
  superuser or owner a read-only transaction still allows some side effects
  (`pg_terminate_backend`, `set_config`). Tests must run the command as the
  reader role from C4.1. No `chrono` or `time` feature of `sqlx` is added;
  that is why two types are decoded by hand.
- **Review fix (2026-11-18):** the `sql` command connects as the reader,
  not with whatever URL it is given. `ReaderUrl::establish(url)` lives in
  this step's `sql` module and is reused by the UI (D13.1): it connects,
  asks the server whether the session can write to the inventory schema,
  and refuses if it can or if the check fails. The read-only transaction and
  the statement policy stay as a second lock.

### C3.3 Explicit exports and the old command

- **Goal:** CSV and JSON as explicit exports, and a clear path for the schema
  13 `query` command.
- **Depends on:** C3.2.
- **Files:** `src/sql_command.rs`, `src/sql_command/export.rs` (new),
  `src/cli.rs`.
- **Change:** `--format table|csv|json` (default `table`). CSV: RFC 4180
  quoting, NULL as empty, bytes as hex. JSON: one object per line, written by
  hand from `SqlValue` (no `serde_json` in this path). The old `Query`
  subcommand prints a notice on stderr that it reads the schema 13 store only
  and is removed by K2.1.
- **Tests:** CSV quoting of commas, quotes and newlines; JSON escaping of
  control characters and non-finite floats (as `null`); both formats respect
  the bounds.
- **Prove it:** `aws-inventory sql --format csv ...` and `--format json ...`
  after an e2e run; `tests/cli_properties.rs` still passes.
- **Done when:** the three formats exist.
- **Size:** about 180 added. One change.
- **Watch out:** JSON here is an explicit export, which V03 allows. It must
  not be the default.

## C4 External clients

### C4.1 `aws-inventory schema`, and a reader role

- **Goal:** List tables and columns from the store's own registry, and give
  external clients a role that can only read.
- **Depends on:** C3.2.
- **Files:** `src/cli.rs`, `src/schema_command.rs` (new),
  `crates/engine/src/inventory/postgres/roles.rs` (new),
  `crates/engine/src/inventory/postgres/schema.rs`.
- **Change:**
  1. `aws-inventory schema --database <URL> [--table <name>]`: from `kinds`
     and `kind_schemas`, print kind, table, and per column its name, SQL type,
     and member path (field names from the catalog). Also list the cross-kind
     tables `resources`, `edges`, `tags`, `scope_outcomes` with their columns.
     Members cut by C1.2 are listed as `record only`.
  2. Store creation also runs: `CREATE ROLE inventory_reader NOLOGIN` if
     absent, `GRANT USAGE ON SCHEMA inventory`, `GRANT SELECT ON ALL TABLES IN
     SCHEMA inventory`, `ALTER DEFAULT PRIVILEGES IN SCHEMA inventory GRANT
     SELECT ON TABLES`, `REVOKE` on `work_queue`, `work_tokens` and
     `projection_outbox`, and `ALTER ROLE inventory_reader SET
     default_transaction_read_only = on`. A login role for a person or a
     desktop client is created by the operator and given `inventory_reader`.
- **Tests:** `schema` output equals the registry; a session with only
  `inventory_reader` can select from `resources` and a kind table and cannot
  insert, update, delete, or read `work_queue`.
- **Prove it:** `aws-inventory schema --database $URL --table aws_ec2_vpc`.
- **Done when:** the command and the role exist.
- **Size:** about 240 added. One change.
- **Watch out:** creating a role needs the `CREATEROLE` privilege; if the
  store's user lacks it, skip with a printed notice and do not fail store
  creation. Roles are cluster-wide: a `TestDatabase` must tolerate the role
  already existing.
- **Review fix (2026-11-18):** provision two roles, not one, and before the
  console exists: `inventory_reader` (`SELECT` only,
  `default_transaction_read_only = on`, a `statement_timeout`) and
  `inventory_collector` (read and write in the inventory schema, no
  superuser, no role creation). The collector's URL and the reader's URL are
  distinct settings everywhere (`aws-inventory sweep` takes the collector's;
  `aws-inventory sql`, the UI's reads and external clients take the
  reader's). Tests: the reader cannot write, create, or read outside the
  schema; the collector cannot create a role. Who owns the schema in
  production (a third, owner role) is part of the release gate.

### C4.2 `psql` and a desktop client, documented and proven (X7)

- **Goal:** The documented external command returns the same rows as
  `aws-inventory sql` while a sweep runs.
- **Depends on:** C4.1.
- **Files:** `docs/SQL.md` (new), `tests/sweep_e2e/external_clients.rs`
  (new), `tests/sweep_e2e.rs`.
- **Change:**
  1. `docs/SQL.md`: how to connect with `psql` and with a desktop client
     (host, port, database, a login role that is a member of
     `inventory_reader`, `search_path`); the documented surface: kind tables
     with their fixed columns, `resources`, `edges`, `tags`, `scope_outcomes`;
     what is not promised (internal tables, column order, Steampipe or
     CloudQuery names); timestamp precision (microseconds in columns, full
     value in the record); the `nested` column.
  2. Test: create a login role in the test database's cluster, grant it
     `inventory_reader`; during a sweep slowed by the proxy, run the same
     statement through `aws-inventory sql --format csv` and through `psql -At
     -F, -c` as that role; compare the rows; then assert that the role cannot
     write.
- **Tests:** as above; fails with a message naming `psql` when it is not on
  `PATH`.
- **Prove it:** `cargo test --locked --features sweep --test sweep_e2e -j1 --
  --ignored --test-threads 1 external_clients`.
- **Done when:** X7's external-client part holds with `psql`.
- **Size:** about 200 added (test) and the document. One change.
- **Watch out:** `psql` 16 is on `PATH` on this machine. A desktop client is
  documented by its connection settings only; none is run by a test.
