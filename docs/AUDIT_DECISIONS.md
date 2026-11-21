# External audit decisions

> Status: Current. Records the owner decisions taken on external audit findings
> (UI-07, UI-12, UI-15, UI-16, UI-17, UI-18) and on findings against the plan
> for a future PostgreSQL rewrite (ES-03, ES-04, ES-05, ES-06, ES-12, COV-06,
> UI-09). It gives the state of each in this tree and the evidence checked.
> Each entry states what was verified in code and what was not. The audit
> reviewed a plan that named PostgreSQL; this product is SQLite plus FalkorDB,
> so PostgreSQL-specific parts do not apply.
> Decisions here do not change the requirements in [VISION](VISION.md) or the
> milestones in [ROADMAP](ROADMAP.md); where they touch them, the entry says so.

Status values: **decided** (owner decision recorded, nothing in code required),
**implemented** (code or docs in this tree already do it), **deferred** (decision
recorded, work not started; the gate is named).

| ID | Topic | Status |
| --- | --- | --- |
| UI-15 | Trust boundary for remote stores | decided |
| UI-16 | Superseded generations and licensing | implemented in part; per-crate licensing deferred |
| UI-12 | Frame-time target and measurement | decided; measurement deferred to M6.3 and later |
| UI-17 | Legacy paths before deletion | decided; parity matrix deferred to M9.5 |
| UI-18 | Real-account proof timing | decided; canary placement deferred to O8 |
| UI-07 | Graph backend contract | decided; interface deferred to M6.1 and E3 work |
| ES-03 | Storage contract for the PostgreSQL rewrite | adopted as binding requirement for the PostgreSQL backend; no code change in the SQLite/FalkorDB implementation |
| ES-04 | Dirty dimensions when content hashes are equal | adopted as binding requirement for the PostgreSQL backend; SQLite code already keeps the dimensions apart |
| ES-05 | Unlogged seen set and reconnecting collectors | adopted as binding requirement for the PostgreSQL backend; no code change in the SQLite/FalkorDB implementation |
| ES-06 | COPY staging across grouped page savepoints | adopted as binding requirement for the PostgreSQL backend; no code change in the SQLite/FalkorDB implementation |
| ES-12 | Stable SQL column names under appended fields | adopted as binding requirement; current SQLite naming already meets it (ID-derived, no collision found) |
| COV-06 | SQL identifier contract for the PostgreSQL target | adopted as binding requirement for the PostgreSQL backend; SQLite names are checked and never truncated |
| UI-09 | Lossy progress delivery | adopted as binding requirement for the planned library extraction; no progress `try_send` path exists in this tree |

## UI-15: trust boundary

**Status: decided.**

**Decision.** The product is a single-trust-domain, direct-access tool. The
operator runs the viewer and the CLI on one machine. The viewer reads the local
SQLite store and the operator's own FalkorDB with the operator's own
credentials. There is no multi-tenant authorization layer, no per-user identity,
and no account or tenant isolation inside one store. Anyone who can read the
SQLite file or reach the FalkorDB URL can read everything in it.

Access to a remote store (a database or graph reached over the network, shared
by several people) is out of scope until a separate authenticated service is
designed. That service would enforce authorization on every SQL, graph, detail
and export query; it would also need short-lived credentials, TLS identity
checks, revocation and cache clearing. None of that is designed or built here.
UI sign-in alone is not an authorization boundary and must not be presented as one.

**Evidence.**
- `crates/cmdb-ui/README.md` describes the viewer as read-only and local.
- `crates/cmdb-view/src/graph.rs` and `crates/cmdb-view/src/error.rs` contain the
  in-process graph and view code; no authentication code exists there.
- The HTTP read API that could have carried remote access was removed (ROADMAP
  M6.5, `crates/cmdb-api` absent). `docs/CMDB_ARCHITECTURE.md` marks its
  authn/authz design as superseded by V32.
- `crates/cmdb-ui` has a worker test that asserts no listening socket
  (`crates/cmdb-ui/src/worker/tests/no_socket.rs`), which is the only
  network-boundary control in the viewer.

**Not verified.** Credential scoping guidance for operators (IAM and FalkorDB
ACLs) is not written in this tree.

## UI-16: superseded generations and licensing

**Status: implemented in part.**

**Decision.** Historical documents that describe earlier generations keep their
text and gain a superseded status line that points to the current document. A
document is marked only when its own header or the ROADMAP already says it is
replaced. Current statements are not rewritten in this pass.

Marked as superseded or replaced in the document header today (verified by
reading the first lines of each file):
- `docs/CMDB_ARCHITECTURE.md`: "Plan (superseded draft)"; replaced at M10.3.
- `docs/CMDB_PERFORMANCE_CONTRACT.md`: "Plan (superseded draft)".
- `docs/ARCHITECTURE.md`: "Baseline being replaced".
- `docs/cmdb-spec/`: listed as superseded and frozen in `docs/README.md`.

Partly stale, status line says "Kept and extended" with replacement notes
(not marked superseded): `docs/DELIVERY.md`, `docs/GRAPH_DELIVERY.md`,
`docs/STORAGE.md`.

Schema language is not consistent in the tree. Observed values:
- `crates/engine/src/store/schema.rs`: `SCHEMA_VERSION: i64 = 13`.
- `crates/engine/src/inventory.rs`: error text says "expected schema17
  (schema14/15/16 upgrade on open)".
- `crates/cmdb-ui/README.md` and `crates/cmdb-ui/Cargo.toml`: "schema16".
- `crates/cmdb-bench/src/main.rs`: writes `schema_version=15`.

These cannot all be right. Which schema the code accepts is not settled by this
document; it must be checked against the code before any doc states a number.
The `crates/cmdb-ui` README and description are outside the docs lane and are
not edited here.

**Licensing.** `LICENSE` exists at the repository root (MIT, "Copyright (c) 2026
Lewis"). The workspace sets `license = "MIT"` in the root `Cargo.toml`, and every
crate inherits it with `license.workspace = true`. This is the only license
declared. The commercial split that the audit mentions is not represented, and
no commercial module boundary exists. Per-crate licensing is **deferred** until
an ownership review decides which crates are distributed under which terms.

## UI-12: frame time and performance measurement

**Status: decided; measurement deferred.**

**Decision.** The retained target is the audit's: p99 frame time at or below
8.3 ms (120 FPS) at 250,000 nodes and 1,000,000 edges on an integrated GPU.
A 60 Hz fallback does not satisfy this target. The target is not in this
repository's documents: the audit cites `docs/e2e/build/STEPS_D.md`, which does
not exist in this tree, and `docs/CMDB_PERFORMANCE_CONTRACT.md` uses a different,
superseded budget (p95 under 16.7 ms at 5,000 nodes and 20,000 edges). The
250,000/1,000,000 target is therefore recorded here as the decision, and it
replaces the superseded budget for the native UI.

Required measurement dimensions (from the audit), with what the tree measures
today:

| Dimension | Measured today | Evidence |
| --- | --- | --- |
| Interval between frames (elapsed ms, node count per frame) | Yes, via `CMDB_UI_FRAME_LOG` | `crates/cmdb-ui/src/frame_time.rs`, wired in `crates/cmdb-ui/src/app/actions.rs` |
| CPU draw cost per frame | No | not timed separately |
| GPU completion | No | none |
| Present cadence and vsync effects | No | the frame log is an interval between draw calls, not a present |
| Missed frames against a budget | No; the log has no budget or count | none |
| Input latency | No | none |
| Load and layout time | No | none |
| Peak CPU and GPU memory | No | `cmdb-bench` has no RSS or GPU memory measure (checked with grep for `rss`, `peak`, `cpu`) |
| Percentiles (p50, p95, p99) | No; the frame log is raw lines, no summary | `crates/cmdb-bench` times stages with `Instant` but reports no percentiles |

`cmdb-bench` times pipeline stages with `Instant` (for example
`crates/cmdb-bench/src/pipeline.rs`); it is a throughput and correctness tool,
not a frame harness. `crates/cmdb-ui/src/dag/widget/paint.rs` draws the DAG but
has no timing in it.

**Required before the target can be claimed:** a scripted interaction, a
declared hardware, driver, resolution and refresh rate, a fixture at the stated
size (with high-degree and clustered or cyclic graphs, labels and selection on),
cold and warm runs, raw samples, scripts and artifact hashes. Only the scenarios
that change need re-running. **Deferred** to the M6.3 frame-time trace and to the
native-GPU evidence in M10; no measurement in this tree meets the target.

## UI-17: legacy paths before deletion

**Status: decided; parity matrix deferred to M9.5.**

**Decision.** No legacy path is deleted until a parity matrix exists that lists
each product behavior, marks it ported, retired (with compatibility guidance), or
still required, and gives golden output or invariant evidence for each ported
behavior. Deleting a legacy path without that matrix is a regression risk and is
not allowed.

Legacy code present and wired from the CLI (`src/main.rs`):

| Path | Behavior it provides | Wired by |
| --- | --- | --- |
| `src/analysis/diff.rs` | Observation-aware diff of two scans | `analysis::diff` (`src/main.rs`) |
| `src/analysis/findings.rs` | Evidence-qualified findings | `src/analysis` tests and graph code |
| `src/analysis/graph.rs`, `src/analysis/graph/` | Directional bounded traversal (neighborhood) | `analysis::neighborhood`; `TraversalOptions` in `src/cli.rs` |
| `src/architecture.rs` | Finalized-scan architecture panels (counts, coverage, relationships with observation flags) | `architecture::report` (`src/main.rs`) |
| `src/delivery/` (`attempt.rs`, `publication.rs`) | Atomic export, durable graph plan, ACK and retry, delivery receipts | `delivery::command`, `delivery::export_command` (`src/main.rs`); `crate::delivery::validate` in `src/jobs.rs` |

Each of these is tied to the `scan`-based, schema13-era path. The replacement
(bounded SQL reads in `cmdb-view`, the pending-projection queue) is ROADMAP
M6.1, M6.2 and M9.5. Behaviors that must have a parity entry before deletion:
snapshot and scan identity, partiality (what an incomplete scope reports),
export atomicity, graph acknowledgement and replay after a lost reply,
observation-aware diff, and evidence qualification of findings.

**Not verified.** Whether `cmdb-view` or the CLI reproduces each behavior above
is not established by this document. The matrix itself has not been written.

## UI-18: real-account proof timing

**Status: decided; canary placement deferred.**

**Decision.** Real-account proof is the release gate. The ROADMAP keeps it as
M10.1 (read-only run of every enabled service on the authorized account) and
M5.8 (first real-account run for the wave-5 services). The account is owner
item O8 (`docs/VISION.md`), named by the owner; no account is available to the
repository. M5.8 is stated not to block M6 through M9. The audit asks for an
earlier minimal read-only canary after wire and identity readiness. That
placement is an open owner choice; this document does not move the ROADMAP
milestone.

**What runs today on the emulator** (verified by the ROADMAP's own status
entries and `docs/EMULATOR_TESTING.md`): the pinned AWS emulator with real
SigV4-signed requests, the `inventory-harness` seeding and proxy tests, and the
ignored live emulator tests the ROADMAP records. Emulator evidence does not
prove endpoint resolution, authentication, IAM permissions, quotas or
pagination against AWS. The ROADMAP states that emulator and real-account
evidence are reported separately.

**Not verified in this document.** Whether CI runs the release gates is not
checked here. No fresh Rust build or test result is cited; this document
makes no claim that a build passes.

**Also required by the audit:** release manifests, SBOM and provenance per
version, and head-to-head comparisons run under identical account, regions,
permissions and coverage (ROADMAP M10.2). These are not started.

## UI-07: graph backend contract

**Status: decided; interface deferred.**

**Decision.** The graph backend is specified by its guarantees, not by a shared
Cypher string or a single trait method. Any backend (today FalkorDB; later
Neptune or PostgreSQL-distributed projectors) must satisfy the following before
it is used for projection:

1. **Ownership and fencing.** A projector holds an ownership claim that is
   fenced by a durable generation. Effects from an old owner are rejected.
2. **Ordering.** Batches apply in the order their generations and keys define.
3. **Idempotent retry.** A replayed batch after a lost reply produces the same
   graph state and the same acknowledgement.
4. **Generation storage.** The committed generation is stored durably with the
   local store, so a restart cannot reuse a generation (no ABA).
5. **Consistent reads.** A read reports the generation it reflects, so a
   reader can tell a partial rebuild from a complete one.
6. **Cancellation.** A cancelled rebuild leaves a state that the next owner can
   resume or discard.

**How this applies to FalkorDB today** (verified in code):
- `crates/engine/src/inventory/projection/client/fence.rs` claims ownership with
  `CLIENT ID`, a Lua `EVAL` that compares fixed-width decimal generations
  (`{generation:020}:{owner}`), and `WATCH`/`MULTI`/`EXEC` around the effect.
  A missing ownership key fails closed. The comment there states that durable
  generations prevent ABA.
- `crates/engine/src/inventory/projection/runner/ownership.rs` takes a
  process-lifetime exclusive lock on the local SQLite file (`locking_mode`
  EXCLUSIVE) and allocates a durable generation in `delivery_generation`.
  Ownership has no TTL; it lasts until the connection or process dies.
- `crates/engine/src/inventory/projection/client.rs` runs index creation inside
  the fenced `MULTI` pipeline.
- `crates/engine/src/graph/delivery` holds the acknowledgement and retry path
  described in `docs/GRAPH_DELIVERY.md`.

Ownership is therefore tied to the local SQLite file and to Redis-protocol
primitives. This is the design that the audit says must move into the storage
and backend contract.

**Neptune and other backends.** A Neptune backend requires a conformance suite
first. That suite must show the six guarantees above, including lost-reply
replay, delayed stale claim, old owner after new owner, and exact-revision
acknowledgement, before any Neptune code path is written. No Neptune code exists
in this tree.

**Deferred:** the backend trait with these capabilities is not designed here.
The owning milestones are ROADMAP M6.1 (pending projection) and the graph
abstraction in E3 of the original plan.

## Plan-only findings (PostgreSQL rewrite plan)

These seven findings were raised against a plan for a PostgreSQL rewrite. That
plan is not in this repository. Each is adopted as a binding requirement for the
PostgreSQL backend (UI-09 for the planned library extraction) when that work is
written. None requires a change to the SQLite or FalkorDB code. Where a claim
about current code was checked, the entry cites the paths read. No real defect
in current SQLite code was found for ES-12 or COV-06.

## ES-03: storage contract is out of date

**Finding.** The plan names the source schema14 and specifies a minimal trait,
page and deletion contract, but the source has authority epochs, tag coverage,
replaceable facets, request context, branch authority, typed reference targets,
reviewed evidence and migration rules that such a contract does not cover.

**Status: adopted as binding requirement for the PostgreSQL backend; no code
change in the SQLite/FalkorDB implementation.**

**Evidence.**
- `crates/engine/src/inventory/schema.rs` writes `PRAGMA user_version` 17 (lines
  42 and 92). The audit's schema14 and schema16 are stale for this tree.
- `crates/engine/src/inventory/schema.rs` line 6 defines `tag_observations` with
  a `coverage` column.
- The authority, facet and relation-permission modules the finding names exist:
  `crates/engine/src/inventory/authority.rs`, `facets.rs`, and
  `crates/engine/src/inventory/relation_permissions/`. Their behavior was not
  re-traced here.

**Not verified.** The parent-deletion rule that keeps children with other live
owners, and branch-authority behavior.

**Acceptance tests for the PostgreSQL backend.**
- Plan paths, version, public methods and schema inventory are rebased to the
  exact reviewed commit before any port starts.
- A backend-independent conformance suite covers every current authority, tag,
  facet and replay contract, using the current algorithms as the oracle.
- Crash tests pass on the current store before its storage is swapped.
- Parent deletion keeps every child that still has another live owner.

## ES-04: equal content hashes do not justify skipping writes

**Finding.** C6.6 keeps only rows whose content hash changed. The content hash
excludes selected paths, so a single-hash filter would keep stale SQL columns,
stale detail and stale relations.

**Status: adopted as binding requirement for the PostgreSQL backend; no code
change in the SQLite/FalkorDB implementation. The SQLite commit path already
keeps the dimensions apart.**

**Evidence.**
- `crates/engine/src/inventory/facets.rs` line 34 returns separate
  `relations_changed`, `record_changed` and `bound_context_changed` values.
  Lines 102, 107 and 113 compare stored relation bytes, record bytes and bound
  request bytes against the new values.
- `crates/engine/src/inventory/commit.rs` line 149 decides `changed` from the
  content hash and name. Lines 159-161 refresh the typed record with the
  preserved `last_changed` and invalidate incoming matches on request-context
  change. Lines 167-173 rebuild outgoing relations when `relations_changed` is
  set.
- Tags live in their own table (`crates/engine/src/inventory/schema.rs` line
  6, `TAG_OBSERVATIONS_SQL`) and are not gated by the content hash. Tag write
  paths were not traced.

**Acceptance tests for the PostgreSQL backend.**
- Equal semantic hash with a changed excluded field: typed projection and detail
  are refreshed and `last_changed` is unchanged.
- Equal semantic hash with a changed checked request context: incoming matches
  are invalidated.
- Equal semantic hash with changed relations: outgoing relations are rebuilt.
- Tag and name changes are tracked independently of the content hash.
- Replay and multi-source merge order produce the same final state.
- Each dimension uses a conditional upsert, so an unchanged dimension writes
  nothing.

## ES-05: unlogged seen set and reconnecting collectors

**Finding.** A collector can survive a server crash, reconnect, and continue a
lease or cursor after an unlogged seen set has been truncated. If that path
skips the reset, completion uses incomplete membership evidence.

**Status: adopted as binding requirement for the PostgreSQL backend; no code
change in the SQLite/FalkorDB implementation.**

**Evidence.** A search of `src/` and `crates/` finds no `UNLOGGED` table. The
server-crash case therefore does not arise in the shipped SQLite store as
described. Reconnect and staging-loss handling was not traced.

**Acceptance tests for the PostgreSQL backend.**
- A server or recovery epoch, or a reconnect barrier, invalidates in-flight
  authority after staging loss, before any further claim or commit.
- After staging loss, a complete re-enumeration is forced before any deletion,
  and the durable metadata that forces it is persisted.
- Separate tests for client kill, server crash with the collector alive,
  commit-reply loss, cancellation, and real I/O failure (for example disk full).
  A statement timeout does not count as I/O failure.
- Durable commit settings stay on until exact recovery and projection
  reconciliation are proven.

## ES-06: COPY staging lifetime across grouped page transactions

**Finding.** C6.7 stages rows in a per-connection table with `ON COMMIT DELETE
ROWS`. When several pages run under savepoints in one outer transaction, a
successful page's staged rows survive until the outer commit and can be merged
again by the next page of the same kind.

**Status: adopted as binding requirement for the PostgreSQL backend; no code
change in the SQLite/FalkorDB implementation.**

**Evidence.** A search of `src/` and `crates/` finds no `COPY` staging and no
`ON COMMIT` table. Resource writes are direct upserts, for example the
`INSERT INTO resources ... ON CONFLICT` at `crates/engine/src/inventory/commit.rs`
line 353.

**Acceptance tests for the PostgreSQL backend.**
- Two successful pages of the same kind in one outer transaction do not merge
  the first page's staged rows again.
- The same resource appearing in two pages produces one result under a
  documented ordering rule for same-key observations.
- A failed savepoint followed by a retry produces no duplicate-input upsert
  failure.
- Outer rollback removes all staging rows, including those of released
  savepoints.
- Concurrent writers do not see each other's staging rows.

## ES-12: flattened SQL column names under appended fields

**Finding.** The plan gives the unsuffixed name to the shallowest colliding path.
A later, shallower field can therefore rename an existing column. Append-only
field IDs do not prevent this, because depth outranks IDs.

**Status: adopted as binding requirement; current SQLite naming already meets it.
No collision was found.**

**Evidence.**
- `crates/core/src/inventory/schema.rs` `Column::new` (lines 35-48) builds the
  name from field IDs only: `f`, then `_<id>` for each path element. Names do not
  depend on depth, field name or insertion order. Path names contain only digits
  and single underscores, so distinct paths give distinct names.
- `Column::selected_scalar` (lines 50-56) is named `value`, which cannot collide
  with an `f_`-prefixed name.
- `canonical_columns` (lines 99-108) sorts by path and rejects duplicate paths.
- `crates/engine/src/inventory/schema.rs` `append_column` (lines 341-355) adds the
  suffixes `__nonfinite`, `__seconds` and `__nanos` for Real and Timestamp. These
  suffixes contain `__`, which generated path names never contain, so no
  suffixed name can match another column.
- `create_kind` (lines 303-322) uses `column.name()` directly in the kind-table
  DDL. Names are not stored: `decode_column` (lines 426-441) rebuilds each name
  through `Column::new` on open.
- `ALTER TABLE` statements in `crates/engine/src/inventory/` (`authority.rs`
  line 14, `legacy.rs` lines 24 and 28, `relation_permissions/provenance/schema.rs`
  lines 18-26, `schema.rs` line 67) touch shared tables such as `resource_scopes`,
  `legacy_authorities`, `reviewed_reference_grants`, `reviewed_reference_evidence`
  and `work_queue`. None targets a kind table.

**Not verified.** That field IDs are stable across catalog versions. Name
stability depends on that.

**Acceptance tests for the PostgreSQL backend.**
- Appending a shallower field that collides with an existing name keeps the
  existing column's name.
- Acronym and underscore collisions (for example `IPv6` against `ip_v6`) produce
  distinct names.
- Truncation keeps names unique.
- Reordering fields does not rename any column.
- A removed field, and the same field reintroduced later, map to the same column
  or are rejected explicitly by the contract.

## COV-06: SQL identifier contract for the PostgreSQL target

**Finding.** Planned PostgreSQL names may exceed the 63-byte bound and collide
after truncation.

**Status: adopted as binding requirement for the PostgreSQL backend; no code
change in the SQLite/FalkorDB implementation. The current generator rejects
over-length names instead of truncating them.**

**Evidence.**
- `crates/catalog-tool/src/names.rs` line 14 sets `MAX_SQL_NAME_BYTES` to 128.
- `crates/catalog-tool/src/names/identifier.rs` (lines 23-29) and
  `crates/catalog-tool/src/names/output.rs` (lines 44-50) return `TooLong`.
  Nothing under `crates/catalog-tool/src/names*` truncates a name.
- `crates/catalog-tool/src/names/render.rs` line 29 returns `DuplicateSqlName`
  when two type names derive the same SQL name, so a collision fails the build.
- `crates/core/src/inventory/identifiers.rs` `sql_name` (lines 142-151) requires
  lowercase ASCII, digits and underscores, so SQLite's case-insensitive matching
  cannot make two names collide.
- `crates/engine/src/inventory/schema.rs` `create_kind` (line 322) issues a plain
  `CREATE TABLE`. A kind table whose name matches an existing table fails store
  initialization instead of reusing the table.

**Not verified.** Whether the pinned model can yield a non-AWS type name that
derives to a shared table name. The failure is fail-closed in any case.

**Acceptance tests for the PostgreSQL backend.**
- A versioned identifier algorithm appends a hash suffix to names over the limit.
- Two long names with the same truncated prefix receive distinct names.
- Uniqueness is checked across every SQL object category: tables, indexes,
  constraints, views and columns.
- The full generated name inventory is tested for uniqueness and length before
  any store migration runs.

## UI-09: lossy progress delivery

**Finding.** D1.3 sends each `ProgressEvent` with `try_send` and drops it when
the bounded channel is full, yet expects `Started` first and `Finished` last. A
full channel can drop the completion event and leave the UI waiting forever.

**Status: adopted as binding requirement for the planned library extraction; no
code change in this tree.**

**Evidence.** A search of `src/` and `crates/` finds no progress `try_send`. The
only `try_send` in `crates/` is `crates/cmdb-ui/src/worker.rs` line 161, which
sends `Command::Stop`. No `ProgressEvent` type appears in `src/` or in
`crates/` outside `crates/harness`, where it is the AWS SDK type. The pipeline
in `src/scan/pipeline.rs` uses an awaited `sender.send(event).await` (line 107),
which waits rather than drops.

**Not verified.** Whether the CLI cancellation drains admitted work as the
finding states.

**Acceptance tests for the planned library extraction.**
- With the progress channel full, the terminal event (finished, failed or
  cancelled) is still delivered.
- Progress is latest-value: intermediate events may be replaced, but the most
  recent one is kept.
- The sweep future's return value is the authoritative outcome; the UI never
  waits after the future completes.
- Cancellation cancels and drains admitted work before the terminal state is
  reported.
- Start, cancel and resume ownership stay explicit, and the UI event thread never
  blocks on completion.

## Open items

- `crates/cmdb-ui/README.md` and `crates/cmdb-ui/Cargo.toml` still state schema16
  and the viewer description; outside the docs lane.
- The schema number must be reconciled across `crates/engine` and
  `crates/cmdb-bench` before any document states one.
- Per-crate licensing and the commercial split need an owner decision.
- The frame harness, parity matrix, real-account canary and graph conformance
  suite are not built.
