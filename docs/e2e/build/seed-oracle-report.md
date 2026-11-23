# Seed-oracle repair report — 2026-11-23

## Result

**311/318 matched on the original baseline denominator**, versus the reported
181/318 baseline. The current recipes emit 321 resource oracle rows: **314 match
exactly** on kind, ID, account and region, and seven remain missing. The three
additional rows are the now-successful isolated EMR Containers IAM role, EKS
cluster and virtual cluster; all three match and are excluded from both sides
when normalizing to 318. The final SQLite database contains 988 resources.

The all-service sweep exited **2**, not success:

```text
resource_writes_scope=invocation resource_writes=1105 rows=988 scopes=2087 complete=1358 partial=0 inaccessible=19 unsupported=598 failed=112 unobserved=0 graph_pending=0 work_remaining=0 unattributed_resources=0
child_requests_skipped=21
```

A zero partial count does not mean success when failed scopes remain. All 112
failed-scope warning records, including repeated requests for distinct child
scopes, are retained in [seed-oracle-failed-scopes.tsv](seed-oracle-failed-scopes.tsv).

## Every original mismatch

[seed-oracle-classification.tsv](seed-oracle-classification.tsv) contains all
137 original rows, retaining their original line, kind, ID and scope:

| Baseline category | Rows |
|---|---:|
| No exact ID candidate | 52 |
| Scope mismatch | 24 |
| Kind mismatch candidates | 61 |

| Decision | Rows |
|---|---:|
| Recipe correction | 127 |
| Recipe plus production fix | 3 |
| Recipe plus remaining production/emulator gap | 2 |
| Remaining production gap | 5 |

Fresh runs generate different opaque IDs. The classification's `fresh_result`
is explicitly kind-level attribution, not a claim that an old opaque ID exists
in a different run. The final oracle comparison above uses full exact tuples.
Wrong-service identifier collisions were not used as ownership evidence.

## Seven remaining oracle failures

Exact identities are in [seed-oracle-missing.tsv](seed-oracle-missing.tsv).

| Owning resource | Missing | Reason |
|---|---:|---|
| CloudWatch Alarm | 2 | `DescribeAlarms` remains deliberately deferred: three result collections and evaluated-value safety are not approved. |
| Glue Table | 1 | `GetTables` remains denied as Content; no unapproved broad metadata/content collection was enabled. |
| Control Tower LandingZone | 1 | `ListLandingZones` remains denied as Mutating; pinned documentation says it creates ARNs for existing accounts. |
| Chime SDK Meetings Meeting | 1 | Pinned model has `GetMeeting(MeetingId)` but no discoverable `ListMeetings` root. |
| Timestream ScheduledQuery | 1 | Required `DescribeEndpoints` fails with emulator `AmbiguousService`; query and write share the `timestream` signing name. |
| Timestream Database | 1 | Same mandatory endpoint-discovery emulator failure. |

These are not repaired by renaming kinds, fabricating identities, removing
oracle assertions, or bypassing production endpoint discovery.

## Every aggregate-probe positive

[probe-positive-collection-diff.tsv](probe-positive-collection-diff.tsv) includes
**all 301** aggregate-probe `items N` operations, joined to owning canonical
kinds, before/after persisted kind counts, and operation outcomes. Kind presence
is not per-item identity proof. Seven positive operations previously had zero
owning-kind rows; four still do, including two non-resource bootstrap calls:

| Operation | Probe items | Before → after owning-kind rows | Explanation |
|---|---:|---:|---|
| EC2 DescribeRegions | 17 | 0 → 0 | Region-discovery bootstrap metadata, not inventory resources. |
| EMR Containers ListVirtualClusters | 1 | 0 → 1 | Isolated prerequisite names avoid the concurrent EKS/IAM seed collision. |
| Lightsail GetDomains | 2 | 0 → 2 | Narrow trusted-ARN fix accepts the modeled account-global domain ARNs. |
| STS GetCallerIdentity | 1 | 0 → 0 | Account-identity bootstrap metadata, not inventory resources. |
| Timestream InfluxDB ListDbParameterGroups | 22 | 0 → 22 | Required service ID fixes mixed empty/default ARN pages; 21 empty tag-child sources are counted/skipped. |
| Timestream Query ListScheduledQueries | 1 | 0 → 0 | Mandatory endpoint discovery fails; the direct probe does not exercise that bootstrap. |
| Timestream Write ListDatabases | 1 | 0 → 0 | Same endpoint-discovery failure. |

## Production behavior and compatibility

- Lightsail `global` is accepted only for valid Lightsail `Domain/` ARNs, not
  wrong accounts, partitions, regional domains, or other global resource types.
- Influx list and detail both identify parameter groups by required modeled
  `id`, not optional/empty ARN. No ARN alias or dual representation is retained.
- Required child sources containing exactly empty text are absent for that
  row. Skips are counted in run summaries. Whitespace and other nonempty values
  are passed unchanged; invalid ARNs still surface transport validation errors.
- Failed scope warnings retain service, operation, region and underlying error.
- Catalog identity/digest changes require a compatible catalog/database pair;
  no existing-database migration or compatibility shim was added.
- Resource Explorer's positional `Views.0` recipe is limited to controlled fresh
  state with exactly one setup-created `all-resources` view. Direct `CreateView`
  responses have an emulator timestamp defect; the recipe does not depend on a
  rejected create's side effect. `ListViews` was observed returning `items 1`.

## Actual verification and provenance

The final emulator was pinned 1.0.348, owned by this task, at port 39033. The
initial full seed pass exposed Resource Explorer's required `ViewName`; its
failed setup made no request and its probe confirmed no index/views. After
supplying the required `all-resources` name, the service was seeded into that
proven-clean service state: both steps accepted, followed by native `items 1`.
The combined final seed matrix has 354 accepted steps, zero rejected steps,
321 resources and six edges. Other services were not reseeded or mutated.

The final all-service collector then used a guaranteed-new `mktemp` SQLite
file, issued default profile/CA, and graph `accepted_final_39033`:

```sh
set -o pipefail
DATABASE=$(mktemp "$PWD/.local/oracle/accepted-final-XXXXXX.sqlite")
"$HOME/.cache/tgt-oracle/debug/aws-inventory" sweep \
  --database "$DATABASE" --catalog catalog/aws-api.catalog \
  --profile default --region us-east-1 --endpoint https://127.0.0.1:39033 \
  --ca-bundle "$AUTH/ca.pem" --graph-url redis://127.0.0.1:16497 \
  --graph-name accepted_final_39033 --drain
code=$? # observed: 2; record it even though collection has failed scopes
```

Environment and generated-prerequisite commands are in
[RUN_HERE](../RUN_HERE.md#seed-oracle-identity-repair-2026-11-23). Builds used
`CARGO_TARGET_DIR=$HOME/.cache/tgt-oracle`, `CARGO_INCREMENTAL=0`,
`RUSTC_WRAPPER=` and `-j3`.

Passed checks actually executed:

- Focused Lightsail/global-ARN and fake-transport durability regressions.
- Focused mixed Influx list/detail identity regressions.
- Two fake-transport empty/nonempty tag-source regressions, including malformed
  ARNs and whitespace preserving their exact bytes and surfacing errors.
- Required child source unit regression and skipped metric reset regression.
- Four CLI summary tests, including visible skips and failed-without-partial.
- Sweep-enabled CLI build.
- Pinned overlays generation, kinds generation and catalog compilation checks.
- Native Resource Explorer enumeration and exact final oracle/report comparison.

Earlier failed compile/fixture attempts, rejected view experiments, and the
attempt against an already-existing immutable-catalog database were retained
rather than described as passes. The final run used a new file. Actual outcomes
and logs are under `.local/oracle/run-results.tsv`; final evidence is
`accepted-final-sweep.log`, `accepted-final-classification.log`,
`accepted-final-positive-diff.log`, `accepted-final-database.txt`,
`final-oracle.tsv` and `final-seed-matrix.tsv`. The database is
`.local/oracle/accepted-final-Rnmpnx.sqlite`. Probe output is
`/tmp/seedsfix-resource-explorer-default.report`, not tracked catalog output.

Not run: formatter, Clippy, broad workspace suite, PostgreSQL E2E, formal proofs,
or new aggregate probe (the supplied aggregate report was the comparison input).
No push, CI edits, tracked JSON, or generated catalog/probe-report commit.

## Commits and exact changed paths

[seed-oracle-commits.tsv](seed-oracle-commits.tsv) lists all 113 one-service,
one-recipe commits and their exact paths. Production commits are `e645ac39`
(Lightsail/failure reporting), `3f0125ba` (Influx identities), and `dce713dc`
(empty required child sources). Documentation scaffold is `dd24132f`; this
report and tables are in the final documentation commit.

Production/test/changelog paths, in addition to the 113 recipe paths:

```text
CHANGELOG.md
crates/catalog-tool/src/overlay/builtin/family_t_z/timestream_influxdb_plans_2.rs
crates/catalog-tool/src/overlay/builtin/family_t_z/timestream_influxdb_plans_3.rs
crates/sweep/src/execution/page.rs
crates/sweep/src/execution/page/assembly.rs
crates/sweep/src/execution/page/assembly/page.rs
crates/sweep/src/execution/page/child/context_budget.rs
crates/sweep/src/execution/page/child/policy_tz_completion_tests.rs
crates/sweep/src/execution/page/child/policy_tz_completion_tests/zero_collection.rs
crates/sweep/src/execution/page/child/policy_tz_completion_tests/zero_influx.rs
crates/sweep/src/execution/page/child/policy_tz_completion_tests/zero_influx_tags.rs
crates/sweep/src/execution/page/settlement.rs
crates/sweep/src/execution/page/tag_endpoint.rs
crates/sweep/src/execution/page/tag_endpoint/children.rs
crates/sweep/src/execution/page/tag_endpoint/tests.rs
crates/sweep/src/execution/page/tag_endpoint/tests/empty_source.rs
crates/sweep/src/metrics.rs
crates/sweep/src/metrics/tests.rs
crates/sweep/src/record/trusted_arn.rs
crates/sweep/src/record/trusted_arn/tests.rs
src/sweep_command/enabled.rs
src/sweep_command/enabled/summary.rs
src/sweep_command/enabled/summary/tests.rs
```

Documentation paths:

```text
docs/e2e/RUN_HERE.md
docs/e2e/build/seed-oracle-report.md
docs/e2e/build/seed-oracle-classification.tsv
docs/e2e/build/seed-oracle-missing.tsv
docs/e2e/build/seed-oracle-failed-scopes.tsv
docs/e2e/build/probe-positive-collection-diff.tsv
docs/e2e/build/seed-oracle-commits.tsv
```
