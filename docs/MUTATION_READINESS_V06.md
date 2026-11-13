# Mutation readiness audit for the v0.6 development increment

> Status: Historical evidence. It records the v0.6 audit of the Python mutation
> harness and is not updated; links into `artifacts/` point to an ignored local
> directory and do not resolve in a clone. See [VISION](VISION.md) and
> [ROADMAP](ROADMAP.md).

The complete 95% mutation gate is still **unachieved**. This increment repairs
acceptance and provenance defects in the Python harness. It does not
represent a new Rust mutation campaign, does not reuse interrupted v0.4 counts
as current results, and does not change the threshold or remove any candidate.

## Repairs made before the next campaign

### The final aggregate now enforces the live-test policy itself

The v0.5 `run_all.py` launcher requested all nine guarded integration tests, but
`gate.py aggregate` did not require that policy. A caller could instead aggregate
ordinary unit-only package partitions and obtain a whole-project pass. A
Python-only synthetic reproduction accepted four caught candidates, one per
production package, with no fixture context. This was an acceptance-boundary
defect; it was not evidence about actual Rust mutation outcomes.

The required test names now live in `gate.py`, and the launcher consumes the
same policy. The v0.6 parent snapshot scenario expands that fixed contract to
**ten** tests, including `emulator_snapshot_parents`. The AWS, root, and verified native-engine partition runners also
consume it, so they remain eligible inputs to the stricter aggregate. The root
runner now verifies and freezes the independent fixture summary as well as the
snapshot; all these runners bind the gate implementation as an input. The final
aggregate additionally requires:

| Package | Required live tests | Accepted fixture |
| --- | ---: | --- |
| `aws-inventory` | 2 CLI / durable-worker tests | Supervised AWS Emulator and FalkorDB |
| `inventory-core` | 0; pure core tests remain package-local | No live fixture needed |
| `inventory-engine` | 3 native graph tests | Supervised native FalkorDB or the combined fixture |
| `inventory-aws` | 5 collector snapshot tests, including parent collection | Supervised AWS Emulator |

Every partition containing a package with live tests must enable ignored tests,
require two clean baselines, name every mandatory test for that package, and
bind its fixture environment. The existing receipt parser still verifies actual
test execution, source restoration, candidate completeness, and closed healthy
fixture lifetime. AWS fixture acceptance binds the snapshot digest to both its
environment digest and the exact frozen input path/content pair. A native graph
fixture cannot substitute for an AWS fixture.

Ordinary package reports remain useful diagnostic reports. They can be complete
for their declared package scope while being ineligible for whole-project
acceptance. Historical manifests are not rewritten. Final acceptance deliberately
rejects historical or future manifests that omit these required conditions.

### Aggregation requires one frozen identity for each fixture kind

The prior aggregate validated each campaign's fixture independently but did not
compare their identities. An independent synthetic reproduction combined two
source-identical, internally consistent campaigns with different AWS snapshot
digests and obtained a pass. That allowed selecting evidence from different
fixtures even though the required contract is a frozen source/test/catalog and
fixture closure.

The aggregate now records the fixture identities and rejects multiple AWS
snapshot digests or multiple native FalkorDB manifest digests. It preserves every
attempt and candidate in the report; it does not select a favorable fixture or
discard the extra outcomes. A later run against the same digest remains eligible
without counting the candidate twice. AWS-backed partitions and separately
verified native-engine partitions remain supported as distinct fixture kinds;
pure core partitions need no unrelated fixture.

### Embedded dependency data now belongs to frozen package inputs

The prior AWS/root partition closure included dependency Rust source and Cargo
manifests but omitted engine SQL migrations embedded with `include_str!`. The
root closure also omitted the SDK catalog's embedded `sdk-services.json`.
A later data-only change could therefore leave a package partition apparently
current even though its compiled behavior changed.

Dependency `sql`, `schemas`, and `data` directories and the SDK catalog service
manifest are now hashed along with dependency production source. This preserves
the separation between a dependency's production inputs and unrelated external
test files. The all-package closure already included engine SQL through the
engine's own package; the correction matters particularly for separately frozen
AWS and root partitions and their later aggregation.

### Infrastructure failures cannot remove mutants from the denominator

The old unviable classifier accepted a generic line beginning with `error:`.
That included linker errors, failed build scripts, and a bare Cargo compilation
summary. Those messages do not establish that a source mutation was rejected by
the Rust compiler.

An unviable result now requires an explicit `error[Edddd]:` Rust diagnostic in
the failed build phase. Linker failures, build-script failures, compiler internal
errors, resource failures, and compilation summaries without that diagnostic
remain unresolved. The coded diagnostics `E0514` (incompatible dependency
compiler) and `E0786` (invalid crate metadata) are also infrastructure failures.
`E0463` is treated that way when the missing crate is `std` or `core`; a missing
application crate can still indicate source unviability. This distinction was
independently reproduced with parser inputs before and after the fix. Uncoded
errors are also unresolved pending investigation;
they are not automatically discarded as unviable. Infrastructure signatures
take precedence even when another coded diagnostic appears in the same log.

The numeric decision is unchanged:

```text
caught * 100 >= (caught + missed) * 95
```

There must be at least one viable completed candidate. Missing outcomes,
unresolved outcomes, conflicting completed results, changed inputs, and evidence
errors still prevent a pass. Genuine coded compiler-unviable outcomes remain
visible and are excluded from the score denominator as before.

### Parent SDK test deadlines remain unresolved outcomes

The parent SDK wire fixture previously propagated a generic `Elapsed(())` when
its outer test deadline expired. That could look like a failing assertion and be
credited as a caught mutant. The Rust fixture now emits the stable marker
`parent SDK wire fixture exceeded its 20-second deadline`, and the gate classifies
that marker as an unresolved timeout even when a failed-test summary is present.
Assertions about the production SDK's own timeout classification remain eligible
to catch a mutation. The helper regression checks this distinction; execution of
the Rust fixture belongs to the separate integration lane.

## Executed harness verification

```sh
python3 -m unittest discover -s scripts/mutation -p 'test_*.py' -v
```

Result: **126 tests passed**, exit code 0, 6.312 seconds of test execution and
6.690 seconds including command overhead in this session. The recorded helper
source hashes were unchanged across the run. Parameterized negative controls
cover removal of each of the ten live names and each bound environment value.

The new acceptance tests use synthetic file-backed manifests, candidates,
baseline logs, outcome logs, fixture receipts, and evidence hashes. They execute
the actual aggregate parser while replacing only external Cargo candidate
enumeration and tool-version inspection. They do not run Cargo, call AWS, or
establish a Rust mutation score. A valid synthetic live campaign can pass;
equally complete unit-only evidence, a missing live name, an absent fixture
lifetime, or mixed fixture identities cannot. An independent reviewer reran all
126 helper tests successfully and separately reproduced the corrected mixed-AWS
fixture and compiler-infrastructure classifications.

Evidence:

- [`gate-tests-v06-fixture-integrity.json`](../artifacts/mutation/gate-tests-v06-fixture-integrity.json)
- [`gate-tests-v06-fixture-integrity.log`](../artifacts/mutation/gate-tests-v06-fixture-integrity.log)
- [`test_acceptance.py`](../scripts/mutation/test_acceptance.py)
- [`gate.py`](../scripts/mutation/gate.py)
- [`run_all.py`](../scripts/mutation/run_all.py)

No Rust compilation or mutation execution was performed by this audit lane.
The root integration lane owns those commands and their separate receipts.
The preceding 114-test receipt is retained separately as `gate-tests-v06.json`;
it predates the partition-runner alignment. `gate-tests-v06-final.json` records
the subsequent 117-test result under the old nine-case acceptance contract.
`gate-tests-v06-current.json` records 117 tests with the ten-case policy, and
`gate-tests-v06-reviewed.json` records 118 tests including the parent wire
deadline marker. These remain historical receipts; the latest 126-test result
also binds the cross-campaign identity and compiler-infrastructure fixes. None
of these Python receipts claims that the newly required Rust parent case was
executed.

## Oracle build selection: validated fix

The initial live fixture attempt is preserved under `artifacts/emulator-v06`.
Its package-only oracle command rebuilt dependencies under a narrower feature
graph and reached the **600-second compilation deadline**, exiting 124 before
test execution. The original command, log, and timeout receipt remain unchanged.

The oracle now keeps the workspace/all-feature package selection used by the
ordinary test build while selecting only the `emulator_snapshot` integration
target. The integration lane validated that selection with:

```sh
cargo test --workspace --exclude inventory-sdk-catalog --all-features \
  --test emulator_snapshot --locked --offline -j1 --no-run --verbose
```

This check exited **0 in 3.036 seconds wall time**, with Cargo reporting **2.86
seconds**. Every SDK entry was `Fresh`, and exactly one `emulator_snapshot`
executable was selected. It confirms cache reuse and target selection for this
recorded environment; `--no-run` executed no Rust tests. Evidence:
[`oracle-workspace-build-check.json`](../artifacts/verification-v06/oracle-workspace-build-check.json)
and its [log](../artifacts/verification-v06/oracle-workspace-build-check.log).

The separate callback/receipt suite passed **5 tests: 3 existing and 2 new**.
It verifies both 600-second oracle passes and stopping before the mutation
command if either fails. Its [receipt](../artifacts/oracle-callback-tests-v06.json)
records mocked callback and local process-supervision scope; these five are not
five new tests added to the 126-test mutation-helper result above.

The earlier note that `emulator-v06-verified` was running was a historical
checkpoint. That attempt subsequently failed before fixture/test execution:
the emulator's `--version` subprocess exceeded its 10-second probe timeout
([receipt](../artifacts/verification-v06/emulator-v06-verified.json),
[log](../artifacts/verification-v06/emulator-v06-verified.log)). The following
attempt, `artifacts/emulator-v06-final`, passed **2,522 collection checks** and
both **5/5 snapshot baselines**, then failed its outer health check during the
nested full-workspace suite. Those successful baselines do not establish a
complete integration pass.

## Supervised cancellation and terminal receipts

The `emulator-v06-final` outer receipt records `fixture_unavailable`, `URLError`,
exit 124, and 154.8 seconds elapsed. Its nested `workspace-live.json` remained
`running`. The prior supervisor started Cargo in a separate process group but
did not handle SIGTERM: terminating the verifier could bypass its cleanup and
leave that Cargo group running. The exact health-probe cause remains **unknown**
because that historical receipt retained only the exception class. The attempt
and its evidence remain unchanged.

The repaired supervisor handles SIGINT/SIGTERM, drains its owned command group,
and writes a terminal receipt before restoring the previous handlers. Group
cleanup also checks descendants when the original leader has already exited.
The outer cleanup budget is **35 seconds total, including 30 seconds of graceful
shutdown**; a nested supervisor inherits **8 seconds total**, so its cleanup and
receipt can finish before the outer escalation. An exhausted deeper supervision
budget fails before spawning another command. Native fixture teardown also
fails closed and preserves any existing primary error.

The original **3-second health timeouts and strict failure gate are unchanged**.
New failures record the probe stage, exception/reason classes, numeric errno or
TLS verification code, and probe duration without copying raw exception URLs.
No retry or successful-health assumption was added.

The focused suite passed **12 tests** in 6.666 seconds of test execution, with
source hashes unchanged; an independent reviewer also passed all 12. Real Python
child/grandchild cases cover cooperative and forced nested cleanup, terminal
receipts, exited leaders, and cancellation during the final health check.
Additional checks cover signal-handler restoration, spawn/budget failures, safe
diagnostics, and preservation of primary errors. Evidence:
[`integration-supervision-tests-v06.json`](../artifacts/integration-supervision-tests-v06.json)
and its [log](../artifacts/integration-supervision-tests-v06.log). These are helper
tests, not Rust mutation execution or live backend evidence.

The complete owned integration run under `artifacts/emulator-v06-complete`
subsequently **passed**, with the [top-level runner](../artifacts/verification-v06/emulator-v06-complete.json)
exiting 0 after **330.158 seconds**. Its retained evidence records:

- **2,522 collection checks passed, zero failed** in the [collection summary](../artifacts/emulator-v06-complete/summary.json).
- Both five-case snapshot baselines passed: [first](../artifacts/emulator-v06-complete/oracle-baseline-1.json)
  in 19.979 seconds and [second](../artifacts/emulator-v06-complete/oracle-baseline-2.json)
  in 18.241 seconds.
- **530 Rust tests passed across 28 suites, zero failed and zero ignored**, as
  recorded in the [workspace log](../artifacts/emulator-v06-complete/workspace-live.log).
  The [workspace supervisor](../artifacts/emulator-v06-complete/workspace-live.json)
  completed in 176.813 seconds with unforced, successful cleanup.
- **5/5 actual FalkorDB delivery cases passed**, including SQL/JSONL/property and
  provenance comparisons, zero-attempt resumed replay, and complete cleanup,
  in the [delivery verification](../artifacts/emulator-v06-complete/delivery-verification.json).
- The [outer command](../artifacts/emulator-v06-complete/mutation-command.json)
  completed with exit 0 after 182.286 seconds, with its owned process group
  drained and no forced termination.

These are successful local integration and supervision results. This run
executed no Rust mutations and establishes no mutation score; the complete 95%
mutation target remains outstanding, with the separate 2,959-candidate
enumeration retained below.

## Supporting source and migration checks

The source operation auditor was rerun against the current Rust source and the
pinned SDK modules in the local compiler cache. Its earlier delimiter error was
a real, transient missing function brace in a newly written parent test, now
fixed. The auditor still includes test modules, fails closed on unresolved empty
SDK sends, and now reports the path and exact source location for malformed
syntax. Passing a `ResourceModel` to the existing Cloud Control list builder
does not introduce a new SDK operation.

The source-only report records **73 explicit callsites, 72 unique operations,
15 service clients, and one credential-provider boundary** across 32 AWS source
files. No operation was added or removed relative to the frozen operation
matrix. It separately records the runtime descriptor dimension: 397 root types
plus 21 parent-dependent types, 965 property relationship rules, and 21 parent
relationship rules. The pure shared catalog helper validates unique and
disjoint identities, exactly one plan per child type, and root prerequisites;
the auditor binds both catalog files and the exact additive runtime catalog.
This source reconciliation does not establish successful API execution.

The auditor/catalog helper suite passed **26 tests**. An independent Python
SQLite probe also passed **36 checks** while applying the exact schema-six SQL
to temporary copies of the delivered v0.5 database. At its recorded catalog
revision, it verified the version change, a total of 418 resource views, exact
columns and execution of the 21 new views, preservation of prior objects and
data, and full rollback on a late view-name collision. That probe does not
replace the Rust SQLx migration tests or automatically certify later catalog
changes.

Supporting evidence:

- [`aws-operation-source-reconciliation-v06-source-only.json`](../artifacts/aws-operation-source-reconciliation-v06-source-only.json)
- [`operation-reconciliation-tests-v06-shared.json`](../artifacts/operation-reconciliation-tests-v06-shared.json)
- [`schema-six-manual.json`](../artifacts/review-v06/schema-six-manual.json)

The v0.4/v0.5 operation matrix and execution receipts were preserved. A new live
summary must be bound separately after the integration lane finishes; historical
SDK/emulator results do not become current just because source parsing passes.

## Historical survivors: useful diagnoses, not a current score

The v0.4 diagnostics were interrupted and their closures predate v0.5/v0.6.
Source inspection finds the following targeted test contracts already present
in the recovered v0.5 code. Only a fresh campaign can show whether the current
mutations corresponding to the old survivors are caught.

| Historical diagnosis | Current test contract to preserve and rerun |
| --- | --- |
| Root CLI returned success for partial/cancelled snapshots | `tests/cli_properties.rs`: `partial_and_cancelled_snapshots_preserve_exit_status_and_evidence` |
| AWS selector unions, optional text byte/NUL bounds, region/selector caps, Cloud Control prerequisites | `crates/aws/src/options_property_tests.rs`: literal selector mapping and independent exact-boundary properties |
| Durable queue accepted invalid payload/key/timestamp values when later SQLite constraints masked validation | `crates/engine/tests/job_validation_properties.rs`: object payload, byte boundary, epoch boundary and unchanged queue-state assertions |
| Limiter period/policy-map boundaries and rejected-hint messages | `crates/engine/tests/limiter.rs`: rational bounds, exact map cap and rejected hint contracts |
| Invalid graph names were hidden by a nonexistent scan fixture | `graph_contracts.rs`: a real frozen scan and exact validation-before-output assertions |
| Large graph metadata needed byte splitting | `graph_contracts.rs`: bounded metadata-heavy exports with exact resource totals |
| Malformed projection envelopes reached I/O or acknowledgement state | `graph_contracts.rs`: sixteen independently corrupted envelope cases plus inclusive query-size/row boundaries |
| Server rejection mistaken for an already-existing index | Required native rejected-batch test; failed batch must remain unacknowledged and scan unfinalized |

Seven of the historical graph misses concern defensive size guards or internal
batch margins in `BatchWriter::emit` / `BatchWriter::push`. Several have no
demonstrated public behavior difference in the retained diagnosis. Keep them in
the universe. Do not label them equivalent or exclude them merely to reach
95%. Reproduce the current candidate and either identify an observable contract
and test it, or retain its missed outcome in the denominator.

The highest-priority additional coverage comes from behavior introduced after
those diagnostics: bounded parent fan-out and its failure/cancellation cases,
export cleanup/publication, automatic delivery status propagation, ambiguous
graph write retry and checkpoint replay, and schema-v5 migration rollback.
The v0.5 tree contains tests for the latter groups, but none has a current full
mutation score. Avoid spending another round reimplementing the old boundary
tests without first observing the new frozen candidates and actual survivors.

Historical evidence retained unchanged:

- `artifacts/mutation/core-v04/`: complete old core partition, 56/56 caught.
- `artifacts/mutation/root-v04/`: 12 raw diagnostic outcomes.
- `artifacts/mutation/aws-v04/`: 33 raw diagnostic outcomes.
- `artifacts/mutation/engine-v04/`: 203 raw diagnostic outcomes across two shards.
- `artifacts/mutation/aws-v04-survivor-plan.json` and
  `artifacts/mutation/engine-v04/survivor-notes.json`: earlier diagnoses.

## Capacity and scheduling evidence

These measurements sum the build/test phase durations recorded for each old
non-baseline outcome. They include unsuccessful and interrupted diagnostic work;
they are local calibration data, not a throughput guarantee or an estimate for
the new source tree.

| Historical package | Outcomes sampled | Median build + test seconds | Mean build + test seconds |
| --- | ---: | ---: | ---: |
| Core | 56 | 3.298 | 4.298 |
| Engine | 203 | 10.105 | 17.235 |
| AWS | 33 | 28.710 | 40.418 |
| CLI | 12 | 29.331 | 40.798 |

Applying those per-package medians and means to the **old preliminary 2,535
candidate universe** gives approximately 16.70 and 23.98 hours of serial phase
time, respectively. This excludes baseline/setup/finalization and is not a new
candidate enumeration. The v0.5/v0.6 production source and test suites grew, so
these numbers cannot support an in-turn completion promise or a current ETA.

## Fresh v0.6 candidate enumeration

The current source was independently enumerated with pinned `cargo-mutants`
27.1.0 on November 13, 2026. The command completed in 4.914 seconds and produced
**2,959 candidates**:

| Package | Current candidates |
| --- | ---: |
| `inventory-core` | 56 |
| `inventory-engine` | 810 |
| `inventory-aws` | 1,683 |
| `aws-inventory` | 410 |
| **Total** | **2,959** |

The candidate list and source closure are retained in
[`artifacts/mutation/enumeration-v06`](../artifacts/mutation/enumeration-v06/).
The execution receipt is
[`mutation-enumeration-v06.json`](../artifacts/verification-v06/mutation-enumeration-v06.json).
The candidate JSON SHA-256 is
`0c2416c2cd6c780efa44a17946a76ffed41db91be8c900f67e8e2833ef9cd07f`.

This is an enumeration only: **zero current candidates were executed by that
command, and it establishes no mutation score**. Its diagnostic manifest uses
package-local tests without live fixture bindings. It is deliberately ineligible
for whole-project acceptance. The final campaign must freeze the stricter
`run_all.py` policy, including two baselines and all ten live cases, rather than
promote this preliminary manifest into a passing campaign. The historical
timing table above remains historical; no current runtime estimate is inferred
from the new count.

## Conditions for the next final campaign

1. Finish the intended source/test/catalog changes and run the complete ordinary
   and live baselines. Preserve new guarded test names in the fixed acceptance
   mapping when the contract expands.
2. Prepare the exact pinned mutation executable, toolchain, complete offline
   dependency cache, verified AWS Emulator, Redis server and FalkorDB module.
   An installed SDK archive collection is not proof that this compiler cache is
   ready. Do not launch competing SDK compilers on a constrained host.
3. Freeze a fresh complete closure and universe using `run_all.py launch`, with
   fresh campaign/snapshot/control paths and one owned fixture lifetime. Use a
   dedicated target outside the authoring tree and one Cargo job/worker.
4. Do not edit the frozen snapshot. Keep every outcome, timeout, resource failure,
   survivor, source-restoration receipt and backend-health receipt. An orderly
   stopped run is incomplete, and its outcomes cannot be called a final pass.
5. Investigate current survivors with behavior assertions. Any source/test/data
   fix requires a new valid frozen campaign; do not merge favorable counts across
   different closures or fixture identities, or drop a difficult candidate.
6. Only accept the current complete aggregate after its outer fixture closes,
   the final universe is independently re-enumerated, every candidate is
   resolved, and the unchanged integer threshold passes.

Reference applied while assessing bounded runtime/test contracts:
`holzman-rust/references/runtime-performance-architecture.md`. This audit does
not claim that every tool or assurance gate in that skill has been executed.
