# Architecture reports and snapshot queries

> Status: Baseline being replaced. It is a worked example of the baseline
> `ingest`, `architecture` and snapshot SQL commands on schema 13; that schema
> and the old `scan` path are deleted in ROADMAP M9.5 and the DAG on real data
> comes from M6. See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

The repository includes a deterministic configuration example in
[`examples/architecture-events.jsonl`](../examples/architecture-events.jsonl).
It exercises the same `CollectionEvent` / `CollectionPage` schema consumed by
the CLI. Every resource and edge is synthetic. The account, IDs, addresses,
DNS names, health state, and permission failure do not describe a live account.

The fixture contains a VPC, three subnets, two security groups, an instance and
a NAT interface; two route tables, an internet gateway, a NAT gateway and a
DynamoDB gateway endpoint; and an Application Load Balancer with a listener,
default and conditional rules, one target group, and one health membership.
The example has selected fields, not a full AWS response dump. Missing optional
fields, such as a NAT address's allocation ID, remain null.

## Run the example locally

From the repository root, use a fresh database path and the commands below.
These commands ingest a local file and query SQLite. They do not contact AWS
and do not require the `aws` Cargo feature or AWS credentials.

```bash
set -e

cargo run --locked -- ingest \
  --db architecture.sqlite \
  --input examples/architecture-events.jsonl \
  --scan-id synthetic-architecture-001 || test "$?" -eq 2

cargo run --locked -- report \
  --db architecture.sqlite \
  --scan-id synthetic-architecture-001 || test "$?" -eq 2

cargo run --locked -- architecture \
  --db architecture.sqlite \
  --scan-id synthetic-architecture-001 \
  --limit 100 || test "$?" -eq 2
```

`ingest`, `report`, and `architecture` print valid JSON and exit with code **2**
for this intentional partial scan. The guards accept that expected code and
keep the sequence usable under `set -e`; other nonzero outcomes stop it. The SQL
examples below return code 0 on success because they return query rows rather
than a scan-status envelope.

Use a new database filename or a new scan ID for another import. The ingestion
command starts and finalizes a scan; it is not an instruction to resume an
already finalized CLI import. The fixture bytes and observation timestamps are
deterministic, while the scan's import start/end times reflect when it is run.

Expected fixture results:

| Measure | Expected value |
| --- | ---: |
| JSONL events | 15 |
| Terminal pages / complete scopes | 14 |
| Explicit failure / failed scope | 1 |
| Selected resources | 19 |
| Selected relationships | 40 |
| Route rows | 7 |
| Blackhole routes | 1 |
| Unobserved edge sources | 0 |
| Unobserved edge targets | 1 |
| Final scan status | `partial` |

The failure is deliberate: the `aws_vpc_peering_connection` scope records
`access_denied`. The missing peering ID is `pcx-unobserved-arch`, referenced by
the route for `10.99.0.0/16`. The fixture does not claim that this denial caused
the route to become a blackhole; these are separate observations.

All observation timestamps are `1794441600000` milliseconds, representing
2026-11-12 at 00:00 UTC. `source = direct` exercises the native source path in
the event schema; it does not turn synthetic data into a live AWS observation.

## Read the architecture report

The `architecture` command requires a finalized scan and accepts complete,
partial, and cancelled outcomes. It returns the scan summary plus eight panels:

| Panel | What it reports |
| --- | --- |
| `resource_counts` | Resource counts grouped by kind and source |
| `coverage` | Counts of complete, failed and incomplete scopes |
| `relationships` | Counts by relation, including unobserved sources and targets |
| `internet_facing_load_balancers` | The reported AWS scheme, load-balancer state and observed listeners |
| `routes` | Individual route rows with separate destination, target and state fields |
| `target_health` | Target membership counts grouped by target group and health state |
| `unresolved_references` | Relationships whose source or target was not observed in this scan |
| `failures` | Recorded failure counts grouped by category |

Each panel is `{ "truncated": boolean, "rows": [...] }`. The row limit applies
independently to each panel and must be between 1 and 1,000. The implementation
probes one extra row to distinguish a full result from truncation. With `--limit
100`, none of this fixture's panels should be truncated. To exercise truncation,
run the same command with `--limit 1`; the routes and resource-count panels then
contain one row and explicitly report truncation.

Panel definitions and their semantics are implemented in
[`src/architecture.rs`](../src/architecture.rs).

## Keep SQL tied to one scan

Use the following views for a reproducible report:

| View | Selection |
| --- | --- |
| `scan_resources` | One preferred observation of each resource within each scan |
| `scan_edges` | One preferred observation of each relationship within each scan |
| `scan_vpc_routes` | One row per route from the selected route-table resources |
| `scan_architecture_edges` | Relationships joined to source/target resources in the same scan, with observation flags |
| `scan_coverage` | Scope progress and failure state for each scan |

Every example below filters `scan_id`. Joins also include `scan_id`, so an
observation from another scan cannot make a missing target appear resolved.
These views are defined in
[`001_inventory.sql`](../crates/engine/sql/001_inventory.sql) and
[`002_architecture.sql`](../crates/engine/sql/002_architecture.sql).

The convenient `aws_*` resource views and `latest_resources` select preferred
observations across scans. They are useful for a current inventory, but adding
`WHERE scan_id = ...` to such a latest view does not reconstruct an older
snapshot whose resources have since been superseded. Use `scan_*` views for
historical comparisons. Source preference is `direct`, then `aws_config`, then
`resource_explorer`; preference does not establish freshness. Inspect `source`
and `observed_at` when interpreting a real scan.

The `query` command takes SQL as its positional argument. It accepts one bounded
read-only `SELECT` or `WITH` statement. Its row/work limits can reject an overly
broad query; the architecture command's per-panel truncation behavior is a
separate interface. See [STORAGE.md](STORAGE.md) for those limits.

### 1. Check coverage before interpreting missing objects

```bash
cargo run --locked -- query --db architecture.sqlite --limit 100 \
  "SELECT collector, parent_id, source, coverage_status, last_page
   FROM scan_coverage
   WHERE scan_id = 'synthetic-architecture-001'
   ORDER BY collector, parent_id"
```

Expected: 15 rows, with fourteen complete scopes and one failed peering scope.
The parent IDs distinguish the load-balancer's listener scope, the listener's
rule scope, and the target group's health scope. A successful parent enumeration
does not certify that every child scope succeeded.

### 2. Inspect routes without losing destinations or state

```bash
cargo run --locked -- query --db architecture.sqlite --limit 100 \
  "SELECT route_table_id, route_index,
          destination_cidr_block, destination_ipv6_cidr_block,
          destination_prefix_list_id, gateway_id, nat_gateway_id,
          vpc_endpoint_id, vpc_peering_connection_id, state, origin
   FROM scan_vpc_routes
   WHERE scan_id = 'synthetic-architecture-001'
   ORDER BY route_table_id, route_index"
```

Expected: seven rows. `rtb-public-arch` contains a local route and an internet
gateway default route. `rtb-private-arch` contains a local route, two routes to
the same NAT gateway, a DynamoDB prefix-list route to the gateway endpoint, and
the peering blackhole route. The prefix-list contents are not expanded here.

| Route table | Index | Destination field/value | Target field/value | State |
| --- | ---: | --- | --- | --- |
| `rtb-public-arch` | 0 | IPv4 `10.42.0.0/16` | `gateway_id = local` | `active` |
| `rtb-public-arch` | 1 | IPv4 `0.0.0.0/0` | `gateway_id = igw-arch` | `active` |
| `rtb-private-arch` | 0 | IPv4 `10.42.0.0/16` | `gateway_id = local` | `active` |
| `rtb-private-arch` | 1 | IPv4 `0.0.0.0/0` | `nat_gateway_id = nat-arch` | `active` |
| `rtb-private-arch` | 2 | IPv4 `10.200.0.0/16` | `nat_gateway_id = nat-arch` | `active` |
| `rtb-private-arch` | 3 | Prefix list `pl-dynamodb-arch` | `gateway_id = vpce-dynamodb-arch` | `active` |
| `rtb-private-arch` | 4 | IPv4 `10.99.0.0/16` | `vpc_peering_connection_id = pcx-unobserved-arch` | `blackhole` |

The route view derives `vpc_endpoint_id` from a `gateway_id` beginning with
`vpce-`, matching the EC2 route response shape. `local` stays a route value; it
does not create a fake gateway resource.

### 3. Find unresolved references and inspect ownership evidence

```bash
cargo run --locked -- query --db architecture.sqlite --limit 100 \
  "SELECT from_kind, from_id, relation, to_kind, to_id,
          source_observed, target_observed,
          json_extract(properties, '$.target_kind') AS referenced_kind,
          json_extract(properties, '$.target_resource_id') AS referenced_id,
          json_extract(properties, '$.owner_resolution') AS owner_resolution
   FROM scan_architecture_edges
   WHERE scan_id = 'synthetic-architecture-001'
     AND (source_observed = 0 OR target_observed = 0)
   ORDER BY from_id, relation, to_id"
```

Expected: one `ROUTES_TO` edge from `rtb-private-arch` to the unobserved peering
reference, whose full target ID is
`arn:aws:ec2:us-east-1:111122223333:vpc-peering-connection/pcx-unobserved-arch`.
The DynamoDB VPC endpoint itself is observed. `to_kind` is null because it comes from an observed target resource;
`referenced_kind` still reports the collector's typed reference evidence.
`target_observed = 0` means this scan did not observe that identity. It does not
establish that the target was deleted or never existed. An assumed account in
the target ARN can also prevent a shared resource from resolving.

### 4. Compare route rows with aggregated graph relationships

```bash
cargo run --locked -- query --db architecture.sqlite --limit 100 \
  "SELECT from_id AS route_table_arn, to_id AS nat_gateway_arn,
          json_array_length(json_extract(properties, '$.evidence'))
            AS route_observations,
          properties
   FROM scan_architecture_edges
   WHERE scan_id = 'synthetic-architecture-001'
     AND relation = 'ROUTES_TO'
     AND json_extract(properties, '$.target_kind') = 'aws_vpc_nat_gateway'"
```

Expected: one graph edge with two route observations. Both destinations remain
in the edge's evidence array and the route table's `routes` property. Graph
identity is `(from_id, to_id, relation)`; a separate edge for every destination
would collide with that identity. A relationship count therefore need not equal
a route-row count.

### 5. Inspect configured listener and rule forwarding

```bash
cargo run --locked -- query --db architecture.sqlite --limit 100 \
  "SELECT e.from_kind, e.from_id, e.to_id AS target_group_arn,
          e.target_observed,
          json_extract(r.properties, '$.priority') AS rule_priority,
          json_extract(r.properties, '$.conditions') AS conditions,
          coalesce(json_extract(r.properties, '$.default_actions'),
                   json_extract(r.properties, '$.actions')) AS actions
   FROM scan_architecture_edges AS e
   JOIN scan_resources AS r
     ON r.scan_id = e.scan_id AND r.id = e.from_id
   WHERE e.scan_id = 'synthetic-architecture-001'
     AND e.relation = 'FORWARDS_TO'
   ORDER BY e.from_kind, e.from_id, e.to_id"
```

Expected: three configured-forwarding rows: the listener default action, the
default rule, and the conditional host-header rule. All reference the observed
target group. These are distinct source resources, so they are distinct graph
relationships. Original action arrays retain the target weights and order;
an edge by itself does not evaluate conditions, priorities, or effective traffic.

### 6. Join registered targets to observed resources

```bash
cargo run --locked -- query --db architecture.sqlite --limit 100 \
  "SELECT json_extract(h.properties, '$.target_group_arn') AS target_group_arn,
          json_extract(h.properties, '$.target_type') AS target_type,
          json_extract(h.properties, '$.target_id') AS target_id,
          json_extract(h.properties, '$.target_port') AS target_port,
          json_extract(h.properties, '$.target_health_state') AS health_state,
          e.target_observed, target.kind AS observed_target_kind,
          target.name AS observed_target_name
   FROM scan_resources AS h
   LEFT JOIN scan_architecture_edges AS e
     ON e.scan_id = h.scan_id AND e.from_id = h.id AND e.relation = 'TARGETS'
   LEFT JOIN scan_resources AS target
     ON target.scan_id = h.scan_id AND target.id = e.to_id
   WHERE h.scan_id = 'synthetic-architecture-001'
     AND h.kind = 'aws_elbv2_target_health'
   ORDER BY target_group_arn, target_id, target_port"
```

Expected: one healthy instance registration on port 8080, joined to
`aws_ec2_instance` named `synthetic-app`. Membership identity includes the target
group, target ID, port, availability zone, and QUIC identifier; it is not just
the target's resource ID. The health state is an observation, not evidence that
a public client can reach the application.

## Topology semantics and limitations

The fixture intentionally retains distinct kinds of evidence:

- `IN_VPC`, subnet/security-group references, route-table associations, and
  gateway attachments describe configuration relationships.
- `ROUTES_TO` retains active and blackhole route evidence, including destinations.
- `FORWARDS_TO` describes configured ELB forwarding; rules and weights remain
  properties for later evaluation.
- `HAS_TARGET` describes registration membership, and `TARGETS` resolves that
  membership to a typed target reference where possible.
- Failure and observation flags describe collection coverage independently of
  the configuration's state.

The example's load balancer is marked `internet-facing` and has a synthetic
HTTP listener. Its security-group examples are deliberately incomplete, and
the fixture does not evaluate ACLs, return paths, DNS, IAM, endpoint policies,
or packet delivery. No relationship is a reachability result. Even a complete
set of collection scopes would be observations taken over time, not an atomic
AWS-wide snapshot. The mapping contracts and AWS documentation references are
in [NETWORKING.md](NETWORKING.md) and [LOAD_BALANCING.md](LOAD_BALANCING.md).

For a local graph-export example, explicitly preserve the partial outcome:

```bash
cargo run --locked -- graph-export \
  --db architecture.sqlite \
  --scan-id synthetic-architecture-001 \
  --graph synthetic_architecture \
  --output architecture-graph.jsonl \
  --allow-partial
```

This writes a local graph plan. It does not connect to FalkorDB. Use a fresh
output path; export does not overwrite an existing plan. The unresolved peering
reference remains a placeholder with provenance, not a newly discovered AWS
resource.
