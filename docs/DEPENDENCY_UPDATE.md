# Dependency update and current local verification

Date: 2026-11-12 (America/Chicago).

This update refreshes the collector and local verification dependencies and
repeats the complete authenticated AWS Emulator → Rust collector → SQLite →
JSON/JSONL → FalkorDB workflow. The initial import's receipts remain in
`local-evidence/`; this update has separate current evidence.

## Dependency changes

| Component | Current selection | Change |
| --- | --- | --- |
| Official AWS service catalog | 437 exact pins from `release-2026-11-12` | Refresh from official revision `c8e11bea896b54530c587c29ce30ebd6619af62a`; 22 service releases updated |
| AWS HTTP/TLS | Hyper 1 / HTTP/2 0.4.20, Rustls 0.23.46, webpki 0.103.17 | Preserve all current defaults except the legacy `rustls` alias; remove obsolete Hyper 0.14, HTTP/2 0.3, Rustls 0.21 and webpki 0.101 from the lockfile |
| SQLx | Upstream commit `10462fa4414c6c7f1ed1d39bb31678e3b726878c` | Use upstream's merged support for the patched SQLite family |
| Native SQLite | `libsqlite3-sys` 0.39.1, bundled SQLite 3.54.1 | Replace vulnerable 0.37.0 / SQLite 3.51.3; preserve native allocation/query limits |
| SHA-256 | sha2 0.11.1, base16ct 1.0.0 | Upgrade 0.10.9; adapt to the new output type while retaining lowercase, zero-padded digest strings |
| Property testing | proptest 1.12.0 | Remove the old 1.11 pin; update its random-number dependencies |
| Fixture AWS SDK | boto3/botocore 1.44.2 | Refresh the isolated local Python environment and reproducible requirement pins |
| JSON-schema support | rpds-py 2026.9.1 | Refresh the helper dependency |
| Unused direct dependencies | Removed AWS `tokio-util` and engine `tokio-stream` | `cargo machete` now reports no unused dependencies |

All 437 SDK archives were fetched and checksum-verified: 242,234,008 bytes.
The selected default features are independently compared with every archive's
manifest, preserving signing and protocol defaults while excluding the legacy
TLS alias. Two generator regression tests reject dropped SigV4a/HTTP features.
FalkorDB 6.0.4 and Redis 8.10.3 were rechecked against their latest upstream
releases and remain current. No system package or unrelated project was changed.

The SQLx commit is the [merged upstream SQLite-support change](https://github.com/transact-rs/sqlx/pull/4502),
not a local fork of SQLite or a suppressed advisory. Published SQLx 0.9.0 caps
the SQLite dependency below the patched family. Pinning the upstream commit
keeps the bundle and build limits reproducible.

"Updated" means the newest supported compatible dependency graph. SQLx's
upstream constraint supports SQLite bindings below 0.40, so 0.39.1 is selected.
Botocore requires `python-dateutil>=2.1,<3`, so 2.9.0.post0 remains selected
rather than incompatible 4.0. Transitive `crypto-common` requires exactly
generic-array 0.14.7. These constraints are recorded, not bypassed.

## Implementation and regression boundaries

Digest values, resource/scope identities, export checksums and graph checkpoint
identities retain their previous lowercase hexadecimal representation. The
SHA library no longer implements `LowerHex` on its output, so call sites now
use RustCrypto's borrowed `HexDisplay` adapter. Existing value assertions were
preserved; the relevant test helpers were adapted to the same library API.
The live delivery verifier independently uses Python's hashlib for artifact
integrity and compares complete payloads with SQLite and real graph contents.

Two new production-API integration tests verify the SQLite runtime version and
all four required native compile options. They query the library linked into
`Store`, not the machine's separate `sqlite3` executable.

The discovery shell now boxes its large child future while retaining ownership
and awaiting it in the same task. This addresses the previous 31,128-byte
future diagnostic without introducing a detached worker or changing discovery
results or cancellation ownership.
The graph CLI's large delivery future uses the same ownership pattern.
Cancellation now records a secondary target-preparation error instead of
discarding the fallible result.

SQLite 3.54 reports `SQLITE_TOOBIG` for an oversized `printf` result that the
old library returned as NULL. The adapter now maps native code 18 to the exact
typed `StoreError::Limit("SQLite value bytes")`; the store contract and regression
assertion explicitly require that outcome. Other SQLx errors retain their
original classification. Existing row, byte and execution-work limits remain
covered by the same scenario.

The existing `.local/` databases and prior evidence were retained. The new
interactive stack will use a fresh authenticated fixture session and a fresh
graph checkpoint destination.

The canonical validation script now builds its final binary and examples with
the same workspace feature graph used by its tests. The separate all-services
build uses `scripts/local-stack.sh sdk-build`, two Cargo jobs and a 32 GiB memory
cap in `.local/all-sdk-target`. It does not replace the collector's binary while
its snapshot verification is running.

## Whole-codebase refactor

The user explicitly authorized refactoring the whole imported codebase. Every
first-party Rust file, including examples, generated catalog code, integration
tests and property suites, is now at most 300 lines. A permanent
`scripts/check-source-size.sh` gate runs before canonical validation. Modules
separate catalog validation, identity mapping, service collection, persistence,
query policy, retry admission, graph planning, delivery and CLI supervision.
The generator now produces alphabetic SDK facade modules, preserving the
original public reexports and exact service pins. Internal catalog letter
features follow the corresponding service features.

DDD changes preserve the serialized and stored formats:

- Job completion accepts only `TerminalJobState`, so queued/running completion
  requests cannot be constructed. Result/error diagnostics retain their existing
  independently optional semantics, including failed jobs with result payloads.
- Persisted scan rows parse into running/finalized variants with the timestamps
  required for that state. Corrupt combinations produce typed conflicts before
  downstream writes; two new Store-API tests inject corrupt SQLite rows.
- Worker intake uses `Intake::Accepting` or `Stopped(StopReason)` and cancellation
  decisions use an enum. Existing output strings and cancellation precedence
  remain unchanged.
- IAM enrichment uses listed/fetched observation variants rather than a boolean
  control argument; export properties retain their original representation.

The collection-page trust boundary remains private to the store. Preparation
permits stay alive through atomic persistence; graph acknowledgements still
follow confirmed server responses. No source-size exceptions, include fragments,
ignored tests, or lint waivers were added. The ten existing live tests keep their
guards for ordinary runs and are explicitly enabled in the local live workflow.
Their exact moved names were also updated in the mutation acceptance contract.

## Acceptance evidence

Current command outcomes, exact test counts, running endpoints and evidence
hashes are recorded below after the complete verification run finishes.

No mutation score, universal AWS provider support, live-account IAM validation,
or formal proof is inferred from this local integration run. The strict black-hat review retains its independent verdict; functional checks
do not establish compliance with every engineering-style constraint.

### Verified baseline before the isolated local GPU candidate

The following receipts describe binary SHA-256
`8fe9df4f994c84de546cf3f2c1614895f17ff9cab0fdfaa3e533aef02bf867e8`.
The subsequent local GPU candidate is isolated and is not accepted by these
receipts. Its integration requires fresh behavior and lint results.

| Gate | Observed result | Receipt |
| --- | --- | --- |
| Canonical validation | 526 Rust tests passed; ten guarded live tests reserved for explicit local execution; 310 helper tests passed | `dependency-evidence/pre-local-gpu/canonical-complete.log` |
| Explicit live Rust workspace | 536 passed, zero failed, zero ignored | `dependency-evidence/pre-local-gpu/workspace-live.log` |
| Native emulator behavior | 2,522 assertions passed, zero failed; 73 Rust operation boundaries observed | `dependency-evidence/pre-local-gpu/live.log` |
| SQLite/export/FalkorDB | All five cases matched exact identities, properties, provenance and cardinality; each resume made zero write attempts | `dependency-evidence/pre-local-gpu/delivery-verification.json` |
| All SDK clients | All 437 service SDKs compiled together; catalog rebuilt successfully after generated-module refactor | `dependency-evidence/pre-local-gpu/all-sdks-final.log`, `all-sdks-refactored.log` |
| Extended source Clippy | Passed with warnings and explicit panic, indexing, arithmetic, ignored-result and large-future diagnostics denied | `dependency-evidence/pre-local-gpu/strict.log` |
| Dependency audit | Zero vulnerabilities and zero warnings; no advisory suppression | `dependency-evidence/pre-local-gpu/audit.json` |
| Unused dependencies | None reported | `dependency-evidence/pre-local-gpu/machete.log` |
| Manual invocation QA | 120 captured invocations; documented incorrect initial expectations retained and corrected; no unresolved functional failure in this scope | `dependency-evidence/manual-qa/REPORT.md`, `results.json`, `runs/` |
| Red Queen bounded CLI contract | Three generations, three zero-survivor generations, all 27 permanent checks and lineage replay passed | `dependency-evidence/red-queen/summary.json` |
| Strict whole-project engineering review | **REJECTED**: function-size/parameter, domain-typing and async assurance requirements remain open | `dependency-evidence/BLACK_HAT_REVIEW.md` |

The interactive scan persisted 150 observed resources, 340 edges and 131 complete
scopes with no failures. Direct graph reads found 193 Resource nodes, including
43 explicitly unresolved references, and 340 edges. Replaying its saved graph
plan skipped all six batches and attempted zero writes. SQLite, exports and old
receipts remain available under the ignored `.local/` directory.

A failed first canonical invocation inherited the machine's `sccache` wrapper;
the mutation-assurance helper correctly rejected that unexpected override. The
successful canonical run cleared compiler wrapper variables explicitly. No gate
was disabled. QA also retained and corrected initial assumptions that partial
and cancelled results exit zero: the documented CLI contract returns exit 2.

The source review is not owner-approved debt or a passing global gate. File-size
compliance and operational success do not imply that all strict skill constraints,
mutation coverage, instrumentation, formal proof, or real-account behavior have
been established.
