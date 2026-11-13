# Initial local setup and Rust assurance review (historical)

> Status: Historical evidence. It reviews the initial import of 2026-11-12; its
> dependency, unused-dependency and large-future findings were reassessed in
> [DEPENDENCY_UPDATE.md](DEPENDENCY_UPDATE.md), and it has not been restamped
> as a pass. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

Date: 2026-11-12 (America/Chicago).

This review distinguishes operational local setup from blanket Rust release
approval. The imported Rust source and dependency lockfile have not been changed.
Setup adds a supervised local stack and Arch-compatible extraction of the
verified emulator package. The original archive's historical receipts are not
counted as current verification.

## Black-hat review

**Bead**: none; direct local deployment request

**State**: imported 0.6.0 baseline plus local setup

**Reviewer**: black-hat-reviewer, active Codex execution context

**Source checkout**: this repository

**Attempt**: 1

**STATUS: REJECTED** for full strict Rust/release assurance.

### Phase 1: Contract and scope parity

| Requirement | Result | Evidence |
| --- | --- | --- |
| Build the real Rust collector | Passed | `local-evidence/build-final.log` |
| Keep core synchronous | Passed for inspected boundary | `local-evidence/domain-boundary.json`; no `.await` or spawning matches in `crates/core/src` |
| Preserve partial coverage as partial | Observed | Fresh emulator native/parent/role scans complete; sources and root Cloud Control retain known partial outcomes |
| Claim universal AWS coverage or a 95% mutation score | Not established | `START_HERE.md` explicitly lists both as open; no such claim is made here |

### Phase 2: Farley engineering rigor

| Function/file | Observed size | Limit | Result |
| --- | --- | --- | --- |
| `src/main.rs:67`, `dispatch` | ends before `query` at line 142; exceeds 25 lines | 25 function lines | Rejected |
| `src/scan.rs` | 541 lines | 300 file lines | Rejected |
| `src/jobs.rs` | 985 lines | 300 file lines | Rejected |
| `crates/engine/src/store.rs` | 1,783 lines | 300 file lines | Rejected |

These are baseline findings. No splitting or contract-changing refactor was
performed as part of installing and reviewing this release.

### Phase 3: Holzman Rust

The repository's canonical source clippy passes with warnings denied. The
additional panic/async lint invocation fails at
`crates/aws/src/discovery.rs:328`: the awaited future is **31,128 bytes**.
`local-evidence/strict-clippy.log` contains the exact compiler diagnostic.
Later targets in this failing invocation are not represented as passed.

`cargo audit --json` reports eight advisories: legacy `h2` 0.3.27; bundled
`libsqlite3-sys` 0.37.0; two advisories against `rustls` 0.21.12; and four against
`rustls-webpki` 0.101.7. The old TLS/HTTP stack is enabled through the SDK's
legacy `rustls` feature, alongside the modern client. The SQLite dependency is
SQLx 0.9's bundled library. The machine's `sqlite3` CLI version does not replace
that bundled library. See `local-evidence/audit.json` for exact versions,
advisories and patched ranges. No advisory has been ignored or waived.

Primary references: [HTTP/2 advisory](https://rustsec.org/advisories/RUSTSEC-2026-0258.html),
[SQLite advisory](https://rustsec.org/advisories/RUSTSEC-2026-0459.html),
[certificate verification advisory](https://rustsec.org/advisories/RUSTSEC-2026-0382.html).

### Phase 4: DDD and functional structure

`crates/core/src/lib.rs:58` exposes resource/account/region/kind identifiers as
public `String` fields and validates them later. This matches the supplied
contract but does not satisfy the stricter newtype/parse-at-boundary skill.
`src/main.rs:42` also uses `unwrap_or_else`, which the functional-rust style
rules prohibit even though it is a fallback combinator, not `unwrap()`.
Neither observation alone demonstrates a reachable runtime panic.

### Phase 5: Evidence integrity

Fresh fixture evidence is bound to source and binary hashes. One local run was
correctly rejected because a concurrent canonical build replaced the executable:
`provenance:binary_unchanged` and `provenance:scan_commands_bound` failed. That
run is retained locally as failed evidence and is not used for acceptance.
The clean rerun starts after the build finishes. Builds/source edits must not
overlap a verification run.

### Findings and required repairs

| ID | Severity | Location | Disposition | Required repair |
| --- | --- | --- | --- | --- |
| BH-1 | High | `Cargo.lock` | blocker | Replace the legacy SDK transport feature path and move SQLx to a supported patched SQLite dependency; rerun TLS, storage, migration and live integration checks |
| BH-2 | Medium | `crates/aws/src/discovery.rs:328` | blocker | Reduce the discovery future size with measured ownership/layout changes; rerun strict async clippy and discovery/cancellation behavior tests |
| BH-3 | Medium | `src/main.rs:67`, large files listed above | blocker | Decompose under accepted domain boundaries and preserve behavior |
| BH-4 | Medium | `crates/core/src/lib.rs:58` | blocker | Introduce boundary-parsed domain identifiers with an explicit compatibility/migration contract |
| BH-5 | Medium | formal/coverage/mutation lanes | blocker | Add reviewed implementation-bound harnesses and execute the missing assurance lanes before claiming strict release approval |
| BH-6 | Low | `crates/aws/Cargo.toml`, `crates/engine/Cargo.toml` | blocker | Resolve `cargo machete` findings for `tokio-util` and `tokio-stream` using dependency/feature evidence |
| BH-7 | Low | `src/main.rs:42` and similar fallback style | blocker | Reconcile the strict functional skill's fallback-combinator policy without changing error behavior |

**Verdict: REJECTED for strict release assurance.** Local operational evidence
does not waive these findings. Review-only findings are not presented as fixed.

## Rust skill applicability and evidence

| Skills | Application in this task | Evidence / limitation |
| --- | --- | --- |
| black-hat-reviewer, architectural-drift | Contract, size, type and evidence review | Findings above; no approval |
| holzman-rust, functional-rust, async-rust-reviewer | Canonical gates, stricter panic/async lint, domain boundaries, task ownership inspection | Canonical gate passes; stricter async gate fails; `TaskTracker` owns store preparation and classifies join panic/cancellation |
| explore, rust-contract, scott-ddd-refactor | Map crates, inspect supplied contract and domain state | Core models, executor, SQLx store, AWS adapters and CLI mapped; no production redesign performed |
| test-planner, test-reviewer, test-writer, bdd-enforcer | Map setup behaviors to real CLI/server/SQLite/graph scenarios | Existing suites reused; no new Rust tests or implementation were written; no assertion-free smoke run is treated as full coverage |
| hands-on-qa, qa-enforcer, truth-serum | Invoke CLI and lifecycle, inspect actual persisted state and delivery | Current receipts are required; no archived result is promoted to current proof |
| proof-planner, proof-plan-reviewer | Assess missing proof coverage and prerequisites | No accepted proof plan or formal obligations exist for this imported baseline |
| proof-writer, proof-reviewer, proof-to-implementation, formal-verifier | Check source/harness binding before considering proof claims | No implementation-bound Verus/Kani/Flux/Loom harnesses found; no formal PASS or self-approved plan is claimed |
| verus, verus-reviewer | Check verifier availability and proof/trust surface | Installed Verus uses Rust 1.95; project pin is nightly 1.102; no target/bridge exists, so no proof command is represented as passed |
| kani | Check tool and harness inventory | Kani 0.68/CBMC 6.11 installed; no Kani harnesses found; no unbounded solver job launched |
| flux-rs | Check tool and annotation applicability | Installed compiler plugin is older than the project toolchain; no Flux annotations/target exist |
| loom | Check concurrency-model coverage | Virtual-time/property tests exist; no Loom-instrumented production synchronization target exists; no Loom proof claimed |
| miri | Check pinned-toolchain support | Miri component absent on nightly-2026-11-11; no UB run claimed; native SQLite/TLS integration is not Miri evidence |
| rust-fuzzer | Inspect parser/adversarial testing coverage | Proptest suites exist; no coverage-guided Rust fuzz target exists; no fuzz campaign claimed |
| evidence-packaging | Preserve current commands, raw logs, failures and scope | Selected sanitized logs in `local-evidence`; historical ZIP receipts remain separate |

Fjall, Dioxus and Makepad specialists do not apply: this application uses SQLx
SQLite and a CLI. Beads/JJ/Moon delivery orchestration does not apply to this
new Git import: it has no `.beads` or `.moon` configuration or requested bead.
No GPU agent, runtime migration, SIMD rewrite, or performance claim was made.
The installed SDK catalog's optional all-437-services build was not run.

Holzman references read: `nasa-jpl-standards.md`,
`runtime-performance-architecture.md`, `latency-throughput-playbook.md`,
`zero-cost-abstractions.md`. Async references read:
`structural-correctness.md`, `cancellation-safety.md`,
`async-verification-gate.md`. Truth-serum reference read:
`adversarial-audit.md`.

## Truth-serum decision

**Strict assurance: REJECTED.** Commands were executed in the active context;
no agent summaries or historical logs establish current PASS. No Rust test was
deleted, weakened, disabled or changed. Missing formal/coverage/mutation work
remains unverified. The synthetic debug benchmark is only a local exercise,
not a release-performance or live-AWS throughput result.

**Local operational setup:** see `LOCAL_RESULTS.md` for the final current
runtime and verification receipts.
