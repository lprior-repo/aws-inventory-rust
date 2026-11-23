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

### Pinned-model safety evidence

The ignored safety target shares one immutable model load and generated result
across five scenarios. All five passed over all 436 pinned services: no
classifier-denied operation is emitted; a tampered `CreateVpc` promotion fails
with `OverlayPromotesDenied`; an added deny removes a previous callable entry
and still passes checking; every existing embedded callable remains identical
after merging; and regenerating every file produces identical bytes.
The three embedded STS secret denials also remain denied. This checkout has far
more embedded policies than the archive's four-callable baseline, so the
compatibility scenario checks every existing callable, not just those four.
The shared-model checker invokes the same model-policy validator as the disk
overlay gate; temporary-root CLI scenarios separately prove that gate's added
deny handling. The all-model safety target completed in 24.57 seconds.

### Thin-slice generation (A1.6)

The eight selected services generate 28 callable enumeration policies and 25
enumeration deferrals. These sum to the 53 candidates in `aws-plans.report`.
No service needs substitution. `kms/DescribeKey` is omitted; S3 operations below
bucket discovery remain deferred. Both generator and overlay write/check commands
passed. Reports remain ignored generated outputs and are not committed.

| Service | Candidates | Generated callable | Generated deferred | Merged callable enumerate, before → after |
| --- | ---: | ---: | ---: | ---: |
| cloudwatch | 9 | 3 | 6 | 8 → 8 |
| dynamodb | 9 | 5 | 4 | 6 → 6 |
| iam | 17 | 8 | 9 | 9 → 9 |
| kms | 4 | 1 | 3 | 3 → 3 |
| lambda | 5 | 5 | 0 | 4 → 4 |
| s3 | 2 | 1 | 1 | 2 → 2 |
| sns | 6 | 4 | 2 | 3 → 3 |
| sqs | 1 | 1 | 0 | 1 → 1 |
| **Total** | **53** | **28** | **25** | **36 → 36** |

The merged column describes existing reviewed enumeration policies, not just
the classifier's enumeration candidates; reviewed overrides can promote other
read operations. The complete merged overlay report is byte-identical before
and after generation, including global totals of 1,565 enumerate and 632 expand
callables. This tree already has embedded policy entries for every operation,
and those entries take precedence over generated policy, including deferrals
and denials. Generation therefore enables **zero additional merged callables**.

The three enumeration deferral reasons, in frequency order:

1. `identity: no ARN member and no modeled identifiers`: 21.
2. `items: no single list member`: 3.
3. `input Scope needs a reviewed fixed value`: 1.

#### Embedded decisions shadowing generated callables

These four generated enumerations do not survive as merged enumerations.
Their embedded stated reasons are preserved verbatim:

| Operation | Embedded decision | Stated reason |
| --- | --- | --- |
| `lambda/ListFunctions` | deny | `secret` |
| `iam/ListInstanceProfiles` | defer | InstanceProfiles.Roles embeds Role.AssumeRolePolicyDocument. Requires bounded trust-policy sanitization and reviewed profile-to-role edges matching the existing adapter before generic publication. |
| `sns/ListPhoneNumbersOptedOut` | deny | `content` |
| `sns/ListPlatformApplications` | defer | PlatformApplications exposes PlatformApplicationArn plus open Attributes map. Stable ARN and NextToken are modeled, but safe attribute-only projection/credential-return guarantees are not. Requires shared allowlisted map projection and reviewed platform-application kind before persisting this response; do not assume PlatformCredential/PlatformPrincipal cannot be returned. |

The other 24 generated callables are also overridden by existing embedded
enumeration policies. Those entries state no deny/defer reason; their reviewed
identity, pagination and other field choices win unchanged:

- cloudwatch: `ListAlarmMuteRules`, `ListDashboards`, `ListMetricStreams`.
- dynamodb: `ListBackups`, `ListBulkOperations`, `ListExports`, `ListImports`,
  `ListTables`.
- iam: `ListAccountAliases`, `ListGroups`, `ListOpenIDConnectProviders`,
  `ListRoles`, `ListSAMLProviders`, `ListServerCertificates`, `ListUsers`.
- kms: `ListKeys`.
- lambda: `ListCapacityProviders`, `ListCodeSigningConfigs`,
  `ListEventSourceMappings`, `ListLayers`.
- s3: `ListBuckets`.
- sns: `ListSubscriptions`, `ListTopics`.
- sqs: `ListQueues`.

In particular, generated Lambda policies include `ListFunctions` with identity
`FunctionArn`, but embedded `secret` denial still prevents its operation plan.
No catalog compilation was run after thin-slice generation: A2.2 kind rows and
an explicit review of these embedded decisions are required before changing
callability. The generated file alone must not be interpreted as enabling that
Lambda operation. `moon.yml` remains unchanged by directive; use the documented
plain command gates rather than adding pipeline wiring.

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

## Wave 1 kind generation

`catalog-tool kinds generate [--source DIR] [--root DIR] [--write | --check]`
derives missing operation kinds from the pinned models and checked callable
policies. The current tree uses embedded typed mappings, not `catalog/kinds.tsv`.
Generated TSV is a lower-priority input: embedded keys win, and generated input
cannot supply review metadata, relationships, bootstrap roles or unmapped defaults.
Missing generated input is allowed by compilation; the generator gate requires
the exact generated bytes and report. Generated outputs remain ignored.

Exact CloudFormation names are checked against the pinned source. Otherwise the
kind uses the model's SDK ID and resource name. Display paths require string
members. Distinct item shapes producing the same resource are deferred in
operation-name order by overlay generation and reported by kind generation.
The direct gate is `catalog-tool kinds generate --check`; no CI wiring is added.

Before PostgreSQL shortening, the local catalog contained 19,051,875 bytes,
1,864 schema tables (6 over 63 bytes; maximum 79), and 236 projected columns
(0 over 63 bytes; maximum 7). The ignored `names_runtime_catalog` scenario
recorded a 2,377 ms debug catalog load; schema/store timing requires the separate
PostgreSQL work. The pre-change catalog is an ignored local comparison fixture,
not a tracked generated binary.

A2.1 checks passed: the `kind_map`-filtered catalog-tool suite, all three new
kind CLI scenarios, the 13 existing overlay CLI scenarios, and library/binary
Clippy with warnings denied. Both generator check commands passed. Kind
generation emits zero rows and zero conflicts because existing typed mappings
already cover the checked callables. The intermediate model compile comparison
reported drift; the generated binary is rewritten and checked after A2.3/A2.2.

### PostgreSQL identifier shortening

Catalog table names and projected field-path columns fit PostgreSQL's 63-byte
limit. Longer names retain their first 54 ASCII bytes followed by `_` and the
first eight hexadecimal BLAKE3 digits of the complete derived name. Core exports
`shorten_sql_name` for physical-column and index-name construction after adding
suffixes; each consumer must check collisions in its own identifier namespace.
The pre-shortening derivation remains bounded to 128 bytes.

`kinds generate --write` also writes `catalog/aws-names.report`; `--check`
compares it without overwriting drift. The report lists each long table-name
replacement and exact total/long/distinct counts. Naming retains the SQLite
reserved prefix and keywords and also rejects PostgreSQL reserved words.
Shortening changes published identifiers and the catalog hash: regenerate the
catalog and create fresh databases rather than reopening pre-change stores.

A2.3 measurements after regeneration: 19,062,877 catalog bytes; 1,925 unique
tables, **zero over 63 bytes**, maximum 63; 236 projected columns, **zero over
63 bytes**, maximum 7. The report records 14 shortened names and 1,925 distinct
names. Core identifier/column scenarios and the naming suite passed. Debug
catalog load measured 2,378 ms versus 2,377 ms before; compile/write took 39
wall-clock seconds including Cargo startup. No comparable pre-change compile
timing was captured. Compile `--write` followed by `--check` and kind generation
`--check` both exited zero. Database create/open timing belongs to the
PostgreSQL integration lane and is not claimed here.

### Wider catalog regression checks (A2.2)

Bootstrap now generates kinds and the shortening report after overlay checking
and before compiling the catalog. In this worktree run the generator stages
individually: its plain model copy must not be bootstrapped as a submodule, and
the submodule/LFS setup stages are intentionally not run.

The compiled-catalog scenarios live under
`crates/catalog-tool/tests/cli_models_compile/kinds.rs`:

```sh
cargo test -j3 --locked -p catalog-tool --test cli_models_compile kinds:: -- --ignored --nocapture
```

The byte-comparison scenario needs the ignored pre-change
`.local/a2-before.catalog` captured before regeneration. It compares the exact
encoded four original plans without decoding obsolete long table identifiers.
The other scenarios prove every callable kind has a schema, table names are
distinct, and the eight selected services have kinds available for review.
The checked-in typed kind mappings remain authoritative; there is no
`catalog/kinds.tsv` in this tree.

The complete A2.2 gate remains held on the previously reported 60-second
request-pacing timeout in ignored `sweep_e2e` scenarios. They are not rerun or
relaxed here, and unsupported/failed scope totals are not claimed.

All four compiled-catalog scenarios passed: 2,197 callable plans have schemas,
the 1,925 table names are distinct, the four original encoded plans are
byte-identical, and all eight selected services have reviewable kinds. Manual
review found no mapping requiring an override; generation remains empty because
the existing typed mappings already cover those operations. The raw-map
accessor has root round-trip and nested-sibling boundary coverage. Both new
required-manifest-artifact scenarios and core/catalog-tool library/binary
Clippy gates passed with warnings denied.

The individual regeneration stages and every subsequent `--check` exited 0,
including `models compile --write` followed by `models compile --check`.
Compilation took 132 wall-clock seconds in this run, including Cargo startup;
there is no comparable pre-change P16 measurement. The model-only CLI also
exited 0 and reported 436 services, 19,636 operations, 8,365 read-named
operations, 2,735 readonly operations, and 2,209 root list/describe operations.
Generated catalog/report files remain ignored; only the tracked manifest hashes
are updated, including the generated kinds, names, and prior overlay artifacts.

## First-page native emulator probe (A5.2)

The PostgreSQL-independent harness command uses the sweep's exact request,
signing, status, decoding and truncation-presence path without storing rows:

```sh
cargo run --locked -j3 -p inventory-harness -- probe \
  --catalog catalog/aws-api.catalog \
  --endpoint "$EMULATOR_ENDPOINT" \
  --ca-bundle "$EMULATOR_CA_BUNDLE" \
  --region us-east-1 \
  --report catalog/emulator-probe.report
```

Set `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and, when required,
`AWS_SESSION_TOKEN` to the native emulator's credentials. The endpoint must be
an HTTPS loopback IP origin with an explicit nonzero port. All five options are
required. The CLI does not shell out to Python.

Every enumeration gets a service/operation/protocol row. Parent-bound plans are
listed as `skipped child` without a network request. Root calls run at concurrency
eight, with one attempt per operation, and never follow pagination tokens.
`items N` counts the first page; `empty`, `unsupported`, and `failed CLASS DETAIL`
remain visible. Rows are sorted and followed by separate outcome totals and
per-protocol totals. Error details are single-line, capped at 1,024 bytes; decoder
failure rows include `shape=...`, recording the failing operation and output shape
as decoder-defect evidence rather than changing overlays to hide it. Catalog and
report sizes are bounded to 64 MiB. This is not a complete inventory or a
callability-policy change.

Local behavior scenarios cover four outcomes, skipped children, stable reports,
first-page-only execution, response HTTP versions, and the real CLI without Python:

```sh
cargo test --locked -j3 -p inventory-harness --test probe
```

The opt-in native proof starts a fresh emulator, runs the thin-slice TOML seeds,
probes the complete catalog, and writes the persistent report above. It requires
a row for every thin-slice enumeration and item-bearing results in at least five
protocol families:

```sh
INVENTORY_EMULATOR_BINARY=/absolute/path/to/aws-emulator \
  cargo test --locked -j3 -p inventory-harness \
  --features native-emulator-tests --test probe_native -- --nocapture
```

The native report is evidence from one seeded emulator run, not an assertion that
all catalog operations are implemented by the emulator.

The pinned 1.0.348 native run passed: 2,134 enumeration rows, including 72
item-bearing operations, 814 empty, 485 unsupported, 184 failed and 579 skipped
children. All seven protocol families returned items, so no protocol substitute
was needed. The committed `catalog/emulator-probe.report` is the complete evidence.

Decoder defect recorded by this run: `bedrock-agentcore-control ListBrowsers`,
protocol `restJson1`, output shape `ListBrowsersResponse`, failed with
`invalid response: invalid timestamp`. The operation remains visible as
`failed decode`; no overlay was changed to remove it.

### Generator provenance and reproducibility

Formatting embedded overlay/kind/naming/class sources changes the catalog's
source fingerprint even when runtime capabilities are unchanged. Regenerate the
catalog and manifest after merging all generator changes. Two current-tree
compiles 40 seconds apart and a third with copied catalog inputs and a separate
model checkout at another path produced byte-identical catalogs:
`b1e5b7c6a0a7f053dadf567e1f75e1f2af1d16b64d9e3eac7a3409ec2c34757c`.
The prior `7a8a4452...` catalog differs only in the 32-byte embedded-source
fingerprint; both files are 19,062,877 bytes. No time or absolute input path
dependence was observed.

## PostgreSQL sweep backend

Build with `--features sweep`, then pass `--database postgres://...` or
`--database postgresql://...`. The command opens an existing `inventory` schema,
checks its catalog digest, or creates a missing schema from the supplied
catalog. `--resume` never creates a missing schema. A file path retains the
transitional SQLite behavior; new PostgreSQL schemas have no legacy unattributed
SQLite resources. `--rebuild` atomically replaces native pending projection work
with a revision-fenced full-rebuild marker.

The ignored `sweep_e2e` suite defaults to `INVENTORY_E2E_STORE=postgres`.
Set `INVENTORY_POSTGRES_URL` to the native throwaway server's administrator URL;
the harness creates and removes isolated databases beside its owned emulator
and graph processes. `INVENTORY_E2E_STORE=file` selects the compatibility mode.
`scripts/e2e-baseline.sh` runs both modes. `INVENTORY_EMULATOR_BINARY` can name an
already-installed pinned binary without changing the installation directory.

The narrow native lifecycle scenario is:

```sh
cargo test --locked -p aws-inventory --features sweep -j3 --bin aws-inventory \
  given_full_catalog_when_native_cli_store_is_created_and_resumed_then_schema_and_rebuild_round_trip \
  -- --ignored --nocapture
```

## T0 sweep deadline diagnosis (2026-11-23)

The generated catalog contains **2,197 callable plans** (1,565 enumeration,
632 expansion), not merely the older 28-operation generated subset. The
VPC/subnet regression passed the whole catalog without selecting its two
enumerations. It therefore tested thousands of unrelated operations instead
of the original account/region bootstrap plus paginated VPC/subnet contract.

On pinned emulator 1.0.348, the file-backed denial scenario's prerequisite
full sweep exceeded the unchanged 60-second CLI deadline. A `strace` run
failed after 66.32 seconds (230.114 seconds including its cold compilation).
A subsequent complete attempt trace failed after 64.57 seconds (65.616
seconds including compilation), with the CLI killed after 60.045 seconds.
It completed **340 HTTP requests**: 257 HTTP 200, 62 HTTP 501, 15 HTTP 400,
four HTTP 403 and two HTTP 404. EC2 alone received 176 calls. First-to-last
completed request took 54.690 seconds; their combined transport durations
were only 305 ms, maximum 17 ms. Same-service calls were approximately
201 ms apart, matching the configured five requests/second.

For example, IAM `ListMFADevices` returned HTTP 501 `NotImplemented` once,
classified `UnsupportedType`; `GetLoginProfile` returned HTTP 404
`NoSuchEntity`, classified `Permanent`. No transient attempt failures were
observed. The repeated EC2 VPC-encryption calls were successful parent-bound
requests, not unsupported-operation retries. The first diagnostic capture
was limited to 64 KiB; the complete count above uses the subsequent 1 MiB
capture, not that truncated output or TLS connection counts.

The diagnosis commands were the file-mode
`cargo test --locked --features sweep --test sweep_e2e -j1 controlled_tls_403
-- --ignored --test-threads 1 --nocapture`, first under
`strace -f -tt -T -e trace=write,connect,execve -s2048` and then with
`INVENTORY_E2E_TRACE=1`. All exited 101. Private target directory:
`/home/lewis/.cache/cargo-target-w2-pacing`; fixture binaries came from the
read-only primary checkout via the documented emulator/graph overrides.

Increasing the deadline cannot repair this contract mismatch: production
deliberately returns exit 2 for any unsupported or failed completed scope
(or exit 4 while work remains). No exit semantics or production AWS pacing
should be relaxed. The approved fix is explicit operation selection retaining
required ancestors, but not unrelated descendants. T0 remains **UNMET** at
this diagnosis commit; both backend acceptance runs follow the fix.

