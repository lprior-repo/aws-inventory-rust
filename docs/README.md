# Documentation index

Every document in this repository belongs to one of four classes. Each document
says which in a status line under its title.

- **Plan**: what is being built. [VISION.md](VISION.md) wins over every other
  document.
- **Kept and extended**: a subsystem of today's tree that the new architecture
  keeps. The row says what the plan changes.
- **Baseline being replaced**: how the collector works today, through Cloud
  Control and hand-written typed SDK adapters. Accurate for the current tree,
  not the way forward. The row names the roadmap package that replaces or
  deletes it.
- **Historical evidence**: dated records of what was run and found. Not
  updated.

State on 2026-11-13: the baseline collector works as the baseline documents
describe (39 native kinds through typed SDK adapters, 421 reviewed Cloud
Control types, SQLite schema 13, FalkorDB projection, emulator harness).
The current maintainer gate is `moon ci`; it uses Rust and shell, not Python.
`inventory_engine::store::SCHEMA_VERSION` is the single schema-version source,
and both packagers exclude `.local`. The pinned API models, field registry,
binary record codec and hashing, binary catalog, endpoint evaluator, signing
and pooled transport foundations have landed. The new universal sweep,
binary persistence and live Makepad path remain unfinished.
[ROADMAP.md](ROADMAP.md) records the package evidence.

Many historical and baseline documents link into `artifacts/`. That directory
is ignored by git and is not in a clone, so those links resolve only on a
machine that kept the files.

## Plan

| Document | What it covers |
| --- | --- |
| [VISION.md](VISION.md) | The product, owner decisions V01 to V34, the target architecture, design rules, failure handling, proof, and how work lands. |
| [ROADMAP.md](ROADMAP.md) | The build plan, packages M0 to M10, with needs, exit evidence and status. |
| [cmdb-spec/](cmdb-spec/README.md) | Superseded detailed draft of the CMDB requirements. Still the reference for behavior VISION does not restate; VISION replaces it where they differ. Frozen. |
| [CMDB_ARCHITECTURE.md](CMDB_ARCHITECTURE.md) | Superseded draft of a multi-tenant continuous service. Passages that contradict a decision are marked with its ID; the "current anchor" columns still describe the tree. Replaced at M10.3. |
| [CMDB_PERFORMANCE_CONTRACT.md](CMDB_PERFORMANCE_CONTRACT.md) | Superseded draft of budgets and hypotheses, marked where VISION changes them. Its definitions and comparison protocol are kept (V19, M4.4, M5.6, M10.2). |
| [AUDIT_DECISIONS.md](AUDIT_DECISIONS.md) | Decisions and evidence for the external audit findings UI-07, UI-12, UI-15, UI-16, UI-17 and UI-18: trust boundary, frame-time measurement, legacy deletion gates, real-account timing, graph backend contract, licensing. |

## Kept and extended

| Document | What it covers | What the plan changes |
| --- | --- | --- |
| [SCHEDULER.md](SCHEDULER.md) | `ApiExecutor`: admission, pacing, retries, cooldowns, cancellation. | Buckets per account, region and quota group, learned rates stored and reused (M4.1); retry classes and budgets from the catalog (M4.2). |
| [RETRY_AND_WORKFLOWS.md](RETRY_AND_WORKFLOWS.md) | Decision note: one retry owner, a local durable queue, no workflow engine. | The wire engine has no internal retries at all (M2); the decision stands. |
| [STORAGE.md](STORAGE.md) | The SQLx/SQLite store: WAL, single writer, atomic idempotent page commits, views, query limits. | Kept: SQLite WAL, one writer, atomic idempotent commits. Replaced in M3: schema 13, JSON property blobs, observation history and upgrades give way to a generated schema 14 with binary records and typed columns, created fresh (V24), with hash reconciliation and no history (V22). SQL files 001 to 013 are deleted in M9.5. |
| [DURABLE_JOBS.md](DURABLE_JOBS.md) | Fenced whole-scan job queue: leases, attempts, cancellation, recovery. | A page-level durable work queue inside the store (M3.3) and bounded slices (M7.4) are added. |
| [GRAPH_DELIVERY.md](GRAPH_DELIVERY.md) | FalkorDB delivery: idempotent batches, acknowledgements, retry, checkpoints. | Whole-snapshot plan files are replaced by a pending-projection queue that sends changed and deleted keys (M6.1). |
| [DELIVERY.md](DELIVERY.md) | Export and FalkorDB delivery after a scan, or from an existing snapshot. | Mostly kept. Replaced: delivery tied to the old `scan` and to whole snapshots (M6.1, M9.5). |
| [EXPORT.md](EXPORT.md) | The JSON and JSONL export format, provenance modes, bounds and atomic publication. | Kept as the one explicit place JSON is produced (V03). Its snapshot and provenance model follows schema 13; no roadmap package redefines it yet. |
| [EMULATOR_TESTING.md](EMULATOR_TESTING.md) | The pinned AWS Emulator harness: installation, fixtures, what the evidence proves. | Kept as the proof environment. Replaced: Python seeding and recording (M2.9, M2.10) and the Cloud Control scans and assertions (M9.5). Its v0.4 and v0.5 sections are historical evidence. |
| [../scripts/local-stack.sh](../scripts/local-stack.sh) | The local FalkorDB and emulator stack wrapper. `help` lists its commands. | Python prerequisites go in M2.10; the obsolete typed-SDK inventory build step was removed in M1.7. |
| [PROPERTY_TESTING.md](PROPERTY_TESTING.md) | Property tests of the executor, store, jobs and CLI. | Kept. The emulator oracle section depends on Python scripts (M2.10) and Cloud Control descriptors (M9.5). |
| [SOURCES.md](SOURCES.md) | Primary sources, attribution and licensing. | Kept. Historical SDK source-audit evidence remains; the pinned AWS API model catalog is the current source of service and operation metadata. |

## Baseline being replaced

| Document | What it covers | Replaced or deleted by |
| --- | --- | --- |
| [../README.md](../README.md) | Project overview. Its opening states the direction; its sections document the baseline CLI. | Rewritten as packages land; M6.4 adds the walkthrough to the DAG. |
| [../START_HERE.md](../START_HERE.md) | Build-and-verify handoff for the baseline, version 0.6.0. | New work starts from the roadmap. |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Architecture of the version 0.6 collector as built. Passages that contradict a decision are marked. | One document describing what exists, at M10.3. |
| [COLLECTORS.md](COLLECTORS.md) | The typed SDK adapters and the Cloud Control path: scopes, fields, relationships. | M5 per service; M9.5 for Cloud Control and the old `scan` path. |
| [NETWORKING.md](NETWORKING.md) | The `network` adapter (VPC routing, gateways, endpoints, ACLs, peering). | M5.1. |
| [GLOBAL_INVENTORY.md](GLOBAL_INVENTORY.md) | The `global_inventory` adapter (S3, IAM, Route 53, CloudFront). | M5.2, M5.3. |
| [LOAD_BALANCING.md](LOAD_BALANCING.md) | The `load_balancing` adapter (ELBv2). | M5.5. |
| [TASK_DEFINITIONS.md](TASK_DEFINITIONS.md) | The `ecs` and `task_definitions` adapters. | M5.5. |
| [DISCOVERY.md](DISCOVERY.md) | Organizations, Config aggregator and Resource Explorer discovery, and role assumption. | M5.2 for the calls. Accounts come from local profiles in the first version (V09); Organizations discovery is after 1.0. |
| [AWS_CALL_MATRIX.md](AWS_CALL_MATRIX.md) | Every typed SDK operation the baseline calls, by callsite. | M5 and M9.5; M2.6 uses it as the differential-proof list. |
| [AWS_PROPERTY_TESTS.md](AWS_PROPERTY_TESTS.md) | Property tests and oracles of the typed adapters and their SDK retry configuration. | M5 and M9.5, with the adapters. |
| [SDK_RETRY_AUDIT.md](SDK_RETRY_AUDIT.md) | Historical source audit of SDK retry behavior. Its independent conclusion, one retry owner, remains relevant to the typed adapters. | Kept as historical evidence; M1.7 deletes the SDK-derived inventory, not the adapters. |
| [CLOUDCONTROL_COVERAGE.md](CLOUDCONTROL_COVERAGE.md) | The Cloud Control catalog, the full schema partition and selection rules. | M9.5. The type list survives as a checklist (M1.5, M9.4). |
| [COLLECTION_PLAN.md](COLLECTION_PLAN.md) | Plan and evidence for the 421 Cloud Control types and their relationship rules. | M9.5. The reviewed rules are carried over in M5. |
| [PARENT_COLLECTIONS.md](PARENT_COLLECTIONS.md) | Parent-dependent Cloud Control collection. | M9.5; bindings come from the models and overlay (M1.4). |
| [UNIVERSAL_COVERAGE.md](UNIVERSAL_COVERAGE.md) | Listing every list-capable type through Cloud Control, and runtime reference edges. | M9.5; complete coverage comes from M9. |
| [COLLECTION_FIXTURES.md](COLLECTION_FIXTURES.md) | Synthetic JSONL event fixtures for the `ingest` path. | M9.5 with schema 13; seed files in M2.9. |
| [ANALYSIS.md](ANALYSIS.md) | `diff`, `neighbors`, `path` and `findings` over scan snapshots. | No snapshots or history in the first version (V22); traversal moves to FalkorDB (M6.2); schema 13 goes in M9.5. |
| [ARCHITECTURE_QUERIES.md](ARCHITECTURE_QUERIES.md) | Worked example of the `architecture` report and snapshot SQL. | M9.5 with schema 13; the DAG on real data is M6. |
| [MUTATION_TESTING.md](MUTATION_TESTING.md) | The Python mutation gate and its 95% target. | M2.10 replaces it in Rust or drops it with a recorded reason. |

The obsolete HTTP read API (`crates/cmdb-api`) has been removed (M6.5, V32).
Section 4 of [CMDB_ARCHITECTURE.md](CMDB_ARCHITECTURE.md) retains its superseded
draft contract as history. `cmdb-view` still parses one JSON export and `cmdb-ui`
still renders a synthetic fixture; in-process SQLite/FalkorDB reads and UI
integration remain planned in M6.2/M6.3.

## Historical evidence

| Document | What it covers |
| --- | --- |
| [../CHANGELOG.md](../CHANGELOG.md) | Changes by release, newest first. |
| [VERIFICATION.md](VERIFICATION.md) | Verification record of the baseline, version 0.6.0, as recorded on 2026-11-13. The latest such record; not re-run since. |
| [AWS_OPERATION_RECONCILIATION.md](AWS_OPERATION_RECONCILIATION.md) | Source and run reconciliation of AWS operations for v0.6. |
| [CLOUDCONTROL_EVIDENCE.md](CLOUDCONTROL_EVIDENCE.md) | Generated rollup of the v0.5 Cloud Control emulator evidence. A harness test compares it byte for byte with its generator, so it carries no status line. |
| [RUST_REVIEW.md](RUST_REVIEW.md) | Rust source and async ownership review of the imported archive, 2026-11-12. |
