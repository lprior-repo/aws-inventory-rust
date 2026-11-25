CREATE VIEW aws_vpc_route_table AS
SELECT r.*, json_extract(properties, '$.route_table_id') AS route_table_id,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.owner_id') AS owner_id,
       json_extract(properties, '$.is_main') AS is_main,
       json_extract(properties, '$.routes') AS routes,
       json_extract(properties, '$.associations') AS associations,
       json_extract(properties, '$.propagating_vgw_ids') AS propagating_vgw_ids
FROM latest_resources r WHERE kind = 'aws_vpc_route_table';

CREATE VIEW aws_vpc_internet_gateway AS
SELECT r.*, json_extract(properties, '$.internet_gateway_id') AS internet_gateway_id,
       json_extract(properties, '$.owner_id') AS owner_id,
       json_extract(properties, '$.attachments') AS attachments
FROM latest_resources r WHERE kind = 'aws_vpc_internet_gateway';

CREATE VIEW aws_vpc_nat_gateway AS
SELECT r.*, json_extract(properties, '$.nat_gateway_id') AS nat_gateway_id,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.subnet_id') AS subnet_id,
       json_extract(properties, '$.state') AS state,
       json_extract(properties, '$.connectivity_type') AS connectivity_type,
       json_extract(properties, '$.availability_mode') AS availability_mode,
       json_extract(properties, '$.addresses') AS addresses,
       json_extract(properties, '$.route_table_id') AS route_table_id
FROM latest_resources r WHERE kind = 'aws_vpc_nat_gateway';

CREATE VIEW aws_vpc_endpoint AS
SELECT r.*, json_extract(properties, '$.vpc_endpoint_id') AS vpc_endpoint_id,
       json_extract(properties, '$.vpc_endpoint_type') AS vpc_endpoint_type,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.owner_id') AS owner_id,
       json_extract(properties, '$.service_name') AS service_name,
       json_extract(properties, '$.state') AS state,
       json_extract(properties, '$.route_table_ids') AS route_table_ids,
       json_extract(properties, '$.subnet_ids') AS subnet_ids,
       json_extract(properties, '$.security_group_ids') AS security_group_ids,
       json_extract(properties, '$.network_interface_ids') AS network_interface_ids,
       json_extract(properties, '$.private_dns_enabled') AS private_dns_enabled,
       json_extract(properties, '$.dns_entries') AS dns_entries
FROM latest_resources r WHERE kind = 'aws_vpc_endpoint';

CREATE VIEW aws_vpc_network_acl AS
SELECT r.*, json_extract(properties, '$.network_acl_id') AS network_acl_id,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.owner_id') AS owner_id,
       json_extract(properties, '$.is_default') AS is_default,
       json_extract(properties, '$.entries') AS entries,
       json_extract(properties, '$.associations') AS associations
FROM latest_resources r WHERE kind = 'aws_vpc_network_acl';

CREATE VIEW aws_vpc_peering_connection AS
SELECT r.*, json_extract(properties, '$.vpc_peering_connection_id') AS vpc_peering_connection_id,
       json_extract(properties, '$.status_code') AS status_code,
       json_extract(properties, '$.status_message') AS status_message,
       json_extract(properties, '$.requester_vpc_id') AS requester_vpc_id,
       json_extract(properties, '$.requester_owner_id') AS requester_owner_id,
       json_extract(properties, '$.requester_region') AS requester_region,
       json_extract(properties, '$.accepter_vpc_id') AS accepter_vpc_id,
       json_extract(properties, '$.accepter_owner_id') AS accepter_owner_id,
       json_extract(properties, '$.accepter_region') AS accepter_region
FROM latest_resources r WHERE kind = 'aws_vpc_peering_connection';

CREATE VIEW aws_elbv2_load_balancer AS
SELECT r.*, json_extract(properties, '$.load_balancer_arn') AS load_balancer_arn,
       json_extract(properties, '$.load_balancer_name') AS load_balancer_name,
       json_extract(properties, '$.type') AS load_balancer_type,
       json_extract(properties, '$.scheme') AS scheme,
       json_extract(properties, '$.dns_name') AS dns_name,
       json_extract(properties, '$.state') AS state,
       json_extract(properties, '$.ip_address_type') AS ip_address_type,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.subnet_ids') AS subnet_ids,
       json_extract(properties, '$.security_group_ids') AS security_group_ids,
       json_extract(properties, '$.availability_zones') AS availability_zones
FROM latest_resources r WHERE kind = 'aws_elbv2_load_balancer';

CREATE VIEW aws_ec2_application_load_balancer AS
SELECT * FROM aws_elbv2_load_balancer WHERE load_balancer_type = 'application';
CREATE VIEW aws_ec2_network_load_balancer AS
SELECT * FROM aws_elbv2_load_balancer WHERE load_balancer_type = 'network';
CREATE VIEW aws_ec2_gateway_load_balancer AS
SELECT * FROM aws_elbv2_load_balancer WHERE load_balancer_type = 'gateway';

CREATE VIEW aws_elbv2_listener AS
SELECT r.*, json_extract(properties, '$.listener_arn') AS listener_arn,
       json_extract(properties, '$.load_balancer_arn') AS load_balancer_arn,
       json_extract(properties, '$.port') AS port,
       json_extract(properties, '$.protocol') AS protocol,
       json_extract(properties, '$.ssl_policy') AS ssl_policy,
       json_extract(properties, '$.alpn_policy') AS alpn_policy,
       json_extract(properties, '$.certificates') AS certificates,
       json_extract(properties, '$.default_actions') AS default_actions,
       json_extract(properties, '$.default_target_group_arns') AS default_target_group_arns
FROM latest_resources r WHERE kind = 'aws_elbv2_listener';

CREATE VIEW aws_elbv2_target_group AS
SELECT r.*, json_extract(properties, '$.target_group_arn') AS target_group_arn,
       json_extract(properties, '$.target_group_name') AS target_group_name,
       json_extract(properties, '$.target_type') AS target_type,
       json_extract(properties, '$.protocol') AS protocol,
       json_extract(properties, '$.protocol_version') AS protocol_version,
       json_extract(properties, '$.port') AS port,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.load_balancer_arns') AS load_balancer_arns,
       json_extract(properties, '$.health_check_enabled') AS health_check_enabled,
       json_extract(properties, '$.health_check_protocol') AS health_check_protocol,
       json_extract(properties, '$.health_check_port') AS health_check_port,
       json_extract(properties, '$.health_check_path') AS health_check_path,
       json_extract(properties, '$.ip_address_type') AS ip_address_type
FROM latest_resources r WHERE kind = 'aws_elbv2_target_group';

CREATE VIEW aws_elbv2_target_health AS
SELECT r.*, json_extract(properties, '$.target_group_arn') AS target_group_arn,
       json_extract(properties, '$.target_type') AS target_type,
       json_extract(properties, '$.target_id') AS target_id,
       json_extract(properties, '$.target_port') AS target_port,
       json_extract(properties, '$.availability_zone') AS availability_zone,
       json_extract(properties, '$.quic_server_id') AS quic_server_id,
       json_extract(properties, '$.health_check_port') AS health_check_port,
       json_extract(properties, '$.target_health_state') AS target_health_state,
       json_extract(properties, '$.target_health_reason') AS target_health_reason,
       json_extract(properties, '$.target_health_description') AS target_health_description
FROM latest_resources r WHERE kind = 'aws_elbv2_target_health';

CREATE VIEW aws_elbv2_rule AS
SELECT r.*, json_extract(properties, '$.rule_arn') AS rule_arn,
       json_extract(properties, '$.listener_arn') AS listener_arn,
       json_extract(properties, '$.priority') AS priority,
       json_extract(properties, '$.is_default') AS is_default,
       json_extract(properties, '$.conditions') AS conditions,
       json_extract(properties, '$.actions') AS actions,
       json_extract(properties, '$.target_group_arns') AS target_group_arns
FROM latest_resources r WHERE kind = 'aws_elbv2_rule';

CREATE VIEW scan_vpc_routes AS
SELECT r.scan_id, r.id AS route_table_arn, r.account_id, r.region, r.source,
       r.observed_at, json_extract(r.properties, '$.route_table_id') AS route_table_id,
       json_extract(r.properties, '$.vpc_id') AS vpc_id, route.key AS route_index,
       json_extract(route.value, '$.destination_cidr_block') AS destination_cidr_block,
       json_extract(route.value, '$.destination_ipv6_cidr_block') AS destination_ipv6_cidr_block,
       json_extract(route.value, '$.destination_prefix_list_id') AS destination_prefix_list_id,
       json_extract(route.value, '$.gateway_id') AS gateway_id,
       json_extract(route.value, '$.nat_gateway_id') AS nat_gateway_id,
       json_extract(route.value, '$.transit_gateway_id') AS transit_gateway_id,
       json_extract(route.value, '$.vpc_peering_connection_id') AS vpc_peering_connection_id,
       json_extract(route.value, '$.network_interface_id') AS network_interface_id,
       json_extract(route.value, '$.instance_id') AS instance_id,
       json_extract(route.value, '$.instance_owner_id') AS instance_owner_id,
       coalesce(json_extract(route.value, '$.vpc_endpoint_id'),
           CASE WHEN substr(json_extract(route.value, '$.gateway_id'),1,5)='vpce-'
                THEN json_extract(route.value, '$.gateway_id') END) AS vpc_endpoint_id,
       json_extract(route.value, '$.egress_only_internet_gateway_id') AS egress_only_internet_gateway_id,
       json_extract(route.value, '$.local_gateway_id') AS local_gateway_id,
       json_extract(route.value, '$.carrier_gateway_id') AS carrier_gateway_id,
       json_extract(route.value, '$.core_network_arn') AS core_network_arn,
       json_extract(route.value, '$.odb_network_arn') AS odb_network_arn,
       json_extract(route.value, '$.ip_address') AS ip_address,
       json_extract(route.value, '$.state') AS state,
       json_extract(route.value, '$.origin') AS origin, route.value AS route_json
FROM scan_resources r,
     json_each(CASE WHEN json_type(r.properties, '$.routes') = 'array'
                   THEN json_extract(r.properties, '$.routes') ELSE '[]' END) route
WHERE r.kind = 'aws_vpc_route_table' AND route.type = 'object';

CREATE VIEW aws_vpc_route AS
SELECT route.* FROM scan_vpc_routes route
JOIN latest_resources current
  ON current.scan_id = route.scan_id AND current.id = route.route_table_arn
WHERE current.kind = 'aws_vpc_route_table';

CREATE VIEW scan_architecture_edges AS
SELECT e.scan_id, e.from_id, src.kind AS from_kind, src.name AS from_name,
       e.to_id, dst.kind AS to_kind, dst.name AS to_name, e.relation,
       e.source, e.observed_at,
       CASE WHEN src.id IS NULL THEN 0 ELSE 1 END AS source_observed,
       CASE WHEN dst.id IS NULL THEN 0 ELSE 1 END AS target_observed,
       json_extract(e.edge_json, '$.properties') AS properties
FROM scan_edges e
LEFT JOIN scan_resources src ON src.scan_id = e.scan_id AND src.id = e.from_id
LEFT JOIN scan_resources dst ON dst.scan_id = e.scan_id AND dst.id = e.to_id;

PRAGMA user_version = 2;
