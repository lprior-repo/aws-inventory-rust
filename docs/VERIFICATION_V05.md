# Verification and delivery boundaries — 0.5.0

This report records the November 12, 2026 v0.5 implementation and its executed
checks. The complete live Rust workspace passed **503 tests, with zero failures
and zero ignored tests**. The populated AWS Emulator baseline passed **1,188
assertions**. Four finalized SQLite snapshots were independently compared with
portable JSONL artifacts and a real local FalkorDB, including exact identities,
relationships, provenance, snapshot state and zero-command checkpoint resume.

These are local execution results. **Universal AWS resource coverage and the
project-wide 95% mutation target have not been achieved.** Unsupported operations
and providers remain failed/partial observations even when the harness correctly
asserts their failure behavior. Historical evidence is preserved in
[VERIFICATION_V04.md](VERIFICATION_V04.md), including earlier versions and the
interrupted mutation campaigns.

## Result index

The machine-readable result index is
[verification-v05.json](../artifacts/verification-v05.json). It records the
scope limits, both executable identities, exact case results and evidence hashes.

| Check | Observed result | Receipt |
| --- | --- | --- |
| Complete ordinary Rust workspace | 494 passed, 0 failed, 9 service-dependent tests ignored | [workspace-v05-retry3.json](../artifacts/workspace-v05-retry3.json) and its log |
| Complete live Rust workspace | 503 passed, 0 failed, 0 ignored, across 27 reported suites; 132.766 seconds including command overhead | [workspace-live.json](../artifacts/emulator-v05/workspace-live.json), [log](../artifacts/emulator-v05/workspace-live.log) |
| Ordinary executable/library/example build | Exit 0 with locked, offline dependencies and one Cargo job | [ordinary-production-v05.json](../artifacts/ordinary-production-v05.json) |
| Production Clippy | Exit 0 after the coverage evidence refresh; official Clippy driver in Cargo build mode, libraries/binaries/examples, `-D warnings` | [clippy-production-final-v05.json](../artifacts/clippy-production-final-v05.json) |
| Rust formatting | Exit 0 | [fmt-final-v05.json](../artifacts/fmt-final-v05.json) |
| Frozen and expanded SQLite view generation | Exit 0; checked both migration 003 and migration 005 | [generated-views-v05.json](../artifacts/generated-views-v05.json) |
| All pinned SDK archives | 437 verified; 241,799,148 archive bytes | [sdk-cache-v05.json](../artifacts/sdk-cache-v05.json) |
| Pinned schema/rule audit | 13 tests passed | [schema-test log](../artifacts/cloudcontrol-schema-tests-v05.log) |
| Operation auditor | 13 independent tests passed | [operation-reconciliation-tests-v05.json](../artifacts/operation-reconciliation-tests-v05.json) |
| Delivery verifier negative controls | 15 independent tests passed | [delivery-verifier-tests-v05.json](../artifacts/delivery-verifier-tests-v05.json) |
| Mutation gate/supervisors | 92 Python tests passed; this is not a mutation score | [gate-tests-v05.json](../artifacts/mutation/gate-tests-v05.json) |
| Fresh populated emulator baseline | 1,188 checks passed, 0 assertion failures | [summary.json](../artifacts/emulator-v05/summary.json) |
| Repeated collector snapshot oracle | Two runs, each 4 passed and 0 failed | [first receipt](../artifacts/emulator-v05/oracle-baseline-1.json), [second receipt](../artifacts/emulator-v05/oracle-baseline-2.json) |
| Exact SQLite → JSONL → FalkorDB verification | All four snapshots passed; every replay made 0 graph attempts | [delivery-verification.json](../artifacts/emulator-v05/delivery-verification.json) |
| Portable files retained in this bundle | Four JSONL exports byte-match the live-verified artifacts; native JSON contains the same record stream | [portable-export-receipts.json](../artifacts/emulator-v05/portable-export-receipts.json) |
| Final embedded coverage | Ordinary rebuild passed; 2 focused tests passed; all 1,881 CLI entries exactly match the bound matrix | [embedded-coverage-final-v05.json](../artifacts/embedded-coverage-final-v05.json) |

The ordinary and live suites overlap. Do not add their counts and present them
as distinct tests. Property-test case counts likewise are not additional named
Rust tests. The nine guarded integration cases comprise four collector snapshot
tests, two actual CLI/worker tests, and three native FalkorDB tests.

## Exact snapshot and graph results

The verifier starts from read-only SQLite queries and independently compares
all graph identities and edge endpoint/relation/source tuples,
observed/unresolved classification, scan keys and marker state. It also checks
that the graph contains no unexpected extra nodes. Expected relationships are
not inferred from an equal node count. Resource kinds and complete stored
payloads are compared in JSONL; this particular graph verifier does not compare
every projected node property.

| Snapshot | Collection status | Observed resources | Relationships | Falkor Resource nodes | Unresolved nodes | Initial batches | Resume attempts |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native service collectors | complete | 150 | 340 | 193 | 43 | 6 | 0 |
| Config / Resource Explorer | partial | 11 | 0 | 11 | 0 | 4 | 0 |
| Cloud Control catalog | partial | 389 | 32 | 414 | 25 | 6 | 0 |
| Assumed-role IAM | complete | 13 | 22 | 25 | 12 | 5 | 0 |

Resource-node counts include placeholders for referenced identities that were
not observed in that snapshot. Each graph also contains one `InventoryScan`
marker. Unresolved references remain visible; a successful projection does not
turn them into observed resources or change a partial scan to complete.

Every initial push acknowledged exactly the exported batch count. Every second
push applied zero batches, skipped all acknowledged batches and made zero
attempts. Exact graph content and deterministic inventory bytes were unchanged.
Each case deleted its owned graph afterward and verified that its key no longer
existed. The fixture owner stopped both local servers after the checks.

All-observation JSONL was compared record by record with stored SQL payloads
and independently joined structured scope/page columns. The verifier checked
header/footer identity, status, provenance, ordering, EOF, record counts,
complete file hashes, command receipts and persisted receipts. Its 15 negative
controls reject changed payload/provenance even after a maliciously consistent
file hash is recalculated, same-count wrong graph identities, failed replay
receipts and failures hidden by cleanup.

The retained files are under
[artifacts/emulator-v05/exports](../artifacts/emulator-v05/exports):
`native.jsonl`, `sources.jsonl`, `cloud-control.jsonl`, `assumed-role.jsonl` and
`native.json`. They are synthetic fixture inventories, not real AWS account
exports. Portable records preserve the application's existing metadata
redaction. JSON/JSONL export is distinct from ingest's CollectionEvent format
and from graph-export's Cypher-plan JSONL.

## AWS operation and provider coverage

The executable descriptor catalog contains **397 reviewed regional types across
76 service families**, with **4,104 provider columns** and **945 relationship
rules across 244 types**. Native collectors implement 32 resource/component
kinds. These dimensions overlap; neither their sum nor the 437 SDK dependencies
is a count of independently implemented services.

The fresh operation audit reconciles **72 explicit SDK operations at 73 send
sites**, across 15 SDK services, plus the separately identified AssumeRole
credential-provider boundary. It adds no SDK operations compared with v0.4:
the expanded provider catalog uses the same two Cloud Control methods.

| Operation outcome | Explicit operations | Provider boundaries |
| --- | ---: | ---: |
| Successful production collector/provider call | 63 | 1 |
| Successful direct SDK probe | 4 | 0 |
| Emulator operation not implemented | 2 | 0 |
| Successful HTTP response rejected by the SDK decoder | 3 | 0 |
| **Total observed identities** | **72** | **1** |

The four probes are `ec2.describe_regions`, `s3.head_bucket`,
`resourceexplorer2.get_default_view` and `resourceexplorer2.list_views`. They
prove the exercised SDK request/response path; they do not prove that a
conditional production branch executed.

The pinned emulator returns HTTP 501 for `organizations.list_accounts` and
`organizations.list_children`. Resource Explorer `get_index`, `get_view` and
`list_resources` return timestamp shapes rejected by the official Rust SDK.
Those three outcomes remain decoding failures despite HTTP 200. No emulator
response or SDK parser was patched to promote them to successful inventory.

The request ledger contains **2,067 records**, of which **1,072** are reconciled
Rust collector/provider/probe requests. Python setup traffic is excluded from
operation coverage. The final operation receipt verifies the complete identity
set, phase sets, attempts and HTTP-status histograms:
[aws-operation-source-reconciliation-v05.json](../artifacts/aws-operation-source-reconciliation-v05.json).
The preserved [historical matrix](../catalog/aws-operations.json) is identified
as v0.4 evidence; its original hashes were not rewritten to describe newer code.
See [AWS_OPERATION_RECONCILIATION.md](AWS_OPERATION_RECONCILIATION.md).

### Cloud Control: configured, attempted and read

All **397** selected types received a Rust `ListResources` attempt. **386** had
successful list and detailed read evidence. Their 389 successful `GetResource`
responses produced 389 stored resource observations; some types have multiple
fixture resources. The snapshot contains 32 relationships, 775 complete scopes
and 11 failed list scopes. Thirteen independently authored semantic fixtures
contribute 30 expected edges and 111 exact architecture/redaction assertions;
the other two observed edges come from the existing catalog fixture.

The 11 unavailable types are eight selected Bedrock types and
`AWS::S3Express::DirectoryBucket`, `AWS::S3Tables::TableBucket`, and
`AWS::Scheduler::ScheduleGroup`. Their namespaces are absent from the pinned
emulator registry. Each returned `TypeNotFoundException`; no detailed read was
invented. The complete names and bound evidence are in
[CLOUDCONTROL_EVIDENCE.md](CLOUDCONTROL_EVIDENCE.md).

Most Cloud Control providers in this emulator use a generic property store.
These runs validate the official SDK, list/read path, mapping, redaction,
SQLite writes and projection for seeded models. They do not validate the real
provider's creation rules, permissions, regional support, eventual consistency,
compound identifiers in every deployment, or complete architecture semantics.

The full pinned registry matrix has 1,881 schemas: 397 selected, 180 lacking
both required handlers, 37 with documented parent-model requirements, 373 with
additional schema-derived parent requirements, and 894 with unreviewed scope.
See [CLOUDCONTROL_COVERAGE.md](CLOUDCONTROL_COVERAGE.md). Native/global adapters
cover some excluded resource families through specialized workflows; exclusion
from this generic adapter does not imply absence from every collection source.

## Reliability behavior exercised

AWS inventory requests have one retry owner: SDK request attempts are configured
at one, and the shared executor admits each explicit retry through the same
account/region/quota-family policy. Throttling, transient failures, timeouts,
server delay hints, cancellation and attempt ceilings have deterministic and
property-based tests. The default is five total attempts, bounded at ten. Quota
policies are configured and adapt to feedback; the scanner does not automatically
know every account's Service Quotas values or promise unbounded concurrency.

FalkorDB delivery reconnects for transient retryable errors, applies per-attempt
and overall deadlines, and automatically replays only recognized exporter-made
idempotent plans. It verifies the complete plan before sending, and commits a
SQLx acknowledgement after server success. One native integration test drops the
reply **after the server committed a resource write**, then confirms retry creates
no duplicate nodes or relationships. Authentication/query failures, invalid
plans, changed checkpoints, shutdown and cleanup failures retain explicit
outcomes. See [GRAPH_DELIVERY.md](GRAPH_DELIVERY.md).

The live CLI integration now performs native collection, automatic portable
export, graph projection, offline delivery resume and explicit graph-plan
resume. The durable worker integration persists an assumed-role scan. Both ran
in the 503-test suite. Delivery failure does not mutate or discard the collected
SQLite snapshot; completed artifact receipts survive a later graph failure.

The exporter test above the query cap writes and reads 10,001 resources, checks
exact ordering and identities, incrementally verifies bytes/hash, and validates
all framing. Other tests cover partial/cancelled opt-in, running-scan rejection,
source precedence, raw observation provenance, record/byte limits, no-clobber
publication races, owned-partial cleanup and cancellation drainage.

## Defects found and corrected during this increment

- A migration test's simulated v3 database still contained v5 views. The fixture
  now removes only the actual v5 additions before downgrading its version. Real
  v1/v2/v3/v4 upgrades, preserved owner objects/data/fingerprints and conflict
  rollback all pass. Migration 003 remains unchanged; migration 005 adds 197
  views transactionally.
- A decoded-cell limit regression used excessive row crossings and could hit
  the independent wall-clock limit under parallel load. It now proves exact
  success at 100,000 cells and the specific cell-limit error immediately above
  that boundary, with fewer rows. No production limit was weakened.
- Generic `CertificateArn` suffix mapping incorrectly assumed ACM. Fourteen DMS
  paths now preserve exact generic references; an MSK token-endpoint TLS path
  points to Secrets Manager. Only explicitly reviewed ACM paths retain that
  target type. All 25 certificate-family sites are audited.
- EC2 snapshot identities now use the correct accountless ARN form. Security
  group names are not treated as IDs, and ECS VPC Lattice target groups are
  distinguished from ELB target groups.
- Delivery timeout reporting now distinguishes completed drainage from failed
  cleanup; a later graph error preserves already completed file receipts.
- Coverage refresh rejects stale source/catalog/binary/fixture evidence,
  duplicate type rows, mismatched command/scan receipts and transport successes
  without persisted detailed reads.
- The independent graph verifier now decodes native noncompact scalar replies,
  checks exact identities/content rather than counts alone, and preserves
  primary failures when cleanup also fails.

The initial failed workspace receipts and
[initial expanded emulator run](../artifacts/emulator-v05-first/summary.json)
remain in the archive. The initial emulator run passed 1,161 assertions and
failed one because the eight new Bedrock namespace gaps had not yet been
classified. The final run uses an evidence-backed gap list and the additional
DMS/MSK fixtures; the underlying type failures still remain failures.

## Source and executable provenance

The live collection used ordinary executable SHA-256
`b00a23d5dea91327a365f31ec5c58e41bb4169146f9f53fcd7c41ed126e8cc80`.
Its source closure is
`933e260caad06cbf59701c03782722d783ef2d9dc0ff79c8f7df2290e6f966fa`,
its descriptor catalog is
`7d551d1ae5d32e83dc7bf45d5ef94fca35b0cecbc510cd5a3fe75aafa73300e4`,
and its architecture fixture is
`becbfa28fca31c361c165d7533f5b4a9945c34cf7ef86c6b3a2abc65587ac5ea`.
The source receipt lists the exact 68 files it covers. The runtime source,
descriptors and fixture stayed fixed for the final owned run.

After successful execution, the capability matrix was updated with the verified
read evidence. This changes the CLI's embedded offline report data. A subsequent
ordinary rebuild, two passing focused coverage tests and exact comparison of
all 1,881 CLI entries with the refreshed matrix are recorded in
[embedded-coverage-final-v05.json](../artifacts/embedded-coverage-final-v05.json).
The rebuilt executable SHA-256 is
`bb43b453c9ae0876b708ac4571ba0b01792ea131c3dee2e121c7ba242251cf36`.
That rebuilt executable is not mislabeled as the exact earlier live-tested
binary. No collection or delivery logic changed during that evidence insertion.
The final build-mode Clippy receipt likewise identifies the refreshed matrix.

The official emulator source was fully cloned and pinned to
`f515581569b98a9dcbd2e686c12f912fdce51bba`; the signed binary is version 1.0.336.
The local graph fixture uses Redis 8.10.3 and FalkorDB 6.0.4. Rust is pinned to
`nightly-2026-11-11`, SQLx to the locked 0.9.0 resolution, and bundled SQLite to
3.51.3. The declared Rust 1.101 minimum was not separately tested on stable.
The all-SDK source and archive audits do not mean all 437 optional service crates
were compiled or exercised against endpoints.

The official Clippy driver runs in Cargo build mode because the separate
metadata-only EC2 compilation previously exceeded this host's memory. Production
library/binary/example lint results are distinct from historical v0.4 test-target
style diagnostics. No blanket lint suppression or metadata-only success claim
was introduced.

## Reproduction

See [README.md](../README.md) and [START_HERE.md](../START_HERE.md) for the pinned
build. Install `scripts/requirements.txt` into a Python virtual environment for
all schema/fixture helper checks. `scripts/verify.sh` accepts that interpreter
through `INVENTORY_PYTHON`. It includes canonical metadata checks, which need
sufficient memory for the EC2 SDK; the recorded build-mode alternative above is
explicitly distinguished.

The exact final verifier invocation is retained in
[mutation-command.json](../artifacts/emulator-v05/mutation-command.json). The
wrapper `scripts/emulator/mutate.py` supplies an owned emulator/FalkorDB lifetime,
freezes the synthetic Config recorder, repeats the four snapshot oracle tests,
and then invokes `scripts/emulator/verify_delivery.py --run-workspace-tests`.
**Despite the wrapper/receipt name, this invocation does not run a mutation
campaign.** Its command is the ordinary/live test and delivery verifier.

All fixture server endpoints are loopback, both TLS hops are verified, and
emulator-generated credentials remain outside the package. The harness uses
bounded request recording and independent expected observations. Follow
[EMULATOR_TESTING.md](EMULATOR_TESTING.md) for installer pins, reproduction paths,
source compatibility and native-server setup.

## Mutation status and remaining work

**The complete v0.5 95% mutation gate has not run or passed.** The historical
v0.4 core campaign caught 56/56 candidates. Historical engine, AWS and CLI
campaigns were interrupted diagnostics; their survivors, unresolved outcomes
and incomplete candidate sets cannot be converted into a final score. The old
2,535-candidate enumeration predates the new code and is not the current total.

The serial runner prepares one frozen source/fixture scope and dedicated target,
requires all nine live tests and two clean baselines, and rejects missing
candidates, inconsistent hashes, OOMs, infrastructure failures and timeouts as
unresolved. At least 95% of viable candidates must be caught. Its 92 helper tests
exercise those gates and failure classifications, not the application mutation
score. See [MUTATION_TESTING.md](MUTATION_TESTING.md) for continuation.

Other unproved work includes the unselected provider schemas, reviewed parent
and global/home-region workflows, every service-specific runtime resource,
all Cartography/Steampipe tables, actual-account IAM and regional validation,
quota calibration and production throughput. Architecture references do not
prove effective authorization, network reachability or observed runtime traffic.
Config/Explorer keep their own sparse/lower-case property contracts and are not
silently interpreted as CloudFormation models.

The source bundle contains 86 first-party Rust files and 45,439 physical Rust
lines across application, library, tests and examples. That count excludes SDK
sources, the upstream emulator and generated SQL/catalog data. The expanded
coverage comes from reviewed descriptors and shared collectors; source-line
count is not evidence that a resource or API works.
