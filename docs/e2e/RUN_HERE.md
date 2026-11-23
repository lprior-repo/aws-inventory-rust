# Running the flow

> Status: Record of an actual run on 2026-11-15, on a 2-CPU Linux x86_64
> machine with 8 GB of memory and a 6 GiB process limit, from the `main`
> archive of this repository. Emulator evidence only. Nothing here touched real AWS.

## Wave 0 checkout differences (2026-11-22)

The A0.1–A0.3 work started at commit
`e560dc72f96a52b02d1615534da8bf29621b22bc`, not the archive described below.
Kinds and relations are reviewed Rust tables under
`crates/catalog-tool/src/kind_map/`; there is no `catalog/kinds.tsv`.

Emission-aware live/pending relation compilation and its inactive-relation
report already exist in this checkout. Additional emissions and classified
kinds count as producers; replacing this with the archive's proposed
`RelationSplit` would regress those behaviors. Security groups are callable,
so the archive's expected SecurityGroup compile failure is obsolete.

The generated, ignored catalog is 19,051,875 bytes, with SHA-256
`9a420c28bfd22fb5b14a9a1c25f8bcd294eede6b8b02c2fc44e0808618c36ec8`.
It was not regenerated or committed by this change. The runtime reader and
`src/sweep_command/enabled.rs` already share the 64 MiB catalog bound.
The historical seven-check and size-guard results below are not evidence
for this checkout; only commands explicitly reported with the change were run.

With the same pinned models and generated inputs,
`cargo run --locked -p catalog-tool -j2 -- models compile --check`
reaches publication and reports `catalog differs; run catalog-tool models compile --write`.
The identical command at baseline `e560dc72` reports the same drift, so this
is pre-existing generated-output drift, not a new capability-validation failure.
The comparison used a private target directory and a temporary detached
baseline worktree, which was removed afterward.

The contextual diagnostics preserve catalog decoder errors. Invalid reviewed
paths name the kind and path; callable-plan failures name the service,
operation and actual offending member path. Fixed-input validation retains
bounded, iterative traversal and maps its internal rejection to the fixed path.

Scoped verification used `HOME=/home/lewis` and
`CARGO_TARGET_DIR=/home/lewis/.cache/cargo-target-a0c`:

- `cargo check --locked -p catalog-tool -j2`: passed.
- `cargo test --locked -p catalog-tool --lib runtime_catalog -j2`:
  37 passed, 0 failed.
- `cargo clippy --locked -p catalog-tool --lib -j2 -- -D warnings`: passed.
- Explicit changed-file formatting used `rustfmt --edition 2024 --config skip_children=true`.

Integration contracts assert all fields of the intended contextual errors:
scalar-view disagreement uses `KindContext` with path `column` and reason
`operations of this kind disagree on the item type`; a present non-string
display path uses its kind, actual path and `path does not end in a string`.
A missing relation source uses `CapabilityContext` with the supplying service,
operation and requested path. These follow A0.1 and A0.3 rather than retaining
the old bare-`Capability` assertions. The affected large integration suites
are split into bounded modules without dropping scenarios.

After integration into `wave0`, the complete requested package gates passed:

- `cargo fmt -p catalog-tool -p inventory-core -p cmdb-view -- --check`.
- `cargo test --locked -p catalog-tool -p inventory-core -p cmdb-view -j2`:
  1,297 passed across 87 suites, 0 failed, 1 existing native-FalkorDB test ignored.
- `cargo clippy --locked -p catalog-tool -p inventory-core -p cmdb-view --all-targets -j2 -- -D warnings`.

The first full test attempt exposed a missing `json!` import in IAM document
tests; the repaired document module supplies the import. The workspace-wide
error-consumer search found no other crate directly matching the compile
error; the Smithy compiler wraps it transparently. No production behavior
changed in this integration-contract repair.

No workspace suite, emulator sweep, PostgreSQL check, real-AWS call or binary
regeneration was run for this change.

## Wave 1 overlay bootstrap (2026-11-23)

The `w1/overlay` lane first merged `wave0` at `d56fa7e7`. It ran the generator
stages of `scripts/bootstrap-catalog.sh` individually, without submodule/LFS
initialization: universal, parent candidates, model report, field check,
endpoints, classes, plans, overlay, coverage, binary catalog, IAM policy and
manifest. All completed successfully with `HOME=/home/lewis`, a private
`cargo-target-w1-overlay` directory, `--locked`, narrow packages and three jobs.
The universal denominator has 1,881 schemas; parent candidates have 410 parents,
20 reviewed children and 24 implemented workflows.

The supplied plain model copy initially made binary compilation fail with
`model source /tmp/w1-overlay/.local/aws-models is not its repository root /tmp/w1-overlay`.
With coordinator approval it was replaced by a standalone local clone, not a
submodule. Its HEAD is `46135f8468cd5cb60d983b97290c9f6661e359c6`, matching
`models.lock`, and `git status --porcelain` was empty. The subsequently requested
authoritative `scripts/check-models.sh` also exited 0:
`AWS API models: 436 files at 46135f8468cd5cb60d983b97290c9f6661e359c6`.

After regeneration, `models compile --check` exited 0. The catalog was
19,051,875 bytes both before and after, with SHA-256
`15700c763c9159bb18be17cec7e4e0c86af3259d8b9be4be2d8b4677e203d84e`
in both cases; the byte comparison found zero differences. No tracked catalog,
manifest or model-lock change resulted. This checkout therefore does not
reproduce the older binary drift documented above.

A0.2's ignored `sweep_e2e` sub-gate remains **UNMET**: the previously observed
60-second pacing timeout was neither rerun nor changed. A1 proceeds under the
coordinator's explicit decision that restored compile consistency satisfies its
substantive prerequisite; this is not a claim that all A0.2 gates passed.

### Generated overlay source

This checkout keeps reviewed policies in embedded Rust tables, not the archive's
handwritten TOML files. Generated-only TOML is loaded from
`catalog/overlay/generated/<service>.toml` with bounded files and directory
entries. Embedded policies win per operation and keep their review metadata;
generated-only entries retain generated provenance even in a mixed service.
Missing generated directories preserve the embedded policy set. Generated files
must use `reviewer = "generated"`; parsing with hand origin rejects that label.
This boundary does not change deny classification or permit unreviewed calls.

The archive's `catalog/aws-relations.report` does not exist in this tree:
inactive-relation diagnostics are an explicitly requested `--pending-report`
output rather than a default bootstrap artifact.

Generator gate commands are run directly with `cargo run --locked -p
catalog-tool -j3 -- overlays generate --check` and `cargo test --locked -p
catalog-tool -j3 --test overlay_generation_safety -- --ignored`. No `moon.yml`
wiring is added because this wave explicitly excludes CI/pipeline changes.
A1.1 verification: six generated-source scenarios passed, library Clippy passed
with warnings denied, and `overlay --check` preserved the baseline report
byte-for-byte. After refreshing the ignored binary, `models compile --check`
passed. Its only changes were the 32 source-receipt bytes at one-based offsets
19,051,809–19,051,840; no runtime policy or catalog layout changed.

### Model-derived enumeration policies

Derivation rechecks classification before constructing any entry. It rejects
owner-sensitive inputs, ambiguous item collections, unresolved token pairs and
identities absent from the model. Modeled pagination is inferred by the existing
checker; generated page sizes are never written. Every proposed callable entry
passes the same operation checker used by the overlay gate.

`catalog/overlay/defer-inputs.txt` contains the sorted input-name deny list.
Deferral reasons are bounded to 1,024 UTF-8 bytes. The internal derived result is
the standard `Result<OperationOverlay, String>` rather than an equivalent custom
two-variant enum, avoiding a large-enum lint or a needless boxed allocation.

Generated TOML uses a fixed rules date (`2026-11-15`), sorted operations and
explicit basic-string escaping. Control characters, more than 4,096 operations
and output exceeding 1 MiB are refused before publication. Unresolved endpoints
defer enumeration; expansion and unclassified reads remain explicitly deferred.
Denied operations are omitted entirely.

### Generator command

`catalog-tool overlays generate [--service NAME]... [--write | --check]` reads
the sorted `catalog/overlay/generated.services` selection (or `*` for all models).
An explicit service overrides the file but cannot be combined with `--check`.
Without either mode flag the command prints its report without writing files.
Write mode uses exclusive temporary files, compares the initial byte snapshots,
atomically renames completed files and removes stale generated TOML. Check mode
requires the exact file set, file bytes and report bytes.

The optional `catalog/must-deny.txt` can add denials to the embedded deny list;
both generation and the overlay gate consume it. Absence keeps every embedded
denial. It never authorizes an operation forbidden by those embedded rules.
Generated files and initial publication snapshots are bounded to 64 MiB total;
reports are bounded to 4 MiB. Counts cover enumeration candidates only, and every
service and total must satisfy `callable + deferred = enumerate_candidates`.

A1.4 verification: 13 real-filesystem CLI/API scenarios passed; library and
binary Clippy passed with warnings denied. With an empty selection, generator
write/check and the unchanged `overlay --check` passed. Moon wiring is omitted
by the no-CI directive; the exact direct gate commands appear above.

## One command

![How a run is proven on the emulator](img/35-test-loop.png)

![Version 1: everything on one laptop](img/15-deploy-laptop.png)

```sh
scripts/e2e-baseline.sh
```

It installs the pinned runtime if it is missing, builds, and runs the ignored
`sweep_e2e` suite. The suite starts its own emulator and its own Redis with
the FalkorDB module, seeds VPCs and subnets through `aws-wire`, runs the
shipped `aws-inventory sweep`, projects into FalkorDB and compares the stored
rows and the graph with the seed.

Run here as written, with the runtime already installed: exit 0, 2 passed,
247.29 s.

Tools it needs on `PATH`: `cargo` (rustup reads `rust-toolchain.toml`), a C
toolchain, `git`, `curl`, `gpg`, `dpkg-deb`, `bsdtar`, `make`.

The baseline script now selects `--features sweep`. This enables profile
credentials and the wire collector without the legacy typed service adapters.
`--features aws` still includes the sweep and all adapters for `scan` and
Cloud Control. `inventory-aws` keeps adapters enabled by default for existing
direct callers; profile-only callers must set `default-features = false`.

The dependency regression uses `cargo metadata` to identify resolved packages
and `cargo tree -p aws-inventory -e normal` to select the active normal
dependency graph. Metadata alone unifies workspace members' default features,
including the adapters, so it cannot establish the sweep-only graph.

## The steps, as run

| Step | Command | Result here |
| --- | --- | --- |
| Toolchain | `rustup toolchain install nightly-2026-11-11` | `rustc 1.102.0-nightly (9c5648833 2026-11-10)` |
| Models | Fetch `github.com/aws/api-models-aws` at `46135f8468cd5cb60d983b97290c9f6661e359c6` into `.local/aws-models` | 436 model files, 188 MB. In a clone, `git submodule update --init --depth 1 .local/aws-models` does this |
| Core crates | `cargo check --locked -p inventory-core -p aws-wire -p inventory-engine` | Clean, no warnings |
| Harness | `cargo build --locked -p inventory-harness` | Built in 2 min 5 s |
| Emulator | `inventory-harness install .local/runtime` | Version 1.0.348; package SHA-256 `279a243a…d4a8` matched the pin; signature step passed |
| Graph runtime | `inventory-harness install-graph .local/runtime/graph` | Redis 8.10.3 built from source, `falkordb.so` 6.0.4 fetched, pinned digests matched |
| Collector and test binary | `cargo test --locked --features aws --test sweep_e2e --no-run -j1` | Built on the third attempt; see "Memory" below |
| End-to-end suite | `cargo test --locked --features aws --test sweep_e2e -j1 -- --ignored --test-threads 1` | **2 passed, 0 failed**, 258.69 s |

The two tests that passed:

- `production_sweep_persists_and_projects_real_seeded_inventory`
- `controlled_tls_403_preserves_existing_inventory`

What the suite covers, as [VERIFICATION](../VERIFICATION.md) describes it: for
the 4 callable operations, signed requests, pagination, decoding, storage in
schema 14, the `IN_VPC` edge, projection into FalkorDB, a zero-write re-sweep,
resume after a kill, and no deletion on partial or denied sweeps. Both tests
passing here means that flow works on this machine. It says nothing about any
other service, about speed, or about real AWS.

## Catalog checks

Run from the repository root with the models in place, using the built
`catalog-tool`.

| Check | Result here |
| --- | --- |
| `models report --check` | pass |
| `models fields --check` | pass |
| `models endpoints --check` | pass |
| `classes --check` | pass |
| `plans --check` | pass |
| `overlay --source .local/aws-models --root . --check` | pass |
| `models compile --check` | **fail**: `runtime capability metadata is invalid or unsupported` |

The failing check is the `binary-catalog` task of `moon ci`. Narrowed in a
scratch copy of `catalog/`:

- With every `relation` row of `catalog/kinds.tsv` removed except
  `AWS::EC2::Subnet VpcId`, the compile succeeds, and then reports that the
  committed `catalog/aws-api.catalog` differs from the result.
- Adding back one more row, `AWS::EC2::SecurityGroup VpcId`, makes it fail
  again. That kind has no callable operation.

So in this archive `catalog/kinds.tsv` is ahead of `catalog/overlay/`: 41 of
its 42 relation rows have a source or target kind other than VPC and Subnet,
the only two kinds an operation can enumerate today, and the committed binary
catalog was compiled from an earlier, consistent state. One such row was
tested; the other 40 were not tested one by one. The
sweep works because it reads that committed catalog. This could not be checked
against git history, which the archive does not carry. It is package A0 in
[A_COVERAGE.md](A_COVERAGE.md), and it blocks every other catalog change.

The error also names no service, kind or path. `RuntimeCatalogCompileError`
has a `CapabilityContext` variant that does; the bare `Capability` variant is
returned from about 60 places.

## Size guard

`scripts/check-source-size.sh` lists files through git, so it could not run in
the archive. Counting the same way by hand (`wc -l` over every `.rs` file), 5
files are over the 300-line limit:
`crates/catalog-tool/src/runtime_catalog/model_path.rs` (302),
`crates/core/src/catalog/runtime/decode/plans.rs` (305),
`crates/cmdb-ui/src/app.rs` (304),
`crates/harness/src/install/graph/installer_tests.rs` (305) and
`crates/cmdb-ui/src/worker/tests.rs` (327). On a clone in this state the
`source-size` task of `moon ci` would fail. Steps A0.1 and A0.4 fix them.

## PostgreSQL

V51 and V53 arrived after the run above, which used the current store, not
PostgreSQL. To see what running a server on this machine needs, PostgreSQL 16.15 was installed here
(`apt-get install postgresql`), and a throwaway server was started on a
private unix socket, queried with `psql` and stopped:

```sh
initdb -D <dir>/data --auth=trust -U inv
pg_ctl -D <dir>/data -o "-c listen_addresses='' -c unix_socket_directories=<dir>" -w start
psql -h <dir> -U inv -d postgres -Atc 'select version()'
pg_ctl -D <dir>/data -m fast -w stop
```

`initdb` refuses to run as root. This machine runs as root, as most CI
containers do, so the server ran as the `postgres` user through `runuser`. The
binaries are under `/usr/lib/postgresql/16/bin` and are not on `PATH`. No
project code talks to PostgreSQL yet.

## Memory

`aws-sdk-ec2` is the largest crate in the build. With two jobs `rustc` was
killed at 4.3 GB; with `-j1` it was killed at 6.0 GB against the 6 GiB limit.
The build passed with `-j1` after adding swap. Use `-j1` and allow more than
6 GB for that one crate.

That historical build used the `aws` feature, which links the typed adapters.
The `sweep` feature added in D1.1 removes those adapters from the sweep build;
the profile loader still brings STS, SSO and SSO OIDC through `aws-config`.

## Not run

- `moon ci` as a whole, `cargo test --workspace`, Clippy and the size guards.
- The `emulator_live` suites of the harness (the old `scan` path and Cloud
  Control).
- `cmdb-ui`. This machine has no display.
- Any benchmark. The 258.69 s is the wall time of the test suite on 2 CPUs in
  a debug build, not a measure of sweep speed.
- Anything against real AWS.

## Sizes

`.local/runtime` 3.8 GB (the emulator's full source clone is most of it),
`.local/aws-models` 188 MB, Cargo target directory 3.8 GB.

## Wave 0 feature split (2026-11-22)

D1.1 was measured with the checked-in Rust 1.100.0 toolchain, two Cargo jobs,
and `CARGO_TARGET_DIR=$HOME/.cache/cargo-target-d11`. These are warm-cache,
incremental debug builds, not a controlled clean-build comparison:

| Root feature | Command after `cargo test --locked -p aws-inventory` | Wall time | Peak RSS | Debug binary |
| --- | --- | --- | --- | --- |
| `aws` | `--features aws --test sweep_e2e --no-run -j2` | 151.15 s | 3,191,756 KiB | 219,932,024 bytes |
| `sweep` | `--features sweep --test sweep_e2e --no-run -j2` | 3.07 s | 954,124 KiB | 119,747,136 bytes |

Both successful builds reported zero swaps. The normal sweep dependency graph
contains only `aws-sdk-sts` 1.126.0, `aws-sdk-sso` 1.121.0 and
`aws-sdk-ssooidc` 1.123.0; no EC2 adapter is compiled by that graph.

The current sweep also consumes the pure IAM trust-document summary parser,
which was not represented in the step's original profile-only description.
It now lives in the unconditional `inventory_aws::iam_trust` module with a
dedicated error type. Consequently `inventory-core`, `serde_json` and
`percent-encoding` remain unconditional; the SDK-backed adapters do not.
All callers use the new module directly; the previous helper is removed.

The ignored sweep test executable was run with `--ignored --test-threads 1`,
`INVENTORY_EMULATOR_BINARY` pointing to the extracted 1.0.348 package and
`INVENTORY_REDIS_SERVER`/`INVENTORY_FALKOR_MODULE` pointing to the pinned graph
runtime. The downloaded package matched the SHA-256 pin recorded above.
The initial run failed both scenarios in 0.73 s with
`compiled catalog exceeds its 16 MiB bound` (exit 101). The E2E helper had a
stale 16 MiB limit while production accepts
`inventory_core::catalog::MAX_CATALOG_BYTES` (64 MiB). The helper now uses the
same constant and bounded one-byte-overread convention as production; the
generated catalog and production limit are unchanged.

Focused checks passed: `cargo fmt -p aws-inventory -p inventory-aws -p
inventory-engine -- --check`; `inventory-aws --no-default-features --lib`
(15 tests), default adapter `inventory-aws --lib` (356 tests), and the root
`sweep_dependencies` (2), `sweep_pagination_contract` (5) and
`sweep_tag_endpoint_contract` (4) suites with `--features sweep`. All test
commands used `--locked -j2`. Clippy with `--locked -j2 -- -D warnings` passed
for `aws-inventory --features sweep` and `inventory-aws --no-default-features`.

After aligning the test-local catalog limit, the exact rerun was
`cargo test --locked -p aws-inventory --features sweep --test sweep_e2e -j2
-- --ignored --test-threads 1`, with the same emulator and graph overrides.
Formatting passed. Both scenarios then failed with
`sweep CLI exceeded its 60-second deadline`: 0 passed, 2 failed, 135.34 s,
exit 101. The stale catalog bound is fixed; end-to-end passage remains
unproven because of this CLI timeout. No timeout or assertion was relaxed.
