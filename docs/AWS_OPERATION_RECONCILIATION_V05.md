# AWS operation reconciliation for v0.5

The preserved v0.4 operation matrix inventories **72 explicit SDK operations at
73 Rust send sites**, across 15 SDK services. One additional `sts.assume_role`
boundary belongs to `aws-config`'s credential provider and is counted separately.
The v0.5 source audit finds the same boundaries: zero added and zero removed.

Cloud Control breadth has changed from the historical 200 descriptors to 397,
with 945 declared relationship rules. These descriptors still use
`cloudcontrol.list_resources` and `cloudcontrol.get_resource`. Type and
relationship counts do not increase the number of SDK operations, and neither
count establishes successful execution against AWS or the emulator.

## Reproduce the source audit

From the repository root, with the pinned SDK sources present in Cargo's cache:

```bash
python3 scripts/catalog/reconcile_aws_operations.py
```

The output is `artifacts/aws-operation-source-reconciliation-v05.json`.
`catalog/aws-operations.json` and
`artifacts/aws-operation-source-reconciliation-v04.json` remain unchanged and
are identified by SHA-256 in the new receipt.

The auditor masks Rust comments, string literals, raw strings and character
literals while preserving line positions. It enumerates every empty-argument
`.send()` under `crates/aws/src`, follows the direct builder chain, resolves its
SDK client namespace, and compares the complete path/function/operation
multiset with the reviewed matrix. Nonempty channel sends are excluded. New or
unrecognized call shapes fail closed rather than inheriting a nearby operation.
The auditor also verifies every pinned SDK operation module and the credential
provider module against `Cargo.lock` and their recorded hashes.

Source and catalog hashes are recorded separately from line numbers. The two
Cloud Control send sites moved from lines 275/287 to 304/316; the operation
identities remain unchanged. Source snapshots are checked before and after the
audit. File, byte, token, call and request-ledger bounds prevent unconstrained
loading. This is a lexical source audit, not Rust type checking, an AST proof,
branch coverage, a Cargo build, or live execution evidence.

## Bind a completed v0.5 emulator run

The source-only receipt starts with `emulator_evidence.status = "pending"`.
After the new harness produces a complete asserted run, refresh it with:

```bash
python3 scripts/catalog/reconcile_aws_operations.py \
  --emulator-summary artifacts/emulator-v05/summary.json
```

Use the actual completed run directory. The auditor requires the harness's
recorded source closure, raw and canonical catalog hashes, unchanged binary,
architecture fixture hash, and bound catalog/Cloud Control command receipts.
It then checks every operation identity, the credential-provider distinction,
and the operation attempt/status histograms against the bounded Rust request
ledger. Fixture creation traffic is excluded from those histograms.

The receipt retains the harness's separate collector, direct-probe, unsupported
and SDK-decoding outcomes. Binding a run does not convert these outcomes into
an all-endpoints-passed claim. Detailed per-type persisted Cloud Control reads
remain the responsibility of the separately asserted capability matrix.

## Completed v0.5 execution binding

The final run in `artifacts/emulator-v05` is now bound in
`artifacts/aws-operation-source-reconciliation-v05.json`. The nested
`emulator_evidence.status` is `current_execution_receipts_bound`, with
`current_run_bound = true`.
All 1,188 baseline assertions passed. The request ledger contains 2,067 total
records; 1,072 requests belong to the reconciled Rust collector, provider and
direct-probe phases. All 73 operation identities were observed.

| Recorded outcome | Explicit SDK operations | Separate provider boundaries |
| --- | ---: | ---: |
| Collector success | 63 | 1 (`sts.assume_role`) |
| Direct SDK probe success | 4 | 0 |
| Emulator operation not implemented | 2 | 0 |
| SDK response decoding failure | 3 | 0 |
| Total | 72 | 1 |

The successful direct probes were `ec2.describe_regions`, `s3.head_bucket`,
`resourceexplorer2.get_default_view`, and `resourceexplorer2.list_views`.
`organizations.list_accounts` and `organizations.list_children` returned HTTP
501. `resourceexplorer2.get_index`, `resourceexplorer2.get_view`, and
`resourceexplorer2.list_resources` received HTTP 200 responses that the Rust SDK
rejected while decoding. These outcomes remain failures or probe evidence in
the receipt; they do not become collector successes.

Cloud Control issued 397 list requests: 386 returned HTTP 200 and 11 returned
HTTP 400. Its 389 get requests returned HTTP 200. The per-type capability
evidence separately reconciles these responses with persisted detailed reads.

The binding verifies the 68-file source closure, raw and canonical descriptor
catalog hashes, architecture fixture hash, current executable bytes, the
catalog command receipt, and the Cloud Control command receipt and scan ID.
Every recorded operation's request count, HTTP-status histogram, phase set,
service/API identity and credential-provider classification matches the bounded
Rust request ledger. The harness's embedded-catalog, unchanged-source,
unchanged-binary and unchanged-fixture assertions all passed.

| Bound input | SHA-256 |
| --- | --- |
| Collector executable | `b00a23d5dea91327a365f31ec5c58e41bb4169146f9f53fcd7c41ed126e8cc80` |
| Source closure | `933e260caad06cbf59701c03782722d783ef2d9dc0ff79c8f7df2290e6f966fa` |
| Descriptor catalog | `7d551d1ae5d32e83dc7bf45d5ef94fca35b0cecbc510cd5a3fe75aafa73300e4` |
| Architecture fixture | `becbfa28fca31c361c165d7533f5b4a9945c34cf7ef86c6b3a2abc65587ac5ea` |

The historical matrix and v0.4 reconciliation retain their original hashes.
This execution used the local emulator and native FalkorDB; it does not establish
live AWS behavior, production quotas, exhaustive conditional-branch coverage or
a completed mutation score.

## Auditor tests

```bash
python3 -m unittest discover -s scripts/catalog \
  -p test_operation_reconciliation.py -v
```

The tests cover misleading comments/strings, nested call arguments, client
namespace changes, channel sends, unsupported call shapes, lexical bounds,
stale source/catalog evidence, provider relabeling, missing operation identities,
fixture-traffic exclusion, and inflated transport histograms. Their result is
recorded in `artifacts/operation-reconciliation-tests-v05.json`. They validate
the auditor; they are not AWS operation execution or mutation-score evidence.
