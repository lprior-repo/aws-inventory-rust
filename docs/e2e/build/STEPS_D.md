# Track D steps: sweep library and UI

> Status: Plan, 2026-11-15, revised for V53 (PostgreSQL is the only store; the
> UI, the whole-account load and the benchmark generator read and write
> PostgreSQL only). Step-by-step breakdown of the packages in
> [D_LIBRARY_UI.md](../D_LIBRARY_UI.md). D1 comes first in the whole plan
> after A0, because it owns `src/sweep_runtime/` and every speed step lands in
> the new crate after it. Nothing here has been built. Revised 2026-11-18: the
> whole app is outlined in [MAKEPAD_UI.md](../MAKEPAD_UI.md), the pinned
> Makepad source (`506ec05`) was read for the draw and timing steps (D6, D7),
> packages D9 to D14 are new, and the findings of the 2026-11-18 review are
> applied (reader and collector roles, the X1 contract, presented-frame
> timing). "Makepad:" paths are in the pinned checkout.

Conventions: as in [STEPS_A.md](STEPS_A.md). `crates/cmdb-ui/src` and
`crates/cmdb-view/src` are under the under-60-line function gate. This machine has
no display, so UI steps name what is proven headless (worker and layout tests)
and what needs a machine with a GPU (frame times).

## D1 Sweep as a library

### D1.1 Split the `aws` feature so the sweep builds without the SDK crates

- **Goal:** The sweep needs only the profile loader
  (`crates/aws/src/model_credentials.rs`, 212 lines), but the `aws` feature
  links all of `inventory-aws`, including 19 typed `aws-sdk-*` crates. Building
  `aws-sdk-ec2` was killed twice at this machine's 6 GiB limit. Give the sweep
  its own feature.
- **Depends on:** A0.2.
- **Files:** `crates/aws/Cargo.toml`, `crates/aws/src/lib.rs`, `Cargo.toml`
  (root `[features]` and the `inventory-aws` dependency),
  `src/sweep_command.rs`, `src/main.rs` (any `cfg(feature = "aws")` on the
  sweep path), `tests/sweep_e2e.rs`, `scripts/e2e-baseline.sh`,
  `docs/e2e/RUN_HERE.md`.
- **Change:**
  1. `crates/aws/Cargo.toml`: add `[features] default = ["adapters"]`,
     `adapters = [...]` listing as `dep:` every `aws-sdk-*` crate (sts, ec2,
     dynamodb, lambda, ecs, eks, config, resourceexplorer2,
     elasticloadbalancingv2, cloudcontrol, s3, iam, route53, cloudfront,
     organizations, sqs, sns, kms, sfn), `aws-smithy-types`,
     `percent-encoding`, `serde_json`, `inventory-engine` and whatever else
     only the adapters use. Mark those dependencies `optional = true`.
     `aws-config`, `aws-types`, `aws-credential-types`, `aws-wire` and
     `thiserror` stay unconditional.
  2. `crates/aws/src/lib.rs`: `pub mod model_credentials;` stays
     unconditional; every other module, `mod scan`, `pub use scan::scan` and
     the test modules get `#[cfg(feature = "adapters")]`.
  3. Root `Cargo.toml`: `inventory-aws = { path = "crates/aws", optional =
     true, default-features = false }`; `[features] sweep =
     ["dep:inventory-aws"]`, `aws = ["sweep", "inventory-aws/adapters"]`.
  4. `src/sweep_command.rs`: `cfg(feature = "aws")` becomes `cfg(feature =
     "sweep")`. The `scan` command and everything else that needs adapters
     keeps `cfg(feature = "aws")`.
  5. `tests/sweep_e2e.rs` starts with `#![cfg(feature = "aws")]`; it becomes
     `feature = "sweep"`. The baseline script uses `--features sweep`.
- **Tests:** existing tests under both feature sets; a root test using
  `cargo metadata` asserts that with only `sweep` the resolved normal
  dependencies contain no `aws-sdk-` crate other than those `aws-config`
  itself requires (expected: `aws-sdk-sts`, `aws-sdk-sso`, `aws-sdk-ssooidc`;
  the exact set is not verified).
- **Budget:** build cost, recorded before and after: peak compiler memory and
  wall time of `cargo test --locked --features <f> --test sweep_e2e --no-run
  -j1`, and the size of the debug `aws-inventory` binary.
- **Prove it:** `cargo test --locked --features sweep --test sweep_e2e -j1 --
  --ignored --test-threads 1` gives 2 passed, built within 6 GiB without swap;
  `cargo check --workspace --all-targets --all-features --locked -j1` still
  passes on a machine with enough memory.
- **Done when:** the sweep builds and passes without `aws-sdk-ec2` being
  compiled, and the two build numbers are recorded.
- **Size:** about 60 lines of TOML and attributes. One change.
- **Watch out:** `cargo check --all-features` still builds everything, so the
  full gate still needs more than 6 GiB. Do not delete any adapter here; that
  is K1. `Cargo.lock` does not change (no crate is added or removed).

### D1.2 Move the sweep runtime into `crates/sweep`

- **Goal:** A library crate `inventory-sweep` holding everything in
  `src/sweep_runtime/` and the collection half of `src/sweep_command/`.
- **Depends on:** D1.1, C5.1.
- **Files:** `crates/sweep/Cargo.toml` (new), `crates/sweep/src/lib.rs` (new;
  from `src/sweep_runtime.rs`), `crates/sweep/src/{call,config,execution,
  input,record,request}.rs` and their subdirectories (moved from
  `src/sweep_runtime/`, 23 files, 2,780 lines with `src/sweep_runtime.rs`),
  `crates/sweep/src/credentials.rs` (new), `Cargo.toml` (workspace member and
  root dependency), `src/sweep_runtime.rs` and `src/sweep_runtime/` (deleted),
  `src/sweep_command/enabled.rs`, `crates/aws/src/model_credentials.rs`.
- **Change:**
  1. Move the files unchanged except for paths: `crate::sweep_runtime::` and
     `super::` references that crossed the old module root become `crate::`.
  2. The library must not depend on `aws-config`. Add `pub trait
     CredentialSource: Send + Sync { fn region(&self) -> &str; fn
     resolve(&self) -> impl Future<Output = Result<aws_wire::sign::Credentials,
     CredentialFailure>> + Send; }` and `pub struct CredentialFailure { pub
     retryable: bool, pub message: String }` in `credentials.rs`. `Call`,
     `BootstrapContext` and `run` take `&C where C: CredentialSource` in place
     of `&ProfileCredentials`. `call.rs::credential_error` maps `retryable` to
     `ApiError::Transient`, otherwise `Permanent`.
  3. `inventory-aws` implements `CredentialSource` for `ProfileCredentials`
     behind a new feature `sweep-credentials` that depends on
     `inventory-sweep` (so the dependency points from the AWS crate to the
     library, not back).
  4. `SweepError::Credentials(#[from] CredentialError)` becomes
     `Credentials(String)`.
  5. Public surface of the crate: `run`, `SweepConfig`, `SweepSummary`,
     `ScopeOutcomes`, `SweepError`, `CredentialSource`, `CredentialFailure`.
     Everything else stays `pub(crate)` or narrower.
  6. `collect` in `src/sweep_command/enabled.rs` keeps reading the catalog,
     opening the store and loading credentials, then calls
     `inventory_sweep::run`.
- **Tests:** the moved unit tests (cursor, scalar, identity, child, config)
  run in the new crate; the e2e suite passes unchanged.
- **Budget:** P1 and P6 on `thin` must not change beyond the band once B5.2
  exists; before that, the e2e suite's wall time is noted.
- **Prove it:** `cargo test -p inventory-sweep --locked`; `cargo test --locked
  --features sweep --test sweep_e2e -j1 -- --ignored --test-threads 1`;
  `ls src/sweep_runtime` fails.
- **Done when:** no sweep logic is left in `src/` and the Rust lines added and
  deleted are stated (expected: about 2,780 moved, under 150 new).
- **Size:** about 150 added, 2,780 moved. One change, reviewed as a move.
- **Watch out:** from here on `src/sweep_runtime/` does not exist; every later
  step names `crates/sweep/src/`. The library still reads `Ctrl-C`
  (`tokio::signal::ctrl_c` in `execution.rs::run`); D1.3 removes that. The
  crate needs `uuid` for `sweep_id`.

### D1.3 Cancellation and progress events

- **Goal:** One entry that takes a configuration, a cancellation handle and a
  bounded channel of progress events, so a caller other than the CLI can run
  and stop a sweep.
- **Depends on:** D1.2.
- **Files:** `crates/sweep/src/lib.rs`, `crates/sweep/src/control.rs` (new),
  `crates/sweep/src/execution.rs`, `crates/sweep/src/execution/draining.rs`,
  `crates/sweep/src/execution/page.rs`, `src/sweep_command/enabled.rs`.
- **Change:**
  1. `pub struct SweepControl { pub cancel: CancellationToken, pub progress:
     Option<tokio::sync::mpsc::Sender<ProgressEvent>> }`.
  2. `pub enum ProgressEvent { Started { sweep: String, account: String,
     regions: u32, root_scopes: u64 }, Scope { outcome: ScopeOutcome },
     Page { rows: u64, resource_writes: u64 }, Queue { pending: u64,
     in_flight: u32 }, Finished { summary: SweepSummary } }`.
  3. Events are sent with `try_send`. A full channel drops the event and
     increments `SweepSummary.metrics`-side counter `progress_dropped`; the
     sweep never waits on its listener.
  4. `run(catalog, store, config, credentials, control)`. Remove
     `tokio::signal::ctrl_c` from the library: `run` selects on
     `control.cancel.cancelled()` and calls `executor.cancel()`, returning
     `SweepError::Api(ApiError::Cancelled)` as today.
  5. The CLI creates the token, spawns nothing, and in its own `select!`
     cancels the token on `Ctrl-C`. It passes `progress: None`.
  6. A cancelled sweep leaves its queue rows in the store; `--resume`
     continues it, as after a kill.
- **Tests:** with a fake store and a local test server: events arrive in
  order (`Started` first, `Finished` last); a channel of capacity 1 that is
  never read does not slow the sweep and reports drops; cancelling mid-sweep
  returns `Cancelled` and a resume completes with the same final store digest
  as an uninterrupted run.
- **Budget:** P1 and P6 on `thin` with `progress: None` must not change beyond
  the band; with a listener, record the difference.
- **Prove it:** `cargo test -p inventory-sweep --locked control`; e2e suite
  (its resume-after-kill case still passes).
- **Done when:** the CLI holds no sweep logic beyond argument mapping, signal
  handling and printing.
- **Size:** about 240 added, 30 deleted. One change.
- **Watch out:** every channel is bounded; no task is detached. `Queue`
  events must not cost a store query per page: derive `in_flight` from the
  scheduler (B3.2) and send `pending` only when a claim returns.

### D1.4 Gate the new crate, and state the numbers

- **Goal:** Bring the sweep crate under the function-size gate and close D1.
- **Depends on:** D1.3.
- **Files:** `scripts/check-function-size.sh` (add `crates/sweep/src` to
  `paths`), `crates/sweep/src/execution.rs`, `execution/draining.rs`,
  `execution/page.rs`, `call.rs`, `docs/e2e/D_LIBRARY_UI.md` (mark D1 done),
  `START_HERE.md` and `docs/ARCHITECTURE.md` where they place the sweep in the
  binary.
- **Change:** the gate does not cover `src/sweep_runtime/` today, and several
  functions there are long by reading (the limit is under 60 lines): `execution.rs::run`,
  `bootstrap_account`, `discover_regions`, `lease_budget`;
  `draining.rs::drain_work`; `call.rs::sign_and_send`; `page.rs::process_lease`
  (the exact list is whatever the script prints). Split each into named steps
  with no behavior change.
- **Tests:** existing.
- **Budget:** P1 and P6 on `thin` must not change beyond the band (the splits
  change no behavior).
- **Prove it:** `scripts/check-function-size.sh` exits 0 with the new path;
  e2e suite.
- **Done when:** the gate covers the crate, and the D1 report states Rust
  lines added and deleted for D1.1 to D1.4.
- **Size:** about 120 added, 100 deleted. One change.
- **Watch out:** B3.2 rewrites `drain_work`; if B3.2 is next for the same
  worker, split only what B3.2 does not replace.

## D2 Sweep control in the UI

### D2.1 Worker commands for start, cancel and resume

- **Goal:** The UI's background worker can run a sweep through the library on
  its own runtime.
- **Depends on:** D1.3, C6.11, C6.12, D13.1.
- **Files:** `crates/cmdb-ui/Cargo.toml` (dependencies on `inventory-sweep`,
  `inventory-engine`, `inventory-aws` with `sweep-credentials`, `aws-wire`),
  `crates/cmdb-ui/src/worker/protocol.rs` (74 lines), `worker.rs` (196
  lines), `worker/runner.rs`, `worker/lifecycle.rs`,
  `crates/cmdb-ui/src/worker/sweep.rs` (new), `worker/tests.rs` (split per
  A0.4).
- **Change:**
  1. `Command` gains `StartSweep { request: SweepRequest }`, `CancelSweep`.
     `SweepRequest { profile: Option<String>, region: Option<String>, resume:
     bool, endpoint: Option<String>, ca_bundle: Option<PathBuf>, max_rps:
     Option<u32> }`. `WorkerConfig` gains `catalog: Option<PathBuf>` (flag
     `--catalog`, default `catalog/aws-api.catalog`) and the graph settings it
     already has are reused for projection.
  2. `WorkerEvent` gains `Sweep(SweepStatus)` where `SweepStatus` is `Idle`,
     `Running { started: ..., scopes_done: u64, rows: u64, writes: u64,
     outcomes: ScopeOutcomes, pending: u64, in_flight: u32 }`, `Finished {
     summary }`, `Failed { message }`, `Cancelled`.
  3. `worker/sweep.rs`: on `StartSweep`, refuse if one is running; refuse if
     no collector URL is configured (D13.1); open (or create) a `PgStore` on
     the collector URL, never on the reader's, load credentials,
     create a `SweepControl` with a progress channel of capacity 256, and run
     `inventory_sweep::run` as a future owned by the runner (stored in its
     state and polled in its loop, or a `JoinHandle` the runner aborts on
     `Stop`). Fold progress events into one `SweepStatus` and emit it at most
     ten times per second. After the sweep, run the projection loop the CLI
     runs when a graph URL is configured.
  4. `protocol.rs::current_events` keeps only the newest graph and the newest
     page; extend it to keep the newest `Sweep` status as well, and never to
     drop `Finished`, `Failed` or `Cancelled`.
  5. The UI process opens no listening socket (unchanged).
- **Tests:** headless worker tests against the emulator and a `TestDatabase`:
  `StartSweep` ends in `Finished` and the counts in the last status equal
  `SELECT count(*) FROM resources`; a second `StartSweep` while running is
  refused; `CancelSweep` ends in `Cancelled` and a `StartSweep { resume: true
  }` completes; `Stop` during a sweep ends the worker with no task left.
- **Budget:** P1 of a sweep started from the worker against the same sweep
  from the CLI, on `thin`; P14 frame time while a sweep runs must stay within
  the band of an idle UI (the sweep runs off the UI thread).
- **Prove it:** `cargo test -p cmdb-ui --locked -- --ignored worker_sweep`.
- **Done when:** the worker can sweep, cancel and resume.
- **Size:** about 380 added. Two changes: (1) protocol and status folding,
  (2) the sweep task.
- **Watch out:** the UI crate now links the collector. That direction is
  allowed (the app depends on the collector), never the reverse. The worker holds
  two pools on one server under two roles: the collector's `PgStore` pool and
  `cmdb-view`'s reader pool; keep their sum well under the server's
  `max_connections`. No code path may hand the collector pool to a read
  query or to the console (review finding 2); a test asserts the console and
  table tasks hold only the reader pool.

### D2.2 The progress panel, and the UI half of X1

- **Goal:** One UI action starts a sweep of the emulator and shows its
  progress.
- **Depends on:** D2.1.
- **Files:** `crates/cmdb-ui/src/app.rs`, `app/actions.rs`, `ui.rs` (278
  lines), `chrome.rs`, `templates.rs`, `crates/cmdb-ui/src/sweep_panel.rs`
  (new).
- **Change:** a panel with a "Sweep" button, a "Cancel" button while running,
  a "Resume" button when the store has a pending sweep, and lines for scopes
  done and pending, rows, writes, and the six outcome counts. On `Finished`
  the table and counts refresh (`DataWorker::refresh`). Errors show the
  message from `Failed`.
- **Tests:** state tests for the panel model (which buttons are enabled in
  which status); the worker-level test of D2.1 is the functional proof on a
  machine without a display.
- **Budget:** P14: frame time with the panel visible during a sweep.
- **Prove it:** on a machine with a display and a PostgreSQL server up (C6.2):
  seed an emulator and keep it running with `inventory-harness e2e run --keep`,
  start `cmdb-ui
  --database <URL> --catalog ...`, press "Sweep"; the counts shown equal `psql
  -c "select count(*) from inventory.resources"`. Record the session.
- **Done when:** X1's second sentence holds, with the headless test in the
  gate and the manual run recorded.
- **Size:** about 300 added. One change.
- **Watch out:** `app.rs` is at the size limit (A0.4). Seeding the emulator is
  not a UI action: the product has no write capability. The contract, now
  stated in README X1: the CLI's one command covers seeding, sweeping and
  projecting through the harness; the UI's one action covers the sweep and
  the projection against an account that already exists (an emulator the
  harness seeded, or a real one). The UI never seeds.

## D3 SQL console in the UI

### D3.1 `cmdb-view` runs one bounded read-only statement

- **Goal:** The same statement returns the same rows in the CLI and the UI,
  because both call the same code.
- **Depends on:** C3.2, C6.11, C4.1, D13.1.
- **Files:** `crates/cmdb-view/Cargo.toml` (dependency on `inventory-engine`),
  `crates/cmdb-view/src/sql.rs` (new), `crates/cmdb-view/src/lib.rs`.
- **Change:** `pub async fn run_sql(reader: &ReaderUrl, statement: &str,
  bounds: Bounds) -> Result<SqlResult, ViewError>` calling
  `inventory_engine::sql` (C3.1, C3.2). `ReaderUrl` (C3.2) can only be made
  from a connection the server reported as unable to write, so this function
  cannot be called with the collector's URL. `SqlResult { columns: Vec<String>, rows: Vec<Vec<SqlValue>>,
  truncated: Option<&'static str> }`. Re-export `Bounds` and `SqlValue`.
- **Tests:** a statement run through `cmdb-view` and through the CLI's
  function returns equal values; a write is refused by the server's role and
  not only by the statement policy (connect as the reader, send `CREATE
  TABLE`, expect a permission error); a `ReaderUrl` cannot be built from the
  collector's URL; the bound is reported in `truncated`.
- **Budget:** build cost: `cmdb-ui` clean build time before and after
  `cmdb-view` depends on the engine.
- **Prove it:** `cargo test -p cmdb-view --locked sql -- --ignored`.
- **Done when:** one function serves both callers.
- **Size:** about 100 added. One change.
- **Watch out:** this is the first dependency of `cmdb-view` on
  `inventory-engine`. Until K2.1 the engine still carries the schema 13 store
  (`crates/engine/src/store/`) and its dependencies, so `cargo tree -p
  cmdb-view` grows by them from here until K2.1 (C6.12 and K2.1 name the
  crates). If that or the build cost is unwelcome, move
  `inventory_engine::sql` into its own small crate and depend on that instead.

### D3.2 The console widget

- **Goal:** A text input that runs one statement and shows the rows.
- **Depends on:** D3.1.
- **Files:** `crates/cmdb-ui/src/worker/protocol.rs`, `worker/queries.rs`,
  `crates/cmdb-ui/src/console.rs` (new), `app.rs`, `ui.rs`.
- **Change:** `Command::Sql { revision, statement }`; `WorkerEvent::Sql {
  result: Result<SqlResult, String> }`, newest revision wins. The widget shows
  the column headers and up to the row bound in the existing portal list, the
  truncation notice, and the error text. Values render as in the CLI table
  (C3.2).
- **Tests:** worker test: the statement of X7 returns the seeded value; a
  stale revision's result is dropped; the model truncates long cells for
  display without changing the copied value.
- **Budget:** P14 with a 1,000-row result shown; P12 for the statement itself
  is C3.2's.
- **Prove it:** `cargo test -p cmdb-ui --locked -- --ignored worker_sql`; on a
  display, type the X7 statement and compare with the CLI.
- **Done when:** X7's UI part holds.
- **Size:** about 280 added. One change.
- **Watch out:** the result set lives in the worker event; bound it by the
  same `Bounds` so a large result cannot grow the UI's memory.

## D4 Whole-account load from SQL

### D4.1 `cmdb-bench generate` through the real store, with edges

- **Goal:** A benchmark database of 250,000 resources and 1,000,000 edges
  created by the real store code (`PgStore`). It lands before C6.12, which
  deletes the existing generator in `crates/cmdb-bench/src/main.rs`.
- **Depends on:** C6.8 (commits with references and edges).
- **Files:** `crates/cmdb-bench/src/main.rs` (or `src/generate.rs` if B5.2's
  split has landed), `crates/cmdb-bench/src/generate_pg.rs` (new),
  `crates/cmdb-bench/Cargo.toml` (dependency on `inventory-engine`).
- **Change:** `cmdb-bench generate <postgres URL> <accounts>
  <resources-per-account> [--edges-per-resource <n>] [--catalog <file>]`
  creates the store with `PgStore::create` from the committed catalog, then
  writes through `seed_works`, `claim` and `commit_page`: resources spread
  over the catalog's kinds and 17 regions with deterministic records, and
  edges between them carried in `PageCommit.edges`. Bound: 10,000,000
  resources (`MAX_RESOURCES` today). A file-path argument still reaches the
  existing generator (`execute_schema` and its two hand-written tables,
  `ec2_instances` and `iam_roles`), untouched, until C6.12 removes it.
- **Tests:** a small generate into a `TestDatabase`, then `measure`: counts
  equal the arguments; rerun with the same arguments gives the same content
  digest.
- **Budget:** generation time and database size at 250,000 and 1,000,000
  resources (this is also a write-path measurement: P9).
- **Prove it:** `cmdb-bench generate postgres://... 10 25000
  --edges-per-resource 4` then `cmdb-bench measure postgres://...`.
- **Done when:** a PostgreSQL benchmark database can be generated and
  measured.
- **Size:** about 240 added. One change.
- **Watch out:** `PageCommit.edges` requires both ends to exist (foreign
  keys); generate resources first, edges in later pages. Regenerate after
  C1.4, when the kind tables gain their typed columns.

### D4.2 Stream the whole account into compact arrays

- **Goal:** `cmdb-view` loads every node and every edge into flat arrays with
  identities interned to integers.
- **Depends on:** C6.11.
- **Files:** `crates/cmdb-view/src/account.rs` (new), `account/intern.rs`
  (new), `account/load.rs` (new), `database/postgres.rs`, `lib.rs`.
- **Change:**
  1. `pub struct AccountGraph { pub accounts: Vec<Box<str>>, pub regions:
     Vec<Box<str>>, pub kinds: Vec<Box<str>>, pub relations: Vec<Box<str>>,
     pub node_account: Vec<u16>, pub node_region: Vec<u16>, pub node_kind:
     Vec<u32>, pub node_id: TextArena, pub node_name: TextArena, pub
     edge_from: Vec<u32>, pub edge_to: Vec<u32>, pub edge_relation: Vec<u16> }`
     with `TextArena { bytes: Vec<u8>, offsets: Vec<u32> }`.
  2. `pub async fn load(database: &Database, limits: LoadLimits, progress:
     impl FnMut(LoadProgress)) -> Result<AccountGraph, ViewError>`: inside one
     `REPEATABLE READ` snapshot, page `resources` by key (10,000 rows per
     page), intern, then page `edges` and map both ends to node indices
     through a temporary `HashMap` that is dropped at the end. An edge whose
     end is not loaded is counted in `LoadProgress.dangling`, not stored.
  3. `LoadLimits { max_nodes: u32, max_edges: u32, max_bytes: usize }`,
     defaults 4,000,000, 16,000,000 and 2 GiB; exceeding one stops with
     `ViewError::TooLarge` naming it.
- **Tests:** counts equal SQL counts; a database changed during the load is
  read at the snapshot; limits stop the load; interning returns the same index
  for the same key.
- **Budget:** P12 load time and peak memory at 250,000 nodes and 1,000,000
  edges. Command: `cmdb-bench load <URL>` (new subcommand in
  `crates/cmdb-bench/src/load.rs`).
- **Prove it:** `cmdb-bench load postgres://...` prints nodes, edges, seconds
  and peak RSS; the counts equal `psql` counts.
- **Done when:** the D4 exit evidence is recorded.
- **Size:** about 400 added. One change.
- **Watch out:** `edges` is keyed by ten text columns; a million rows of that
  is most of the transfer. If the row shows the edge read dominating, the
  measured alternatives are a server-side join that returns two row numbers,
  or `COPY ... TO STDOUT (FORMAT binary)`; pick from the rows.

### D4.3 The worker loads the account

- **Goal:** The UI worker performs the load and hands the arrays to the UI
  without copying them.
- **Depends on:** D4.2.
- **Files:** `crates/cmdb-ui/src/worker/protocol.rs`,
  `worker/account_task.rs` (new), `crates/cmdb-ui/src/data/mod.rs`.
- **Change:** `Command::LoadAccount { revision }`; `WorkerEvent::Account {
  graph: Arc<AccountGraph>, progress: LoadProgress }` and intermediate
  `WorkerEvent::AccountProgress`. The existing 500-row table page and the
  FalkorDB neighbourhood stay as they are.
- **Tests:** worker test on the bench database: final counts equal SQL; a
  newer revision cancels an older load.
- **Budget:** P14: the UI thread's longest frame during the load.
- **Prove it:** `cargo test -p cmdb-ui --locked -- --ignored worker_account`.
- **Done when:** the UI holds an `Arc<AccountGraph>` for the bench database.
- **Size:** about 180 added. One change.
- **Watch out:** the graph database is not needed for this (V46 keeps it for
  traversal).

## D5 Layout off the UI thread

### D5.1 A hierarchical layout into flat arrays

- **Goal:** Positions for every node in the hierarchy account, region,
  service, kind, resource, as a pure function.
- **Depends on:** D4.2.
- **Files:** `crates/cmdb-ui/src/dag/hierarchy.rs` (new),
  `dag/hierarchy/pack.rs` (new), `dag/hierarchy/tests.rs` (new).
- **Change:**
  1. Group nodes by `(account, region, service, kind)`, where service is the
     second part of the kind name (`AWS::EC2::VPC` gives `EC2`).
  2. Pack bottom-up: resources of one kind in a square grid of fixed cell
     size; kinds inside a service, services inside a region, regions inside an
     account, each by a row-packing of rectangles sorted by size with a fixed
     aspect target.
  3. Output `pub struct HierarchyLayout { pub node_xy: Vec<[f32; 2]>, pub
     groups: Vec<Group> }`, `Group { level: u8, rect: [f32; 4], first_child:
     u32, child_count: u32, node_count: u32, label: u32 }`.
  4. Deterministic: the same `AccountGraph` gives the same bytes.
- **Tests:** no two node cells overlap; every node is inside its kind's
  rectangle and so on up; an empty graph; one node; 250,000 nodes within the
  time budget below on the test machine.
- **Budget:** P13 layout time at 250,000 nodes. Command: `cmdb-bench layout
  <target>` (new subcommand calling the same function).
- **Prove it:** `cargo test -p cmdb-ui --locked hierarchy`; the P13 row.
- **Done when:** the row is recorded.
- **Size:** about 350 added. One change.
- **Watch out:** `f32` arithmetic is allowed by the lints, but converting a
  count to `f32` with `as` is not; go through `u16`/`u32` to `f32::from` or
  `f64::from` and bound the grid size.

### D5.2 Incremental layout on the worker

- **Goal:** The UI thread never waits on layout for longer than one frame.
- **Depends on:** D5.1, D4.3.
- **Files:** `crates/cmdb-ui/src/worker/layout_task.rs` (new),
  `worker/protocol.rs`, `dag/hierarchy.rs`.
- **Change:** the worker computes the layout after the load, per account,
  yielding between accounts and checking the revision; it emits
  `WorkerEvent::Layout { layout: Arc<HierarchyLayout>, complete: bool }` as
  accounts finish. A reload with the same node set reuses the previous
  layout; added nodes are placed in free cells of their kind's grid without
  moving the others when a cell is free.
- **Tests:** partial layouts are valid layouts of their accounts; a stale
  revision stops; adding one node moves no existing node.
- **Budget:** P13 as time to first partial and time to complete; P14 longest
  UI frame during layout.
- **Prove it:** `cargo test -p cmdb-ui --locked -- --ignored worker_layout`.
- **Done when:** the D5 exit evidence is recorded.
- **Size:** about 220 added. One change.
- **Watch out:** the layout buffers are shared by `Arc`; the renderer must
  not mutate them.

## D6 Instanced renderer

### D6.0 Spike: retained instances at the pinned Makepad revision

- **Goal:** Settle, by running code on a machine with a GPU, the things the
  source could not tell us (MAKEPAD_UI.md section 3), before any renderer
  step depends on them.
- **Depends on:** nothing. It can run in wave 0.
- **Files:** `crates/cmdb-ui/examples/retained_spike.rs` (new),
  `docs/performance-evidence/retained-spike.md` (new).
- **Change:** a standalone example, not wired into the app:
  1. A draw shader with a per-instance position, size and color, and one
     transform uniform.
  2. Build a block of N instances off the UI thread with
     `RetainedInstances::new(slots, Arc<[f32]>)` (Makepad:
     `platform/src/retained_instances.rs`) and bind it each draw with
     `CxDraw::add_retained_instances(&draw_vars, &block)` (Makepad:
     `draw/src/draw_list_2d.rs`).
  3. Change the transform with `Cx::set_draw_item_uniform(list, item, name,
     value)` (Makepad: `platform/src/cx_api.rs`) on drag and wheel, with no
     re-recording of the block.
  4. A second block bound with `add_retained_instances_count(.., 0)` and
     switched on by a key, to prove a level of detail comes back without an
     upload.
  5. Run at N = 50,000, 250,000, 1,000,000 and 4,000,000 on an integrated and
     a discrete GPU; enable `gpu_frame_timer` and `MAKEPAD_TRACE=present`.
  6. Try `MAKEPAD_HAL_BACKEND=headless` and `offscreen`.
- **Tests:** none; this is evidence.
- **Budget:** the first P14 rows for instanced drawing: GPU busy time and
  missed frames at each N, per machine.
- **Prove it:** `cargo run --release -p cmdb-ui --example retained_spike --
  250000` on each machine; the report.
- **Done when:** the report answers, with numbers: how the shader's instance
  slots are declared and read back; how the draw item's list and index are
  obtained; the largest N each machine accepts and holds at its refresh rate;
  what happens past the device's memory envelope; whether the app starts
  headless. If a retained block cannot be drawn at this revision, the report
  says so and names the fallback: `begin_many_instances` with culling to the
  visible groups, which changes D6.1's budget.
- **Size:** about 200 lines, in `examples/`. One change.
- **Watch out:** `add_retained_instances_count` asserts on a stride mismatch
  and on a count above the block's; the spike is where a panic is acceptable.
  No example in the pinned Makepad tree uses retained instances outside
  `draw_list_2d.rs`, so there is nothing to copy. Makepad's own notes say to
  time release builds only.

### D6.1 Nodes as instances, pan and zoom as a transform

- **Goal:** Draw all nodes from a buffer that is uploaded when the layout
  changes, not every frame.
- **Depends on:** D6.0, D5.2, D7.1 (baseline recorded first).
- **Files:** `crates/cmdb-ui/src/dag/widget/mod.rs`,
  `dag/widget/paint.rs` (271 lines), `dag/widget/instances.rs` (new),
  `dag/widget/account_view.rs` (new), `dag/widget/paint_style.rs`,
  `crates/cmdb-ui/src/worker/layout_task.rs` (D5.2),
  `worker/instance_task.rs` (new).
- **Change:**
  1. A draw shader (`DrawNodes`, new) with per-instance position, size, color
     index and state bits, and a transform uniform, declared the way D6.0's
     report found to work.
  2. `pub const NODE_SLOTS: usize` in `instances.rs` is the one place the
     stride is written. `pub fn node_block(layout: &HierarchyLayout, graph:
     &AccountGraph) -> Result<RetainedInstances, InstanceError>` builds the
     block on the worker. `WorkerEvent::Instances { nodes:
     RetainedInstances, .. }` carries it to the UI; nothing is copied there.
  3. Before the first bind, compare `NODE_SLOTS` with the shader's instance
     slots and refuse to draw, with a visible error, if they differ. Makepad
     asserts on a mismatch; our check runs first.
  4. Each draw binds the block with `add_retained_instances`. Pan and zoom
     call `set_draw_item_uniform` for the transform and request a redraw;
     they rebuild nothing.
  5. The whole-account view is a second mode of the graph tab. The existing
     neighbourhood view (`DrawVector`, `GRAPH_LIMITS`) stays as it is.
- **Tests:** `node_block` emits one instance per node with the layout's
  coordinates and `NODE_SLOTS` floats each; a slot mismatch is refused, not
  panicked; a counter proves that a hundred pans rebuild no block.
- **Budget:** P14 at 2,000, 50,000 and 250,000 nodes against D7.1's baseline:
  missed frames, GPU busy p99, UI-thread p99. UI-thread time per frame must
  not grow from 50,000 to 250,000.
- **Prove it:** D7's harness on a machine with a GPU and a 120 Hz display.
- **Done when:** the rows are recorded.
- **Size:** about 360 added. Two changes: (1) block builder and worker event,
  (2) the draw path.
- **Watch out:** if D6.0 found retained blocks unusable, this step uses
  per-frame `begin_many_instances` over the visible groups only, and says so
  in its rows. `mod.rs` and `paint.rs` are near the size limit; the account
  view goes in its own file.

### D6.2 Edges as instances, with aggregation at far zoom

- **Goal:** Draw up to 1,000,000 edges, or their aggregates, within the frame
  budget.
- **Depends on:** D6.1.
- **Files:** `crates/cmdb-ui/src/dag/widget/edges.rs` (new),
  `dag/hierarchy/aggregate.rs` (new), `worker/instance_task.rs`.
- **Change:**
  1. A draw shader (`DrawEdges`, new): one instance is two endpoints, a width
     and a relation color index. The quad is expanded along the segment in
     the vertex stage. Edges are straight; nothing is routed.
  2. On the worker, per hierarchy level, compute the aggregated edges between
     groups (`from group, to group, count`) and build one block per level,
     plus the block of resource-level edges.
  3. All edge blocks stay bound. The inactive ones are bound with
     `add_retained_instances_count(.., 0)`, so changing level uploads
     nothing. The level comes from the zoom: resource edges only when their
     groups are larger on screen than a threshold; otherwise the aggregate,
     with width from the count.
  4. Edge blocks have a byte bound of their own (MAKEPAD_UI.md section 7.7);
     an account over it draws aggregates only and says so in the status line.
- **Tests:** aggregate counts sum to the edge count at every level; an edge
  inside one group disappears into it at that level; switching level rebuilds
  no block (counter).
- **Budget:** P14 at 1,000,000 edges across the scripted zoom path; P13 for
  the aggregation and block build; bytes held on CPU and GPU.
- **Prove it:** D7's harness.
- **Done when:** the rows are recorded.
- **Size:** about 380 added. Two changes: (1) aggregation and blocks, (2)
  draw path and level switch.
- **Watch out:** a million instances visible at once will not hold 8.3 ms on
  a laptop's built-in GPU whatever the draw path; the aggregation is what makes
  O14's target reachable, so its thresholds come from D7 measurements, and
  D6.0 gives the first numbers.

### D6.3 Labels, level of detail, and a memory bound in place of the caps

- **Goal:** Labels above a zoom threshold, group rectangles and counts at far
  zoom, and the 2,000-node and 4,000-edge caps replaced by a memory bound.
- **Depends on:** D6.2.
- **Files:** `crates/cmdb-ui/src/dag/graph.rs` (`MAX_NODES`, `MAX_EDGES`),
  `dag/layout/mod.rs` (`MAX_LAYOUT_NODES`), `dag/widget/paint.rs`
  (`MAX_LABELS`), `dag/widget/lod.rs` (new).
- **Change:** group rectangles are retained blocks, one per level, built on
  the worker with the layout. Draw the levels and labels the zoom calls for
  (MAKEPAD_UI.md section 7.5): labels for groups larger than a screen
  threshold; node labels only for nodes inside the viewport when fewer than
  `MAX_LABELS` (200 today, to be set from D7) are visible. Label choice runs
  over visible groups, never over all nodes. Register `perf_monitor` channels
  for level choice and label build so their cost shows in the harness. The
  whole-account view is bounded by `LoadLimits` (D4.2), not by `MAX_NODES`.
  The neighbourhood view keeps its own limits (`GRAPH_LIMITS`).
- **Tests:** label selection at several zoom levels; nothing is dropped
  silently: the status line shows loaded and drawn counts.
- **Budget:** P14 across the zoom path with labels on.
- **Prove it:** D7's harness.
- **Done when:** the whole bench account is drawn with no "dropped" count.
- **Size:** about 280 added, 60 deleted. One change.
- **Watch out:** text is the expensive part of a frame; cap visible labels by
  measurement.

### D6.4 Picking through a spatial grid

- **Goal:** Clicking a node selects it in constant time and still opens its
  detail and neighbourhood.
- **Depends on:** D6.3.
- **Files:** `crates/cmdb-ui/src/dag/hierarchy/grid.rs` (new),
  `dag/widget/input.rs` (125 lines), `dag/widget/mod.rs`.
- **Change:** a uniform grid over the layout bounds built on the worker with
  the layout (cell to node index ranges). A click maps through the inverse
  transform to a cell and tests the few nodes in it. Selection sends the
  existing `Command::Expand` with the node's key, so the detail panel and the
  FalkorDB neighbourhood work as today.
- **Tests:** every node is found by a click at its center; a click on empty
  space selects nothing; picking at 250,000 nodes takes a bounded number of
  comparisons.
- **Budget:** P14: a click must not produce a frame over budget.
- **Prove it:** `cargo test -p cmdb-ui --locked grid`; D7's harness with
  scripted clicks.
- **Done when:** selection works on the whole-account view.
- **Size:** about 220 added. One change.
- **Watch out:** `layout.hit_test` (linear) stays for the neighbourhood view.

## D7 Frame-time harness

### D7.1 The harness, on the current painter

- **Goal:** A scripted pan and zoom path that records presented frames, GPU
  time and UI-thread time, built before the renderer changes so the baseline
  and the size at which the current painter fails are recorded.
- **Depends on:** nothing. In wave 0 it runs on the database file that
  `cmdb-bench generate` writes today; after C6.12 the same harness takes a
  PostgreSQL URL and D4.1's database.
- **Files:** `crates/cmdb-ui/src/frame_harness.rs` (new),
  `frame_harness/path.rs` (new), `frame_harness/report.rs` (new),
  `crates/cmdb-ui/src/main.rs`, `app.rs`, `worker.rs` (argument parsing),
  `docs/performance-evidence/frames.md` (new).
- **Change:** `cmdb-ui --database <target> --frame-harness <seconds>`:
  1. After the first complete draw, drive a fixed path by elapsed time: zoom
     from the whole view to one node and back, pan across, three clicks.
  2. Record, per frame: the presented time (Makepad's `present` trace topic,
     Makepad: `platform/src/present_trace.rs`), GPU busy time
     (`gpu_frame_timer::set_enabled(true)` and `take_frames()`, Makepad:
     `platform/src/gpu_frame_timer.rs`), and UI-thread time through
     `perf_monitor` channels (Makepad: `platform/src/perf_monitor.rs`).
  3. On exit print: display refresh rate, resolution and scale factor; window
     size; GPU, driver and backend; frames presented; frames missed (interval
     over 1.5 refresh periods) as a count and a share; p50, p95, p99 and
     maximum of GPU busy time and of UI-thread time; node, edge and visible
     label counts; whether vsync was on.
  4. Exit code 0. The harness reports; it does not judge.
- **Tests:** percentile arithmetic; missed-frame counting against a given
  refresh period; the path is a pure function of elapsed time.
- **Budget:** this step defines P14. Baseline rows: the current painter at
  500 and 2,000 nodes (its cap) and the largest size it accepts.
- **Prove it:** on a machine with a GPU: `cmdb-ui --database <path or URL>
  --frame-harness 20`, once on a 60 Hz and once on a 120 Hz display if both
  exist.
- **Done when:** baseline rows exist with every field of item 3.
- **Size:** about 320 added. Two changes: (1) path and report, (2) capture.
- **Watch out:** the interval between draw events is not a measure of cost:
  Makepad draws once per vsync (Makepad: `platform/src/os/hal/event_loop.rs`),
  so a 1 ms frame and an 8 ms frame both read 8.3 ms on a 120 Hz display. On
  a 60 Hz display nothing can show 120 frames per second; report headroom
  (GPU and UI-thread time) there and say so. The machine the plan was written
  on has no display; the owner's test machines do (V56). How
  the app reads the refresh rate from Makepad is not verified; if it cannot,
  the operator states it and the report marks it as stated.

### D7.2 Frame times at the stated size (X8)

- **Goal:** X8 as restated: on a 120 Hz display at a stated resolution and on
  stated hardware, over the scripted path at 250,000 nodes and 1,000,000
  edges, no more than 1% of frames are missed, and p99 GPU busy time and p99
  UI-thread time are each under the 8.3 ms period.
- **Depends on:** D6.4, D4.1.
- **Files:** `docs/performance-evidence/frames.md`.
- **Change:** run the harness on the bench database three times on the
  two stated machines (V56, V59: a MacBook Pro with the M3 Pro chip, and a
  machine with an RTX 3070 class GPU on a monitor of 120 Hz or more) and record every field,
  a table per machine. If the target is missed,
  the rows say where on the path, and the thresholds of D6.2 and D6.3 are
  tuned from that.
- **Tests:** none.
- **Prove it:** the command and the rows.
- **Done when:** X8 holds, or the measured gap and its location on the path
  are reported with the largest size that does hold.
- **Size:** no Rust, or small threshold changes.
- **Watch out:** the size in O14 is an unconfirmed default, and which 3070
  and whether both machines must pass are open. State the
  machine, the chip, the display and the power source exactly. Thermal throttling on a laptop changes
  the third run; record all three, not the best.

### D7.3 An unpaced throughput check for CI

- **Goal:** Catch a rendering regression on machines with no display.
- **Depends on:** D6.0 (whether the app starts headless), D6.1.
- **Files:** `crates/cmdb-ui/src/frame_harness/report.rs`, `moon.yml` (an
  opt-in task), `docs/performance-evidence/frames.md`.
- **Change:** run the harness with the window hidden and Makepad's batch
  pacing (`MAKEPAD_HIDE_WINDOWS` with `MAKEPAD_PAINT_INTERVAL`, Makepad:
  `platform/src/os/hal/event_loop.rs`), which draws as fast as the GPU
  completes frames. Report frames per second and UI-thread time per frame at
  a fixed size.
- **Tests:** none beyond the harness's own.
- **Budget:** the row is tracked wave to wave under the regression rule.
- **Prove it:** the task's output on the CI machine.
- **Done when:** a row exists, or D6.0's report says the mode does not work
  and this step is closed as not possible.
- **Size:** about 60 added. One change.
- **Watch out:** this is not evidence for X8 and its rows are labelled
  "unpaced, no display". A software rasterizer's numbers say nothing about a
  GPU.

## D8 Parity table

### D8.1 Every CLI command has a UI action or a written reason

- **Goal:** The table V47 asks for.
- **Depends on:** D2.2, D3.2.
- **Files:** `docs/e2e/PARITY.md` (new).
- **Change:** one row per subcommand of `src/cli.rs` (`Catalog`, `Coverage`,
  `Plan`, `Demo`, `Ingest`, `Export`, `Deliver`, `Query`, `Sql`, `Schema`,
  `Report`, `Architecture`, `Diff`, `Neighbors`, `Path`, `Findings`,
  `Discover`, `Job`, `GraphExport`, `GraphPush`, `Scan`, `Sweep`): the UI
  action, or the
  reason there is none (for example: schema 13 only, removed by K2).
- **Tests:** a root test parses the table and fails when `src/cli.rs` has a
  subcommand with no row.
- **Prove it:** `cargo test --locked --test cli_properties parity`.
- **Done when:** the table matches the tree.
- **Size:** about 60 lines of test and the document. One change.
- **Watch out:** `Neighbors`, `Path` and `Findings` exist in the CLI on the
  old store; say which store each row refers to.

## D9 Scope rail, search and filters

### D9.1 The scope rail is built from the data

- **Goal:** Replace the filter chips written by hand into `ui.rs` ("prod",
  "staging", "sandbox", two regions, four kinds) with a tree of account,
  region, service and kind, each with its count, read from the store.
- **Depends on:** C6.11.
- **Files:** `crates/cmdb-view/src/database/facets.rs` (new),
  `crates/cmdb-ui/src/worker/protocol.rs`, `worker/queries.rs`,
  `crates/cmdb-ui/src/scope_rail.rs` (new), `scope_rail/model.rs` (new),
  `ui.rs` (the hand-written chips and `ec2_count`, `s3_count`, `rds_count`,
  `lambda_count` labels are deleted), `data/mod.rs` (`Facet`, `Counts`,
  `Filter`).
- **Change:**
  1. `pub async fn facets(&self, limits: FacetLimits) ->
     Result<FacetTree, ViewError>`: one grouped query for counts by account,
     region and kind; service is derived from the kind name. Bounded: at most
     the configured number of accounts and kinds; past it the tree says how
     many were not listed.
  2. `Command::Facets { revision }`, `WorkerEvent::Facets { tree }`.
  3. The rail is a `PortalList` over the flattened, expandable tree. A click
     on a level sets `Filter` (account, region, service, kind) for the table
     and, in the whole-account view, frames that group.
- **Tests:** the tree's counts sum to the resource count at every level; an
  account with no resources in a region shows no row for it; the model's
  expand and collapse; the limit is reported, not silent.
- **Budget:** P12 for the facet query at 250,000 resources; P14 with the rail
  fully expanded.
- **Prove it:** `cargo test -p cmdb-view --locked facets -- --ignored`;
  `cargo test -p cmdb-ui --locked scope_rail`.
- **Done when:** no account, region or kind name is written in `ui.rs`.
- **Size:** about 380 added, 90 deleted. Two changes: (1) query and event,
  (2) the widget.
- **Watch out:** `ui.rs` is 278 lines; the rail moves out of it. The count
  query must use the indexes C6.11 creates; record its plan.

### D9.2 Search and fly-to

- **Goal:** Find a resource by ID, name or ARN and move to it.
- **Depends on:** D9.1, D6.4 for the fly-to on the account view.
- **Files:** `crates/cmdb-view/src/database/search.rs` (new),
  `crates/cmdb-ui/src/worker/protocol.rs`, `worker/queries.rs`,
  `crates/cmdb-ui/src/search.rs` (new), `ui.rs` (`search_input` exists).
- **Change:** `Command::Search { revision, text }` runs one bounded query
  through the reader: exact match on ID and ARN first, then prefix match on
  name, at most 50 results. `WorkerEvent::SearchResults`. Choosing a result
  selects the resource in the table, fills the detail panel and, in the
  account view, animates the transform to the node over a fixed time.
- **Tests:** exact ID match ranks first; a stale revision is dropped; text
  with SQL metacharacters is bound as a parameter, never concatenated; the
  fly-to path is a pure function of start, end and elapsed time.
- **Budget:** P12 for the search query at 250,000 resources; P14 during the
  fly-to.
- **Prove it:** `cargo test -p cmdb-view --locked search -- --ignored`.
- **Done when:** a seeded resource is found by each of ID, name and ARN.
- **Size:** about 260 added. One change.
- **Watch out:** a prefix search needs an index that supports it; decide the
  index from the query plan and record it. Do not debounce by sleeping on
  the UI thread; send revisions and let the newest win.

### D9.3 Filters dim the graph and narrow the table

- **Goal:** One filter state for both views.
- **Depends on:** D9.1, D6.3.
- **Files:** `crates/cmdb-ui/src/data/mod.rs` (`Filter`),
  `dag/widget/account_view.rs`, `dag/widget/instances.rs`, `table.rs`.
- **Change:** the table applies the filter in its query. The account view
  applies it without rebuilding the node block: a small per-group visibility
  table is a uniform or a tiny retained block of its own, and the node
  shader dims instances whose group is filtered out. Derived owner, team and
  application filters are added by G3 through the same state.
- **Tests:** setting a filter rebuilds no node or edge block (counter);
  table rows and highlighted nodes are the same set.
- **Budget:** P14 when a filter changes: no missed frame.
- **Prove it:** `cargo test -p cmdb-ui --locked filter`; D7's harness with a
  scripted filter change.
- **Done when:** both views follow one filter.
- **Size:** about 240 added. One change.
- **Watch out:** how much data a uniform can carry at this Makepad revision
  is one of D6.0's questions; if it is small, use a retained block indexed
  by group.

## D10 The table

### D10.1 Paging by key in both directions

- **Goal:** Every row is reachable; the 500-row limit goes.
- **Depends on:** C6.11.
- **Files:** `crates/cmdb-ui/src/table.rs` (296 lines),
  `table/model.rs` (new), `worker/protocol.rs`, `worker/queries.rs`,
  `crates/cmdb-view/src/database/postgres.rs` (C6.11's `page`).
- **Change:** the table model holds a window of pages around the visible
  rows and asks the worker for the next or previous page by key as the
  `PortalList` nears an end. At most a fixed number of pages are kept; older
  ones are dropped. The header shows the filtered count and the position.
- **Tests:** scrolling to the end of 10,000 rows reaches the last key; pages
  are dropped past the bound and fetched again on the way back; rows changed
  by a sweep while paging neither repeat nor vanish within one direction.
- **Budget:** P12 page latency; P14 while scrolling fast through 250,000
  rows; memory held by the table is bounded.
- **Prove it:** `cargo test -p cmdb-ui --locked -- --ignored worker_page`.
- **Done when:** `PAGE_LIMIT` no longer limits what can be seen.
- **Size:** about 300 added, 60 deleted. One change.
- **Watch out:** split `table.rs` first. Offsets are not used; a count over
  a large filter can be slow, so it is its own bounded query and may show
  "more than N".

### D10.2 Typed columns for the selected kind, and sorting

- **Goal:** With a kind selected the table shows that kind's typed columns.
- **Depends on:** D10.1, C1.4, C4.1.
- **Files:** `crates/cmdb-view/src/database/kind_page.rs` (new),
  `crates/cmdb-ui/src/table/columns.rs` (new), `table.rs`.
- **Change:** column names and types come from the store's own registry (the
  one `aws-inventory schema` reads). The page query selects the common
  columns plus up to a fixed number of the kind's columns; the user chooses
  which. Sorting is by key or by one typed column, done by the server, with
  the key as the tie-break so paging stays stable.
- **Tests:** a kind's seeded value appears under its column; a sort by a
  typed column pages without repeating a row; a column name is taken from
  the registry and quoted, never from user text.
- **Budget:** P12 for a sorted page on a typed column at 250,000 rows of one
  kind; record whether it needs an index.
- **Prove it:** `cargo test -p cmdb-view --locked kind_page -- --ignored`.
- **Done when:** T1's "shown in the UI's table" includes typed columns.
- **Size:** about 320 added. One change.
- **Watch out:** a kind can have over a thousand columns; the table never
  selects them all.

## D11 Detail and traversal

### D11.1 The detail panel shows the whole record

- **Goal:** The selected resource's identity, name, decoded record as a field
  tree, and its edges in and out with their source.
- **Depends on:** C6.11.
- **Files:** `crates/cmdb-ui/src/detail.rs` (new), `detail/tree.rs` (new),
  `worker/protocol.rs`, `crates/cmdb-view/src/database/postgres.rs`
  (`detail`, `edges_page`).
- **Change:** the worker decodes the stored record with the catalog into a
  bounded tree of name and value nodes (depth, node count and bytes all
  bounded; past a bound the tree says what was cut). Edges are listed with
  relation, direction, the other end, and whether the edge is declared by
  the model, a reviewed rule, or inferred. Clicking the other end selects
  it. Tags and derived owner, team and application are added by G3.
- **Tests:** a nested record renders as a tree; bounds cut and say so; an
  inferred edge is labelled inferred; values the deny rules redact are shown
  as redacted.
- **Budget:** P12 for one detail read; P14 opening a record with 2,000
  fields.
- **Prove it:** `cargo test -p cmdb-ui --locked detail`.
- **Done when:** no field of a seeded record is missing from the panel.
- **Size:** about 340 added. One change.
- **Watch out:** the record is CBOR; decode on the worker, never on the UI
  thread.

### D11.2 Neighbours, path and blast radius on the account view

- **Goal:** Traversal results drawn as a highlight over the whole account.
- **Depends on:** D11.1, D6.4, E3.2.
- **Files:** `crates/cmdb-ui/src/worker/graph_tasks.rs` (151 lines),
  `worker/protocol.rs`, `crates/cmdb-ui/src/traversal.rs` (new),
  `dag/widget/highlight.rs` (new), `crates/cmdb-view/src/graph.rs`
  (`neighbors`, `blast_radius`, `path` exist).
- **Change:** `Command::Traverse { revision, kind: Neighbors { depth } |
  Path { to } | BlastRadius { depth } }`. The graph database returns keys;
  the worker maps them to node indices of the loaded account; the UI draws
  the nodes and edges of the result with per-frame instancing above the
  retained layers and dims the rest. The existing neighbourhood view stays
  as a second way to look at the same result. `cmdb-bench traverse <graph
  URL> <key>` (new subcommand) times the three traversals for P17.
- **Tests:** keys not in the loaded account are counted, not dropped
  silently; every bound of the traversal is reported when hit; with the
  graph database stopped the panel says "graph unavailable" and the rest of
  the app works.
- **Budget:** P17 (traversal latency, a new row) for each of the three;
  P14 with a 5,000-node highlight.
- **Prove it:** `cargo test -p cmdb-ui --locked -- --ignored worker_traverse`.
- **Done when:** the three traversals are visible on the account view.
- **Size:** about 320 added. One change.
- **Watch out:** the highlight layer is bounded; a blast radius larger than
  the bound is shown as counts by group, not as nodes.

## D12 Coverage

### D12.1 The coverage tab

- **Goal:** Show what was and was not collected, so a missing resource is
  never mistaken for an absent one.
- **Depends on:** C6.11, A5.1.
- **Files:** `crates/cmdb-view/src/database/coverage.rs` (new),
  `crates/cmdb-ui/src/coverage.rs` (new), `worker/protocol.rs`.
- **Change:** one bounded query over `scope_outcomes` for the latest sweep
  of each account: per account, region and service, the counts of complete,
  partial, inaccessible, unsupported, failed and throttled scopes, with the
  stored error class. A second list: kinds that are callable but have never
  returned a seeded or real resource (implemented, not verified). Selecting
  a row lists its scopes.
- **Tests:** the tab's totals equal the sweep summary's outcome counts; a
  denied scope is shown as inaccessible, not as empty.
- **Budget:** P12 for the query after a full sweep.
- **Prove it:** `cargo test -p cmdb-view --locked coverage -- --ignored`.
- **Done when:** X3's list is visible in the UI.
- **Size:** about 260 added. One change.
- **Watch out:** freshness over time (F3) extends this tab later; do not
  build history here.

## D13 Connections, roles and states

### D13.1 Two database URLs, checked, failing closed

- **Goal:** The collector and reader connections of MAKEPAD_UI.md section 5.
  This is the fix for review finding 2 on the UI side.
- **Depends on:** C4.1 (the roles exist), C3.2 (`ReaderUrl`).
- **Files:** `crates/cmdb-ui/src/worker.rs` (argument and environment
  parsing), `worker/connections.rs` (new), `crates/cmdb-view/src/lib.rs`
  (re-export), the engine's `sql` module from C3.1 and C3.2 (where
  `ReaderUrl` is defined).
- **Change:**
  1. `ReaderUrl` is defined once, in the engine's `sql` module (C3.2), and
     re-exported by `cmdb-view`; `pub struct CollectorUrl(String)` sits beside
     it. `ReaderUrl::establish(url)` connects, asks the server whether the
     session can write to the inventory schema (`has_schema_privilege`,
     `has_table_privilege` on a known table, and `transaction_read_only`),
     and returns an error if it can or if the check fails.
  2. Settings: `INVENTORY_READER_URL`, `INVENTORY_COLLECTOR_URL`, or a
     settings file readable only by its owner. A debug build also accepts
     `--database` for the reader. No URL is logged; errors print the host
     and database, never the password.
  3. With no collector URL the app is a viewer: `StartSweep` is refused with
     that reason. With no established reader, the table, account load,
     search, detail and console are all disabled and say why.
- **Tests:** a superuser or collector URL is refused as a reader; a reader
  URL is accepted; an unreachable server disables reads and leaves the
  window usable; no test output contains a password.
- **Budget:** none; start-up time with the check is recorded once.
- **Prove it:** `cargo test -p inventory-engine --locked reader_url --
  --ignored`; `cargo test -p cmdb-ui --locked connections`.
- **Done when:** D2.1 and D3.1 can only be given the right kind of URL.
- **Size:** about 240 added. One change.
- **Watch out:** a role's rights can change after the check; the reader
  queries still run in read-only transactions with a timeout (C3.2), so the
  check is a second lock, not the only one. Where secrets are finally kept
  is O20.

### D13.2 Header, status line, and a screen for every state

- **Goal:** The states listed in MAKEPAD_UI.md section 6 each have a
  designed screen, and the status line always says what is loaded and drawn.
- **Depends on:** D13.1, D4.3.
- **Files:** `crates/cmdb-ui/src/app.rs` (304 lines; split per A0.4),
  `app/state.rs` (new), `chrome.rs`, `status_line.rs` (new), `ui.rs`.
- **Change:** one `AppState` enum per area (store, graph database, reader
  role, account load, sweep) with explicit transitions, and one function
  from state to what is shown. Header totals come from the store. The status
  line shows loaded and drawn counts, last frame time, sweep state and each
  connection's role.
- **Tests:** every state has a rendering in the model; a transition table
  test covers each event in each state; loaded and drawn counts differ only
  with a reason shown.
- **Budget:** P14 unchanged.
- **Prove it:** `cargo test -p cmdb-ui --locked state`.
- **Done when:** no failure leaves an empty panel without a message.
- **Size:** about 300 added, 80 deleted. One change.
- **Watch out:** `app.rs` and `ui.rs` are at or near the size limit; this
  step is where they are split by area.

## D14 Platforms and packaging

### D14.1 The platform matrix

- **Goal:** The app builds, starts and passes the frame harness on each
  platform, recorded.
- **Depends on:** D6.4, D7.1.
- **Files:** `docs/performance-evidence/platforms.md` (new).
- **Change:** on macOS (Metal) and Linux (Vulkan), the two targets (V58):
  build in
  release, start, run the harness, record the backend Makepad chose, the
  display, and the rows. Differences in input (pinch, scroll direction,
  scale factor) are listed.
- **Tests:** none.
- **Prove it:** the document's rows.
- **Done when:** each platform has a row or a stated reason it has none.
- **Size:** no Rust unless a platform needs a fix.
- **Watch out:** this can start as soon as D7.1 exists; it does not wait for
  the renderer.

### D14.2 Packaging, after the milestone

- **Goal:** An installable build per platform.
- **Depends on:** the exit test.
- **Files:** to be named by the worker after reading Makepad's packaging
  tools at the pinned revision (not read for this plan).
- **Change:** a signed, versioned package per platform; where settings and
  logs live; how an update replaces a running app. Feeds the release gate
  ([RELEASE_GATE.md](../RELEASE_GATE.md)).
- **Tests:** install, start, upgrade and remove on a clean machine.
- **Prove it:** the recorded run on each platform.
- **Done when:** the release gate's installation line can be tested.
- **Size:** not estimated.
- **Watch out:** this is outside the milestone.
