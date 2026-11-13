# Portable inventory export

> Status: Kept and extended. An explicit export is the only place the plan
> allows JSON (V03); the scan-snapshot format and provenance modes described
> here follow schema 13, and the roadmap has no package yet that redefines
> export over binary records. See [VISION](VISION.md) and
> [ROADMAP](ROADMAP.md).

The `export` command writes a complete selected scan as a streaming JSON document
or JSON Lines file. It reads the local SQLx/SQLite store; it makes no AWS or
FalkorDB requests. Resource properties and tags remain structured JSON values.
Relationships, collection coverage, failures, source names and observation times
are included.

```bash
aws-inventory export --db inventory.sqlite --scan-id YOUR_SCAN_ID \
  --output inventory.jsonl --format jsonl --provenance normalized

aws-inventory export --db inventory.sqlite --scan-id YOUR_SCAN_ID \
  --output inventory-all.json --format json --provenance all-observations
```

The output directory must already exist. Each output path must be new. Add
`--allow-partial` to export a finalized partial or cancelled scan. Its actual
status, coverage and failures remain in the artifact. A running scan is always
rejected. A complete scan means its selected attempted scopes completed; export
does not enlarge the scan's resource, service, region or relationship coverage.

Add `--universal-references` to derive and persist universal `REFERENCES` edges
from observed resource properties before writing the snapshot. The flag is off
by default, so existing exports keep their exact edge set. The resolver indexes
the scan's own resource ids, Cloud Control primary identifiers and ARN
components, matches exact ARNs and ARN-shaped primary identifiers, and resolves
unique identifiers scoped to the owner's account and region. Only provider
primary identifiers of at least 8 bytes enter that unique-identifier tier;
shorter values (provider enums such as `abc` or `yes`) are never indexed, while
ARN component resolution keeps its own 3-byte floor. An emitted edge's
`properties.reference` is `arn` for an exact observed ARN and `identifier` for a
unique-in-scope identifier match (a full identity, an ARN component, or a
primary identifier at or above the floor); there is no separate confidence
property, so `identifier` always means "unique within account and region".
Ambiguous matches are counted, never guessed. Derived edges live in a synthetic
`universal_reference` scope that replay rewrites idempotently.

Status legality, output-path checks and the pre-export synthetic edge snapshot
run before resolution, so a rejected export never changes the scan's edge set.
The artifact streams the persisted derived edges and its header counts describe
the scan after persistence, so derivation necessarily precedes the file write.
To keep failed exports side-effect free, the exporter restores the captured
synthetic edge set (or its absence) whenever writing, sync, cancellation or
final publication fails; only a successful publication keeps the new set.

## Provenance modes

| Mode | Resource and edge records | Provenance |
|---|---|---|
| `normalized` | One preferred observation per resource ID, and one per `(from_id, to_id, relation)` edge identity. These are the existing `scan_resources` and `scan_edges` views. | Every resource and edge retains its selected `source` and `observed_at`. Alternative observations are omitted. |
| `all-observations` | Every persisted observation for this scan, including repeated identities across sources, scopes and pages. | Each resource and edge additionally carries an `observation` object containing its original `scope` and `page_number`. |

The preferred-source order is direct SDK, Cloud Control, AWS Config, then Resource
Explorer. Within a source, the store's timestamp and deterministic scope/page
tie-breakers apply. An older direct observation can therefore outrank a newer
discovery observation; its original timestamp is preserved.

Both modes include all stored scope-coverage records and collection failures.
The `header.scan.resources` and `header.scan.edges` fields always describe the
normalized scan. The final `summary` and command receipt count the records
actually exported. In all-observations mode those resource/edge counts can be
larger. The `records` count includes resources, edges, scopes and failures, and
excludes the header and final summary.

An edge can refer to an uncollected resource. Export preserves that reference
without inventing a resource record. FalkorDB's separate projection can represent
such references as unresolved nodes.

## Versioned file format

The header identifies the artifact as `aws-inventory.snapshot`, version `1`.
Its JSON spelling for the provenance mode is `normalized` or `all_observations`.
Consumers should check the artifact identity and version before processing it.

JSON files contain three top-level fields:

- `header`: artifact identity, version, provenance mode, scan status/times and
  normalized scan counts.
- `records`: a streamed array of tagged resource, edge, scope and failure records.
- `summary`: emitted counts, including the number of data records.

For example, a complete empty inventory scope produces this valid JSON document:

```json
{
  "header": {
    "artifact": "aws-inventory.snapshot",
    "version": 1,
    "provenance": "normalized",
    "scan": {
      "scan_id": "example",
      "resources": 0,
      "edges": 0,
      "completed_scopes": 1,
      "failed_scopes": 0,
      "status": "complete",
      "started_at": 1,
      "finished_at": 2
    }
  },
  "records": [
    {
      "record": "scope",
      "data": {
        "scope": {
          "account_id": "123456789012",
          "region": "us-east-1",
          "collector": "ec2",
          "parent_id": null,
          "source": "direct"
        },
        "last_page": 1,
        "complete": true,
        "failed": false
      }
    }
  ],
  "summary": {
    "resources": 0,
    "edges": 0,
    "scopes": 1,
    "failures": 0,
    "records": 1
  }
}
```

JSONL uses the same payloads. Its first line is
`{"record":"header","data":{...}}`, its middle lines are the same tagged
objects as JSON's `records` array, and its last line is
`{"record":"summary","data":{...}}`. Each line is a complete compact JSON
object followed by a newline. Embedded newlines inside values are JSON escaped.

| Record tag | `data` payload |
|---|---|
| `header` | `ExportHeader`; JSONL only, corresponding to JSON's top-level `header`. |
| `resource` | `{ "resource": Resource, "observation": ObservationProvenance? }`. The resource retains ID, account, region, kind, optional name, properties, tags, source and observation time. |
| `edge` | `{ "edge": Edge, "observation": ObservationProvenance? }`. The edge retains both endpoints, relation, source, observation time and properties. |
| `scope` | Collection scope, last committed page, complete flag and failed flag. |
| `failure` | The original `CollectionFailure`: collection scope, category and message. |
| `summary` | `ExportCounts`; JSONL only, corresponding to JSON's top-level `summary`. |

The optional `observation` field is omitted in normalized mode. In
all-observations mode it contains `{ "scope": CollectionScope, "page_number": N }`.

This format is distinct from `CollectionEvent` JSONL accepted by `ingest`, and
from `GraphBatch` JSONL emitted by `graph-export`. There is no reciprocal snapshot
import command in this increment. The public Rust types derive Serde
serialization/deserialization for consumers and round-trip validation.

This is an inventory interchange format, not a complete SQLite backup. It omits
opaque continuation tokens, collection-page fingerprints, durable jobs and
projection acknowledgements. Keep the SQLite database when those are required.

## Deterministic order and receipts

Records are written in this order:

1. Resources, ordered by resource ID. All-observations mode additionally orders
   by collection scope key and page number.
2. Edges, ordered by source ID, target ID and relation. All-observations mode
   additionally orders by collection scope key and page number.
3. Scope coverage, ordered by collection scope key.
4. Failures, ordered by collection scope key, category and message.

No export-time timestamp or output path is inserted into the file. Exporting the
same immutable scan with the same format and provenance mode produces identical
bytes with the current format implementation. The command receipt contains
`bytes` and a SHA-256 of every output byte, including punctuation and newlines,
as well as scan identity/status and emitted counts.

For JSONL, a complete file has `summary.records + 2` lines. A consumer should
require the final summary and reconcile its counts instead of treating EOF in a
partial file as completion.

## Memory and output bounds

The exporter consumes SQLx streams. It does not call the bounded ad-hoc SQL query
interface, collect the full inventory into a vector, or build a complete JSON
document in memory. A test exports 10,001 resources, beyond the ad-hoc query's
10,000-row cap.

The writer retains one current decoded record, a reusable encoding buffer capped
at 2,359,296 bytes, and a 65,536-byte file buffer. SQLx's row buffering, decoded
JSON allocations and SQLite's own caches are additional; these numbers are not
a total process RSS claim. SQLite can use temporary files while ordering query
results.

| Bound | Default and maximum |
|---|---|
| Data records across resources, edges, scopes and failures | 10,000,000 |
| Total serialized output bytes | 1,099,511,627,776 bytes (1 TiB) |
| One encoded envelope/document field | 2,359,296 bytes (2.25 MiB) |
| File buffer | 65,536 bytes |

`ExportOptions.max_records` and `max_bytes` can lower the first two budgets.
Both must be positive and cannot exceed their maxima. Exceeding any bound is an
explicit failure with no final file; the exporter never silently truncates data.
Counts for normalized resources/edges and successful/failed scopes are checked
against the finalized scan before publication.

## Atomic publication and cancellation

The exporter creates `OUTPUT.partial` with exclusive creation, writes it, drains
buffered file operations and syncs its contents. It then installs `OUTPUT` using
a same-directory hard link, which atomically fails if a destination already
exists, and removes its temporary link. On Unix newly created files use mode
`0600`. Filesystems that do not support this installation operation return an
error; there is no overwrite fallback.

An existing output, including a live inventory database or a symlink, is never
replaced. An existing partial file belongs to another attempt and is left intact.
Concurrent exports targeting the same output can publish only one final file.

The library accepts a `CancellationToken`. Cancellation is observed before reads,
while waiting for database rows, between records and immediately before final
publication. File writes already in progress are drained before the owned partial
file is removed. To obtain that cleanup, callers must cancel the token and await
the export future; abruptly dropping the future or killing the process can leave
a clearly named partial file. Ongoing filesystem operations are awaited rather
than abandoned.

Publication is the commit point. Once the final hard-link operation starts, it
is not raced against cancellation. A concurrent cancellation can therefore lose
to a successful publication, which returns a successful receipt. If removing the
temporary link fails after publication, the distinct `PublishedCleanup` error
states that the final file already exists. Cleanup failures before publication
retain the original error and cleanup detail. File contents are synced, but this
API does not claim directory-entry durability across power loss.

When `--universal-references` is enabled, derived reference edges are committed
to the store before the artifact is written (the artifact and its header counts
include them). If any later step fails — a record or byte budget, a file write
or sync error, cancellation, or the final hard link — the exporter restores the
synthetic `universal_reference` scopes captured before the export, so
`store.edges()` matches its pre-call state even though the edges were briefly
rewritten. Restoring an already-successful earlier replay therefore keeps that
replay's edges rather than clearing them. If the restore itself fails, the
`ExportError::Restore` error reports both the publication failure and the store
failure. Abruptly dropping the future after persistence, or killing the process,
can leave the rewritten scopes exactly as it can leave a partial file; only an
awaited failure is guaranteed to restore them.

The snapshot is immutable through the `Store` API. External modification of the
underlying database while exporting is outside that contract.

## Automatic scan outputs and offline delivery

The scan command can request export after collection with `--output-dir`. The
normal collection selection still determines what is scanned:

```bash
aws-inventory scan --db inventory.sqlite --region us-east-1 --service all \
  --output-dir outputs --export-format jsonl --provenance all-observations
```

To also project the snapshot into an already running FalkorDB server, set the
URL in the named environment variable and add `--falkor`:

```bash
export FALKORDB_URL='redis://127.0.0.1:6379'
aws-inventory scan --db inventory.sqlite --region us-east-1 --service all \
  --output-dir outputs --falkor
```

`deliver` performs those output steps for an existing finalized scan without
making AWS collection calls:

```bash
aws-inventory deliver --db inventory.sqlite --scan-id YOUR_SCAN_ID \
  --output-dir outputs --export-format json --provenance all-observations --falkor
```

Each attempt uses a new directory under `OUTPUT_DIR/SCAN_ID/`. The graph delivery
checkpoint is shared at `OUTPUT_DIR/projection.sqlite`, so acknowledged graph
batches can be skipped on a later attempt with the same destination, graph,
scan and plan. Collection and delivery status are reported separately. Partial
or cancelled snapshots require `--allow-partial`; an active cancellation stops
admission, and a later explicit `deliver --allow-partial` can export the finalized
snapshot.

The provenance flag controls the inventory interchange file. FalkorDB projection
continues to use the normalized preferred-resource and preferred-edge snapshot,
even when the accompanying inventory export contains all observations.

## Library API and verification

```rust,ignore
use inventory_engine::export::{
    export_snapshot, ExportFormat, ExportOptions, ProvenanceMode,
};

let options = ExportOptions {
    format: ExportFormat::Jsonl,
    provenance: ProvenanceMode::AllObservations,
    allow_partial: false,
    ..ExportOptions::default()
};
let receipt = export_snapshot(&store, &scan_id, &output, options, &cancellation).await?;
```

The implementation reuses the store's bounded query-only SQLx pool for both
file-backed and in-memory databases. Export has no new dependency or separate
background worker.

Focused tests cover both file formats and provenance modes, exact domain-value
round trips, deterministic bytes and hashes, preferred-source selection,
source/page preservation, coverage/failure evidence, partial/cancelled/running
scan policy, budget boundaries, no-clobber races, cancellation and cleanup. A
fixed-seed property test exercises Unicode, escaping, nested values, empty
snapshots and all source variants with 16 bounded cases.

```bash
cargo test -p inventory-engine --lib export:: --locked --offline -j1
cargo test -p inventory-engine --test export --test export_properties --locked --offline -j1
```

These commands are focused verification targets. Full AWS coverage, graph
relationship completeness and mutation-score certification are separate
obligations.
