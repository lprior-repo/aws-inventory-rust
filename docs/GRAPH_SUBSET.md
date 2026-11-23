# Inventory graph contract (E3.1)

`inventory-graph` is an MIT, backend-neutral port depending only on
`inventory-core`, Redis, Tokio and thiserror. Its sole implementation is
`falkor::FalkorDb`. No collector or app uses it yet: E3.2/E3.3 migrate those
callers. Existing engine/view queries and tag projection remain untouched.
The live tree has tag builders beyond the 187-line projection file described
in the plan; tag traversal/projection is not part of this nine-method port.

## Query language

All query text in the new crate lives in `src/cypher.rs`. It uses these
openCypher constructs, not procedures or engine-specific Cypher functions:

- `MATCH`, `OPTIONAL MATCH`, labels, directed and undirected relationships.
- `MERGE`, property maps, `SET`, `DELETE`, `DETACH DELETE`.
- `$parameters`, strings, nulls, lists and integer literals.
- `WHERE`, comparisons, `AND`, `OR`, `NOT`, `IN`, `IS NOT NULL`.
- `WITH DISTINCT`, `RETURN`, `ORDER BY`, `LIMIT`.
- `FOREACH` with `CASE WHEN ... THEN [1] ELSE [] END` for conditional merges.

There are no variable-length path queries. Bounded frontier queries and a
Rust breadth-first calculation implement traversal and shortest paths. Future
variable-length patterns must have explicit lower and upper bounds. The
`CYPHER name=value` parameter preamble is FalkorDB transport syntax, not part
of the portable query body. Neptune would bind the same parameters through
its own transport; no Neptune runtime conformance is claimed here.

## Identity, replacement and pagination

A `NodeKey` is the core `ResourceKey`, including account, region, kind and id.
The stored `key` is its existing full scoped digest, encoded in lowercase hex.
`NodeRow` mirrors the existing projection row (key, optional name, hash).
Edges retain their relation label, relation property and bounded provenance.
`EdgeKey` is `EdgeRow`: all four fields participate in edge identity.

`replace_edges` atomically removes **all incident resource edges**, incoming
and outgoing, and merges the supplied incident edges in a single query. It
rejects unrelated edges before I/O. Missing endpoints are not created;
conditional merges skip them without suppressing later edges. Empty input
removes all incident edges. Callers must supply the complete incident set,
not merely the node's outgoing edges. Tag edges are not resource edges.

Node pages are ordered by digest, with an exclusive cursor. Edge pages use
(source digest, target digest, relation, provenance), also exclusive. Cursors
are not SQL lexical key order and pages are not a retained snapshot: writers
must be quiescent when checking equality across stores. Missing nodes delete
idempotently; node upserts and full edge replacement are replayable.

## Bounds and failure semantics

- Query text: at most 1 MiB; growth uses fallible reservation.
- Input strings: at most 4,096 bytes; relation/provenance at most 128 bytes.
- Replacement/page: at most 2,000 edges/rows; invalid limits fail before I/O.
- Traversal: depth 1–3, at most 500 nodes and 2,000 edges, four layer reads
  including the depth-boundary probe. Node/edge vectors reserve their bounds.
- Neighbors traverse both directions; blast radius follows incoming edges
  (A -> B means A refers to B). Edge direction is preserved in results.
- `Truncation` distinguishes complete, depth-limited and budget-limited reads.
  A path is shortest within the requested depth; no path within that depth is
  `NotFound`. Budget-limited absence is `GraphError::Truncated`, never a false
  claim that no path exists. Identical source and target yield `SameNode`.
- Clear deletes at most 1,000,000 vertices and checks for leftovers; a graph
  above that cap returns a limit error, never successful partial clearing.
- Each backend admits at most 16 commands; admission plus connection/query
  takes at most ten seconds. Each traversal also has a ten-second deadline.
  Commands request a server-side 10,000 ms timeout. There is no retry layer,
  application spawn, detached worker or retained connection pool.
- Scalar reply columns are parsed as borrowed UTF-8 before constrained core
  types are constructed; malformed fields, excess rows and oversized scalar
  values fail closed. Redis framing allocation is owned by the Redis client;
  the endpoint is trusted, not a hostile protocol parser boundary.

A dropped or timed-out request can have an ambiguous server outcome. Writes
are idempotent, not automatically retried. Projection ownership/fencing is
still the existing engine's responsibility and must be preserved at E3.2;
this unused port does not claim to replace it. `clear` is an exclusive-owner
maintenance action, not safe to race with new writes.

No performance claim is made: E3.1 does not migrate a hot-path caller or have
a measured budget. Heap-backed bounded strings/vectors retain the existing
wire representation; inputs are borrowed and trait dispatch is static RPIT
with `Send`, not boxed futures or async-trait. No new engine choice is made.

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

Builder tests retain scoped hashing, escaped parameters, relation labels,
provenance and incident clearing from the existing projection tests. Unit
tests also cover malformed responses, cycles/self-edges, depth and budgets.

Run from the worktree with its private Cargo target directory:

```sh
cargo test -p inventory-graph --locked -j3
INVENTORY_GRAPH_URL=redis://127.0.0.1:6379/ cargo test -p inventory-graph --locked -j3 -- --ignored
```
