# AWS operation reconciliation for v0.6

> Status: Historical evidence. It records the v0.6 source and run
> reconciliation of the baseline collector and is not updated; links into
> `artifacts/` point to an ignored local directory and do not resolve in a
> clone. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

**The v0.6 source, collection, full live workspace and graph-delivery checks passed for the bound run. The whole-project 95% mutation target remains open.**

The current reconciliation receipt binds the source tree and compiled collector to the run. Collection passed **2,522 harness checks with zero failed checks**, both repetitions of the five collector snapshots passed, and the full live first-party workspace passed **530 tests, zero failed and zero ignored**. All five snapshots then passed exact JSON/SQLite and real FalkorDB projection comparisons, checkpointed replay and cleanup. Ten portable exports were retained and verified. The earlier interrupted attempt remains documented below. See [VERIFICATION.md](VERIFICATION.md) for the current release result index.

The previous report is preserved separately in `AWS_OPERATION_RECONCILIATION_V05.md`. Historical v0.4/v0.5 counts are not substituted for the current source or run.

## Source boundaries and resource breadth

| Source-audit measure | Reconciled count | Meaning |
| --- | ---: | --- |
| Explicit SDK operation identities | 72 | Distinct operations used by production inventory/discovery code |
| Explicit Rust SDK send sites | 73 | `GetCallerIdentity` has two production call sites |
| Separate credential-provider boundary | 1 | `sts.assume_role`, owned by the `aws-config` provider |
| Total operation boundaries | 73 | 72 explicit identities plus the provider boundary |
| SDK service namespaces | 15 | Namespaces represented by the explicit operations |
| AWS source files inspected | 32 | Current source tree inspected by the lexical auditor |
| Nonempty channel sends excluded | 6 | Event transport is not an AWS SDK operation |
| Added / removed SDK boundaries | 0 / 0 | Same operation identities as the retained historical matrix |

The different uses of **73** must remain distinct: 73 explicit send sites correspond to 72 explicit operation identities; adding the provider boundary gives 73 operation identities in the request evidence. An operation identity can have many attempts and appear in more than one scan.

Cloud Control has **421 configured resource types across 76 service families**, split into **397 frozen root descriptors and 24 parent-dependent descriptors**. They still use only `cloudcontrol.list_resources` and `cloudcontrol.get_resource`. The parent workflow supplies `ResourceModel` on each relevant list request and reuses the exact returned identifier for each read. It does not introduce another SDK operation identity.

The merged descriptors define **4,309 provider columns**, **968 property relationship rules across 255 types**, and **24 runtime `CHILD_OF` workflows**. In total, **268 distinct types** have a property rule or a parent workflow. SQLite schema 13 contains the 421 source-specific provider views. These are implemented schema and mapping counts; they do not establish that every relationship rule or every real AWS provider was exercised. [Root catalog](../catalog/cloudcontrol-resources.json) · [Parent catalog](../catalog/cloudcontrol-parent-resources.json) · [Parent workflow contract](PARENT_COLLECTIONS.md)

## Historical reviewed additive operations

The original source review recorded five native KMS operations (`ListKeys`,
`DescribeKey`, `GetKeyRotationStatus`, `ListResourceTags`, `ListAliases`) and
five Step Functions operations (`ListStateMachines`, `DescribeStateMachine`,
`ListTagsForResource`, `ListActivities`, `DescribeActivity`) after the frozen
79-operation matrix. Their former overlay bound source locations, hashes and
SDK module fingerprints; M1.7 removed that overlay with the typed-SDK-derived
inventory. Historical v0.4/v0.5/v0.6 receipts remain unchanged.

The former 89-operation / 91-callsite total (79 baseline operations at 80
callsites plus 10 reviewed additions at 11 callsites) is historical, not a
current inventory or proof that every operation was exercised. The Rust
operation scanner and its SDK-inventory audit were removed in M1.7. This
document no longer provides a reproduction command for that retired audit.

A source-only receipt has `emulator_evidence.status = "pending"` and `current_run_bound = false`. It must not replace an execution-bound receipt as though both had the same scope. The auditor protects `catalog/aws-operations.json` and the historical v0.4/v0.5 reconciliation files from overwrite.

The auditor masks Rust comments, string literals, raw strings and character literals while preserving line positions. It enumerates empty-argument `.send()` calls under `crates/aws/src`, resolves direct builder chains and SDK client namespaces, and compares the complete path/function/operation multiset with the reviewed matrix. Nonempty channel sends are excluded. New or unrecognized call shapes fail closed rather than inheriting a nearby operation. Pinned SDK operation and credential-provider module hashes are checked against the lockfile and retained provenance.

The two Cloud Control send sites moved from the historical lines 275/287 to **333/345** in this bound source tree. Their operation identities are unchanged. File hashes are authoritative; line positions alone are not source identity. The auditor checks the source closure before and after its run and bounds files, bytes, tokens, calls and request records. This is a lexical audit, not Rust type checking, an AST proof, branch coverage, a Cargo build or live AWS execution.

## Bind a completed collection run

After the harness has completed its collection and assertions, bind a fresh reconciliation receipt to the actual run directory:

```bash
cargo build -p inventory-harness
target/debug/inventory-harness reconcile-aws-operations \
  --emulator-summary artifacts/emulator-v06-complete/summary.json \
  --output artifacts/aws-operation-source-reconciliation-local-bound.json
```

The bound receipt verifies the source closure, raw and canonical catalog hashes, unchanged executable, architecture fixture hashes, embedded catalog receipt and root Cloud Control command/scan identity. Its execution metadata also retains the parent catalog, parent fixture and separate parent command identity. The auditor checks operation identities, credential-provider classification, phase sets, attempt counts and HTTP-status histograms against the bounded Rust request ledger. Python fixture traffic cannot satisfy these rows.

The current report records `emulator_evidence.status = "current_execution_receipts_bound"` and `current_run_bound = true`. This status means the recorded collection evidence is bound. It does not require or imply a completed full workspace run, successful graph delivery, live AWS fidelity or a mutation score.

## Recorded collection outcomes

The bound ledger contains **2,673 requests**: **1,336 Rust collector/provider/probe requests** and **1,337 Python fixture/startup requests**. All **73 operation identities** were observed. The classifications are:

| Recorded outcome | Explicit SDK operations | Separate provider boundaries | Total |
| --- | ---: | ---: | ---: |
| `collector_success` | 63 | 1 (`sts.assume_role`) | 64 |
| `explicit_sdk_probe_success` | 4 | 0 | 4 |
| `emulator_not_implemented` | 2 | 0 | 2 |
| `sdk_decode_failure` | 3 | 0 | 3 |
| **Total** | **72** | **1** | **73** |

The four successful probes are `ec2.describe_regions`, `s3.head_bucket`, `resourceexplorer2.get_default_view` and `resourceexplorer2.list_views`. They establish the separately invoked SDK/endpoint behavior; they do not prove that those conditional production branches ran.

`organizations.list_accounts` and `organizations.list_children` returned HTTP 501. `resourceexplorer2.get_index`, `resourceexplorer2.get_view` and `resourceexplorer2.list_resources` received HTTP 200 responses that the Rust SDK rejected during decoding. These remain explicit unsupported or decoding outcomes. A `collector_success` row is an operation classification that must be read alongside semantic assertions and persisted scopes, not a declaration that every resource mapping is complete.

### Root and parent Cloud Control scans

| Recorded phase | List requests | Detailed read requests | Persisted resources | Relationships | Complete / failed scopes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Frozen 397-root selection | 397: 386 HTTP 200, 11 HTTP 400 | 401, all HTTP 200 | 401 | 32 | 787 / 11 |
| 21 selected children plus six prerequisite root types | 72, all HTTP 200 | 179, all HTTP 200 | 179 | 185 | 269 / 0 |

Each phase additionally made one successful `GetCallerIdentity` request. The root scan remains **partial** because eleven selected resource types are not registered by the pinned emulator. It contains detailed reads for **386 root types**. Its resource count is higher than the historical v0.5 count because the new parent fixtures add resources under existing root types; the selected root descriptor bytes remain frozen.

The parent scan is **complete** for its selected scopes. Its 179 observations comprise **18 prerequisite resources and 161 children**. Its 185 relationships comprise **161 exact `CHILD_OF` edges and 24 independently authored architecture edges**. All **1,328 parent assertions passed**. Across the two Cloud Control scans, the database contains detailed observations for **407 distinct types**: 386 successful root types plus all 21 child types. These scans overlap on prerequisites and must not be summed into a distinct AWS resource count.

The 101 EKS addon fixtures force a second list page. The literal parent-A chain returns 100 matching rows and then an empty terminal page; parent B receives an empty first page with a continuation token, followed by its one child. The production snapshot also records the existing catalog fixture's separate parent context. Parent filter chains retain independent scopes; no empty nonterminal page is treated as exhaustive completion.

The remaining snapshots retain their own results: native inventory is complete with **150 resources and 340 relationships**; Config/Explorer is partial with **11 resources and zero relationships**; assumed-role IAM is complete with **13 resources and 22 relationships**. SQLite integrity is reported as `ok`. Successful harness checks preserve the known partial outcomes instead of converting them to complete scans.

## Verified local collection and delivery

| Stage | Evidence for this bound run |
| --- | --- |
| Source/operation reconciliation | Passed; 72 explicit operations plus the provider boundary, with unchanged identities |
| Ordinary collector and fixture assertions | 2,522 passed, zero failed; known unsupported/decoding outcomes retained |
| Five snapshot cases, first repetition | Completed, exit 0 |
| Five snapshot cases, second repetition | Completed, exit 0 |
| Full first-party workspace including all ten guarded live cases | 530 passed, zero failed, zero ignored across 28 reported suites |
| Exact JSON/SQLite and FalkorDB delivery | Five of five cases passed every projected-property, identity, provenance, label/cardinality and scan-marker comparison |
| Checkpointed replay and cleanup | All five cases: zero graph attempts, zero applied batches, successful graph cleanup |
| Retained portable artifacts | Five JSONL files match live-verified bytes/hashes; five JSON files match their JSONL record streams |
| Whole-project 95% mutation target | Not established by these commands; no current passing campaign claimed |

The first and second five-case repetitions completed with exit 0. The full workspace receipt records successful completion in **176.813 seconds**; its retained log reports 530 passed, zero failed and zero ignored across 28 suites. The command excluded the dependency-only `inventory-sdk-catalog` package, so this is not a compilation or execution result for every published SDK crate.

The delivery verifier then compared every intended resource/edge property and graph marker against SQLite, including exact property/label cardinality and unresolved references. Each case passed replay without another graph attempt and completed cleanup:

| Snapshot | Observed resource nodes | Unresolved target nodes | Relationships | First applied batches | Batches skipped on replay |
| --- | ---: | ---: | ---: | ---: | ---: |
| Native | 150 | 43 | 340 | 6 | 6 |
| Config/Explorer | 11 | 0 | 0 | 4 | 4 |
| Root Cloud Control | 401 | 25 | 32 | 6 | 6 |
| Parent-dependent Cloud Control | 179 | 6 | 185 | 5 | 5 |
| Assumed-role IAM | 13 | 12 | 22 | 5 | 5 |

Unresolved targets are references without an observation in that snapshot; they are not additional collected resources. Partial collection status remains partial in JSON and the graph marker. The supervised command completed with exit 0 in **182.286 seconds**, with `drained = true` and `forced = false`. The complete owned runner completed with exit 0 in **330.158 seconds**, including fixture setup and baselines. The wrapper was historically named `mutate.py` (now ported to Rust as the `inventory-harness mutate` subcommand); this command ran ordinary tests and delivery verification, not a mutation campaign.

### Portable JSON and JSONL

The retention script (historical Python artifact; export functionality now in Rust) ran only after successful live delivery. It exported all five bound scans in both formats, retaining **ten artifacts totaling 3,397,132 bytes**. Every JSONL SHA-256, byte count and record count matches its independently live-verified artifact. Each JSON header, ordered record stream and footer matches the corresponding JSONL structurally. Partial Config/Explorer and root Cloud Control exports retain exit **2** in both formats; complete cases retain exit **0**. Input receipts, collector bytes and SQLite database hashes remained unchanged.

The portable export receipt records command arguments, scan IDs, binary/input/report hashes, exported byte counts and file digests. The execution receipt records exit 0 in **3.354 seconds**. The script made no AWS or graph requests. Files are retained under exports/, including native.json and parents.json.

### Preserved interrupted attempt

The earlier run passed its collection assertions and two snapshot repetitions, but its later outer command stopped with `status = "fixture_unavailable"`, `fixture_error_type = "URLError"` and exit **124** after **154.8 seconds**. Its workspace receipt remained `running`, and that attempt produced no completed delivery-verification receipt. Its separate reconciliation remains evidence of that source/collection binding. None of those failed or incomplete receipts was rewritten as the successful `emulator-v06-complete` result.

The successful current run establishes the observed local request, decoding, mapping, SQLite, export and graph-projection behavior. The emulator's generic Cloud Control store does not execute each real provider's implementation. This evidence does not establish real AWS regional availability, production IAM behavior, quota ceilings, throughput, exhaustive relationship coverage or session-expiration credential refresh.

## Bound provenance

The report binds an **84-file source closure**, the compiled collector, both descriptor catalogs, the merged runtime catalog, independent fixtures and exact command/ledger artifacts. At reconciliation review, every source-file hash, bound artifact hash and executable hash matched the retained receipt.

| Bound input | SHA-256 |
| --- | --- |
| Collector executable | `57660e895513595a9247fc22bd9b80c921417de9c2fb32a59f250830d0728adf` |
| Source closure | `a79a481e63ac3fafba9fe5501b7b347b4c40ced4df899567e222f7069dc6fd29` |
| Frozen root catalog | `7d551d1ae5d32e83dc7bf45d5ef94fca35b0cecbc510cd5a3fe75aafa73300e4` |
| Parent catalog | `26593330f8388812474ca9d8c81f0f5d23ac2a57410a11b99bd24ee5116e354a` |
| Canonical merged runtime catalog | `06b2dc9e209cd6a95383061f85d69250da68ba2eed4ab16463dff4c5604231f5` |
| Parent fixture | `500b9f29c6007c88f4f5cfd702cad210b32ed03b89cddf3d261f83dfd8cd94d7` |
| Request ledger | `d9f8783f24dc101f6fda82adc85a6df12168ec80aab9281dc09b9aba9a0a1895` |
| Successful delivery report | `f5fe2946823013669a3d4fc798dd3e5a611613adfe403a4745451cbb29ddf59e` |
| Portable export receipt | `d5dd3050df43dd51bcb60d9d0024134bffa78dfc469e8181181a05b2ecb06283` |

The historical operation matrix and historical reconciliation receipts retain their own hashes. A changed source tree or binary requires fresh execution binding; old provider success counts cannot certify new code.

## Auditor tests

```bash
cargo test -p inventory-harness --test operation_reconciliation --test execution_binding
```

The auditor tests cover misleading comments and literals, nested arguments, namespace changes, channel sends, unsupported call shapes, lexical bounds, stale source/catalog evidence, provider relabeling, missing operation identities, fixture-traffic exclusion and inflated request histograms. They validate the audit mechanism. They are not AWS operation execution or mutation-score evidence; current test receipts are indexed in [VERIFICATION.md](VERIFICATION.md).
