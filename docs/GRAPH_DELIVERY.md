# FalkorDB delivery, retries, and checkpoint recovery

Graph delivery uses the existing bounded `ApiExecutor` as its only automatic
retry owner. It reconnects after temporary transport or retryable server errors,
replays the same validated idempotent batch, and writes a SQLx acknowledgement
only after the server confirms the graph command. It does not use Redis
`ConnectionManager` retries or wrap checkpoint commits in a second retry loop.

## Public interfaces

The existing API remains available:

```rust,ignore
push_file(input: PathBuf, checkpoint: PathBuf, url: &str)
    -> Result<u64, GraphError>
```

It returns the number of newly applied batches and uses the defaults below.
Callers that own cancellation and need retry evidence use:

```rust,ignore
push_file_with_options(
    input: PathBuf,
    checkpoint: PathBuf,
    url: &str,
    options: PushOptions,
    cancellation: &CancellationToken,
) -> Result<PushSummary, GraphError>
```

| Option | Default | Accepted range |
| --- | ---: | ---: |
| `max_attempts` | 5 | 1–10 total attempts per batch |
| `attempt_timeout_ms` | 30,000 | 1–300,000 |
| `max_backoff_ms` | 5,000 | 1–300,000 |
| `overall_timeout_ms` | 900,000 | 1–86,400,000 |

The attempt timeout covers reconnecting and sending one command. The overall
deadline also covers plan inspection, checkpoint preparation, every batch,
backoff, and acknowledgement writes. Cleanup has a separate bounded drain.
The server query timeout is the smaller of 20,000 milliseconds and the attempt
timeout. The executor supplies its existing bounded exponential jitter.

`PushSummary` reports `applied_batches`, `skipped_batches`, `attempts`, `retries`,
and `reconnects`. Attempts count delivery attempts, including failed connections;
they are not a count of every Redis handshake command. Reconnects count
connection attempts after the first. A fully checkpointed replay sends no graph
commands and reports zero attempts.

## Complete validation and frozen contents

Before any server write, the reader validates the entire version-1 plan:
identity, contiguous sequence numbers, stage order, end marker, row bounds,
query bounds, and JSON framing. It retains one SHA-256 fingerprint per batch and
a digest of their complete ordered sequence. It does not buffer every query or
load the inventory graph into memory.

The existing hard bounds remain 256 declared rows per batch, a 2 MiB query,
bounded JSON lines, and one million batches. The fingerprint vector is bounded
by that batch limit, approximately 32 MB of hash payload. During delivery each
batch must match its previously validated fingerprint before its query can be
sent. Changing the input between passes cannot introduce an unvalidated write.

The checkpoint now has a `projection_plan` table in addition to
`projection_ack`. Its key is the exact destination URL's opaque SHA-256 digest,
graph name, and scan ID. The stored plan digest binds the entire plan, including
an unacknowledged tail. Changing that tail under an existing checkpoint fails
before any reconnect or new graph write. Existing checkpoints are supported:
their complete acknowledgement sequence/digests are checked against the plan
before the first whole-plan binding is inserted.

The checkpoint does not store the raw connection URL. Equivalent URLs written
differently, including changed credentials, have different destination digests;
they may replay the same idempotent graph writes rather than reuse another
destination's acknowledgement records.

## What may retry

Transport retry classification uses Redis's typed connection-loss and timeout
predicates. Server retries are limited to the typed `LOADING`, `TRYAGAIN`,
`CLUSTERDOWN`, and `MASTERDOWN` cases. Authentication, permissions, invalid
configuration, malformed protocol, arbitrary server errors, invalid queries,
`READONLY`, and redirect responses are terminal. This client does not implement
Sentinel discovery or cluster slot routing.

Automatic ambiguous replay is limited to the exporter's recognized query
grammar:

- The exact resource-key index operation.
- Scan-marker `MERGE` statements with validated bound scalar values.
- Resource and relationship `MERGE` statements with data-only row maps,
  expected fields, correctly escaped JSON string literals, matching scan IDs,
  and matching scan-scoped keys.

The row prefix is parsed as data rather than accepted because a query merely
contains the word `MERGE`. A statement inserted outside quoted data cannot gain
retry eligibility. Existing trusted custom plans remain usable, but unfamiliar
query grammar gets one attempt, including timeout cases. Graph plans contain
write queries and remain trusted operator inputs, not a general query sandbox.
Manually restarting a custom plan can repeat an unacknowledged write; the
exporter's replay guarantee does not make an arbitrary custom query idempotent.

Each failed attempt relinquishes its connection; the next admitted attempt
opens a new one. The canonical Redis multiplexed connection owns its driver,
which is aborted when its last handle drops. A lost response can still mean the
server applied the command. Retrying the same immutable `MERGE` batch preserves
node and relationship identity. Repeated creation of the resource-key index
accepts only the existing exact duplicate-index response as success.

## Acknowledgements and interruption

The successful sequence is server confirmation, SQLx acknowledgement commit,
then the applied-batch counter. A server rejection, exhausted retry budget,
transport failure, or cancelled send never writes an acknowledgement for that
attempt. If a confirmed graph write is followed by a failed or interrupted
checkpoint commit, resuming may replay that immutable batch safely.

Cancellation drops the currently owned request/backoff future; no detached
application retry task continues. The connection handle is dropped and the
SQLx pool is closed with a bounded drain. Cancelling the client cannot promise
that an already accepted server query was undone. Checkpoints and replayable
batch identity handle that ambiguity.

`GraphError::Cancelled`, `GraphError::Deadline`, and
`GraphError::RetryExhausted` distinguish cancellation, overall timeout, and
attempt exhaustion. Redis errors exposed through `Display`, `Debug`, or error
sources retain a typed category but omit endpoint credentials, query contents,
and raw server detail. SQLx acknowledgement failures remain terminal and keep
the already committed progress available for a later invocation.

A graph reset or external graph modification is not inferred from local
acknowledgements. Use a fresh checkpoint when intentionally replaying a plan
into a reset graph. New inventory contents require a new frozen scan/plan;
changing an existing bound plan is rejected.

## Cancellation-aware export

`export_snapshot_with_cancellation(store, scan_id, graph, output, token)` adds
owned cancellation to the existing export API. Cancellation covers header reads
as well as streaming data. The exporter establishes exclusive ownership of its
`.partial` file before selecting cancellation. A pre-existing temporary file
belonging to another invocation is not removed.

The writer remains owned outside the selected streaming future. After an error
or cancellation, the exporter awaits its pending flush with a bounded drain,
drops the file, and removes its own temporary. Successful output still uses
atomic no-clobber installation. `GraphError::Cleanup { source }` reports a failed
drain or removal, including checkpoint-pool shutdown failure, so orchestration
can report cleanup as incomplete instead of treating every returned error as
successful cancellation cleanup.

## Verification

`crates/engine/tests/graph_retry.rs` uses an owned TCP server to inject connection
loss, retryable replies, permanent permission/query/authentication errors, and
pending requests. It checks retry counts, reconnect counts, identical replayed
bytes, exact budget exhaustion, cancellation/deadline behavior, immutable
unacknowledged tails, and the absence of acknowledgements for failed writes.

The ignored native test
`live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint`
uses a proxy in front of real FalkorDB. The proxy forwards a resource write,
waits for the real server's successful response, then deliberately drops that
response. Assertions check the reconnect/retry, node/relationship cardinality,
checkpoint state, zero-command resume, and a full replay with a fresh checkpoint.
Both proxy and server fixture lifetimes remain owned by the test/harness.

The v0.5 focused verification passed 21 tests: two private cleanup/checkpoint
regressions and 19 graph integration tests, including all three native FalkorDB
cases. The fixture answered `PING` before and after the test commands and its
owned server context closed. `artifacts/graph-full-native-v05/result.json`
records the commands, source hashes, log hashes, and health observations.
Production engine Clippy also passed with `--lib --bins --examples
--all-features -- -D warnings`; its full invocation and result are recorded in
`artifacts/graph-production-clippy-v05.json`. These results cover the source
hashes in those receipts and do not substitute for the full application gate.

These are retry and integration checks, not a 95% mutation-score claim. The full
mutation campaign remains unexecuted on the new implementation.
