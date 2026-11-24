# Inventory graph contract

`inventory-graph` is an MIT, backend-neutral port depending only on
`inventory-core`, Redis, Tokio, futures-util and thiserror. Its sole implementation is
`falkor::FalkorDb`. Inventory projection and `cmdb-view::GraphConnection`
use `GraphBackend` with static dispatch. The nine resource operations are
extended with typed tag reads/writes, projection ownership and target identity
so migration retains tag affiliations, fencing and rebuild detection.

## Query language

All query text in the new crate lives in `src/cypher.rs`. It uses these
openCypher constructs, not procedures or engine-specific Cypher functions:

- `MATCH`, `OPTIONAL MATCH`, labels, directed and undirected relationships.
- `MERGE`, property maps, `SET`, `DELETE`, `DETACH DELETE`.
- `$parameters`, strings, nulls, lists and integer literals.
- `WHERE`, comparisons, `AND`, `OR`, `NOT`, `IN`, `IS NOT NULL`.
- `WITH DISTINCT`, `RETURN`, `ORDER BY`, `LIMIT`.
- `FOREACH` with `CASE WHEN ... THEN [1] ELSE [] END` for conditional merges.
- `UNWIND`, `startNode` and `endNode` for bounded frontier reads preserving
  stored edge direction, including parallel relationships and self-loops.

There are no variable-length path queries. Bounded frontier queries and a
Rust breadth-first calculation implement traversal and shortest paths. Frontier
and excluded-node digest lists are parameters, not repeated query-body literals.
Future variable-length patterns must have explicit lower and upper bounds. The
`CYPHER name=value` parameter preamble is FalkorDB transport syntax, not part
of the portable query body. Neptune would bind the same parameters through
its own transport; no Neptune runtime conformance is claimed here.

## Identity, replacement and pagination

A `NodeKey` is the core `ResourceKey`, including account, region, kind and id.
The stored `key` is its existing full scoped digest, encoded in lowercase hex.
`NodeRow` mirrors the existing projection row (key, optional name, hash).
Edges retain their relation label, relation property and bounded provenance.
`EdgeKey` is `EdgeRow`: all four fields participate in edge identity.

`replace_edges` removes **all incident resource edges**, incoming and outgoing,
and merges the supplied incident edges. Up to 1,000 edges sharing one relation
label use a single atomic query; larger or mixed-label sets clear once and
append bounded batches, with the same partial visibility and replay semantics
as streaming replacement. Unrelated edges fail
before I/O. A missing owner is an error, including for an empty replacement;
missing other endpoints are skipped without suppressing later edges. Empty
input removes all incident edges. Supply the complete set, not merely the node's
outgoing edges. Tag edges are not resource edges.

Node pages are ordered by digest, with an exclusive cursor. Edge pages use
(source digest, target digest, relation, provenance), also exclusive. Cursors
are not SQL lexical key order and pages are not a retained snapshot: writers
must be quiescent when checking equality across stores. Missing nodes delete
idempotently; node upserts and full edge replacement are replayable.

## Bounds and failure semantics

- Query text: at most 1 MiB; growth uses fallible reservation.
- Input strings: at most 4,096 bytes; relation/provenance at most 128 bytes.
- Replacement/page: at most 2,000 edges/rows; invalid limits fail before I/O.
- Traversal: depth 1–3, at most 500 nodes and 2,000 edges, at most four layer
  reads including the depth-boundary probe, then one bounded pass for all
  edges between selected nodes. Completion retains breadth-first edges and
  appends previously unseen edges in source/target/relation/provenance order;
  only an actually refused edge marks the edge budget exhausted.
  Node/edge vectors reserve their bounds.
- Neighbors traverse both directions; blast radius follows incoming edges
  (A -> B means A refers to B). Edge direction is preserved in results.
- `Truncation` distinguishes complete, depth-limited and budget-limited reads.
  Depth truncation requires an endpoint outside the visited set; cross edges
  between visited nodes do not make a complete induced neighborhood truncated.
  A path is shortest within the requested depth; no path within that depth is
  `NotFound`. Budget-limited absence is `GraphError::Truncated`, never a false
  claim that no path exists. Identical source and target yield `SameNode`.
  The view exposes budget-limited absence as `PathResult::Truncated`, distinct
  from `NotFound`, without hiding transport or decoding errors.
- Clear deletes at most 10,000,000 vertices and fetches at most one sentinel row
  to check for leftovers; a graph above that cap returns a limit error.
- Each backend admits at most 16 commands; admission plus connection/query
  takes at most ten seconds. Each traversal also has a ten-second deadline.
  Commands request a server-side 10,000 ms timeout. The adapter disables the
  Redis client's independent 500 ms response timeout: the shared adapter deadline
  applies instead, so a dropped server connection is reported as refused/reset,
  not a premature response timeout. There is no retry layer,
  application spawn, detached worker or retained connection pool.
  All reads in one traversal share a connection and admission permit. Failure
  returns directly without reconnecting or retrying; the UI worker owns recovery.
- Scalar reply columns are parsed as borrowed UTF-8 before constrained core
  types are constructed; malformed fields, excess rows and oversized scalar
  values fail closed. Redis framing allocation is owned by the Redis client;
  the endpoint is trusted, not a hostile protocol parser boundary.

A dropped or timed-out request can have an ambiguous server outcome. Writes
are idempotent, not automatically retried. The store still owns the delivery
lock and generation. `claim_projection` establishes the adapter's ownership
session; each mutation checks its generation token with Redis `WATCH` and
executes inside `MULTI`. That Redis protocol is transport-specific, not Cypher.
The session is serialized across its transaction; no application task is spawned.
Projection acknowledges only complete resource/tag/incident-edge replacements.
An unavailable backend leaves the durable revision pending; server query errors
remain fatal. `clear` is an exclusive-owner maintenance action.

A developer-only paired hub benchmark compares the actual traversal with its
induced-edge completion compiled in or out. It checks 500 visited nodes and a
10,000-incident-edge hub, then prints 20-sample median/worst latency separately
for neighbors, incoming blast radius and shortest paths. It uses a fresh,
process/time-specific graph and deletes the graph and projection fence afterward.
Heap-backed bounded strings/vectors retain the wire representation; inputs are
borrowed and trait dispatch is static RPIT with `Send`, not boxed futures or
async-trait. Projection streams the complete incident set in chunks of at most
2,000 edges, retaining at most one chunk and the current store page rather than
the complete resource degree. The adapter clears stale incident edges once,
then applies chunks in order. Each query remains within 1 MiB and uses a
constant-depth `UNWIND $rows` body, not a clause chain per edge. The adapter
groups consecutive equal relation labels and subdivides each group into at most
1,000-row queries, preserving input order, relation labels and provenance.
The generic stream error preserves storage failures
without converting them into graph failures.

Paged replacement is not atomic. Readers can observe no incident edges after
the clear, then the successfully applied prefix between queries. This is the
same window as the previous clear-then-stream projector. Failure or cancellation
leaves the outbox revision pending; a full replay clears again before rebuilding
the exact complete set. Acknowledgement occurs only after the final chunk.
Later invalid pages are rejected after the accepted prefix is visible. The
engine validates ownership before emitting each edge, and fault-injected real
SQLite scenarios require no acknowledgement after later transport, limit or
unrelated-edge failures, followed by exact convergence on a complete replay.
Keeping old edges visible until every page is accepted would require retaining
and validating the full degree before writing, or versioned staging plus atomic
publication, read filters and cleanup. The former loses bounded memory; the
latter adds write/read complexity and temporary storage proportional to degree.

Public errors carry no Redis types. Backend diagnostics retain at most
1,024 Unicode characters; the FalkorDB adapter performs the conversion.

## Executable scenarios

`tests/falkor_live.rs::graph_backend_conformance` is ignored unless explicitly
requested with a live FalkorDB. It uses a process-specific graph and attempts
cleanup on success or a returned error. Assertion failures unwind; use an
isolated runtime, and stop/remove it after the test command in either case.

| Given | When | Then |
| --- | --- | --- |
| Five scoped nodes, including identical ids in two accounts | Upsert twice and enumerate exclusive pages | Five exact scoped keys, no duplicates |
| Incoming, outgoing, cross-account and parallel-provenance edges | Replace incident sets and paginate one edge per page | All four exact edge keys survive replay |
| A dependency chain plus an isolated node | Neighbors, blast radius and shortest paths | Correct direction, two-hop path, zero-hop and not-found outcomes |
| A traversal with one-node budget | Read and search | Explicit budget truncation, not false absence |
| A populated graph | Empty replacement, unrelated replacement, missing endpoints, repeated deletion, clear | Incident-only changes, rejected unrelated writes, no dangling edges, empty final graph |
| 2,001 edges plus stale incident data and an unrelated scoped self-edge | Apply paged replacement twice, shrink, then submit an unrelated edge | Exact complete/replayed set, stale removal, shrink removal, unrelated edge preservation and exact rejection |

Builder tests retain scoped hashing, escaped parameters, relation labels,
provenance and incident clearing from the existing projection tests. Unit
tests also cover malformed responses, cycles/self-edges, depth and budgets.

The fifteen former view traversal/query/path scenarios live in
`crates/graph-store/src/traversal/view_tests.rs`, exercising the surviving
traversal, parser, shortest-path and query builders with their original inputs.
The view's live graph and scoped tag scenarios continue through the port.
`crates/engine/tests/inventory_store/projection_trait.rs` records backend calls
while using a real SQLite inventory/outbox to check successful, deleted and
offline delivery. Its high-degree scenarios require exact chunk sizes for
2,000, 2,001 and 5,000 incident edges; a second-chunk failure retains the exact
revision and a full retry removes stale/prefix data before acknowledging the
exact final 5,000-edge set. The ignored projection live suite adds the 2,001-edge
case against a fresh graph and an isolated SQLite `VACUUM INTO` snapshot of the
configured pending inventory; it does not modify the source database.
Identity scenarios separately change the stored endpoint and graph name with
an empty outbox: both require a durable full rebuild before further graph
writes. An unchanged identity leaves the empty outbox unchanged. Builder
scenarios assert the complete traversal query strings and the shared adapter
chunk plan (clear once, then ordered append batches) at 2,000/2,001/5,000 edges.
Malformed cleanup sentinels and unexpected index errors fail closed; writes
without a claimed generation fail before server contact.

## Query ownership check

`scripts/check-cypher-module.sh` checks Rust string literals case-insensitively,
including multiline continuations, graph command names and compact keyword
forms. Only `crates/graph-store/src/` and the legacy schema 13 directory
`crates/engine/src/graph/` are directory exceptions. Named protocol/adversarial
test fixtures and the read-only legacy verifier use exact per-file exceptions
in `scripts/cypher-allowlist.txt`, each with a reason; no test directory is exempt.
`scripts/test-check-cypher-module.sh` proves ordinary Rust is accepted and each
forbidden form in its own temporary file is rejected. The script is not wired
into `moon.yml` by this change.

Acceptance fixtures and schema 13 inspection queries are centralized under
`cypher::diagnostics`. Mutating and hostile fixtures require the `test-fixtures`
feature, enabled only in development dependencies; legacy read-only verification
remains available without that feature. Legacy metadata/functions are not part
of the portable runtime subset.

`inventory_graph::falkor::inspect` provides read-only, typed count operations
for benchmark topology, scoped tag provenance, orphan tags, exact resource
metadata and fixture tag assignments/cardinality. These use `GRAPH.RO_QUERY`
and the adapter's existing admission, byte and timeout bounds, without claiming
projection ownership. `TagFixturePhase::{Initial, Updated}` describes only
the deterministic `bench-<ordinal>` fixture. Resource expectations are supplied
as at most 1,000 named `NodeRow` values; account, region, kind, id, name and
content hash are checked together. Empty expectations return zero. Query text
and scalar count decoding have exact-value unit scenarios for each operation.
This adapter-only inspection API does not extend `GraphBackend`.

Benchmark topology and resource oracle callers use these typed functions;
no benchmark production query site is exempt from the ownership script.
The developer example uses feature-gated inspection functions that skip only
induced completion, retaining the same traversal, connection and timeout bounds.

Run from the checkout with a private Cargo target directory. For native tests,
use the pinned graph runtime described in [Running the flow](e2e/RUN_HERE.md).

```sh
cargo test -p inventory-graph --locked -j3
cargo test -p inventory-engine --locked -j3 --test inventory_store projection_trait
cargo test -p cmdb-view --locked -j3
scripts/check-cypher-module.sh
scripts/test-check-cypher-module.sh
```

With `INVENTORY_GRAPH_URL` set to the isolated test harness's FalkorDB URL:

```sh
cargo test -p inventory-graph --locked -j3 --test falkor_live -- --ignored
```

## Induced-completion measurement

On the original graph lane, the release example measured 20 paired samples
per operation on an isolated FalkorDB. Its fresh graph had 500 visited nodes,
10,000 edges incident to the hub and 10,996 total edges. Both variants used the
same connection, depth 1, 500-node budget and 2,000-edge budget. The variant
without completion skips only the induced query.

| Operation | Without: median / worst (µs) | With: median / worst (µs) |
| --- | ---: | ---: |
| Neighbors | 6,179,937 / 9,113,585 | 6,242,053 / 7,292,278 |
| Blast radius | 958,700 / 2,429,955 | 1,086,294 / 4,001,909 |
| Path | 6,248,837 / 8,654,622 | 6,607,344 / 8,984,001 |

No completed median exceeded twice its corresponding baseline, so this
measurement did not trigger an additional induced-query bound. The run deleted
the graph and ownership fence successfully. These are local measurements, not
an isolated-hardware or cross-engine performance claim: other compilation and
test work overlapped part of the run. The earlier literal-frontier baseline
reached the 10-second read deadline during warmup, before induced completion;
the recorded run uses parameterized frontier and excluded-node lists.

## FalkorDB clause-depth regression

The former builder added a clause chain for each edge. On the original lane's
isolated server, the repeated clause chain hit a plan-depth failure even with
two resource nodes and one repeated edge. Reducing the batch size would not
remove the recursive plan shape.

The production builder instead has one `UNWIND`, two `MATCH` clauses, one
conditional `FOREACH` and one `WITH`, for either one or 1,000 append rows.
Replacement adds one fixed incident-edge clear. No chained-statement fallback
is used. A reduced per-query row bound would not fix the recursive plan shape.
The backend retains the 1,000-row statement bound for wire bytes and groups
consecutive identical relation labels because openCypher does not parameterize
relationship types.
