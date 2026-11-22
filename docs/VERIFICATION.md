# Verification and delivery boundaries — 0.6.0

> Status: Historical baseline evidence for collector version 0.6.0.
> The counts below precede the dependency refresh. Current package gates are
> recorded in [ROADMAP](ROADMAP.md), and current runtime gates in `HARNESS.md`.
> Links into `artifacts/` point to an ignored local directory and do not resolve
> in a clone. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

## Current Moon CI contract

The canonical CI entrypoint is `moon ci`, configured for the root Cargo workspace as one Moon project. CI runs formatting, source/function size guards, pinned AWS model provenance, generated parent/model/field/endpoint/operation-class/collection-plan/overlay/binary-catalog/SQL-view checks, all-target/all-feature workspace `cargo check`, source-only Clippy, all-target/all-feature tests, and the workspace build. Cargo tasks share the `cargo-workspace` mutex and form an explicit dependency chain, so generated checks and compilation cannot overlap. The pinned Rust channel remains `1.100.0 stable`; Moon toolchain synchronization is disabled.

GitHub Actions checks out the AWS model submodule recursively and installs the native Linux packages required by Makepad before `moon ci`. A `v*` tag runs the same CI gate and uploads the resulting Linux `aws-inventory` and `cmdb-ui` debug binaries as build artifacts; it does not deploy or publish a release.

Both hosted workflows bootstrap `rust-toolchain.toml` with `rustup show` and
verify `cargo --version` before Moon setup. The first pushed Moon run
([43582215000](https://github.com/lprior-repo/aws-inventory-rust/actions/runs/43582215000))
failed because Moon's Rust installer did not provide the pinned toolchain's
`cargo`; the bootstrap makes that runner prerequisite explicit.

The Linux prerequisites include `libpulse-dev` and `libdrm-dev`: Makepad links
PulseAudio and DRM when the workspace UI test target is built.

Run [43606088788](https://github.com/lprior-repo/aws-inventory-rust/actions/runs/43606088788)
passed toolchain/native setup and 15 Moon tasks, then failed the literal AppConfig
experiment fixture: its parent selector used `ApplicationId` instead of the
reviewed `ApplicationIdentifier`. The correction retains the recorded nested
provider properties; the targeted parent/architecture-edge regression passes.

Commit `9841354b2b61ad470c5b8adffaddf4a2baefee35` passed hosted `moon ci`
in [run 43609151434](https://github.com/lprior-repo/aws-inventory-rust/actions/runs/43609151434).
The complete CI job finished successfully in 18m3s, including Makepad native linking.

### Live emulator end-to-end — 2026-11-14

Environment: emulator 1.0.348 at `https://127.0.0.1:47666` with its own CA,
native FalkorDB at `redis://127.0.0.1:6379`, isolated static profile files
(`.local/e2e-env.sh`), debug binaries, catalog format 3 with runtime extension.

The suite now starts its own pinned emulator (per-test `HOME`, loopback, auth on)
and its own `redis-server` with the FalkorDB module, writes an isolated static
profile from the credentials the emulator issues, and stops all three on drop.
Install the runtime once with
`cargo run -p inventory-harness -- install .local/runtime` and
`cargo run -p inventory-harness -- install-graph .local/runtime/graph`;
`INVENTORY_EMULATOR_BINARY`, `INVENTORY_REDIS_SERVER` and `INVENTORY_FALKOR_MODULE`
override the default locations.

`cargo test --features aws --test sweep_e2e -- --ignored --test-threads 1`
passed both tests in 158 s:

- `production_sweep_persists_and_projects_real_seeded_inventory`: Rust-typed
  VPC/subnet seeding through `aws-wire`; CLI sweep exit 0; SQLite counts,
  identities and `IN_VPC` edges equal the oracle; first projection with an
  unreachable graph keeps the outbox, the next CLI run drains it, and every
  Falkor edge matches SQLite with `reviewed-rule` provenance; unchanged rerun
  writes zero rows; an interrupted sweep held at a signed continuation by a TLS
  barrier proxy is killed, reaped and resumed under the same sweep ID; repeated
  one-page partial runs exit 2 and delete nothing.
- `controlled_tls_403_preserves_existing_inventory`: a separate rcgen CA/leaf
  peer verifies SigV4 and answers enumerations with EC2 403; two CLI runs exit 2
  with inaccessible scopes, zero resource writes and an identical snapshot of
  resources, edges and miss counters.

A manual sweep into `.local/e2e/run1/inventory.sqlite` stored 7 VPCs and 9
subnets with 9 resolved `IN_VPC` edges; FalkorDB graph `aws_inventory_live`
returned 16 nodes and 9 edges, and `cmdb-ui` displayed the counts, detail panel
and graph neighbourhood from the same stores. Debug sweep wall time fell from
31.5 s to 12.8 s after the `ShapeOffsets` index (same scopes, 64 resources).

Defects found only by the live run and fixed: catalog blob exceeding
`SQLITE_MAX_LENGTH`; length-prefixed single-field identities; kind schemas
without relation target columns; `Model` instead of `ReviewedRule` provenance;
writer emitting `Null` binding fields the reader rejects; quadratic service and
shape lookup; EC2 trailing-slash namespace; edge pages always returning a
cursor; viewer graph worker creating a Tokio timer outside its runtime.

Final gates on the landed tree: workspace all-target/all-feature tests 1,809
passed, 0 failed, 21 ignored; warnings-denied source Clippy clean; every
generated-data `--check` (including `models compile --check`) passes;
`moon ci --force` completed 18/18 tasks; the live suite was rerun after the
size refactors and passed both tests in 156 s. Files ≤300 lines and functions
≤25 lines hold after `cargo fmt`.

### Response-decoder integration — 2026-11-14

`aws_wire::response::decode` routes all seven protocols into bounded canonical
records or classified, display-redacted service errors. Coordinator execution
of the library plus the three response fixture targets passed 64 tests.
Warnings-denied `aws-wire` source Clippy also passed.

Published successful-response accounting:

| Protocol | Client cases decoded and compared | Server-only cases excluded |
| --- | ---: | ---: |
| awsJson1.0 | 25 | 2 |
| awsJson1.1 | 48 | 1 |
| restJson1 | 94 | 8 |
| awsQuery | 36 | 0 |
| ec2Query | 27 | 0 |
| restXml | 81 | 2 |
| rpcv2Cbor | 40 | 2 |

All 351 client cases invoke the public root decoder and compare recursive
record values against the published parameters through real compiled fixture
catalogs. Separately, 57 published client modeled-error cases assert exact
codes and classes through the public root decoder. Optional unset structure
members normalize to absence; sparse collection and document nulls do not.
Additional regressions cover malformed input, resource bounds, XML union
cardinality/defaults and decimal/negative/floating timestamp precision.

The pinned real-model catalog was regenerated with S3 unwrapped-output metadata:
10,995,362 bytes. This does not add reviewed overlays or callable operations.
These checks do not establish fuzz coverage, cross-format byte/hash parity,
allocation/throughput results or the universal emulator-to-SQLite-to-FalkorDB-
to-Makepad collection pipeline.

Static review inputs were adjudicated against source and coordinator evidence,
not accepted as execution proof:

- Recursive mismatch suppression: no action; `compare_value` returns `()` and
  asserts every recursive result, so callers cannot discard a failing boolean.
- Same-sized unexpected keys: no action; expected members require their exact
  field ID or map name, in addition to the exact entry count.
- Overlapping error accounting or malformed fixture statuses: no action for
  this pinned corpus. Coordinator Bun enumeration found 57 client error cases,
  zero invalid statuses and zero IDs shared between error and non-error owners
  across all seven files.
- S3 unwrapped-root rejection: no action; the published
  `GetBucketLocationUnwrappedOutput` case supplies output `xmlName` equal to
  `LocationConstraint`, which root validation honors. That case passes within
  the 81 REST XML client cases.

The first canonical gate exposed a 302-line trait module and stale generated
catalog. The retained-trait list was extracted without changing its entries;
the catalog was written with `models compile --write`. The focused source-size
and binary-catalog checks then passed: 1,164 Rust files at most 300 lines and
10,995,362 catalog bytes.

Final coordinator execution of `moon ci --force` passed all 18 tasks, including
generated-data checks, binary-only core tests, all-target/all-feature workspace
check/tests/build, warnings-denied source Clippy and Makepad native linking.
Ignored live component tests are not counted as executed coverage.

### DNS cancellation repair — 2026-11-14

The wire connector now owns a Hickory asynchronous resolver and its 32-lookup
admission limit. Dropping a lookup releases its permit without leaving an
uncancellable OS resolver worker. System DNS configuration, search domains and
hosts files are used; configuration/load failures propagate during client
creation, without a public-DNS fallback. Platform-specific NSS plugins are not
reproduced.

Coordinator execution passed `cargo test -p aws-wire --lib --test transport
--locked --offline -j1`: 118 tests across two suites. The new local UDP DNS
regression cancels an unanswered query, answers a subsequent query and checks
the resolved address. Warnings-denied source Clippy also passed. These are
narrow transport checks, not a new full-workspace or live-sweep result.

### Request and streaming-record integration — 2026-11-14

The completed integration gate passed formatting and
`cargo test --workspace --all-targets --all-features --locked --offline -j1 --no-fail-fast`:
1,712 tests passed across 128 test runs, zero failed, 17 ignored.
`cargo build --workspace --all-targets --all-features --locked --offline -j1`
then passed, including Makepad native linking. Ignored live component tests
are not counted as executed coverage.

Coordinator-executed checks in the isolated `request-landing` worktree:

- `cargo test -p aws-wire --locked --offline --lib request::`: 38 passed.
  This includes both Query fixture runners, 142 REST JSON and 109 REST XML
  client cases, seven adversarial XML-comparator scenarios, and rejection of
  a selected REST protocol absent from the service metadata. Expected request
  URIs, resolved hosts, headers and bodies are not rewritten by the runner.
- `cargo test -p aws-wire --test request_body_fixtures`: 18 passed for the
  body-only encoders and their numeric/semantic comparisons.
- `cargo test -p inventory-core --lib record::builder`: five passed, covering
  canonical byte parity with the existing encoder, ordered collections,
  duplicate/state rejection, terminal failure, and byte/node/depth limits.
  This is not an allocation, throughput or independent hash benchmark.
- `cargo test -p catalog-tool --locked --offline --lib smithy::compile::defaults`:
  seven passed after replacing unchecked numeric conversions and handwritten
  base64 parsing. Source Clippy for `catalog-tool` passed with warnings denied.
- `cargo test -p aws-inventory --locked --offline --test catalog_models`:
  passed against the independent committed model report. The dedicated
  `awsQueryCompatible` flag contributes to report counts without becoming a
  selectable wire protocol.
- Warnings-denied source Clippy passed across the workspace; after the protocol
  and report repairs, it also passed for the changed `aws-wire` and
  `inventory-core` libraries. Both source guards passed: 1,104 Rust files,
  none over 300 lines. The function guard covers its configured paths, not
  every workspace function.

Generated-data checks passed for the pinned 436 models, 49,069 field IDs,
endpoint cases, operation classes, collection plans, reviewed overlays,
10,995,359-byte binary catalog and SQL views. The overlay report currently
contains zero reviewed services and zero callable operations; passing its
validator does not authorize unreviewed enumeration.

Independent record-builder review reported no concrete defects. Request
review found the REST protocol-metadata mismatch above; the production fix
and public encoding regression passed coordinator execution. Reviews are
static input, not substitutes for the command evidence.

Response decoders and the universal emulator-to-SQLite-to-FalkorDB-to-Makepad
pipeline are not proved by this request/record landing.

### Universal request transport smoke

The model-driven request encoders, SigV4 signer and pooled HTTPS client were
exercised against the authenticated local AWS emulator at `https://127.0.0.1:47666`.
All seven read-only calls returned HTTP 200; changing one signature byte returned
HTTP 403 for every call. The calls were DynamoDB `ListTables` (AWS JSON 1.0),
CloudWatch Logs `DescribeLogGroups` (AWS JSON 1.1), IAM `ListUsers` (AWS Query),
EC2 `DescribeVpcs` (EC2 Query), API Gateway `GetRestApis` (REST JSON),
S3 `ListBuckets` (REST XML), and CloudWatch `ListMetrics` (RPC v2 CBOR).
The request guard rejected endpoints outside that explicit emulator address;
no resources were mutated. The local log is `.oracles/wire-emulator-v39.log`.
This proves request serialization, authentication and transport, not response
decoding or the universal SQLite → FalkorDB → Makepad collection pipeline.

The generated collection-plan and binary API-catalog checks verify deterministic artifacts against the pinned inputs, not live collection parity. The ignored live FalkorDB test verifies graph delivery behavior, and the ignored prepared-emulator snapshot tests verify specific collector components against a frozen snapshot. Both opt-in Moon tasks require their explicit service/environment prerequisites and neither is a universal AWS E2E pipeline. Universal resource coverage, real AWS semantics, and universal live-emulator collection remain unfinished; see [CLOUDCONTROL_COVERAGE](CLOUDCONTROL_COVERAGE.md) and [UNIVERSAL_COVERAGE](UNIVERSAL_COVERAGE.md).

Local verification commands:

```sh
moon query tasks
moon task-graph
moon ci
moon run :live-falkordb-component
moon run :live-emulator-component
```

The live commands are opt-in and fail preflight when required prerequisites are absent. FalkorDB requires `FALKORDB_TEST_URL` pointing to an isolated disposable graph service. Emulator snapshot execution requires a locally prepared, authenticated emulator, its generated endpoint/credentials/configuration/CA, explicit `INVENTORY_EMULATOR_TEST=1`, and the independently verified snapshot and baseline-summary paths and SHA-256 values required by the test. Never provide ambient AWS credentials or use a production account.

## Historical development tree (earlier 2026-11-13)


- **781 Rust tests passed, zero failures, 15 ignored** across the workspace
  (`cargo test --workspace --all-targets`).
- All-target/all-feature clippy is clean for the application workspace (the SDK catalog is excluded); formatting, the 300-line source gate
  and the 25-line function gate pass.
- Operation coverage is live-scanned and verified: **76 exercised operations of
  20,537**, with 79 historical reviewed explicit operations at 80 production
  callsites plus 10 reviewed additive operations at 11 callsites across 19 SDK
  services; `coverage operations --check` and the additive `--check-additions`
  validator pass.
- Native live emulator phase: **263 attempts, 168 resources, 344 edges**, with
  SQS/SNS/KMS/Step Functions assertions passing. The prepared live baseline
  passes **2,664 checks with zero failures**, including the Config freeze and
  parent pagination gates.
- Workload oracle: 1M and 10M resources generated and independently verified
  (10,000 accounts, deletions, duplicates, reorders, empty/global cases); peak
  RSS under 5 MB.
- CMDB viewer: Makepad table and hierarchical DAG render in a live Wayland
  window; capture at `/tmp/dag-final.png` during development. No frame-time or
  API-wiring evidence yet.
- Remaining blockers are explicit: live AWS authorization/quota calibration,
  whole-project 95% mutation score, tombstone/retention semantics, event
  ingestion, distributed quota coordination, PostgreSQL evaluation, fault/soak
  campaigns, and full live-emulator harness repair.

> The blocker list above predates the vision. History and retention are out of
> the first version (V22), change events are deferred (V10), quota coordination
> across processes is not needed on one laptop (V08) and PostgreSQL waits for a
> measurement (V05); the 95% mutation score is not a roadmap gate.

### Expanded parent workflow state

This increment adds **three additional parent-dependent Cloud Control types**,
bringing the configured catalog to **421 types across 76 service families**
(397 root + 24 parent-dependent). SQLx/SQLite schema version is now **13**;
migration 013 adds the views for the three new parent types.

**New parent-dependent types added:**

- `AWS::EKS::AccessEntry` (parent: EKS::Cluster)
- `AWS::EKS::PodIdentityAssociation` (parent: EKS::Cluster)
- `AWS::AppConfig::ExperimentDefinition` (parent: AppConfig::Application)

The configured descriptors now contain **968 property relationship rules** (up
from 965), and 268 types have at least one property rule or parent workflow, as
reported by `aws-inventory coverage`. Catalog JSON files, Rust code and tests
have been regenerated to reflect the expanded counts.

The retained v0.6 baseline documented below (530 tests, schema 6, 21 parent types)
predates this expansion and is kept for historical comparison.

The rest of this document records the retained v0.6 release run and is
historical; it is not evidence for the current development tree.

This increment adds **21 parent-dependent Cloud Control types**, bringing the
configured catalog to **418 types across 76 service families**. SQLx/SQLite
schema 6 exposes **4,273 provider columns**. The complete live first-party Rust
workspace passed **530 tests, zero failures and zero ignored tests**. The
populated official emulator collection passed **2,522 assertions**, both
repetitions of its five collector snapshots passed, and **all five snapshots
passed exact JSONL and real FalkorDB property verification**.

The successful owned run is `artifacts/emulator-v06-complete`. Its workspace
and enclosing command both completed with exit 0, and every owned process group
drained without forced termination. Every graph replay made zero attempts;
all five owned graphs were removed and checked after verification. The earlier
interrupted attempt remains separately preserved. The current source and
collection are bound by
[AWS_OPERATION_RECONCILIATION.md](AWS_OPERATION_RECONCILIATION.md).

**Universal AWS coverage and the whole-project 95% mutation target remain open.**
A successful test of an expected unsupported endpoint preserves that endpoint's
failure; it does not make the endpoint supported. Earlier results remain in
`VERIFICATION_V05.md` and
`VERIFICATION_V04.md`.

## Executed result index

The machine-readable `artifacts/verification-v06.json`
records the final scope, executable/source identities, case results and evidence
hashes. The complete owned fixture lifetime exited 0 in **330.158 seconds**.

| Check | Observed result | Evidence |
| --- | --- | --- |
| Ordinary Rust workspace | 520 passed, 0 failed, 10 ignored; 28 reported suites; 106.979 seconds including command overhead | Receipt (`artifacts/verification-v06/workspace-v06-retry3.json`), log (`artifacts/verification-v06/workspace-v06-retry3.log`) |
| Complete live Rust workspace | 530 passed, 0 failed, 0 ignored; 28 reported suites; 176.813 seconds | Receipt (`artifacts/emulator-v06-complete/workspace-live.json`), log (`artifacts/emulator-v06-complete/workspace-live.log`) |
| Exact SQLite → JSONL → FalkorDB verification | All five snapshots passed; all projected properties, identities and cardinalities match; replay made zero attempts; cleanup passed | Delivery report (`artifacts/emulator-v06-complete/delivery-verification.json`), enclosing command (`artifacts/emulator-v06-complete/mutation-command.json`), closed fixture lifetime (`artifacts/verification-v06/emulator-v06-complete.json`) |
| Portable retained exports | All five JSONL files byte-match the live-verified artifacts; all five JSON documents contain the same ordered record streams | Export receipts (`artifacts/emulator-v06-complete/portable-export-receipts.json`), command receipt (`artifacts/verification-v06/portable-exports-v06.json`) |
| Production build and Clippy | Exit 0; official Clippy driver in Cargo build mode, libraries/binaries/examples, warnings denied; 120.605 seconds | Receipt (`artifacts/verification-v06/clippy-build-mode-v06-first.json`), binary identities (`artifacts/verification-v06/frozen-binaries.json`) |
| SDK auditor Clippy | Exit 0; ordinary `cargo clippy` for the SDK catalog audit example, warnings denied | Receipt (`artifacts/verification-v06/sdk-auditor-clippy-v06.json`) |
| Rust formatting | Exit 0 | Receipt (`artifacts/verification-v06/fmt-check-final.json`) |
| Generated SQLite views | Exit 0; frozen migrations 003/005 preserved and additive migration 006 checked | Receipt (`artifacts/verification-v06/sql-views-check-final.json`) |
| Parent catalog regeneration | Exit 0; selected schema hashes and generated descriptors match | Receipt (`artifacts/verification-v06/parent-catalog-check-final.json`) |
| Schema helper tests | 19 passed | Receipt (`artifacts/verification-v06/schema-helper-tests-final.json`) |
| Catalog/auditor helper tests | 38 passed | Receipt (`artifacts/verification-v06/catalog-helper-tests-final.json`) |
| Emulator helper tests, including supervision regressions | 125 passed | Receipt (`artifacts/verification-v06/emulator-helper-tests-v06-supervision-final.json`) |
| Mutation acceptance/supervisor helper tests | 126 passed; this is not a Rust mutation score | Receipt (`artifacts/verification-v06/mutation-helper-tests-final.json`) |
| Locked SDK dependency fetch and archive verification | 437 archives verified, totaling 241,799,148 bytes | Fetch (`artifacts/verification-v06/sdk-fetch-v06.json`), verification (`artifacts/verification-v06/sdk-cache-verification-v06.json`) |
| Fresh Rust SDK source audit | 437 SDKs, 19,617 operation modules, 3,779 paginator modules and 407 waiter modules | Audit (`artifacts/sdk-capabilities-v06.json`), command receipt (`artifacts/verification-v06/sdk-source-audit-v06.json`) |
| Populated official emulator collection | 2,522 assertions passed, zero assertion failures; expected partial outcomes retained | Summary (`artifacts/emulator-v06-complete/summary.json`) |
| Repeated snapshot oracle | Two successful runs, each with all five cases passing | First (`artifacts/emulator-v06-complete/oracle-baseline-1.json`), second (`artifacts/emulator-v06-complete/oracle-baseline-2.json`) |
| Current mutation candidate enumeration | 2,959 candidates; no candidate executed by this command | Manifest (`artifacts/mutation/enumeration-v06/manifest.json`), receipt (`artifacts/verification-v06/mutation-enumeration-v06.json`) |

Ordinary and live Rust suites overlap; their counts must not be added as distinct
tests. Property-test examples are not additional named Rust tests. The ten live
cases comprise five collector snapshot tests, two CLI/durable-worker tests, and
three native FalkorDB tests. The SDK catalog is dependency-only and excluded
from the application workspace test command; its separate auditor is checked
as recorded above.

The four distinct Python helper suites contain **308 passing tests** in total.
Focused reruns of those same tests are not additional tests.

## Parent collection and architecture behavior

The new selection includes eight API Gateway REST types, five API Gateway V2
types, two AppConfig types, three EKS types, ELBv2 Listener, and two CloudWatch
Logs types. The complete names and owner-field contracts are in
[PARENT_COLLECTIONS.md](PARENT_COLLECTIONS.md) and the
[parent catalog](../catalog/cloudcontrol-parent-resources.toml).

Selecting a child adds its required root exactly once. Related child collectors
share root discovery, retain independent parent models and continuation cursors,
and follow empty pages that still have a token. Returned properties must match
the expected owner before the collector emits a `CHILD_OF` relationship.
Malformed or incomplete prerequisite discovery cannot become a successful empty
child inventory. Duplicate models, repeated identifiers, page limits and bounded
fan-out retain explicit outcomes.

The actual parent scan stored **179 resources: 18 prerequisite resources and
161 children**. It produced **185 relationships: 161 `CHILD_OF` edges and 24
additional architecture edges**, with **269 complete scopes and zero failed
scopes**. All **1,328 parent assertions** passed. The fixtures include 101 EKS
add-ons: one parent receives 100 matching items and an empty terminal page;
another receives an empty nonterminal page followed by its child. Independent
fixture filtering and the compiled Rust collector both exercised these cases.
Parent scan (`artifacts/emulator-v06-complete/parents.json`) ·
Assertions (`artifacts/emulator-v06-complete/parent-assertions.json`) ·
Python fixture preflight (`artifacts/emulator-parent-seed-v06/summary.json`)

An in-process HTTP fixture separately invokes the production Cloud Control SDK
path, injects an initial HTTP 429 throttle, and checks models, cursors, attempt
counts and retry counters. It is part of the passing ordinary Rust suite.
Two generated-data properties exercise parent identity/ownership contracts.
The outer wire-test deadline is reported as an unresolved infrastructure timeout
in mutation accounting, never as a caught mutation.

The configured descriptors contain **965 property relationship rules across
255 types**, plus **21 parent workflows**; **265 distinct types** have either
form of relationship evidence. These are configured rules, not a claim that
every rule was observed. The additive SQLx migration preserves previous
observations, fingerprints, scans, provider views and user-owned objects, and
rolls back conflicts transactionally. New migration and plan tests ran in the
ordinary Rust suite. Catalog totals (`artifacts/verification-v06/catalog-summary.json`)

## Retained SQLite snapshots

These are separate synthetic snapshots. Root and parent scenarios overlap on
prerequisites; adding their row counts would overstate distinct AWS resources.

| Snapshot | Collection status | Resources | Relationships | Scopes | Failures | FalkorDB resource nodes | Unresolved references |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native services | complete | 150 | 340 | 131 | 0 | 193 | 43 |
| Config / Resource Explorer | partial | 11 | 0 | 3 | 2 | 11 | 0 |
| 397 root Cloud Control types | partial | 401 | 32 | 798 | 11 | 426 | 25 |
| 21 parent-dependent types and prerequisites | complete | 179 | 185 | 269 | 0 | 185 | 6 |
| Assumed-role IAM | complete | 13 | 22 | 24 | 0 | 25 | 12 |

The graph columns above were independently derived from SQLite and exactly
matched the actual FalkorDB graphs. A separate `InventoryScan` marker belongs
to each projected graph. Unresolved identities remain references and are not
counted as collected resources.
Database (`artifacts/emulator-v06-complete/inventory.sqlite`) ·
Database assertions (`artifacts/emulator-v06-complete/database-evidence.json`)

The strengthened delivery verifier checks every projected resource property,
unresolved-node property, edge property and scan-marker property, along with
labels, property cardinalities, exact identity sets, endpoints and relationships.
Nested JSON is compared structurally while preserving types, arrays, SQL NULL
and the JSON string `null`. JSONL records are checked against independently
joined scope/page provenance and raw SQL payloads, with framing, ordering, EOF,
bytes and hashes verified. Successful replay must make zero graph attempts,
apply zero batches, retain exactly the same graph and export, and remove the
owned graph during cleanup. Its 38 helper tests include negative controls for
same-count wrong content and extra properties. All five current graph cases
passed these checks. Initial pushes applied and acknowledged 6, 4, 6, 5 and 5
batches in the table's order. Every replay applied zero batches, skipped exactly
those acknowledged batches and made zero attempts. Each case then removed its
owned graph and independently confirmed its key no longer existed.
Exact delivery report (`artifacts/emulator-v06-complete/delivery-verification.json`)

All five snapshots are retained in both JSON and JSONL under
`artifacts/emulator-v06-complete/exports/`.
The ten files were regenerated offline with the same frozen collector binary.
Each JSONL's bytes and SHA-256 match the corresponding live-verified artifact;
each JSON document matches its JSONL header, ordered records and footer
structurally. Partial snapshots retain their partial status and exit 2.
The bound database and input hashes remained unchanged. The complete export
commands, source/report hashes and output receipts are retained in
`artifacts/emulator-v06-complete/portable-export-receipts.json`.
These are synthetic fixture inventories. JSON/JSONL inventory export is
distinct from ingest's event format and the graph export's Cypher plan format.

## AWS operation and provider limits

The source audit reconciles **72 explicit operation identities at 73 send sites**,
across **15 SDK service namespaces**, plus the separate AssumeRole credential
provider. All **73 operation boundaries** appeared in the request ledger.
**64** have successful production collector/provider classifications; **four**
were successful explicit SDK probes; **two** Organizations operations returned
501; **three** Resource Explorer responses were rejected by the SDK decoder.
The latter remain failures despite HTTP 200. No endpoint or SDK parser was
patched to convert those failures into successes.
[AWS_OPERATION_RECONCILIATION.md](AWS_OPERATION_RECONCILIATION.md)

Every one of the 397 root descriptors received a list attempt; **386 root types**
had detailed reads. All **21 new child types** had list and detailed read evidence.
There are therefore **407 types with successful detailed reads out of 418
configured types** in this fixture. Eleven selected root namespaces are missing
from the pinned emulator: eight Bedrock types, S3 Express DirectoryBucket,
S3 Tables TableBucket and Scheduler ScheduleGroup. Their failures remain in the
partial root snapshot. The operation ledger contains **2,673 requests**, of which
**1,336** belong to Rust collectors/provider/probes and **1,337** to Python
fixture/startup work; setup calls cannot satisfy collector coverage.

The pinned schema universe contains **1,881 types**. Selection leaves **1,463
excluded types**: 180 without both required read/list handlers, 389 needing
additional parent workflows, and 894 with unreviewed scope. Native/global
adapters cover some families excluded from the generic adapter. The embedded
CLI coverage matrix deliberately preserves source-bound historical root
results; current parent execution is reported separately rather than relabelled
as historical evidence. [CLOUDCONTROL_COVERAGE.md](CLOUDCONTROL_COVERAGE.md)

Most emulator Cloud Control models use a generic property store. These checks
exercise signing, SDK serialization/decoding, mapping, redaction, ownership,
pagination, storage and delivery for the authored fixtures. They do not certify
real provider creation semantics, all compound identifiers, regional support,
IAM combinations, eventual consistency or exhaustive architecture semantics.
The 32 direct/native resource kinds overlap with the Cloud Control descriptors;
437 available SDK dependencies are not 437 implemented collectors.

## Retry, scheduling and workflow ownership

Explicit AWS inventory calls have one retry owner: the shared `ApiExecutor`.
The SDK is configured for one attempt so every retry is admitted and counted
through the global/account/region/quota-family limits. Default collection uses
five total attempts, bounded at ten. Tests cover throttling, transient failures,
per-attempt timeouts, bounded jitter, server delay hints, cancellation and
shared cooldowns. Credential-provider refresh is a separate boundary.

A fenced SQLx durable job queue persists scan attempts and recovery state.
JSON/FalkorDB delivery can retry a finalized SQLite scan without repeating AWS
collection. Graph delivery reconnects with bounded deadlines and replays only
recognized idempotent projections, recording acknowledgements after server
success. The live tests include an ambiguous network failure after a write was
committed, to verify replay without duplicate graph content.
[GRAPH_DELIVERY.md](GRAPH_DELIVERY.md) · [DURABLE_JOBS.md](DURABLE_JOBS.md)

Limits are configured and adapt to feedback; this is not automatic discovery
of every account's service quotas. No live AWS quota calibration or claim of
hundreds of thousands of simultaneous requests is established by local tests.

## Preserved failed attempts and corrections

- The first v0.6 ordinary suite found an old CLI version expectation; the second
  found two old schema-version expectations in migration fixtures. Those tests
  were corrected to assert the current manifest/schema contract. The subsequent
  full ordinary suite passed; both failed logs remain preserved.
- `artifacts/emulator-v06` passed all 2,522 collection assertions, then its
  package-only snapshot command triggered an unnecessary SDK feature rebuild
  and reached the 600-second compilation deadline. No oracle tests ran in that
  command. Workspace/all-feature selection fixed the build selection; a
  `--no-run` check reused the SDK cache and completed in 3.036 seconds.
- `artifacts/emulator-v06-verified` stopped before collection when the emulator
  version probe reached its ten-second startup deadline. That bounded startup
  probe now allows thirty seconds. It did not produce an inventory database.
- `artifacts/emulator-v06-final` passed collection and both five-case baselines,
  then the outer supervisor recorded `fixture_unavailable` / `URLError` during
  the full workspace command. Its old diagnostics did not retain the reason,
  so the precise cause of that health failure is unresolved. A separate harness
  defect was identified and fixed: SIGTERM bypassed the nested supervisor's
  cleanup and left its Cargo child and receipt unfinished. The original attempt
  is retained and cannot be counted as a completed pass. The separate fresh
  `emulator-v06-complete` run passed the entire workspace and delivery checks.

The failed attempt receipts are under
`artifacts/verification-v06/`. No test threshold,
unsupported endpoint, snapshot expectation or mutation denominator was relaxed
in response to these failures. The strict three-second health probe timeouts
remain unchanged. The supervisor now handles SIGINT/SIGTERM, drains owned groups
including members whose leader exited, and writes terminal receipts. Its
35-second outer / eight-second nested cleanup budgets allow the inner owner to
finish before outer escalation. Twelve focused real-process/helper regressions
passed, and an independent reviewer reproduced them.
Source-bound supervision receipt (`artifacts/integration-supervision-tests-v06.json`)

## Mutation status

**The current whole-project 95% mutation gate has not passed.** Fresh enumeration
produced **2,959 candidates**: 56 core, 810 engine, 1,683 AWS and 410 CLI. This
command executed zero mutations. The historical core result of 56/56 remains
historical; interrupted other-package campaigns are diagnostic evidence only.

The repaired final aggregate requires complete source-bound outcomes, two
clean baselines, all ten named live cases, consistent frozen fixture identities
and a healthy closed fixture lifetime. Unit-only partitions cannot establish a
whole-project pass. Timeouts, OOMs, toolchain/cache failures, fixture failures
and missing results remain unresolved. The threshold is unchanged: at least
95% of viable candidates must be caught, with no unresolved outcomes.
[MUTATION_TESTING.md](MUTATION_TESTING.md) ·
`MUTATION_READINESS_V06.md`

## Build identity and reproduction

The collector is `aws-inventory 0.6.0`, built with pinned
`1.100.0 stable`, locked dependencies and one Cargo job. The frozen production
executable SHA-256 is
`57660e895513595a9247fc22bd9b80c921417de9c2fb32a59f250830d0728adf`;
its source closure SHA-256 is
`a79a481e63ac3fafba9fe5501b7b347b4c40ced4df899567e222f7069dc6fd29`.
The two executable identities and exact build mode are recorded in
`artifacts/verification-v06/frozen-binaries.json`.

The complete official AWS Emulator source checkout is pinned to commit
`f515581569b98a9dcbd2e686c12f912fdce51bba`; the signature-verified executable is
1.0.336. The native fixture pins Redis 8.10.3 and FalkorDB 6.0.4. A successful
installer smoke check is retained separately from graph projection proof.
All fixture endpoints are loopback and both TLS hops remain verified.

Production Clippy used the official driver in Cargo build mode because the
separate metadata-only EC2 compilation previously exceeded this host's memory.
This establishes the recorded production library/binary/example lint result;
it is not a claim that the canonical metadata-only `cargo check` or every
all-target lint configuration passed on this host. The SDK auditor's separate
ordinary Clippy invocation passed. All 437 SDK archives were inspected; all
437 optional service crates were not compiled or exercised against endpoints.

[START_HERE.md](../START_HERE.md) provides build and collection commands.
[EMULATOR_TESTING.md](EMULATOR_TESTING.md) provides the complete owned-lifetime
integration command. The wrapper is named `mutate.py`, but invoking the ordinary
workspace/delivery verifier through it is **not a mutation campaign**.

## Moon CI and artifact delivery

`moon ci` is the canonical current maintainer gate. Earlier results in this
document remain historical evidence, not a pass for the current source.
Moon 2.2.4 and 1.100.0 stable are pinned in `.moon/`.

The single Cargo workspace project runs formatting, source and function
bounds, model provenance, generated parent candidates, model statistics,
field IDs, endpoints, operation classes, collection plans, the binary catalog
and SQL views. It then checks the binary-only core configuration, all
workspace targets and features, production-only Clippy with warnings denied,
workspace tests and the final build. Cargo tasks share one mutex; quality
tasks do not reuse cached pass results.

On 2026-11-14 the integration checkout ran the gate after SDK cleanup
`68ad572`: exit 0, all 17 CI tasks completed in 8m 1s 236ms. This includes
the generated-data checks, both core feature configurations, the workspace
checks and tests, warnings-denied source Clippy and the Makepad build.
The two opt-in live tasks were not part of that run.

```sh
moon ci
moon run :lint-src
```

GitHub pull requests and pushes to `main` run this gate after recursive
submodule checkout and native Makepad dependency installation. Tags matching
`v*` also upload the built Linux debug collector and Makepad binaries. This
is artifact delivery, not deployment or a release-profile build.

The two live component tasks are opt-in and reject missing fixture settings:

```sh
moon run :live-falkordb-component
moon run :live-emulator-component
```

They exercise existing graph replay and frozen emulator snapshots. Neither
establishes the new model-driven sweep → binary SQLite → FalkorDB → Makepad
pipeline. That complete end-to-end proof remains pending until the runtime,
binary persistence, graph projection and viewer integration are implemented
and exercised together.
