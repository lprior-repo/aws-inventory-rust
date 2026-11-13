# Rust development harness

The collector, storage, export and graph delivery are Rust. The remaining
Python is development tooling: fixture seeding, independent assertions,
snapshot freezing and the mutation gate. This document tracks the move of that
tooling into `crates/harness`, which ships as the `inventory-harness` binary and
is never linked into the collector.

Python is retained as the parity oracle until each Rust slice reproduces it
byte-for-byte. Deleting a `.py` file before its replacement passes parity would
discard the only independent check that the replacement is faithful.

## Commands

```sh
# Freeze an oracle from an asserted baseline; refuses any provenance break.
cargo run -p inventory-harness -- snapshot freeze \
  --evidence .local/sessions/SESSION --output frozen.json \
  --binary target/debug/aws-inventory

# Fetch, verify and extract the pinned emulator package.
cargo run -p inventory-harness -- install .local/runtime
```

`--root` overrides the workspace root that source fingerprints and pinned
catalog digests are resolved against.

## Status

| Slice | Python source | Rust | Evidence |
| --- | --- | --- | --- |
| Fixture fingerprints, canonical digests, runtime catalog overlay | `scripts/emulator/fixture_provenance.py` (83) | `src/provenance.rs` | `tests/provenance.rs`; digest vectors cross-checked against `hashlib` |
| Oracle snapshot freeze | `scripts/emulator/snapshot.py` (251) | `src/snapshot/` | `tests/snapshot/`; the 2,122,576-byte byte-identical claim was recorded against session `20261113T053127-3186412`, which is absent from this checkout — historical, unverified here |
| Pinned emulator installer | `scripts/emulator/install.py` (90) | `src/install.rs`, `src/install/signature.rs` | `tests/install.rs`; GPG status parsing, early refusal of incomplete pins |
| Emulator process ownership and environment | `scripts/emulator/run.py` start/stop portion | `src/emulator.rs` | spawns the pinned binary on an ephemeral loopback port, discovers the endpoint from its JSON log, derives credentials/CA, exports the collector environment |
| S3 fixture seeding | `scripts/emulator/seed_*.py` (S3 portion) | `src/seed.rs` | live test seeds through the real SDK and confirms the emulator lists the bucket back |
| Catalog reproduction | `catalog/upstream/*.py` (1,275) | `src/catalog/` | `tests/catalog.rs` — 7 tests; root catalog, parent overlay and upstream manifest byte-identical; capability matrix matches after reversing only the documented emulator-evidence merge |
| Operation/coverage audit | `scripts/catalog/*.py` (1,321) | `src/audit/` | `tests/audit.rs` — 10 tests; four negative controls with Python-identical messages; refreshed coverage document byte-identical |
| Packaging and SDK catalog reproduction | `crates/catalog-tool` (`package` subcommand; retired Python packager), `scripts/generate_sdk_catalog.py` (458) | `src/package/` | `tests/package.rs` — 7 tests; 437-service regeneration byte-identical, ZIP entries and CRC equal to the retired Python packager, network refresh refuses with a typed error |
| Asserters (Cloud Control) | `scripts/emulator/assert_cloud_control.py` (142) | `src/assertions/cloud_control*` | `tests/assertions.rs`; the 111-check identical claim was recorded against session `20261113T053127-3186412`, which is absent from this checkout — historical, unverified here |
| Request recorder and remaining seeders | `scripts/emulator/{transport,seed_*}.py` | not started | — |
| Remaining asserters (regional, global, parents) | `scripts/emulator/assert_*.py` | not started | — |
| Delivery verification | `scripts/emulator/verify_delivery.py` (477) | not started | — |
| Mutation gate | `scripts/mutation/*.py` (3,688) | not started | — |

### Live coverage against the installed emulator

The pinned emulator is installed by this harness in `.local/runtime` and every
`emulator_live` test runs without Python:

| Check | Result |
| --- | --- |
| Install | clone at `f5155815`, 175 MB package, GPG signature verified, SHA-256 `e2c621ec…97f1c` |
| Native services | `complete`: 25 scopes, 27 API calls, 0 failures on a fresh emulator |
| Reviewed Cloud Control types (retained v0.6 baseline, 418 types) | 407 `complete`; 11 `TypeNotFoundException` — 8 Bedrock, `S3Express::DirectoryBucket`, `S3Tables::TableBucket`, `Scheduler::ScheduleGroup` |
| Universal Cloud Control (`--resource-type all`) | `complete` in 8 s: 1,291 attempted, 999 succeeded, 292 provider-unsupported, **0 failed**, 590 explicit skips (410 parent + 180 unsupported handlers) |
| Native + universal in one scan (`--service all,cloud-control --cloud-control --resource-type all`) | `complete` in 8 s: 10 resources, same universal partition, native scopes alongside |
| Cloud Control create → poll → list | created `AWS::SNS::Topic`, 16 status polls, listed and then pulled by the collector as one observed resource |
| Cloud Control pagination | six seeded `AWS::SNS::Topic` resources at `--page-size 2 --max-pages 10`: the list scope spans at least two pages, every page but the last carries a non-null `next_token` |
| Seeded S3 | scan reports the bucket the test created through the SDK |
| Embedded mode | unit test boots the in-process emulator on a 64 MiB stack, scans the seeded bucket and round-trips a request |

The universal run reports `attempted == succeeded + unsupported + failed`
(1,291 = 999 + 292 + 0) and `total == list_capable + skipped` (1,881 = 1,291 +
590). Skips are named per type in the scan JSON, not omitted.

Those numbers are pinned in the test as literals rather than re-derived from
the catalog the scan was generated from, so a scan that stops early cannot pass
by agreeing with itself: 1,881 total, 1,291 list-capable, 410 parent-required
skips, 180 unsupported-handler skips, 1,291 attempted, 999 succeeded, 292
provider-unsupported, 0 failed, `/api/in_flight == 0`.

The 292 `TypeNotFoundException` names are compared as an exact set against
`crates/harness/fixtures/universal-unsupported-aws-emulator-1.0.336.json`, which
records the pinned emulator version and commit. A newer emulator is new
evidence: the test fails until the fixture and the literals are refreshed
together after verifying the delta.

The 11 reviewed-set gaps are not collector defects: they are the emulator's
Cloud Control type registry, and they match the committed `list_error` set in
`catalog/cloudcontrol-capabilities.json` exactly. The coverage test asserts set
equality against that matrix, so a gap cannot appear silently, and a newer
emulator that supports more types fails the test until the evidence is
refreshed. A missing type is a provider boundary, not a scan failure: it is
persisted as `unsupported_type`, the scan stays `complete`, and only real
authorization/decode/transport errors produce `status: partial` with exit 2.

Pagination evidence comes from the Cloud Control list scope, not from the
per-resource enrichment scopes: six seeded SNS topics at page size 2 must span
at least two list pages, with a non-null `next_token` on every page except the
last.

The live gate needs the collector rebuilt with the AWS feature (otherwise scans
fail with "collectors are a compile-time feature"):

```sh
cargo build --bin aws-inventory --features aws
cargo test -p inventory-harness --test emulator_live -- --ignored --test-threads=1
```

Six tests run serially against one owned emulator. Fixture seeds come from the
harness binary:

```sh
cargo run -p inventory-harness -- seed s3 --bucket rust-fixture-bucket
cargo run -p inventory-harness -- seed cloud-control \
  --prefix inventory-page-topic --count 6
```


## Parity method

Every slice is accepted the same way:

1. Run the Python tool against the real input and capture its output.
2. Run the Rust tool against the identical input.
3. Compare bytes. `cmp` for single documents; sorted `jq -S` where the Python
   emits unordered JSON; per-entry CRC and byte comparison for archives.
4. Prove rejection, not just acceptance: tamper one input (binary digest,
   source file, receipt hash, scope flag, catalog row) and require a specific
   failure with no output published.

The frozen-oracle parity command recorded for that historical session (the
session is absent from this checkout, so the comparison is not reproducible
here):

```sh
C=/path/to/checkout
S=$C/.local/sessions/20261113T053127-3186412
python3 $C/scripts/emulator/snapshot.py --evidence "$S" \
  --output /tmp/py-snapshot.json --binary "$C/target/debug/aws-inventory"
target/debug/inventory-harness snapshot freeze --evidence "$S" \
  --output /tmp/rs-snapshot.json --binary "$C/target/debug/aws-inventory" --root "$C"
cmp /tmp/py-snapshot.json /tmp/rs-snapshot.json
```

## Embedded emulator seam

`harness/embedded` is a **separate workspace** containing the in-process
emulator facade. It is deliberately excluded from the main workspace: the
embedded crate links the complete service router, and AWS documents a first
build with a `target/` footprint on the order of 100 GB. Ordinary
`cargo test --workspace` must never pay that cost.

```sh
cd harness/embedded && cargo test        # explicit, heavy build
```

Pinned to `f515581569b98a9dcbd2e686c12f912fdce51bba` — the same commit as
`scripts/emulator/upstream.json` (package 1.0.336), so the in-process and
standalone fixtures cannot drift apart. Newer releases exist upstream; a patch
release may change behavior, so the pin moves only with a re-run of both
harnesses.

Constraints that shaped the test:

- The facade is constructed on a 64 MiB thread and driven by workers with
  16 MiB stacks. Building the router on a default test thread overflows the
  stack and aborts — this is upstream's own documented pattern, not a tuning
  choice.
- `EmbeddedOptions` enables signature verification explicitly and turns
  telemetry off. Telemetry is process-wide: one instance disabling it affects
  every instance in the process.
- Each instance owns separate resource state, but credential keys are minted
  once per process, so auth-enabled instances accept the same keys.
- Direct SDK calls use no socket and no TLS. Operations that return listener
  URLs (signed URLs) still point at a real listener; fetching one requires the
  `certificate_authority()` roots.
- Server mode remains the primary fixture path. Embedded mode is an additional
  seam for per-test isolation, not a replacement for the signed standalone
  emulator that produces the frozen oracle.

## Constraints

- Workspace lints apply: no `unwrap`/`expect`/`panic`/`todo`/`unimplemented`,
  no slice indexing, no `as` conversions, no unchecked arithmetic, no `unsafe`.
- Every Rust file stays at or below 300 lines (`scripts/check-source-size.sh`).
- Canonical digests keep the Python escaping rule (`ensure_ascii=True`), so
  previously recorded receipt hashes remain valid after the migration.
- The pinned emulator package is Linux amd64; the installer refuses other
  platforms rather than guessing a substitute artefact.
