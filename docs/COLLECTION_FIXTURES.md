# Collection fixtures and their test universes

> Status: Baseline being replaced. These JSONL event fixtures feed the baseline
> `ingest` command and schema 13 SQL checks; schema 13 is deleted in ROADMAP
> M9.5 and the new path is proven with emulator seed files instead (M2.9). See
> [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

These files are deterministic **synthetic event inputs** for local ingestion and
SQL checks. They contain no observations collected from an AWS account. A source
label exercises that path in the inventory model; it does not establish how the
fixture data was obtained.

The breadth fixture is frozen at the original **v0.3/v0.4 200-type selection**.
The current executable catalog has 397 types across 76 service families and
4,104 provider columns; this historical fixture has not been regenerated to
claim coverage of those additions. Current catalog eligibility and executed
provider evidence are documented separately in
[CLOUDCONTROL_COVERAGE.md](CLOUDCONTROL_COVERAGE.md) and
[CLOUDCONTROL_EVIDENCE.md](CLOUDCONTROL_EVIDENCE.md).

| Fixture | Event pages | Failure events | Resources | Relationships | Expected status |
| --- | ---: | ---: | ---: | ---: | --- |
| breadth-events.jsonl (removed from repository) | 1 | 0 | 200 | 0 | `complete` |
| ecs-events.jsonl (removed from repository) | 1 | 0 | 3 | 2 | `complete` |
| [architecture-events.jsonl](../tests/cli_properties/fixtures/architecture-events.jsonl) | 14 | 1 | 19 | 40 | `partial` |
| [global-events-before.jsonl](../tests/cli_properties/fixtures/global-events-before.jsonl) | 1 | 0 | 7 | 7 | `complete` |
| global-events-after.jsonl (removed from repository) | 1 | 0 | 7 | 7 | `complete` |

`complete` means the fixture's declared scopes ended with terminal pages. It
does not mean an AWS account, service, or region was completely inventoried.
The architecture fixture intentionally preserves a denied scope, a blackhole
route, and an unresolved reference.

## Breadth fixture: the frozen original 200 descriptors

The breadth universe is the exact type list retained in
[cloudcontrol-v04-types.toml](../catalog/cloudcontrol-v04-types.toml): **200
descriptors across 65 catalog service families**. Those descriptors remain in
[cloudcontrol-resources.json](../catalog/cloudcontrol-resources.json), whose
current 397-type selection is larger. There is exactly one fixture resource per
original descriptor, with its descriptor's `kind` and exact CloudFormation
`type_name`. The original view-column definitions remain unchanged. The catalog
maps those preserved type names to their `aws_cc_*` views; the fixture makes no
claim about the 197 added types.

Each resource has:

- An ID of the form `urn:aws-inventory:fixture:<type-slug>`, such as
  `urn:aws-inventory:fixture:aws-ec2-instance`.
- Account `000000000000`, region `us-east-1`, display name `synthetic`, and
  `source = cloud_control`.
- Fixture tags and the fixed observation time `1794441600000`, representing
  2026-11-12 at 00:00 UTC.
- `properties.cloudformation_type`, `resource_identifier`,
  `provider_properties`, and `schema_sha256`.

The `schema_sha256` value is copied exactly from that descriptor's pinned
provider schema. It identifies the schema used to derive the fixture's field
selection; it does not certify the placeholder values as a valid AWS resource
configuration.

`provider_properties` contains harmless placeholders for safe descriptor
columns. Strings begin with `synthetic-`; integers are `1`, numbers are `1.25`,
booleans are `false`, arrays are empty, and objects are empty. A descriptor with
mixed JSON types uses `synthetic-mixed`. Top-level fields listed as write-only or
sensitive, and names covered by the catalog's common sensitive-name list, are
omitted. Empty arrays/objects contain no nested secret or write-only fields.
No real IDs, credentials, environment values, scripts, object contents, or
secret contents are supplied.

The only scope uses collector `synthetic_cloudcontrol`, parent ID `fixture`,
account `000000000000`, region `us-east-1`, and source `cloud_control`. Its only
page is page 1 with `next_token = null`. The page is below the model's 16 MiB
event bound, and each resource is below the 1 MiB property bound.
There are deliberately **zero edges**. Placeholder values resembling reference
fields are not used to invent real resource relationships.

### What the breadth fixture verifies

Successful ingestion and querying can verify event decoding, persistence of the
Cloud Control source, the generic source view, and the original 200
source-specific views' selected columns. It does not exercise rows in the 197
added views. It does not test AWS List/Read operations, IAM
permissions, provider behavior, regional availability, API throttling, or live
account completeness. Native SDK mapper tests and Cloud Control adapter tests
exercise different boundaries.

The `aws_cc_*` views expose Cloud Control provider fields. Native selected-field
views such as `aws_ec2_instance` have a different field contract; the presence of
the same resource kind does not make their column layouts interchangeable.

## Run and query the breadth fixture

The breadth fixture is no longer in the repository (removed with `examples/` in
commit 27ad5349). The commands below need a breadth event file supplied by the
caller in place of `<breadth-events.jsonl>`. They do not require AWS credentials
or the `aws` Cargo feature.

```bash
cargo build --locked

./target/debug/aws-inventory ingest \
  --db breadth.sqlite \
  --input <breadth-events.jsonl> \
  --scan-id synthetic-breadth-001

./target/debug/aws-inventory report \
  --db breadth.sqlite \
  --scan-id synthetic-breadth-001

./target/debug/aws-inventory query --db breadth.sqlite --limit 1 \
  "SELECT count(*) AS resources,
          count(DISTINCT json_extract(properties, '$.cloudformation_type')) AS types
   FROM scan_resources
   WHERE scan_id = 'synthetic-breadth-001'"

./target/debug/aws-inventory query --db breadth.sqlite --limit 1 \
  "SELECT cloudformation_type, instance_id, source
   FROM aws_cc_ec2_instance
   WHERE scan_id = 'synthetic-breadth-001'"
```

Expected: 200 resources, 200 distinct CloudFormation types, zero edges, one
complete scope, and no failed scopes. The EC2 row has the placeholder
`instance_id = "synthetic-instance_id"` and `source = "cloud_control"`.

### Check every frozen-fixture view, including its selected columns

The following optional Bash check also requires `jq`. It joins the frozen type
list to the current catalog and runs `SELECT *` against each original view, so it exercises
the column projections rather than only checking whether a view name exists.
Use the fresh database imported above.

```bash
set -euo pipefail

mapfile -t fixture_views < <(
  jq -r --argjson legacy "$(grep -o '"AWS::[^"]*"' catalog/cloudcontrol-v04-types.toml | jq -s '.')" \
    '.resource_types[]
     | select(.type_name as $type | $legacy | index($type) != null)
     | .view_name' catalog/cloudcontrol-resources.json
)
test "${#fixture_views[@]}" -eq 200

for fixture_view in "${fixture_views[@]}"; do
  [[ "$fixture_view" =~ ^aws_cc_[a-z0-9_]+$ ]]
  ./target/debug/aws-inventory query --db breadth.sqlite --limit 1 \
    "SELECT * FROM \"$fixture_view\"
     WHERE scan_id = 'synthetic-breadth-001'" |
    jq -e 'length == 1
           and .[0].source == "cloud_control"
           and (.[0].id | startswith("urn:aws-inventory:fixture:"))' >/dev/null
done

printf 'Validated %s frozen synthetic catalog views\n' "${#fixture_views[@]}"
```

A missing view, a rejected projection, a duplicate row, an unexpected source, or
an unsuccessful CLI query stops this check. The shell command is an instruction
for verifying a built repository; the fixture file itself is not a record of a
live AWS validation run.

Schema 5 still creates all 397 current views in this database. The 197 new
source-specific views legitimately contain no rows for this fixture. An empty
view here is fixture absence, not a successful AWS enumeration of that type.

The `aws_cc_*` views select the latest observation of their source. The
`scan_resources` and `scan_edges` views retain scan-specific selections. Use a
fresh database for this exact one-row-per-view check; later observations of the
same identities can supersede rows in a latest-source view.

## ECS relationship fixture

The separate ECS fixture has one task-definition revision, one service, and one
ELBv2 target group. All three use fixture URNs and account `000000000000`.
Its `source = direct` records exercise native selected-field views.

The service has two relationships:

| Relation | Synthetic target | Evidence |
| --- | --- | --- |
| `USES_TASK_DEFINITION` | Definition family `synthetic`, revision 7 | The service's task-definition reference |
| `USES_TARGET_GROUP` | `synthetic-target-group` | Primary group, container `synthetic-container`, port 8080 |

Both targets are present, so there are zero unresolved source or target
references. The task definition includes CPU/memory, `awsvpc` mode, ARM64/Linux
runtime platform, one container/port mapping, and explicit value-omission flags.
It has no secret contents or environment values. These are core event fixtures,
not AWS SDK response fixtures or task definitions that can be submitted to ECS.
The service configuration does not establish target registration, health,
filesystem access, public exposure, or packet reachability.

```bash
./target/debug/aws-inventory ingest \
  --db ecs-fixture.sqlite \
  --input <ecs-events.jsonl> \
  --scan-id synthetic-ecs-001

./target/debug/aws-inventory query --db ecs-fixture.sqlite --limit 10 \
  "SELECT family, revision, network_mode, cpu, memory
   FROM aws_ecs_task_definition"

./target/debug/aws-inventory query --db ecs-fixture.sqlite --limit 10 \
  "SELECT relation, source_observed, target_observed
   FROM scan_architecture_edges
   WHERE scan_id = 'synthetic-ecs-001'
   ORDER BY relation"
```

Expected: family `synthetic`, revision 7, network mode `awsvpc`, CPU `"256"`,
memory `"512"`, and two relationship rows with both observation flags set to 1.
`aws_ecs_service.load_balancers` and `target_group_arns` retain the synthetic
container/port configuration and target reference.

## Architecture fixture and exit codes

[ARCHITECTURE_QUERIES.md](ARCHITECTURE_QUERIES.md) describes the existing
architecture fixture, its seven exact route rows, and the unresolved peering
reference. That file is unchanged by the breadth fixture.

The intentional partial scan makes `ingest`, `report`, and `architecture` emit
JSON and exit with code 2. When running with `set -e`, explicitly accept that
expected code:

```bash
./target/debug/aws-inventory ingest \
  --db architecture.sqlite \
  --input tests/cli_properties/fixtures/architecture-events.jsonl \
  --scan-id synthetic-architecture-001 || test "$?" -eq 2
```

The breadth and ECS fixtures finish successfully with exit code 0. Import start
and finish timestamps reflect the time of execution; observation timestamps and
fixture contents are deterministic. Use a new database path or scan ID for
another import. The ingestion command starts and finalizes a scan rather than
resuming an already finalized CLI import.

## Global before/after pair

The after fixture (`global-events-after.jsonl`) was removed with `examples/` in
commit 27ad5349 and is not in the repository. The before fixture remains at
tests/cli_properties/fixtures/global-events-before.jsonl.

The two global fixtures exercise all seven native kinds introduced in v0.4 and
seven architecture relationships. They are small selected-property examples,
not full captured SDK responses. In the later fixture, the bucket policy-status
observation changes, a VPC becomes observed, and the managed policy is no longer
observed. The role's policy reference remains, so the graph preserves an
unresolved endpoint rather than inventing a deletion or silently removing the
relationship. The before graph has three unobserved targets: a DNS name, a VPC
and a Lambda ARN.

The expected resource diff examines eight union identities: one newly observed,
one no longer observed, one changed and five unchanged. All seven relationship
identities remain unchanged when top-level observation timestamps are excluded.
CloudFront → bucket → Lambda forms a two-hop declared path; the Lambda target
is unresolved. This is configuration evidence and does not prove invocation
permission or runtime traffic.

The v0.4 CLI harness validates the new SQL views, findings, bounded comparisons,
path and neighborhood commands, queue submission without credential loading,
and FalkorDB replay for this fixture alongside the earlier architecture and
200-type breadth fixtures.
