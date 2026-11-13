# Snapshot comparison and architecture analysis

> Status: Baseline being replaced. It describes the `diff`, `neighbors`, `path`
> and `findings` commands over scan snapshots in SQLite schema 13; the first
> version keeps no snapshots or history (V22), graph traversal moves to FalkorDB
> through `cmdb-view` (ROADMAP M6.2), and SQL files 001 to 013 are deleted in
> M9.5. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

The offline analysis commands operate on **finalized local observations**. They
make no AWS requests. They answer what the inventory observed and which declared
relationships connect those observations. They do not turn missing records into
deletions or declared paths into network/IAM reachability verdicts.

The implementation is `src/analysis.rs` and `src/analysis/`. It uses the existing
SQLx `Store` interface and the snapshot-specific `scan_resources`, `scan_edges`,
`scan_architecture_edges`, `scan_vpc_routes` and `scan_coverage` views. It does not
join to `latest_resources` when answering a snapshot question.

## Commands

These examples use an already built CLI and an existing inventory database:

```bash
aws-inventory diff --db inventory.sqlite \
  --before SCAN_BEFORE --after SCAN_AFTER \
  --limit 100 --max-items 100000

aws-inventory neighbors --db inventory.sqlite \
  --scan-id SCAN_ID --resource-id RESOURCE_ARN \
  --direction both --max-depth 2 --max-nodes 1000 --max-edges 4000

aws-inventory path --db inventory.sqlite \
  --scan-id SCAN_ID --from RESOURCE_ARN --to REFERENCED_ARN \
  --direction outgoing --max-depth 8 --max-nodes 1000 --max-edges 4000

aws-inventory findings --db inventory.sqlite \
  --scan-id SCAN_ID --limit 100
```

Running scans are rejected because their observations can still change. Partial
and cancelled snapshots can be inspected, and their actual scan status remains
visible. The normal CLI returns exit code 2 for a result with partial/cancelled
status. A truncated detail display is separately marked; it does not silently
change the meaning of complete comparison counts.

Opening an older inventory database follows the existing Store schema migration
contract. After initialization, analysis uses query-only readers.

## Snapshot diff

### Identity and selection

Resource identity is its exact stored `id`. Relationship identity is the tuple
`(from_id, to_id, relation)`. The Store selects each snapshot's preferred
observation using its existing source priority and timestamp rules before the
comparison sees it. Two observations of the same logical object under different
identifiers remain different; analysis does not invent identity reconciliation.

Each entity class has its own counts and bounded detail rows:

| Change | Meaning |
|---|---|
| `newly_observed` | Present in the after snapshot and absent from the selected before observations. This does not prove creation. |
| `no_longer_observed` | Present in the before snapshot and absent from the selected after observations. `deletion_inferred` is always false. |
| `changed` | Identity metadata, properties, tags or selected source differ. |
| `unchanged` | The compared fields are equal; a new observation timestamp alone does not count as change. |

Resource field comparison covers account, region, kind, name, properties, tags
and source. Edge comparison covers properties and source after matching exact
edge identity. A separate `source_changed` count explains changes that can come
from a different source's detail contract. Native SDK output, Cloud Control
provider properties, Config records and Resource Explorer metadata are not
treated as interchangeable schemas.

JSON objects compare structurally, so key order is immaterial. Array order
remains significant because the application cannot assume that every provider
array is a set. Only the top-level `observed_at` metadata is ignored. A timestamp
inside provider properties remains provider data and can trigger a change.
Details include before/after identity metadata and names of changed fields;
they do not duplicate potentially large configuration values into the report.

### Streaming and bounds

Comparison merges two sorted streams. It retains one lookahead item from each
input, the current compared pair, counters and bounded detail rows. It does not
collect a full snapshot in memory. Resource and relationship comparisons run
sequentially.

The Store reader pool deliberately has one connection. The public diff command
opens **two Store handles to the same database before beginning the streams**,
giving each input its own bounded reader. Holding two simultaneous streams from
one Store would allow the first to occupy the only reader while the second waits.
Both handles are closed after success or a returned analysis error.

`--max-items` bounds examined **union identities per entity class**, not rows per
input and not the size of the entire database. One additional union identity is
probed to distinguish an exact boundary from a truncated comparison. Stream
lookahead can read a further input record. The backend may still perform work to
evaluate its snapshot-selection views; this is not a database query-plan or RSS
guarantee.

`--limit` bounds displayed resource changes and displayed relationship changes
independently. Each class also has a 6 MiB encoded-detail budget. Counts continue
after the detail display fills, up to the comparison limit. The report separates
`details_truncated` from `comparison_truncated`:

- `comparison_truncated=false`: counts cover all identities in that comparison.
- `details_truncated=true`: displayed details are a deterministic bounded subset.
- `comparison_truncated=true`: counts cover only the examined prefix and the
  overall result is partial.

Both input scan statuses are reported. Even when both are complete, selection
can differ between scans: regions, collectors, accounts, permissions and provider
behavior all matter. Absence remains an observation difference.

## Neighborhood and potential dependency impact

Traversal uses breadth-first search with stable SQL edge ordering. Every start
identity must be either an observed resource or an endpoint of a collected edge
in the selected scan. An unknown arbitrary ID is rejected. A known reference can
be a start node without acquiring fabricated resource properties.

| Direction | Traversed relationships | Useful interpretation |
|---|---|---|
| `outgoing` | Follow `from_id` to `to_id` | What this observed resource refers to. |
| `incoming` | Follow `to_id` to `from_id` | Which observed/reference entities refer to it. |
| `both` | Traverse either endpoint | Local architecture neighborhood. |

Incoming traversal identifies **potential dependents under the collected
relationship semantics**. It does not compute the operational blast radius of
an outage. A role reference, subnet membership, route target and listener forward
action have different meanings, retained in each edge's `relation` field.

The output contains nodes, their shortest discovered relationship distance,
their observed/unresolved status, and encountered relationships with source,
observation time and available ownership-resolution evidence. Nodes and edges
are deterministically sorted in the output. Cycles, self references and reverse
visits do not duplicate them.

`max_depth` defines the requested radius. Nodes at that distance are retained but
not expanded. `depth_boundary_nodes` reports that boundary explicitly. Edges
whose endpoints are both unexpanded boundary nodes are not enumerated; the output
is a traversal graph, not a promise of the entire induced subgraph.

Exhausting the node, edge, encoded-detail or edge-inspection budget stops the
traversal and records its exact truncation reason. An edge is admitted only when
both endpoint nodes can be represented, so work caps do not leave dangling
references in the returned graph.

The graph fetches at most `max_edges + 1` adjacency rows for a node. A node is
expanded at most once; a retained edge is inspected at most once from each
endpoint, plus one overflow probe. The explicit inspection bound is therefore
`2 * max_edges + 1`. The SQLx query layer's existing SQL-length, row, cell,
byte and execution-progress limits also apply and fail explicitly.

## Path queries

A path query returns one deterministic breadth-first path for the requested
direction. Equal-length alternatives use the first path encountered under stable
edge ordering. Incoming or undirected traversal preserves the original edge
orientation and marks reversed steps rather than rewriting the stored relation.

The start and target may be the same known identity, producing a zero-hop path.
Unresolved reference endpoints can participate in paths; their absence of observed
configuration remains visible. A found path is an observed/reference relationship
path, and `effective_reachability_inferred` remains false.

`not_found_within_limits` means exactly that. It does not assert that resources
are isolated, that the target does not exist, or that no longer path exists.
Depth defines the requested search radius, and work truncation is separately
reported. A found path has the shortest hop count for the selected direction
when no work truncation preceded discovery.

## Evidence findings

The first six rules report specific collected facts:

| Rule | Required evidence | Deliberately unproven |
|---|---|---|
| `internet_facing_http_listener` | An internet-facing ELBv2 resource and a same-scan HTTP listener linked to it. | Public reachability; an HTTP listener can redirect to HTTPS. |
| `default_route_to_internet_gateway` | A normalized IPv4/IPv6 default route naming an `igw-` target. | An attached/usable gateway, public addressing or effective network access. |
| `blackhole_route` | A normalized route whose collected state is `blackhole`. | Fresh reachability, cause or resolution. |
| `unresolved_architecture_reference` | A relationship source or target lacks a selected resource observation in that scan. | Nonexistence of the endpoint. |
| `s3_policy_reported_public` | A native S3 observation with successful policy-status evidence and `is_public=true`. | Effective anonymous access; bucket/account access blocks and other controls can change it. |
| `collection_scope_gap` | A persisted selected scope is failed or incomplete. | The state of the resources hidden by the gap. |

S3 policy status uses the global collector's explicit
`policy_status.configured=true` and `policy_status.is_public=true` values. A denied
read, null property or absent policy does not match. The analysis never infers
public access merely because block settings are missing or false. Raw policy and
ACL interpretation are not part of this rule.

Each rule has a row limit, a one-row overflow probe and a `truncated` flag. An
empty rule result is not a certificate of safety or complete collection. The
native normalized route and listener contracts matter; raw supplemental records
can lack those fields even when they contain related provider data.

## Fixed limits and resource semantics

| Boundary | Maximum |
|---|---:|
| Diff/finding detail rows per entity class/rule | 1,000 |
| Examined diff union identities per class | 1,000,000, plus the bounded probe/lookahead |
| Traversal depth | 16 |
| Traversal nodes | 1,000 |
| Traversal relationships | 4,000 |
| Traversal inspected edge rows | `2 * max_edges + 1`, at most 8,001 |
| Graph encoded node/edge detail | 14 MiB |
| Each diff entity class's encoded details | 6 MiB |
| Complete encoded analysis report | 16 MiB |

Encoded output bounds do not equal process RSS. Maps, queues, predecessor keys,
decoded input rows, SQLite and serialization bookkeeping have additional bounded
or backend-owned allocations. The output byte counter avoids allocating an
additional full serialized report solely to check size. No peak-RSS, latency or
throughput result is claimed for these commands.

No child tasks are spawned by analysis. SQLite reads await the Store's native
worker interface; traversal and diff processing include cooperative scheduling
checkpoints. Dropping a read future does not write partial analysis state into
the inventory. Reports are returned only after successful completion; there is
no persisted analysis result to repair after cancellation.

## Verification scope

Focused tests exercise timestamp-only equality, JSON object order, meaningful
array/tag/provider-time changes, source transitions, absence without deletion,
partial inputs, output/count caps, real cap probes, two-reader diff behavior,
cycles, duplicate observations with source precedence, unresolved references,
directional path orientation, shortest-path tie ordering, exact edge-budget
reverse visits, depth-zero behavior, identity quoting, snapshot isolation and
the six rule contracts.

The parent verification report records the actual compile, lint and execution
results. These tests are synthetic local SQLite checks. They do not establish
live AWS access, effective permissions, traffic behavior or production scale.
