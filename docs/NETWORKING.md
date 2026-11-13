# Native VPC networking inventory

The `network` selector collects six kinds of VPC configuration through the native
AWS Rust EC2 SDK. The `ec2`, `vpc`, and `ebs` selectors also include these scopes.
Every record has `source = Direct`, the observation time, its account and region,
the tags returned by AWS, and the `Name` tag when available. The implementation
lives in `crates/aws/src/network.rs`.

This is a configuration inventory. A route, an attachment, an active peering
connection, or an ACL allow entry does not establish end-to-end reachability.
Routing decisions, address assignment, ACL order, security groups, endpoint and
IAM policies, return paths, and service state require additional evaluation.

## Resources and API calls

| Resource kind | Read operation | Requested page size | Selected properties |
| --- | --- | --- | --- |
| `aws_vpc_route_table` | `DescribeRouteTables` | Clamp to 5–100 | `route_table_id`, `vpc_id`, `owner_id`, `is_main`, `routes`, `associations`, `propagating_vgw_ids` |
| `aws_vpc_internet_gateway` | `DescribeInternetGateways` | Clamp to 5–1,000 | `internet_gateway_id`, `owner_id`, `attachments` with VPC IDs and attachment states |
| `aws_vpc_nat_gateway` | `DescribeNatGateways` | Clamp to 5–1,000 | `nat_gateway_id`, `vpc_id`, `subnet_id`, `state`, `connectivity_type`, `availability_mode`, `addresses`, `route_table_id`, creation/deletion times and failure details |
| `aws_vpc_endpoint` | `DescribeVpcEndpoints` | Clamp to 1–1,000 | `vpc_endpoint_id`, `vpc_endpoint_type`, `vpc_id`, `owner_id`, `service_name`, `service_region`, `state`, associated route tables/subnets/security groups/interfaces, DNS settings and entries, policy text, explicit service-network and resource-configuration ARNs |
| `aws_vpc_network_acl` | `DescribeNetworkAcls` | Clamp to 5–1,000 | `network_acl_id`, `vpc_id`, `owner_id`, `is_default`, `entries`, `associations` |
| `aws_vpc_peering_connection` | `DescribeVpcPeeringConnections` | Clamp to 5–1,000 | `vpc_peering_connection_id`, status code/message, expiration time, requester/accepter VPC IDs, owner IDs, regions, CIDRs and peering options |

The page limits follow the corresponding AWS API documentation:
[route tables](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRouteTables.html),
[internet gateways](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeInternetGateways.html),
[NAT gateways](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeNatGateways.html),
[VPC endpoints](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeVpcEndpoints.html),
[network ACLs](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeNetworkAcls.html), and
[VPC peering](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeVpcPeeringConnections.html).
Endpoint documentation caps the response at 1,000 items without documenting the
5-item minimum used by the other five operations. No filters omit transitional,
failed, or deleted states still visible through these APIs.

Optional fields remain null when absent. Empty returned lists remain empty; they
are not enriched through unbounded follow-up calls. Timestamp fields are checked
conversions to milliseconds since the Unix epoch.

## Routes: destination, target, and association

Each route remains a row in `route_table.properties.routes`, preserving its
position in the AWS response. The fields are explicit:

| Meaning | JSON fields |
| --- | --- |
| Destination match | `destination_cidr_block`, `destination_ipv6_cidr_block`, `destination_prefix_list_id` |
| Target references | `gateway_id`, `egress_only_internet_gateway_id`, `nat_gateway_id`, `transit_gateway_id`, `local_gateway_id`, `carrier_gateway_id`, `instance_id`, `instance_owner_id`, `network_interface_id`, `vpc_peering_connection_id`, `core_network_arn`, `odb_network_arn`, `ip_address` |
| Route metadata | `state`, `origin` |

These distinctions follow AWS's [Route response schema](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Route.html)
and [route-table guide](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Route_Tables.html).
CIDRs and prefix-list IDs are not treated as next-hop resource nodes. The literal
gateway ID `local` remains in the route JSON and produces no fake gateway ARN.
Unknown future `gateway_id` prefixes and target IP addresses remain available in
the JSON even when the collector cannot assign them a resource identity.

Recognized route targets produce `ROUTES_TO` relationships. Multiple routes to the
same target produce **one** edge for the same source, target, and relation. Its
`properties.evidence` array contains each route's zero-based `route_index`,
`target_field`, destination fields, `state`, and `origin`. Full route details remain
in the route-table resource. This matches the database's edge identity and avoids
losing one route to an otherwise identical target edge. A blackhole route is
retained with its state; it is not converted to a working path.

Association JSON retains `route_table_association_id`, `route_table_id`, `main`,
`subnet_id`, `gateway_id`, association `state`, and `status_message`.
`is_main` means an AWS association reports `main = true`; the association state
remains available during transitions. `MAIN_ROUTE_TABLE_FOR` records that
association. `ASSOCIATED_WITH_SUBNET` requires an explicit subnet ID in the
response. AWS explicitly states that
[implicit main-table associations do not include subnet IDs](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRouteTables.html).
The collector therefore does not invent implicit subnet edges.

For exact route rows using SQLite's JSON support:

```sql
SELECT
    rt.route_table_id,
    json_extract(route.value, '$.destination_cidr_block') AS ipv4_destination,
    json_extract(route.value, '$.destination_ipv6_cidr_block') AS ipv6_destination,
    json_extract(route.value, '$.destination_prefix_list_id') AS prefix_list_id,
    json_extract(route.value, '$.gateway_id') AS gateway_id,
    json_extract(route.value, '$.nat_gateway_id') AS nat_gateway_id,
    json_extract(route.value, '$.state') AS state,
    json_extract(route.value, '$.origin') AS origin
FROM aws_vpc_route_table AS rt
CROSS JOIN json_each(rt.properties, '$.routes') AS route;
```

This query describes the collected route configuration. It does not expand a
prefix list or determine which route would win for a specific packet.

## Relationship contract

| Source | Relation | Target / evidence |
| --- | --- | --- |
| Route table | `IN_VPC` | Reported VPC ID |
| Route table | `MAIN_ROUTE_TABLE_FOR` | VPC, main association ID and state |
| Route table | `ASSOCIATED_WITH_SUBNET` | Explicit subnet association only |
| Route table | `ASSOCIATED_WITH_GATEWAY` | Recognized internet/VPN gateway association |
| Route table | `ROUTES_TO` | Recognized target ID or an AWS-returned target ARN; aggregated route evidence |
| Route table | `PROPAGATES_FROM` | VPN gateway ID listed in `propagating_vgws` |
| Internet gateway | `ATTACHED_TO_VPC` | VPC ID and attachment state |
| NAT gateway | `IN_VPC`, `IN_SUBNET` | Reported VPC/subnet IDs; no subnet edge when absent |
| NAT gateway | `USES_NETWORK_INTERFACE`, `USES_ELASTIC_IP` | Address record's ENI or allocation ID, address index and status |
| NAT gateway | `USES_ROUTE_TABLE` | `route_table_id` only when it has the expected `rtb-` prefix |
| VPC endpoint | `IN_VPC` | Reported VPC ID |
| VPC endpoint | `USES_SUBNET`, `USES_SECURITY_GROUP`, `USES_NETWORK_INTERFACE`, `USES_ROUTE_TABLE` | References present in their respective lists |
| VPC endpoint | `TARGETS_SERVICE_NETWORK`, `TARGETS_RESOURCE_CONFIGURATION` | Explicit ARNs returned by AWS |
| Network ACL | `IN_VPC`, `ASSOCIATED_WITH_SUBNET` | VPC and explicit ACL/subnet association IDs |
| Peering connection | `REQUESTER_VPC`, `ACCEPTER_VPC` | Separate participant IDs, account owners, regions and connection state |

Repeated references share an edge; the evidence array can retain repeated AWS
observations. Every edge retains reference provenance in addition to the
association or route evidence.

### NAT, endpoints, ACLs, and peering details

NAT records include both zonal and regional availability modes. They preserve
`auto_scaling_ips`, `auto_provision_zones`, address status and availability-zone
fields, and attached-appliance descriptions. A regional NAT gateway can lack a
subnet ID. The current [NAT response documentation](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_NatGateway.html)
describes `routeTableId` inconsistently with its field name, so the raw value is
always retained and a route-table edge requires an `rtb-` ID. Appliance descriptions
remain properties; no active data-path claim is inferred from them.

Endpoint records retain their type, DNS entries/options, private-DNS setting,
requester-managed flag, IP-address type, policy document as returned text, errors,
and creation time. A service name is retained as a name; it is not transformed
into an invented ARN. Details are selected from the
[VpcEndpoint schema](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_VpcEndpoint.html).

ACL entries preserve the returned order and each `rule_number`, ingress/egress
flag, protocol string, allow/deny action, IPv4/IPv6 CIDR, port range, and ICMP
type/code, including wildcard values. The collector emits association edges and
does not turn allow rules into access edges. Rule meaning and response fields are
defined by the [NetworkAclEntry schema](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_NetworkAclEntry.html).

Peering records preserve requester/accepter information separately, including
their DNS and ClassicLink peering options when returned. The
[connection schema](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_VpcPeeringConnection.html)
notes that CIDR information is returned only for active peering connections;
missing CIDRs therefore do not prove a VPC has none. Status is preserved without
inferring routes, connectivity, or transitive peering.

## Ownership and unresolved references

Resource ARNs use the AWS EC2 resource-type paths, including `natgateway`, from
the [EC2 service authorization reference](https://docs.aws.amazon.com/service-authorization/latest/reference/list_ec2.html).
For resource types whose response supplies `owner_id`, that account controls the
resource identity even when it differs from the account being scanned. The scan
scope still identifies the account and region through which the observation was
collected.

The NAT and peering response shapes do not supply a top-level resource owner or
resource ARN. Their identities are scoped to the scanning account and region,
and `owner_resolution = assumed_scanning_account` records that limitation.
Peering additionally has `identity_scope = scanning_account_and_region`; its
requester and accepter owners are never substituted for an authoritative owner
of the connection itself. The same peering connection can consequently appear
as observations from both participant scans.

References expose `target_resource_id`, `target_kind`, `target_account_id`,
`target_region`, `owner_resolution`, and `region_resolution`. They distinguish:

- An owner/region explicitly supplied for a peering participant or route target.
- A VPC owner derived from a route table, ACL, or gateway's reported owner, labeled
  with that derivation rather than as a separately reported VPC-owner field.
- An explicit route-table or ACL subnet association whose owner is derived from
  that resource's reported VPC owner, labeled `same_vpc_owner_from_response`.
- An `igw-` route target or gateway association whose internet-gateway owner is
  derived from the route table's reported VPC owner with that same label.
- A missing target owner, which produces a provisional ARN under the scanning
  account and the explicit `assumed_scanning_account` marker.
- A returned ARN, used verbatim with `aws_returned_arn` provenance.

The subnet derivation is supported by AWS's
[shared VPC ownership rules](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-share-limitations.html):
subnets, route tables, ACLs, and attached internet gateways belong to the VPC owner,
while participants can describe them. This derivation is not applied to
ENIs or security groups, which can belong to participants. Endpoint references
without separately established network ownership retain their explicit account
assumption. Association and route references without sufficient owner evidence can therefore
remain unresolved even when another account's resource exists in the inventory.
No additional account traversal or Describe calls are issued only to guess those
owners. A consumer should inspect the provenance before treating a provisional
reference as an established identity.

## Execution, bounds, and partial results

One EC2 client is reused across the six networking scopes for each selected
account/region. The scopes run within the parent collector's bounded concurrency.
Each individual request goes through `ApiExecutor` under the same
account/region/EC2 `read` bucket used by the existing EC2 collector. SDK retries
remain disabled at the SDK layer so the shared executor observes and limits every
attempt. Per-operation retries, throttling cooldowns and cancellation are provided
by that executor; a separate networking selector does not bypass the EC2 limit.

Responses stream one page at a time through the shared bounded event channel.
There is no account-wide resource accumulator. The only networking deduplication
map is for the current page's edges. The following limits are explicit:

| Bound | Limit / behavior |
| --- | --- |
| Scope pages | Configured `max_pages`, maximum 10,000 |
| Scope listed resources | 100,000, enforced before mapping a returned page |
| Nested components per resource | 4,096 across mapped list fields and tags |
| Nested components per page | 65,536 |
| Unique edges per page | Core maximum 100,000 |
| Evidence observations per edge | 4,096 |
| Individual resource/edge properties | Core maximum 1 MiB of serialized JSON |
| Aggregate normalized page | Core maximum 16 MiB of serialized JSON |
| Cursor history | Shared driver: token length and total retained-token byte bounds, plus cycle detection |

Duplicate resource identities within one response page are rejected before
storage rather than becoming a database key conflict. Nested component and property-size violations produce an `invalid_response`
failure for the scope. Invalid pages are discarded as a unit; records are not
silently truncated. Previously delivered pages remain available. A scope reaching
its configured page or resource limit while a cursor remains emits an explicit
failure and does not manufacture a terminal success page.

An empty page with a nonempty continuation token continues. A missing or empty
token is terminal. Access denial, exhausted retries, malformed data, cancellation,
and cursor cycles are recorded as scope failures through the shared pagination
driver. A closed output channel terminates collection. The other networking
scopes can still complete after a service or permission failure in one scope.
Normalization bounds are checked after the SDK decodes a response; they are not
a fixed process-memory guarantee for arbitrary service responses.

These six operations require the corresponding `ec2:Describe...` permissions.
The normal scan also requires its existing caller-identity and optional region
discovery permissions. No create, modify, delete, or acceptance operations are
called by this module.

## Coverage limits and validation

The module does not collect full transit-gateway routing, VPN configurations,
prefix-list entries, firewall policy, DNS resolution, reachability-analysis
results, or every referenced gateway/interface/address resource. A reference to
an uncollected object remains useful evidence but does not imply its full
configuration was collected. The existing EC2 collector provides VPCs, subnets,
security groups, ENIs, instances and volumes; other service collectors supply
their own relationships.

Network module tests exercise route destination/target separation, blackhole
preservation, repeated-target edge aggregation, main and explicit associations,
reported and uncertain owners, regional NATs without subnets, endpoint reference
deduplication, ACL direction/rule metadata, cross-account and cross-region
peering roles, malformed or duplicate identities, component caps, empty-token normalization,
and denial/page-limit behavior after a nonterminal page. These are deterministic
SDK-structure and event-stream tests. They do not assert that a live AWS account
has been queried or that service-side permissions and quotas have been validated.
