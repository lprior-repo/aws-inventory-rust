# Changes

## Unreleased — Direction: endpoint wire engine

- `docs/VISION.md` and `docs/ROADMAP.md` now define the product and the build
  plan: direct service endpoint collection through a model-driven wire engine,
  binary records, hash-based reconciliation, full-send quota use, a laptop as
  the first runtime, and the CMDB layer in the first version. Cloud Control
  collection is retired as endpoint collection replaces it.
- The working tree was committed as the baseline (`78d3ac2`).
- M0 (`bbd0821`): `inventory_engine::store::SCHEMA_VERSION` is the single
  source of the SQLite schema version and `aws-inventory catalog` reports it
  (it reported a stale 12); both packagers exclude `.local`; `scripts/verify.sh`
  no longer invokes Python and runs the function-size check.
- M0.2/M0.3 verification after the dependency upgrade: the all-target workspace
  check and the full `bash scripts/verify.sh` gate pass. The gate ran with
  neither Python nor Python3 on `PATH`: source/function size guards, formatting,
  generated-data checks, split SDK-facade checks, Clippy, tests, doctests and
  the final build all passed.
- Preserve the concurrently merged O1 codec experiment under
  `evidence/record-codec-bench` without reverting production CBOR. Its
  `inventory-core` dependency now pins the original experiment's Git revision;
  its original measurements are explicitly historical, not current-codec results.
  Both size guards pass after integration (778 Rust files); the isolated
  benchmark's locked release run passes both fixtures and its cross-codec
  field assertions. Original result tables remain unchanged.
- M0.7a: `inventory-engine` and `cmdb-view` deny `missing_docs`. 378 public
  items are documented from their bodies, 22 are narrowed to crate visibility,
  and unused public functions are deleted (`Store::visit_edges`,
  `CoverageState::is_terminal_success`, three `LodLimits` getters and two
  `PageRequest` getters). An independent review found six comments that
  stated behavior the code does not have; all were corrected. Rust lines: 727
  added, 207 deleted.
- `crates/aws-wire` is scaffolded, and Smithy's AWS protocol test suites
  (Smithy 1.75.0, seven protocols, 901 HTTP cases) are committed as JSON ASTs
  under `crates/aws-wire/tests/fixtures/protocol-tests/` with the conversion
  script `scripts/protocol-tests/convert.sh` and provenance digests. The
  conversion uses the released Smithy CLI and `jq`; it is not part of the
  build or the gate.
- M3.1b: `Record::content_hash` (BLAKE3 through the `blake3` library) with
  field-ID exclusion paths, `FORMAT_VERSION = 1`, the borrowed validated view
  `MapRef::from_bytes`, and typed reads (`ValueKind`, `MapRef::get_typed`).
  The hash is computed in one pass without allocation. 50 new tests. Rust
  lines: 1,350 added, 19 deleted.
- M1.2b: the append-only field-ID registry `catalog/field-ids.txt` (49,069
  member names; the line number is the ID) and `catalog-tool models fields
  [--check | --write]`. The gate now also runs the submodule check, the
  model report check and the registry check. Rust lines: 1,525 added, 17
  deleted.
- M2.2a: `aws_wire::sign`, SigV4 request signing as a thin wrapper over the
  `aws-sigv4` library, proven with 37 of AWS's published test vectors. Rust
  lines: 1,270 added, none deleted.
- M1.5a: `catalog_tool::names`, the rule that turns a type name into a SQL
  table name (`AWS::EC2::VPCDHCPOptionsAssociation` is
  `aws_ec2_vpc_dhcp_options_association`), with the reviewed exceptions in
  `catalog/name-exceptions.tsv`. No command yet; `catalog/kinds.tsv` follows
  with M1.4. Rust lines: 1,368 added, none deleted.
- M3.1c, fixed: `Record::from_bytes` and `MapRef::from_bytes` accepted maps
  with unsorted or duplicate keys, for which lookups could report a present
  field as absent. Validation now rejects them (`KeyOutOfOrder`,
  `DuplicateKey`) and rejects unsupported key types up front. Bytes written
  by the encoder are unaffected. Rust lines: 644 added, 102 deleted.
- Fixed, found by an adversarial run against the built `inventory-harness`
  binary (41 commands, 2 defects): `assert cloud-control` exited 0 while
  printing failed checks (missing, empty, corrupt and wrong-schema databases
  all "succeeded"), and the commands that take `--flag value` pairs accepted
  unknown and repeated flags, so `install <dir> --unexpected x` created the
  destination before failing elsewhere. A failed check now exits 1 after
  printing the checks; an unknown or repeated flag is rejected before any
  work. Three tests run the binary.
- Fixed, found by an adversarial run against the built `aws-inventory`
  binary (72 scenarios, 1 defect): a scan that ends `partial` or `cancelled`
  exited 2 with nothing on standard error. It now prints one line there
  saying the report on standard output lists what failed.
- Fixed, found by an adversarial run against the built `catalog-tool`
  binary (160 commands, 5 defects, all in `package`): the archive was
  written through a predictable sibling name, so a symlink or a file already
  at that name was overwritten or destroyed; a destination inside the
  packaged root packaged itself; a `MANIFEST.sha256` that was a link to a
  payload truncated it. The archive, the manifest and generated SDK files
  are now published through an exclusively created temporary file and a
  rename, with the mode `File::create` would give; a destination inside the
  root is rejected; excluded directories are not entered; file count, depth
  and bytes are bounded before reading. The harness's second copy of the
  packager is deleted and re-exports this one. The destination directory
  must not be writable by untrusted users (the check is not atomic with
  publication). Rust lines: 1,069 added, 652 deleted.
- Fixed, found by the architecture audit: `cmdb-view` removed duplicate
  resource IDs only when they were adjacent after sorting by account, region
  and kind, so the same ID seen under two scopes was listed twice while
  lookup by ID returned one of them. A resource's identity is its ID: the
  first observation in input order is kept and every listed row is found by
  lookup. Rust lines: 245 added, 6 deleted.
- Fixed, found by the architecture audit, in the harness: the emulator
  process was a bare handle between spawn and readiness, so an early error
  could leave it running; `Emulator::stop` always returned success; snapshot
  documents were read without a size bound. One guard now owns the process
  from spawn to drop and kills and reaps it exactly once, `stop` returns the
  real error, readiness reads only complete log lines within a byte and
  attempt bound, and one bounded reader serves the snapshot and audit code.
  Rust lines: 742 added, 85 deleted.
- Project agents in `.omp/agents/` include the local GPU builders and
  `aws-build-sol` (ChatGPT Sol). Builders implement independent packages in
  parallel; the coordinator owns serialized compilation, formatting and tests.
- Decision V34: records are CBOR through the `minicbor` library. A codec
  benchmark (`docs/performance-evidence/record-codec-2026-11-13.md`) found it
  faster to build and smaller than our own format, `rkyv`, `postcard` and
  JSON. Our own format was deleted before landing; it and the benchmark are on
  the branch `evidence/o1-record-codec`.
- M1.1: the AWS API models are pinned as a shallow git submodule at
  `.local/aws-models` (commit `46135f8`, 436 files), checked by
  `scripts/check-models.sh`. Lock, verify and fetch commands of our own were
  built and discarded after review (decision V23 as changed).
- M1.2a (`cde710a`): stream the pinned Smithy JSON model files through
  `catalog-tool models report`; commit `catalog/aws-models.report` and support
  exact `--write`/`--check` workflows. The actual pinned source matches all
  expected model totals; 43 catalog-tool tests and all-target Clippy pass.
  Rust footprint: 1,494 lines added, 4 deleted.
- M3.1a (`5910de1`): canonical CBOR records through minicbor, validated scalar
  types, allocation-free borrowed lookup/iteration, generic framing validation,
  32-container ownership limits and a writer capped at 16 MiB before copying.
  122 core tests and all-target Clippy pass; independent reviews are closed.
  Rust footprint: 2,808 added including tests; 941 production physical lines,
  above the 500-line reporting target because validation and typed errors are
  retained. Content hashing and format version remain M3.1b.
- Every document now carries a status line (plan, kept and extended, baseline
  being replaced, or historical evidence) and `docs/README.md` indexes them by
  class. Statements the owner's decisions superseded are marked with the
  decision ID in `docs/ARCHITECTURE.md`, `docs/CMDB_ARCHITECTURE.md` and
  `docs/CMDB_PERFORMANCE_CONTRACT.md`. `CONTRACT.md` is declared the baseline
  contract.
- Corrected stale counts and links in `README.md`, `START_HERE.md`,
  `NEXT_AGENT_PROMPT.txt`, `docs/CMDB_ARCHITECTURE.md`, `docs/VERIFICATION.md`
  and `docs/UNIVERSAL_COVERAGE.md`. Removed the duplicate
  `docs/PERFORMANCE_CONTRACT.md`.
- M6.5 / V32: removed the obsolete `cmdb-api` crate, binary and HTTP endpoint
  tests, its workspace member, the direct `axum`/`tower`/`tower-http` workspace
  declarations, and its function-size gate path. Removed the launch command
  and updated current documentation; earlier API additions below are historical.
  `cmdb-view` still parses one JSON export and `cmdb-ui` still renders a
  synthetic fixture. In-process SQLite/FalkorDB reads and UI integration remain
  planned in M6.2/M6.3; unrelated transitive HTTP dependencies are retained.
  Rust footprint: 1,528 lines deleted.
- Refresh external Rust dependencies to the latest non-yanked stable registry
  releases, including all 437 existing exact-pinned AWS SDK crates; add
  `minicbor` 2.3.1 with `std` for `inventory-core` and regenerate all three Cargo
  lockfiles. Preserve optional/client features and migrate ZIP filename handling
  for ZIP 9.5.1 rather than retaining the older API.
- Refresh the existing SDK catalog's version/checksum/source provenance from
  upstream `release-2026-11-13` (`c88e7f9c26a66d0500bc1cc820ce73b76fa22136`),
  without adding typed service dependencies. Keep SQLx at reviewed revision
  `10462fa4414c6c7f1ed1d39bb31678e3b726878c` for SQLite 0.39 compatibility
  (published 0.9.0 excludes it), and Makepad 2.0 at
  `506ec05105b9adb08d1d21f9879df181b5244ea7` rather than downgrading to the
  registry's latest 1.0.0.
- Pair embedded and standalone AWS Emulator pins at stable `v1.0.348`, commit
  `7461ab1bbe540afe8063ce03ee5477ebb649df48`. Verify the downloaded package's
  detached AWS signature and SHA-256. All six authenticated standalone live
  tests and both authenticated embedded tests pass on this pin. Unify the embedded
  BigInt dependency at 0.12 after the initial upstream 0.12/0.14 type mismatch;
  no registry source was patched. Keep
  1.0.336 evidence historical and add the actual 1.0.348 universal capture:
  1,291 attempted, 999 listed, 292 unsupported, zero failed.
  Legacy baseline compatibility fixes retain SNS's authoritative provider
  identifiers and replace the stale 418-type selector ceiling with catalog
  membership validation; no Cloud Control collectors were added.
- Regenerate the SDK operation inventory from the actual registry and lockfile:
  24,720 operations across 468 cached versions and 437 services. Historical
  execution evidence never transfers SDK versions; receipts require matching
  source, version and module fingerprints. Current exact-version evidence
  credits 47 operations, not the former 76 cross-version rollup.

## Unreleased — Universal listable coverage, reference graph and Rust evidence harness

- Added native Step Functions state machine and activity adapters (schema 012),
  bringing native coverage to 39 kinds. `ListStateMachines`/`ListActivities`
  paginate and enrich each item with `DescribeStateMachine`/`DescribeActivity`
  and `ListTagsForResource` in per-item child scopes. Definitions, variable
  references and execution data are never stored; tag failures retain the item
  with `tags_collected=false`. `ASSUMES_ROLE` and `ENCRYPTED_BY` edges are
  emitted only for exact in-scope IAM role and KMS key ARNs, and logging
  destinations produce no CloudWatch edge. The `step-functions` selector also
  accepts `stepfunctions` and `sfn`.

- Added native KMS key and alias adapters (schema 011), bringing native coverage
  to 37 kinds. `ListKeys`/`DescribeKey` paginate and enrich keys, with
  customer-managed rotation status and tags read in per-key child scopes;
  `ListAliases` maps exact alias ARNs and keeps the target key ID as a property.
  Rotation and tag failures retain the key with explicit collection flags, and no
  key policy body, key material or alias-to-key edge is produced.

- Added universal Cloud Control listable coverage. `--resource-type all`
  attempts every list-capable type in the pinned schema (1,291 of 1,881), not
  just the reviewed 418, through one bounded shared adapter. Against the pinned
  emulator the verified partition is 1,291 attempted, 999 listed, 292
  provider-unsupported and 0 failed, with 410 parent-scoped and 180
  handler-less types reported as explicit skips. Scan JSON carries the full
  partition and names every skip.
- Added opt-in runtime reference edges (`--universal-references`, default off).
  Resolution uses a bounded, account/region-scoped identity index with exact
  ARN, unique primary identifier and ARN trailing-component tiers, emits
  idempotent synthetic `REFERENCES` edges, and streams resource observations so
  large scans are not materialised in memory.
- Added current-state freshness views (schema 007) with deterministic source
  preference: newer authoritative observations win, equal timestamps use source
  priority, and historical/per-source evidence remains available.
- Added an explicit account/region/collector coverage registry (schema 008):
  account onboarding, role identity, status transitions and per-scope coverage
  results are durable and queryable before any collection starts.
- Added native SQS and SNS adapters (schema 009/010), bringing native coverage
  to 35 kinds. SQS maps queues, attributes and redrive `DELIVERS_TO` edges; SNS
  maps topics and account-wide subscriptions with `SUBSCRIBES_TO` edges,
  deterministic pending-subscription identities and presence-only policy
  reporting (no policy bodies).
- Added `cmdb-bench`: a streaming deterministic fleet generator and independent
  oracle. Verified 1M and 10M resources (10,000 accounts, skewed distribution,
  duplicates, reorders, deletions, empty regions, global services) with peak
  RSS under 5 MB and byte-deterministic output.
- Added `cmdb-view`, a bounded read model over exported SQLite snapshots, and
  `cmdb-api`, a read-only loopback HTTP API carrying the snapshot fingerprint
  with typed errors, bounded pagination, graph neighborhoods, LOD and coverage.
- Added `cmdb-ui`, a Makepad 2.0 viewer over the synthetic fixture: filter rail,
  virtualized table, detail rail and a deterministic account→region→kind→resource
  DAG with resource overlay edges, hover/selection, wheel zoom and drag pan.
- Added executable performance/failure contracts and the CMDB architecture
  baseline, including explicit boundaries: no auth/write API, no event
  ingestion, no tombstones/retention and no measured fleet capacity yet.
- Repaired the operation-coverage callsite scanner so it resolves clients
  imported through `use super::*` and reports verified exercised operations
  against the reviewed matrix instead of publishing an unavailable count.
- Replaced the Python emulator snapshot/assertion entry point with the Rust
  `inventory-harness` operator binary: pinned emulator installation, snapshot
  freezing, S3 and Cloud Control seeding and Cloud Control assertions.
- Replaced the Python release packager with `cargo run -p catalog-tool --
  package --destination <zip>`. Selection rules, manifest bytes and payloads
  match the retired tool; entry order is sorted and every entry carries a fixed
  1980-01-01 timestamp, so archives over an unchanged tree are byte-identical.
- Added six ignored live emulator tests that own a fresh emulator, seed through
  the real SDKs and scan with the shipped collector: native services, reviewed
  Cloud Control gaps, the universal partition, combined native+universal,
  multi-page Cloud Control listing, and an exported seeded bucket.
- Added `catalog/cloudcontrol-universal.json`: 1,881 schema types partitioned
  into 1,291 list-capable, 410 parent-required and 180 without list handlers,
  with digest and pinned source archive identity.

- Added `cmdb-api`, a read-only loopback HTTP query API over one parsed
  snapshot: typed GET endpoints for meta, filtered/paginated resources, bounded
  graph neighborhoods, LOD and coverage, all carrying the snapshot fingerprint,
  with typed JSON errors and no write path or authentication. Non-loopback
  binds are refused until an auth story exists.

Current development-tree evidence: 781 Rust tests pass with zero failures (15
ignored by default). The native live emulator phase passes with 263 attempts,
168 resources and 344 edges, including SQS/SNS/KMS/Step Functions assertions.
The prepared live baseline passes 2,664 checks with zero failures, including the
Config freeze and parent pagination gates. All-target clippy is clean, formatting
and the 300-line source/25-line function gates pass. Cloud Control listing now
sends the page-size budget as `max_results`; the live pagination gate proves a
three-page list chain at page size two. Operation-coverage `--check` verifies 76
exercised operations plus 10 reviewed additive operations and refuses to publish
when the live scan cannot resolve reviewed callsites.

## 0.6.0 — Parent-scoped collection and exact graph verification

- Added 21 reviewed parent-scoped Cloud Control types across API Gateway
  REST/V2, AppConfig, EKS, ELBv2 and CloudWatch Logs. The runtime catalog now
  has 418 types; the original 397-type descriptor file remains byte-identical.
  Child-only selections add required roots automatically and share discovery.
- Added bounded parent-model admission, independent pagination scopes,
  duplicate-model rejection, ownership validation and explicit incomplete
  prerequisite outcomes. Missing or failed parents cannot produce a false
  complete-empty child scan. Every confirmed child retains parent/model
  provenance and an evidence-bearing `CHILD_OF` edge.
- Added 169 provider columns and 20 property relationship rules. SQLite
  schema 6 adds 21 views transactionally, bringing the provider view count to
  418 and column count to 4,273. Name conflicts roll back the entire upgrade.
- Updated offline planning to distinguish known root requests from unknown
  parent-dependent request counts and expose the actual nested traversal
  bounds. Coverage overlays the parent selection on the retained 1,881-schema
  universe while keeping historical evidence separately identified.
- Added parent property tests, adversarial prerequisite/cancellation/budget
  cases, and a production SDK HTTP test for real 429 retry accounting,
  serialized models and cursor isolation. Added a fifth guarded emulator
  snapshot with 161 children, 161 parent edges and literal architecture
  expectations, including 101 EKS add-ons to force pagination.
- Extended SQLite-to-JSONL-to-FalkorDB verification to every projected resource,
  edge and scan-marker property, with exact property/label counts and strict
  JSON semantics. Added a checksum-pinned native Redis/FalkorDB installer with
  bounded downloads/extraction and owned build-process cleanup.
- Tightened snapshot provenance to bind all five commands, current source,
  both catalogs, both semantic fixtures, the executing binary and distinct
  persisted scans. Current and historical evidence cannot be interchanged.
- Fixed nested supervisor cancellation so SIGTERM drains owned process groups
  and closes receipts, including when the original group leader already exited.
  Added bounded health-failure diagnostics and real-process regressions without
  relaxing fixture health timeouts.
- Repaired mutation acceptance: the final aggregate itself requires all ten
  named live cases and two clean baselines, rejects mixed fixture identities,
  includes embedded dependency SQL/data in source closure, and keeps compiler
  infrastructure errors and test deadlines unresolved. These acceptance fixes
  do not establish a complete 95% mutation result.

The completed local run passed 530 Rust tests with zero failures or ignored
tests, 2,522 emulator assertions, and all five exact JSONL/FalkorDB projections.
Ten portable JSON/JSONL files are retained. Fresh enumeration found 2,959 mutation
candidates but executed none. See `docs/VERIFICATION.md` for receipts and gaps.

## 0.5.0 — Broader coverage, portable export and automatic graph delivery

- Expanded the reviewed regional Cloud Control selection from 200 to 397 types
  across 76 service families. Added a complete 1,881-schema capability matrix
  with explicit read/list, required-parent and unreviewed-scope exclusions, and
  a bounded offline `coverage` command.
- Expanded schema-derived architecture extraction to 945 relationship rules
  across 244 selected types. Rules validate typed ARN service/resource forms,
  preserve returned owners, label short-ID owner assumptions, and derive edges
  only after sensitive-property redaction. Corrected accountless EBS snapshot
  ARN identities and distinguished VPC Lattice target groups from ELB targets.
- Added 197 provider-column views through transactional SQLite schema 5. The
  original 200-view migration and original columns remain unchanged; all 397
  descriptors expose 4,104 provider columns. Existing observations, fingerprints
  and caller-owned objects survive the additive upgrade.
- Added streaming portable JSON and JSONL artifacts with versioned envelopes,
  normalized or all-observation modes, source/page provenance, scope outcomes,
  failures, bounded records/bytes and content hashes. Finalized partial scans
  require explicit opt-in; running scans cannot be exported.
- Connected `scan` and durable job intent to optional file/FalkorDB delivery.
  Added `export` and `deliver` commands. Delivery can resume a finalized SQLite
  scan without reissuing AWS calls, preserves completed artifact receipts across
  later failures, and records collection and delivery outcomes independently.
- Added bounded FalkorDB reconnect/retry using the shared executor. Only
  exporter-recognized idempotent plans receive automatic ambiguous-write replay;
  acknowledgements follow server success and bind the complete immutable plan.
  Cancellation and timeout paths own and drain file/checkpoint resources.
- Added independent architecture fixtures, redaction/property cases, migration
  regressions, a 10,001-resource export test and actual-server fault injection
  that drops a reply after a committed graph write. Expanded the live CLI case
  to collect, export, project and checkpoint-resume in one workflow.
- Bound coverage refresh to the executing binary's embedded catalog, exact
  collector/fixture hashes, command receipts, finalized scan and persisted
  detailed reads. A transport success without a usable observation cannot be
  promoted to resource coverage.

The historical verification receipts are in `docs/VERIFICATION_V05.md`. The SDK
dependency catalog remains 437 optional, checksum-pinned service crates. This
increment does not claim every AWS resource, live-account validation, a new
throughput result or a completed project-wide 95% mutation score.

## 0.4.0 — Global inventory, durable jobs and snapshot analysis

- Added S3 bucket metadata, IAM roles/instance profiles/customer-managed policy
  metadata, Route 53 hosted zones/records and CloudFront distributions. Seven
  new native kinds run outside the regional loop and retain bounded child
  failures, composite cursors and architecture references.
- Added Organizations account/hierarchy discovery, optional existing-role
  account fan-out and existing Config/Resource Explorer source discovery.
  Incomplete account listings stop automatic target selection.
- Added a SQLx/SQLite job queue with transactional claims, immutable lease
  fencing, bounded attempt history, idempotent submission, durable cancellation,
  expired-worker recovery and explicit retry of terminal unsuccessful jobs.
- Added worker supervision with bounded cancellation drainage, heartbeat
  renewal during drainage and bounded per-job/output summaries.
- Added streaming snapshot comparison, bounded neighborhood/path exploration,
  source-aware findings and synthetic regression fixtures.
- Added additive schema4 migration with seven native and three expanded SQL
  views, preserving old observations, custom objects and page fingerprints.
- Added a pinned, signed AWS Emulator integration harness, a complete 72-operation
  matrix plus the SDK AssumeRole boundary, populated fixtures, bounded request
  recording and independent direct-observation assertions. All 200 Cloud Control
  types are attempted; upstream gaps remain explicit.
- Added separate official SDK wire-replay tests for Resource Explorer timestamp
  conformance, terminal handling for HTTP 501, and safer diagnostics when the SDK
  rejects an HTTP success response.
- Added bounded Proptest suites for core identities/serialization, API retry and
  pagination histories, SQLx page transactions, durable job leases, discovery
  configuration and actual CLI behavior. Minimized regressions are retained.
- Fixed the public AssumeRole configuration helper to preserve the explicit
  one-attempt retry setting through SDK service defaults, including DynamoDB.
  Properties check all 15 implemented service configurations and region clones.
- Added exact emulator snapshot oracles for native, supplemental, Cloud Control
  and assumed-role collection, plus actual CLI/worker/graph round trips. The
  mutation fixture freezes its own synthetic Config recorder before collecting
  the oracle, preserving provider timestamps in comparisons.
- Added a pinned, complete-candidate mutation runner and strict 95% aggregate
  gate, with source/fixture receipts, required live baselines, bounded owned
  processes and explicit unresolved infrastructure failures. Core is complete
  at 56/56 caught; the whole-project gate is still incomplete.

This increment retains the 200 reviewed Cloud Control descriptors and all 437
pinned SDKs. Configured coverage, local validation and live AWS validation remain
separate; no live AWS collection or higher throughput claim accompanies it.

## 0.3.0 — Schema-backed breadth and collection planning

- Added a shared official Cloud Control SDK collector and 200 reviewed regional
  list/read descriptors, with bounded enrichment, provider identity provenance,
  sensitive-field redaction and explicit per-resource failures.
- Added an offline collection plan and 200 generated provider-column SQLite
  views in the aws_cc_* namespace. The view generator is Rust.
- Added ACTIVE ECS task-definition revisions, role/filesystem references, and
  ECS service primary/alternate target-group relationships.
- Added transactional schema3 migration, preserving observations and page
  fingerprints while introducing Direct4 > CloudControl3 > Config2 > Explorer1.
  Unknown added managed-table columns reject the migration before mutation.
- Added a Rust source auditor for all 437 pinned SDKs: 19,617 operation modules,
  3,779 paginators and 407 waiters, with archive checksums and reproducible output.
- Audited native retry hooks and current Rust workflow options. Retained one
  request retry owner so every explicit inventory SDK attempt re-enters the
  existing quota scheduler; no nested retry or workflow server was introduced.

The 200-type catalog is configured schema-backed capability. Live AWS regional
coverage and throughput remain unvalidated. See the current verification report.

## 0.2.0 — Architecture inventory expansion

- Added native route-table, internet-gateway, NAT-gateway, VPC-endpoint,
  network-ACL and VPC-peering collectors.
- Added native ELBv2 load-balancer, listener, ALB-rule, target-group and
  target-health membership collectors. Target groups are enumerated regionally,
  including groups with no attached load balancer.
- Added fractional request policies and shared server-directed cooldowns.
  Existing integer policy JSON and Rust constructors remain supported.
- Added a transactional SQLite schema-v2 migration, typed views, exact-snapshot
  route rows, endpoint-observation flags and load-balancer type aliases.
- Added the bounded `architecture` report command, source-aware coverage and
  explicit per-panel truncation.
- Aligned Config and Resource Explorer names for six network and four ELBv2
  resource kinds. Explorer uses ARN evidence to distinguish Classic ELB;
  supplemental records retain their source-specific detail and enrichment limits.
- Added a synthetic architecture fixture, query examples, collector design
  contracts and regression tests.

The native inventory now has 24 resource/component kinds. SDK availability is
still the pinned catalog of 437 official service SDK crates, not 437 implemented
collectors. Live AWS coverage and throughput have not been established.

## 0.1.0 — Initial working foundation

- Added 13 native resource kinds, Config aggregator and Resource Explorer input,
  a bounded request executor, SQLx/SQLite storage and replayable FalkorDB export.
- Added atomic page/checkpoint commits, immutable finalized snapshots, explicit
  partial outcomes, bounded SQL queries, a synthetic demo and SDK provenance.
- Retained original benchmark observations, including an unfavorable repeat;
  no stable throughput improvement was established.

See `docs/VERIFICATION.md` for observed test/build results and the remaining
validation boundaries for the current source tree.
