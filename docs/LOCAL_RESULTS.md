# Verified local deployment

Date: 2026-11-12 (America/Chicago).

**Local operational setup: PASS. Strict Rust release assurance: REJECTED.**
See [the review](LOCAL_REVIEW.md) for the unresolved findings. No Rust source,
test, dependency manifest or Cargo lockfile was changed during installation.

The private source repository is
[lprior-repo/aws-inventory-rust](https://github.com/lprior-repo/aws-inventory-rust).
The source ZIP was verified against its included payload manifest before changes.
Original ZIP SHA-256:
`c4fcb7025d3100721bf417732a89ccf76a3ab2dcca217efcf15bf17bfa0ed65b`.

## Running stack

The supervised local service is `aws-inventory-local-bac9ec719363.service`.
At verification time:

- AWS Emulator 1.0.336: `https://127.0.0.1:39913`, authentication on, telemetry off.
- Redis 8.10.3 with FalkorDB 6.0.4: `redis://127.0.0.1:41299`.
- Native scan: `e3721bef-c091-4e54-ad75-c0302fb5356b`.
- Collected resources: **150**; relationships: **340**; failed scopes: **0**.
- FalkorDB graph: `aws_inventory`, **193** resource nodes including **43**
  explicitly unresolved references, plus the scan marker.

Ports and fixture identities change after restarting. Use
`scripts/local-stack.sh status` for the current values.
`scripts/local-stack.sh logs` shows startup/verification progress.

The scan is saved under
`.local/sessions/20261113T031336-2286521/inventory.sqlite`.
The durable JSONL and Cypher artifacts are under `.local/interactive-exports/`.
The graph was populated with a real `deliver` invocation, and independent
`GRAPH.QUERY` reads confirmed 150 observed resources and 340 relationships.
It remains running for interactive use; the verification-only servers stopped.

## Current verification

| Check | Observed result | Raw evidence |
| --- | --- | --- |
| AWS-enabled binary, all ordinary application examples | Build exit 0 | [build](local-evidence/build-final.log) |
| Canonical `scripts/verify.sh` | Exit 0; 308 helper tests, 520 Rust tests, 10 explicitly guarded integrations ignored in this ordinary lane | [canonical log](local-evidence/canonical.log) |
| Fresh authenticated emulator | 2,522 checks passed, 0 failed; 73 observed Rust operation boundaries | [summary](local-evidence/summary.json) |
| Frozen collector snapshot baselines | Five integration tests passed twice | [baseline 1](local-evidence/oracle-baseline-1.log), [baseline 2](local-evidence/oracle-baseline-2.log) |
| Full workspace with live fixture/graph environment | **530 passed, 0 failed, 0 ignored**, exit 0 | [workspace log](local-evidence/workspace-live.log) |
| Five exact SQL/export/FalkorDB comparisons | All five passed, including properties, provenance and cardinality | [delivery receipt](local-evidence/delivery-verification.json) |
| Checkpointed replay | All five made **0 attempts**, applied **0 batches** | [delivery receipt](local-evidence/delivery-verification.json) |
| Verification cleanup | All graph cases cleaned; workspace child drained without forced termination; outer command exit 0 | [supervisor receipt](local-evidence/mutation-command.json) |
| Interactive collector and graph | Manual EC2 scan: 62 resources/127 edges; then native snapshot delivered: 150 resources/340 edges | [native delivery](local-evidence/persistent-delivery.json), [graph reads](local-evidence/persistent-graph.jsonl) |
| Lifecycle and errors | Start/status/stop/restart; duplicate start; malformed command exits 2; no-argument CLI exits 2; write SQL rejected with exit 1 | [status](local-evidence/local-status.log), [stop](local-evidence/stop.log), [bad command](local-evidence/invalid-command.log), [SQL rejection](local-evidence/query-rejected.log) |
| Stronger async/panic lint gate | Exit 101: 31,128-byte discovery future | [strict clippy](local-evidence/strict-clippy.log) |
| Dependency audit | Exit 1: eight advisories | [audit](local-evidence/audit.json) |
| Unused dependencies | Exit 1: two candidates | [machete](local-evidence/machete.log) |

The five snapshots overlap and must not be summed into a distinct resource count:

| Snapshot | State | Collected resources | Relationships |
| --- | --- | ---: | ---: |
| Native | complete | 150 | 340 |
| Supplementary sources | partial | 11 | 0 |
| Root Cloud Control | partial | 401 | 32 |
| Parent-dependent Cloud Control | complete | 179 | 185 |
| Assumed-role IAM | complete | 13 | 22 |

Known emulator incompatibilities remain explicit failed operations and partial
scan statuses. Passing expected-gap assertions does not make those APIs succeed.

The wrapper file is named `mutate.py`, and its generic child receipt is named
`mutation-command.json`. This run executed ordinary workspace tests and exact
delivery verification, **not a mutation campaign**. No mutation score or line
coverage percentage is claimed. No implementation-bound Verus, Kani, Flux,
Loom, Miri, or coverage-guided fuzz PASS is claimed.

## Fixes made and retained evidence

- The emulator installer now falls back to `bsdtar` on Arch after package hash
  and signature verification. The patched installer completed successfully.
- `scripts/local-stack.sh` provides install/start/status/stop/cli/verify/logs
  commands with authenticated local fixtures, explicit TLS trust, isolated AWS
  settings and supervised process lifetimes.
- `.local/` is ignored. Credentials and server binaries were not pushed.
- The initial helper lane failed because the agent shell had `RUSTC_WRAPPER`;
  clean-environment execution passed all 308 helper tests. Gates were preserved.
- A concurrent-build verification attempt was rejected by hash checks. Its
  [failure log](local-evidence/rejected-concurrent-build.log) is retained, and
  it does not contribute to the passing counts above.

Exact command text, workdir and observed exit codes are in
[commands.json](local-evidence/commands.json). Selected evidence hashes are in
[SHA256SUMS](local-evidence/SHA256SUMS). The synthetic debug benchmark is a
bounded local exercise; no optimization or live AWS performance is claimed.

The manual checks cover the installation/lifecycle and selected CLI workflows.
They are not a claim that every CLI flag has been manually exercised. The
automated suite provides the separately stated workspace test evidence.
