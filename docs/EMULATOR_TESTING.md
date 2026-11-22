# Testing the Rust collector with AWS Emulator

> Status: Kept and extended. The pinned emulator stays the proof environment
> (VISION section 9); Python seeding and recording are replaced by the Rust
> seeder and deleted (ROADMAP M2.9, M2.10), the Cloud Control scans and
> assertions go in M9.5, and the v0.4 and v0.5 sections are historical evidence.
> Links into `artifacts/` point to an ignored local directory and do not resolve
> in a clone. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

The integration harness starts a fresh, authenticated local [AWS Emulator][emulator], creates disposable fixtures, runs the **shipped Rust CLI**, and saves request, collector, and SQLite evidence. Python provides fixture setup and transport recording. It does not replace Rust collection.

The current executable catalog contains **421 Cloud Control types**: **397 frozen root types and 24 reviewed parent-dependent types**. The harness selects those groups in separate scans and compares the compiled CLI's combined catalog with both source manifests. SQLite schema version 13 exposes 421 provider views and 4,309 selected columns. The parent workflow contract is in [PARENT_COLLECTIONS.md](PARENT_COLLECTIONS.md); selection and mapping semantics are in [CLOUDCONTROL_COVERAGE.md](CLOUDCONTROL_COVERAGE.md).

The retained **v0.5** run attempted all 397 root types and persisted detailed reads for 386 types, with eleven unsupported list scopes retained as failures. Its source-bound evidence remains in [CLOUDCONTROL_EVIDENCE.md](CLOUDCONTROL_EVIDENCE.md). The historical v0.5/v0.4 sections below preserve those results; neither is evidence that the new parent collector or current binary has passed. Current command outcomes belong in [VERIFICATION.md](VERIFICATION.md).

The test has two separate coverage dimensions:

| Dimension | Current inventory | Evidence required |
| --- | ---: | --- |
| Explicit Rust collector/discovery SDK operations | 72 distinct operations at 73 production callsites | Recorded operation, phase, HTTP result and collector outcome |
| SDK credential-provider boundary | 1: STS `AssumeRole` | Observed refreshable-provider traffic and subsequent identity |
| Cloud Control root resource types | 397 frozen entries using `ListResources` and `GetResource` | Per-type list/read results and stored resource observations |
| Parent-dependent Cloud Control types | 24 additional entries using the same two operations | Prerequisite discovery, exact request models, independent pagination, detailed child rows and parent edges |

The resource types exercise different providers through the same two Cloud Control operations; the parent workflows introduce no additional SDK operation identities. SDK dependencies also do not establish collector coverage. The authoritative operation inventory is [catalog/aws-operations.toml](../catalog/aws-operations.toml), explained in [AWS_CALL_MATRIX.md](AWS_CALL_MATRIX.md). Type inventories are [cloudcontrol-resources.json](../catalog/cloudcontrol-resources.json) and [cloudcontrol-parent-resources.toml](../catalog/cloudcontrol-parent-resources.toml); the full capability audit covers all 1,881 retained schemas.

## Pinned upstream source and binary

The current source and binary pin is the latest stable upstream release,
v1.0.348. Its downloaded package checksum and detached AWS signature were
verified during the dependency upgrade. The coordinator's standalone live
gate passes all six scenarios; see `HARNESS.md`
for its command and scope. The separate authenticated embedded gate also passes
both scenarios; its command and compatibility constraint are in the same guide.

| Item | Value |
| --- | --- |
| Repository | `https://github.com/aws/aws-emulator.git` |
| Commit | `7461ab1bbe540afe8063ce03ee5477ebb649df48` |
| Release | `1.0.348`, released `2026-11-13` |
| Package | `aws-emulator_1.0.348-1_amd64.deb` |
| Package SHA-256 | `279a243ac342922d0d0741e31e2912ddb9a78f6efac34ec25c94086df029d4a8` |
| AWS publishing primary-key fingerprint | `98BCF336A3BE53B734E967C52E5BF49BD940F624` |

`crates/harness/assets/upstream.toml` records the
exact download URL, signing-key URL, commit and hash. The current installer is
`cargo run -p inventory-harness -- install <destination>`; it verifies the
source pin, package hash, detached GPG signature and publishing-key fingerprint.
The Rust runner checks the binary's reported version and commit before startup.

The retained package verification
and checkout verification are
historical 1.0.336 evidence, not proof for the current pin. Legacy Python runner
and snapshot commands below retain that older pin and must not be used to claim
1.0.348 coverage. The superseded
`crates/harness/fixtures/universal-unsupported-aws-emulator-1.0.336.json`
is no longer in the tree; it remains recoverable from git history.

A separate authenticated CLI capture on 2026-11-13 against version 1.0.348,
commit `7461ab1bbe540afe8063ce03ee5477ebb649df48`, produced
`crates/harness/fixtures/universal-unsupported-aws-emulator-1.0.348.toml`.
Its 292 names were extracted from the new SQLite database and exactly match the
older set; the complete partition also remains unchanged (1,291 attempted,
999 succeeded, 292 unsupported, 0 failed). The new fixture includes capture
provenance and binary/report/database SHA-256 values. See
`HARNESS.md` for the
command and isolation details. This manual capture alone does not establish
full-suite or embedded-mode acceptance.

The reproducible commands below use `/tmp/aws-inventory-emulator`. Choose a filesystem with sufficient free space for the clone, downloaded package and extracted executable; these paths are examples, not required workspace locations.

This package is for Linux amd64. The official native platform prerequisites include 64-bit Linux and glibc 2.17 or later; upstream publishes separate aarch64 packages and documents containers for other operating systems. Building the entire emulator from source or linking its embedded mode requires substantial resources: upstream documents **at least 100 GB of target-directory space**, Rust/Cargo and a C compiler. The tests here use the verified published executable while retaining the full source checkout for inspection. [Official setup][setup] · [Linux installation and signature verification][linux] · [Repository build guidance][emulator]

## Prerequisites and installation

Run these commands from the `aws-inventory-rs` project root. The installer needs Python 3.11 or later, `git`, `curl`, `gpg` and `dpkg-deb`. The recording proxy also needs `openssl`. The Rust toolchain must satisfy this workspace's `rust-version` in [Cargo.toml](../Cargo.toml).

The emulator package can be installed without Python:
`cargo run -p inventory-harness -- install <destination>` (see
`HARNESS.md`).

```bash
cargo run -p inventory-harness -- install /tmp/aws-inventory-emulator

cargo build --locked -p aws-inventory --features aws --bin aws-inventory -j1
cargo build --locked -p inventory-aws --example emulator_branches -j1
```

If using a custom Cargo target directory, pass the resulting binary paths explicitly to the runner. Coordinate Cargo jobs when multiple workers share a target directory; do not run another cold emulator build in that directory.

The installer destination contains:

| Relative path | Contents |
| --- | --- |
| `aws-emulator/` | Full source checkout |
| `aws-emulator_amd64.deb` | Pinned package |
| `aws-emulator_amd64.deb.sig` | Detached signature |
| `aws-publishing.asc` | Public signing key |
| `keyring/` | Verification keyring |
| `unpacked/opt/aws-emulator/bin/aws-emulator` | Verified executable |

The source commit is an immutable pin. Replacing it with an unreviewed latest release changes the test target. Update `crates/harness/assets/upstream.toml`, inspect the relevant operation implementations, verify the replacement package, and produce a new evidence directory when deliberately changing releases.

### Install the local graph runtime

The graph delivery checks use a real native FalkorDB instance. On Linux x86-64, install its pinned runtime without Docker or a system service:

```bash
cargo run -p inventory-harness -- install-graph /tmp/aws-inventory-graph --jobs 2
```

The [installer](../crates/harness/src/install/graph.rs) requires `make` and a C compiler. It verifies the pinned Redis 8.10.3 archive and FalkorDB 6.0.4 module hashes, bounds extraction, builds `redis-server` and `redis-cli`, checks the resulting versions, and writes `installation.json`. Its output paths are:

| Relative path | Purpose |
| --- | --- |
| `redis-8.10.3/src/redis-server` | Locally built Redis server |
| `redis-8.10.3/src/redis-cli` | Matching command-line client |
| `falkordb.so` | Pinned x64 FalkorDB module |
| `installation.json` | Download hashes, binary hashes, versions and build receipt |

An installation receipt has `graph_runtime_tested: false`: installation alone does not establish that a module loaded or that graph delivery passed. The supervised workflow below supplies that separate test boundary.

## Run the integration harness

Use a fresh output directory for each run. The harness refuses to reuse an existing inventory database.

```bash
cargo run -p inventory-harness -- mutate \
  --emulator /tmp/aws-inventory-emulator/unpacked/opt/aws-emulator/bin/aws-emulator \
  --binary target/debug/aws-inventory \
  --branch-probe target/debug/examples/emulator_branches \
  --output artifacts/emulator-local-run
```

To validate fixture setup without running the CLI, use a separate output directory and `--seed-only`:

```bash
cargo run -p inventory-harness -- mutate \
  --emulator /tmp/aws-inventory-emulator/unpacked/opt/aws-emulator/bin/aws-emulator \
  --output artifacts/emulator-local-seed-check \
  --seed-only
```

A seed-only result is never collector coverage. The harness launches the emulator, proxy, fixture requests and Rust child processes within the same process/session lifetime. This also works in environments where separate execution sessions have isolated network namespaces. Running a server in one isolated session and a client in another will not reach the same loopback listener.

The full run executes these paths:

| Path | Purpose |
| --- | --- |
| Native `scan --service all` | Regional and global native resource collection |
| `discover --organization --hierarchy --config-aggregators --resource-explorer` | Organizations topology and existing supplemental sources |
| `scan --service config,resource-explorer --discover-sources` | Config and Resource Explorer rows through production source selection |
| Cloud Control scan with the explicit 397 root type names | Frozen root selection and generic list/read collection |
| Cloud Control scan with the explicit 24 child type names | Parent discovery and child collection; selection automatically adds the six root prerequisites |
| `scan --service iam --role-arn …` | Official refreshable AssumeRole provider followed by an actual scan |
| `emulator_branches` example | Explicit SDK requests for conditional branches the normal fixture scan can skip |
| `architecture` and `report` | Analysis of a finalized scan in the produced SQLite database |
| `graph-export` | A complete native-scan projection written as FalkorDB-compatible Cypher commands; this step does not execute them against a graph server |

The ordinary scans use the explicit region `us-east-1`, bounded pages, small page sizes, a concurrency limit and the shared API executor. Small page sizes create real continuation requests where service limits allow them. EC2 describe APIs retain their service-specific minimum page sizes. These settings exercise collection mechanics; the local emulator's speed is not evidence of AWS throughput or quota compliance.

### Supervised snapshots, live tests and graph delivery

To keep the AWS Emulator, proxy, prepared state and FalkorDB inside one owned lifetime, run the wrapper with a fresh evidence directory:

```bash
cargo run -p inventory-harness -- mutate \
  --emulator /tmp/aws-inventory-emulator/unpacked/opt/aws-emulator/bin/aws-emulator \
  --binary target/debug/aws-inventory \
  --branch-probe target/debug/examples/emulator_branches \
  --output artifacts/emulator-v06-local \
  --redis-server /tmp/aws-inventory-graph/redis-8.10.3/src/redis-server \
  --falkor-module /tmp/aws-inventory-graph/falkordb.so \
  --command-deadline 7200 \
  -- cargo run -p inventory-harness -- verify-delivery \
  --binary target/debug/aws-inventory \
  --redis-cli /tmp/aws-inventory-graph/redis-8.10.3/src/redis-cli \
  --database artifacts/emulator-v06-local/inventory.sqlite \
  --output artifacts/emulator-v06-local/delivery-verification.json \
  --run-workspace-tests
```

The wrapper prepares and asserts the fixture, freezes **five** cases—native, supplemental sources, root Cloud Control, parent-dependent Cloud Control and assumed-role IAM—and runs each snapshot twice before the requested child command. Config recording stops after the independently checked rows stabilize so later comparisons retain exact capture timestamps. The delivery command checks each finalized scan's JSON against SQLite, compares the graph projection against a live server and checks checkpointed replay.

The current workspace has **ten guarded integration cases**: five collector snapshots, two CLI/durable-worker cases and three native FalkorDB cases. They require the wrapper's verified fixture environment and are otherwise ignored. `--run-workspace-tests` enables them in the first-party workspace, excluding the dependency-only SDK catalog crate. Reproduction commands here describe the intended checks, not a current passing receipt.

### Separate Rust branch probes

[crates/aws/examples/emulator_branches.rs](../crates/aws/examples/emulator_branches.rs) uses the real Rust SDK and emits `production_branch_reached: false`.

- `DescribeRegions` is probed explicitly because the main test uses one region. Allowing it to drive a multi-region scan of one emulator would conflict with upstream's isolation limitations.
- `HeadBucket` is normally a fallback when `ListBuckets` omits the bucket region. The emulator can supply that field, so a separate probe checks `HeadBucket` against the rich bucket fixture with `ExpectedBucketOwner` set.
- The probe also checks `GetCallerIdentity`, the emulator account and the returned bucket region.
- Resource Explorer `GetIndex`, `GetDefaultView`, `ListViews`, `GetView` and `ListResources` are probed independently so an early discovery decode failure does not prevent observing the remaining endpoints. Each result records `sdk_decoded`; failures retain a bounded error chain and make the probe command partial.

These calls use the phase `rust-probe:branch-probes`. They establish SDK/endpoint behavior for those operations. They do not establish that the corresponding production conditional branch executed. The harness never strips emulator response fields to manufacture production-path coverage.

## Endpoint, authentication and TLS contract

The emulator binds to the literal loopback address `127.0.0.1`, with a dynamically allocated nonzero port. The recording proxy also binds to loopback. Both connections use HTTPS and verify certificates:

| Connection | Trust |
| --- | --- |
| Python/Rust client → recording proxy | Temporary test CA and a signed server leaf certificate with `CA:false`, `serverAuth` and the `127.0.0.1` SAN |
| Recording proxy → emulator | The emulator's generated `ca.pem` |

The proxy forwards the original signed Host/header/body values so the emulator verifies the actual SigV4 request. It records metadata rather than signed headers or request/response bodies. TLS verification remains enabled; a certificate error is a failed test setup, not a reason to set `verify=False` or disable Rust TLS checks.

The harness starts the emulator with **`--auth on --telemetry off`**. It clears inherited `AWS_*` settings in its child-process environment, disables EC2 metadata, uses the generated credentials/configuration files for that emulator instance, and checks STS account **`000000000000`** before fixture setup. The production CLI uses those isolated files. The separate Rust branch probe receives this same server's generated keys through task-specific environment variables and constructs an explicit SDK credential provider; it does not resolve an ambient credential chain. The harness verifies the `EMULATOR` access-key prefix before passing the keys. Neither path uses a live AWS profile. The emulator creates new credentials for each process; use its current files rather than credentials from an earlier run. [Official SDK connection instructions][sdk]

The relevant settings are:

| Setting | Purpose |
| --- | --- |
| `AWS_ENDPOINT_URL` | The explicit proxy endpoint for every service, including SDK-owned STS traffic |
| `AWS_SHARED_CREDENTIALS_FILE` | This emulator process's generated `emulator_credentials` file |
| `AWS_CONFIG_FILE` | Isolated test configuration with the explicit endpoint |
| `INVENTORY_EMULATOR_ACCESS_KEY_ID` / `INVENTORY_EMULATOR_SECRET_ACCESS_KEY` | Generated emulator keys consumed by the explicit Rust branch-probe credential provider; never printed or persisted in evidence |
| `AWS_CA_BUNDLE` | CA used by Python/Botocore |
| `SSL_CERT_FILE` | CA used by the Rust TLS provider |
| `AWS_EC2_METADATA_DISABLED=true` | Prevent instance-metadata credential fallback |
| `AWS_REGION` / `AWS_DEFAULT_REGION` | The single test region |
| `EMULATOR_TELEMETRY=off` | Additional explicit telemetry opt-out |

The official documentation specifically lists `SSL_CERT_FILE` for Rust. `AWS_CA_BUNDLE` alone is insufficient. Use the IP literal for S3: upstream documents path-style access and does not support virtual-host bucket addressing. [Official SDK connection instructions][sdk]

Fixture mutations happen only in this disposable emulator. The production discovery implementation continues to discover existing sources and roles without creating them. The fixture modules create the roles, views, indexes, Config recorder and aggregators needed to test those read paths locally. State is discarded when the process stops. Upstream documents in-memory state and recommends starting a fresh instance for isolated tests. [Emulator scope][scope]

## Fixtures and what they prove

The runner invokes five fixture modules:

| Module | Scope |
| --- | --- |
| `seed_global.py` | IAM roles/profiles/policies, configured and minimally configured S3 buckets, Route53 records and CloudFront distributions |
| `seed_regional.py` | EC2 networking and instances, DynamoDB, Lambda, ECS, EKS and load balancer parent/detail paths |
| `seed_discovery.py` | Organizations OUs, Config aggregation/recorded rows, Resource Explorer sources, same-account role assumption and Cloud Control SQS fixtures |
| `seed_catalog.py` | Catalog-wide synthetic Cloud Control read-contract models, thirteen independent architecture models, and native Logs fixtures |
| `seed_parents.py` | Twelve parent fixtures, 63 semantic child models across the original 21 fixture-backed types, and 98 extra EKS addon models for pagination; three newer catalog-only workflows are exercised by model tests pending fixture expansion |

Every seed result records its outcome. A failed fixture may prevent a dependent detail call from executing; that leaves missing or error evidence, not a successful test. Each run must be assessed from `seed-results.json`, the operation matrix and SQLite results together.

### Generic property store and native integrations

The pinned emulator's Cloud Control `NativeResources::for_engine` connects three resource types to their owning service stores:

- `AWS::SQS::Queue`
- `AWS::Logs::LogGroup`
- `AWS::Logs::Destination`

Other supported namespaces generally use Cloud Control's own generic property store. `seed_catalog.py` supplies synthetic identifiers and properties for read-contract testing. These are **not deployable CloudFormation templates**. Their acceptance is not validation of the live provider's schema, provisioning, eventual consistency, permissions or integration with the corresponding native collector. [Pinned Cloud Control native-provider implementation][native-bridges]

The retained v0.5 run observed these eleven root catalog types rejected with **`TypeNotFoundException`**. Each failed both fixture creation and the production Rust list request; none had a subsequent detailed read:

| Catalog type | Observed seed limitation |
| --- | --- |
| `AWS::S3Express::DirectoryBucket` | Namespace not registered by this emulator |
| `AWS::S3Tables::TableBucket` | Namespace not registered by this emulator |
| `AWS::Scheduler::ScheduleGroup` | Namespace not registered by this emulator |
| `AWS::Bedrock::Agent` | Namespace not registered by this emulator |
| `AWS::Bedrock::ApplicationInferenceProfile` | Namespace not registered by this emulator |
| `AWS::Bedrock::AutomatedReasoningPolicy` | Namespace not registered by this emulator |
| `AWS::Bedrock::DataAutomationProject` | Namespace not registered by this emulator |
| `AWS::Bedrock::Flow` | Namespace not registered by this emulator |
| `AWS::Bedrock::Guardrail` | Namespace not registered by this emulator |
| `AWS::Bedrock::KnowledgeBase` | Namespace not registered by this emulator |
| `AWS::Bedrock::Prompt` | Namespace not registered by this emulator |

The first three were also present in the historical v0.4 selection. The eight Bedrock gaps were added to the acknowledged set only after the initial expanded run demonstrated matching seed errors, Rust list errors and persisted failures, and a source inspection confirmed the absent namespaces. That failed initial run remains preserved.

The current runner explicitly selects all 397 roots for its root case and all 24 children for a separate parent case, retaining refused scopes as errors. Child requests depend on successful prerequisite discovery and can have several list chains per type. An attempted list does not imply a successful list, a populated result or a successful read. Consult the exact run's receipts and assertions; retained root evidence is not a parent-workflow result. The emulator's registration decision is in its [pinned Cloud Control state implementation][cloudcontrol-state].

Thirteen supported generic-provider types are seeded with independently authored architecture properties. `assert_cloud_control.py` checks 111 assertions on the Rust-to-SQLite result: thirteen exact resource identities, 30 exact relationships, target kinds, ownership evidence, merged property paths, redaction and observation timestamps. It also rejects a security-group name that resembles an ID, distinguishes ECS VPC Lattice target groups from ELB target groups, and preserves DMS certificates and MSK CA secrets without misclassifying them as ACM certificates. Python seed requests remain excluded from collector coverage.

### Parent models and forced pagination

[cloudcontrol-parents.toml](../crates/aws/tests/fixtures/cloudcontrol-parents.toml) is an independently authored fixture with two parents for each of six roots. Each of the original 21 fixture-backed child types has two semantic records under parent A and one under parent B. Ninety-eight additional EKS Addon records bring that one type to **101 records** and the complete child fixture to **161 records**. These repetitive extras test pagination; they are separate from the 63 semantic records. The three newer catalog-only types have literal model tests but no live fixture records yet.

The pinned emulator applies `ResourceModel` to each examined window, whose default size is 100. The fixture orders all 100 A addons before B's addon, so B's first page is empty with a continuation token and its second page contains its child. Both parent chains must reach at least page two and complete. The collector deliberately omits the reserved `MaxResults` field; this test forces pagination by fixture size. See the [pinned ListResources implementation][cloudcontrol-list] and [AWS API contract](https://docs.aws.amazon.com/cloudcontrolapi/latest/APIReference/API_ListResources.html).

`seed_parents.py` uses signed Cloud Control create/status requests for generic models and native Logs `CreateLogGroup` for the two log-group parents. It retains the actual create-returned identifiers and source/fixture digests; it does not patch emulator state. Generic models are synthetic read contracts, not deployable CloudFormation templates. Native Logs ARN identity, including its returned suffix, is checked separately. Python seed calls have distinct fixture labels and cannot satisfy Rust collection coverage.

`assert_parents.py` requires exact sets of **161 detailed child observations**, **161 `CHILD_OF` edges** and **24 additional architecture edges**. It compares kinds, owner identities, provider fields, redaction, model provenance, independent scope completion and actual seed identifiers, and rejects cross-parent attachments or fabricated pagination. It does not assume the emulator's filtering is sufficient: the collector independently checks the child's returned model before creating an edge.

A selected child's root is automatically included once. Failed lists, failed required parent reads and invalid context leave incomplete prerequisite evidence; independent successful parents retain their children. Empty child discovery is complete only after an exhaustive successful root enumeration. Aggregate bounds are 10,000 child-model scopes and 100,000 distinct child identifiers per root group, with at most four bindings, 1,024 bytes per value and 8,192 encoded model bytes. Additional dependency levels such as ELBv2 listener rules remain excluded. [Parent workflow details](PARENT_COLLECTIONS.md)

### Source and binary binding

Before collection, the harness executes the compiled CLI's offline `catalog` command and requires its descriptors to equal the merged 421-type catalog. `summary.json.execution` records that receipt, both catalog hashes, the binary hash, the Rust source/manifest fingerprint, both architecture-fixture hashes and separate root/parent command receipts and scan IDs. After collection, source, binary and fixture hashes must remain unchanged. The root coverage refresher verifies these bindings again and requires same-scan detailed SQLite observations; changed mappings cannot reuse an older successful provider count. Parent assertions remain a separately identified result.

Resource Explorer has live owner integrations for **SQS queues and SNS topics**, plus its own indexes and views. It does not discover every service that happens to be mounted in the emulator. [Pinned Resource Explorer owner integration][explorer-owners]

Config's automatic recorder observes **S3 buckets, SQS queues and IAM roles**. Global IAM recording requires explicit inclusion. The discovery fixtures also record two synthetic DynamoDB configuration items with `PutResourceConfig`, exercising supplemental row parsing without claiming that the emulator discovered native DynamoDB tables. [Pinned Config recording implementation][config-recording]

## Interpreting evidence

The runner writes:

| Artifact | Contents |
| --- | --- |
| `summary.json` | Pin, run checks, operation outcomes, provider boundary, per-type Cloud Control coverage |
| `requests.jsonl` | Service/operation, phase, request sequence, HTTP status, response size/hash, pagination marker presence and public type name |
| `seed-results.json` | Fixture outcomes and seed failures |
| `native.json`, `discovery.json`, `sources.json`, `cloud-control.json`, `parents.json`, `assumed-role.json` | Exact CLI arguments, exit status and structured output; root and parent Cloud Control scans are separate |
| `branch-probes.json` | Separately labeled Rust SDK probe assertions |
| `global-assertions.json`, `regional-assertions.json` | Fixture-specific resource and relationship assertions against the finalized native scan |
| `cloud-control-assertions.json` | Independent provider-model identity, relationship, redaction and provenance assertions |
| `parent-assertions.json` | Exact parent/child attachment, detail, property, architecture, completeness and pagination assertions |
| `database-evidence.json` | SQLite integrity, finalized scan states, scopes, resources, relationships and failures |
| `inventory.sqlite` | The actual local SQLx-backed scan store |
| `architecture.json`, `report.json` | Analysis output for a selected finalized scan |
| `graph-export.json`, `native-graph.jsonl` | Export command receipt and generated Cypher projection; no graph-server execution is implied |
| `oracle-baseline-1.json`, `oracle-baseline-2.json`, `snapshot-receipt.json` | Supervised workflow: two unmutated repetitions of all five snapshot cases and their independently checked baseline |
| `workspace-live.json`, `workspace-live.log` | Full first-party workspace test execution with the prepared emulator and local FalkorDB |
| `delivery-verification.json` | Exact SQL payload/provenance comparisons against exported records and a real local FalkorDB, followed by checkpointed replay |
| `server.jsonl` | Upstream server diagnostics; retain only after applying the harness's audit-log policy |

The request recorder never retains Authorization headers, access keys, session tokens, cursor contents, raw request bodies or raw response bodies. Source inventory output is still subject to the application's normal redaction rules. Temporary proxy CA/leaf private keys are not part of the deliverable. Do not copy generated credential files or temporary TLS material into the artifact directory.

Coverage statuses have deliberately different meanings:

| Status | Meaning |
| --- | --- |
| `collector_success` | A production Rust collector request received a successful HTTP response; inspect result assertions and SQLite scopes before claiming correct collection |
| `explicit_sdk_probe_success` | A separately labeled Rust request received a successful HTTP response and has an accepted typed/assertion receipt; the production branch is not thereby covered |
| `probe_response_unverified` | A Rust probe received successful HTTP, but no accepted typed/assertion receipt establishes its outcome |
| `sdk_decode_failure` | A Rust probe rejected a successful response during SDK decoding; its error chain overrides HTTP-success classification |
| `explicit_sdk_probe_error` | A typed Rust probe reported a service or transport error, distinct from a successful-response decoding failure |
| `explicit_sdk_probe_assertion_failed` | The SDK decoded a response, but an explicit fixture assertion failed |
| `emulator_not_implemented` | The operation was attempted and the emulator returned HTTP 501 |
| `error_response` | A recorded attempt returned another failure class |
| `not_exercised` | No matching Rust request was observed |

Python seed/probe requests cannot satisfy these Rust operation rows. A branch receipt is accepted only from the `branch-probes` command with exit code 0 or 2 and valid structured output. Resource Explorer success requires both `sdk_decoded: true` and `assertions_passed: true`; missing fields stay unverified. A partial command can therefore retain successful individual probes while preserving failed probes. SDK credential-provider `AssumeRole` is labeled separately from the 72 explicit operations. It is owned by the AWS SDK identity cache and is outside the application executor's attempt accounting.

The current `coverage:every_operation_attempted` gate requires an observed request for every expected operation and the provider boundary. The root per-type list gate checks the frozen 397-type selection; the parent case adds exact semantic and prerequisite assertions. Neither gate means every operation succeeded, every provider was implemented, or every resource detail mapping was correct. Request evidence must be read alongside scopes and semantic checks. Official fidelity guidance similarly distinguishes covered test cases from complete service behavior. [Fidelity documentation][fidelity]

Additional gates reject unexplained operation failures and require successful reads for every registered catalog type. `known-gaps.json` names the exact expected upstream failure classifications and the eleven unregistered selected types for this pin. Recognizing one of those gaps does not convert its operation outcome into success. A run with all harness checks passing can still contain these explicitly acknowledged incompatibilities.

## Known limitations and current evidence

At this pin, **Organizations `ListAccounts` and `ListChildren` return HTTP 501 `NotImplemented`**. The source contains unconditional scaffold returns, and the standalone fixture run observed both responses. `CreateAccount` is also unimplemented, so the harness does not fabricate member accounts. Root/OU reads and OU creation work, allowing real hierarchy pagination while preserving incomplete account/membership evidence. [ListAccounts implementation][list-accounts] · [ListChildren implementation][list-children]

### Resource Explorer timestamp incompatibility

The pinned emulator returns numeric epoch timestamps where Resource Explorer requires date-time strings. A fresh authenticated reproduction captured, for example:

```json
{
  "CreatedAt": 1794506072.52364,
  "LastUpdatedAt": 1794506073.037352
}
```

The official `GetIndex` response syntax declares strings for both fields, and the emulator's own Smithy model marks them `timestampFormat: date-time`. The pinned Rust SDK `aws-sdk-resourceexplorer2 1.126.0` decodes them using `Format::DateTimeWithOffset`, which requires JSON strings. Botocore accepts the numeric representation and produces Python `datetime` values; that permissiveness let the earlier Python fixture test succeed. [Official GetIndex response][get-index-api] · [Pinned Resource Explorer model][explorer-model]

`emulator-resource-explorer-wire-check.json` records the public fixture JSON, response hash, field types and an actual Rust `discover` result. Rust discovery returns a partial report with Resource Explorer unavailable after HTTP 200 fails SDK decoding. The same raw mismatch occurs in `GetView.View.LastUpdatedAt` and `ListResources.Resources[].LastReportedAt`, including resource-property timestamps. Separate Rust probes must record their own typed outcomes; a successful HTTP response alone is insufficient.

The source points to a shared schema-generation omission: the macro's member schemas do not preserve `timestampFormat`, while the JSON serializer defaults to epoch seconds when no timestamp-format trait reaches it. This is upstream compatibility evidence. Neither emulator source nor response bodies are modified to hide it. [Pinned schema generation][schema-codegen] · [Pinned JSON timestamp serializer][json-codec]

### Fidelity boundaries

The official emulator documentation limits its guarantees:

- Account and region state are not consistently isolated within one process. Use one account/region per instance and distinct processes for separate scenarios.
- IAM and resource-policy behavior are not equivalent to AWS. This test does not prove effective production authorization or cross-account trust semantics.
- It is not validated for production, security testing, long-running workloads or load testing.
- Published fidelity results cover selected behaviors and environments; missing tests and successful HTTP responses are not completeness guarantees.

These limits also apply to retry and concurrency claims. Local successful calls do not establish real AWS throttle limits, retry distributions, service availability by region or sustained throughput. [Official scope and limitations][scope] · [Fidelity documentation][fidelity]

## Historical v0.5 integration results

The final v0.5 collection summary records **1,188 passing checks and zero failed checks**. Its `execution` object binds the ordinary Rust executable to the descriptor catalog, source/manifest fingerprint, independent architecture fixtures and exact command receipts. The tested CLI SHA-256 is `b00a23d5dea91327a365f31ec5c58e41bb4169146f9f53fcd7c41ed126e8cc80`.

The request ledger contains **2,067 requests: 1,072 from Rust collection/probes and 995 from Python fixture setup/startup**. All **73 operation boundaries** were observed: the 72 explicit operation identities and the SDK-owned AssumeRole boundary. Outcomes remain distinct:

| Final classification | Count | Observed behavior |
| --- | ---: | --- |
| `collector_success` | 64 | 63 explicit operation identities and the separately flagged AssumeRole provider received successful HTTP |
| `explicit_sdk_probe_success` | 4 | Typed, asserted DescribeRegions, HeadBucket, GetDefaultView and ListViews probes succeeded |
| `emulator_not_implemented` | 2 | Organizations ListAccounts and ListChildren returned HTTP 501 |
| `sdk_decode_failure` | 3 | Resource Explorer GetIndex, GetView and ListResources returned HTTP 200 but failed the pinned SDK's timestamp decoding |

There were no unattempted operation identities or unverified probe-success classifications. These counts describe distinct SDK operations, not provider types or complete branch coverage. The 1,188 assertions comprise 984 regional assertions, 53 global assertions, **111 Cloud Control architecture assertions**, and 40 orchestration, provenance, persistence and coverage checks.

The actual persisted snapshots are:

| Scan | Terminal result | Resources | Relationships | Scope outcome |
| --- | --- | ---: | ---: | --- |
| Native regional/global inventory | Complete | 150 across all 32 native kinds | 340 | 131 complete, none failed |
| Config and Resource Explorer | Partial | 11 from Config | 0 | Two complete; one Explorer discovery scope failed |
| Cloud Control catalog | Partial | 389 observations across 386 successfully read types | 32 | 775 complete; 11 unsupported type-list scopes failed |
| Assumed-role IAM | Complete | 13 | 22 | 24 complete, none failed |

Cloud Control attempted all **397 type lists**. The 386 successful types produced 389 observations because four separate SQS queues were read. The 32 stored relationships include the 30 independently authored semantic-fixture edges and two native-bridge edges. All thirteen semantic fixture models passed their identity, target-kind, ownership, property-path, redaction and timestamp checks. These observed cases do not establish that all 945 declared relationship rules have been exercised. The eleven namespace failures remain persisted as `TypeNotFoundException` failures and keep the Cloud Control scan partial. [Bound type evidence](CLOUDCONTROL_EVIDENCE.md)

The four snapshots are separate scan/source observations and overlap; their counts must not be added into a distinct AWS resource count. Source discovery retains two failure records for its one failed Explorer scope, and both records remain in export. The local assumed-role case proves initial acquisition and subsequent IAM collection; it does not test session-expiration refresh or production cross-account authorization.

Two unmutated executions of all four snapshot cases then passed against the same prepared fixture state. The Config recorder was stopped after the independently checked fixture rows were captured, retaining exact capture timestamps rather than suppressing fields that varied during continuous recording. Both repetitions retained exact normalized resources, edges, scope outcomes and failure evidence.

The subsequent full first-party workspace run passed **503 tests, zero failed and zero ignored**, using `--workspace --exclude inventory-sdk-catalog --all-features --locked --offline -j1 -- --include-ignored --test-threads=1`. This includes the previously ignored emulator and native FalkorDB cases. The dependency-only SDK catalog crate is explicitly excluded; this is not a test result for every compiled AWS SDK package. The health-supervised command completed with exit code 0 and a matching retained log hash. This execution ran normal tests and delivery verification; it was not a mutation-testing campaign.

### Exact export and FalkorDB delivery

The delivery verifier compared each snapshot's export payload and provenance against SQLite, sent the graph to a real local FalkorDB instance, and read back exact resource, unresolved-target and edge identities/provenance. All four cases passed. The graph state retained the source snapshot's complete or partial status:

| Snapshot | Observed resource nodes | Unresolved target nodes | Relationships | Batches applied on first delivery | Batches skipped on replay |
| --- | ---: | ---: | ---: | ---: | ---: |
| Native | 150 | 43 | 340 | 6 | 6 |
| Config/Explorer | 11 | 0 | 0 | 4 | 4 |
| Cloud Control | 389 | 25 | 32 | 6 | 6 |
| Assumed-role IAM | 13 | 12 | 22 | 5 | 5 |

Each replay made **zero graph attempts and applied zero batches**, using the persisted acknowledgments. An unresolved target is a referenced identity without a resource observation in that snapshot; it is not counted as a collected AWS resource. The verification receipt also records exact export counts for scopes and failures, comparison hashes, cleanup results and the full live test receipt.

The recorded collection includes **93 requests carrying a noninitial pagination cursor across 29 operations**. Offline classifier checks cover 41 pagination models. The native scan observed a peak of 19 in-flight attempts in this fixture run, with zero throttles or retries; those observations cannot establish production AWS quotas, retry behavior under throttling or large-account throughput. The direct upstream calls from the repeated tests happen after the recording proxy closes and are validated by the separate snapshot/test receipts, not added to the original 2,067-request ledger.

## Historical v0.4 integration results

The final run in `artifacts/emulator-v04` used ordinary Cargo-built executables rebuilt after the final source fixes. `plain-binaries-v04-final.json` records their SHA-256 hashes, changed-source hashes and build receipts. The earlier expanded run is preserved separately; its binaries and request totals are not substituted for the final evidence.

The final ledger contains **1,276 requests: 686 from Rust collection/probes and 590 from Python fixture setup/startup**. Only the Rust requests contribute to the **73 observed operation identities**, covering all 72 explicit operations plus the SDK-owned AssumeRole boundary. Their outcomes are:

| Final classification | Count | Meaning in this run |
| --- | ---: | --- |
| `collector_success` | 64 | At least one production Rust request received successful HTTP; includes the separately flagged STS AssumeRole provider boundary |
| `explicit_sdk_probe_success` | 4 | Verified DescribeRegions, HeadBucket, GetDefaultView and ListViews receipts; production conditional branches are not thereby covered |
| `emulator_not_implemented` | 2 | Organizations ListAccounts and ListChildren returned HTTP 501 |
| `sdk_decode_failure` | 3 | Resource Explorer GetIndex, GetView and ListResources returned HTTP 200 but failed typed SDK decoding |

No operation was left unattempted or classified as an unverified probe response. The three Resource Explorer error receipts retain `sdk_error_kind: success_response_rejected`, HTTP 200 and the JSON string-type error chain. The two successful Explorer probes passed explicit fixture assertions.

The **1,069 passing checks** consist of **53 global fixture assertions, 984 regional fixture assertions and 32 orchestration, storage, graph-export and coverage checks**. They include repeated checks of individual resource identities, account/region attribution and relationships; they are not 1,069 distinct AWS API tests. Known incompatibilities remain failed operation outcomes even though the exact expected-gap checks pass.

The actual persisted scan results are:

| Scan | Terminal result | Resources | Relationships | Scope outcome |
| --- | --- | ---: | ---: | --- |
| Native regional/global inventory | Complete | 150 across all 32 native kinds | 340 | 131 complete, none failed |
| Config and Resource Explorer sources | Partial | 11 from Config | 0 | Two complete; Explorer source-discovery scope failed |
| Cloud Control catalog | Partial | 200 observations across 197 successfully read types | 2 | 397 complete, three unsupported type-list scopes failed |
| Assumed-role IAM inventory | Complete | 13 | 22 | 24 complete, none failed |

These are separate immutable scans, and their row counts overlap; adding them does not give a distinct AWS resource count. SQLite integrity was `ok`, and all four scans were finalized. Discovery retained one organization root and two OUs while preserving incomplete account/membership coverage. Zero returned accounts is not treated as evidence that the organization has no accounts.

Cloud Control issued a list request for all **200 types**. Exactly **197 types** received successful list and read responses. The three namespace failures in that selection—S3Express DirectoryBucket, S3Tables TableBucket and Scheduler ScheduleGroup—returned HTTP 400 `TypeNotFoundException` on list and had no subsequent read. The generic-property-store limitations still apply to the 197 successful types. Fixture setup also retained an HTTP 501 for `AcceptVpcPeeringConnection`; the pending peering resource was collected without claiming that acceptance succeeded.

The runtime ledger contains **93 noninitial-cursor requests across 29 distinct operations**. The offline classifier checks 41 pagination mappings; the remaining mappings did not all encounter a continuation page in this fixture run. Request counts, local elapsed time and observed in-flight work do not establish live AWS limits, large-account throughput or complete pagination-branch coverage.

The native architecture report and inventory report completed. Graph export wrote the same **150 resources and 340 relationships in six batches** to `native-graph.jsonl`. This validates projection generation; this harness did not execute those commands against a FalkorDB server. The assumed-role scenario verifies initial credential acquisition and a subsequent IAM scan, not session-expiration refresh or live cross-account authorization.

## Query the bundled fixture databases

The retained v0.5 inventory.sqlite, exactly 4 MiB, and v0.4 inventory.sqlite, approximately 2.9 MB, contain generated emulator fixtures and finalized scan evidence, not live AWS inventory or AWS credentials. Their historical JSONL exports and native JSON export are also retained. They do not contain the new parent-workflow run. Current release artifacts and package contents are listed separately in [VERIFICATION.md](VERIFICATION.md).

From the project root, copy the historical database before querying it with the current CLI. Store initialization can upgrade an older schema even when the subsequent SQL query is read-only; the copy preserves the original evidence bytes.

```bash
inventory_query_dir="$(mktemp -d)"
cp artifacts/emulator-v05/inventory.sqlite "$inventory_query_dir/inventory.sqlite"
target/debug/aws-inventory query \
  --db "$inventory_query_dir/inventory.sqlite" \
  --limit 10 \
  'SELECT s.scan_id, s.status,
          (SELECT COUNT(*) FROM resource_observations r
           WHERE r.scan_id = s.scan_id) AS resources,
          (SELECT COUNT(*) FROM edge_observations e
           WHERE e.scan_id = s.scan_id) AS relationships
   FROM scans s
   ORDER BY s.started_at, s.scan_id'
```

This local query lists the four finalized scans and their persisted row counts. It makes no AWS requests. The partial Config/Explorer and Cloud Control scans remain visible alongside the two complete scans; their failure evidence is retained in the same database.

## Earlier runs and diagnostic evidence

| Evidence | Verified result |
| --- | --- |
| Standalone discovery seed check | 47 fixture calls succeeded; 16 assertions passed; Config returned six rows over six pages and Explorer Search returned seven rows over seven pages; both Organizations 501 responses retained. This was a Python fixture validation, not Rust collector coverage. |
| First v0.4 full run; historical receipt not included in this checkout | Failed TLS interoperability before any Rust operation reached the recorder: 601 total requests, zero observed Rust operations, 16 passing and 11 failing run checks. Rust rejected the original proxy certificate with `CaUsedAsEndEntity`. This is failed setup evidence. |
| Second v0.4 full run; historical receipt not included in this checkout | 1,298 recorded requests; 69 observed Rust operation identities; 26 passing checks and one failing coverage check. Four Resource Explorer operations were not reached because automatic source discovery stopped at the `GetIndex` SDK decode failure. This is incomplete operation coverage. |
| Expanded v0.4 run before the final rebuild; historical receipt not included in this checkout | 1,290 requests; 73 Rust operation identities; 1,069 passing checks and zero failed checks, with the same five acknowledged operation incompatibilities. The final rebuild was rerun in the distinct `emulator-v04` directory. |
| Initial v0.5 expanded run | 397 type-list attempts; 386 successfully read types; 1,161 passing checks and one failing coverage check. Eight added Bedrock namespaces were unsupported. Their gaps were accepted only after source/error review; this receipt was not used to bind the final capability matrix. |
| Resource Explorer wire reproduction | Five diagnostic assertions passed: numeric timestamps observed in GetIndex/GetView/ListResources, Botocore accepted them, and the actual Rust discovery path returned the expected partial/unavailable result. |

The second summary's `collector_success` label is based on HTTP status and includes `GetIndex` despite its typed decode failure. The expanded and final harnesses add `sdk_decode_failure` overrides from the explicit Rust probes; the earlier evidence is retained without rewriting its historical classifications. The final run achieves operation-attempt coverage while retaining the demonstrated endpoint incompatibilities.

The production supplemental collector uses Resource Explorer **`ListResources`**. The earlier standalone fixture check used `Search`; it does not establish the current collector's `ListResources` data-path result. The first run's certificate issue was fixed by issuing the proper server certificate, while retaining verification on both TLS connections. Expanded Resource Explorer probes exercised the remaining endpoints independently of the failing automatic-selection step and confirmed the typed failures in the final run.

### Offline recorder checks

The Rust harness includes comprehensive offline tests for transport, evidence capacity, phase preservation, credential/cursor exclusion, and receipt validation. Run them with:

```bash
cargo test -p inventory-harness
```

These tests are offline; they do not validate TLS handsha…

The retained **v0.5** combined emulator Python suite passed **29 tests**, including those 19 recorder tests, seven independent Cloud Control assertion negative controls and three subprocess-supervision regressions. Its separate suites passed thirteen schema/enrichment tests and twelve coverage-binding negative controls. The latter reject stale catalog/source/fixture/binary evidence, duplicate matrix rows and HTTP-success records without matching typed persisted reads. These historical counts do not include the new parent oracle or graph installer checks.

Run the parent fixture/schema checks independently:

```bash
cargo run --locked -p catalog-tool -- universal --check
cargo test -p inventory-aws --test assert_parents
```

### Offline Rust SDK protocol fixtures

The separate [tests/sdk-wire](../tests/sdk-wire/README.md) crate has **six passing tests** against the unchanged, pinned Resource Explorer SDK. It uses the official Smithy [`StaticReplayClient`][replay-client] to supply literal in-memory responses, verifies each request, and checks typed decoding. Three positive responses use modeled ISO 8601 strings; negative variants replace each of five timestamp locations with a JSON number independently:

| Operation | Timestamp locations checked independently |
| --- | --- |
| `GetIndex` | `CreatedAt`, `LastUpdatedAt` |
| `GetView` | `View.LastUpdatedAt` |
| `ListResources` | `Resources[].LastReportedAt`, `Resources[].Properties[].LastReportedAt` |

The tests accept the modeled string representations and reject every numeric variant despite HTTP 200. The successful-response fixtures are constructed from the official API shapes; they are not captured successful emulator responses. The replay client makes no network calls, and no emulator body or SDK parser is changed. These protocol tests explain the integration incompatibility without turning the failing emulator operations into passes. [GetIndex response][get-index-api] · [GetView response][get-view-api] · [ListResources response][list-resources-api]

The crate has its own lockfile and target directory so its Smithy test-only features do not rebuild the main collector's EC2 dependency graph. The [test README](../tests/sdk-wire/README.md) gives reproducible commands and bounds. `sdk-wire-tests-v04.log` records all six passing tests; `sdk-wire-clippy-v04.log` records strict Clippy success with warnings denied.

The new parent collector additionally has a bounded in-process HTTP fixture in `crates/aws/src/cloud_control/parents/wire_tests.rs`. It invokes the production Cloud Control SDK path, checks independent parent models and cursors over empty/nonempty pages, injects an initial throttle and compares executor attempt/retry counters. Its collector and server share one owned future with a fixed deadline. This is an implemented test contract; the retained Resource Explorer protocol receipt above does not certify this new test. See current verification artifacts for its execution status.

## Diagnosing a failed run

Start with `summary.json`, then the specific CLI artifact and relevant scope failures. Keep the original failed evidence when rerunning with a fix.

- **Zero Rust operations, successful Python seeds:** inspect Rust stderr and `branch-probes.json` for TLS trust or certificate-use errors. Check `SSL_CERT_FILE`, the CA/leaf chain, the IP SAN and the endpoint visible to the child process. A Python TLS success alone does not prove Rust TLS interoperability.
- **Config or Explorer rows absent:** inspect discovery selection, not just the list endpoint. Confirm the aggregator ARN/account/region, active index, visible/default view and fixture seed outcomes. Ambiguous or denied sources remain explicit outcomes.
- **Missing detail operation:** verify its parent fixture exists and its list returned a nonempty page. Conditional branches and failed seed calls can legitimately prevent a child request.
- **Only seed/probe requests for an operation:** leave the production coverage gap visible. Do not relabel the requests or infer execution from the existence of an SDK dependency.
- **`TypeNotFound` or `NotImplemented`:** retain the operation/type and error evidence; check the pinned upstream implementation before changing collector behavior.

The public [AWS Emulator evaluation suite][evaluation] contains a `manifest.jsonl` mapping tests to services and operations. It is additional fidelity evidence, not a substitute for exercising this application's actual SDK calls and persistence behavior. [Official project compatibility workflow][check-project]

[emulator]: https://github.com/aws/aws-emulator
[setup]: https://docs.aws.amazon.com/emulator/latest/userguide/setting-up.html
[linux]: https://docs.aws.amazon.com/emulator/latest/userguide/getting-started-linux.html
[sdk]: https://docs.aws.amazon.com/emulator/latest/userguide/point-sdk-to-emulator.html
[scope]: https://docs.aws.amazon.com/emulator/latest/userguide/what-is-service.html
[fidelity]: https://docs.aws.amazon.com/emulator/latest/userguide/fidelity.html
[evaluation]: https://github.com/aws/aws-emulator-evaluation-suite
[check-project]: https://docs.aws.amazon.com/emulator/latest/userguide/check-project.html
[native-bridges]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-cloudcontrol-cloudapiservice/src/state/native.rs
[cloudcontrol-state]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-cloudcontrol-cloudapiservice/src/state.rs
[cloudcontrol-list]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-cloudcontrol-cloudapiservice/src/ops/list_resources.rs
[explorer-owners]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-resourceexplorer2-resourceexplorer/src/state/owners.rs
[config-recording]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-configservice-starlingdoveservice/src/state/recording.rs
[list-accounts]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-organizations-awsorganizationsv20161128/src/ops/list_accounts.rs
[list-children]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-organizations-awsorganizationsv20161128/src/ops/list_children.rs
[get-index-api]: https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetIndex.html
[get-view-api]: https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetView.html
[list-resources-api]: https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_ListResources.html
[replay-client]: https://docs.rs/aws-smithy-http-client/latest/aws_smithy_http_client/test_util/struct.StaticReplayClient.html
[explorer-model]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-resourceexplorer2-resourceexplorer/smithy-model/com-amazonaws-resourceexplorer2-resourceexplorer.json
[schema-codegen]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/vendor/smithy-macros/src/lib.rs
[json-codec]: https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/vendor/smithy-schema-server/src/json.rs
