# Mutation testing: fixed scope, measured results, and the 95% gate

> Status: Baseline being replaced. The mutation gate described here is Python
> tooling over the baseline packages; ROADMAP M2.10 either replaces it in Rust
> or drops it with a recorded reason, and the roadmap sets no mutation-score
> target of its own. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

The target is **at least 95% of viable completed mutations caught by tests across
all four production packages**. A partial run cannot satisfy that target.
Neither property-test case counts nor ordinary test counts are mutation scores.

## Portable handoff and the complete serial runner

The historical v0.4 engine, AWS and CLI campaigns are stopped diagnostics.
Their source-restoration/interruption receipts are retained. Survivor-driven
test additions require a fresh source closure. The complete serial runner below
is implemented. The older 87-test receipt remains historical evidence in
`artifacts/python-handoff-tests-v04.json`. See `MUTATION_READINESS_V06.md` for the
current acceptance audit, repairs, and capacity limits.

Prepare the signed emulator with the Rust harness, its Python
requirements as described in `EMULATOR_TESTING.md`, and native Redis/FalkorDB
binary paths. Build ordinary binaries first:

```sh
cargo run -p inventory-harness -- install /tmp/aws-inventory-emulator
cargo build --locked -p aws-inventory --features aws --bin aws-inventory -j1
cargo build --locked -p inventory-aws --example emulator_branches -j1

cargo run -p inventory-harness -- mutate \
  --workspace "$PWD" \
  --campaign "$PWD/artifacts/mutation/full-next" \
  --snapshot "$PWD/../inventory-mutation-full-next" \
  --target-dir "$PWD/../inventory-mutation-full-target" \
  --fixture-output "$PWD/artifacts/mutation/full-next-fixture" \
  --emulator /tmp/aws-inventory-emulator/unpacked/opt/aws-emulator/bin/aws-emulator \
  --binary "$PWD/target/debug/aws-inventory" \
  --branch-probe "$PWD/target/debug/examples/emulator_branches" \
  --redis-server /path/to/redis-server \
  --falkor-module /path/to/falkordb.so \
…
  --timeout 240 --build-timeout 600 --wall-timeout 86400
```

The `/path/to/` arguments must identify your native installation. Choose fresh
campaign, snapshot, fixture and control paths; existing evidence is not replaced.
The dedicated Cargo target must be outside the authoring checkout. All four
packages are frozen together. **Ten required live tests** are assigned to their
own packages so an unrelated core mutation does not falsely require an AWS
integration test in that core-only test process. Two unmodified complete
baselines run before candidate execution. SDK caches can be reused, but fresh
source timestamps force first-party baseline validation.

The launcher owns the wrapper and runs the strict aggregate only after the
outer fixture lifetime has closed successfully. To request orderly shutdown,
create the chosen stop file from another terminal:

```sh
touch artifacts/mutation/full-next.STOP
```

A stopped run is incomplete and cannot pass. The receipt and logs record the
stop/drain outcome. Do not reuse interrupted diagnostic counts as final results.
Finish and verify the intended code, test, catalog, and lint changes before
freezing the final acceptance campaign. Ordinary validation has its own
source-bound receipts; it cannot substitute for mutation execution.

The first completed historical v0.4 partition was `inventory-core`: **56 candidates, 56 caught,
zero missed, zero unviable, and zero timeouts**. The source was restored exactly.
The run took 284.27 seconds including its clean baseline; its baseline build
took 33.63 seconds and baseline tests took 9.07 seconds on this shared host.
These are local calibration observations, not AWS throughput measurements.
The strict report is `artifacts/mutation/core-v04/report.json`.
Its historical package score was 100%, but `gate_passed` is deliberately **false** because
the remaining packages are not covered by that report.

The whole-project 95% target remains **unachieved and incomplete**. The historical
v0.4 parallel root, AWS, and engine campaigns were stopped after compiler resource
failures and useful test gaps were identified. Their raw outcomes, interruption
records, and exact clone-restoration hashes are preserved as diagnostics; they
are not completed quality-gate evidence. The root diagnostic recorded 12
outcomes (6 caught, 1 missed, 4 compiler-unviable, 1 SIGKILL unresolved); the AWS
diagnostic recorded 33 (20 caught, 12 missed, 1 SIGKILL unresolved).

The replacement serial runner, `scripts/mutation/run_all.py`, is implemented and
its control paths are tested, but **its full mutation campaign has not run**.
Later source/test changes require a fresh complete manifest and clean baselines.

## Run the complete serial campaign

Use the unmodified collector/branch-probe build and the real local emulator/Falkor
binaries. Substitute their actual paths:

```sh
cargo run -p inventory-harness -- mutate \
  --workspace . \
  --campaign artifacts/mutation/full \
  --snapshot ../aws-inventory-mutation-full \
  --target-dir ../aws-inventory-mutation-target \
  --fixture-output artifacts/mutation/full-fixture \
  --emulator /path/to/aws-emulator \
  --binary /path/to/aws-inventory \
  --branch-probe /path/to/emulator_branches \
  --redis-server /path/to/redis-server \
  --falkor-module /path/to/falkordb.so
```

This freezes all four packages, enumerates every candidate before execution,
uses one mutation worker and one Cargo job, and runs two complete baselines.
Each candidate then runs its mutated package's entire suite, including ignored
integration tests. The manifest maps required fixture tests to their packages:
two root CLI tests, five AWS snapshot tests, and three native engine graph tests.
The AWS contract includes `emulator_snapshot_parents`, exercising the frozen
parent-dependent collection scenario. The combined baseline requires all ten;
each candidate requires the fixture tests belonging to its mutated package.

The aggregate enforces this same fixed policy independently of the launcher.
An otherwise complete unit-only AWS, engine, or CLI partition is insufficient:
final acceptance requires ignored live tests, every mandatory name for the
selected packages, two clean baselines, the bound fixture environment, and a
completed healthy outer fixture receipt. The shared policy is
`gate.REQUIRED_LIVE_TESTS`; the full and supported package runners consume it.

The parent keeps one verified emulator/Falkor context alive throughout. Only
after that outer context closes successfully does it re-enumerate the full
universe and write `full/final-report.json`. Exit zero requires the complete
95% gate. Exit one means the complete score failed; exit two means execution or
evidence was incomplete. No previous caught mutants are selectively skipped.

The startup output and launcher receipt identify a filesystem stop hook. For
the example paths, request a bounded owned shutdown with:

```sh
touch artifacts/mutation/full.STOP
```

The supervisor interrupts the owned wrapper and allows its nested workers to
drain and restore source. A stopped run never starts the final aggregation or
claims a score. Keep the interrupted evidence and choose new paths for a fresh
campaign. Direct tool-session termination can bypass Python cleanup; the earlier
diagnostic interruptions explicitly disclose that limitation.

## Scope and candidate inventory

The historical v0.4 preliminary full enumeration contained 2,535 candidates:

| Production package | Historical preliminary candidates |
| --- | ---: |
| `aws-inventory` | 345 |
| `inventory-core` | 56 |
| `inventory-engine` | 615 |
| `inventory-aws` | 1,519 |
| Total | 2,535 |

The complete preliminary list is
`artifacts/mutation/candidates-preliminary.json`. It is explicitly historical and
preliminary: authors were still adding tests, and later source changes invalidate
its use as the current denominator. Each execution partition freezes a fresh
complete package list before running mutations. The final aggregate independently
re-enumerates all four packages on the delivered source tree and requires every
candidate in that universe to be accounted for.

After the current code and lint work stabilizes, this command records a fresh
source-bound candidate inventory without executing mutations:

```sh
cargo run -p inventory-harness -- snapshot freeze \
  --workspace "$PWD" \
  --campaign "$PWD/artifacts/mutation/enumeration-next" \
  --package aws-inventory \
  --package inventory-core \
  --package inventory-engine \
  --package inventory-aws \
  --evidence "$PWD/crates/harness/src/cli.rs"
```

Choose a new directory name each time. `freeze` checks the pinned executable and
version, hashes the complete selected source closure before and after listing,
and writes `candidates.json`, its command log, and `manifest.json`. It invokes
`cargo-mutants mutants --list --json`, which may read Cargo metadata; it does
not build or run mutants or tests. Run it after the active compiler lane is
idle, with the verified Rust toolchain/cache environment. This inventory has no
fixture or executed outcomes and cannot pass final acceptance. The complete
runner creates its own fresh live-bound manifest when the campaign is launched.

There are no project-specific file, function, regex, or survivor filters in the
gate. It does not use `--iterate`, `--in-diff`, or `--baseline skip`. A shard is a
declared deterministic partition of the complete list; it does not reduce the
final denominator or make unexecuted candidates disappear.

The declared scope is the four packages' production library/binary source.
External dependencies and the generated SDK catalog's declarative reexports
are not mutation targets. Tests and `#[cfg(test)]` modules are skipped by the
tool. Non-runtime examples and benchmarks, including the SDK auditor, SQL view
generator, benchmark program, and emulator branch probe, are outside its
library/binary discovery. The aggregate lists the example targets separately.
Macros and generated code are not expanded into extra mutation targets. The
tool's built-in skipped-call default is `with_capacity`; this is recorded as a
tool limitation, not a project-specific exclusion. Feature-inactive candidates
remain visible and can survive. This score is not line or branch coverage.

## Pinned tool

The official GitHub releases API identified `cargo-mutants` **27.1.0**, released
2026-06-02, as the latest release checked on 2026-11-12. The installer pins that
release rather than following a moving `latest` download.

```sh
cargo install cargo-mutants --version 27.1.0
cargo mutants --version
```

The direct executable requires the `mutants` subcommand. It is installed from
the official Linux x86_64 release asset, without a Cargo compilation. The
installer validates the published archive SHA-256 before extracting the single
regular executable, then verifies its version. The gate also requires the
exact executable SHA-256.

| Item | SHA-256 |
| --- | --- |
| Official release archive | `dfe6dc37d0342c891d2829b5a695aa57c2d0edecef7e7d0399a30cc6e206411e` |
| Installed executable | `f985f265ee3ea3e453aa98b04c52134953911f692f8ce8abf137f3873202a2d0` |

Installation evidence is written to
`/tmp/aws-inventory-test-tools/cargo-mutants-27.1.0-install.json`.
The code is Linux-specific because that exact release binary is pinned.

## What permits a pass

The score is calculated with integer arithmetic at the decision boundary:

```text
viable_completed = caught + missed
score_requirement_met = viable_completed > 0
                        and caught * 100 >= viable_completed * 95
```

Every candidate also needs a resolved outcome. The rules are:

| Outcome | Treatment |
| --- | --- |
| Successful build, actual failing-test result | Caught; enters the numerator and denominator |
| Successful build and successful executed tests | Missed; enters the denominator |
| Build failure with an explicit Rust diagnostic and no infrastructure failure | Unviable; retained in the report, excluded from the score denominator |
| Build or test timeout | Unresolved; never credited as caught and prevents a pass |
| SIGKILL, resource exhaustion, missing test execution, missing logs, contradictory outcomes | Unresolved or invalid evidence; prevents a pass |
| Candidate without accepted outcome | Untested; prevents a pass |

An all-unviable run has no score and cannot pass. A displayed 100% score for
completed mutants is insufficient when other candidates remain untested or
unresolved. A metadata-only `cargo check` result is never mutation-test evidence.
The gate does not treat a Cargo exit code alone as a failed test: a caught result
must also have the expected build/test phases and a failing-test result in the
captured log. An unviable result requires an `error[Edddd]:` diagnostic in the
failed build phase. Linker failures, failed build scripts, compiler internal
errors, and a bare Cargo compilation summary remain unresolved. The coded
diagnostics `E0514` and `E0786` also remain unresolved because they identify an
incompatible dependency compiler or invalid crate metadata. `E0463` is an
infrastructure failure when the missing crate is `std` or `core`; an ordinary
missing application crate can still be source unviability. These infrastructure
signals take precedence even if a source-type diagnostic is also present.
The specific internal CLI, snapshot-collection, durable-property, engine barrier,
graph-integration, inventory-export property/streaming, graph-retry fixture, and
parent SDK wire fixture
deadline messages are also unresolved even when the Rust test harness exits
normally with a failed-test status. Ordinary assertions about SDK timeout
classification or the production graph delivery deadline remain eligible to
catch a mutant. Both the tool's original summary and the stricter gate
classification are retained for each attempt.

Multiple attempts are retained. Repeating a caught result does not increase
the numerator. If completed attempts disagree on the same frozen inputs, the
candidate becomes unresolved instead of selecting the more favorable result.
A later consistent completed run can resolve a prior timeout; the timeout
attempt remains visible in the report and is never counted as a caught attempt.

Different frozen fixtures cannot be combined to select a favorable result for
each candidate. Across accepted campaigns, the aggregate requires one AWS
snapshot SHA-256 and one native FalkorDB fixture-manifest SHA-256 for each fixture
kind present. It reports all observed identities and retains all attempts, but
mixed identities within either kind prevent a pass. Separately verified native
engine and AWS-backed partitions remain supported; pure core partitions do not
require an unrelated live fixture.

Only `aggregate` can produce a whole-first-party pass. `report` intentionally
returns a failing gate status for a package partition, including the completed
100% core partition. That return code is not a claim that the core tests failed.

## Frozen package closures

Freezing partitions separately allows independent test development without
invalidating unrelated results:

- A package's own source, tests, and Cargo manifest are bound by hashes.
- Its first-party dependencies' production source, manifests, and embedded
  `sql`, `schemas`, and `data` inputs are also bound. The root package additionally
  binds the SDK catalog's embedded `sdk-services.json`.
- Workspace `Cargo.toml`, `Cargo.lock`, toolchain, Cargo configuration, and
  relevant shared fixture roots are bound.
- Engine depends on core; AWS depends on engine/core; the root binary depends
  on all adapters and its SDK metadata catalog.
- A core result remains applicable after an unrelated AWS test-file edit. A
  core production-source edit invalidates dependent engine/AWS/root results.
- Exact bytes are used. Editing an inline `#[cfg(test)]` block inside a
  dependency's production source file conservatively invalidates its hash too.

The final aggregate checks every partition against the current delivered
closure. It also re-enumerates the entire four-package candidate universe, so a
stale, shortened, or incomplete partition cannot establish a project pass.
The candidates file, manifest, run receipts, outcome JSON, individual logs, and
diff files are hashed. Hashes provide reproducible evidence and detect accidental
changes; they are not a remote attestation against a malicious report producer.

All mutation baseline and test processes receive:

```text
PROPTEST_DISABLE_FAILURE_PERSISTENCE=1
PROPTEST_RNG_SEED=137043
CARGO_NET_OFFLINE=true
```

Failure persistence is disabled only for mutation runs. Otherwise a failing
mutant could append a minimized seed file and change the inputs for later
mutants. Normal development tests keep persistence enabled. The mutation logs
retain test failures, minimized examples, and the recorded deterministic seed.

## Freeze and run a package

Run this only after that package's closure is ready. Use a new campaign directory
for a new source/test closure; never overwrite an earlier result.

```sh
cargo run -p inventory-harness -- snapshot freeze \
  --workspace /path/to/aws-inventory-rs \
  --campaign /path/to/evidence/core \
  --package inventory-core

cargo run -p inventory-harness -- snapshot \
  --workspace /path/to/aws-inventory-rs \
  --campaign /path/to/evidence/core \
  --destination /path/to/disposable-core-checkout

cargo run -p inventory-harness -- snapshot run \
  --workspace /path/to/disposable-core-checkout \
  --campaign /path/to/evidence/core \
  --target-dir /path/to/disposable-core-target
```

The gate refuses in-place mutation of the authoring workspace. The disposable
checkout must carry the manifest marker created by `snapshot`. It is restored
and hashed after execution. An interrupted or unrestored run cannot pass.
No author should edit that checkout while a mutation command owns it.

The fixed Cargo policy is package-local tests, all features of the explicitly
selected production package, `--locked`, `--offline`, and one Cargo build job.
The generated SDK catalog is not selected for all-features compilation. Default
mutation test and build timeouts are 120 and 300 seconds respectively; timeout
values are recorded in the exact invocation. A command-wide wall limit defaults
to one day. Timeouts may be increased for a clearly justified rerun; they do not
turn the timed-out attempt into a caught result.

## Real emulator and FalkorDB tests

Unit-only AWS mutation testing does **not** run the emulator. Tests of mapping
helpers alone may miss a mutation that removes a complete collector or CLI
execution path. Integration campaigns must explicitly enable their ignored
tests and prove that the required tests actually ran.

For example, inside the harness-owned native FalkorDB context:

```sh
cargo run -p inventory-harness -- snapshot freeze \
  --workspace /path/to/aws-inventory-rs \
  --campaign /path/to/evidence/engine \
  --package inventory-engine \
  --include-ignored \
  --required-test live_falkordb_replay_and_parameter_round_trip \
  --required-test live_falkordb_rejected_batch_is_never_acknowledged_or_finalized \
  --required-test live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint \
  --required-env FALKORDB_TEST_URL \
  --fixture-receipt /path/to/context/fixture-command.json \
  --fixture-kind falkordb_native \
  --fixture-manifest /path/to/context/fixture-manifest.json \
  --evidence /path/to/aws-inventory-rs/crates/harness/src/cli.rs
```

The supervising harness wrapper must already own that running receipt and export
the matching `INVENTORY_MUTATION_FIXTURE_SHA256`.

`--include-ignored` adds `-- --include-ignored` to test execution and requires
**two clean unmutated baselines**: one gate preflight and cargo-mutants' own
baseline. Every required test name must report success in both baselines and in
an otherwise successful mutant test run. A skipped or zero-test baseline is
not sufficient.

`--required-env` binds an environment value by hash; it never writes the value
to the manifest. Use it for safe fixture/context flags, not for recording
credential values. The subprocess inherits the harness's generated local
AWS/Falkor environment. The gate does not print AWS secrets or full environment
dumps. `--input` binds explicit snapshot/config inputs by content hash before
and after the run. Their contents are not copied into the report; ephemeral
input files may be removed after the run because the run receipt retains their
verified before/after hashes.

The supported AWS/root/full runners freeze the independent fixture summary,
snapshot, and gate implementation. AWS acceptance requires the snapshot content
digest to match its environment digest and the frozen input path/content pair.
A change to embedded SQL or either Cloud Control catalog invalidates the
corresponding source closure; historical execution cannot silently certify the
new inputs.

The emulator/Falkor context must enclose both baselines and the entire mutation
subprocess. A server started in a tool execution namespace that ends before the
test subprocess starts is not sufficient. The parent harness owns startup,
seeding, typed fixture contracts, credentials, and cleanup. Mutation testing does
not change the known limits of local emulator fidelity or establish live AWS
permissions, quotas, or isolation.

Root/AWS composition commands additionally set `fixture_receipt` when freezing,
binding the active outer `mutation-command.json` command, workspace, and log.
The outer wrapper checks the verified TLS AWS health endpoint and, when present,
the local Falkor PING response periodically and after the child exits. Final
reporting requires a completed zero-exit receipt, observed health checks, and an
unchanged wrapper log hash. A fixture failure invalidates that execution instead
of crediting subsequent failed assertions as caught mutants. An in-child package
report may explicitly allow a pending outer receipt to avoid a circular wait;
it is marked incomplete and cannot pass the whole-project gate. Only the later
strict report/aggregate can accept the closed fixture lifetime.

The gate forwards both SIGINT and SIGTERM to its separately owned mutation
process group. It allows a bounded interrupt/drain interval before forced
termination, restores its caller's signal handlers, and records interrupted
execution as unresolved. Two subprocess regressions verify that external
termination and internal deadlines stop and reap the owned child and grandchild.
This correction applies to processes started after it was added; already imported
older runners retain their original behavior and evidence limitations.

The initial engine-only coordinator has readiness, real baseline/graph-test
traffic, and retained server logs, but no independent periodic/final backend
health receipt. Its evidence must not be described as having the newer outer
wrapper supervision. Backend-failure signatures require review; future frozen
engine campaigns can add that supervision without rewriting earlier evidence.

A native-only Falkor coordinator can bind its own fixture without AWS snapshot
variables. It writes a manifest describing the native server/module and test
inputs, exports that file's exact digest as `INVENTORY_MUTATION_FIXTURE_SHA256`,
and calls:

```python
from pathlib import Path

gate.freeze(
    workspace, campaign, ["inventory-engine"], tool,
    include_ignored=True,
    required_tests=gate.REQUIRED_LIVE_TESTS["inventory-engine"],
    required_environment=gate.REQUIRED_LIVE_ENVIRONMENT["inventory-engine"],
    inputs=[Path(gate.__file__).resolve()],
    fixture_receipt=receipt_path,
    fixture_kind="falkordb_native",
    fixture_manifest=manifest_path,
)
```

The gate automatically records the native manifest as an explicit input and
binds the digest environment variable. It uses the same completed outer receipt,
periodic/final health, and log-integrity requirements, while the report names
the fixture `falkordb_native`. A native manifest cannot substitute for a missing
AWS snapshot context. The CLI exposes the corresponding `--fixture-kind` and
`--fixture-manifest` options.

## Shards and build caches

`run --shard k/n` uses cargo-mutants' pinned `slice` algorithm and retains the
same complete manifest. Every shard has its own required baselines. Start each
concurrent worker with its own disposable source tree and target directory;
never mutate one tree from two processes. Run-directory allocation is atomic.

Each shard's exact selected IDs are compared with the tool's `mutants.json`.
The final aggregate uses all candidates, not just the shards that finished.
Do not use regex selection to hide survivors. Do not silently discard previous
attempts when reporting a campaign.

Cargo-mutants normally copies source to a fresh tree and excludes `target`.
Its official documentation recommends `--in-place` when reusing build artifacts;
this gate uses that mode only in disposable checkouts. A copied or reflinked
existing target directory can avoid recompiling unchanged third-party SDK
artifacts. Cache reuse must be confirmed by the actual baseline log, not assumed.
New snapshots refresh only first-party Rust source mtimes, keeping content hashes
unchanged, so artifacts from an interrupted mutant cannot appear fresh merely
because restored source files have older timestamps. Unchanged SDK inputs remain
outside that operation. Prefer a cache copied from the final unmutated build.
Keep the SDK/toolchain/features/Rust flags consistent. Leave `--cap-lints=false`:
changing Rust flags can force the large EC2 SDK to rebuild. Do not use
`--check` as a substitute for build/test mutation phases.

The initial root fixture preparation was stopped by SIGKILL while compiling the
first-party AWS library concurrently with an active AWS mutation worker. Its
failed baseline and logs are retained; no mutants were credited from it. The
coordinated retry refreshes only unmutated first-party cache files, then acquires
the other target's observed Cargo lock between commands. A recorded, bounded
lock hold serializes cache warmup until both root baselines pass; it does not
change candidate selection, tests, or timeout classification. Waiting beyond
the bounded hold aborts the root attempt and releases the lock after cleanup.

The dependency graph pins Proptest 1.11 with `std`. Proptest 1.12 changed the
test RNG graph in a way that enabled an additional feature of the SDK's existing
`getrandom` dependency, invalidating the cached EC2 build in this environment.
The 1.11 test RNG uses a separate dependency version, preserving the observed
SDK feature graph. This is a recorded dependency/build constraint, not a reason
to omit property tests or lower the mutation scope.

Large SDK targets need disk-backed storage and explicit capacity planning.
The `/tmp` filesystem may have far less room than the workspace filesystem.
More workers multiply target-directory space and compiler memory. Measure
baseline and per-mutation times before choosing concurrency.

## Reports and the final gate

```sh
cargo run -p inventory-harness -- snapshot report \
  --workspace /path/to/final/aws-inventory-rs \
  --campaign /path/to/evidence/core \
  --output /path/to/evidence/core-report.json

cargo run -p inventory-harness -- snapshot aggregate \
  --workspace /path/to/final/aws-inventory-rs \
  --campaign /path/to/evidence/core \
  --campaign /path/to/evidence/engine \
  --campaign /path/to/evidence/aws \
  --campaign /path/to/evidence/root \
  --universe-output /path/to/evidence/final-universe.json \
  --output /path/to/evidence/final-mutation-report.json
```

The final universe output must be a new file. Reports contain every candidate,
its attempts and log paths, score arithmetic, package counts, fixture identities,
untested and unresolved candidates, exclusions, and evidence errors. The aggregate exits
successfully only when the complete fixed universe is resolved, evidence is
valid/current, and the measured viable score reaches 95%.

The report parser has adversarial Rust tests in the harness crate. These tests
exercise timeout accounting, missing test execution, compiler failures
versus infrastructure failures, contradictory retries, duplicate candidates,
package/test filters, all ten required live tests, package runner policy,
embedded dependency data, cross-campaign fixture consistency, shard partitioning,
JSON duplicates, and evidence path traversal. These tests validate
the gate; their count is not added to the Rust project's mutation score.

## Primary references

- [Official cargo-mutants 27.1.0 release](https://github.com/sourcefrog/cargo-mutants/releases/tag/v27.1.0)
- [Official release API](https://api.github.com/repos/sourcefrog/cargo-mutants/releases/tags/v27.1.0)
- [Candidate listing and JSON output](https://mutants.rs/list.html)
- [Outcome files and their versioned format](https://mutants.rs/mutants-out.html)
- [Baseline requirements](https://mutants.rs/baseline.html)
- [Package mutation and test selection](https://mutants.rs/workspaces.html)
- [Cargo and test arguments](https://mutants.rs/cargo-args.html)
- [In-place execution and cache reuse](https://mutants.rs/in-place.html)
- [Sharding](https://mutants.rs/shards.html)
- [Strict lint behavior](https://mutants.rs/lints.html)
- [Pinned outcome classification implementation](https://github.com/sourcefrog/cargo-mutants/blob/v27.1.0/src/outcome.rs)
- [Pinned Cargo invocation implementation](https://github.com/sourcefrog/cargo-mutants/blob/v27.1.0/src/cargo.rs)
- [Pinned shard implementation](https://github.com/sourcefrog/cargo-mutants/blob/v27.1.0/src/shard.rs)

The installed binary's help and the pinned source were checked in addition to
the live documentation, because online documentation can describe a different
release. Rust property-test dependency and feature evidence is retained in
`artifacts/property-unit-graph-v04.json`.
