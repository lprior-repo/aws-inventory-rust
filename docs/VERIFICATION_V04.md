# Verification and delivery boundaries

This report records version 0.4.0 work in the delivery workspace on November 12,
2026. Earlier 0.3.0, 0.2.0 and 0.1.0 evidence is preserved and labelled below.
A command that only checks types is not described as an executable build;
download verification is not collector coverage; a synthetic result is not live
AWS performance.

## Environment

- Linux x86_64, kernel 6.18.57, shared virtualized host.
- Pinned toolchain `nightly-2026-11-11`: Rust 1.102.0-nightly
  (`9c5648833`, LLVM 23.1.5).
- SQLx 0.9.0, bundled SQLite 3.51.3; WAL and `synchronous=FULL`.
- The benchmark uses four Tokio runtime workers and database files on the
  workspace's overlay filesystem. Reported CPU identity and compiler details
  are retained in `artifacts/benchmark-environment.json`.
- All commands use the locked dependency resolution. The workspace declares
  Rust 1.101 as its minimum; a separate stable-MSRV build was not performed.

## Current property and mutation work

**Handoff status:** the user requested the current source ZIP before the full
mutation campaign completed. No full mutation campaign remains active. The
initial engine/AWS/CLI runs are preserved as interrupted diagnostics; the new
serial runner is included for continuation. `START_HERE.md` is the entry point.

The most recent complete ordinary workspace run is
`artifacts/property-workspace-final-v04.json` and its matching log: **424 passed,
zero failed, seven ignored**, across 21 suites. These seven tests require
prepared local services: four collector snapshot tests, two CLI integration
tests and one native FalkorDB test. Tests added subsequently in response to
engine survivors have their own evidence; the 424 count describes that exact
recorded run rather than every later edit.

The retry-configuration property exposed a production defect at the public
`assume_role_config` boundary: an explicitly assigned retry value with unknown
origin could be overridden by the pinned DynamoDB service defaults. The helper
now records programmatic shared origin. All 60 AWS property tests pass after the
fix. Normal CLI loaders already recorded this origin, so this is not evidence
that ordinary CLI scans previously made hidden DynamoDB retries. The minimized
failing input, pre-fix log and passing run are retained; see
[AWS_PROPERTY_TESTS.md](AWS_PROPERTY_TESTS.md).

The updated ordinary executable SHA-256 is
`70c5ff6fcd32a188691a93d3eb2971d3fc28f7ca549d5d9f27a97ba8aac49380`.
Its probe hash and build receipts are in
`artifacts/plain-binaries-properties-v04.json`. Fresh mutation-fixture baselines
using these ordinary binaries passed **1,071 checks**, including two additional
checks that the synthetic Config recorder is stopped and its snapshot stable.
All 73 operation boundaries were observed with the same 68-success/five-failure
compatibility split described below. The provider timestamps remain exact;
they were not removed to make the oracle pass.

`artifacts/aws-operation-source-reconciliation-v04.json` records all current AWS
source hashes, verifies that the 73 explicit callsite positions and 72 operation
identities still match the earlier audit, and links fresh per-operation evidence.
The original catalog is retained unchanged with its historical hashes.

The root mutation fixture then passed two complete unmutated CLI package
baselines: **59 tests per baseline**, including both real CLI/emulator/FalkorDB
tests. The AWS package's two baselines include **224 unit tests and all four
required collector snapshot tests**. Repeated baselines are stability checks,
not additional distinct tests.

| Mutation package | Frozen candidates | Current evidence boundary |
| --- | ---: | --- |
| `inventory-core` | 56 | Complete: 56 caught, no survivors, unviable candidates or unresolved outcomes |
| `inventory-engine` | 615 | Diagnostic run; survivor-driven test changes require a new final source closure |
| `inventory-aws` | 1,519 | Diagnostic run stopped; 13 additional option tests and bounded-driver repairs now pass in the 237-test AWS suite |
| `aws-inventory` | 345 | Diagnostic run stopped; both required live baselines passed, and a new partial/cancelled CLI exit regression passes |
| Whole application | 2,535 | **95% requirement not achieved or certified** |

Core's 100% score is a package result. The aggregate requires every current
candidate, consistent source/test/fixture hashes, real failing-test evidence
for caught mutants, and no unresolved timeout or infrastructure failure. The
strict gate does not credit SIGKILL, a dead fixture or a test's own deadline as
a detected mutation. See [MUTATION_TESTING.md](MUTATION_TESTING.md) for the scope,
formula, reproducible commands and evidence requirements.

The broader Clippy build-mode run that includes test targets did **not** pass:
`artifacts/clippy-properties-all-targets-v04.json` records exit 101 and its log
contains 255 test-code indexing/arithmetic diagnostics (200 root, 55 AWS).
These are outstanding lint issues, distinct from the earlier compiler-memory
failure. The ordinary test executions remain separate passing evidence; no
blanket lint suppression was added. The latest focused AWS run passed 237 tests,
the engine follow-up run passed 131 including both real FalkorDB tests, and the
actual CLI partial/cancelled regression passed. A combined ordinary workspace
rerun after all follow-ups was not performed before this handoff.

## Earlier 0.4.0 implementation and endpoint evidence

The earlier ordinary workspace run in `artifacts/tests-v04-final.log` passed
**329 tests with zero failures**, with one server-dependent test ignored. That
same ignored FalkorDB test passes in the separate real-server lane. The final
source also has a separate emulator-probe guard test and six tests in the
isolated `tests/sdk-wire/` workspace. These counts describe distinct test scopes;
Python assertions and API operations are not added to the Rust test count.

The ordinary AWS-enabled executable and the `emulator_branches` example built
successfully. Clippy corrections in that run only collapse one nested condition and
box the large `JobCommand::Submit` argument variant; affected AWS and root CLI
suites are rerun after those edits. `artifacts/clippy-v04-source-fixes.patch`
records the exact changes. Runtime tests use ordinarily built executables, not
Clippy-produced artifacts. Executable and source hashes for the later property
work are recorded separately above; the earlier endpoint matrix retains its
original binary/source hashes.

| Verification lane | Observed result |
| --- | --- |
| Ordinary workspace tests | 329 passed, zero failed; one real-server test ignored in this lane |
| AWS-enabled binary and explicit branch probe | Built successfully with the locked dependencies |
| First-party Clippy, libraries/binaries/examples and all application features | Passed with warnings denied through the official driver in Cargo build mode |
| SDK catalog auditor Clippy | Passed through the ordinary `cargo clippy` command |
| Canonical metadata-only all-targets check | Environment terminated the EC2 dependency compiler with SIGKILL on three attempts; not a pass |
| SDK protocol replay workspace | Six tests and strict Clippy passed; no network or emulator involved |
| Recorder/receipt regressions | 19 Python tests passed, including 72 operation mappings and 41 pagination mappings |
| CLI, SQLx/SQLite and real FalkorDB fixture suite | 269 CLI checks passed, plus the opt-in native Rust graph test |
| Signed AWS Emulator integration | Every implemented operation boundary attempted; upstream failures retained explicitly |

### Actual emulator results

The final signed integration run is documented in `EMULATOR_TESTING.md` and
`artifacts/emulator-v04/summary.json`. It observes **73 unique operation
boundaries**: 72 explicit SDK operations and the SDK-owned `AssumeRole` boundary.
The outcome split is 64 successes through collector/discovery/provider paths,
four independently verified direct SDK probes, two unsupported Organizations
operations, and three Resource Explorer response-decoding failures. Therefore
**68 boundaries have successful response evidence and five remain failures**.
The direct probes do not establish that their conditional production branches
were reached.

The harness passes **1,069 checks**, comprising 53 global resource checks, 984
regional checks and 32 orchestration/coverage checks. These include recognizing
known upstream failures correctly; passing the regression harness does not make
those operations compatible. It observes actual noninitial pagination tokens on
29 operations. Each request is attributable to a fixture, a production Rust
command, or an explicit Rust probe, and fixture traffic cannot satisfy coverage.

The native scan completes with **150 resources, 340 relationships, all 32 native
kinds, 131 completed scopes and zero failed scopes**. The source scan retains 11
Config resources and a partial outcome for unavailable Resource Explorer data.
Same-account AssumeRole and the resulting IAM scan complete with 13 resources
and 22 relationships. Native architecture/report commands succeed, and graph
export produces six batches representing the native snapshot. The emulator
harness itself does not start a graph server.

Cloud Control attempts every one of the **200 catalog types**. Of these, **197**
have successful nonempty list/get evidence; three types are unregistered in the
pinned emulator. The resulting snapshot has 200 resource instances because some
types have multiple fixtures. Most provider fixtures use the emulator's generic
property store; this does not validate real CloudFormation provider behavior.

The separate CLI/FalkorDB fixture lane checks all 200 SQL views and 2,289 provider
column values, schema4 migration, global inventory, snapshot analysis, durable
queue commands and replayed graph delivery. It uses actual SQLx/SQLite and
FalkorDB 6.0.4 in Redis 8.10.3, but synthetic input observations. Its graph-server
results are not misattributed to the emulator resource snapshot.

### SDK decoding and compiler boundaries

The independent SDK protocol workspace accepts three modeled ISO timestamp
responses and rejects five independently varied numeric timestamp fields. The
unchanged pinned Rust SDK and official replay transport are used. These six
in-memory tests isolate the Resource Explorer protocol discrepancy; they do not
turn emulator failures into successes or replace tests against live AWS.

The three canonical check failures are retained in
`artifacts/verification-v04-first-attempt.log`,
`artifacts/verification-v04-second-attempt.log`, and
`artifacts/verification-v04-no-hint-attempt.log`. The last attempt tested a
materially different EC2 hint configuration, which was not retained. All failed
inside the large EC2 dependency before the full first-party lint lane.

`artifacts/clippy-build-mode-v04.json` records the successful alternative:
`RUSTC_WORKSPACE_WRAPPER` points at the pinned official `clippy-driver`, and
`CLIPPY_ARGS=-Dwarnings__CLIPPY_HACKERY__` enables actual warning-denying lint
callbacks during `cargo build --workspace --exclude inventory-sdk-catalog --lib
--bins --examples --all-features --locked --offline -j1`. Ten independent driver
controls cover lint detection, warning-denial exit status, MIR-based await/lock
linting, test-target selection and ordinary dependency reuse. No first-party
lint was suppressed and the normal verification script remains unchanged.
This is a passing first-party Clippy lane, not a claim that the canonical
metadata-only command passed.

No live AWS scan, IAM authorization audit, multi-account/region isolation test,
production quota calibration, throughput record or exhaustive branch coverage
is claimed by these local results.

## Historical 0.3.0 build and test gates

`bash scripts/verify.sh` completed with exit status 0 on the final source tree.
The full output is retained in `artifacts/verification-v03.log`.

| Gate | Observed result |
|---|---|
| `cargo fmt --all --check` | Passed |
| Offline SDK catalog reproduction | Passed; 437 services |
| Rust generator reproduces all Cloud Control SQLite views | Passed; 200 views |
| All application/adapter targets and features type-check | Passed |
| Catalog with no service SDK features enabled | Passed |
| Clippy for first-party libraries, binaries and examples, with `-D warnings` | Passed |
| Focused Clippy for the Rust all-SDK auditor, with `-D warnings` | Passed |
| Combined workspace test run, excluding all-services catalog compilation | 201 passed, 0 failed; 1 server-dependent test ignored in this ordinary run |
| `cargo build --features aws --locked -j1` | Passed; actual AWS-enabled executable produced |

The 201 executed Rust tests comprise 7 CLI/catalog/plan/report tests, 100 AWS
adapter tests, 5 model-bound tests, 3 engine unit tests, 5 graph integration
tests, 42 limiter tests, 12 schema-upgrade tests and 27 SQLx store integration
tests. The adapter suite includes 20 Cloud Control tests and 19 new ECS
task-definition/load-balancer tests, alongside the existing networking,
ELBv2, supplemental mapping, retry-hint and selection checks.
The separately executed native FalkorDB integration test also passed on the
0.3.0 source, giving **202 distinct passing Rust tests** across the ordinary
and opt-in runs. The ignored test in the ordinary run is that same opt-in test.

Notable regressions cover cancelled writes after SQLite's first uncommitted
insert, rollback across SQL batch boundaries, reader reuse after interruption,
conflicting page replay, source precedence, honest partial scopes, quota pacing
under contention, cancellation before credential loading, cursor cycles, child
limits before enrichment, shared-resource ownership and graph export refusing
to replace the live inventory database. New regressions exercise server delay
units and shared cooldowns, fractional-rate recovery, exact ELBv2 membership
identity, route edge deduplication, supplemental Classic/v2 disambiguation,
transactional schema upgrades, snapshot-specific endpoint joins and route-panel
evidence retention.

New 0.3.0 regressions cover schema-derived field redaction, composite provider
identifiers, child failures before list-only fallbacks, malformed siblings,
pre-cancelled and mid-page cancellation, ACTIVE task-definition discovery,
target-group evidence, and transactional schema-3 upgrades. Migration tests
preserve page fingerprints and custom views/triggers/indexes, check actual
writer foreign-key enforcement, roll back broken upgrades, and reject unsupported
extensions to managed table columns before mutation.

### Historical 0.3.0 CLI, breadth and graph checks

The final AWS-enabled `aws-inventory 0.3.0` executable passed **234 CLI checks**.
The checks exercised the actual binary, SQLx/SQLite and a real local
FalkorDB 6.0.4 module loaded in Redis 8.10.3. All inventory inputs were synthetic;
no AWS requests were made.

| Evidence | Observed result |
|---|---|
| Catalog | 200 Cloud Control types, 65 families, 25 native kinds, 205 distinct combined kinds, 437 SDK crates, schema 3 |
| Offline plan | 200 types × 2 profiles × 2 explicit regions = 800 initial list requests if every scope is reached; total API requests explicitly unknown |
| Plan validation | A single selected RDS type works; unknown types and an oversized selection are rejected |
| Breadth fixture | 200 resources, 200 distinct provider types, 0 edges, 1 complete scope, 0 failed scopes |
| Source-specific SQL views | All 200 views queried with `SELECT *`; all 2,289 selected provider column values compared with their synthetic provider JSON |
| ECS fixture | Definition family/revision, `awsvpc` and selected runtime fields; service-to-definition and service-to-target-group edges both resolve |
| Architecture fixture | 19 resources, 40 edges, 14 complete scopes, 1 failed scope; partial status retained |
| Architecture panels | 7 routes, including 1 blackhole; 1 unresolved reference; bounded/truncated reports and all six documented SQL examples checked |
| Architecture graph delivery | 5 initial batches, 0 acknowledged batches on resume, 5 batches on fresh-checkpoint replay; stable 20 Resource nodes and 40 edges |
| Breadth graph delivery | 4 initial batches, 0 on resume, 4 on fresh-checkpoint replay; stable 200 Resource nodes, 200 distinct kinds and 0 edges |

The architecture graph has 19 observed nodes plus one unresolved reference
placeholder. Its scan marker remains `partial` after delivery and replay.
The breadth graph's fixture marker remains `complete`; that describes the
fixture scopes, not real AWS coverage. The native graph integration test also
checks quotes, backslashes, newlines, Unicode and injection-shaped property data.

Evidence is retained in `artifacts/cli-smoke-v03.json`,
`catalog-v03.json`, `collection-plan.json`, `breadth-report.json`,
`breadth-graph.jsonl`, `architecture-report-v03.json`,
`architecture-graph-v03.jsonl`, `falkordb-smoke-v03.json`,
`falkordb-live-v03.log` and `cli-native-v03.log`.
`verification-v03-summary.json` pins the tested executable and source hashes.
The server was loopback-only and stopped in the harness cleanup path; temporary
databases, server binaries and the executable are not included in the source ZIP.

The executable SHA-256 is
`ddfc0d6f75964cd948d92b0d6cf640cb961ffff9859510a3a30f6c38b67382d9`.
The breadth fixture, complete catalog, runnable offline examples and normal
verification script are included. See `COLLECTION_FIXTURES.md` for their
different test universes and reproduction commands.

The executed CLI driver is retained as `artifacts/cli-native-v03-harness.py`.
It records the delivery workspace and local Redis/module paths used in this
run; those path constants must be adapted before running it elsewhere. It is
verification tooling, not part of the Rust application's scan path.

### Compiler setting required by the observed build

Two conventional attempts to generate the large EC2 SDK's executable code were
terminated by the build environment, even though type checking passed. The
pinned nightly's selective `profile-hint-mostly-unused` support respected the
hint in AWS's SDK manifest and allowed executable compilation to complete.
That setting is retained in the repository configuration, so the documented
ordinary Cargo commands reproduce the enabled setting. No experimental
512-codegen-unit override or global compiler flag was retained.

The original 0.1.0 adapter build completed in 6 minutes 1 second; its combined
test build took 8 minutes 33 seconds because feature unification required another
dependency build. Those historical build times explain the retained compiler
setting. They are not scan speed or reproducible build-time guarantees. The
current 0.3.0 build/test timings are recorded in `verification-v03.log`.

### Initial 0.1.0 CLI smoke checks

The AWS-enabled executable then passed six offline command checks:

1. Synthetic demo: seven resources and four edges.
2. Report: complete status and seven completed scopes.
3. SQL query: the expected EC2 instance/type/VPC fields.
4. Graph export: five batches containing seven resources and four edges.
5. JSONL ingest of an empty terminal page: complete empty scope.
6. JSONL ingest of a synthetic denial: partial status and exit code 2.

Results are retained in `artifacts/cli-smoke.json`. These checks made no AWS
requests. Temporary SQLite databases were closed and removed afterward.

### Historical 0.2.0 CLI and topology checks

The final AWS-enabled `aws-inventory 0.2.0` executable passed **21 CLI checks**.
The catalog reported 24 native kinds, SQLite schema 2 and the 437-crate SDK
catalog. The original synthetic demo, report, SQL and graph export still worked.
The new architecture fixture then exercised partial ingestion/reporting, all
eight architecture panels, real truncation, limit rejection, all six documented
SQL examples, explicit partial-export consent, and three FalkorDB delivery cases.

| New fixture evidence | Observed result |
|---|---:|
| Resource observations | 19 |
| Relationships | 40 |
| Completed / failed scopes | 14 / 1 |
| Expanded routes | 7 |
| Blackhole routes | 1 |
| Unresolved edge targets / sources | 1 / 0 |
| Listener | HTTP, port 80 |
| Final scan status / report exit code | partial / 2 |
| Rows from the six documented SQL queries | 15, 7, 1, 1, 3, 1 |

Two NAT routes retained two evidence entries on one graph relationship. The
endpoint route exposed its `vpce-` target, and target health joined the observed
EC2 instance on port 8080. With limit 100, all panels were complete. With limit 1,
the routes and resource-count panels were explicitly truncated; the one-row
unresolved-reference panel was not. Limit 1001 failed instead of silently capping.

Outputs are retained in `cli-smoke-v02.json`, `catalog-v02.json`,
`architecture-report.json`, `architecture-graph.jsonl` and `cli-native-v02.log`
under `artifacts/`. The report and graph are clearly synthetic sample outputs.
No AWS calls were made. Temporary inventory and delivery databases were removed.

## SDK download and provenance verification

The generated catalog and retained official manifest contain 437 service SDK
crates from `release-2026-11-11`, resolved to immutable source commit
`0f29bbb75b57d337bd00cf46c0d29d6988e2c372`.

The following command completed successfully after all downloads and after the
registry was relocated from a memory-backed directory to disk:

```bash
python scripts/generate_sdk_catalog.py --check --verify-cache
```

Its result was 437 verified archives, 241,799,148 compressed bytes, and no newer
registry versions than the pinned snapshot at check time. All 437 package names,
versions and checksums matched `Cargo.lock`. The optional-feature graph was
accepted by `cargo metadata --locked --all-features --no-deps`.

The source delivery contains the catalog, original manifest, refresh/check
utility and lockfile. The multi-gigabyte extracted registry cache is not bundled;
`cargo fetch --locked` reproduces the downloads. No all-437-services compilation
or all-437-services collector implementation is claimed.

### Historical 0.3.0 Rust source audit

The Rust audit example reverified all 437 exact archive checksums and inspected
647,023,212 bytes of generated source. Its report records 19,617 operation
modules, 3,779 paginator modules and 407 waiter modules. All 437 configuration
sources expose retry configuration and the other documented public integration
seams; none exposes the audited public retry-strategy/runtime-plugin setter.
Every counted operation registers the four checked retry classifiers.

Two runs of the final auditor produced byte-identical reports. The retained
`artifacts/sdk-capabilities-v03.json` is 2,304,697 bytes with SHA-256
`fb2cb8b410b151ba8a23a2e824fdf03b9d29d494809b181b4914d1d2d6c58ce9`.
Its embedded auditor-source hash matches the delivered Rust source. The auditor
also passed the final focused Clippy gate with warnings denied.

These are lexical source and archive-integrity findings. Unpacked source hashes
are recorded independently, not byte-compared with every archive entry. The
audit does not compile every SDK, exercise every operation, establish quota
values, or turn every SDK into an inventory collector. `SDK_RETRY_AUDIT.md`
documents the method and integration limits in detail.

## Historical 0.1.0 storage and scheduler measurements

These measurements predate the 0.2.0 and 0.3.0 additions;
the benchmark was not rerun for these increments. No new performance gain is claimed.
The release-mode harness runs 10,000 synthetic resource observations and one
relationship per resource. Each storage case includes fixture construction,
validation/serialization/hashing, page writes and final counts; it excludes
connection setup and shutdown. The database uses WAL and `synchronous=FULL` on
the workspace filesystem. These are small fixture records, not representative
large AWS configuration documents.

All measured runs are retained, including the unfavorable repeat:

| Run | Resources per second, 1 resource/page | Resources per second, 100 resources/page |
|---|---:|---:|
| Initial row-by-row baseline | 648.8 | 1,744.7 |
| First batched candidate | 585.9 | 13,850.8 |
| Repeated row-by-row baseline | 467.7 | 6,371.2 |
| Final implementation, paired repeat | 439.7 | 4,999.1 |

The first comparison favored batching substantially; the paired repeat did not.
This shared host and workspace had varying CPU and filesystem load, including
the separate AWS-enabled compiler job during the repeat. **These measurements
do not establish a stable throughput improvement.** Batching reduces the number
of SQLx statement submissions by construction, but an end-to-end speedup needs
a controlled benchmark. The final implementation retains static statements for
single-row chunks and uses bounded multi-row statements for larger chunks.

The final 100-resource-page case took 2.0004 seconds for 10,000 resources plus
10,000 edges; the singleton-page case took 22.7434 seconds. Page size and commit
frequency matter. Neither number is an AWS calls-per-second rate or a capacity
guarantee. Raw outputs are `benchmark-baseline.json`, `benchmark-batched.json`,
`benchmark-baseline-repeat.json` and `benchmark-final.json` under `artifacts/`.

The harness also executes 512 independent mock quota buckets with a two
millisecond simulated operation. At global caps of 1, 8 and 64, the observed
peak in-flight counts matched those caps, every operation succeeded, and final
in-flight counts were zero. The final run reported approximately 313, 2,378 and
18,891 mock operations per second respectively. This measures local scheduling
with artificial waits; it excludes real network, AWS serialization, service
quotas, credential work and throttling.

Reproduction command:

```bash
cargo run -p inventory-engine --release --example benchmark --locked -- 10000
```

## Initial native FalkorDB execution

The graph path was executed against official FalkorDB 6.0.4 loaded into a local
Redis 8.10.3 server. The downloaded module binary's SHA-256 was:

```text
0e71f1b4add13ab1831dc68dc03fdb715e0e813e23216163cc5ab37b0ae3f1f2
```

The server bound only to loopback, used an isolated synthetic graph, and was
stopped after the checks. Server binaries are not part of the source delivery.

Observed CLI results, retained in `artifacts/falkordb-smoke.json`:

| Case | Applied batches | Result |
|---|---:|---|
| Initial synthetic projection | 5 | 7 resource nodes and 4 relationships |
| Resume using the same SQLx delivery checkpoint | 0 | No duplicate writes |
| Replay using a fresh delivery checkpoint | 5 | Still 7 nodes and 4 relationships |

The opt-in Rust test `live_falkordb_replay_and_parameter_round_trip` also passed
against that server. It checks quotes, backslashes, newlines and Unicode,
injection-shaped property data, an unresolved target placeholder, the completion
marker, acknowledged resume and full replay. Its output is retained in
`artifacts/falkordb-live-test.txt`. This test is ignored in ordinary test runs
because it requires an explicitly supplied server URL.

### Historical 0.2.0 topology projection and replay

The new architecture fixture was projected into the same locally isolated
FalkorDB/Redis versions, with a fresh synthetic graph and fresh SQLx checkpoint.

| Delivery | Applied batches | Observed graph after delivery |
|---|---:|---|
| Initial projection | 5 | 20 Resource nodes, 40 relationships, 1 unresolved placeholder |
| Resume with the same checkpoint | 0 | No new acknowledged batches |
| Full replay with a fresh checkpoint | 5 | Still 20 Resource nodes, 40 relationships, 1 unresolved placeholder |

The 20 graph nodes comprise 19 observed resources plus the referenced peering
placeholder. The separate `InventoryScan` marker retained `state=partial` after
replay. It did not become complete merely because graph delivery succeeded.
The opt-in `live_falkordb_replay_and_parameter_round_trip` Rust test then passed
against that server, including special characters and injection-shaped data.

Evidence is retained in `artifacts/falkordb-smoke-v02.json` and
`artifacts/falkordb-live-v02.log`. The server was stopped in the harness's cleanup
path, and its binaries and temporary databases are not part of the source bundle.

## What has not been established

No live AWS account was scanned in this workspace. The implementation's native
resource mapping and pagination tests do not establish live IAM permissions,
cross-account behavior, regional endpoint coverage, real service throttling,
credential refresh under load, or counts across a user's AWS estate.

The delivered coverage is 32 direct resource/component kinds in `COLLECTORS.md`,
200 catalog-selected Cloud Control types in `COLLECTION_PLAN.md`, and records
returned by explicitly selected or discovered Config and Resource Explorer sources.
The native and Cloud Control sets overlap, giving 212 distinct catalog kinds.
Existing aggregators/views and Organizations hierarchies can be discovered;
the scanner does not create sources or roles. Organizations target fan-out and
same-account role assumption are implemented, while positive cross-account
enumeration cannot be established with this emulator's stubbed operations and
account-isolation limitations. The durable queue restarts whole scan attempts;
automatic within-scan pagination resume, all-service schema parity, deletion
inference and distributed rate coordination remain expansion work.

Supplemental naming recognizes the new networking and actual ELBv2 resource
kinds, but source detail remains different. Config's raw nested routes/actions
remain under `configuration`; Explorer stays metadata-only. Neither supplemental
source currently emits native-equivalent relationships or target-health membership.
Cloud Control emits its catalog-defined references from sanitized provider
fields; it is not equivalent to native topology enrichment. Its catalog includes
regional list/read types, with global and parent-model exclusions documented.
Its bounded list/read and redaction checks run after the SDK deserializes the
response. They do not prove a hard network-body or process-RSS ceiling. No
complete native tag/authentication/certificate enrichment or effective network
reachability verdict is claimed.

No sustained million-resource benchmark, peak-RSS bound, allocation profile,
tail-latency distribution, production release binary, dependency advisory audit,
SBOM, signed provenance, mutation score or code-coverage percentage is claimed.
`RUST_REVIEW.md` records the unavailable tools and incomplete formal skill gates.

The application and collectors are Rust. SQLx calls the native SQLite C engine,
FalkorDB is an external server, and catalog acquisition/regeneration uses
development-time Python utilities. The SDK source auditor and SQL view generator
are Rust. There is no Python or Go collector subprocess in the scan path.
