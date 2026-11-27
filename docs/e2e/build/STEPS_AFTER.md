# Steps after the exit test

> Status: Plan, 2026-11-15, revised for V53 (PostgreSQL is the only store).
> Step-by-step breakdown of the packages in [AFTER.md](../AFTER.md), lighter
> than the milestone tracks. It starts once X1 to X10 hold. Two things moved
> out of here into the milestone: the PostgreSQL store (H1 became C6, by V51)
> and the first half of F5 (remembered scopes, now B6.4, because it removes
> calls). The early head-to-head harness is B9; J1 is the full run. Nothing
> here has been built.

Conventions: as in [STEPS_A.md](STEPS_A.md). Each step still records its
budget numbers with `cmdb-bench` (V52).

## F Continuous

### F1.1 Several profiles in one database

- **Goal:** One database holds every account; today one invocation takes one
  profile and the store refuses a second active sweep.
- **Depends on:** exit test; C6.5.
- **Files:** `crates/engine/src/inventory/postgres/work.rs`
  (the active-sweep check and its advisory lock),
  `crates/sweep/src/lib.rs`, `src/cli/sweep.rs`.
- **Change:** an active sweep is per account: `work_queue` rows of different
  accounts may coexist, each with its own sweep ID. `sweep --profile a
  --profile b` runs the accounts one after another or, with `--parallel
  <n>`, at once, sharing one `ApiExecutor` (buckets are already per account,
  B1.1) and one writer.
- **Tests:** two emulator profiles if one emulator can present two accounts
  (not verified), otherwise two fixture accounts through the store tests: no
  cross-account deletion, summaries per account.
- **Budget:** P1 for two accounts against twice one account.
- **Prove it:** e2e run with two profiles, or the recorded limit.
- **Done when:** two accounts live in one database.
- **Size:** about 250 added.
- **Watch out:** scope completion and deletion are keyed by scope, which
  includes the account; check every query that assumes one sweep ID.

### F1.2 `serve`: sweep on an interval, stalest first, never two at once

- **Goal:** ROADMAP M7.1.
- **Depends on:** F1.1, D1.3.
- **Files:** `src/cli.rs` (subcommand `Serve`), `src/serve_command.rs` (new),
  `crates/sweep/src/schedule.rs` (new), store claim order.
- **Change:** `aws-inventory serve --interval <duration> --profile ...`: loop
  of sweep, then idle until the next interval boundary; a `CancellationToken`
  ends it on a signal. Within a sweep, claim order becomes "oldest
  `scope_outcomes.observed_at` first" (a column on `work_queue` set at seed
  time, indexed), replacing `ORDER BY scope_key`. A lock row or advisory lock
  prevents two `serve` processes on one database.
- **Tests:** deterministic-time test of the loop; stalest-first order; a
  second process is refused; kill and restart resumes.
- **Budget:** proxy log shows zero requests in the idle time; P2 per cycle.
- **Prove it:** over three intervals against the emulator, each seeded change
  is stored by the sweep after it.
- **Done when:** the M7.1 evidence holds.
- **Size:** about 320 added.
- **Watch out:** changing claim order changes resume determinism; the kill
  matrix (B4.3) must be rerun with the new order.

### F2.1 Deletion in continuous mode, and unobserved scopes

- **Goal:** ROADMAP M7.2.
- **Depends on:** F1.2, B4.5.
- **Files:** `crates/engine/src/inventory/postgres/completion.rs`,
  `crates/sweep/src/execution/draining.rs`.
- **Change:** a scope that was swept before and is not seeded now (regions
  narrowed, operation deferred) gets outcome `unobserved` for the sweep and
  deletes nothing. Deletion still needs two complete enumerations of the same
  scope.
- **Tests:** seeded deletion gone from SQL and the graph after the second
  complete sweep; narrowing regions deletes nothing and reports the scopes.
- **Budget:** P2 unchanged.
- **Prove it:** e2e test over three sweeps.
- **Done when:** the M7.2 evidence holds.
- **Size:** about 180 added.
- **Watch out:** B6.4's remembered scopes are also "not asked"; they must
  count as unobserved, never as complete.

### F3.1 Freshness and coverage as a view and a panel

- **Goal:** ROADMAP M7.3.
- **Depends on:** F2.1, C4.1.
- **Files:** `crates/engine/src/inventory/postgres/schema/` (view
  `scope_freshness`),
  `crates/cmdb-view/src/database/`, `crates/cmdb-ui/src/freshness_panel.rs`
  (new).
- **Change:** SQL view over the latest `scope_outcomes` row per scope: account,
  region, service, operation, outcome, detail, age in seconds. The UI panel
  lists it with filters.
- **Tests:** the view and the panel agree with stored outcomes.
- **Budget:** P12 for the view at 100,000 scopes.
- **Prove it:** `aws-inventory sql "select outcome, count(*) from
  scope_freshness group by 1"`.
- **Done when:** the M7.3 evidence holds.
- **Size:** about 220 added.
- **Watch out:** `scope_key` is a binary encoding; the view needs decoded
  columns, so store them as columns at write time rather than decoding in SQL.

### F4.1 Slice mode

- **Goal:** ROADMAP M7.4: run for a bounded time, stop, resume. Lambda needs
  it later.
- **Depends on:** D1.3, B3.2.
- **Files:** `crates/sweep/src/lib.rs` (`SweepConfig.deadline`),
  `crates/sweep/src/execution/scheduler.rs`, `src/cli/sweep.rs`.
- **Change:** `--max-seconds <n>`: the scheduler stops claiming at the
  deadline minus the lease margin, lets in-flight pages finish, and exits with
  a distinct code meaning "more work". `--resume` continues.
- **Tests:** a sweep finished in several slices equals an uninterrupted one
  (the kill-matrix digest).
- **Budget:** total API calls over the slices equal one uninterrupted sweep
  (P3); learned rates survive between slices (B2.2).
- **Prove it:** e2e with three slices.
- **Done when:** the M7.4 evidence holds.
- **Size:** about 150 added.
- **Watch out:** nothing may assume a long-lived process beyond the store.

### F5.1 The remaining failure rows of VISION section 7

- **Goal:** Retry budget per bucket, expired pagination token restarts one
  enumeration, expired credentials refreshed once, expired signature signed
  again, sleep and network loss (M4.2, M4.5). Remembered scopes are done
  (B6.4).
- **Depends on:** B2.3, B6.4.
- **Files:** `crates/sweep/src/call.rs` (`classify_service` maps
  `ExpiredToken` and `ExpiredSignature` to `Transient` today),
  `crates/sweep/src/execution/page.rs`, `crates/engine/src/limiter/retry.rs`.
- **Change:** distinct handling per error class as VISION 5.3 describes; one
  fault-injection test per row, named after the row, through the harness
  proxy (it already offers every fault of M4.3).
- **Tests:** one per row; a failure common to every endpoint marks no scope
  failed, unsupported or inaccessible.
- **Budget:** P3 retries under each fault are bounded and recorded.
- **Prove it:** the ignored e2e tests with the proxy.
- **Done when:** every row of VISION section 7 marked M4.2 or M4.5 has a
  passing test.
- **Size:** about 400 added, in two or three changes.
- **Watch out:** `ApiExecutor` stays the only retry owner; token restart and
  re-signing are decisions of the page loop that re-enter admission.

## G Tags

### G1.1 Tags in bulk

- **Goal:** Tags through the Resource Groups Tagging API, joined by ARN (V31,
  M8.1).
- **Depends on:** exit test; E1.2 (`identities` gives ARN to key).
- **Files:** `catalog/overlay/resource-groups-tagging-api.toml` (hand entry
  for `GetResources`), `crates/catalog-tool/src/runtime_catalog/plans/` (a
  plan purpose `Tags`), `crates/core/src/catalog/runtime/types.rs`
  (`PlanPurpose::Tags`), `crates/sweep/src/execution/page/tags.rs` (new).
- **Change:** a classified, overlay-reviewed enumeration whose rows are not
  resources: each row's ARN is looked up in `identities` and its tags fill
  `PageCommit.tags` (empty today). Tags are replaced per resource per sweep.
- **Tests:** every tagged seeded resource has its tags in SQL; a kind the bulk
  API covers makes no per-resource tag call; an ARN with no stored resource is
  counted, not stored.
- **Budget:** P3: tag calls per sweep; P2 with tags unchanged writes nothing.
- **Prove it:** e2e with tagged seeds.
- **Done when:** the G1 evidence holds for kinds the bulk API covers.
- **Size:** about 320 added.
- **Watch out:** tags must not enter the record's content hash, or a tag
  change would rewrite the resource; they have their own table and their own
  comparison.

### G1.2 The service's own tag call where the bulk API has none

- **Goal:** Coverage for kinds the bulk call does not return.
- **Depends on:** G1.1, A3.2.
- **Files:** overlay `tags` key (exists in the syntax, not compiled),
  `runtime_catalog/plans/compile.rs`, generator rule in
  `overlay/generate/expand.rs`.
- **Change:** compile `tags` (item member path of a tag list) into the plan;
  for a kind with no bulk coverage and a bound `ListTagsFor*` expansion, mark
  that expansion as the tag source. Report kinds with no tag source.
- **Tests:** list-of-pairs and map-shaped tag members; a kind with neither is
  in the report.
- **Budget:** P3 per-resource tag calls, recorded.
- **Prove it:** e2e on one such kind the emulator serves.
- **Done when:** the report lists every kind's tag source or its absence.
- **Size:** about 240 added.

### G2.1 Owner, team and application from a mapping file

- **Goal:** M8.1 and M8.2: derived values stored apart from the record and
  derived again without a sweep when the mapping changes.
- **Depends on:** G1.1.
- **Files:** `crates/engine/src/inventory/derived.rs` (new), schema table
  `derived(account, region, kind, id, owner, team, application, mapping_digest)`,
  `src/cli.rs` (subcommand `Derive`), mapping file format (TOML: `owner =
  ["Owner", "owner"]`, `team = [...]`, `application = [...]`).
- **Change:** one set-based statement per attribute from `tags`; rerun when
  the mapping digest differs.
- **Tests:** after the mapping changes, derived values change with no API
  call (proxy log empty).
- **Budget:** derive time per 100,000 resources.
- **Prove it:** `aws-inventory derive --database ... --mapping map.toml`.
- **Done when:** the G2 evidence holds.
- **Size:** about 260 added.

### G3.1 Derived values in the graph and the UI

- **Goal:** M8.3.
- **Depends on:** G2.1, E3.2, D6.3.
- **Files:** `crates/graph-store/src/cypher.rs` (node properties),
  `crates/cmdb-ui/src/` (an application view and a "derived from tags" mark).
- **Change:** project owner, team and application as node properties; the UI
  groups by application and lists unassigned resources.
- **Tests:** an application view lists its resources and the unassigned ones.
- **Budget:** P15 with the extra properties; P14 for the view.
- **Prove it:** worker test and a manual session.
- **Done when:** the G3 evidence holds.
- **Size:** about 280 added.

## H On AWS

Version 1 runs on one person's laptop (V54), so running the collector
inside AWS (H1, H3 to H6) is after version 1. H2.1 and H2.2 are the
exception: the laptop sweeps whole organizations (V64), so both are part of
version 1 and are done after the milestone, before the release gate.
H2.2 (the role as a template) is set aside for now (V75): access to each
account is assumed to exist, however the organization grants it.

### H1.1 A managed PostgreSQL in AWS

- **Goal:** What is left of H1 after C6: the end-to-end suite and the sweep
  benchmark against a managed PostgreSQL in AWS, beside the rows from the
  local container or native server.
- **Depends on:** B5.5.
- **Files:** `docs/performance-evidence/sweep-benchmark.md`.
- **Change:** point `INVENTORY_POSTGRES_URL` at the managed instance (TLS:
  H4.1's feature), run the e2e suite and `cmdb-bench sweep --profile full
  --server managed`, and record the rows with the instance class and the
  server settings block.
- **Budget:** P1 to P10.
- **Prove it:** the rows.
- **Done when:** recorded.
- **Size:** no Rust.

### H2.1 Accounts from Organizations and assumed roles

- **Goal:** Beside local profiles, discover every account of an organization
  and assume a read-only role in each, from the laptop (V64). Part of
  version 1.
- **Depends on:** F1.1, D1.2 (`CredentialSource`).
- **Files:** `crates/aws/src/assumed_role.rs` (new; implements
  `CredentialSource` through `aws-config`'s assume-role provider),
  `src/cli/sweep.rs` (`--role-name`, `--accounts-from-organizations`).
- **Change:** list accounts with the callable Organizations enumeration
  already in the catalog, then one `CredentialSource` per account. Refresh
  before expiry.
- **Tests:** two emulator accounts through assumed roles, or the recorded
  limit if one emulator cannot present two.
- **Budget:** P3: STS calls per sweep.
- **Prove it:** e2e or the recorded limit.
- **Done when:** the H2 evidence holds.
- **Size:** about 300 added.
- **Watch out:** `sts:AssumeRole` is a credential-minting call; it is made by
  the credentials provider, never through the catalog, and the three STS
  operations denied as `secret` stay denied.

### H2.2 The read-only role as a template

- **Goal:** A template the customer deploys.
- **Depends on:** A1.7. If L2.1 has already built the `iam-policy` command
  (it is pulled into version 1), this step adds only the template.
- **Files:** `deploy/read-only-role.yaml` (new),
  `crates/catalog-tool/src/iam_policy.rs` (new).
- **Change:** `catalog-tool iam-policy --partition <partition> --account <12-digit-id>` prints the IAM actions of every
  callable operation (from each service's `arnNamespace` and operation name),
  grouped to fit policy size limits; the template embeds the output and is
  checked by the gate.
- **Tests:** every callable operation maps to exactly one action; regenerating
  is byte-identical.
- **Prove it:** `catalog-tool iam-policy --partition aws --account 123456789012 --check`.
- **Done when:** the template matches the catalog.
- **Size:** about 220 added.
- **Watch out:** IAM action names do not always equal operation names; the
  mismatches need a small reviewed table, found on the real account (J2).

### H3.1 Container image and a scheduled Fargate task

- **Goal:** `serve` against PostgreSQL in the customer's account (O15, O16).
- **Depends on:** F1.2, H2.1.
- **Files:** `deploy/Dockerfile` (new), `deploy/fargate-task.json` (new),
  `deploy/README.md` (new).
- **Change:** a static release binary with the `sweep` feature only, the
  catalog embedded or beside it, configuration by environment.
- **Tests:** the image runs the e2e flow against the emulator in a container.
- **Budget:** image size; cold start to first request.
- **Prove it:** the task definition and one recorded run.
- **Done when:** the H3 evidence holds.
- **Size:** no product Rust beyond configuration parsing.

### H4.1 The desktop app against remote stores

- **Goal:** Tables, console and DAG against a remote PostgreSQL and graph.
- **Depends on:** C6.11, D6.4.
- **Files:** `Cargo.toml` (`sqlx` TLS feature), `crates/cmdb-view/src/database/postgres.rs`.
- **Change:** TLS to PostgreSQL with certificate verification on; connection
  settings in the UI; latency-tolerant loading (D4.2's pages already stream).
- **Tests:** against a PostgreSQL container started with a test certificate.
- **Budget:** P12 load time over a link with added latency.
- **Prove it:** a recorded session.
- **Done when:** the H4 evidence holds.
- **Size:** about 200 added.
- **Watch out:** the name of the `sqlx` TLS feature at the pinned revision is
  not verified (C6.1).

### H5.1 Neptune behind the graph trait

- **Goal:** V46's optional backend.
- **Depends on:** E3.3.
- **Files:** `crates/graph-store/src/neptune.rs` (new).
- **Change:** a second `GraphBackend` using Neptune's openCypher HTTPS
  endpoint with SigV4 through `aws-wire`.
- **Tests:** the traversal tests pass against both backends.
- **Budget:** P15 for both.
- **Prove it:** the test run on a Neptune instance the owner provides.
- **Done when:** the H5 evidence holds.
- **Size:** about 350 added.

### H6.1 Lambda fan-out

- **Goal:** One invocation per account and region, on slice mode, only when
  account counts make it worth the orchestration.
- **Depends on:** F4.1, H3.1.
- **Files:** `deploy/lambda/` (new).
- **Change:** an orchestrator that seeds the queue once and invokes slices; a
  connection pooler in front of PostgreSQL.
- **Budget:** recorded comparison with the Fargate run (P1, cost).
- **Prove it:** the comparison.
- **Done when:** the H6 evidence holds.
- **Size:** not estimated.

## I SSO and the repository split

### I1.1 SSO

- **Goal:** Sign-in for the UI.
- **Change:** not planned. There is no SSO to start (V61).

### I2.1 Split into an open and a commercial workspace

- **Goal:** none. Dropped by V63: everything is MIT and there is no closed
  part, so every crate keeps inheriting `license = "MIT"` and there is one
  workspace.
- **What is left:** making the private repository public, which has no date
  and no step.

## J Proof

### J1.1 The full head-to-head on the emulator

- **Goal:** CloudQuery, Steampipe and Cartography, same dataset and machine,
  three runs each in rotated order, median reported (M5.6, V19).
- **Depends on:** B9.2, A7.2.
- **Files:** `docs/performance-evidence/head-to-head/`.
- **Change:** add Cartography's tool definition (it needs a Neo4j it owns);
  run on the full seed.
- **Prove it:** the published table with versions, dataset, machine and the
  kinds every tool collected.
- **Done when:** the table is committed.
- **Size:** no Rust.

### J2.1 The first real-account run

- **Goal:** Read-only, in the account the owner names (O8, M5.8).
- **Depends on:** exit test; the owner's authorization.
- **Change:** run with `--max-rps` set low first; record coverage and
  outcomes, every endpoint, signature, pagination or decode difference from
  the emulator, and the learned rates.
- **Budget:** P1 to P10 on real quotas, reported separately from emulator
  rows.
- **Prove it:** the report.
- **Done when:** the M5.8 evidence exists.
- **Watch out:** this is the first test of endpoint resolution, real SigV4
  checks, real quotas and rpcv2Cbor on real services. Nothing before it proves
  them.

### J3.1 Head-to-head on the real account

- **Goal:** M10.2.
- **Depends on:** J2.1, J1.1.
- **Change:** tools run one after another, never at once (they share quota).
- **Prove it:** the published table.
- **Done when:** "fastest" can be claimed or not, from the table (V19).

## K Removal

### K1.1 Delete each typed adapter and its SDK crate

- **Goal:** M2.6 and M5.7.
- **Depends on:** A7.2 for the service; D1.1.
- **Files:** `crates/aws/src/{ec2,ecs,eks,dynamo,kms,lambda,load_balancing,
  network,sns,sqs,step_functions,...}`, `crates/aws/Cargo.toml`.
- **Change:** one change per service, when the wire engine returns the same
  fields: compare the adapter's output with the swept record field by field
  against the emulator, then delete the adapter and its `aws-sdk-*`
  dependency.
- **Tests:** the differential test for that service, deleted with the
  adapter.
- **Budget:** build memory and time after each deletion.
- **Prove it:** `cargo tree` shows none of those crates except what
  `aws-config` needs.
- **Done when:** the `adapters` feature is empty and removed.
- **Size:** deletes far more than it adds.

### K2.1 Cloud Control behind the verification command; delete the old scan path and everything only it needs

- **Goal:** Amended M9.5 (V39), and the end of SQLite in the workspace (V53).
  C6.12 removed the schema 14 SQLite store; this step removes everything else
  that still links SQLite.
- **Depends on:** A6.2, K1.1, C6.12.
- **Files, deleted:** `crates/aws/src/cloud_control*` (what A6.1 needs moves
  into the harness first), `src/scan*`, `src/plan*`, and the commands that
  only the old pipeline serves (`Query`, `Ingest`, `Demo`, `Scan`, and the
  export, delivery, job and analysis commands unless D8.1's parity table kept
  one by porting it); `crates/engine/src/store.rs` and `store/` (schema 13,
  17 entries, including `sql_policy.rs`, whose lexer already lives in
  `crates/engine/src/sql/policy.rs` since C3.1); `crates/engine/src/jobs.rs`
  and `jobs/`; `crates/engine/src/graph.rs` and `graph/`;
  `crates/engine/src/export.rs` and `export/`; the SQL files 001 to 013;
  `crates/engine/examples/resource_views.rs` and the `sql-views` gate task;
  the engine tests of the old pipeline (`store.rs`, `store_properties.rs`,
  `migrations.rs`, `jobs.rs`, `job_properties.rs`,
  `job_validation_properties.rs`, `scan_lifecycle.rs`, `registry.rs`,
  `graph.rs`, `graph_contracts.rs`, `graph_retry.rs`, `export.rs`,
  `export_properties.rs`, `global_views.rs`, `catalog_upgrade.rs`,
  `parent_catalog_upgrade.rs`, `reference*.rs`, `retry_properties.rs`,
  `sqlite_dependency.rs`); `tests/cli_properties*` cases and
  `tests/emulator_cli*` that drive the old commands; the harness's
  `snapshot/`, `audit/persisted.rs` and the schema 13 half of
  `assertions/cloud_control.rs`.
- **Files, changed:** every `Cargo.toml` that names the `sqlite` feature of
  `sqlx` (`inventory-engine`, `inventory-harness`, the root dev-dependency,
  per C6.12) drops it; `.cargo/config.toml` loses `LIBSQLITE3_FLAGS` (with
  `SQLITE_MAX_COLUMN=256`) and its comment; `crates/catalog-tool/src/names/
  identifier.rs` loses the SQLite keyword list and the `sqlite_` prefix rule
  if nothing else needs them (PostgreSQL's reserved words stay);
  `inventory_core::inventory::KindSchema::new` loses its `sqlite_` table
  check; `Cargo.lock` is regenerated and no longer lists `libsqlite3-sys` or
  `sqlx-sqlite`. Documents whose subject is the deleted pipeline
  (`docs/STORAGE.md`, `docs/DURABLE_JOBS.md`, `docs/EXPORT.md`,
  `docs/DELIVERY.md`, `docs/ANALYSIS.md`, `docs/COLLECTION_PLAN.md`,
  `docs/LOCAL_SETUP.md`, `docs/LOCAL_WALKTHROUGH.md` and the like) are deleted
  or rewritten in the same change; K4.1 checks the rest.
- **Prove it:**
  - `cargo tree --workspace -e all -i sqlx-sqlite` and `cargo tree --workspace
    -e all -i libsqlite3-sys` both report that the package does not match
    anything in the graph.
  - `grep -rIl "sqlite\|Sqlite\|SQLITE" Cargo.toml .cargo crates src tests
    scripts moon.yml` prints nothing (documents may still mention the
    history).
  - `cargo tree -p aws-inventory -e normal -i aws-sdk-cloudcontrol` reports no
    match; `inventory-harness verify cloud-control` still runs.
  - The gate passes without the `sql-views` and `parent-catalog` tasks if
    their inputs were deleted (state which were removed from `moon.yml`).
- **Done when:** the K2 evidence holds and no SQLite is linked anywhere.
- **Size:** a large deletion (not estimated; the old pipeline is most of
  `crates/engine/src` outside `inventory/`, `limiter/` and `sql/`).
- **Watch out:** `limiter/` is shared with the new pipeline and stays. Check
  D8.1's parity table before deleting a command: a command the owner wants
  kept must be ported to the new store first, as its own step.

### K3.1 Delete Python

- **Goal:** M2.10.
- **Depends on:** A4.2 (seeding without Python), K2.1.
- **Files:** every `.py` file under `scripts/` and `catalog/upstream/`, both
  `requirements.txt`.
- **Change:** delete; each script still referenced by a document is replaced
  by its Rust command in that document.
- **Prove it:** `git ls-files '*.py' '*requirements.txt'` prints nothing; the
  gate passes.
- **Done when:** the K3 evidence holds.

### K4.1 Manifest and documents in line with the tree

- **Goal:** M0.5 and the stale documents.
- **Depends on:** K3.1.
- **Files:** `MANIFEST.sha256` (15.9 MB today, mostly `.local/` paths),
  `START_HERE.md`, `CONTRACT.md`, `NEXT_AGENT_PROMPT.txt`, `docs/VISION.md`
  (merge V35 to V53 into section 2, from the ledger in
  [DECISIONS.md](../DECISIONS.md)), `docs/ARCHITECTURE.md`.
- **Change:** regenerate the manifest from tracked files only; correct every
  statement the milestone made wrong.
- **Prove it:** every path in the manifest is tracked; no document describes
  the UI as a synthetic fixture or names a store other than PostgreSQL.
- **Done when:** the K4 evidence holds.

## S Scale to 10,000 accounts

The plan was sized for about 10 accounts. The target is now up to 10,000 in
one organization (V69), reached a rung at a time: 10, 100, 1,000, 10,000.
The instruction is to plan for massive scale (V76) and not to treat "does
it fit on a laptop" as a blocker (V74): build it as fast and as frugal as
Rust allows, and let the ladder report what it finds. The parts that are
cheap to shape early are shaped in the milestone steps that first build
them (B1.2, B7.2, C6.3, C6.5 each carry a scale note). These steps come
after the milestone and after H2.1, and are the proof. They are
written at the level of intent: each names what has to change and what
proves it, and the worker who takes one writes the file list after reading
the code as it then stands (V72). The numbers are arithmetic from the
plan's own counts and nothing here has been run.

### S1.1 An organization to test against

- **Goal:** A repeatable stand-in for an organization of N accounts, for N
  up to 10,000, that needs no real AWS.
- **Depends on:** B5.4 (the fault proxy), D4.1 (the bench generator), H2.1.
- **Change:** a stub endpoint that answers account discovery, role
  assumption and list calls for N account IDs, with a stated share of
  scopes empty, a stated number of resources per account, and the proxy's
  rate limit per account and region. `cmdb-bench generate` gains an
  `--accounts` size so the store and the graph can be filled without a
  sweep.
- **Prove it:** a sweep of the stub at N = 10 gives the same stored result
  as the emulator scenario of the same shape.
- **Done when:** every later step in this package can name its N.
- **Watch out:** the stub proves our side only: memory, queue, store and
  connection behaviour. It says nothing about real quotas. Not verified:
  whether the pinned emulator can present more than one account.

### S1.2 Rate budgets that exist only while they are used

- **Goal:** Memory for rate budgets depends on the accounts in flight, not on
  the size of the organization.
- **Depends on:** B1.2, B2.2.
- **Change:** 10,000 accounts, 17 regions and 436 services are about 74
  million budget keys, against a registry bound of 65,536 today. A budget is
  created when its first request is admitted and dropped when its account
  and region finish; its learned rate stays in `learned_rates`. The sweep
  keeps a bounded window of accounts in flight and moves through the
  organization.
- **Budget:** P7 (peak memory) is flat from N = 100 to N = 10,000 at a fixed
  window.
- **Prove it:** the S1.1 stub at N = 10,000 with the window at its default.
- **Done when:** peak memory at 10,000 accounts is within a stated margin
  of peak memory at 100.
- **Watch out:** `learned_rates` itself grows with the organization. Keep
  only rows that differ from the default.

### S1.3 Plan a window at a time, and store exceptions instead of everything

- **Goal:** Control data written per sweep is proportional to what was
  found and what went wrong, not to accounts times regions times operations.
- **Depends on:** S1.2, B4.5.
- **Change:** at 10,000 accounts there are about 373 million root listings
  per sweep. One `work_queue` row and one `scope_outcomes` row for each
  would be several hundred million rows of control data a day, far more
  than the resource data. Root scopes for an account and region are seeded
  when it enters the window and removed when it completes. A listing that
  completed and found nothing is counted per account, region and service
  and not stored as a row; every other outcome is stored as now.
- **Tests:** deletion still needs two complete listings (the evidence moves
  from a stored outcome row to the completion step, and the test proves a
  failed or skipped listing still deletes nothing); a kill at any point
  resumes without repeating a completed account and region.
- **Budget:** P8 (bytes written) per sweep at N = 1,000, before and after.
- **Prove it:** the kill matrix of B4.3 on the stub at N = 100.
- **Done when:** the rows written by a warm sweep of an unchanged
  organization do not grow with the number of empty scopes.
- **Watch out:** this changes what `scope_outcomes` means, so the coverage
  screen (D12.1) and X3's report read the counts as well as the rows.

### S1.4 Connections shared across accounts, and limits checked at start

- **Goal:** Open connections depend on regions and services in use, never on
  the number of accounts.
- **Depends on:** B7.2.
- **Change:** the endpoint for a service in a region is the same host for
  every account, and each request carries its own account's signature. The
  pool is keyed by endpoint, so 10,000 accounts reuse the same connections.
  At start the product reads the process's open-file limit, raises it where
  the system allows, sizes the pool and the account window to fit, and says
  so if the limit is the constraint.
- **Budget:** P4 (connections opened and requests reused) at N = 1,000.
- **Prove it:** connections opened in a sweep of the stub at N = 1,000
  compared with N = 10.
- **Done when:** the two counts are within a stated margin.
- **Watch out:** not verified: the default open-file limit for a
  double-clicked app on macOS, which is much lower than a server's.

### S1.5 Credentials for 10,000 accounts

- **Goal:** A role is assumed when an account enters the window, reused
  until it expires, and refreshed while in flight.
- **Depends on:** H2.1.
- **Change:** credentials are held only for accounts in the window. The
  role-assumption calls are paced by their own budget, because they are
  charged to the calling account, not to the 10,000 targets. An account
  where the role is missing is recorded once as inaccessible and remembered
  (B6.4), not retried every day at full cost.
- **Prove it:** the stub at N = 10,000 with a stated share of accounts
  refusing the role.
- **Done when:** the sweep completes with those accounts listed by name.
- **Watch out:** not verified: the real limit on role assumptions per
  second for one calling account.

### S1.6 The organization in the app: aggregates for all of it, resources for what is in view

- **Goal:** The app shows the whole organization at once (V70) without
  holding every resource in memory.
- **Depends on:** D4.3, D6.3.
- **Change:** the plan's "load the whole account once" does not hold for an
  organization. The app always holds counts per account, region, service
  and kind for everything, and edge totals between them, from a summary the
  store maintains. Individual resources and edges are loaded for the
  accounts in view, up to the budget in O14, and released when out of
  view. The level-of-detail steps (D6.2, D6.3) gain one level above the
  account: the organization.
- **Budget:** P12 (load time and memory held by the app) bounded whatever N
  is; the frame test of
  D7.2 run on an organization of 10,000 accounts at the top level.
- **Prove it:** the frame harness on a bench organization.
- **Done when:** the top level of 10,000 accounts meets X8's wording, and
  zooming into any one account loads it within a stated time.
- **Watch out:** "the entire organization at once" is true of the picture
  and not of the individual resources. If the owner needs every resource
  of every account interactive at the same moment, this step is not
  enough, and O14 has to be restated.

### S1.7 A slower cadence for scopes that stay empty, off by default

- **Goal:** A fallback that is designed and tested before it is needed.
- **Depends on:** S1.3, B6.4.
- **Change:** listing everything every day at 10,000 accounts is about
  4,300 requests a second around the clock for root listings alone. If the
  ladder shows that does not fit a day, a scope that has completed empty on
  a stated number of consecutive sweeps is listed weekly instead of daily.
  It returns to daily the moment it is non-empty. The switch is off until
  the owner turns it on (O22), because it means a resource created in a
  long-empty service can go unseen for up to a week.
- **Tests:** an empty scope drops to weekly; a resource appearing there is
  found at the weekly listing and the scope returns to daily; with the
  switch off nothing is skipped.
- **Prove it:** requests per sweep on the stub, switch off and on.
- **Done when:** both numbers are recorded and the trade is written where
  the user turns the switch on.

### S1.8 Partition the large tables by account, if it measures better

- **Goal:** The store stays fast as the organization grows.
- **Depends on:** C6.3, S1.1.
- **Change:** `resources`, `edges`, `resource_scopes` and the largest kind
  tables are generated as hash partitions on the account. Compare, on a
  bench organization at the 1,000-account rung, with and without: writer
  rows per second (P9), the app's reads (P12), database size (P8), and
  vacuum time.
- **Prove it:** `cmdb-bench sweep` and `cmdb-bench measure` on both layouts.
- **Done when:** the layout is chosen from the rows and recorded.
- **Watch out:** partitions multiply the number of tables, and there are
  already about 2,200 kind tables. Not verified: lock usage and planning
  time with partitioned kind tables on PostgreSQL 19.

### S2.1 The ladder: 10, 100, 1,000, 10,000

- **Goal:** Each rung is reached, measured and recorded before the next is
  attempted.
- **Depends on:** S1.1 to S1.6, S1.8, E3.4.
- **Files:** `docs/performance-evidence/scale.md` (new).
- **Change:** at each rung, on the stated laptop, record: cold and warm
  sweep time, requests, retries and throttles, peak memory, CPU time,
  connections, database size on disk, graph size in memory and on disk,
  time to open the app, and frame rows at the top level. There is no time
  window to fit: the sweep runs as fast as AWS allows (V73), and the rung
  records how fast that was and what the machine spent. A rung passes when
  the sweep completes correctly and no cost grew faster than the number of
  accounts.
- **Prove it:** the evidence file, one table per rung.
- **Done when:** every rung has its table, and for the highest rung that did
  not complete, what stopped it. Whether 10,000 accounts fit on one laptop
  is a result this step reports, not a condition it must meet (V74).
- **Watch out:** the first three rungs can be measured on the stub. Only a
  real organization shows real quotas, and the largest one available
  decides how far up the ladder the live evidence goes (L6.1).

## L The release gate

One step per line of [RELEASE_GATE.md](../RELEASE_GATE.md). These are
written at the level of the other steps here: what to do and what proves it.
File names marked "new" are proposals; the worker who takes the step fixes
them. None of this is started, and nothing in it has been run.

### L1.1 Install, upgrade, roll back, remove (R1)

- **Goal:** A person on a clean machine can install the product, sweep,
  upgrade with data in place, go back, and remove it.
- **Depends on:** D14.2 (packaging), C1.7 (migration journal), C6.2, E3.4
  (which graph engine ships).
- **Files:** `docs/INSTALL.md` (new), a harness scenario `release_upgrade`
  (new), the packaging scripts from D14.2.
- **Change:**
  1. Write the install path for macOS and Linux (V58): the binaries, PostgreSQL by container or native
     server, the two database roles, and the graph database, which is part
     of version 1 and ships inside the product (V57, V60). The user does
     not install it. How it ships depends on the engine E3.4 chooses: an
     engine linked into the binaries needs nothing more; a server has to
     be bundled, started, stopped and health-checked by the product.
  2. A scenario that installs build N, sweeps the seeded emulator, installs
     build N+1 whose catalog adds a kind and a column, checks that the
     journal completed and that per-kind counts are unchanged, then installs
     build N again.
  3. State what rollback means: the older binary refuses a newer schema
     unless the upgrade recorded itself as reversible. Additive changes
     (new columns, new tables) are reversible; anything else is not, and
     the upgrade says so before it starts.
  4. Removal: one command drops the `inventory` schema and the roles, after
     confirmation.
- **Tests:** the scenario, including a kill in the middle of the upgrade
  followed by a resume.
- **Prove it:** the scenario exits 0; a recorded run of `docs/INSTALL.md` on
  each platform by someone who did not write it.
- **Done when:** R1's evidence exists for macOS and for Linux.
- **Watch out:** "roll back" is undefined until item 3 is agreed. Do not
  promise a downgrade the journal cannot perform.

### L2.1 The read-only AWS policy, generated and proven minimal (R2)

- **Goal:** The published policy is exactly what the callable operations
  need.
- **Depends on:** A1.7, A7.2.
- **Files:** `crates/catalog-tool/src/iam_policy.rs` and the `catalog-tool
  iam-policy` command, both described in H2.2 and built here because a
  laptop user needs the policy too (V54); a harness test. The role template
  of H2.2 stays in package H.
- **Change:**
  1. Generate the IAM action list from the catalog's callable operations,
     using each model's service signing name and operation name. Where the
     IAM action differs from the operation name, the overlay carries the
     mapping and the generator fails if one is missing.
  2. The collector records every operation it attempted in a sweep. A test
     asserts that set is a subset of the generated policy.
  3. The policy is split to fit IAM's size limits; the split is generated.
- **Tests:** an operation added to the catalog appears in the policy; an
  operation in the deny list never does; the attempted set never exceeds
  the policy.
- **Prove it:** `catalog-tool iam-policy --partition aws --account 123456789012 --check`; on the real-account run
  (L6.1), a sweep under exactly that policy with zero access-denied
  outcomes that the policy should have allowed.
- **Done when:** R2's evidence exists.
- **Watch out:** the emulator does not enforce IAM, so minimality is only
  proven on a real account. Not verified: how many operations have an IAM
  action whose name differs.

### L2.2 Three database roles with tested limits (R3)

- **Goal:** The product never needs a superuser after setup.
- **Depends on:** C4.1 (the reader role), D13.1 (two URLs).
- **Files:** the store-creation code from C6.4 and C4.1, a harness test.
- **Change:** setup creates `inventory_collector` and `inventory_reader` as
  the owner, grants each exactly what DATA_MODEL section 8 lists, and
  records the grants. Every command states which role it needs and refuses
  to run with more than it needs where that is checkable.
- **Tests:** the collector role cannot create or drop tables outside the
  migration path; the reader cannot write and cannot read the work queue,
  tokens or outbox; a sweep, a query and an upgrade each succeed with the
  role they claim.
- **Prove it:** the role test in the harness.
- **Done when:** R3's evidence exists.

### L3.1 Authentication to both stores, and verified TLS off the local machine (R4)

- **Goal:** On the laptop both stores are reachable only from the laptop
  and need a password; anything not local is refused unless it is verified
  TLS.
- **Depends on:** C6.2, D13.1.
- **Files:** connection setup in the store and graph clients, a release
  configuration profile, harness tests with a local certificate authority.
- **Change:** version 1 is one laptop (V54), so the default policy is
  local: both stores bound to the loopback address or a local socket, each
  with a password, and the installer never opens either to the network. A
  connection to a host that is not local is allowed only with TLS and
  certificate and host verification. The status line names which of the
  two is in use.
- **Tests:** a wrong password; a store found listening on a public
  address; and, for a non-local host, a wrong certificate, a wrong host
  name and a server that offers no TLS. Each is refused with a message that
  names the cause and prints no secret.
- **Prove it:** the five refusal tests.
- **Done when:** R4's evidence exists.
- **Watch out:** not verified: which TLS options the pinned PostgreSQL and
  graph client libraries expose.

### L3.2 No secret in any output (R5)

- **Goal:** Passwords, tokens, access keys and signed headers never reach
  logs, errors, crash output, exports or the UI.
- **Depends on:** D13.1.
- **Files:** the logging and error types, a harness scenario
  `secret_canary` (new).
- **Change:** secret-bearing values get a type whose display form is
  redacted, so printing one by accident prints nothing useful. Connection
  URLs are printed without their password. Request logging never includes
  the `Authorization` header or session token.
- **Tests:** the scenario seeds known canary strings as the database
  password, the AWS secret key and the session token, runs a sweep, forces
  an error on each path, runs an export, and searches every file and stream
  the run produced for the canaries.
- **Prove it:** the scenario finds zero canaries.
- **Done when:** R5's evidence exists.

### L4.1 Backup and restore (R6)

- **Goal:** A documented backup restores into an empty server, and the
  graph is rebuilt from it.
- **Depends on:** E5.1 (the comparer), E4.2 (rebuild).
- **Files:** `docs/BACKUP.md` (new), a harness scenario.
- **Change:** document a `pg_dump` of the `inventory` schema and its
  restore; state that the graph database is not backed up because it is
  rebuilt. The scenario dumps after a sweep, restores into a new database,
  rebuilds the graph, and compares.
- **Tests:** per-kind counts, edge counts and content hashes equal before
  and after; E5.1 reports graph equals SQL on the restored pair.
- **Prove it:** the scenario exits 0.
- **Done when:** R6's evidence exists.

### L4.2 Outages, during a sweep and at rest (R7)

- **Goal:** With PostgreSQL down, the graph database down, or AWS
  unreachable, the product says what is wrong, loses nothing, and recovers.
- **Depends on:** B4.6, E4.3, F5.1.
- **Files:** harness scenarios; the app's state screens from D13.2.
- **Change:** no new mechanism. This step runs the six cases (three
  outages, each during a sweep and at rest) end to end through the CLI and
  through the app, and records what the user sees.
- **Tests:** for each case: the message names the failed dependency; no
  resource row is lost or duplicated; the sweep resumes when the dependency
  returns, without restarting the process where B4.6 and E4.3 say it
  should.
- **Prove it:** the six scenarios.
- **Done when:** R7's evidence exists.

### L5.1 Provenance, dependencies and licences (R8)

- **Goal:** Release artifacts can be reproduced and checked, and every
  dependency's licence fits a product that is MIT throughout (V63).
- **Depends on:** E3.4 (the graph engine), K1.1.
- **Files:** release scripts, an audit configuration (new), a licence
  report (generated).
- **Change:** build release artifacts from a locked toolchain and lock
  file, publish checksums and a signed manifest, run an advisory audit and
  a licence check as gate tasks, and review the result against V48.
- **Tests:** two builds of the same commit give the same checksums, or the
  differences are listed and explained; the audit task fails on a seeded
  vulnerable version.
- **Prove it:** the gate tasks; the published manifest.
- **Done when:** R8's evidence exists.
- **Watch out:** the graph engine is the first item on the review. It ships
  inside the product (V60), and FalkorDB is under the Server Side Public
  License (SSPLv1). If E3.4 keeps FalkorDB, whether it may ship with an
  all-MIT product is for a lawyer, not for this plan. Not verified: whether
  the build is reproducible today.

### L5.2 One diagnostics command (R9)

- **Goal:** An operator gets everything support would ask for from one
  command, with no secret in it.
- **Depends on:** C4.1, D12.1, L3.2.
- **Files:** a `doctor` subcommand of the CLI (new); the same report as a
  screen in the app.
- **Change:** report versions (binary, catalog digest, schema, PostgreSQL,
  graph database), each connection and the role it holds, the last sweep
  and its outcome counts, coverage, queue depth, outbox depth, and store
  size. Every check is bounded by a timeout and reports "unknown" instead
  of hanging.
- **Tests:** each dependency down in turn; the L3.2 canary search over the
  output.
- **Prove it:** `aws-inventory doctor --database <URL>` on a healthy and on
  a broken installation.
- **Done when:** R9's evidence exists.

### L6.1 The authorized real-account run, as release evidence (R10)

- **Goal:** The first run against a real account, read-only, with every
  difference from the emulator written down.
- **Depends on:** J2.1, L2.1, and the owner naming the account (O8).
- **Files:** a dated evidence file.
- **Change:** J2.1 is the run; this step is its release form: under the
  generated policy from L2.1, with the release connection policy from
  L3.1, recording coverage, outcomes per scope, learned rates, and each
  endpoint, signature, paging or decode difference.
- **Prove it:** the evidence file, and a second sweep of the same account
  that changes no resource state other than what really changed.
- **Done when:** R10's evidence exists.
- **Watch out:** nothing is run against a real account until the owner
  names it. This is the first time permissions and quotas are real.

### L6.2 Published limits (R11)

- **Goal:** What a user is told matches what was proven.
- **Depends on:** A7.2, L6.1.
- **Files:** `docs/LIMITS.md` (new, generated where possible).
- **Change:** generate from the catalog and the evidence: what is never
  collected and why, each kind's status (implemented, verified on the
  emulator, verified live), the largest account swept, and the largest
  graph drawn at the stated frame rate with the hardware it was drawn on.
- **Tests:** the generated lists equal the catalog's; a kind with no live
  evidence is never listed as verified live.
- **Prove it:** the generator's check mode.
- **Done when:** R11's evidence exists.

