# Portable handoff — AWS Inventory in Rust 0.6.0

This bundle contains the Rust application, design documents, pinned dependency
manifests, synthetic fixtures and verification receipts. The current release
adds parent-scoped collection and stronger graph/evidence verification.
**Universal AWS coverage and the whole-project 95% mutation target remain open.**

The completed v0.6 local run passed **530 Rust tests with zero failures and zero
ignored tests**, **2,522 emulator assertions**, and exact SQLite/JSONL/FalkorDB
verification for all five snapshots. Every graph replay made zero attempts and
cleanup succeeded. The four Python helper suites passed 308 tests. See
[the verification report](docs/VERIFICATION.md) for the receipts and precise
scope. Fresh mutation enumeration found 2,959 candidates; that enumeration did
not execute mutations or establish a score.

## What is implemented

| Area | Current behavior |
| --- | --- |
| Cloud Control | 418 reviewed regional resource types across 76 families: the frozen 397 root descriptors plus 21 parent-scoped descriptors |
| Parent workflows | API Gateway REST/V2, AppConfig, EKS, ELBv2 Listener and CloudWatch Logs children; required roots added automatically; exact models and ownership checked before `CHILD_OF` edges |
| Native collection | 32 resource/component kinds, existing Config aggregators, existing Resource Explorer views, Organizations discovery and existing role assumption |
| SQL | SQLx with SQLite schema 6; 418 source-specific provider views exposing 4,273 provider columns; transactional upgrades preserve existing observations and user objects |
| Architecture | 965 property relationship rules across 255 types plus 21 parent workflows; 265 types have at least one rule; a configured rule is distinct from an observed edge |
| Delivery | Streaming JSON/JSONL; bounded, replayable FalkorDB projection with transactional local acknowledgements and separate delivery retry |
| Execution | Shared request admission, configured/adaptive quota buckets, bounded retry/backoff/deadlines, cancellation and a fenced SQLx job queue |
| SDK availability | 437 official service SDK crates pinned to the November 12, 2026 release; archives verified; only implemented clients compiled by normal application builds |
| Verification | Five local snapshot scenarios; exact property checks in SQLite, JSONL and real FalkorDB; ten guarded Rust integration cases; strict mutation acceptance accounting |

Native and Cloud Control kinds overlap. Adding their counts would overstate
unique coverage. The new parent workflows reuse the existing Cloud Control
`ListResources` and `GetResource` operation identities.

## Build and try it

Run from this directory with Rustup and a C toolchain installed. Keep
`.cargo/config.toml`: it carries the pinned selective SDK code-generation
setting and SQLite allocation limits. The toolchain and dependencies are locked.
Use disk-backed storage for the Rust installation and Cargo target: SDK builds
can exceed a small memory-backed `/tmp` filesystem.

```sh
cargo fetch --locked
python3 scripts/generate_sdk_catalog.py --check --verify-cache
cargo build --locked --features aws -j1
cargo test --workspace --exclude inventory-sdk-catalog --all-features --locked -j1

# Synthetic local demo; no AWS calls.
cargo run --locked -- demo --db demo.sqlite
cargo run --locked -- export --db demo.sqlite --scan-id synthetic-demo-001 \
  --output inventory.json --format json --provenance all-observations

# Offline planning automatically includes AWS::EKS::Cluster as a prerequisite.
cargo run --locked -- plan --service cloud-control --cloud-control \
  --resource-type AWS::EKS::Addon --region us-east-1
cargo run --locked -- coverage --service EKS --limit 100
```

The SDK catalog's `all-services` feature is a separate, expensive build of all
437 clients. It does not implement new collectors. The embedded `coverage`
evidence deliberately retains source-bound historical root results; current
parent execution evidence is a separate artifact referenced by
`docs/VERIFICATION.md`. A recorded `not_run` in that historical matrix is not
rewritten as a current pass merely because a descriptor is now selected.

For maintainer checks, install `scripts/requirements.txt` into a Python virtual
environment and set `INVENTORY_PYTHON` before running `scripts/verify.sh`. Python
is development/fixture tooling; the deployed collector is Rust. Ordinary tests
leave ten integration cases ignored until their local servers are prepared.

## Collect, export and project

Use an existing AWS credential profile. Set `FALKORDB_URL` in the execution
environment when graph delivery is requested:

```sh
target/debug/aws-inventory scan --profile audit --region us-east-1,us-west-2 \
  --service all --cloud-control --discover-sources --db inventory.sqlite \
  --output-dir outputs --falkor --provenance all-observations --allow-partial

# A specific parent-scoped family, with prerequisites added automatically.
target/debug/aws-inventory scan --profile audit --region us-east-1 \
  --service cloud-control --cloud-control --resource-type AWS::EKS::Addon \
  --db addons.sqlite

# Reissue delivery from persisted observations without repeating AWS reads.
target/debug/aws-inventory deliver --db inventory.sqlite --scan-id SCAN_ID \
  --output-dir outputs --falkor --allow-partial
```

`--allow-partial` includes finalized partial/cancelled scans with their actual
status and failures. Running scans are never exported. Every delivery uses a
new immutable attempt directory; completed artifacts remain if a later graph
step fails. The shared `outputs/projection.sqlite` tracks acknowledged batches.
After resetting a remote graph, use a fresh checkpoint database.

Read `docs/PARENT_COLLECTIONS.md`, `docs/DELIVERY.md`, `docs/EXPORT.md` and
`docs/GRAPH_DELIVERY.md` for scope, record schemas, deadlines, retry ownership
and recovery. JSON/JSONL inventory export, ingest's `CollectionEvent` stream and
graph-export's Cypher plan are separate formats. Artifact re-import is not
implemented.

## Reproduce local integration

The complete official AWS Emulator source checkout is pinned to commit
`f515581569b98a9dcbd2e686c12f912fdce51bba`. The verified signed binary is 1.0.336.
The native graph installer pins Redis 8.10.3 and FalkorDB 6.0.4 with SHA-256
checksums. Both are local synthetic fixtures.

```sh
python3 scripts/emulator/install.py /DISK/aws-inventory-emulator
python3 scripts/emulator/install_graph.py /DISK/aws-inventory-graph
python3 -m venv /DISK/aws-inventory-checks
/DISK/aws-inventory-checks/bin/python -m pip install -r scripts/requirements.txt
```

`docs/EMULATOR_TESTING.md` contains the owned-lifetime command that starts the
servers, seeds the fixtures, runs collector baselines twice, executes all ten
live Rust cases and checks every exported graph property. The parent fixture
contains 12 roots and 161 children across all 21 new types; 101 EKS add-ons force
pagination, including an empty filtered page with a continuation token.

`docs/VERIFICATION.md` is the current result index. Historical v0.4/v0.5 results
remain in their own files. Local assertions exercise request serialization,
SDK decoding, mapping, storage and delivery. The emulator's generic Cloud
Control property store does not certify real provider deployment behavior,
regional support, IAM permission combinations, quotas or eventual consistency.
Known emulator endpoint/decoding failures remain explicit partial outcomes.

## Mutation gate and remaining work

The required whole-project score is at least 95% of viable mutations caught.
The final aggregate requires complete source-bound candidate coverage, two
clean baselines, ten named live tests, consistent frozen fixture identities and
no unresolved failures. Unit-only partitions cannot establish a complete pass;
toolchain failures and test timeouts cannot shrink the denominator or count as
caught mutations. See `docs/MUTATION_TESTING.md` and
`docs/MUTATION_READINESS_V06.md` for commands and the acceptance repairs.

The complete current mutation campaign has not passed. Historical core results
and interrupted other-package campaigns are retained as evidence of their own
scope, not reused as a current full-project score.

The 1,881-schema universe still excludes 1,463 types: 180 without both read/list
handlers, 389 needing additional parent workflows and 894 with unreviewed
collection scope. Prioritize reviewed global/home-region placement and the next
parent families; preserve exact identities and architecture evidence. Live AWS
validation and quota calibration require an authorized AWS environment. The
scheduler uses configured/adaptive limits and does not claim automatic discovery
of every service quota or hundreds of thousands of simultaneous requests.

The ZIP excludes Cargo targets/caches, server binaries, the large upstream
checkout and generated credentials. It retains pins, installers and synthetic
verification databases. `MANIFEST.sha256` covers every packaged payload file;
the packaging tool verifies ZIP CRCs and payload hashes before publication.
