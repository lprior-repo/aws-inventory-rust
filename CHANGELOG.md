# Changes

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
