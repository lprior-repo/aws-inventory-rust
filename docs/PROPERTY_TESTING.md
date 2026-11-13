# Property tests and retry contracts

The test suite uses Proptest to generate bounded inputs and command histories,
Tokio's paused clock for retry scheduling, and real SQLx/SQLite databases for
durable effects. The mutation gate is described in [MUTATION_TESTING.md](MUTATION_TESTING.md).
A passing property suite and a mutation percentage answer different questions:
properties exercise a stated contract across generated inputs; mutation testing
checks whether the suite detects particular changes to the implementation.

## Independent models

| Suite | Contract and independent oracle |
| --- | --- |
| `crates/core/tests/properties.rs` | Unicode identifiers and byte boundaries; resource, relationship, scope and page validation; event round trips; ordinary `serde_json::to_vec` output length compared with the production bounded serializer. |
| `crates/engine/tests/retry_properties.rs` | Generated API-outcome tapes predict the first terminal result and exact attempt budget. A paused clock checks dispatch inequalities, server-delay lower bounds, cancellation, permit recovery and independence of quota buckets. |
| `crates/engine/tests/job_properties.rs` | A command-history model checks submission, competing claims, heartbeats, expiration, cancellation, explicit retry, append-only attempt history and stale lease fencing across a real database close/reopen. |
| `crates/engine/tests/job_validation_properties.rs` | Public queue operations distinguish JSON objects from other JSON values, enforce UTF-8 byte boundaries, reject invalid scheduling/lease capabilities and preserve queue state after rejection. |
| `crates/engine/tests/store_properties.rs` | A committed-page ledger checks contiguous pages, identical replay, changed-payload conflicts, duplicate records, cursor cycles, provenance, finalization and atomic rollback. Successful reads must agree with the model after each command and after database reopen. |
| `crates/engine/tests/graph_contracts.rs` | A finalized real SQLite scan exercises graph-name validation, byte-driven batch splitting, projection grammar and inclusive limits. A real FalkorDB rejection must leave the failed batch unacknowledged and the projection incomplete. |
| `crates/aws/src/common/property_tests.rs` | Retry hints, integer units, HTTP dates, delay ceilings, error classification, diagnostic bounds, pagination cycles, page caps, token-history byte bounds and preservation of pages preceding a fetch failure. |
| `crates/aws/src/discovery/pages/property_tests.rs` | Discovery pagination, independent counters, identity deduplication, cursor cycles and exact aggregate bounds. |
| `crates/aws/src/discovery/property_tests.rs` | Account/role identity validation, coverage and deadlines, plus SDK configuration propagation across the implemented service clients. |
| `crates/aws/src/supplemental/property_tests.rs` | Config normalization and Resource Explorer identity parsing preserve source semantics and reject malformed identities. |
| `tests/cli_properties.rs` | The actual binary must produce correct plans and query results, enforce a hard result budget and read-only SQL policy, and persist job submission/cancel/retry behavior. Partial ingestion and cancelled persisted snapshots must return exit code 2 while retaining their evidence. These tests never start a cloud scan. |

Inputs, command counts and shrinking are bounded. Suites set fixed RNG seeds;
ordinary developer runs retain Proptest's failure persistence. Regression files
should be reviewed as test inputs, including when a minimized example exposes an
incorrect test assumption rather than a production defect.

Mutation runs set `PROPTEST_DISABLE_FAILURE_PERSISTENCE=1`: a deliberately broken
implementation must not append new seeds to the frozen source tree and alter the
inputs used for later candidates. The failing seed and minimized example remain
in the candidate's test log. Source, tests, configuration, lockfile, tool version
and required external fixture hashes are recorded by the mutation gate.

## Automatic retries

The production `ApiExecutor` owns retries for explicit inventory requests. Its
budget includes the first attempt. It acquires global and quota-bucket capacity
for each actual attempt, releases capacity during backoff, checks cancellation,
applies request deadlines and records attempts, successes, failures, throttles,
timeouts, retries and in-flight counts. Throttling affects the corresponding
bucket; it must not prevent an unrelated service bucket from making progress.

Generated outcome tapes check that success, access denial, permanent failure and
cancellation terminate the tape. Retryable failures consume exactly the allowed
number of attempts. A server retry directive is either honored as a lower bound
or rejected explicitly when it exceeds supported bounds. A long wait must not
hold an API concurrency permit.

Explicit inventory SDK clients receive `max_attempts(1)`. The AWS Rust SDK
[documents this as disabling its internal retry loop](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/retries.html).
That allows the shared executor to count and rate-limit every explicit attempt,
without multiplying an outer retry budget by a hidden SDK retry budget. SDK
credential resolution and refresh are a separate boundary; see
[AWS_PROPERTY_TESTS.md](AWS_PROPERTY_TESTS.md) for the audit and its limits.

Job retries are also separate from request retries. A recovered or explicitly
retried job creates a new scan attempt. Job replay does not claim to resume a
live opaque AWS pagination cursor. API retry policy cannot make a failed or
partial scan silently complete.

## Emulator integration oracle

`scripts/emulator/mutate.py` owns a fresh signed AWS Emulator process, generated
credentials, verified TLS and optional local FalkorDB. It first runs the ordinary
collector and the existing independent resource/relationship checks. It exports
that SQLite evidence with `scripts/emulator/snapshot.py`, then runs the unmodified
Rust snapshot suite twice before admitting a mutation command.

The five AWS integration cases cover native collectors, optional Config/Resource
Explorer sources, the 397 root Cloud Control descriptors, the 21 parent-scoped
descriptors, and an assumed role in the fixture account. The parent fixture has
two parents per family, 161 children and 101 EKS add-ons to force a second
emulator page, including an empty filtered page with a continuation token.
The tests compare resource and relationship
observations as multisets, exact scope/page flags, terminal state and recorded
failures. Nested properties and arrays remain part of the comparison. Only the
collector's top-level observation time is removed from equality comparison; its
actual value must fall within the measured collection interval.

An initial repeated baseline exposed changing Config capture timestamps in the
pinned emulator's background recorder. The mutation fixture waits for all eleven
seeded Config records and stops its own synthetic recorder before the ordinary
baseline scan. Two independent queries must agree afterward. Provider capture
timestamps remain exact oracle values; the test does not discard them. The
ordinary collector never stops or changes an AWS Config recorder.

`tests/emulator_cli.rs` additionally executes the real scan, durable worker,
query, report, architecture, graph-export and graph-push routes. It checks the
persisted observations, request accounting and graph delivery acknowledgments.
FalkorDB's three guarded engine integration tests cover actual graph writes,
replay, string parameters and delivery outcomes. Together with the five AWS
cases and two CLI cases, the final mutation gate requires ten named live tests.
The delivery verifier also compares every exported property against SQLite and
the actual FalkorDB graph, including unresolved nodes and scan markers.

Two new parent property tests check exact model serialization and value-sensitive
scope identity, plus idempotent selection closed under required prerequisites.
A separate HTTP fixture uses the production Cloud Control
SDK path to check serialized models, independent cursors, empty nonterminal
pages, a real 429 response and executor-owned retry accounting. Its outer
deadline is classified as an unresolved test timeout by the mutation gate.

These tests are ignored in an ordinary offline Cargo run because their local
servers must be prepared explicitly. When selected, absent or invalid fixture
configuration is a test failure. The gate requires named integration tests to
pass in the unmodified baseline; an ignored or silently skipped test cannot
satisfy that requirement. Emulator capability gaps remain explicit partial
results, as documented in [EMULATOR_TESTING.md](EMULATOR_TESTING.md).

## Running the suites

```sh
cargo test -p inventory-core --test properties --locked
cargo test -p inventory-engine --test retry_properties --test job_properties --test store_properties --locked
cargo test -p inventory-aws --lib property_tests --locked
cargo test --features aws --test cli_properties --locked
```

Proptest is pinned to 1.11.0 with its standard-library support and without default
optional features. Its RNG dependency graph is separate from the pinned AWS SDK
graph. This avoids the additional SDK feature unification observed with 1.12.0
in this constrained build environment while preserving normal shrinking and
failure replay. The version and reason are recorded in the workspace manifest;
upgrades should verify the resolved feature graph as well as test behavior.

The mutation runner supports isolated shards and complete aggregation. A partial
package result is progress evidence. The project-wide 95% requirement can pass
only after every frozen first-party candidate is accounted for, all required
baselines pass, no timeout or infrastructure failure remains unresolved, and the
aggregate score satisfies the threshold without rounding up.
