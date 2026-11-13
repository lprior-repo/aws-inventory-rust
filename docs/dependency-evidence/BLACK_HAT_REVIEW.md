# Whole-candidate Black Hat and asynchronous architecture review

**Bead**: N/A — user-authorized imported project, no Beads repository  
**State**: Refactored candidate; final canonical/live evidence pending  
**Reviewer**: black-hat-reviewer; source author self-review, not independent implementation acceptance  
**Source checkout**: /home/lewis/Documents/Codex/2026-11-12/let/aws-inventory-rs  
**Attempt**: 1 after whole-project decomposition  
**Snapshot UTC**: 2026-11-13T05:32:07.598712+00:00  
**HEAD**: `afbdcac1db3bab6a5be7eb71a253d4dceb07385c` plus uncommitted candidate  
**Rust source snapshot SHA256**: `9ede1dc5ac87f1732d136dfc8edd7b4a772c8e43c1a2f0769afb2df2fd35815e` (sorted relative path, NUL, file bytes, NUL)

## Gate Result

**STATUS: REJECTED**

This rejects full compliance with the explicitly invoked strict skills. The file-size requirement passes. No new executable correctness failure was reproduced by this read-only audit, and the final runtime suite was still running at the review boundary. A rejected strict review must not be presented as project approval or silently converted to owner-approved debt.

---

## PHASE 1: Contract & Bead Parity

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Every first-party Rust source file at most 300 lines | PASS | 451 `.rs` files under src/crates/tests, target trees excluded; maximum 294 at `crates/engine/tests/export.rs`. |
| Preserve CLI, stored representations and export formats | PENDING final execution | Argument field/Serde attributes were moved intact; SQL persistence uses the same column names and bound values. Inspected `store/persistence.rs:4`, `jobs/execution.rs:100`, `export/publication.rs:12`. |
| Atomic ingestion and exact replay | STATIC SUPPORT, runtime pending | `persist_page` commits resources/edges/fingerprint/checkpoint in one transaction. Prior fingerprint equality returns before requiring a still-running scan; replay remains intentional. |
| Domain lifecycle improvement | PASS scoped source inspection | `TerminalJobState` excludes Queued/Running (`jobs.rs:136`); `FinalScanStatus` excludes Running; `PersistedScan` parses a running/final timestamp combination into explicit variants (`store/lifecycle.rs:41`). |
| Preserve original CLI scenario coverage | STATIC PASS scoped names | All 77 original root CLI test function names retained; one typed-intake transition test added. Names do not prove assertion execution. |
| All SDK availability distinct from implemented collection | PASS source/document boundary | Catalog remains a set of optional public SDK re-exports; 437 crates compiling is not a claim of 437 implemented collectors or emulator APIs. |
| Final source evidence and immutable acceptance pin | PENDING | Candidate is uncommitted; final canonical/live logs must be complete and associated with the eventual commit. |
| Formal proof/source parity | No proof claim accepted | No verifier result is treated as implementation proof. No new proof harness was executed by this review. |

Public Rust API migration: `Completion.state` changes from `JobState` to `TerminalJobState`. Database and JSON state spellings remain unchanged, but downstream Rust clients constructing `Completion` must migrate. This is a real source API change, not wire-format equivalence.

---

## PHASE 2: Farley Engineering Rigor

AST inspection found **1584 syntactic function items**. **269 non-test functions exceed 25 physical lines**, and **24 non-test functions exceed five parameters**. Including tests gives 568 over-length functions and 25 over-parameter functions; test implementation shape alone is not used here to invalidate behavioral assertions.

| Function | Lines | Limit | Status |
|----------|-------|-------|--------|
| `supplemental/config/aliases.rs:20 config_aliases` | 194 | 25 | FAIL; mostly match-table data; move data into auditable constants/helpers rather than hiding it. |
| `src/plan.rs:13 create` | 191 | 25 | FAIL; selection, accounting and report construction remain one function. |
| `crates/aws/src/scan.rs:19 scan_inner` | 143 | 25 | FAIL; orchestration still spans several lifecycle steps. |
| `crates/aws/src/common/pagination.rs:22 paginate_inner` | 105 | 25 | FAIL; cursor transitions, validation, failure construction and forwarding are composed in one function. |

Full non-test function findings are listed below; the 300-line file gate does not establish the 25-line function gate. No source-size exceptions or include-fragment workaround were found in the reviewed decomposition.

---

## PHASE 3: Holzman Rust (The Big 6)

| Rule | Status |
|------|--------|
| Zero first-party `unsafe` blocks/functions/impls/traits | PASS static search under src and crates/*/src; not a dependency/native-code soundness claim. |
| Zero `.unwrap()` / `.expect()` | PASS static search under the same source roots. |
| Zero `panic!` / `todo!` / `unimplemented!` / `unreachable!` / `dbg!` | PASS static search under the same source roots. |
| Checked arithmetic | PENDING final strict Clippy evidence; inspected limiter counters and persistence conversions use checked/saturating operations. Static absence of a panic macro does not prove arithmetic safety. |
| Fixed bounds and backpressure | POSITIVE scoped inspection: maximum attempts, bounded channels/fanout, SQLite batches and page budgets exist; not a proof of every third-party allocation or poll duration. |
| Fully typed domain identifiers | FAIL strict policy: `inventory-core::Resource`, `Edge`, `CollectionScope` expose raw string IDs/regions/kinds and separate `validate()` methods (`crates/core/src/lib.rs:58`, `:95`, `:117`). |
| No behavioral boolean parameters | FAIL strict policy: examples include `src/delivery.rs:113 allow_partial`, `src/delivery.rs:226 cancelled`, `crates/engine/src/graph/delivery.rs:55 replay_safe`, `crates/aws/src/discovery.rs:171 global`. A `From<bool>` boundary conversion is distinct from a domain control-flag API. |

The raw `Resource`/`CollectionPage` types also serve as transport DTOs. Ingestion does validate them before creating `PreparedPage` (`store/prepared.rs:32`), so the primitive-type finding is **not evidence that invalid raw input is persisted**. It is an unmet strict type-level contract: the public model remains constructible in invalid states, and trusted identity types are not carried throughout the domain. Preserve transport shapes while introducing constrained domain values.

---

## PHASE 4: Ruthless Simplicity & DDD

| Check | Status |
|-------|--------|
| Scoped Option-based lifecycle repair | PASS: `PersistedScan` carries exactly the timestamps required by its variant; `Intake::Accepting/Stopped(StopReason)` replaces nullable worker stop strings. |
| Whole-domain illegal states unrepresentable | FAIL strict policy: public DTO/domain primitives and semantic boolean controls remain. |
| Optional diagnostics are not mistaken for lifecycle state | PASS: `Completion.result/error` are independent retained diagnostics; failed/cancelled attempts can legitimately preserve partial output. Their presence does not define the terminal state. |
| Pure core / async shell separation | POSITIVE: inventory-core has only serde/serde_json/thiserror normal dependencies and no runtime dependency. Storage/AWS adapters own I/O. |
| CUPID / legibility | PARTIAL: named modules improve review; broad parent `use super::*` coupling and long multi-stage functions remain. No invented framework was added to solve a local split. |
| Outcome taxonomy | FAIL strict async policy: worker timeout/drain/lease errors remain flattened through anyhow and JSON keys rather than one exhaustive typed boundary outcome (`src/jobs/worker/supervision.rs:28`). |

---

## PHASE 5: The Bitter Truth

The decomposition materially improves navigation and enforces the requested file ceiling without deleting tests. The typed terminal/scan/intake states are substantive improvements. It would still be false to label the whole codebase fully compliant with Black Hat, Holzmann, Scott DDD or the asynchronous skill: function/parameter constraints and several strict architectural obligations remain unmet.

No bare `tokio::spawn`, `spawn_local`, unbounded channel, or explicit task-abort loop was found in the source scan. The one tracked blocking page-preparation task uses `TaskTracker`, retains its join handle and distinguishes panic/cancellation (`store/scan.rs:42`). The caller-owned future trees, bounded API executor, transactional SQLite effects, no-clobber file publication and idempotent graph ACK order are positive patterns. This is scoped source inspection, not a scheduling proof.

---

## Findings (Ordered by Severity)

| Finding | Severity | File:Line | Status |
|---------|----------|-----------|--------|
| BH-001 Strict function/parameter gate remains violated | HIGH | `src/plan.rs:13`, full AST appendix | blocker |
| BH-002 Strict domain identity/control types incomplete | HIGH | `crates/core/src/lib.rs:58`, `src/delivery.rs:113` | blocker |
| ASYNC-001 Full mandatory async gate not established | HIGH | `src/jobs/worker/supervision.rs:28`, `store/scan.rs:95` | blocker |
| EV-001 Final acceptance evidence is pending/unpinned | HIGH | current uncommitted candidate | blocker until parent final run/pin |
| DOC-001 Historical review statements can be read as current | MEDIUM | `docs/RUST_REVIEW.md:249` | blocker unless dated/superseded |

### BH-001: File decomposition does not satisfy function decomposition

**Problem**: 269 non-test function spans exceed the invoked 25-line rule and 24 signatures exceed five parameters.

**Required Fix**: Extract named domain transitions, mapping/calculation steps and argument records. Keep generated/static catalog data as data. Do not replace a long function with numbered chunks or add lint/size waivers.

### BH-002: Boundary DTOs still double as primitive domain models

**Problem**: Validation protects ingestion, but invalid values remain publicly constructible and domain signatures still branch on booleans.

**Required Fix**: Keep Serde/CLI/persisted DTOs stable, parse once into private-field identifier/unit types and semantic modes, and make core workflows consume those types. Add invalid-boundary and legal-transition tests; do not claim global DDD compliance from the three repaired state types.

### ASYNC-001: Mandatory strict asynchronous obligations remain open

**Evidence**: 12 non-test async function spans exceed the async skill's 60-line rule (appendix). `Store::summary`, `scan_status` and `checkpoint` in `store/scan.rs:95` lack their own instrumentation, although upstream call spans can still exist. Worker supervision distinguishes several situations in messages/JSON, but returns `anyhow::Result<Value>`; it has no uniform typed Ok/domain-error/cancelled/timeout/panic outcome or complete drain certificate. No tokio-console or OTLP dependency/configuration was found. Virtual-time and rollback tests exist; no exhaustive scheduling proof is inferred.

**Required Fix**: Decompose the long async protocols, preserve existing cancellation/drain ordering, provide typed terminal outcomes and observable drain progress, complete the skill's span/observability requirements, and execute the required gate on the final candidate. No runtime migration is justified by this review. The local CLI being functional does not establish all of these stricter assurance claims.

### EV-001: Final tests must match final source

**Evidence**: Read-only inspection of `../work/refactor-tools/canonical-complete.log` showed source checks/Clippy completed and final test compilation underway. The all-SDK post-refactor log showed `Finished dev ... in 0.61s`; that reuses the previously compiled SDK artifacts and recompiles the catalog wrapper. Older `strict-complete.log` predates decomposition and cannot certify all final source changes. My focused CLI test log was interrupted while waiting for the parent build lock, and is not a pass.

**Required Fix**: Finish the canonical/live/strict lanes, retain actual exit status and raw output, commit the candidate, and associate final evidence with that exact pin. Do not relabel pre-refactor evidence as post-refactor verification.

### DOC-001: Old review capability claims need temporal context

**Evidence**: `docs/RUST_REVIEW.md:249` says the asynchronous reference bundle is missing and dependencies/property evidence is unavailable. In this session the installed structural-correctness and cancellation-safety references were readable, current dependency audit evidence exists, and actual property suites are present.

**Required Fix**: Preserve historical evidence with a clear date/superseded banner and add a current report. Do not silently rewrite the old result as if the earlier reviewer had executed current tools.

---

## Quality Gates

| Gate | Result | Evidence |
|------|--------|----------|
| Source size | PASS static computation | 451 Rust files, maximum 294; no target directories counted. |
| Function and parameter AST gate | FAIL | ast-grep `function_item`, `function_item > parameters > parameter`, `... > self_parameter`; counts above and appendix below. |
| First-party forbidden construct search | PASS static search | `rg` over src and crates/*/src produced no matches for unsafe blocks/functions/impls/traits, unwrap/expect or forbidden panic/debug macros. |
| Pure core runtime boundary | PASS source inspection | crates/core/Cargo.toml and crates/core/src/lib.rs. |
| Final cargo test / final live graph workflow | PENDING parent evidence | No build/test command was launched during this read-only review. |
| Final strict Clippy and complete async skill gate | PENDING/UNMET | Parent owns actual execution; source-level strict findings remain independently of test success. |
| SDK wrapper compile | Observed completed log, not all-service API runtime proof | ../work/dependency-update/all-sdks-refactored.log. |
| Performance / formal verification | NOT ESTABLISHED | No new performance or formal-verifier command was run by this review. |

Measurement scope: AST ranges run from the function declaration through the closing brace, including blank/comment lines and signatures/where-clauses, excluding preceding attributes/doc comments. Self receivers count as parameters. Non-test excludes tests-directory/name paths and functions inside inline modules named `tests` or ending `_tests`; examples remain non-test. Unexpanded macro-generated functions are not counted. These are physical-line counts, not claimed logical-SLOC metrics or a proof of every macro expansion.

---

## Verdict

**STATUS: REJECTED**

The whole candidate meets the requested 300-line file ceiling and contains useful typed state repairs. Final functional acceptance is still the parent's executable responsibility. Strict skill approval is rejected for the explicit, unwaived source/assurance findings above; no runtime failure was invented to justify that verdict.

### Summary

No approved waiver was assumed. This is a self-review of the shared candidate, not an independent approval.

---

## Required Repair Actions (if REJECTED)

1. **HIGH BH-001**: Repair every non-test function/parameter violation with semantic decomposition; retain the behavioral tests.
2. **HIGH BH-002**: Complete constrained identity/value types and semantic domain mode APIs while preserving transport/persistence formats.
3. **HIGH ASYNC-001**: Complete typed outcome, drain/instrumentation/observability and long-protocol requirements, then run the actual asynchronous gate.
4. **HIGH EV-001**: Complete and pin final canonical/live/strict evidence; preserve failures as historical failures.
5. **MEDIUM DOC-001**: Date and supersede stale review claims.

## Appendix A: All non-test function spans over 25 lines

| Location | Function | Physical lines |
|----------|----------|----------------|
| `crates/aws/examples/emulator_branches.rs:21` | `main` | 55 |
| `crates/aws/examples/emulator_branches.rs:87` | `probe` | 67 |
| `crates/aws/examples/emulator_branches.rs:155` | `probe_explorer` | 68 |
| `crates/aws/examples/emulator_branches.rs:224` | `probe_result` | 35 |
| `crates/aws/src/cloud_control/catalog_validation.rs:50` | `validate_selection` | 34 |
| `crates/aws/src/cloud_control/catalog_validation.rs:104` | `validate_descriptor` | 81 |
| `crates/aws/src/cloud_control/collection.rs:5` | `collect` | 50 |
| `crates/aws/src/cloud_control/collection.rs:78` | `remember_list_page` | 52 |
| `crates/aws/src/cloud_control/collection.rs:132` | `collect_type_with` | 56 |
| `crates/aws/src/cloud_control/collection.rs:190` | `enrich_with` | 58 |
| `crates/aws/src/cloud_control/identity.rs:37` | `resource_identity` | 49 |
| `crates/aws/src/cloud_control/identity.rs:87` | `tags` | 30 |
| `crates/aws/src/cloud_control/identity.rs:118` | `map_description` | 51 |
| `crates/aws/src/cloud_control/identity.rs:170` | `add_native_aliases` | 100 |
| `crates/aws/src/cloud_control/parents/catalog.rs:39` | `validate_catalog` | 34 |
| `crates/aws/src/cloud_control/parents/catalog.rs:74` | `validate_bindings` | 29 |
| `crates/aws/src/cloud_control/parents/children.rs:4` | `collect_child` | 55 |
| `crates/aws/src/cloud_control/parents/children.rs:60` | `collect_child_item` | 48 |
| `crates/aws/src/cloud_control/parents/children.rs:109` | `attach_parent` | 46 |
| `crates/aws/src/cloud_control/parents/children.rs:156` | `emit_failed_read` | 35 |
| `crates/aws/src/cloud_control/parents/discovery.rs:4` | `collect_group_with` | 36 |
| `crates/aws/src/cloud_control/parents/discovery.rs:41` | `collect_parent_pages` | 51 |
| `crates/aws/src/cloud_control/parents/discovery.rs:101` | `collect_parent_item` | 83 |
| `crates/aws/src/cloud_control/parents/grouping.rs:4` | `groups` | 29 |
| `crates/aws/src/cloud_control/parents/grouping.rs:34` | `parent_context` | 39 |
| `crates/aws/src/cloud_control/property_paths.rs:37` | `expand_pointer` | 40 |
| `crates/aws/src/cloud_control/property_paths.rs:78` | `common_redactions` | 61 |
| `crates/aws/src/cloud_control/property_paths.rs:162` | `sanitized_properties` | 37 |
| `crates/aws/src/cloud_control/relationships.rs:4` | `reference_arn_matches` | 47 |
| `crates/aws/src/cloud_control/relationships.rs:67` | `map_relationships` | 75 |
| `crates/aws/src/common.rs:160` | `advance` | 38 |
| `crates/aws/src/common/error_classification.rs:4` | `classify_sdk_error` | 82 |
| `crates/aws/src/common/pagination.rs:22` | `paginate_inner` | 105 |
| `crates/aws/src/discovery/execution.rs:40` | `discover_with_client` | 32 |
| `crates/aws/src/discovery/organizations.rs:34` | `collect` | 30 |
| `crates/aws/src/discovery/organizations/hierarchy.rs:5` | `collect_hierarchy` | 62 |
| `crates/aws/src/discovery/organizations/listing.rs:5` | `list_accounts` | 29 |
| `crates/aws/src/discovery/organizations/listing.rs:36` | `list_roots` | 29 |
| `crates/aws/src/discovery/organizations/listing.rs:67` | `list_units` | 32 |
| `crates/aws/src/discovery/organizations/listing.rs:101` | `list_members` | 33 |
| `crates/aws/src/discovery/organizations/mapping.rs:4` | `map_account` | 29 |
| `crates/aws/src/discovery/pages.rs:82` | `accept` | 36 |
| `crates/aws/src/discovery/pages.rs:138` | `advance` | 32 |
| `crates/aws/src/discovery/pages.rs:176` | `collect` | 47 |
| `crates/aws/src/discovery/roles.rs:25` | `assume_role_config` | 45 |
| `crates/aws/src/discovery/sources/configuration.rs:5` | `collect_config` | 45 |
| `crates/aws/src/discovery/sources/configuration.rs:51` | `map_aggregator` | 36 |
| `crates/aws/src/discovery/sources/configuration.rs:88` | `map_aggregation_scopes` | 29 |
| `crates/aws/src/discovery/sources/explorer.rs:5` | `collect_explorer` | 32 |
| `crates/aws/src/discovery/sources/explorer.rs:77` | `list_explorer_views` | 37 |
| `crates/aws/src/discovery/sources/explorer.rs:116` | `select_explorer_view` | 30 |
| `crates/aws/src/discovery/sources/views.rs:69` | `map_view` | 33 |
| `crates/aws/src/discovery/sources/views.rs:112` | `validate_scoped_arn` | 38 |
| `crates/aws/src/dynamo.rs:16` | `collect` | 38 |
| `crates/aws/src/dynamo.rs:56` | `describe` | 46 |
| `crates/aws/src/ec2/collection.rs:5` | `instances` | 38 |
| `crates/aws/src/ec2/mapping.rs:16` | `resource` | 37 |
| `crates/aws/src/ec2/mapping.rs:54` | `link` | 28 |
| `crates/aws/src/ec2/mapping.rs:83` | `map_instance` | 52 |
| `crates/aws/src/ecs.rs:19` | `collect` | 39 |
| `crates/aws/src/ecs/clusters.rs:17` | `describe_clusters` | 41 |
| `crates/aws/src/ecs/service_mapping.rs:4` | `map_service` | 138 |
| `crates/aws/src/ecs/service_mapping.rs:147` | `merge_reference_edge` | 33 |
| `crates/aws/src/ecs/service_mapping.rs:181` | `target_group_link` | 43 |
| `crates/aws/src/ecs/services.rs:5` | `services` | 40 |
| `crates/aws/src/ecs/services.rs:47` | `describe_services` | 43 |
| `crates/aws/src/ecs/tasks.rs:5` | `tasks` | 78 |
| `crates/aws/src/ecs/tasks.rs:84` | `map_task` | 44 |
| `crates/aws/src/eks.rs:19` | `collect` | 39 |
| `crates/aws/src/eks.rs:60` | `cluster` | 32 |
| `crates/aws/src/eks.rs:93` | `map_cluster` | 52 |
| `crates/aws/src/eks.rs:147` | `nodegroups` | 37 |
| `crates/aws/src/eks.rs:186` | `nodegroup` | 40 |
| `crates/aws/src/eks.rs:227` | `map_nodegroup` | 41 |
| `crates/aws/src/global_inventory/cloudfront.rs:28` | `collect` | 53 |
| `crates/aws/src/global_inventory/cloudfront.rs:105` | `distribution_tags` | 40 |
| `crates/aws/src/global_inventory/cloudfront/mapping.rs:4` | `map_distribution` | 89 |
| `crates/aws/src/global_inventory/cloudfront/mapping.rs:94` | `distribution_resource` | 28 |
| `crates/aws/src/global_inventory/cloudfront/references.rs:4` | `distribution_edges` | 45 |
| `crates/aws/src/global_inventory/cloudfront/references.rs:50` | `behavior_edges` | 31 |
| `crates/aws/src/global_inventory/iam/mapping.rs:30` | `map_role` | 36 |
| `crates/aws/src/global_inventory/iam/mapping.rs:67` | `map_profile` | 28 |
| `crates/aws/src/global_inventory/iam/policies.rs:5` | `policies` | 44 |
| `crates/aws/src/global_inventory/iam/policies.rs:51` | `policy_details` | 36 |
| `crates/aws/src/global_inventory/iam/profiles.rs:5` | `profiles` | 39 |
| `crates/aws/src/global_inventory/iam/profiles.rs:46` | `profile_details` | 35 |
| `crates/aws/src/global_inventory/iam/roles.rs:5` | `roles` | 36 |
| `crates/aws/src/global_inventory/iam/roles.rs:43` | `role_details` | 39 |
| `crates/aws/src/global_inventory/iam/roles.rs:84` | `attached_policies` | 60 |
| `crates/aws/src/global_inventory/iam/trust.rs:4` | `trust_summary` | 30 |
| `crates/aws/src/global_inventory/iam/trust.rs:57` | `trust_statement` | 37 |
| `crates/aws/src/global_inventory/iam/trust.rs:95` | `trust_edges` | 34 |
| `crates/aws/src/global_inventory/observation.rs:33` | `observed` | 29 |
| `crates/aws/src/global_inventory/observation.rs:64` | `section` | 31 |
| `crates/aws/src/global_inventory/observation.rs:105` | `edge_set` | 36 |
| `crates/aws/src/global_inventory/pagination.rs:24` | `record` | 31 |
| `crates/aws/src/global_inventory/route53.rs:35` | `collect` | 41 |
| `crates/aws/src/global_inventory/route53/mapping.rs:4` | `record_identity` | 30 |
| `crates/aws/src/global_inventory/route53/mapping.rs:58` | `map_record` | 46 |
| `crates/aws/src/global_inventory/route53/mapping.rs:105` | `record_target` | 37 |
| `crates/aws/src/global_inventory/route53/records.rs:5` | `records` | 58 |
| `crates/aws/src/global_inventory/route53/zones.rs:5` | `collect_zone` | 49 |
| `crates/aws/src/global_inventory/route53/zones.rs:56` | `zone_tags` | 49 |
| `crates/aws/src/global_inventory/route53/zones.rs:120` | `map_zone` | 44 |
| `crates/aws/src/global_inventory/s3.rs:42` | `collect` | 45 |
| `crates/aws/src/global_inventory/s3/configuration.rs:31` | `replication` | 35 |
| `crates/aws/src/global_inventory/s3/configuration.rs:68` | `website` | 29 |
| `crates/aws/src/global_inventory/s3/configuration.rs:99` | `ownership` | 29 |
| `crates/aws/src/global_inventory/s3/configuration.rs:130` | `notifications` | 54 |
| `crates/aws/src/global_inventory/s3/details.rs:23` | `read_section` | 43 |
| `crates/aws/src/global_inventory/s3/details.rs:86` | `read_tags` | 45 |
| `crates/aws/src/global_inventory/s3/location.rs:41` | `resolve_location` | 40 |
| `crates/aws/src/global_inventory/s3/mapping.rs:4` | `map_bucket` | 64 |
| `crates/aws/src/global_inventory/s3/mapping.rs:69` | `bucket_edges` | 72 |
| `crates/aws/src/global_inventory/s3/security.rs:27` | `encryption` | 34 |
| `crates/aws/src/global_inventory/s3/security.rs:63` | `public_access_block` | 29 |
| `crates/aws/src/global_inventory/s3/security.rs:94` | `policy_status` | 26 |
| `crates/aws/src/lambda.rs:20` | `collect` | 34 |
| `crates/aws/src/lambda.rs:65` | `map_function` | 44 |
| `crates/aws/src/lib.rs:79` | `validate` | 85 |
| `crates/aws/src/load_balancing/collection.rs:23` | `load_balancers` | 36 |
| `crates/aws/src/load_balancing/collection.rs:61` | `listeners` | 45 |
| `crates/aws/src/load_balancing/collection.rs:108` | `rules` | 28 |
| `crates/aws/src/load_balancing/collection.rs:138` | `target_groups` | 48 |
| `crates/aws/src/load_balancing/collection.rs:190` | `target_health_with` | 34 |
| `crates/aws/src/load_balancing/pages.rs:6` | `map_page` | 51 |
| `crates/aws/src/load_balancing/pages.rs:67` | `map_load_balancer` | 58 |
| `crates/aws/src/load_balancing/pages.rs:139` | `map_listener` | 38 |
| `crates/aws/src/load_balancing/rules.rs:4` | `map_rule` | 41 |
| `crates/aws/src/load_balancing/rules.rs:46` | `map_actions` | 49 |
| `crates/aws/src/load_balancing/rules.rs:96` | `map_condition` | 41 |
| `crates/aws/src/load_balancing/targets.rs:4` | `map_target_group` | 45 |
| `crates/aws/src/load_balancing/targets.rs:50` | `map_target_health` | 50 |
| `crates/aws/src/load_balancing/targets.rs:101` | `target_reference` | 59 |
| `crates/aws/src/load_balancing/targets.rs:161` | `returned_target_arn` | 34 |
| `crates/aws/src/network/access.rs:4` | `map_network_acl` | 44 |
| `crates/aws/src/network/access.rs:62` | `map_peering` | 53 |
| `crates/aws/src/network/collection.rs:5` | `route_tables` | 26 |
| `crates/aws/src/network/collection.rs:33` | `internet_gateways` | 26 |
| `crates/aws/src/network/collection.rs:61` | `nat_gateways` | 26 |
| `crates/aws/src/network/collection.rs:89` | `endpoints` | 26 |
| `crates/aws/src/network/collection.rs:117` | `network_acls` | 26 |
| `crates/aws/src/network/collection.rs:145` | `peering_connections` | 29 |
| `crates/aws/src/network/gateways.rs:4` | `map_nat_gateway` | 82 |
| `crates/aws/src/network/gateways.rs:87` | `map_endpoint` | 98 |
| `crates/aws/src/network/page.rs:41` | `arn_link` | 31 |
| `crates/aws/src/network/page.rs:73` | `link_resolved` | 40 |
| `crates/aws/src/network/page.rs:114` | `finish` | 28 |
| `crates/aws/src/network/resource_mapping.rs:4` | `resource` | 47 |
| `crates/aws/src/network/route_evidence.rs:57` | `route_links` | 83 |
| `crates/aws/src/network/routing.rs:4` | `map_route_table` | 84 |
| `crates/aws/src/network/routing.rs:89` | `map_internet_gateway` | 33 |
| `crates/aws/src/scan.rs:19` | `scan_inner` | 143 |
| `crates/aws/src/scan.rs:198` | `discover_regions` | 32 |
| `crates/aws/src/scan.rs:232` | `collect_region` | 31 |
| `crates/aws/src/source_discovery.rs:15` | `collect` | 32 |
| `crates/aws/src/source_discovery.rs:49` | `record_selection` | 52 |
| `crates/aws/src/supplemental/config.rs:5` | `config` | 39 |
| `crates/aws/src/supplemental/config.rs:45` | `parse_config_row` | 54 |
| `crates/aws/src/supplemental/config/aliases.rs:20` | `config_aliases` | 194 |
| `crates/aws/src/supplemental/explorer.rs:5` | `explorer` | 43 |
| `crates/aws/src/supplemental/explorer.rs:49` | `map_explorer` | 40 |
| `crates/aws/src/supplemental/resource_types.rs:4` | `resource_kind` | 41 |
| `crates/aws/src/supplemental/resource_types.rs:83` | `explorer_elbv2_kind` | 34 |
| `crates/aws/src/task_definitions/collection.rs:5` | `collect` | 39 |
| `crates/aws/src/task_definitions/collection.rs:45` | `remember_list_page` | 39 |
| `crates/aws/src/task_definitions/collection.rs:86` | `collect_with` | 38 |
| `crates/aws/src/task_definitions/collection.rs:126` | `describe_with` | 28 |
| `crates/aws/src/task_definitions/components.rs:4` | `map_logs` | 29 |
| `crates/aws/src/task_definitions/components.rs:34` | `map_container` | 63 |
| `crates/aws/src/task_definitions/components.rs:98` | `map_volume` | 35 |
| `crates/aws/src/task_definitions/definition.rs:4` | `map_definition` | 147 |
| `crates/aws/src/task_definitions/identity.rs:4` | `arn_parts` | 27 |
| `crates/aws/src/task_definitions/identity.rs:32` | `definition_identity` | 31 |
| `crates/aws/src/task_definitions/relationships.rs:4` | `returned_arn_edge` | 37 |
| `crates/engine/examples/benchmark.rs:97` | `fixture` | 34 |
| `crates/engine/examples/benchmark.rs:142` | `network_case` | 33 |
| `crates/engine/examples/resource_views.rs:10` | `main` | 43 |
| `crates/engine/examples/resource_views.rs:71` | `generate` | 92 |
| `crates/engine/src/export/publication.rs:63` | `snapshot_header` | 36 |
| `crates/engine/src/export/publication.rs:138` | `publish_or_discard` | 27 |
| `crates/engine/src/graph/delivery/execution.rs:5` | `apply_batch` | 59 |
| `crates/engine/src/graph/delivery/execution.rs:65` | `send_attempt` | 32 |
| `crates/engine/src/graph/delivery/pipeline.rs:7` | `push_file_with_options` | 61 |
| `crates/engine/src/graph/delivery/plan.rs:56` | `bind_plan` | 47 |
| `crates/engine/src/graph/delivery/replay.rs:79` | `rows` | 38 |
| `crates/engine/src/graph/delivery/replay.rs:131` | `json_map_keys` | 40 |
| `crates/engine/src/graph/plan_reader.rs:23` | `accept` | 48 |
| `crates/engine/src/graph/snapshot.rs:18` | `export_snapshot_with_cancellation` | 60 |
| `crates/engine/src/jobs/connection.rs:38` | `initialize` | 28 |
| `crates/engine/src/jobs/execution.rs:8` | `submit` | 28 |
| `crates/engine/src/jobs/execution.rs:40` | `claim` | 32 |
| `crates/engine/src/jobs/execution.rs:100` | `complete` | 28 |
| `crates/engine/src/jobs/transitions.rs:19` | `retry` | 32 |
| `crates/engine/src/limiter/admission.rs:45` | `admit` | 26 |
| `crates/engine/src/limiter/execution.rs:46` | `execute_inner` | 34 |
| `crates/engine/src/limiter/pacing.rs:142` | `feedback` | 26 |
| `crates/engine/src/limiter/policy.rs:134` | `validate` | 30 |
| `crates/engine/src/limiter/registry.rs:5` | `bucket` | 32 |
| `crates/engine/src/limiter/retry.rs:6` | `attempt` | 32 |
| `crates/engine/src/store/connection.rs:6` | `open` | 37 |
| `crates/engine/src/store/connection.rs:63` | `connection_options` | 40 |
| `crates/engine/src/store/persistence.rs:75` | `resource_batch` | 26 |
| `crates/engine/src/store/prepared.rs:32` | `prepare_page` | 46 |
| `crates/engine/src/store/query.rs:133` | `sql_value` | 33 |
| `crates/engine/src/store/scan.rs:6` | `begin_scan` | 26 |
| `crates/engine/src/store/scan_state.rs:80` | `validate_page_sequence` | 26 |
| `crates/engine/src/store/scan_state.rs:136` | `finalize_scan` | 32 |
| `crates/engine/src/store/schema.rs:4` | `initialize_schema` | 68 |
| `crates/engine/src/store/schema.rs:102` | `migrate_sources` | 83 |
| `crates/engine/src/store/schema.rs:186` | `verify_managed_columns` | 70 |
| `crates/engine/src/store/sql_policy.rs:15` | `validate_query` | 45 |
| `crates/engine/src/store/sql_policy.rs:69` | `lex_normal` | 35 |
| `crates/engine/src/store/sql_policy.rs:105` | `validate_query_prefix` | 52 |
| `crates/engine/src/store/streams.rs:48` | `json_stream` | 30 |
| `crates/sdk-catalog/examples/audit.rs:61` | `main` | 80 |
| `crates/sdk-catalog/examples/audit.rs:142` | `options` | 45 |
| `crates/sdk-catalog/examples/audit/services.rs:4` | `audit_service` | 177 |
| `src/analysis/diff.rs:15` | `report` | 48 |
| `src/analysis/diff.rs:137` | `next_pair` | 30 |
| `src/analysis/diff.rs:208` | `observe` | 31 |
| `src/analysis/diff.rs:257` | `compare` | 37 |
| `src/analysis/findings.rs:98` | `report` | 30 |
| `src/analysis/graph.rs:15` | `report` | 37 |
| `src/analysis/graph.rs:61` | `seed` | 35 |
| `src/analysis/graph.rs:107` | `expand` | 28 |
| `src/analysis/graph/model.rs:116` | `new` | 31 |
| `src/analysis/graph/model.rs:152` | `accept` | 35 |
| `src/analysis/graph/model.rs:188` | `insert` | 36 |
| `src/analysis/graph/model.rs:246` | `path` | 34 |
| `src/architecture.rs:66` | `report_open` | 30 |
| `src/catalog.rs:3` | `catalog` | 60 |
| `src/catalog.rs:78` | `scope` | 32 |
| `src/coverage.rs:8` | `report` | 48 |
| `src/coverage.rs:58` | `merge_parent_capabilities` | 135 |
| `src/delivery.rs:38` | `validate` | 27 |
| `src/delivery.rs:183` | `after_scan` | 39 |
| `src/delivery.rs:223` | `merge_delivery_result` | 27 |
| `src/delivery/attempt.rs:22` | `deliver_open_store` | 55 |
| `src/delivery/publication.rs:4` | `write_outputs` | 47 |
| `src/delivery/publication.rs:53` | `project` | 45 |
| `src/delivery/publication.rs:100` | `persist_receipt` | 36 |
| `src/demo.rs:41` | `resources` | 39 |
| `src/demo.rs:102` | `store_demo_resource` | 34 |
| `src/jobs.rs:42` | `execute_open` | 46 |
| `src/jobs.rs:89` | `validate_scan` | 30 |
| `src/jobs.rs:121` | `submit` | 47 |
| `src/jobs/worker.rs:16` | `run` | 29 |
| `src/jobs/worker.rs:47` | `run_queue` | 34 |
| `src/jobs/worker/claim.rs:4` | `run_claim` | 57 |
| `src/jobs/worker/summaries.rs:111` | `job_summary` | 28 |
| `src/jobs/worker/summaries.rs:140` | `compact_result` | 38 |
| `src/jobs/worker/supervision.rs:28` | `supervise_futures` | 40 |
| `src/main.rs:67` | `dispatch` | 73 |
| `src/main.rs:184` | `push_graph` | 27 |
| `src/main.rs:237` | `run_discovery` | 46 |
| `src/plan.rs:13` | `create` | 191 |
| `src/plan/descriptors.rs:3` | `cloud_catalog` | 81 |
| `src/plan/selection.rs:3` | `validate_options` | 41 |
| `src/plan/selection.rs:45` | `select_groups` | 40 |
| `src/plan/selection.rs:102` | `cloud_plan` | 43 |
| `src/scan.rs:62` | `run_with_executor` | 37 |
| `src/scan.rs:101` | `deliver_scan_outputs` | 30 |
| `src/scan/pipeline.rs:75` | `collect_job` | 41 |
| `src/scan/pipeline.rs:187` | `finalize_scans` | 34 |
| `src/scan/targets.rs:47` | `prepare_jobs` | 66 |
| `src/scan/targets.rs:129` | `select_accounts` | 36 |
| `src/targets.rs:7` | `validate` | 54 |
| `src/targets.rs:90` | `validate_role_arn` | 27 |

## Appendix B: All non-test functions over five parameters

| Location | Function | Parameters including receiver |
|----------|----------|-------------------------------|
| `crates/aws/src/cloud_control/parents/children.rs:4` | `collect_child` | 6 |
| `crates/aws/src/cloud_control/parents/children.rs:156` | `emit_failed_read` | 6 |
| `crates/aws/src/cloud_control/parents/discovery.rs:101` | `collect_parent_item` | 6 |
| `crates/aws/src/common.rs:86` | `resource` | 6 |
| `crates/aws/src/discovery/pages.rs:82` | `accept` | 6 |
| `crates/aws/src/discovery/pages.rs:138` | `advance` | 6 |
| `crates/aws/src/ec2/mapping.rs:16` | `resource` | 6 |
| `crates/aws/src/ec2/mapping.rs:54` | `link` | 6 |
| `crates/aws/src/global_inventory/cloudfront/mapping.rs:94` | `distribution_resource` | 7 |
| `crates/aws/src/load_balancing/targets.rs:101` | `target_reference` | 6 |
| `crates/aws/src/network/page.rs:26` | `link` | 6 |
| `crates/aws/src/network/page.rs:41` | `arn_link` | 6 |
| `crates/aws/src/network/page.rs:73` | `link_resolved` | 7 |
| `crates/aws/src/network/resource_mapping.rs:4` | `resource` | 6 |
| `crates/aws/src/network/route_evidence.rs:57` | `route_links` | 6 |
| `crates/aws/src/task_definitions/relationships.rs:42` | `file_system_edge` | 6 |
| `crates/engine/src/graph/delivery/execution.rs:65` | `send_attempt` | 6 |
| `src/analysis/diff.rs:15` | `report` | 6 |
| `src/delivery/attempt.rs:130` | `delivery_receipt` | 6 |
| `src/delivery/publication.rs:4` | `write_outputs` | 7 |
| `src/delivery/publication.rs:53` | `project` | 7 |
| `src/jobs.rs:121` | `submit` | 6 |
| `src/jobs/worker/supervision.rs:4` | `supervise` | 6 |
| `src/jobs/worker/supervision.rs:28` | `supervise_futures` | 6 |

## Appendix C: Non-test async spans over 60 lines

| Location | Function | Physical lines |
|----------|----------|----------------|
| `crates/aws/examples/emulator_branches.rs:87` | `probe` | 67 |
| `crates/aws/examples/emulator_branches.rs:155` | `probe_explorer` | 68 |
| `crates/aws/src/cloud_control/parents/discovery.rs:101` | `collect_parent_item` | 83 |
| `crates/aws/src/common/pagination.rs:22` | `paginate_inner` | 105 |
| `crates/aws/src/discovery/organizations/hierarchy.rs:5` | `collect_hierarchy` | 62 |
| `crates/aws/src/ecs/tasks.rs:5` | `tasks` | 78 |
| `crates/aws/src/scan.rs:19` | `scan_inner` | 143 |
| `crates/engine/src/graph/delivery/pipeline.rs:7` | `push_file_with_options` | 61 |
| `crates/engine/src/store/schema.rs:4` | `initialize_schema` | 68 |
| `crates/engine/src/store/schema.rs:102` | `migrate_sources` | 83 |
| `crates/engine/src/store/schema.rs:186` | `verify_managed_columns` | 70 |
| `src/scan/targets.rs:47` | `prepare_jobs` | 66 |
