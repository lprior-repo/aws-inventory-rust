# Rust source and asynchronous ownership review

> Status: Historical evidence. It is the review of the imported archive of
> 2026-11-12; tool availability, SDK compilation, dependency audit and
> source-size statements describe that archive, later work is in
> [DEPENDENCY_UPDATE.md](DEPENDENCY_UPDATE.md), and its failures are retained,
> not overwritten. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

**Date:** 2026-11-12. **Scope:** targeted implementation review with observed
test and compiler evidence. **Full requested skill approval: REJECTED pending
the open verification gates below.** This is not a safety certification,
dependency audit, or proof of maximum throughput.

The focused review found and helped resolve two concrete defects: graph export
could replace the inventory database, and shutdown could start queued profiles
after cancellation. The request executor has executable tests for admission,
pacing, retry, timeout, cancellation, policy bounds, and resource release.
The final combined build/test results belong in `VERIFICATION.md`; this document
records the narrower review and distinguishes missing evidence from a pass.

## Current 0.4.0 implementation and emulator review

The current review also covers Organizations and existing-source discovery,
global S3/IAM/Route53/CloudFront inventory, the SQLx job queue and worker, snapshot
analysis, and the emulator integration harness. The earlier numbered findings
below are retained as historical context, not presented as the complete current
source inventory.

Concrete corrections in this increment include:

| Finding | Correction and observed evidence |
| --- | --- |
| An HTTP 501 could consume transient-error retry attempts. | Classified as permanent before retry hints are considered, including unmodeled errors. A Rust regression covers the decision; the emulator observes one `ListAccounts` attempt and one `ListChildren` attempt per parent. |
| An HTTP 200 can still fail SDK deserialization. | Retained a safe protocol-compatibility diagnostic without copying response bodies. Independent typed Rust probes report the failure instead of counting HTTP success as successful decoding. Six official SDK replay-client tests isolate five incorrect numeric timestamp fields. |
| A branch probe could appear successful based only on its HTTP status. | Require the probe command's typed success and semantic assertion receipts. Nineteen harness tests cover classifiers, evidence phases, receipts, and recording bounds. Python fixture traffic cannot satisfy a Rust operation requirement. |
| A self-signed CA used directly as a TLS server leaf was rejected by Rustls. | Generate a CA and a separate server certificate with the loopback IP SAN and server-auth usage. Verify TLS on both recording-proxy hops; do not disable certificate validation. The first failed run remains distinct from later passing runs. |
| A standalone probe could fall back to ambient credentials. | Require emulator-generated credentials and supply an explicit SDK credentials provider. Reject non-loopback endpoints and unexpected account/fixture identifiers before resource probes. The harness clears inherited AWS configuration and disables IMDS. |
| Route53 null-MX records and source ambiguity need explicit treatment. | Preserve null-MX semantics and composite pagination; retain source-selection failures instead of converting failed discovery into an empty complete scan. Regression tests and populated emulator fixtures cover these cases. |
| Durable job ownership can change after a lease expires or cancellation wins. | Transactional claims, immutable fencing data, lease-boundary checks, heartbeat bounds, and cancel/drain behavior are covered by engine and worker tests. This establishes whole-attempt recovery, not within-scan cursor resumption. |
| A caller-built shared SDK configuration could lose its one-attempt setting in DynamoDB service conversion. | The public AssumeRole helper now inserts programmatic shared origin for its retry setting. A property checks 15 service configurations before/after a region clone across seven origin classes, original retry limits and regions. Normal CLI loaders already supplied this origin; the retained failure is specific to the public helper boundary. |

Property and mutation work is recorded in `PROPERTY_TESTING.md`,
`AWS_PROPERTY_TESTS.md` and `MUTATION_TESTING.md`. Core has a complete 56/56
mutation result. Engine, AWS and CLI results remain incomplete or diagnostic;
there is no whole-project 95% pass. The gate distinguishes a real failing test
from a dead fixture, compiler resource exhaustion or an internal test deadline.

The complete emulator operation matrix covers 72 explicit SDK operations plus
the SDK-owned `AssumeRole` boundary. Direct branch probes are identified separately
from production collection paths. Known upstream failures remain failures even
when the regression harness correctly recognizes them. The final counts,
executable hashes, and exact commands are in `VERIFICATION.md`,
`EMULATOR_TESTING.md`, and the versioned JSON evidence.

The ordinary workspace tests and executable build pass. The canonical
metadata-only EC2 check was terminated by the environment before first-party
linting, and is not reported as passing. A separately recorded first-party
Clippy build-mode run uses the official driver and the same selected libraries,
binaries, examples, features, and warning-denial policy. Its artifacts are not
substituted for the ordinary executable used for runtime tests. The separate
driver controls verify actual lint execution, including a lock held across
`await`, rather than inferring it from a zero exit status.

No emulator result establishes real AWS quota rates, IAM-policy enforcement,
cross-region/account isolation, production latency, or exhaustive branch
coverage. Missing strict-skill tools and reference files remain open gates as
documented below.

## 1. Reviewed source and method

The detailed ownership review covered:

- `crates/engine/src/limiter.rs` and its integration tests.
- `src/scan.rs`, including collection, forwarding, writer failure, and shutdown.
- `crates/engine/src/graph.rs`, including file publication and delivery ACKs.
- The AWS scan setup and cancellation seam in `crates/aws/src/lib.rs`.
- The storage preparation worker and consuming shutdown in
  `crates/engine/src/store.rs`.
- Workspace lint configuration, the synchronous core boundary, and SDK catalog
  provenance/generation.

The initial foundation's textual source scan covered **20 first-party Rust source files** under `src`
and the core, engine, AWS, and SDK catalog source directories. The patterns
included unsafe declarations/blocks, direct `unwrap()`/`expect()`, panic/todo/
unimplemented/unreachable macros, production assertion macros, unchecked access,
and transmutation. The scan found **no production matches**. Its **51 matches**
were assertion macros in inspected `#[cfg(test)]` modules. Test assertions were
kept; they are executable checks rather than production failure handling.

The second implementation increment adds network and ELBv2 normalization, shared
server cooldowns, fractional policies, schema-v2 migration and architecture
reporting. Focused cross-review found and resolved three additional correctness
issues: the report omitted valid route target fields; network references could
use the scanning account instead of the reported shared-VPC owner; and adaptive
fractional recovery could stall because of integer rounding. Regression tests
cover the route evidence, supported owner inheritance and high-rate recovery.
Duplicate network resource IDs are also rejected before events are emitted.
The consolidated test/compiler evidence for this increment is in VERIFICATION.md.

The migration review checked atomic rollback and preservation of observations
and continuation checkpoints. The topology review checked exact-scan joins,
partial/cancelled status retention and explicit limit-plus-one truncation. These
are targeted code reviews, not a proof of reachability, cloud completeness or
all possible task interleavings.

This was a textual scan with manual classification, not a Rust parser, formal
panic-freedom proof, or inspection of dependency source. Compiler-enforced
workspace policy forbids first-party unsafe code and denies unused must-use
results. The configured Clippy policy also denies unchecked indexing, unchecked
arithmetic side effects, lossy `as` conversions, unwrap/expect, panic/todo/
unimplemented, and locks held across await.

## 2. Resolved implementation findings

### R1: Graph export could overwrite a live inventory database

**Original failure:** `finish_export` installed a completed `.partial` file with
`rename`. On the target Unix filesystem, that operation replaces an existing
destination. A caller choosing the inventory database as `--output` could
replace the SQLite file with JSONL while the store was open. Error handling for
an unsuccessful export did not prevent replacement on the success path.

**Change:** final publication now atomically creates a hard link at the output
path, then removes the temporary name. An existing destination causes failure;
the code does not rely on an existence check followed by a racy replacement.
The temporary file is adjacent to the output, so the intended publication stays
within the same filesystem. Filesystems that cannot perform the link return an
explicit error.

**Evidence:** the source change is present in `graph::finish_export`.
`graph_export_cannot_replace_the_inventory_database` exercises the dangerous
path, expects export failure, closes/reopens SQLite, and checks that the
resource remains. Execution of this newly added regression is recorded by the
final graph test lane in `VERIFICATION.md`.

**Residual boundary:** a completed export can remain published if removal of
the temporary name subsequently fails. The operation reports that cleanup
error, and the final data remains intact. Publication does not provide a
cross-filesystem move or an explicit overwrite mode.

### R2: Cancellation continued starting queued profile setup

**Original failure:** after `executor.cancel()`, the lazy profile stream still
started pending jobs. AWS configuration loading happened before the first
executor-managed request, so each newly admitted profile could spend up to its
30-second setup timeout before observing cancellation. At 128 profiles and four
active profiles, slow configuration providers could produce many shutdown waves.

**Change:** `collect_job` checks cancellation before starting unstarted profile
work and retains an outcome for that initialized scan. `inventory_aws::scan`
checks cancellation before SDK setup and selects configuration loading against
the same cancellation signal. The executor exposes observation-only
`is_cancelled()` and traced `cancelled().await`; cancellation remains permanent.

**Evidence:** the executor's
`cancellation_is_observable_before_and_after_the_signal` test passed in the
28-test limiter run. It checks a waiter registered before cancellation and
immediate observation after the signal. The AWS regression
`pre_cancelled_scan_returns_before_sdk_setup` checks that a cancelled scan
returns on its first poll without starting an API attempt; its final execution
result belongs to the AWS lane in `VERIFICATION.md`. The queued-profile
short-circuit and loader `select!` were also inspected in the shared source.

**Residual boundary:** cancellation is cooperative. It cannot preempt a
blocking future poll, erase a request already transmitted to AWS, or undo a
committed storage transaction. Inventory operations remain repeatable reads.

## 3. Ownership and await-boundary observations

| Area | Observed contract | Boundary of the evidence |
|---|---|---|
| Executor tasks | The executor creates no tasks. Callers own operation futures. | Does not audit tasks created internally by Tokio, SDKs, SQLx, or Redis. |
| Executor locks | Registry and timestamp mutex guards exist only in synchronous helpers; none crosses await. | Text/source inspection plus configured lock lint, not model checking. |
| Admission | Local capacity and a per-bucket dispatch reservation precede pacing; global capacity is acquired afterward and released before any pacing recheck/backoff. | The gate meters operation admission, not the exact network delivery time. |
| Rate history | Timestamps are committed after global capacity is available, preventing expired pre-reservations from forming a burst. | Tested under virtual-time global contention; no production traffic benchmark. |
| Retry | Every retry re-enters admission; the static attempt ceiling is ten and the configured ceiling can be smaller. | SDK inventory retries must remain one; credential-provider traffic is outside this budget. |
| Cancellation | Owned permits and attempt accounting release on future drop; token cancellation wakes admission, pacing, backoff, and active calls. | Hard process termination is a crash-recovery case, not graceful draining. |
| Scan routing | `buffer_unordered(4)` bounds active profiles; collection, forwarding, and the single writer are joined and observed. Channels have finite capacities. | This does not establish a fixed RSS ceiling for SDK objects before normalization. |
| Writer failure | The receiver closes and the executor is cancelled; blocked senders unwind and retained committed pages remain durable. | A storage failure can prevent later finalization writes; incomplete scans must remain visible. |
| Preparation worker | A `TaskTracker` owns blocking preparation. One semaphore permit survives preparation through persistence; consuming `Store::close` closes intake and waits for preparation. | Full third-party worker shutdown and scheduling are outside this local ownership inspection. |
| Page effects | SQLx transactions persist page observations and checkpoints together; transaction failure does not publish a successful page checkpoint. | Storage fault/cancellation evidence is maintained in the store verification lane. |
| Graph effects | Transport succeeds before a local delivery ACK is committed. MERGE identities allow replay after an ambiguous response. | At-least-once projection, not a distributed transaction; external graph resets need a new checkpoint. |

The bounded source scan found one explicit first-party blocking spawn, the
tracked storage preparation worker. It found no first-party `tokio::spawn`,
`spawn_local`, unbounded channel, or manual vector of join handles. The core
crate depends only on Serde, serde_json, and thiserror and contains no `.await`
or runtime spawning. This preserves the synchronous model boundary.

## 4. Historical focused execution and current verification

Commands in this table were actually run during the initial 0.1.0 focused review.
They are historical observations, not the current total test count. The 0.2.0
combined verification completed with 152 ordinary passing tests, including 42
limiter tests, plus one separately passed native FalkorDB test. Current evidence
and exact commands are recorded in `VERIFICATION.md` and `verification-v02.log`.

| Command or check | Observed result |
|---|---|
| `cargo test -p inventory-engine --test limiter --locked` | Exit 0; **28 passed, 0 failed**, including the added cancellation observer test. Paused-time test execution was 0.05 seconds; build/lock wait time is not an API performance measurement. |
| `cargo clippy -p inventory-engine --lib --locked -- -D warnings` | Exit 0, including a rerun after the cancellation accessors were added. |
| `rustfmt --check --edition 2024 crates/engine/src/limiter.rs crates/engine/tests/limiter.rs crates/sdk-catalog/src/lib.rs` | Exit 0. |
| `python scripts/generate_sdk_catalog.py --check --verify-cache` | Exit 0; **437** exact SDK archives verified against registry SHA-256 values; **241,799,148** compressed bytes. Rechecked after registry relocation. |
| `cargo metadata --locked --all-features --no-deps --format-version 1` | Exit 0; Cargo accepted 437 optional SDK dependencies, 439 features, empty default features, and inherited Rust version 1.101. |
| Catalog/lock comparison | All 437 names, exact versions, and package checksums matched `Cargo.lock`; every catalog SDK dependency was optional. |
| Forbidden-construct and ownership scans | No production forbidden-pattern matches; test assertion matches were classified; the tracked blocking worker and synchronous locks were inspected. |

The limiter tests cover initial attempts and retries, fractional rate rounding,
global-contention burst prevention, global/local caps, unrelated-bucket
progress, throttle feedback during global waits, adaptive ceilings, retry
classification, exhaustion, timeout, cancellation wait categories,
dropped-future cleanup, policy matching, invalid policies, and bounded jitter.
They do not send AWS requests.

The focused Clippy command is narrower than the full async skill's additional
`unused_async`, `await_holding_refcell_ref`, `large_futures`, and pedantic lint
set. The root verification script handles current combined source targets and
separate test compilation/execution. No all-services SDK compilation is claimed.

## 5. Tool availability: unavailable is not passed

Availability was checked using both executable lookup on `PATH` and
`cargo --list`. No package was installed as part of this review.

| Requested tool | Availability | Gate status |
|---|---|---|
| `cargo audit` | Unavailable | Advisory database audit not run. |
| `cargo deny` | Unavailable | License/source/ban/advisory policy checks not run. |
| `cargo vet` | Unavailable | Dependency review attestations not checked. |
| `cargo geiger` | Unavailable | Dependency unsafe-code inventory not produced. |
| `cargo machete` | Unavailable | Unused dependency analysis not run. |
| `cargo hack` | Unavailable | Feature-combination/powerset checks not run. |
| `cargo mutants` | Unavailable | Mutation testing not run. |
| `cargo llvm-cov` | Unavailable | Instrumented code/branch coverage not measured. |

`rustup component list --installed` showed rustc, Cargo, rustfmt, Clippy,
rust-src, the Rust standard library, and LLVM tools for the pinned toolchain.
Installed LLVM tools do not mean `cargo llvm-cov` is installed. `cargo --list`
advertised the rustup Miri shim, but the installed-component list contained no
Miri component and no Miri verification was performed. Nextest, Kani, and
cargo-careful were not available as Cargo subcommands either.

## 6. Requested skill references and incomplete gates

The applied `holzman-rust` main skill and these references were read during the
implementation/review:

- `references/nasa-jpl-standards.md`
- `references/latency-throughput-playbook.md`
- `references/runtime-performance-architecture.md`
- `references/zero-cost-abstractions.md`
- `references/mechanical-empathy-toolchain.md`

The `async-rust-reviewer` main skill was readable. Its installed bundle contained
`SKILL.md`, `agents/openai.yaml`, and `assets/icon.svg`, but none of its referenced
review documents. An explicit attempt to read
`references/async-verification-gate.md` failed. The missing bundle also includes
the referenced spawn-discipline, structural-correctness, stream-patterns,
cancellation-safety, send-sync-ownership, hexagonal-boundaries,
observability-reference, and benchmark-patterns files. This prevents completion
of the reference-driven formal skill gate; source inspection and focused tests
must not be represented as a substitute.

The following remain open for full skill approval:

| Gate | Current evidence gap or unmet condition |
|---|---|
| Complete asynchronous review gate | Referenced guidance bundle missing; the entire prescribed phase/lint/verification sequence was not executed. |
| Dependency assurance | The audit/deny/vet/geiger tools and corresponding reports are unavailable. A lockfile and matching hashes establish package identity, not absence of vulnerabilities or unsafe dependency code. |
| Coverage and mutation quality | No coverage percentage, branch coverage, or mutation score has been measured. |
| Exhaustive scheduling/protocol verification | Virtual-time scenarios exist; no Loom/Shuttle/Stateright state-space exploration or formal scheduler proof was run. |
| Hostile-input/property/fuzz campaigns | No proptest, cargo-fuzz, or equivalent sustained campaign is claimed. |
| Unsafe/FFI/runtime evidence | First-party unsafe is forbidden. Dependencies and native SQLite/crypto code remain outside that source guarantee; no Miri/sanitizer/UB campaign was run. |
| Feature and platform matrix | The catalog feature graph parses and selected collectors have separate build gates; every SDK feature combination and target platform has not been compiled. |
| Performance acceptance | No reproducible before/after scheduler latency distribution, allocation profile, CPU/cache profile, or measured comparison establishes a fastest implementation. No async Criterion benchmark target was present at this review snapshot. |
| Strict allocation doctrine | Collection allocates normalized pages, strings, bucket state, SDK objects, and JSON buffers after initialization. The strict mission-critical no-post-init-allocation condition is not met. |
| Whole-program time and memory proofs | Hard input/admission/page limits are explicit, but they are not a proof of whole-process RSS or a bound on every synchronous poll/third-party allocation. |
| Production observability | Tracing spans and counters exist; tokio-console and an OTLP backend are not configured. A complete span-propagation and drain-certificate audit remains open. |
| Release assurance | No auditable release binary, generated SBOM, signed provenance, or baseline API-compatibility report is claimed here. |

No unverified gate is recorded as `owner_approved_debt` or
`owner_approved_no_action`: this report does not invent an acceptance waiver.
The targeted fixes and tests are useful evidence; the open gates remain explicit
conditions for any later full skill approval or production acceptance claim.
