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
Control types, SQLite schema 13, FalkorDB projection, emulator harness). Toward
the plan, `scripts/verify.sh` no longer uses Python,
`inventory_engine::store::SCHEMA_VERSION` is the single schema-version source
and both packagers exclude `.local`. The AWS API models are pinned as a git
submodule at `.local/aws-models` (V23, ROADMAP M1.1). The binary record module
(CBOR through the `minicbor` library, V34, ROADMAP M3.1a) was being built, not
landed. Nothing else of
the new architecture exists yet; [ROADMAP.md](ROADMAP.md) records what has
landed since.

Many historical and baseline documents link into `artifacts/`. That directory
is ignored by git and is not in a clone, so those links resolve only on a
machine that kept the files.

## Plan

| Document | What it covers |
| --- | --- |
| [VISION.md](VISION.md) | The product, owner decisions V01 to V34, the target architecture, design rules, failure handling, proof, and how work lands. |
| [ROADMAP.md](ROADMAP.md) | The build plan, packages M0 to M10, with needs, exit evidence and status. |
| [../NEXT_AGENT_PROMPT.txt](../NEXT_AGENT_PROMPT.txt) | The standing brief for the next agent: hard rules, state of the tree, how to run the loop. |
| [cmdb-spec/](cmdb-spec/README.md) | Superseded detailed draft of the CMDB requirements. Still the reference for behavior VISION does not restate; VISION replaces it where they differ. Frozen. |
| [CMDB_ARCHITECTURE.md](CMDB_ARCHITECTURE.md) | Superseded draft of a multi-tenant continuous service. Passages that contradict a decision are marked with its ID; the "current anchor" columns still describe the tree. Replaced at M10.3. |
| [CMDB_PERFORMANCE_CONTRACT.md](CMDB_PERFORMANCE_CONTRACT.md) | Superseded draft of budgets and hypotheses, marked where VISION changes them. Its definitions and comparison protocol are kept (V19, M4.4, M5.6, M10.2). |

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
| [HARNESS.md](HARNESS.md) | `inventory-harness`: commands and the port of Python tooling to Rust. | Gains the data-driven seeder (M2.9) and the fault proxy (M4.3). The "Python kept until byte parity" rule is superseded by V21 and M2.10. |
| [LOCAL_SETUP.md](LOCAL_SETUP.md) | The local emulator and FalkorDB stack and its wrapper script. | Python prerequisites go in M2.10; the all-SDK build step goes with the SDK catalog in M1.7. |
| [PROPERTY_TESTING.md](PROPERTY_TESTING.md) | Property tests of the executor, store, jobs and CLI. | Kept. The emulator oracle section depends on Python scripts (M2.10) and Cloud Control descriptors (M9.5). |
| [SOURCES.md](SOURCES.md) | Primary sources, attribution and licensing. | Kept. Does not yet list the pinned AWS API models; its Cloud Control and SDK references describe the baseline. |

## Baseline being replaced

| Document | What it covers | Replaced or deleted by |
| --- | --- | --- |
| [../README.md](../README.md) | Project overview. Its opening states the direction; its sections document the baseline CLI. | Rewritten as packages land; M6.4 adds the walkthrough to the DAG. |
| [../START_HERE.md](../START_HERE.md) | Build-and-verify handoff for the baseline, version 0.6.0. | New work starts from the roadmap. |
| [../CONTRACT.md](../CONTRACT.md) | Implementation contract of the baseline crates. | VISION sections 4 and 10 are the rules for new work. |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Architecture of the version 0.6 collector as built. Passages that contradict a decision are marked. | One document describing what exists, at M10.3. |
| [COLLECTORS.md](COLLECTORS.md) | The typed SDK adapters and the Cloud Control path: scopes, fields, relationships. | M5 per service; M9.5 for Cloud Control and the old `scan` path. |
| [NETWORKING.md](NETWORKING.md) | The `network` adapter (VPC routing, gateways, endpoints, ACLs, peering). | M5.1. |
| [GLOBAL_INVENTORY.md](GLOBAL_INVENTORY.md) | The `global_inventory` adapter (S3, IAM, Route 53, CloudFront). | M5.2, M5.3. |
| [LOAD_BALANCING.md](LOAD_BALANCING.md) | The `load_balancing` adapter (ELBv2). | M5.5. |
| [TASK_DEFINITIONS.md](TASK_DEFINITIONS.md) | The `ecs` and `task_definitions` adapters. | M5.5. |
| [DISCOVERY.md](DISCOVERY.md) | Organizations, Config aggregator and Resource Explorer discovery, and role assumption. | M5.2 for the calls. Accounts come from local profiles in the first version (V09); Organizations discovery is after 1.0. |
| [AWS_CALL_MATRIX.md](AWS_CALL_MATRIX.md) | Every typed SDK operation the baseline calls, by callsite. | M5 and M9.5; M2.6 uses it as the differential-proof list. |
| [OPERATION_COVERAGE.md](OPERATION_COVERAGE.md) | Generated report of exercised operations against the SDK operation inventory. Generated and byte-checked by `inventory-harness coverage operations`, so it carries no status line. | M1.7 deletes the inventory it is generated from. |
| [AWS_PROPERTY_TESTS.md](AWS_PROPERTY_TESTS.md) | Property tests and oracles of the typed adapters and their SDK retry configuration. | M5 and M9.5, with the adapters. |
| [SDK_CATALOG.md](SDK_CATALOG.md) | The catalog of pinned typed SDK crates and its Python generator. | M1.7; the generator in M2.10. |
| [SDK_RETRY_AUDIT.md](SDK_RETRY_AUDIT.md) | Source audit of the SDK crates' retry behavior. Its conclusion, one retry owner, is kept. | M1.7. |
| [CLOUDCONTROL_COVERAGE.md](CLOUDCONTROL_COVERAGE.md) | The Cloud Control catalog, the full schema partition and selection rules. | M9.5. The type list survives as a checklist (M1.5, M9.4). |
| [COLLECTION_PLAN.md](COLLECTION_PLAN.md) | Plan and evidence for the 421 Cloud Control types and their relationship rules. | M9.5. The reviewed rules are carried over in M5. |
| [PARENT_COLLECTIONS.md](PARENT_COLLECTIONS.md) | Parent-dependent Cloud Control collection. | M9.5; bindings come from the models and overlay (M1.4). |
| [UNIVERSAL_COVERAGE.md](UNIVERSAL_COVERAGE.md) | Listing every list-capable type through Cloud Control, and runtime reference edges. | M9.5; complete coverage comes from M9. |
| [COLLECTION_FIXTURES.md](COLLECTION_FIXTURES.md) | Synthetic JSONL event fixtures for the `ingest` path. | M9.5 with schema 13; seed files in M2.9. |
| [ANALYSIS.md](ANALYSIS.md) | `diff`, `neighbors`, `path` and `findings` over scan snapshots. | No snapshots or history in the first version (V22); traversal moves to FalkorDB (M6.2); schema 13 goes in M9.5. |
| [ARCHITECTURE_QUERIES.md](ARCHITECTURE_QUERIES.md) | Worked example of the `architecture` report and snapshot SQL. | M9.5 with schema 13; the DAG on real data is M6. |
| [LOCAL_WALKTHROUGH.md](LOCAL_WALKTHROUGH.md) | Walkthrough of a baseline scan on one machine's local emulator. | M6.4. |
| [MUTATION_TESTING.md](MUTATION_TESTING.md) | The Python mutation gate and its 95% target. | M2.10 replaces it in Rust or drops it with a recorded reason. |

The HTTP read API has no document of its own. It is described in section 4 of
[CMDB_ARCHITECTURE.md](CMDB_ARCHITECTURE.md) and in the README, and
`crates/cmdb-api` is deleted in M6.5 (V32).

## Historical evidence

| Document | What it covers |
| --- | --- |
| [../CHANGELOG.md](../CHANGELOG.md) | Changes by release, newest first. |
| [VERIFICATION.md](VERIFICATION.md) | Verification record of the baseline, version 0.6.0, as recorded on 2026-11-13. The latest such record; not re-run since. |
| [VERIFICATION_V05.md](VERIFICATION_V05.md) | Verification record for version 0.5.0. |
| [VERIFICATION_V04.md](VERIFICATION_V04.md) | Verification record for version 0.4.0 and earlier. |
| [AWS_OPERATION_RECONCILIATION.md](AWS_OPERATION_RECONCILIATION.md) | Source and run reconciliation of AWS operations for v0.6. |
| [AWS_OPERATION_RECONCILIATION_V05.md](AWS_OPERATION_RECONCILIATION_V05.md) | The same for v0.5. |
| [CLOUDCONTROL_EVIDENCE.md](CLOUDCONTROL_EVIDENCE.md) | Generated rollup of the v0.5 Cloud Control emulator evidence. A harness test compares it byte for byte with its generator, so it carries no status line. |
| [MUTATION_READINESS_V06.md](MUTATION_READINESS_V06.md) | Audit of the mutation harness for v0.6. |
| [CMDB_BASELINE_2026-11-13.md](CMDB_BASELINE_2026-11-13.md) | Dated measurement of baseline SQLite ingestion and mock scheduling. |
| [DEPENDENCY_UPDATE.md](DEPENDENCY_UPDATE.md) | The 2026-11-12 dependency update and the local verification run made then. |
| [LOCAL_RESULTS.md](LOCAL_RESULTS.md) | Result of the initial local deployment, 2026-11-12. |
| [LOCAL_REVIEW.md](LOCAL_REVIEW.md) | Review of the initial local setup and Rust assurance, 2026-11-12. |
| [RUST_REVIEW.md](RUST_REVIEW.md) | Rust source and async ownership review of the imported archive, 2026-11-12. |
| `dependency-evidence/` | Raw receipts and reviews behind DEPENDENCY_UPDATE.md. Frozen. |
| `local-evidence/` | Checksums of the initial local evidence. Frozen. |
| `performance-evidence/` | Raw results behind CMDB_BASELINE_2026-11-13.md. Frozen. |
