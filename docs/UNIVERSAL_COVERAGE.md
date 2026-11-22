# Universal coverage: CloudQuery × Cartographer

> Status: Baseline being replaced. Universal listing through Cloud Control is
> retired by decisions V01 and V04, gets no further work, and is deleted in
> ROADMAP M9.5; complete coverage comes from endpoint collection of every
> service instead (M9). See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

This project collects AWS inventory two ways: native SDK adapters for the
resource kinds that need typed handling, and provider-generic listing through
Cloud Control. The universal layer exists so coverage is bounded by the
provider's resource registry, not by the number of adapters someone wrote.

## Type universe

Source of truth is the pinned CloudFormation registry archive:

| Artifact | Value |
| --- | --- |
| Archive | `catalog/upstream/cloudformation-us-east-1-2026-11-12.zip` |
| Archive SHA-256 | `8226b9cd82f432997fe3918b134ef155245636f35ac37f8006de5abe88dd54a8` |
| Schemas | 1,881 |
| Listable | 1,291 |
| Parent-scoped (workflow required) | 410 |
| Unsupported by the provider's list/read handlers | 180 |

`catalog/cloudcontrol-universal.json` is generated from the archive plus the
capability matrix:

- `capability: "list"` — `ListResources` is callable directly. Includes the 894
  types whose scope is not yet reviewed; they are listable, so universal
  collection does not wait for review.
- `capability: "parent"` — listing requires a parent model. These wait on
  generated parent workflows.
- `capability: "unsupported"` — the schema has no list/read handler.

Every row keeps `schema_sha256`, `visibility`, `exclusion`, and
`parent_type` when a reviewed plan states one. Generation verifies each digest
against the archive bytes and fails on mismatch.

### Parent-workflow candidates

`catalog/cloudcontrol-parent-candidates.json` is generated deterministically by
the Rust `catalog-tool` from the pinned schema archive:

```sh
cargo run -q -p catalog-tool -- parent-candidates --root . --check
```

It covers all 410 types whose list handler requires a model. All 24 reviewed
parent workflows fall inside that set, so 386 remain, which is what
`aws-inventory coverage` reports. The generator currently recognizes only 20 of
the 24 as reviewed and therefore prints 390 remaining; it misses
`AWS::ApiGateway::GatewayResponse`, `AWS::EKS::Nodegroup`,
`AWS::EKS::PodIdentityAssociation` and `AWS::ElasticLoadBalancingV2::Listener`
(not being fixed; this tooling is retired). Among the 390 it lists as open, inferring from required
list-handler/create properties proposes a parent for 325 at high confidence
(same family) and 13 at medium confidence (one cross-family match), leaves 52
low/unmapped, and reports zero disagreements with the reviewed workflows. A
proposal is review input only: the runtime collector never infers a parent from
an arbitrary property name.

```sh
cargo run -p inventory-harness -- catalog universal --root . --check
```


```sh
# native adapters plus every listable Cloud Control type in one scan
aws-inventory scan --db inventory.sqlite --region us-east-1 \
  --service all,cloud-control --cloud-control --resource-type all \
  --attempts 1 --concurrency 16 --rps 200
```

## Coverage denominator

`catalog-tool coverage [--check | --write]` writes
`catalog/coverage-denominator.report`, a deterministic funnel computed only from
committed artifacts: `aws-models.report`, `aws-plans.report`,
`aws-classes.report`, `aws-overlay.report`, `aws-operations.toml`,
`cloudcontrol-capabilities.toml`, and `cloudcontrol-resources.toml`.

- One `catalog-digest` (SHA-256 over the seven input paths and digests) ties
  every count to one catalog state.
- Four funnels: `operations`, `native_sdk_operations`,
  `cloud_control_types`, `resource_kinds`. Each stage is no larger than the
  stage before it; the report is refused otherwise.
- Excluded buckets (`denied_by_overlay`, `missing_read_or_list_handler`, …)
  partition the funnel's starting population and are checked against the
  source's own totals.
- `fixture_tested`, `emulator_tested`, `real_aws_tested` are recorded only
  where the catalog holds that evidence. `not_recorded` marks a stage the
  catalog does not measure; it is never read as zero. `real_aws_tested 0` is
  stated with its basis (`evidence.live_validated=false`).
- `unsupported`, `inaccessible`, `skipped`, and `unobserved` are always listed
  as `not_recorded` unless a committed artifact records them.

## Verified emulator baseline

The pinned emulator (1.0.336, commit `f5155815`) implements a Cloud Control
registry for the reviewed type set. Against it:

| Check | Result |
| --- | --- |
| Native services | complete, 25 scopes, 0 failures |
| Reviewed Cloud Control types (418 at the time of the run; the catalog now has 421) | 407 complete; 11 `TypeNotFoundException` matching the recorded evidence |
| Universal Cloud Control (`--resource-type all`) | complete in 8 s: 1,291 attempted, 999 succeeded, 292 `unsupported_type`, 0 failed, 410 parent + 180 unsupported handler skips |
| Runtime type discovery | Cloud Control's SDK has no `ListTypes`; type discovery comes from the pinned registry archive (CloudFormation `ListTypes` on a live account) |

The emulator answers 999 of the 1,291 listable types (many with empty
inventories because nothing was seeded); the remaining 292 are recorded as
provider boundaries. A Cloud Control create → poll → list round trip was
executed against the same emulator (`AWS::SNS::Topic`) and the collector then
observed the created resource.

### Pinned partition, asserted literally

The live gate pins the partition as literals, so a scan that stops early cannot
pass by agreeing with the catalog it was generated from:

| Counter | Pinned value |
| --- | --- |
| `total` | 1,881 |
| `list_capable` | 1,291 |
| `skipped_parent_required` | 410 |
| `skipped_unsupported` | 180 |
| `attempted` | 1,291 |
| `succeeded` | 999 |
| `unsupported` (`TypeNotFoundException`) | 292 |
| `failed` | 0 |

`attempted == succeeded + unsupported + failed` and
`total == list_capable + skipped_parent_required + skipped_unsupported` must
both hold, and `/api/in_flight` must be zero.

The 292 unsupported type names are compared exactly — set equality, not length —
against the committed fixture
`crates/harness/fixtures/universal-unsupported-aws-emulator-1.0.348.toml`, which
records the emulator version and source commit. A new emulator release is new
evidence: the test fails until the fixture and these literals are refreshed
together after verifying the delta for that release.

### Pagination evidence

The pagination test seeds six `AWS::SNS::Topic` resources through the real SDK
and scans with `--page-size 2 --max-pages 10`. Its evidence is the Cloud Control
list scope: at least two pages at page size 2, with a non-null `next_token` on
every page except the last. Enrichment (`cloud_control_get`) scopes are not
substituted for list pages, so a regression that stops after one list page
cannot pass.

### Live gate

```sh
# The collector must be rebuilt with the AWS feature; otherwise live scans
# fail with "collectors are a compile-time feature".
cargo build --bin aws-inventory --features aws
cargo test -p inventory-harness --test emulator_live -- --ignored --test-threads=1
```

Six tests run serially against one owned emulator: native services, reviewed
Cloud Control types, the universal partition, the combined native + universal
scan, pagination, and the seeded S3 round trip. Fixture seeds use the harness
binary:

```sh
cargo run -p inventory-harness -- seed s3 --bucket rust-fixture-bucket
cargo run -p inventory-harness -- seed cloud-control \
  --prefix inventory-page-topic --count 6
```

## Graph

Universal listing produces observations; the graph layer connects them without a
rule per type. `inventory-engine::export::export_snapshot` can run the reference
resolver before building the snapshot. It is exposed on the CLI as
`aws-inventory export --universal-references` (default off, so existing exports
are unchanged; requires a finalized scan unless `--allow-partial`). The
resolver:

- indexes exact resource identifiers/ARNs from the scan itself, including
  ARN-shaped primary identifiers of resources whose resource id is synthesized
  (`awscc:...`), and their ARN components;
- admits only provider primary identifiers of at least **8 bytes** to the
  unique-identifier tier. Shorter values are provider enums such as `abc` or
  `yes`; they are never indexed, so they cannot fabricate an edge or move the
  ambiguity/unresolved counters. ARN-component resolution keeps its own
  3-byte floor and account/region scoping;
- resolves exact ARN references at any property depth, then the ARN's final
  component (for SNS topics, queues and similar) scoped to the ARN's own account
  and region, plus unique identifier references scoped to the source's account
  and region;
- marks an emitted edge with `properties.reference: "arn"` for an exact observed
  ARN and `properties.reference: "identifier"` for a unique-in-scope identifier
  match (a full identity, an ARN component, or a primary identifier at or above
  the floor); `properties.path` is the property JSON pointer and
  `properties.source` is `universal_reference`. There is no separate confidence
  field: `identifier` means unique within the source's account and region;
- counts ambiguous matches instead of guessing, and never invents
  cross-account/cross-region edges;
- streams scan resources for indexing and candidate passes instead of collecting
  them, so it is not bounded by the export snapshot budget;
- bounds the walk (depth, nodes, payload bytes) and rewrites its synthetic
  `universal_reference` scopes idempotently, so replay produces the same edge
  set.

Universal mode is list-only. Universal list descriptors are mapped with
`map_description(..., complete=false)` (`crates/aws/src/cloud_control/identity.rs`),
so they carry `details_available: false` / `observation_detail: "list_only"`,
no `GetResource` enrichment runs, and no static relationship edges are emitted
(`crates/aws/src/cloud_control/universal/outcomes.rs` leaves the page edge set
empty). The live universal test pins this as `summary.edges == 0`; a non-zero
count means enrichment semantics changed and the evidence must be refreshed.

Reviewed static relationship rules therefore apply to native and per-type
collection only: they run on `complete` descriptors obtained through
`GetResource` (plus parent workflows), never during the universal list pass.
Runtime reference resolution (`export --universal-references`) remains the
universal path for connecting listed resources, as described above, and is
verified by `cargo test -p inventory-engine` (resolver unit tests plus pipeline,
streaming-budget, ARN-component and failed-export immutability tests).
