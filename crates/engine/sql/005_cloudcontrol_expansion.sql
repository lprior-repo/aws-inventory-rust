CREATE VIEW aws_cc_ec2_carrier_gateway AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CarrierGatewayId') AS "carrier_gateway_id",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::CarrierGateway';

CREATE VIEW aws_cc_ec2_client_vpn_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AuthenticationOptions') AS "authentication_options",
       json_extract(r.provider_properties, '$.AuthorizationPolicy') AS "authorization_policy",
       json_extract(r.provider_properties, '$.ClientCidrBlock') AS "client_cidr_block",
       json_extract(r.provider_properties, '$.ClientConnectOptions') AS "client_connect_options",
       json_extract(r.provider_properties, '$.ClientLoginBannerOptions') AS "client_login_banner_options",
       json_extract(r.provider_properties, '$.ClientRouteEnforcementOptions') AS "client_route_enforcement_options",
       json_extract(r.provider_properties, '$.ConnectionLogOptions') AS "connection_log_options",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DevicePostureOptions') AS "device_posture_options",
       json_extract(r.provider_properties, '$.DisconnectOnSessionTimeout') AS "disconnect_on_session_timeout",
       json_extract(r.provider_properties, '$.DnsServers') AS "dns_servers",
       json_extract(r.provider_properties, '$.EndpointIpAddressType') AS "endpoint_ip_address_type",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SelfServicePortal') AS "self_service_portal",
       json_extract(r.provider_properties, '$.ServerCertificateArn') AS "server_certificate_arn",
       json_extract(r.provider_properties, '$.SessionTimeoutHours') AS "session_timeout_hours",
       json_extract(r.provider_properties, '$.SplitTunnel') AS "split_tunnel",
       json_extract(r.provider_properties, '$.TagSpecifications') AS "tag_specifications",
       json_extract(r.provider_properties, '$.TrafficIpAddressType') AS "traffic_ip_address_type",
       json_extract(r.provider_properties, '$.TransitGatewayConfiguration') AS "transit_gateway_configuration",
       json_extract(r.provider_properties, '$.TransportProtocol') AS "transport_protocol",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id",
       json_extract(r.provider_properties, '$.VpnPort') AS "vpn_port"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::ClientVpnEndpoint';

CREATE VIEW aws_cc_ec2_ec2_fleet AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Context') AS "context",
       json_extract(r.provider_properties, '$.ExcessCapacityTerminationPolicy') AS "excess_capacity_termination_policy",
       json_extract(r.provider_properties, '$.FleetId') AS "fleet_id",
       json_extract(r.provider_properties, '$.LaunchTemplateConfigs') AS "launch_template_configs",
       json_extract(r.provider_properties, '$.OnDemandOptions') AS "on_demand_options",
       json_extract(r.provider_properties, '$.ReplaceUnhealthyInstances') AS "replace_unhealthy_instances",
       json_extract(r.provider_properties, '$.ReservedCapacityOptions') AS "reserved_capacity_options",
       json_extract(r.provider_properties, '$.SpotOptions') AS "spot_options",
       json_extract(r.provider_properties, '$.TagSpecifications') AS "tag_specifications",
       json_extract(r.provider_properties, '$.TargetCapacitySpecification') AS "target_capacity_specification",
       json_extract(r.provider_properties, '$.TerminateInstancesWithExpiration') AS "terminate_instances_with_expiration",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.ValidFrom') AS "valid_from",
       json_extract(r.provider_properties, '$.ValidUntil') AS "valid_until"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::EC2Fleet';

CREATE VIEW aws_cc_ec2_eip_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllocationId') AS "allocation_id",
       json_extract(r.provider_properties, '$.EIP') AS "eip",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.InstanceId') AS "instance_id",
       json_extract(r.provider_properties, '$.NetworkInterfaceId') AS "network_interface_id",
       json_extract(r.provider_properties, '$.PrivateIpAddress') AS "private_ip_address"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::EIPAssociation';

CREATE VIEW aws_cc_ec2_fpga_image AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreateTime') AS "create_time",
       json_extract(r.provider_properties, '$.DataRetentionSupport') AS "data_retention_support",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FpgaImageGlobalId') AS "fpga_image_global_id",
       json_extract(r.provider_properties, '$.FpgaImageId') AS "fpga_image_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.Public') AS "public",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UpdateTime') AS "update_time"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::FpgaImage';

CREATE VIEW aws_cc_ec2_instance_event_window AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CronExpression') AS "cron_expression",
       json_extract(r.provider_properties, '$.InstanceEventWindowId') AS "instance_event_window_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TimeRanges') AS "time_ranges"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::InstanceEventWindow';

CREATE VIEW aws_cc_ec2_key_pair AS
SELECT r.*,
       json_extract(r.provider_properties, '$.KeyFingerprint') AS "key_fingerprint",
       json_extract(r.provider_properties, '$.KeyName') AS "key_name",
       json_extract(r.provider_properties, '$.KeyPairId') AS "key_pair_id",
       json_extract(r.provider_properties, '$.KeyType') AS "key_type",
       json_extract(r.provider_properties, '$.PublicKeyMaterial') AS "public_key_material",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::KeyPair';

CREATE VIEW aws_cc_ec2_local_gateway_route AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DestinationCidrBlock') AS "destination_cidr_block",
       json_extract(r.provider_properties, '$.LocalGatewayRouteTableId') AS "local_gateway_route_table_id",
       json_extract(r.provider_properties, '$.LocalGatewayVirtualInterfaceGroupId') AS "local_gateway_virtual_interface_group_id",
       json_extract(r.provider_properties, '$.NetworkInterfaceId') AS "network_interface_id",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::LocalGatewayRoute';

CREATE VIEW aws_cc_ec2_local_gateway_route_table AS
SELECT r.*,
       json_extract(r.provider_properties, '$.LocalGatewayId') AS "local_gateway_id",
       json_extract(r.provider_properties, '$.LocalGatewayRouteTableArn') AS "local_gateway_route_table_arn",
       json_extract(r.provider_properties, '$.LocalGatewayRouteTableId') AS "local_gateway_route_table_id",
       json_extract(r.provider_properties, '$.Mode') AS "mode",
       json_extract(r.provider_properties, '$.OutpostArn') AS "outpost_arn",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::LocalGatewayRouteTable';

CREATE VIEW aws_cc_ec2_local_gateway_route_table_vpc_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.LocalGatewayId') AS "local_gateway_id",
       json_extract(r.provider_properties, '$.LocalGatewayRouteTableId') AS "local_gateway_route_table_id",
       json_extract(r.provider_properties, '$.LocalGatewayRouteTableVpcAssociationId') AS "local_gateway_route_table_vpc_association_id",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::LocalGatewayRouteTableVPCAssociation';

CREATE VIEW aws_cc_ec2_local_gateway_route_table_virtual_interface_group_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.LocalGatewayId') AS "local_gateway_id",
       json_extract(r.provider_properties, '$.LocalGatewayRouteTableArn') AS "local_gateway_route_table_arn",
       json_extract(r.provider_properties, '$.LocalGatewayRouteTableId') AS "local_gateway_route_table_id",
       json_extract(r.provider_properties, '$.LocalGatewayRouteTableVirtualInterfaceGroupAssociationId') AS "local_gateway_route_table_virtual_interface_group_association_id",
       json_extract(r.provider_properties, '$.LocalGatewayVirtualInterfaceGroupId') AS "local_gateway_virtual_interface_group_id",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::LocalGatewayRouteTableVirtualInterfaceGroupAssociation';

CREATE VIEW aws_cc_ec2_local_gateway_virtual_interface AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConfigurationState') AS "configuration_state",
       json_extract(r.provider_properties, '$.LocalAddress') AS "local_address",
       json_extract(r.provider_properties, '$.LocalBgpAsn') AS "local_bgp_asn",
       json_extract(r.provider_properties, '$.LocalGatewayId') AS "local_gateway_id",
       json_extract(r.provider_properties, '$.LocalGatewayVirtualInterfaceGroupId') AS "local_gateway_virtual_interface_group_id",
       json_extract(r.provider_properties, '$.LocalGatewayVirtualInterfaceId') AS "local_gateway_virtual_interface_id",
       json_extract(r.provider_properties, '$.OutpostLagId') AS "outpost_lag_id",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.PeerAddress') AS "peer_address",
       json_extract(r.provider_properties, '$.PeerBgpAsn') AS "peer_bgp_asn",
       json_extract(r.provider_properties, '$.PeerBgpAsnExtended') AS "peer_bgp_asn_extended",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Vlan') AS "vlan"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::LocalGatewayVirtualInterface';

CREATE VIEW aws_cc_ec2_local_gateway_virtual_interface_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConfigurationState') AS "configuration_state",
       json_extract(r.provider_properties, '$.LocalBgpAsn') AS "local_bgp_asn",
       json_extract(r.provider_properties, '$.LocalBgpAsnExtended') AS "local_bgp_asn_extended",
       json_extract(r.provider_properties, '$.LocalGatewayId') AS "local_gateway_id",
       json_extract(r.provider_properties, '$.LocalGatewayVirtualInterfaceGroupArn') AS "local_gateway_virtual_interface_group_arn",
       json_extract(r.provider_properties, '$.LocalGatewayVirtualInterfaceGroupId') AS "local_gateway_virtual_interface_group_id",
       json_extract(r.provider_properties, '$.LocalGatewayVirtualInterfaceIds') AS "local_gateway_virtual_interface_ids",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::LocalGatewayVirtualInterfaceGroup';

CREATE VIEW aws_cc_ec2_network_insights_access_scope AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreatedDate') AS "created_date",
       json_extract(r.provider_properties, '$.ExcludePaths') AS "exclude_paths",
       json_extract(r.provider_properties, '$.MatchPaths') AS "match_paths",
       json_extract(r.provider_properties, '$.NetworkInsightsAccessScopeArn') AS "network_insights_access_scope_arn",
       json_extract(r.provider_properties, '$.NetworkInsightsAccessScopeId') AS "network_insights_access_scope_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UpdatedDate') AS "updated_date"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NetworkInsightsAccessScope';

CREATE VIEW aws_cc_ec2_network_insights_access_scope_analysis AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AnalyzedEniCount') AS "analyzed_eni_count",
       json_extract(r.provider_properties, '$.EndDate') AS "end_date",
       json_extract(r.provider_properties, '$.FindingsFound') AS "findings_found",
       json_extract(r.provider_properties, '$.NetworkInsightsAccessScopeAnalysisArn') AS "network_insights_access_scope_analysis_arn",
       json_extract(r.provider_properties, '$.NetworkInsightsAccessScopeAnalysisId') AS "network_insights_access_scope_analysis_id",
       json_extract(r.provider_properties, '$.NetworkInsightsAccessScopeId') AS "network_insights_access_scope_id",
       json_extract(r.provider_properties, '$.StartDate') AS "start_date",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusMessage') AS "status_message",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NetworkInsightsAccessScopeAnalysis';

CREATE VIEW aws_cc_ec2_network_insights_analysis AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdditionalAccounts') AS "additional_accounts",
       json_extract(r.provider_properties, '$.AlternatePathHints') AS "alternate_path_hints",
       json_extract(r.provider_properties, '$.Explanations') AS "explanations",
       json_extract(r.provider_properties, '$.FilterInArns') AS "filter_in_arns",
       json_extract(r.provider_properties, '$.FilterOutArns') AS "filter_out_arns",
       json_extract(r.provider_properties, '$.ForwardPathComponents') AS "forward_path_components",
       json_extract(r.provider_properties, '$.NetworkInsightsAnalysisArn') AS "network_insights_analysis_arn",
       json_extract(r.provider_properties, '$.NetworkInsightsAnalysisId') AS "network_insights_analysis_id",
       json_extract(r.provider_properties, '$.NetworkInsightsPathId') AS "network_insights_path_id",
       json_extract(r.provider_properties, '$.NetworkPathFound') AS "network_path_found",
       json_extract(r.provider_properties, '$.ReturnPathComponents') AS "return_path_components",
       json_extract(r.provider_properties, '$.StartDate') AS "start_date",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusMessage') AS "status_message",
       json_extract(r.provider_properties, '$.SuggestedAccounts') AS "suggested_accounts",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NetworkInsightsAnalysis';

CREATE VIEW aws_cc_ec2_network_interface_attachment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AttachmentId') AS "attachment_id",
       json_extract(r.provider_properties, '$.DeleteOnTermination') AS "delete_on_termination",
       json_extract(r.provider_properties, '$.DeviceIndex') AS "device_index",
       json_extract(r.provider_properties, '$.EnaQueueCount') AS "ena_queue_count",
       json_extract(r.provider_properties, '$.EnaSrdSpecification') AS "ena_srd_specification",
       json_extract(r.provider_properties, '$.InstanceId') AS "instance_id",
       json_extract(r.provider_properties, '$.NetworkInterfaceId') AS "network_interface_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NetworkInterfaceAttachment';

CREATE VIEW aws_cc_ec2_network_performance_metric_subscription AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Destination') AS "destination",
       json_extract(r.provider_properties, '$.Metric') AS "metric",
       json_extract(r.provider_properties, '$.Source') AS "provider_source",
       json_extract(r.provider_properties, '$.Statistic') AS "statistic"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NetworkPerformanceMetricSubscription';

CREATE VIEW aws_cc_ec2_security_group_vpc_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.GroupId') AS "group_id",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.StateReason') AS "state_reason",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id",
       json_extract(r.provider_properties, '$.VpcOwnerId') AS "vpc_owner_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::SecurityGroupVpcAssociation';

CREATE VIEW aws_cc_ec2_spot_fleet AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.SpotFleetRequestConfigData') AS "spot_fleet_request_config_data",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::SpotFleet';

CREATE VIEW aws_cc_ec2_subnet_cidr_block AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.IpSource') AS "ip_source",
       json_extract(r.provider_properties, '$.Ipv6AddressAttribute') AS "ipv6_address_attribute",
       json_extract(r.provider_properties, '$.Ipv6CidrBlock') AS "ipv6_cidr_block",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::SubnetCidrBlock';

CREATE VIEW aws_cc_ec2_traffic_mirror_filter AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.NetworkServices') AS "network_services",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TrafficMirrorFilter';

CREATE VIEW aws_cc_ec2_traffic_mirror_filter_rule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DestinationCidrBlock') AS "destination_cidr_block",
       json_extract(r.provider_properties, '$.DestinationPortRange') AS "destination_port_range",
       json_extract(r.provider_properties, '$.Protocol') AS "protocol",
       json_extract(r.provider_properties, '$.RuleAction') AS "rule_action",
       json_extract(r.provider_properties, '$.RuleNumber') AS "rule_number",
       json_extract(r.provider_properties, '$.SourceCidrBlock') AS "source_cidr_block",
       json_extract(r.provider_properties, '$.SourcePortRange') AS "source_port_range",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrafficDirection') AS "traffic_direction",
       json_extract(r.provider_properties, '$.TrafficMirrorFilterId') AS "traffic_mirror_filter_id",
       json_extract(r.provider_properties, '$.TrafficMirrorFilterRuleId') AS "traffic_mirror_filter_rule_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TrafficMirrorFilterRule';

CREATE VIEW aws_cc_ec2_transit_gateway_attachment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Options') AS "options",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGatewayAttachment';

CREATE VIEW aws_cc_ec2_transit_gateway_connect AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Options') AS "options",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayAttachmentId') AS "transit_gateway_attachment_id",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id",
       json_extract(r.provider_properties, '$.TransportTransitGatewayAttachmentId') AS "transport_transit_gateway_attachment_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGatewayConnect';

CREATE VIEW aws_cc_ec2_transit_gateway_connect_peer AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConnectPeerConfiguration') AS "connect_peer_configuration",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayAttachmentId') AS "transit_gateway_attachment_id",
       json_extract(r.provider_properties, '$.TransitGatewayConnectPeerId') AS "transit_gateway_connect_peer_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGatewayConnectPeer';

CREATE VIEW aws_cc_ec2_transit_gateway_multicast_domain AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Options') AS "options",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id",
       json_extract(r.provider_properties, '$.TransitGatewayMulticastDomainArn') AS "transit_gateway_multicast_domain_arn",
       json_extract(r.provider_properties, '$.TransitGatewayMulticastDomainId') AS "transit_gateway_multicast_domain_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGatewayMulticastDomain';

CREATE VIEW aws_cc_ec2_transit_gateway_policy_table AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id",
       json_extract(r.provider_properties, '$.TransitGatewayPolicyTableId') AS "transit_gateway_policy_table_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGatewayPolicyTable';

CREATE VIEW aws_cc_ec2_vpc_block_public_access_exclusion AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ExclusionId') AS "exclusion_id",
       json_extract(r.provider_properties, '$.InternetGatewayExclusionMode') AS "internet_gateway_exclusion_mode",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPCBlockPublicAccessExclusion';

CREATE VIEW aws_cc_ec2_vpcdhcp_options_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DhcpOptionsId') AS "dhcp_options_id",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPCDHCPOptionsAssociation';

CREATE VIEW aws_cc_ec2_vpc_endpoint_connection_notification AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConnectionEvents') AS "connection_events",
       json_extract(r.provider_properties, '$.ConnectionNotificationArn') AS "connection_notification_arn",
       json_extract(r.provider_properties, '$.ServiceId') AS "service_id",
       json_extract(r.provider_properties, '$.VPCEndpointConnectionNotificationId') AS "vpc_endpoint_connection_notification_id",
       json_extract(r.provider_properties, '$.VPCEndpointId') AS "vpc_endpoint_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPCEndpointConnectionNotification';

CREATE VIEW aws_cc_ec2_vpc_endpoint_service_permissions AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllowedPrincipals') AS "allowed_principals",
       json_extract(r.provider_properties, '$.ServiceId') AS "service_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPCEndpointServicePermissions';

CREATE VIEW aws_cc_ec2_vpc_gateway_attachment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AttachmentType') AS "attachment_type",
       json_extract(r.provider_properties, '$.InternetGatewayId') AS "internet_gateway_id",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id",
       json_extract(r.provider_properties, '$.VpnGatewayId') AS "vpn_gateway_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPCGatewayAttachment';

CREATE VIEW aws_cc_ec2_vpn_connection_route AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DestinationCidrBlock') AS "destination_cidr_block",
       json_extract(r.provider_properties, '$.VpnConnectionId') AS "vpn_connection_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPNConnectionRoute';

CREATE VIEW aws_cc_ec2_verified_access_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationDomain') AS "application_domain",
       json_extract(r.provider_properties, '$.AttachmentType') AS "attachment_type",
       json_extract(r.provider_properties, '$.CidrOptions') AS "cidr_options",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DeviceValidationDomain') AS "device_validation_domain",
       json_extract(r.provider_properties, '$.DomainCertificateArn') AS "domain_certificate_arn",
       json_extract(r.provider_properties, '$.EndpointDomain') AS "endpoint_domain",
       json_extract(r.provider_properties, '$.EndpointDomainPrefix') AS "endpoint_domain_prefix",
       json_extract(r.provider_properties, '$.EndpointType') AS "endpoint_type",
       json_extract(r.provider_properties, '$.LastUpdatedTime') AS "last_updated_time",
       json_extract(r.provider_properties, '$.LoadBalancerOptions') AS "load_balancer_options",
       json_extract(r.provider_properties, '$.NetworkInterfaceOptions') AS "network_interface_options",
       json_extract(r.provider_properties, '$.PolicyDocument') AS "policy_document",
       json_extract(r.provider_properties, '$.PolicyEnabled') AS "policy_enabled",
       json_extract(r.provider_properties, '$.RdsOptions') AS "rds_options",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SseSpecification') AS "sse_specification",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VerifiedAccessEndpointId') AS "verified_access_endpoint_id",
       json_extract(r.provider_properties, '$.VerifiedAccessGroupId') AS "verified_access_group_id",
       json_extract(r.provider_properties, '$.VerifiedAccessInstanceId') AS "verified_access_instance_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VerifiedAccessEndpoint';

CREATE VIEW aws_cc_ec2_verified_access_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.LastUpdatedTime') AS "last_updated_time",
       json_extract(r.provider_properties, '$.Owner') AS "owner",
       json_extract(r.provider_properties, '$.PolicyDocument') AS "policy_document",
       json_extract(r.provider_properties, '$.PolicyEnabled') AS "policy_enabled",
       json_extract(r.provider_properties, '$.SseSpecification') AS "sse_specification",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VerifiedAccessGroupArn') AS "verified_access_group_arn",
       json_extract(r.provider_properties, '$.VerifiedAccessGroupId') AS "verified_access_group_id",
       json_extract(r.provider_properties, '$.VerifiedAccessInstanceId') AS "verified_access_instance_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VerifiedAccessGroup';

CREATE VIEW aws_cc_ec2_verified_access_instance AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CidrEndpointsCustomSubDomain') AS "cidr_endpoints_custom_sub_domain",
       json_extract(r.provider_properties, '$.CidrEndpointsCustomSubDomainNameServers') AS "cidr_endpoints_custom_sub_domain_name_servers",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FipsEnabled') AS "fips_enabled",
       json_extract(r.provider_properties, '$.LastUpdatedTime') AS "last_updated_time",
       json_extract(r.provider_properties, '$.LoggingConfigurations') AS "logging_configurations",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VerifiedAccessInstanceId') AS "verified_access_instance_id",
       json_extract(r.provider_properties, '$.VerifiedAccessTrustProviderIds') AS "verified_access_trust_provider_ids",
       json_extract(r.provider_properties, '$.VerifiedAccessTrustProviders') AS "verified_access_trust_providers"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VerifiedAccessInstance';

CREATE VIEW aws_cc_ec2_verified_access_trust_provider AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DeviceOptions') AS "device_options",
       json_extract(r.provider_properties, '$.DeviceTrustProviderType') AS "device_trust_provider_type",
       json_extract(r.provider_properties, '$.LastUpdatedTime') AS "last_updated_time",
       json_extract(r.provider_properties, '$.NativeApplicationOidcOptions') AS "native_application_oidc_options",
       json_extract(r.provider_properties, '$.OidcOptions') AS "oidc_options",
       json_extract(r.provider_properties, '$.PolicyReferenceName') AS "policy_reference_name",
       json_extract(r.provider_properties, '$.SseSpecification') AS "sse_specification",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrustProviderType') AS "trust_provider_type",
       json_extract(r.provider_properties, '$.UserTrustProviderType') AS "user_trust_provider_type",
       json_extract(r.provider_properties, '$.VerifiedAccessTrustProviderId') AS "verified_access_trust_provider_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VerifiedAccessTrustProvider';

CREATE VIEW aws_cc_ec2_volume_attachment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Device') AS "device",
       json_extract(r.provider_properties, '$.EbsCardIndex') AS "ebs_card_index",
       json_extract(r.provider_properties, '$.InstanceId') AS "instance_id",
       json_extract(r.provider_properties, '$.VolumeId') AS "volume_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VolumeAttachment';

CREATE VIEW aws_cc_rds_cluster_snapshot AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllocatedStorage') AS "allocated_storage",
       json_extract(r.provider_properties, '$.AvailabilityZones') AS "availability_zones",
       json_extract(r.provider_properties, '$.ClusterCreateTime') AS "cluster_create_time",
       json_extract(r.provider_properties, '$.DBClusterIdentifier') AS "db_cluster_identifier",
       json_extract(r.provider_properties, '$.DBClusterSnapshotArn') AS "db_cluster_snapshot_arn",
       json_extract(r.provider_properties, '$.DBClusterSnapshotIdentifier') AS "db_cluster_snapshot_identifier",
       json_extract(r.provider_properties, '$.DbClusterResourceId') AS "db_cluster_resource_id",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.EngineMode') AS "engine_mode",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.IAMDatabaseAuthenticationEnabled') AS "iam_database_authentication_enabled",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LicenseModel') AS "license_model",
       json_extract(r.provider_properties, '$.MasterUsername') AS "master_username",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.SnapshotCreateTime') AS "snapshot_create_time",
       json_extract(r.provider_properties, '$.SnapshotType') AS "snapshot_type",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StorageEncrypted') AS "storage_encrypted",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::ClusterSnapshot';

CREATE VIEW aws_cc_rds_custom_db_engine_version AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DBEngineVersionArn') AS "db_engine_version_arn",
       json_extract(r.provider_properties, '$.DatabaseInstallationFiles') AS "database_installation_files",
       json_extract(r.provider_properties, '$.DatabaseInstallationFilesS3BucketName') AS "database_installation_files_s3_bucket_name",
       json_extract(r.provider_properties, '$.DatabaseInstallationFilesS3Prefix') AS "database_installation_files_s3_prefix",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.ImageId') AS "image_id",
       json_extract(r.provider_properties, '$.KMSKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::CustomDBEngineVersion';

CREATE VIEW aws_cc_rds_db_shard_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ComputeRedundancy') AS "compute_redundancy",
       json_extract(r.provider_properties, '$.DBClusterIdentifier') AS "db_cluster_identifier",
       json_extract(r.provider_properties, '$.DBShardGroupIdentifier') AS "db_shard_group_identifier",
       json_extract(r.provider_properties, '$.DBShardGroupResourceId') AS "db_shard_group_resource_id",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.MaxACU') AS "max_acu",
       json_extract(r.provider_properties, '$.PubliclyAccessible') AS "publicly_accessible",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBShardGroup';

CREATE VIEW aws_cc_rds_event_subscription AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Enabled') AS "enabled",
       json_extract(r.provider_properties, '$.EventCategories') AS "event_categories",
       json_extract(r.provider_properties, '$.SnsTopicArn') AS "sns_topic_arn",
       json_extract(r.provider_properties, '$.SourceIds') AS "source_ids",
       json_extract(r.provider_properties, '$.SourceType') AS "source_type",
       json_extract(r.provider_properties, '$.SubscriptionName') AS "subscription_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::EventSubscription';

CREATE VIEW aws_cc_rds_integration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdditionalEncryptionContext') AS "additional_encryption_context",
       json_extract(r.provider_properties, '$.CreateTime') AS "create_time",
       json_extract(r.provider_properties, '$.DataFilter') AS "data_filter",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.IntegrationArn') AS "integration_arn",
       json_extract(r.provider_properties, '$.IntegrationName') AS "integration_name",
       json_extract(r.provider_properties, '$.KMSKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.SourceArn') AS "source_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetArn') AS "target_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::Integration';

CREATE VIEW aws_cc_neptune_db_cluster_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Family') AS "family",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Neptune::DBClusterParameterGroup';

CREATE VIEW aws_cc_neptune_db_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Family') AS "family",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Neptune::DBParameterGroup';

CREATE VIEW aws_cc_neptune_event_subscription AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Enabled') AS "enabled",
       json_extract(r.provider_properties, '$.EventCategories') AS "event_categories",
       json_extract(r.provider_properties, '$.SnsTopicArn') AS "sns_topic_arn",
       json_extract(r.provider_properties, '$.SourceIds') AS "source_ids",
       json_extract(r.provider_properties, '$.SourceType') AS "source_type",
       json_extract(r.provider_properties, '$.SubscriptionName') AS "subscription_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Neptune::EventSubscription';

CREATE VIEW aws_cc_doc_db_event_subscription AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Enabled') AS "enabled",
       json_extract(r.provider_properties, '$.EventCategories') AS "event_categories",
       json_extract(r.provider_properties, '$.SnsTopicArn') AS "sns_topic_arn",
       json_extract(r.provider_properties, '$.SourceIds') AS "source_ids",
       json_extract(r.provider_properties, '$.SourceType') AS "source_type",
       json_extract(r.provider_properties, '$.SubscriptionName') AS "subscription_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DocDB::EventSubscription';

CREATE VIEW aws_cc_elasti_cache_serverless_cache_snapshot AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ARN') AS "provider_arn",
       json_extract(r.provider_properties, '$.BytesUsedForCache') AS "bytes_used_for_cache",
       json_extract(r.provider_properties, '$.CreateTime') AS "create_time",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.ServerlessCacheConfiguration') AS "serverless_cache_configuration",
       json_extract(r.provider_properties, '$.ServerlessCacheName') AS "serverless_cache_name",
       json_extract(r.provider_properties, '$.ServerlessCacheSnapshotName') AS "serverless_cache_snapshot_name",
       json_extract(r.provider_properties, '$.SnapshotType') AS "snapshot_type",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::ServerlessCacheSnapshot';

CREATE VIEW aws_cc_elasti_cache_snapshot AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AutoMinorVersionUpgrade') AS "auto_minor_version_upgrade",
       json_extract(r.provider_properties, '$.AutomaticFailover') AS "automatic_failover",
       json_extract(r.provider_properties, '$.CacheClusterCreateTime') AS "cache_cluster_create_time",
       json_extract(r.provider_properties, '$.CacheClusterId') AS "cache_cluster_id",
       json_extract(r.provider_properties, '$.CacheNodeType') AS "cache_node_type",
       json_extract(r.provider_properties, '$.CacheParameterGroupName') AS "cache_parameter_group_name",
       json_extract(r.provider_properties, '$.CacheSubnetGroupName') AS "cache_subnet_group_name",
       json_extract(r.provider_properties, '$.DataTiering') AS "data_tiering",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.NodeSnapshots') AS "node_snapshots",
       json_extract(r.provider_properties, '$.NumCacheNodes') AS "num_cache_nodes",
       json_extract(r.provider_properties, '$.NumNodeGroups') AS "num_node_groups",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PreferredAvailabilityZone') AS "preferred_availability_zone",
       json_extract(r.provider_properties, '$.PreferredMaintenanceWindow') AS "preferred_maintenance_window",
       json_extract(r.provider_properties, '$.ReplicationGroupDescription') AS "replication_group_description",
       json_extract(r.provider_properties, '$.ReplicationGroupId') AS "replication_group_id",
       json_extract(r.provider_properties, '$.SnapshotName') AS "snapshot_name",
       json_extract(r.provider_properties, '$.SnapshotRetentionLimit') AS "snapshot_retention_limit",
       json_extract(r.provider_properties, '$.SnapshotSource') AS "snapshot_source",
       json_extract(r.provider_properties, '$.SnapshotStatus') AS "snapshot_status",
       json_extract(r.provider_properties, '$.SnapshotWindow') AS "snapshot_window",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TopicArn') AS "topic_arn",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::Snapshot';

CREATE VIEW aws_cc_elasti_cache_user AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UserId') AS "user_id",
       json_extract(r.provider_properties, '$.UserName') AS "user_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::User';

CREATE VIEW aws_cc_elasti_cache_user_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UserGroupId') AS "user_group_id",
       json_extract(r.provider_properties, '$.UserIds') AS "user_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::UserGroup';

CREATE VIEW aws_cc_memory_db_snapshot AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ClusterConfiguration') AS "cluster_configuration",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.DataTiering') AS "data_tiering",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.SnapshotName') AS "snapshot_name",
       json_extract(r.provider_properties, '$.Source') AS "provider_source",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MemoryDB::Snapshot';

CREATE VIEW aws_cc_memory_db_user AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UserName') AS "user_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MemoryDB::User';

CREATE VIEW aws_cc_f_sx_backup AS
SELECT r.*,
       json_extract(r.provider_properties, '$.BackupId') AS "backup_id",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.FileSystemId') AS "file_system_id",
       json_extract(r.provider_properties, '$.Lifecycle') AS "lifecycle",
       json_extract(r.provider_properties, '$.ResourceARN') AS "resource_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::FSx::Backup';

CREATE VIEW aws_cc_f_sx_file_cache AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.DNSName') AS "dns_name",
       json_extract(r.provider_properties, '$.DataRepositoryAssociationIds') AS "data_repository_association_ids",
       json_extract(r.provider_properties, '$.FileCacheId') AS "file_cache_id",
       json_extract(r.provider_properties, '$.FileCacheType') AS "file_cache_type",
       json_extract(r.provider_properties, '$.FileCacheTypeVersion') AS "file_cache_type_version",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.Lifecycle') AS "lifecycle",
       json_extract(r.provider_properties, '$.LustreConfiguration') AS "lustre_configuration",
       json_extract(r.provider_properties, '$.NetworkInterfaceIds') AS "network_interface_ids",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.ResourceARN') AS "resource_arn",
       json_extract(r.provider_properties, '$.StorageCapacity') AS "storage_capacity",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::FSx::FileCache';

CREATE VIEW aws_cc_f_sx_snapshot AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ResourceARN') AS "resource_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VolumeId') AS "volume_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::FSx::Snapshot';

CREATE VIEW aws_cc_ecr_pull_through_cache_rule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.EcrRepositoryPrefix') AS "ecr_repository_prefix",
       json_extract(r.provider_properties, '$.UpstreamRegistryUrl') AS "upstream_registry_url",
       json_extract(r.provider_properties, '$.UpstreamRepositoryPrefix') AS "upstream_repository_prefix"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECR::PullThroughCacheRule';

CREATE VIEW aws_cc_ecr_registry_policy AS
SELECT r.*,
       json_extract(r.provider_properties, '$.PolicyText') AS "policy_text",
       json_extract(r.provider_properties, '$.RegistryId') AS "registry_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECR::RegistryPolicy';

CREATE VIEW aws_cc_ecr_repository_creation_template AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AppliedFor') AS "applied_for",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.CustomRoleArn') AS "custom_role_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EncryptionConfiguration') AS "encryption_configuration",
       json_extract(r.provider_properties, '$.ImageTagMutability') AS "image_tag_mutability",
       json_extract(r.provider_properties, '$.ImageTagMutabilityExclusionFilters') AS "image_tag_mutability_exclusion_filters",
       json_extract(r.provider_properties, '$.LifecyclePolicy') AS "lifecycle_policy",
       json_extract(r.provider_properties, '$.Prefix') AS "prefix",
       json_extract(r.provider_properties, '$.RepositoryPolicy') AS "repository_policy",
       json_extract(r.provider_properties, '$.ResourceTags') AS "resource_tags",
       json_extract(r.provider_properties, '$.UpdatedAt') AS "updated_at"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECR::RepositoryCreationTemplate';

CREATE VIEW aws_cc_auto_scaling_lifecycle_hook AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutoScalingGroupName') AS "auto_scaling_group_name",
       json_extract(r.provider_properties, '$.DefaultResult') AS "default_result",
       json_extract(r.provider_properties, '$.HeartbeatTimeout') AS "heartbeat_timeout",
       json_extract(r.provider_properties, '$.LifecycleHookName') AS "lifecycle_hook_name",
       json_extract(r.provider_properties, '$.LifecycleTransition') AS "lifecycle_transition",
       json_extract(r.provider_properties, '$.NotificationMetadata') AS "notification_metadata",
       json_extract(r.provider_properties, '$.NotificationTargetARN') AS "notification_target_arn",
       json_extract(r.provider_properties, '$.RoleARN') AS "role_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AutoScaling::LifecycleHook';

CREATE VIEW aws_cc_auto_scaling_scaling_policy AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdjustmentType') AS "adjustment_type",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AutoScalingGroupName') AS "auto_scaling_group_name",
       json_extract(r.provider_properties, '$.Cooldown') AS "cooldown",
       json_extract(r.provider_properties, '$.EstimatedInstanceWarmup') AS "estimated_instance_warmup",
       json_extract(r.provider_properties, '$.MetricAggregationType') AS "metric_aggregation_type",
       json_extract(r.provider_properties, '$.MinAdjustmentMagnitude') AS "min_adjustment_magnitude",
       json_extract(r.provider_properties, '$.PolicyName') AS "policy_name",
       json_extract(r.provider_properties, '$.PolicyType') AS "policy_type",
       json_extract(r.provider_properties, '$.PredictiveScalingConfiguration') AS "predictive_scaling_configuration",
       json_extract(r.provider_properties, '$.ScalingAdjustment') AS "scaling_adjustment",
       json_extract(r.provider_properties, '$.StepAdjustments') AS "step_adjustments",
       json_extract(r.provider_properties, '$.TargetTrackingConfiguration') AS "target_tracking_configuration"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AutoScaling::ScalingPolicy';

CREATE VIEW aws_cc_auto_scaling_scheduled_action AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutoScalingGroupName') AS "auto_scaling_group_name",
       json_extract(r.provider_properties, '$.DesiredCapacity') AS "desired_capacity",
       json_extract(r.provider_properties, '$.EndTime') AS "end_time",
       json_extract(r.provider_properties, '$.MaxSize') AS "max_size",
       json_extract(r.provider_properties, '$.MinSize') AS "min_size",
       json_extract(r.provider_properties, '$.Recurrence') AS "recurrence",
       json_extract(r.provider_properties, '$.ScheduledActionName') AS "scheduled_action_name",
       json_extract(r.provider_properties, '$.StartTime') AS "start_time",
       json_extract(r.provider_properties, '$.TimeZone') AS "time_zone"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AutoScaling::ScheduledAction';

CREATE VIEW aws_cc_batch_consumable_resource AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AvailableQuantity') AS "available_quantity",
       json_extract(r.provider_properties, '$.ConsumableResourceArn') AS "consumable_resource_arn",
       json_extract(r.provider_properties, '$.ConsumableResourceName') AS "consumable_resource_name",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.InUseQuantity') AS "in_use_quantity",
       json_extract(r.provider_properties, '$.ResourceType') AS "resource_type",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TotalQuantity') AS "total_quantity"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Batch::ConsumableResource';

CREATE VIEW aws_cc_batch_quota_share AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CapacityLimits') AS "capacity_limits",
       json_extract(r.provider_properties, '$.JobQueue') AS "job_queue",
       json_extract(r.provider_properties, '$.PreemptionConfiguration') AS "preemption_configuration",
       json_extract(r.provider_properties, '$.QuotaShareArn') AS "quota_share_arn",
       json_extract(r.provider_properties, '$.QuotaShareName') AS "quota_share_name",
       json_extract(r.provider_properties, '$.ResourceSharingConfiguration') AS "resource_sharing_configuration",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Batch::QuotaShare';

CREATE VIEW aws_cc_batch_scheduling_policy AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.FairsharePolicy') AS "fairshare_policy",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.QuotaSharePolicy') AS "quota_share_policy",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Batch::SchedulingPolicy';

CREATE VIEW aws_cc_batch_service_environment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CapacityLimits') AS "capacity_limits",
       json_extract(r.provider_properties, '$.ServiceEnvironmentArn') AS "service_environment_arn",
       json_extract(r.provider_properties, '$.ServiceEnvironmentName') AS "service_environment_name",
       json_extract(r.provider_properties, '$.ServiceEnvironmentType') AS "service_environment_type",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Batch::ServiceEnvironment';

CREATE VIEW aws_cc_api_gateway_api_key AS
SELECT r.*,
       json_extract(r.provider_properties, '$.APIKeyId') AS "api_key_id",
       json_extract(r.provider_properties, '$.CustomerId') AS "customer_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Enabled') AS "enabled",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.StageKeys') AS "stage_keys",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::ApiKey';

CREATE VIEW aws_cc_api_gateway_client_certificate AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ClientCertificateId') AS "client_certificate_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::ClientCertificate';

CREATE VIEW aws_cc_api_gateway_domain_name_v2 AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CertificateArn') AS "certificate_arn",
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.DomainNameArn') AS "domain_name_arn",
       json_extract(r.provider_properties, '$.DomainNameId') AS "domain_name_id",
       json_extract(r.provider_properties, '$.EndpointAccessMode') AS "endpoint_access_mode",
       json_extract(r.provider_properties, '$.EndpointConfiguration') AS "endpoint_configuration",
       json_extract(r.provider_properties, '$.Policy') AS "policy",
       json_extract(r.provider_properties, '$.RoutingMode') AS "routing_mode",
       json_extract(r.provider_properties, '$.SecurityPolicy') AS "security_policy",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::DomainNameV2';

CREATE VIEW aws_cc_api_gateway_domain_name_access_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessAssociationSource') AS "access_association_source",
       json_extract(r.provider_properties, '$.AccessAssociationSourceType') AS "access_association_source_type",
       json_extract(r.provider_properties, '$.DomainNameAccessAssociationArn') AS "domain_name_access_association_arn",
       json_extract(r.provider_properties, '$.DomainNameArn') AS "domain_name_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::DomainNameAccessAssociation';

CREATE VIEW aws_cc_dax_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AvailabilityZones') AS "availability_zones",
       json_extract(r.provider_properties, '$.ClusterDiscoveryEndpoint') AS "cluster_discovery_endpoint",
       json_extract(r.provider_properties, '$.ClusterDiscoveryEndpointURL') AS "cluster_discovery_endpoint_url",
       json_extract(r.provider_properties, '$.ClusterEndpointEncryptionType') AS "cluster_endpoint_encryption_type",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.IAMRoleARN') AS "iam_role_arn",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.NodeType') AS "node_type",
       json_extract(r.provider_properties, '$.NotificationTopicARN') AS "notification_topic_arn",
       json_extract(r.provider_properties, '$.ParameterGroupName') AS "parameter_group_name",
       json_extract(r.provider_properties, '$.PreferredMaintenanceWindow') AS "preferred_maintenance_window",
       json_extract(r.provider_properties, '$.ReplicationFactor') AS "replication_factor",
       json_extract(r.provider_properties, '$.SSESpecification') AS "sse_specification",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SubnetGroupName') AS "subnet_group_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DAX::Cluster';

CREATE VIEW aws_cc_dax_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.ParameterGroupName') AS "parameter_group_name",
       json_extract(r.provider_properties, '$.ParameterNameValues') AS "parameter_name_values"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DAX::ParameterGroup';

CREATE VIEW aws_cc_dax_subnet_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.SubnetGroupName') AS "subnet_group_name",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DAX::SubnetGroup';

CREATE VIEW aws_cc_open_search_serverless_vpc_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::OpenSearchServerless::VpcEndpoint';

CREATE VIEW aws_cc_sage_maker_domain AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AppNetworkAccessType') AS "app_network_access_type",
       json_extract(r.provider_properties, '$.AppSecurityGroupManagement') AS "app_security_group_management",
       json_extract(r.provider_properties, '$.AuthMode') AS "auth_mode",
       json_extract(r.provider_properties, '$.DefaultSpaceSettings') AS "default_space_settings",
       json_extract(r.provider_properties, '$.DefaultUserSettings') AS "default_user_settings",
       json_extract(r.provider_properties, '$.DomainArn') AS "domain_arn",
       json_extract(r.provider_properties, '$.DomainId') AS "domain_id",
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.DomainSettings') AS "domain_settings",
       json_extract(r.provider_properties, '$.HomeEfsFileSystemCreation') AS "home_efs_file_system_creation",
       json_extract(r.provider_properties, '$.HomeEfsFileSystemId') AS "home_efs_file_system_id",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.SecurityGroupIdForDomainBoundary') AS "security_group_id_for_domain_boundary",
       json_extract(r.provider_properties, '$.SingleSignOnApplicationArn') AS "single_sign_on_application_arn",
       json_extract(r.provider_properties, '$.SingleSignOnManagedApplicationInstanceId') AS "single_sign_on_managed_application_instance_id",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.TagPropagation') AS "tag_propagation",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Url') AS "url",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Domain';

CREATE VIEW aws_cc_sage_maker_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DeploymentConfig') AS "deployment_config",
       json_extract(r.provider_properties, '$.EndpointArn') AS "endpoint_arn",
       json_extract(r.provider_properties, '$.EndpointConfigName') AS "endpoint_config_name",
       json_extract(r.provider_properties, '$.EndpointName') AS "endpoint_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Endpoint';

CREATE VIEW aws_cc_sage_maker_endpoint_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AsyncInferenceConfig') AS "async_inference_config",
       json_extract(r.provider_properties, '$.DataCaptureConfig') AS "data_capture_config",
       json_extract(r.provider_properties, '$.EnableNetworkIsolation') AS "enable_network_isolation",
       json_extract(r.provider_properties, '$.EndpointConfigArn') AS "endpoint_config_arn",
       json_extract(r.provider_properties, '$.EndpointConfigName') AS "endpoint_config_name",
       json_extract(r.provider_properties, '$.ExecutionRoleArn') AS "execution_role_arn",
       json_extract(r.provider_properties, '$.ExplainerConfig') AS "explainer_config",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.MetricsConfig') AS "metrics_config",
       json_extract(r.provider_properties, '$.ProductionVariants') AS "production_variants",
       json_extract(r.provider_properties, '$.ShadowProductionVariants') AS "shadow_production_variants",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcConfig') AS "vpc_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::EndpointConfig';

CREATE VIEW aws_cc_sage_maker_model AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Containers') AS "containers",
       json_extract(r.provider_properties, '$.EnableNetworkIsolation') AS "enable_network_isolation",
       json_extract(r.provider_properties, '$.ExecutionRoleArn') AS "execution_role_arn",
       json_extract(r.provider_properties, '$.InferenceExecutionConfig') AS "inference_execution_config",
       json_extract(r.provider_properties, '$.ModelArn') AS "model_arn",
       json_extract(r.provider_properties, '$.ModelName') AS "model_name",
       json_extract(r.provider_properties, '$.PrimaryContainer') AS "primary_container",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcConfig') AS "vpc_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Model';

CREATE VIEW aws_cc_sage_maker_model_package_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.ModelPackageGroupArn') AS "model_package_group_arn",
       json_extract(r.provider_properties, '$.ModelPackageGroupDescription') AS "model_package_group_description",
       json_extract(r.provider_properties, '$.ModelPackageGroupName') AS "model_package_group_name",
       json_extract(r.provider_properties, '$.ModelPackageGroupPolicy') AS "model_package_group_policy",
       json_extract(r.provider_properties, '$.ModelPackageGroupStatus') AS "model_package_group_status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::ModelPackageGroup';

CREATE VIEW aws_cc_sage_maker_notebook_instance AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AcceleratorTypes') AS "accelerator_types",
       json_extract(r.provider_properties, '$.AdditionalCodeRepositories') AS "additional_code_repositories",
       json_extract(r.provider_properties, '$.DefaultCodeRepository') AS "default_code_repository",
       json_extract(r.provider_properties, '$.DirectInternetAccess') AS "direct_internet_access",
       json_extract(r.provider_properties, '$.InstanceMetadataServiceConfiguration') AS "instance_metadata_service_configuration",
       json_extract(r.provider_properties, '$.InstanceType') AS "instance_type",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LifecycleConfigName') AS "lifecycle_config_name",
       json_extract(r.provider_properties, '$.NotebookInstanceArn') AS "notebook_instance_arn",
       json_extract(r.provider_properties, '$.NotebookInstanceName') AS "notebook_instance_name",
       json_extract(r.provider_properties, '$.PlatformIdentifier') AS "platform_identifier",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.RootAccess') AS "root_access",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VolumeSizeInGB') AS "volume_size_in_gb"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::NotebookInstance';

CREATE VIEW aws_cc_sage_maker_notebook_instance_lifecycle_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.NotebookInstanceLifecycleConfigArn') AS "notebook_instance_lifecycle_config_arn",
       json_extract(r.provider_properties, '$.NotebookInstanceLifecycleConfigName') AS "notebook_instance_lifecycle_config_name",
       json_extract(r.provider_properties, '$.OnCreate') AS "on_create",
       json_extract(r.provider_properties, '$.OnStart') AS "on_start",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::NotebookInstanceLifecycleConfig';

CREATE VIEW aws_cc_sage_maker_app_image_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AppImageConfigArn') AS "app_image_config_arn",
       json_extract(r.provider_properties, '$.AppImageConfigName') AS "app_image_config_name",
       json_extract(r.provider_properties, '$.CodeEditorAppImageConfig') AS "code_editor_app_image_config",
       json_extract(r.provider_properties, '$.JupyterLabAppImageConfig') AS "jupyter_lab_app_image_config",
       json_extract(r.provider_properties, '$.KernelGatewayImageConfig') AS "kernel_gateway_image_config",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::AppImageConfig';

CREATE VIEW aws_cc_sage_maker_image AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ImageArn') AS "image_arn",
       json_extract(r.provider_properties, '$.ImageDescription') AS "image_description",
       json_extract(r.provider_properties, '$.ImageDisplayName') AS "image_display_name",
       json_extract(r.provider_properties, '$.ImageName') AS "image_name",
       json_extract(r.provider_properties, '$.ImageRoleArn') AS "image_role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Image';

CREATE VIEW aws_cc_sage_maker_feature_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EventTimeFeatureName') AS "event_time_feature_name",
       json_extract(r.provider_properties, '$.FeatureDefinitions') AS "feature_definitions",
       json_extract(r.provider_properties, '$.FeatureGroupName') AS "feature_group_name",
       json_extract(r.provider_properties, '$.FeatureGroupStatus') AS "feature_group_status",
       json_extract(r.provider_properties, '$.OfflineStoreConfig') AS "offline_store_config",
       json_extract(r.provider_properties, '$.OnlineStoreConfig') AS "online_store_config",
       json_extract(r.provider_properties, '$.RecordIdentifierFeatureName') AS "record_identifier_feature_name",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.ThroughputConfig') AS "throughput_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::FeatureGroup';

CREATE VIEW aws_cc_sage_maker_pipeline AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ParallelismConfiguration') AS "parallelism_configuration",
       json_extract(r.provider_properties, '$.PipelineDescription') AS "pipeline_description",
       json_extract(r.provider_properties, '$.PipelineDisplayName') AS "pipeline_display_name",
       json_extract(r.provider_properties, '$.PipelineName') AS "pipeline_name",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Pipeline';

CREATE VIEW aws_cc_sage_maker_monitoring_schedule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.EndpointName') AS "endpoint_name",
       json_extract(r.provider_properties, '$.FailureReason') AS "failure_reason",
       json_extract(r.provider_properties, '$.LastModifiedTime') AS "last_modified_time",
       json_extract(r.provider_properties, '$.LastMonitoringExecutionSummary') AS "last_monitoring_execution_summary",
       json_extract(r.provider_properties, '$.MonitoringScheduleArn') AS "monitoring_schedule_arn",
       json_extract(r.provider_properties, '$.MonitoringScheduleConfig') AS "monitoring_schedule_config",
       json_extract(r.provider_properties, '$.MonitoringScheduleName') AS "monitoring_schedule_name",
       json_extract(r.provider_properties, '$.MonitoringScheduleStatus') AS "monitoring_schedule_status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::MonitoringSchedule';

CREATE VIEW aws_cc_sage_maker_project AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.ProjectArn') AS "project_arn",
       json_extract(r.provider_properties, '$.ProjectDescription') AS "project_description",
       json_extract(r.provider_properties, '$.ProjectId') AS "project_id",
       json_extract(r.provider_properties, '$.ProjectName') AS "project_name",
       json_extract(r.provider_properties, '$.ProjectStatus') AS "project_status",
       json_extract(r.provider_properties, '$.ServiceCatalogProvisionedProductDetails') AS "service_catalog_provisioned_product_details",
       json_extract(r.provider_properties, '$.ServiceCatalogProvisioningDetails') AS "service_catalog_provisioning_details",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TemplateProviderDetails') AS "template_provider_details"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Project';

CREATE VIEW aws_cc_sage_maker_workteam AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.MemberDefinitions') AS "member_definitions",
       json_extract(r.provider_properties, '$.NotificationConfiguration') AS "notification_configuration",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.WorkteamArn') AS "workteam_arn",
       json_extract(r.provider_properties, '$.WorkteamName') AS "workteam_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Workteam';

CREATE VIEW aws_cc_sage_maker_workforce AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CognitoConfig') AS "cognito_config",
       json_extract(r.provider_properties, '$.IpAddressType') AS "ip_address_type",
       json_extract(r.provider_properties, '$.OidcConfig') AS "oidc_config",
       json_extract(r.provider_properties, '$.SourceIpConfig') AS "source_ip_config",
       json_extract(r.provider_properties, '$.SubDomain') AS "sub_domain",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.WorkforceArn') AS "workforce_arn",
       json_extract(r.provider_properties, '$.WorkforceName') AS "workforce_name",
       json_extract(r.provider_properties, '$.WorkforceVpcConfig') AS "workforce_vpc_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Workforce';

CREATE VIEW aws_cc_sage_maker_code_repository AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CodeRepositoryArn') AS "code_repository_arn",
       json_extract(r.provider_properties, '$.CodeRepositoryName') AS "code_repository_name",
       json_extract(r.provider_properties, '$.GitConfig') AS "git_config",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::CodeRepository';

CREATE VIEW aws_cc_sage_maker_model_quality_job_definition AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.JobDefinitionArn') AS "job_definition_arn",
       json_extract(r.provider_properties, '$.JobDefinitionName') AS "job_definition_name",
       json_extract(r.provider_properties, '$.JobResources') AS "job_resources",
       json_extract(r.provider_properties, '$.ModelQualityAppSpecification') AS "model_quality_app_specification",
       json_extract(r.provider_properties, '$.ModelQualityBaselineConfig') AS "model_quality_baseline_config",
       json_extract(r.provider_properties, '$.ModelQualityJobInput') AS "model_quality_job_input",
       json_extract(r.provider_properties, '$.ModelQualityJobOutputConfig') AS "model_quality_job_output_config",
       json_extract(r.provider_properties, '$.NetworkConfig') AS "network_config",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.StoppingCondition') AS "stopping_condition",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::ModelQualityJobDefinition';

CREATE VIEW aws_cc_sage_maker_data_quality_job_definition AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.DataQualityAppSpecification') AS "data_quality_app_specification",
       json_extract(r.provider_properties, '$.DataQualityBaselineConfig') AS "data_quality_baseline_config",
       json_extract(r.provider_properties, '$.DataQualityJobInput') AS "data_quality_job_input",
       json_extract(r.provider_properties, '$.DataQualityJobOutputConfig') AS "data_quality_job_output_config",
       json_extract(r.provider_properties, '$.JobDefinitionArn') AS "job_definition_arn",
       json_extract(r.provider_properties, '$.JobDefinitionName') AS "job_definition_name",
       json_extract(r.provider_properties, '$.JobResources') AS "job_resources",
       json_extract(r.provider_properties, '$.NetworkConfig') AS "network_config",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.StoppingCondition') AS "stopping_condition",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::DataQualityJobDefinition';

CREATE VIEW aws_cc_sage_maker_model_bias_job_definition AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.JobDefinitionArn') AS "job_definition_arn",
       json_extract(r.provider_properties, '$.JobDefinitionName') AS "job_definition_name",
       json_extract(r.provider_properties, '$.JobResources') AS "job_resources",
       json_extract(r.provider_properties, '$.ModelBiasAppSpecification') AS "model_bias_app_specification",
       json_extract(r.provider_properties, '$.ModelBiasBaselineConfig') AS "model_bias_baseline_config",
       json_extract(r.provider_properties, '$.ModelBiasJobInput') AS "model_bias_job_input",
       json_extract(r.provider_properties, '$.ModelBiasJobOutputConfig') AS "model_bias_job_output_config",
       json_extract(r.provider_properties, '$.NetworkConfig') AS "network_config",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.StoppingCondition') AS "stopping_condition",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::ModelBiasJobDefinition';

CREATE VIEW aws_cc_sage_maker_model_explainability_job_definition AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.JobDefinitionArn') AS "job_definition_arn",
       json_extract(r.provider_properties, '$.JobDefinitionName') AS "job_definition_name",
       json_extract(r.provider_properties, '$.JobResources') AS "job_resources",
       json_extract(r.provider_properties, '$.ModelExplainabilityAppSpecification') AS "model_explainability_app_specification",
       json_extract(r.provider_properties, '$.ModelExplainabilityBaselineConfig') AS "model_explainability_baseline_config",
       json_extract(r.provider_properties, '$.ModelExplainabilityJobInput') AS "model_explainability_job_input",
       json_extract(r.provider_properties, '$.ModelExplainabilityJobOutputConfig') AS "model_explainability_job_output_config",
       json_extract(r.provider_properties, '$.NetworkConfig') AS "network_config",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.StoppingCondition') AS "stopping_condition",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::ModelExplainabilityJobDefinition';

CREATE VIEW aws_cc_sage_maker_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutoScaling') AS "auto_scaling",
       json_extract(r.provider_properties, '$.ClusterArn') AS "cluster_arn",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.ClusterRole') AS "cluster_role",
       json_extract(r.provider_properties, '$.ClusterStatus') AS "cluster_status",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.FailureMessage') AS "failure_message",
       json_extract(r.provider_properties, '$.InstanceGroups') AS "instance_groups",
       json_extract(r.provider_properties, '$.NodeProvisioningMode') AS "node_provisioning_mode",
       json_extract(r.provider_properties, '$.NodeRecovery') AS "node_recovery",
       json_extract(r.provider_properties, '$.Orchestrator') AS "orchestrator",
       json_extract(r.provider_properties, '$.RestrictedInstanceGroups') AS "restricted_instance_groups",
       json_extract(r.provider_properties, '$.RestrictedInstanceGroupsConfig') AS "restricted_instance_groups_config",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TieredStorageConfig') AS "tiered_storage_config",
       json_extract(r.provider_properties, '$.VpcConfig') AS "vpc_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::Cluster';

CREATE VIEW aws_cc_sage_maker_inference_component AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.EndpointArn') AS "endpoint_arn",
       json_extract(r.provider_properties, '$.EndpointName') AS "endpoint_name",
       json_extract(r.provider_properties, '$.FailureReason') AS "failure_reason",
       json_extract(r.provider_properties, '$.InferenceComponentArn') AS "inference_component_arn",
       json_extract(r.provider_properties, '$.InferenceComponentName') AS "inference_component_name",
       json_extract(r.provider_properties, '$.InferenceComponentStatus') AS "inference_component_status",
       json_extract(r.provider_properties, '$.LastModifiedTime') AS "last_modified_time",
       json_extract(r.provider_properties, '$.RuntimeConfig') AS "runtime_config",
       json_extract(r.provider_properties, '$.Specification') AS "specification",
       json_extract(r.provider_properties, '$.Specifications') AS "specifications",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VariantName') AS "variant_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::InferenceComponent';

CREATE VIEW aws_cc_sage_maker_mlflow_tracking_server AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ArtifactStoreUri') AS "artifact_store_uri",
       json_extract(r.provider_properties, '$.AutomaticModelRegistration') AS "automatic_model_registration",
       json_extract(r.provider_properties, '$.MlflowVersion') AS "mlflow_version",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrackingServerArn') AS "tracking_server_arn",
       json_extract(r.provider_properties, '$.TrackingServerName') AS "tracking_server_name",
       json_extract(r.provider_properties, '$.TrackingServerSize') AS "tracking_server_size",
       json_extract(r.provider_properties, '$.WeeklyMaintenanceWindowStart') AS "weekly_maintenance_window_start"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::MlflowTrackingServer';

CREATE VIEW aws_cc_sage_maker_model_card AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreatedBy') AS "created_by",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.LastModifiedBy') AS "last_modified_by",
       json_extract(r.provider_properties, '$.LastModifiedTime') AS "last_modified_time",
       json_extract(r.provider_properties, '$.ModelCardArn') AS "model_card_arn",
       json_extract(r.provider_properties, '$.ModelCardName') AS "model_card_name",
       json_extract(r.provider_properties, '$.ModelCardProcessingStatus') AS "model_card_processing_status",
       json_extract(r.provider_properties, '$.ModelCardStatus') AS "model_card_status",
       json_extract(r.provider_properties, '$.ModelCardVersion') AS "model_card_version",
       json_extract(r.provider_properties, '$.SecurityConfig') AS "security_config",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::ModelCard';

CREATE VIEW aws_cc_sage_maker_model_package AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdditionalInferenceSpecifications') AS "additional_inference_specifications",
       json_extract(r.provider_properties, '$.ApprovalDescription') AS "approval_description",
       json_extract(r.provider_properties, '$.CertifyForMarketplace') AS "certify_for_marketplace",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.CustomerMetadataProperties') AS "customer_metadata_properties",
       json_extract(r.provider_properties, '$.Domain') AS "domain",
       json_extract(r.provider_properties, '$.DriftCheckBaselines') AS "drift_check_baselines",
       json_extract(r.provider_properties, '$.InferenceSpecification') AS "inference_specification",
       json_extract(r.provider_properties, '$.LastModifiedTime') AS "last_modified_time",
       json_extract(r.provider_properties, '$.MetadataProperties') AS "metadata_properties",
       json_extract(r.provider_properties, '$.ModelApprovalStatus') AS "model_approval_status",
       json_extract(r.provider_properties, '$.ModelCard') AS "model_card",
       json_extract(r.provider_properties, '$.ModelMetrics') AS "model_metrics",
       json_extract(r.provider_properties, '$.ModelPackageArn') AS "model_package_arn",
       json_extract(r.provider_properties, '$.ModelPackageDescription') AS "model_package_description",
       json_extract(r.provider_properties, '$.ModelPackageGroupName') AS "model_package_group_name",
       json_extract(r.provider_properties, '$.ModelPackageName') AS "model_package_name",
       json_extract(r.provider_properties, '$.ModelPackageStatus') AS "model_package_status",
       json_extract(r.provider_properties, '$.ModelPackageStatusDetails') AS "model_package_status_details",
       json_extract(r.provider_properties, '$.ModelPackageVersion') AS "model_package_version",
       json_extract(r.provider_properties, '$.SamplePayloadUrl') AS "sample_payload_url",
       json_extract(r.provider_properties, '$.SecurityConfig') AS "security_config",
       json_extract(r.provider_properties, '$.SkipModelValidation') AS "skip_model_validation",
       json_extract(r.provider_properties, '$.SourceAlgorithmSpecification') AS "source_algorithm_specification",
       json_extract(r.provider_properties, '$.SourceUri') AS "source_uri",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Task') AS "task",
       json_extract(r.provider_properties, '$.ValidationSpecification') AS "validation_specification"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SageMaker::ModelPackage';

CREATE VIEW aws_cc_bedrock_agent AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ActionGroups') AS "action_groups",
       json_extract(r.provider_properties, '$.AgentArn') AS "agent_arn",
       json_extract(r.provider_properties, '$.AgentCollaboration') AS "agent_collaboration",
       json_extract(r.provider_properties, '$.AgentCollaborators') AS "agent_collaborators",
       json_extract(r.provider_properties, '$.AgentId') AS "agent_id",
       json_extract(r.provider_properties, '$.AgentName') AS "agent_name",
       json_extract(r.provider_properties, '$.AgentResourceRoleArn') AS "agent_resource_role_arn",
       json_extract(r.provider_properties, '$.AgentStatus') AS "agent_status",
       json_extract(r.provider_properties, '$.AgentVersion') AS "agent_version",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.CustomOrchestration') AS "custom_orchestration",
       json_extract(r.provider_properties, '$.CustomerEncryptionKeyArn') AS "customer_encryption_key_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FailureReasons') AS "failure_reasons",
       json_extract(r.provider_properties, '$.FoundationModel') AS "foundation_model",
       json_extract(r.provider_properties, '$.GuardrailConfiguration') AS "guardrail_configuration",
       json_extract(r.provider_properties, '$.IdleSessionTTLInSeconds') AS "idle_session_ttl_in_seconds",
       json_extract(r.provider_properties, '$.KnowledgeBases') AS "knowledge_bases",
       json_extract(r.provider_properties, '$.MemoryConfiguration') AS "memory_configuration",
       json_extract(r.provider_properties, '$.OrchestrationType') AS "orchestration_type",
       json_extract(r.provider_properties, '$.PreparedAt') AS "prepared_at",
       json_extract(r.provider_properties, '$.RecommendedActions') AS "recommended_actions",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TestAliasTags') AS "test_alias_tags",
       json_extract(r.provider_properties, '$.UpdatedAt') AS "updated_at"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Bedrock::Agent';

CREATE VIEW aws_cc_bedrock_knowledge_base AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FailureReasons') AS "failure_reasons",
       json_extract(r.provider_properties, '$.KnowledgeBaseArn') AS "knowledge_base_arn",
       json_extract(r.provider_properties, '$.KnowledgeBaseConfiguration') AS "knowledge_base_configuration",
       json_extract(r.provider_properties, '$.KnowledgeBaseId') AS "knowledge_base_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StorageConfiguration') AS "storage_configuration",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UpdatedAt') AS "updated_at"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Bedrock::KnowledgeBase';

CREATE VIEW aws_cc_bedrock_flow AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.CustomerEncryptionKeyArn') AS "customer_encryption_key_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.ExecutionRoleArn') AS "execution_role_arn",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TestAliasTags') AS "test_alias_tags",
       json_extract(r.provider_properties, '$.UpdatedAt') AS "updated_at",
       json_extract(r.provider_properties, '$.Validations') AS "validations",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Bedrock::Flow';

CREATE VIEW aws_cc_bedrock_guardrail AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutomatedReasoningPolicyConfig') AS "automated_reasoning_policy_config",
       json_extract(r.provider_properties, '$.BlockedInputMessaging') AS "blocked_input_messaging",
       json_extract(r.provider_properties, '$.BlockedOutputsMessaging') AS "blocked_outputs_messaging",
       json_extract(r.provider_properties, '$.ContentPolicyConfig') AS "content_policy_config",
       json_extract(r.provider_properties, '$.ContextualGroundingPolicyConfig') AS "contextual_grounding_policy_config",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.CrossRegionConfig') AS "cross_region_config",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FailureRecommendations') AS "failure_recommendations",
       json_extract(r.provider_properties, '$.GuardrailArn') AS "guardrail_arn",
       json_extract(r.provider_properties, '$.GuardrailId') AS "guardrail_id",
       json_extract(r.provider_properties, '$.KmsKeyArn') AS "kms_key_arn",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.SensitiveInformationPolicyConfig') AS "sensitive_information_policy_config",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusReasons') AS "status_reasons",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TopicPolicyConfig') AS "topic_policy_config",
       json_extract(r.provider_properties, '$.UpdatedAt') AS "updated_at",
       json_extract(r.provider_properties, '$.Version') AS "version",
       json_extract(r.provider_properties, '$.WordPolicyConfig') AS "word_policy_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Bedrock::Guardrail';

CREATE VIEW aws_cc_bedrock_prompt AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.CustomerEncryptionKeyArn') AS "customer_encryption_key_arn",
       json_extract(r.provider_properties, '$.DefaultVariant') AS "default_variant",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UpdatedAt') AS "updated_at",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Bedrock::Prompt';

CREATE VIEW aws_cc_bedrock_application_inference_profile AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.InferenceProfileArn') AS "inference_profile_arn",
       json_extract(r.provider_properties, '$.InferenceProfileId') AS "inference_profile_id",
       json_extract(r.provider_properties, '$.InferenceProfileIdentifier') AS "inference_profile_identifier",
       json_extract(r.provider_properties, '$.InferenceProfileName') AS "inference_profile_name",
       json_extract(r.provider_properties, '$.Models') AS "models",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.UpdatedAt') AS "updated_at"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Bedrock::ApplicationInferenceProfile';

CREATE VIEW aws_cc_bedrock_data_automation_project AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.CustomOutputConfiguration') AS "custom_output_configuration",
       json_extract(r.provider_properties, '$.KmsEncryptionContext') AS "kms_encryption_context",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LastModifiedTime') AS "last_modified_time",
       json_extract(r.provider_properties, '$.OverrideConfiguration') AS "override_configuration",
       json_extract(r.provider_properties, '$.ProjectArn') AS "project_arn",
       json_extract(r.provider_properties, '$.ProjectDescription') AS "project_description",
       json_extract(r.provider_properties, '$.ProjectName') AS "project_name",
       json_extract(r.provider_properties, '$.ProjectStage') AS "project_stage",
       json_extract(r.provider_properties, '$.ProjectType') AS "project_type",
       json_extract(r.provider_properties, '$.StandardOutputConfiguration') AS "standard_output_configuration",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Bedrock::DataAutomationProject';

CREATE VIEW aws_cc_bedrock_automated_reasoning_policy AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.DefinitionHash') AS "definition_hash",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.KmsKeyArn') AS "kms_key_arn",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.PolicyArn') AS "policy_arn",
       json_extract(r.provider_properties, '$.PolicyId') AS "policy_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UpdatedAt') AS "updated_at",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Bedrock::AutomatedReasoningPolicy';

CREATE VIEW aws_cc_msk_replicator AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CurrentVersion') AS "current_version",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.KafkaClusters') AS "kafka_clusters",
       json_extract(r.provider_properties, '$.LogDelivery') AS "log_delivery",
       json_extract(r.provider_properties, '$.ReplicationInfoList') AS "replication_info_list",
       json_extract(r.provider_properties, '$.ReplicatorArn') AS "replicator_arn",
       json_extract(r.provider_properties, '$.ReplicatorName') AS "replicator_name",
       json_extract(r.provider_properties, '$.ServiceExecutionRoleArn') AS "service_execution_role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MSK::Replicator';

CREATE VIEW aws_cc_msk_vpc_connection AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Authentication') AS "authentication",
       json_extract(r.provider_properties, '$.ClientSubnets') AS "client_subnets",
       json_extract(r.provider_properties, '$.SecurityGroups') AS "security_groups",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetClusterArn') AS "target_cluster_arn",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MSK::VpcConnection';

CREATE VIEW aws_cc_redshift_endpoint_authorization AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Account') AS "account",
       json_extract(r.provider_properties, '$.AllowedAllVPCs') AS "allowed_all_vp_cs",
       json_extract(r.provider_properties, '$.AllowedVPCs') AS "allowed_vp_cs",
       json_extract(r.provider_properties, '$.AuthorizeTime') AS "authorize_time",
       json_extract(r.provider_properties, '$.ClusterIdentifier') AS "cluster_identifier",
       json_extract(r.provider_properties, '$.ClusterStatus') AS "cluster_status",
       json_extract(r.provider_properties, '$.EndpointCount') AS "endpoint_count",
       json_extract(r.provider_properties, '$.Grantee') AS "grantee",
       json_extract(r.provider_properties, '$.Grantor') AS "grantor",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.VpcIds') AS "vpc_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::EndpointAuthorization';

CREATE VIEW aws_cc_redshift_event_subscription AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CustSubscriptionId') AS "cust_subscription_id",
       json_extract(r.provider_properties, '$.CustomerAwsId') AS "customer_aws_id",
       json_extract(r.provider_properties, '$.Enabled') AS "enabled",
       json_extract(r.provider_properties, '$.EventCategories') AS "event_categories",
       json_extract(r.provider_properties, '$.EventCategoriesList') AS "event_categories_list",
       json_extract(r.provider_properties, '$.Severity') AS "severity",
       json_extract(r.provider_properties, '$.SnsTopicArn') AS "sns_topic_arn",
       json_extract(r.provider_properties, '$.SourceIds') AS "source_ids",
       json_extract(r.provider_properties, '$.SourceIdsList') AS "source_ids_list",
       json_extract(r.provider_properties, '$.SourceType') AS "source_type",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.SubscriptionCreationTime') AS "subscription_creation_time",
       json_extract(r.provider_properties, '$.SubscriptionName') AS "subscription_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::EventSubscription';

CREATE VIEW aws_cc_redshift_integration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdditionalEncryptionContext') AS "additional_encryption_context",
       json_extract(r.provider_properties, '$.CreateTime') AS "create_time",
       json_extract(r.provider_properties, '$.IntegrationArn') AS "integration_arn",
       json_extract(r.provider_properties, '$.IntegrationName') AS "integration_name",
       json_extract(r.provider_properties, '$.KMSKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.SourceArn') AS "source_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetArn') AS "target_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::Integration';

CREATE VIEW aws_cc_redshift_redshift_idc_application AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationType') AS "application_type",
       json_extract(r.provider_properties, '$.AuthorizedTokenIssuerList') AS "authorized_token_issuer_list",
       json_extract(r.provider_properties, '$.IamRoleArn') AS "iam_role_arn",
       json_extract(r.provider_properties, '$.IdcDisplayName') AS "idc_display_name",
       json_extract(r.provider_properties, '$.IdcInstanceArn') AS "idc_instance_arn",
       json_extract(r.provider_properties, '$.IdcManagedApplicationArn') AS "idc_managed_application_arn",
       json_extract(r.provider_properties, '$.IdcOnboardStatus') AS "idc_onboard_status",
       json_extract(r.provider_properties, '$.IdentityNamespace') AS "identity_namespace",
       json_extract(r.provider_properties, '$.RedshiftIdcApplicationArn') AS "redshift_idc_application_arn",
       json_extract(r.provider_properties, '$.RedshiftIdcApplicationName') AS "redshift_idc_application_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::RedshiftIdcApplication';

CREATE VIEW aws_cc_redshift_scheduled_action AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Enable') AS "enable",
       json_extract(r.provider_properties, '$.EndTime') AS "end_time",
       json_extract(r.provider_properties, '$.IamRole') AS "iam_role",
       json_extract(r.provider_properties, '$.NextInvocations') AS "next_invocations",
       json_extract(r.provider_properties, '$.Schedule') AS "schedule",
       json_extract(r.provider_properties, '$.ScheduledActionDescription') AS "scheduled_action_description",
       json_extract(r.provider_properties, '$.ScheduledActionName') AS "scheduled_action_name",
       json_extract(r.provider_properties, '$.StartTime') AS "start_time",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.TargetAction') AS "target_action"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::ScheduledAction';

CREATE VIEW aws_cc_redshift_snapshot_copy_grant AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.SnapshotCopyGrantName') AS "snapshot_copy_grant_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::SnapshotCopyGrant';

CREATE VIEW aws_cc_redshift_snapshot_schedule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AssociatedClusterCount') AS "associated_cluster_count",
       json_extract(r.provider_properties, '$.ScheduleDefinitions') AS "schedule_definitions",
       json_extract(r.provider_properties, '$.ScheduleDescription') AS "schedule_description",
       json_extract(r.provider_properties, '$.ScheduleIdentifier') AS "schedule_identifier",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::SnapshotSchedule';

CREATE VIEW aws_cc_redshift_usage_limit AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Amount') AS "amount",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.BreachAction') AS "breach_action",
       json_extract(r.provider_properties, '$.ClusterIdentifier') AS "cluster_identifier",
       json_extract(r.provider_properties, '$.FeatureType') AS "feature_type",
       json_extract(r.provider_properties, '$.LimitType') AS "limit_type",
       json_extract(r.provider_properties, '$.Period') AS "period",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UsageLimitId') AS "usage_limit_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::UsageLimit';

CREATE VIEW aws_cc_redshift_qev2_idc_application AS
SELECT r.*,
       json_extract(r.provider_properties, '$.IdcDisplayName') AS "idc_display_name",
       json_extract(r.provider_properties, '$.IdcInstanceArn') AS "idc_instance_arn",
       json_extract(r.provider_properties, '$.IdcManagedApplicationArn') AS "idc_managed_application_arn",
       json_extract(r.provider_properties, '$.IdcOnboardStatus') AS "idc_onboard_status",
       json_extract(r.provider_properties, '$.Qev2IdcApplicationArn') AS "qev2_idc_application_arn",
       json_extract(r.provider_properties, '$.Qev2IdcApplicationName') AS "qev2_idc_application_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::QEV2IdcApplication';

CREATE VIEW aws_cc_glue_blueprint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreatedOn') AS "created_on",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.LastModifiedOn') AS "last_modified_on",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ParameterSpec') AS "parameter_spec",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Blueprint';

CREATE VIEW aws_cc_glue_catalog AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllowFullTableExternalDataAccess') AS "allow_full_table_external_data_access",
       json_extract(r.provider_properties, '$.CatalogId') AS "catalog_id",
       json_extract(r.provider_properties, '$.CatalogProperties') AS "catalog_properties",
       json_extract(r.provider_properties, '$.CreateDatabaseDefaultPermissions') AS "create_database_default_permissions",
       json_extract(r.provider_properties, '$.CreateTableDefaultPermissions') AS "create_table_default_permissions",
       json_extract(r.provider_properties, '$.CreateTime') AS "create_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FederatedCatalog') AS "federated_catalog",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Parameters') AS "parameters",
       json_extract(r.provider_properties, '$.ResourceArn') AS "resource_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetRedshiftCatalog') AS "target_redshift_catalog",
       json_extract(r.provider_properties, '$.UpdateTime') AS "update_time"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Catalog';

CREATE VIEW aws_cc_glue_classifier AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CsvClassifier') AS "csv_classifier",
       json_extract(r.provider_properties, '$.GrokClassifier') AS "grok_classifier",
       json_extract(r.provider_properties, '$.JsonClassifier') AS "json_classifier",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.XMLClassifier') AS "xml_classifier"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Classifier';

CREATE VIEW aws_cc_glue_custom_entity_type AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Name') AS "provider_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::CustomEntityType';

CREATE VIEW aws_cc_glue_data_quality_ruleset AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetTable') AS "target_table"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::DataQualityRuleset';

CREATE VIEW aws_cc_glue_integration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdditionalEncryptionContext') AS "additional_encryption_context",
       json_extract(r.provider_properties, '$.CreateTime') AS "create_time",
       json_extract(r.provider_properties, '$.DataFilter') AS "data_filter",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.IntegrationArn') AS "integration_arn",
       json_extract(r.provider_properties, '$.IntegrationConfig') AS "integration_config",
       json_extract(r.provider_properties, '$.IntegrationName') AS "integration_name",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.SourceArn') AS "source_arn",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetArn') AS "target_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Integration';

CREATE VIEW aws_cc_glue_ml_transform AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.GlueVersion') AS "glue_version",
       json_extract(r.provider_properties, '$.InputRecordTables') AS "input_record_tables",
       json_extract(r.provider_properties, '$.MaxCapacity') AS "max_capacity",
       json_extract(r.provider_properties, '$.MaxRetries') AS "max_retries",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.NumberOfWorkers') AS "number_of_workers",
       json_extract(r.provider_properties, '$.Role') AS "role",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Timeout') AS "timeout",
       json_extract(r.provider_properties, '$.TransformEncryption') AS "transform_encryption",
       json_extract(r.provider_properties, '$.TransformId') AS "transform_id",
       json_extract(r.provider_properties, '$.TransformParameters') AS "transform_parameters",
       json_extract(r.provider_properties, '$.WorkerType') AS "worker_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::MLTransform';

CREATE VIEW aws_cc_glue_registry AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Registry';

CREATE VIEW aws_cc_glue_schema AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CheckpointVersion') AS "checkpoint_version",
       json_extract(r.provider_properties, '$.Compatibility') AS "compatibility",
       json_extract(r.provider_properties, '$.DataFormat') AS "data_format",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.InitialSchemaVersionId') AS "initial_schema_version_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Registry') AS "registry",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Schema';

CREATE VIEW aws_cc_glue_security_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.EncryptionConfiguration') AS "encryption_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::SecurityConfiguration';

CREATE VIEW aws_cc_glue_trigger AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Actions') AS "actions",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EventBatchingCondition') AS "event_batching_condition",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Predicate') AS "predicate",
       json_extract(r.provider_properties, '$.Schedule') AS "schedule",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.WorkflowName') AS "workflow_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Trigger';

CREATE VIEW aws_cc_logs_delivery AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DeliveryDestinationArn') AS "delivery_destination_arn",
       json_extract(r.provider_properties, '$.DeliveryDestinationType') AS "delivery_destination_type",
       json_extract(r.provider_properties, '$.DeliveryId') AS "delivery_id",
       json_extract(r.provider_properties, '$.DeliverySourceName') AS "delivery_source_name",
       json_extract(r.provider_properties, '$.FieldDelimiter') AS "field_delimiter",
       json_extract(r.provider_properties, '$.RecordFields') AS "record_fields",
       json_extract(r.provider_properties, '$.S3EnableHiveCompatiblePath') AS "s3_enable_hive_compatible_path",
       json_extract(r.provider_properties, '$.S3SuffixPath') AS "s3_suffix_path",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::Delivery';

CREATE VIEW aws_cc_logs_delivery_destination AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DeliveryDestinationType') AS "delivery_destination_type",
       json_extract(r.provider_properties, '$.DestinationResourceArn') AS "destination_resource_arn",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OutputFormat') AS "output_format",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::DeliveryDestination';

CREATE VIEW aws_cc_logs_delivery_source AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DeliverySourceConfiguration') AS "delivery_source_configuration",
       json_extract(r.provider_properties, '$.LogType') AS "log_type",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ResourceArns') AS "resource_arns",
       json_extract(r.provider_properties, '$.Service') AS "service",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusReason') AS "status_reason",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::DeliverySource';

CREATE VIEW aws_cc_logs_integration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.IntegrationName') AS "integration_name",
       json_extract(r.provider_properties, '$.IntegrationStatus') AS "integration_status",
       json_extract(r.provider_properties, '$.IntegrationType') AS "integration_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::Integration';

CREATE VIEW aws_cc_logs_log_anomaly_detector AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AnomalyDetectorArn') AS "anomaly_detector_arn",
       json_extract(r.provider_properties, '$.AnomalyDetectorStatus') AS "anomaly_detector_status",
       json_extract(r.provider_properties, '$.AnomalyVisibilityTime') AS "anomaly_visibility_time",
       json_extract(r.provider_properties, '$.CreationTimeStamp') AS "creation_time_stamp",
       json_extract(r.provider_properties, '$.DetectorName') AS "detector_name",
       json_extract(r.provider_properties, '$.EvaluationFrequency') AS "evaluation_frequency",
       json_extract(r.provider_properties, '$.FilterPattern') AS "filter_pattern",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LastModifiedTimeStamp') AS "last_modified_time_stamp",
       json_extract(r.provider_properties, '$.LogGroupArnList') AS "log_group_arn_list"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::LogAnomalyDetector';

CREATE VIEW aws_cc_logs_scheduled_query AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DestinationConfiguration') AS "destination_configuration",
       json_extract(r.provider_properties, '$.ExecutionRoleArn') AS "execution_role_arn",
       json_extract(r.provider_properties, '$.LastExecutionStatus') AS "last_execution_status",
       json_extract(r.provider_properties, '$.LastTriggeredTime') AS "last_triggered_time",
       json_extract(r.provider_properties, '$.LastUpdatedTime') AS "last_updated_time",
       json_extract(r.provider_properties, '$.LogGroupIdentifiers') AS "log_group_identifiers",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ScheduleEndTime') AS "schedule_end_time",
       json_extract(r.provider_properties, '$.ScheduleExpression') AS "schedule_expression",
       json_extract(r.provider_properties, '$.ScheduleStartTime') AS "schedule_start_time",
       json_extract(r.provider_properties, '$.ScheduledQueryArn') AS "scheduled_query_arn",
       json_extract(r.provider_properties, '$.StartTimeOffset') AS "start_time_offset",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Timezone') AS "timezone"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::ScheduledQuery';

CREATE VIEW aws_cc_cloud_trail_trail AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdvancedEventSelectors') AS "advanced_event_selectors",
       json_extract(r.provider_properties, '$.AggregationConfigurations') AS "aggregation_configurations",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CloudWatchLogsLogGroupArn') AS "cloud_watch_logs_log_group_arn",
       json_extract(r.provider_properties, '$.CloudWatchLogsRoleArn') AS "cloud_watch_logs_role_arn",
       json_extract(r.provider_properties, '$.EnableLogFileValidation') AS "enable_log_file_validation",
       json_extract(r.provider_properties, '$.EventSelectors') AS "event_selectors",
       json_extract(r.provider_properties, '$.IncludeGlobalServiceEvents') AS "include_global_service_events",
       json_extract(r.provider_properties, '$.InsightSelectors') AS "insight_selectors",
       json_extract(r.provider_properties, '$.IsLogging') AS "is_logging",
       json_extract(r.provider_properties, '$.IsMultiRegionTrail') AS "is_multi_region_trail",
       json_extract(r.provider_properties, '$.IsOrganizationTrail') AS "is_organization_trail",
       json_extract(r.provider_properties, '$.KMSKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.RecursiveLogging') AS "recursive_logging",
       json_extract(r.provider_properties, '$.S3BucketName') AS "s3_bucket_name",
       json_extract(r.provider_properties, '$.S3KeyPrefix') AS "s3_key_prefix",
       json_extract(r.provider_properties, '$.SnsTopicArn') AS "sns_topic_arn",
       json_extract(r.provider_properties, '$.SnsTopicName') AS "sns_topic_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrailName') AS "trail_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CloudTrail::Trail';

CREATE VIEW aws_cc_dms_certificate AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CertificateArn') AS "certificate_arn",
       json_extract(r.provider_properties, '$.CertificateIdentifier') AS "certificate_identifier"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::Certificate';

CREATE VIEW aws_cc_dms_data_migration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DataMigrationArn') AS "data_migration_arn",
       json_extract(r.provider_properties, '$.DataMigrationCreateTime') AS "data_migration_create_time",
       json_extract(r.provider_properties, '$.DataMigrationName') AS "data_migration_name",
       json_extract(r.provider_properties, '$.DataMigrationSettings') AS "data_migration_settings",
       json_extract(r.provider_properties, '$.DataMigrationType') AS "data_migration_type",
       json_extract(r.provider_properties, '$.MigrationProjectIdentifier') AS "migration_project_identifier",
       json_extract(r.provider_properties, '$.ServiceAccessRoleArn') AS "service_access_role_arn",
       json_extract(r.provider_properties, '$.SourceDataSettings') AS "source_data_settings",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::DataMigration';

CREATE VIEW aws_cc_dms_data_provider AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DataProviderArn') AS "data_provider_arn",
       json_extract(r.provider_properties, '$.DataProviderCreationTime') AS "data_provider_creation_time",
       json_extract(r.provider_properties, '$.DataProviderName') AS "data_provider_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.Settings') AS "settings",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Virtual') AS "virtual"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::DataProvider';

CREATE VIEW aws_cc_dms_event_subscription AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Enabled') AS "enabled",
       json_extract(r.provider_properties, '$.EventCategories') AS "event_categories",
       json_extract(r.provider_properties, '$.SnsTopicArn') AS "sns_topic_arn",
       json_extract(r.provider_properties, '$.SourceIds') AS "source_ids",
       json_extract(r.provider_properties, '$.SourceType') AS "source_type",
       json_extract(r.provider_properties, '$.SubscriptionName') AS "subscription_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::EventSubscription';

CREATE VIEW aws_cc_dms_instance_profile AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.InstanceProfileArn') AS "instance_profile_arn",
       json_extract(r.provider_properties, '$.InstanceProfileCreationTime') AS "instance_profile_creation_time",
       json_extract(r.provider_properties, '$.InstanceProfileName') AS "instance_profile_name",
       json_extract(r.provider_properties, '$.KmsKeyArn') AS "kms_key_arn",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.PubliclyAccessible') AS "publicly_accessible",
       json_extract(r.provider_properties, '$.SubnetGroupIdentifier') AS "subnet_group_identifier",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcSecurityGroups') AS "vpc_security_groups"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::InstanceProfile';

CREATE VIEW aws_cc_dms_migration_project AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.InstanceProfileArn') AS "instance_profile_arn",
       json_extract(r.provider_properties, '$.InstanceProfileName') AS "instance_profile_name",
       json_extract(r.provider_properties, '$.MigrationProjectArn') AS "migration_project_arn",
       json_extract(r.provider_properties, '$.MigrationProjectCreationTime') AS "migration_project_creation_time",
       json_extract(r.provider_properties, '$.MigrationProjectName') AS "migration_project_name",
       json_extract(r.provider_properties, '$.SchemaConversionApplicationAttributes') AS "schema_conversion_application_attributes",
       json_extract(r.provider_properties, '$.SourceDataProviderDescriptors') AS "source_data_provider_descriptors",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetDataProviderDescriptors') AS "target_data_provider_descriptors",
       json_extract(r.provider_properties, '$.TransformationRules') AS "transformation_rules"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::MigrationProject';

CREATE VIEW aws_cc_cloud_formation_stack AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Capabilities') AS "capabilities",
       json_extract(r.provider_properties, '$.ChangeSetId') AS "change_set_id",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DisableRollback') AS "disable_rollback",
       json_extract(r.provider_properties, '$.EnableTerminationProtection') AS "enable_termination_protection",
       json_extract(r.provider_properties, '$.LastUpdateTime') AS "last_update_time",
       json_extract(r.provider_properties, '$.NotificationARNs') AS "notification_ar_ns",
       json_extract(r.provider_properties, '$.ParentId') AS "parent_id",
       json_extract(r.provider_properties, '$.RoleARN') AS "role_arn",
       json_extract(r.provider_properties, '$.RootId') AS "root_id",
       json_extract(r.provider_properties, '$.StackId') AS "stack_id",
       json_extract(r.provider_properties, '$.StackName') AS "stack_name",
       json_extract(r.provider_properties, '$.StackPolicyBody') AS "stack_policy_body",
       json_extract(r.provider_properties, '$.StackStatus') AS "stack_status",
       json_extract(r.provider_properties, '$.StackStatusReason') AS "stack_status_reason",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TimeoutInMinutes') AS "timeout_in_minutes"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CloudFormation::Stack';

CREATE VIEW aws_cc_emr_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Applications') AS "applications",
       json_extract(r.provider_properties, '$.AutoScalingRole') AS "auto_scaling_role",
       json_extract(r.provider_properties, '$.AutoTerminationPolicy') AS "auto_termination_policy",
       json_extract(r.provider_properties, '$.Configurations') AS "configurations",
       json_extract(r.provider_properties, '$.CustomAmiId') AS "custom_ami_id",
       json_extract(r.provider_properties, '$.EbsRootVolumeIops') AS "ebs_root_volume_iops",
       json_extract(r.provider_properties, '$.EbsRootVolumeSize') AS "ebs_root_volume_size",
       json_extract(r.provider_properties, '$.EbsRootVolumeThroughput') AS "ebs_root_volume_throughput",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Instances') AS "instances",
       json_extract(r.provider_properties, '$.JobFlowRole') AS "job_flow_role",
       json_extract(r.provider_properties, '$.KerberosAttributes') AS "kerberos_attributes",
       json_extract(r.provider_properties, '$.LogUri') AS "log_uri",
       json_extract(r.provider_properties, '$.ManagedScalingPolicy') AS "managed_scaling_policy",
       json_extract(r.provider_properties, '$.MasterPublicDNS') AS "master_public_dns",
       json_extract(r.provider_properties, '$.MonitoringConfiguration') AS "monitoring_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OSReleaseLabel') AS "os_release_label",
       json_extract(r.provider_properties, '$.PlacementGroupConfigs') AS "placement_group_configs",
       json_extract(r.provider_properties, '$.ReleaseLabel') AS "release_label",
       json_extract(r.provider_properties, '$.ScaleDownBehavior') AS "scale_down_behavior",
       json_extract(r.provider_properties, '$.SecurityConfiguration') AS "security_configuration",
       json_extract(r.provider_properties, '$.ServiceRole') AS "service_role",
       json_extract(r.provider_properties, '$.SessionEnabled') AS "session_enabled",
       json_extract(r.provider_properties, '$.StepConcurrencyLevel') AS "step_concurrency_level",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VisibleToAllUsers') AS "visible_to_all_users"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EMR::Cluster';

CREATE VIEW aws_cc_emr_security_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.SecurityConfiguration') AS "security_configuration"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EMR::SecurityConfiguration';

CREATE VIEW aws_cc_emr_studio AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AuthMode') AS "auth_mode",
       json_extract(r.provider_properties, '$.DefaultS3Location') AS "default_s3_location",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EncryptionKeyArn') AS "encryption_key_arn",
       json_extract(r.provider_properties, '$.EngineSecurityGroupId') AS "engine_security_group_id",
       json_extract(r.provider_properties, '$.IdcInstanceArn') AS "idc_instance_arn",
       json_extract(r.provider_properties, '$.IdcUserAssignment') AS "idc_user_assignment",
       json_extract(r.provider_properties, '$.IdpAuthUrl') AS "idp_auth_url",
       json_extract(r.provider_properties, '$.IdpRelayStateParameterName') AS "idp_relay_state_parameter_name",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ServiceRole') AS "service_role",
       json_extract(r.provider_properties, '$.StudioId') AS "studio_id",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrustedIdentityPropagationEnabled') AS "trusted_identity_propagation_enabled",
       json_extract(r.provider_properties, '$.Url') AS "url",
       json_extract(r.provider_properties, '$.UserRole') AS "user_role",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id",
       json_extract(r.provider_properties, '$.WorkspaceSecurityGroupId') AS "workspace_security_group_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EMR::Studio';

CREATE VIEW aws_cc_emr_serverless_application AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationId') AS "application_id",
       json_extract(r.provider_properties, '$.Architecture') AS "architecture",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AutoStartConfiguration') AS "auto_start_configuration",
       json_extract(r.provider_properties, '$.AutoStopConfiguration') AS "auto_stop_configuration",
       json_extract(r.provider_properties, '$.IdentityCenterConfiguration') AS "identity_center_configuration",
       json_extract(r.provider_properties, '$.ImageConfiguration') AS "image_configuration",
       json_extract(r.provider_properties, '$.InitialCapacity') AS "initial_capacity",
       json_extract(r.provider_properties, '$.InteractiveConfiguration') AS "interactive_configuration",
       json_extract(r.provider_properties, '$.MaximumCapacity') AS "maximum_capacity",
       json_extract(r.provider_properties, '$.MonitoringConfiguration') AS "monitoring_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.NetworkConfiguration') AS "network_configuration",
       json_extract(r.provider_properties, '$.ReleaseLabel') AS "release_label",
       json_extract(r.provider_properties, '$.RuntimeConfiguration') AS "runtime_configuration",
       json_extract(r.provider_properties, '$.SchedulerConfiguration') AS "scheduler_configuration",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.WorkerTypeSpecifications') AS "worker_type_specifications"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EMRServerless::Application';

CREATE VIEW aws_cc_emr_containers_virtual_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ContainerProvider') AS "container_provider",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.SecurityConfigurationId') AS "security_configuration_id",
       json_extract(r.provider_properties, '$.SessionEnabled') AS "session_enabled",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EMRContainers::VirtualCluster';

CREATE VIEW aws_cc_app_runner_auto_scaling_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutoScalingConfigurationArn') AS "auto_scaling_configuration_arn",
       json_extract(r.provider_properties, '$.AutoScalingConfigurationName') AS "auto_scaling_configuration_name",
       json_extract(r.provider_properties, '$.AutoScalingConfigurationRevision') AS "auto_scaling_configuration_revision",
       json_extract(r.provider_properties, '$.Latest') AS "latest",
       json_extract(r.provider_properties, '$.MaxConcurrency') AS "max_concurrency",
       json_extract(r.provider_properties, '$.MaxSize') AS "max_size",
       json_extract(r.provider_properties, '$.MinSize') AS "min_size",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppRunner::AutoScalingConfiguration';

CREATE VIEW aws_cc_app_runner_observability_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Latest') AS "latest",
       json_extract(r.provider_properties, '$.ObservabilityConfigurationArn') AS "observability_configuration_arn",
       json_extract(r.provider_properties, '$.ObservabilityConfigurationName') AS "observability_configuration_name",
       json_extract(r.provider_properties, '$.ObservabilityConfigurationRevision') AS "observability_configuration_revision",
       json_extract(r.provider_properties, '$.TraceConfiguration') AS "trace_configuration"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppRunner::ObservabilityConfiguration';

CREATE VIEW aws_cc_athena_capacity_reservation AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllocatedDpus') AS "allocated_dpus",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CapacityAssignmentConfiguration') AS "capacity_assignment_configuration",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.LastSuccessfulAllocationTime') AS "last_successful_allocation_time",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetDpus') AS "target_dpus"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Athena::CapacityReservation';

CREATE VIEW aws_cc_athena_named_query AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Database') AS "database",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.NamedQueryId') AS "named_query_id",
       json_extract(r.provider_properties, '$.QueryString') AS "query_string",
       json_extract(r.provider_properties, '$.WorkGroup') AS "work_group"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Athena::NamedQuery';

CREATE VIEW aws_cc_config_aggregation_authorization AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AggregationAuthorizationArn') AS "aggregation_authorization_arn",
       json_extract(r.provider_properties, '$.AuthorizedAccountId') AS "authorized_account_id",
       json_extract(r.provider_properties, '$.AuthorizedAwsRegion') AS "authorized_aws_region",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::AggregationAuthorization';

CREATE VIEW aws_cc_config_conformance_pack AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConformancePackArn') AS "conformance_pack_arn",
       json_extract(r.provider_properties, '$.ConformancePackInputParameters') AS "conformance_pack_input_parameters",
       json_extract(r.provider_properties, '$.ConformancePackName') AS "conformance_pack_name",
       json_extract(r.provider_properties, '$.DeliveryS3Bucket') AS "delivery_s3_bucket",
       json_extract(r.provider_properties, '$.DeliveryS3KeyPrefix') AS "delivery_s3_key_prefix",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::ConformancePack';

CREATE VIEW aws_cc_config_organization_config_rule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ExcludedAccounts') AS "excluded_accounts",
       json_extract(r.provider_properties, '$.OrganizationConfigRuleArn') AS "organization_config_rule_arn",
       json_extract(r.provider_properties, '$.OrganizationConfigRuleName') AS "organization_config_rule_name",
       json_extract(r.provider_properties, '$.OrganizationCustomPolicyRuleMetadata') AS "organization_custom_policy_rule_metadata",
       json_extract(r.provider_properties, '$.OrganizationCustomRuleMetadata') AS "organization_custom_rule_metadata",
       json_extract(r.provider_properties, '$.OrganizationManagedRuleMetadata') AS "organization_managed_rule_metadata"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::OrganizationConfigRule';

CREATE VIEW aws_cc_config_organization_conformance_pack AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConformancePackInputParameters') AS "conformance_pack_input_parameters",
       json_extract(r.provider_properties, '$.DeliveryS3Bucket') AS "delivery_s3_bucket",
       json_extract(r.provider_properties, '$.DeliveryS3KeyPrefix') AS "delivery_s3_key_prefix",
       json_extract(r.provider_properties, '$.ExcludedAccounts') AS "excluded_accounts",
       json_extract(r.provider_properties, '$.OrganizationConformancePackArn') AS "organization_conformance_pack_arn",
       json_extract(r.provider_properties, '$.OrganizationConformancePackName') AS "organization_conformance_pack_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::OrganizationConformancePack';

CREATE VIEW aws_cc_config_stored_query AS
SELECT r.*,
       json_extract(r.provider_properties, '$.QueryArn') AS "query_arn",
       json_extract(r.provider_properties, '$.QueryDescription') AS "query_description",
       json_extract(r.provider_properties, '$.QueryExpression') AS "query_expression",
       json_extract(r.provider_properties, '$.QueryId') AS "query_id",
       json_extract(r.provider_properties, '$.QueryName') AS "query_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::StoredQuery';

CREATE VIEW aws_cc_code_build_fleet AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.BaseCapacity') AS "base_capacity",
       json_extract(r.provider_properties, '$.ComputeConfiguration') AS "compute_configuration",
       json_extract(r.provider_properties, '$.ComputeType') AS "compute_type",
       json_extract(r.provider_properties, '$.EnvironmentType') AS "environment_type",
       json_extract(r.provider_properties, '$.FleetProxyConfiguration') AS "fleet_proxy_configuration",
       json_extract(r.provider_properties, '$.FleetServiceRole') AS "fleet_service_role",
       json_extract(r.provider_properties, '$.FleetVpcConfig') AS "fleet_vpc_config",
       json_extract(r.provider_properties, '$.ImageId') AS "image_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OverflowBehavior') AS "overflow_behavior",
       json_extract(r.provider_properties, '$.ScalingConfiguration') AS "scaling_configuration",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodeBuild::Fleet';

CREATE VIEW aws_cc_code_pipeline_custom_action_type AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Category') AS "category",
       json_extract(r.provider_properties, '$.ConfigurationProperties') AS "configuration_properties",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.InputArtifactDetails') AS "input_artifact_details",
       json_extract(r.provider_properties, '$.OutputArtifactDetails') AS "output_artifact_details",
       json_extract(r.provider_properties, '$.Provider') AS "provider",
       json_extract(r.provider_properties, '$.Settings') AS "settings",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodePipeline::CustomActionType';

CREATE VIEW aws_cc_code_pipeline_webhook AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Authentication') AS "authentication",
       json_extract(r.provider_properties, '$.AuthenticationConfiguration') AS "authentication_configuration",
       json_extract(r.provider_properties, '$.Filters') AS "filters",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetAction') AS "target_action",
       json_extract(r.provider_properties, '$.TargetPipeline') AS "target_pipeline",
       json_extract(r.provider_properties, '$.Url') AS "url"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodePipeline::Webhook';

CREATE VIEW aws_cc_route53_resolver_firewall_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.FirewallFailOpen') AS "firewall_fail_open",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.ResourceId') AS "resource_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::FirewallConfig';

CREATE VIEW aws_cc_route53_resolver_firewall_rule_group_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.CreatorRequestId') AS "creator_request_id",
       json_extract(r.provider_properties, '$.FirewallRuleGroupId') AS "firewall_rule_group_id",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.ManagedOwnerName') AS "managed_owner_name",
       json_extract(r.provider_properties, '$.ModificationTime') AS "modification_time",
       json_extract(r.provider_properties, '$.MutationProtection') AS "mutation_protection",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Priority') AS "priority",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusMessage') AS "status_message",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::FirewallRuleGroupAssociation';

CREATE VIEW aws_cc_route53_resolver_outpost_resolver AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.CreatorRequestId') AS "creator_request_id",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.InstanceCount') AS "instance_count",
       json_extract(r.provider_properties, '$.ModificationTime') AS "modification_time",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OutpostArn') AS "outpost_arn",
       json_extract(r.provider_properties, '$.PreferredInstanceType') AS "preferred_instance_type",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusMessage') AS "status_message",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::OutpostResolver';

CREATE VIEW aws_cc_route53_resolver_resolver_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutodefinedReverse') AS "autodefined_reverse",
       json_extract(r.provider_properties, '$.AutodefinedReverseFlag') AS "autodefined_reverse_flag",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.ResourceId') AS "resource_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::ResolverConfig';

CREATE VIEW aws_cc_route53_resolver_resolver_dnssec_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.ResourceId') AS "resource_id",
       json_extract(r.provider_properties, '$.ValidationStatus') AS "validation_status"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::ResolverDNSSECConfig';

CREATE VIEW aws_cc_route53_resolver_resolver_query_logging_config_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Error') AS "error",
       json_extract(r.provider_properties, '$.ErrorMessage') AS "error_message",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.ResolverQueryLogConfigId') AS "resolver_query_log_config_id",
       json_extract(r.provider_properties, '$.ResourceId') AS "resource_id",
       json_extract(r.provider_properties, '$.Status') AS "status"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::ResolverQueryLoggingConfigAssociation';

CREATE VIEW aws_cc_network_firewall_container_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ContainerAssociationArn') AS "container_association_arn",
       json_extract(r.provider_properties, '$.ContainerAssociationName') AS "container_association_name",
       json_extract(r.provider_properties, '$.ContainerMonitoringConfigurations') AS "container_monitoring_configurations",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.ResolvedCidrCount') AS "resolved_cidr_count",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::NetworkFirewall::ContainerAssociation';

CREATE VIEW aws_cc_network_firewall_tls_inspection_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.TLSInspectionConfiguration') AS "tls_inspection_configuration",
       json_extract(r.provider_properties, '$.TLSInspectionConfigurationArn') AS "tls_inspection_configuration_arn",
       json_extract(r.provider_properties, '$.TLSInspectionConfigurationId') AS "tls_inspection_configuration_id",
       json_extract(r.provider_properties, '$.TLSInspectionConfigurationName') AS "tls_inspection_configuration_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::NetworkFirewall::TLSInspectionConfiguration';

CREATE VIEW aws_cc_network_firewall_vpc_endpoint_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EndpointId') AS "endpoint_id",
       json_extract(r.provider_properties, '$.FirewallArn') AS "firewall_arn",
       json_extract(r.provider_properties, '$.SubnetMapping') AS "subnet_mapping",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcEndpointAssociationArn') AS "vpc_endpoint_association_arn",
       json_extract(r.provider_properties, '$.VpcEndpointAssociationId') AS "vpc_endpoint_association_id",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::NetworkFirewall::VpcEndpointAssociation';

CREATE VIEW aws_cc_vpc_lattice_resource_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllowAssociationToSharableServiceNetwork') AS "allow_association_to_sharable_service_network",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CustomDomainName') AS "custom_domain_name",
       json_extract(r.provider_properties, '$.DomainVerificationId') AS "domain_verification_id",
       json_extract(r.provider_properties, '$.GroupDomain') AS "group_domain",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.PortRanges') AS "port_ranges",
       json_extract(r.provider_properties, '$.ProtocolType') AS "protocol_type",
       json_extract(r.provider_properties, '$.ResourceConfigurationDefinition') AS "resource_configuration_definition",
       json_extract(r.provider_properties, '$.ResourceConfigurationType') AS "resource_configuration_type",
       json_extract(r.provider_properties, '$.ResourceGatewayId') AS "resource_gateway_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::VpcLattice::ResourceConfiguration';

CREATE VIEW aws_cc_vpc_lattice_resource_gateway AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.IpAddressType') AS "ip_address_type",
       json_extract(r.provider_properties, '$.Ipv4AddressesPerEni') AS "ipv4_addresses_per_eni",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ResourceConfigDnsResolution') AS "resource_config_dns_resolution",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcIdentifier') AS "vpc_identifier"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::VpcLattice::ResourceGateway';

CREATE VIEW aws_cc_vpc_lattice_service_network_service_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.DnsEntry') AS "dns_entry",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.ServiceArn') AS "service_arn",
       json_extract(r.provider_properties, '$.ServiceId') AS "service_id",
       json_extract(r.provider_properties, '$.ServiceName') AS "service_name",
       json_extract(r.provider_properties, '$.ServiceNetworkArn') AS "service_network_arn",
       json_extract(r.provider_properties, '$.ServiceNetworkId') AS "service_network_id",
       json_extract(r.provider_properties, '$.ServiceNetworkName') AS "service_network_name",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::VpcLattice::ServiceNetworkServiceAssociation';

CREATE VIEW aws_cc_vpc_lattice_service_network_vpc_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.DnsOptions') AS "dns_options",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.PrivateDnsEnabled') AS "private_dns_enabled",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.ServiceNetworkArn') AS "service_network_arn",
       json_extract(r.provider_properties, '$.ServiceNetworkId') AS "service_network_id",
       json_extract(r.provider_properties, '$.ServiceNetworkName') AS "service_network_name",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::VpcLattice::ServiceNetworkVpcAssociation';

CREATE VIEW aws_cc_backup_backup_selection AS
SELECT r.*,
       json_extract(r.provider_properties, '$.BackupPlanId') AS "backup_plan_id",
       json_extract(r.provider_properties, '$.BackupSelection') AS "backup_selection",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.SelectionId') AS "selection_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Backup::BackupSelection';

CREATE VIEW aws_cc_backup_framework AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.DeploymentStatus') AS "deployment_status",
       json_extract(r.provider_properties, '$.FrameworkArn') AS "framework_arn",
       json_extract(r.provider_properties, '$.FrameworkControls') AS "framework_controls",
       json_extract(r.provider_properties, '$.FrameworkDescription') AS "framework_description",
       json_extract(r.provider_properties, '$.FrameworkName') AS "framework_name",
       json_extract(r.provider_properties, '$.FrameworkStatus') AS "framework_status",
       json_extract(r.provider_properties, '$.FrameworkTags') AS "framework_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Backup::Framework';

CREATE VIEW aws_cc_backup_legal_hold AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationDate') AS "creation_date",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.LegalHoldId') AS "legal_hold_id",
       json_extract(r.provider_properties, '$.RecoveryPointSelection') AS "recovery_point_selection",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Title') AS "title"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Backup::LegalHold';

CREATE VIEW aws_cc_backup_logically_air_gapped_backup_vault AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessPolicy') AS "access_policy",
       json_extract(r.provider_properties, '$.BackupVaultArn') AS "backup_vault_arn",
       json_extract(r.provider_properties, '$.BackupVaultName') AS "backup_vault_name",
       json_extract(r.provider_properties, '$.BackupVaultTags') AS "backup_vault_tags",
       json_extract(r.provider_properties, '$.EncryptionKeyArn') AS "encryption_key_arn",
       json_extract(r.provider_properties, '$.MaxRetentionDays') AS "max_retention_days",
       json_extract(r.provider_properties, '$.MinRetentionDays') AS "min_retention_days",
       json_extract(r.provider_properties, '$.MpaApprovalTeamArn') AS "mpa_approval_team_arn",
       json_extract(r.provider_properties, '$.Notifications') AS "notifications",
       json_extract(r.provider_properties, '$.VaultState') AS "vault_state",
       json_extract(r.provider_properties, '$.VaultType') AS "vault_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Backup::LogicallyAirGappedBackupVault';

CREATE VIEW aws_cc_backup_report_plan AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ReportDeliveryChannel') AS "report_delivery_channel",
       json_extract(r.provider_properties, '$.ReportPlanArn') AS "report_plan_arn",
       json_extract(r.provider_properties, '$.ReportPlanDescription') AS "report_plan_description",
       json_extract(r.provider_properties, '$.ReportPlanName') AS "report_plan_name",
       json_extract(r.provider_properties, '$.ReportPlanTags') AS "report_plan_tags",
       json_extract(r.provider_properties, '$.ReportSetting') AS "report_setting"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Backup::ReportPlan';

CREATE VIEW aws_cc_ssm_default_host_management_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CustomerManagedRoleArn') AS "customer_managed_role_arn",
       json_extract(r.provider_properties, '$.LastModifiedDate') AS "last_modified_date",
       json_extract(r.provider_properties, '$.LastModifiedUser') AS "last_modified_user",
       json_extract(r.provider_properties, '$.PermissionModel') AS "permission_model",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.SettingArn') AS "setting_arn",
       json_extract(r.provider_properties, '$.SettingId') AS "setting_id",
       json_extract(r.provider_properties, '$.Status') AS "status"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SSM::DefaultHostManagementConfiguration';

CREATE VIEW aws_cc_ssm_ops_item AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Category') AS "category",
       json_extract(r.provider_properties, '$.CreatedBy') AS "created_by",
       json_extract(r.provider_properties, '$.CreatedTime') AS "created_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.LastModifiedBy') AS "last_modified_by",
       json_extract(r.provider_properties, '$.LastModifiedTime') AS "last_modified_time",
       json_extract(r.provider_properties, '$.OpsItemArn') AS "ops_item_arn",
       json_extract(r.provider_properties, '$.OpsItemId') AS "ops_item_id",
       json_extract(r.provider_properties, '$.OpsItemType') AS "ops_item_type",
       json_extract(r.provider_properties, '$.Priority') AS "priority",
       json_extract(r.provider_properties, '$.Severity') AS "severity",
       json_extract(r.provider_properties, '$.Source') AS "provider_source",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Title') AS "title",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SSM::OpsItem';

CREATE VIEW aws_cc_ssm_patch_baseline AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApprovalRules') AS "approval_rules",
       json_extract(r.provider_properties, '$.ApprovedPatches') AS "approved_patches",
       json_extract(r.provider_properties, '$.ApprovedPatchesComplianceLevel') AS "approved_patches_compliance_level",
       json_extract(r.provider_properties, '$.ApprovedPatchesEnableNonSecurity') AS "approved_patches_enable_non_security",
       json_extract(r.provider_properties, '$.AvailableSecurityUpdatesComplianceStatus') AS "available_security_updates_compliance_status",
       json_extract(r.provider_properties, '$.DefaultBaseline') AS "default_baseline",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.GlobalFilters') AS "global_filters",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OperatingSystem') AS "operating_system",
       json_extract(r.provider_properties, '$.PatchGroups') AS "patch_groups",
       json_extract(r.provider_properties, '$.RejectedPatches') AS "rejected_patches",
       json_extract(r.provider_properties, '$.RejectedPatchesAction') AS "rejected_patches_action",
       json_extract(r.provider_properties, '$.Sources') AS "sources",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SSM::PatchBaseline';

CREATE VIEW aws_cc_ssm_resource_data_sync AS
SELECT r.*,
       json_extract(r.provider_properties, '$.BucketName') AS "bucket_name",
       json_extract(r.provider_properties, '$.BucketPrefix') AS "bucket_prefix",
       json_extract(r.provider_properties, '$.BucketRegion') AS "bucket_region",
       json_extract(r.provider_properties, '$.KMSKeyArn') AS "kms_key_arn",
       json_extract(r.provider_properties, '$.S3Destination') AS "s3_destination",
       json_extract(r.provider_properties, '$.SyncFormat') AS "sync_format",
       json_extract(r.provider_properties, '$.SyncName') AS "sync_name",
       json_extract(r.provider_properties, '$.SyncSource') AS "sync_source",
       json_extract(r.provider_properties, '$.SyncType') AS "sync_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SSM::ResourceDataSync';

CREATE VIEW aws_cc_ses_configuration_set AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ArchivingOptions') AS "archiving_options",
       json_extract(r.provider_properties, '$.DeliveryOptions') AS "delivery_options",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ReputationOptions') AS "reputation_options",
       json_extract(r.provider_properties, '$.SendingOptions') AS "sending_options",
       json_extract(r.provider_properties, '$.SuppressionOptions') AS "suppression_options",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrackingOptions') AS "tracking_options",
       json_extract(r.provider_properties, '$.VdmOptions') AS "vdm_options"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SES::ConfigurationSet';

CREATE VIEW aws_cc_ses_contact_list AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ContactListName') AS "contact_list_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Topics') AS "topics"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SES::ContactList';

CREATE VIEW aws_cc_ses_email_identity AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConfigurationSetAttributes') AS "configuration_set_attributes",
       json_extract(r.provider_properties, '$.DkimAttributes') AS "dkim_attributes",
       json_extract(r.provider_properties, '$.DkimDNSTokenName1') AS "dkim_dns_token_name1",
       json_extract(r.provider_properties, '$.DkimDNSTokenName2') AS "dkim_dns_token_name2",
       json_extract(r.provider_properties, '$.DkimDNSTokenName3') AS "dkim_dns_token_name3",
       json_extract(r.provider_properties, '$.DkimDNSTokenValue1') AS "dkim_dns_token_value1",
       json_extract(r.provider_properties, '$.DkimDNSTokenValue2') AS "dkim_dns_token_value2",
       json_extract(r.provider_properties, '$.DkimDNSTokenValue3') AS "dkim_dns_token_value3",
       json_extract(r.provider_properties, '$.DkimSigningAttributes') AS "dkim_signing_attributes",
       json_extract(r.provider_properties, '$.EmailIdentity') AS "email_identity",
       json_extract(r.provider_properties, '$.FeedbackAttributes') AS "feedback_attributes",
       json_extract(r.provider_properties, '$.MailFromAttributes') AS "mail_from_attributes",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SES::EmailIdentity';

CREATE VIEW aws_cc_ses_receipt_rule_set AS
SELECT r.*,
       json_extract(r.provider_properties, '$.RuleSetName') AS "rule_set_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SES::ReceiptRuleSet';

CREATE VIEW aws_cc_ses_template AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Template') AS "template"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SES::Template';

CREATE VIEW aws_cc_ses_dedicated_ip_pool AS
SELECT r.*,
       json_extract(r.provider_properties, '$.PoolName') AS "pool_name",
       json_extract(r.provider_properties, '$.ScalingMode') AS "scaling_mode",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SES::DedicatedIpPool';

CREATE VIEW aws_cc_ses_mail_manager_ingress_point AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ARecord') AS "a_record",
       json_extract(r.provider_properties, '$.IngressPointArn') AS "ingress_point_arn",
       json_extract(r.provider_properties, '$.IngressPointId') AS "ingress_point_id",
       json_extract(r.provider_properties, '$.IngressPointName') AS "ingress_point_name",
       json_extract(r.provider_properties, '$.NetworkConfiguration') AS "network_configuration",
       json_extract(r.provider_properties, '$.RuleSetId') AS "rule_set_id",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusToUpdate') AS "status_to_update",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TlsPolicy') AS "tls_policy",
       json_extract(r.provider_properties, '$.TrafficPolicyId') AS "traffic_policy_id",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SES::MailManagerIngressPoint';

CREATE VIEW aws_cc_cognito_identity_pool AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllowClassicFlow') AS "allow_classic_flow",
       json_extract(r.provider_properties, '$.AllowUnauthenticatedIdentities') AS "allow_unauthenticated_identities",
       json_extract(r.provider_properties, '$.CognitoIdentityProviders') AS "cognito_identity_providers",
       json_extract(r.provider_properties, '$.DeveloperProviderName') AS "developer_provider_name",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.IdentityPoolName') AS "identity_pool_name",
       json_extract(r.provider_properties, '$.IdentityPoolTags') AS "identity_pool_tags",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OpenIdConnectProviderARNs') AS "open_id_connect_provider_ar_ns",
       json_extract(r.provider_properties, '$.SamlProviderARNs') AS "saml_provider_ar_ns",
       json_extract(r.provider_properties, '$.SupportedLoginProviders') AS "supported_login_providers"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Cognito::IdentityPool';

CREATE VIEW aws_cc_cognito_user_pool AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccountRecoverySetting') AS "account_recovery_setting",
       json_extract(r.provider_properties, '$.AdminCreateUserConfig') AS "admin_create_user_config",
       json_extract(r.provider_properties, '$.AliasAttributes') AS "alias_attributes",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AutoVerifiedAttributes') AS "auto_verified_attributes",
       json_extract(r.provider_properties, '$.DeletionProtection') AS "deletion_protection",
       json_extract(r.provider_properties, '$.DeviceConfiguration') AS "device_configuration",
       json_extract(r.provider_properties, '$.EmailAuthenticationMessage') AS "email_authentication_message",
       json_extract(r.provider_properties, '$.EmailAuthenticationSubject') AS "email_authentication_subject",
       json_extract(r.provider_properties, '$.EmailConfiguration') AS "email_configuration",
       json_extract(r.provider_properties, '$.EmailVerificationMessage') AS "email_verification_message",
       json_extract(r.provider_properties, '$.EmailVerificationSubject') AS "email_verification_subject",
       json_extract(r.provider_properties, '$.IssuerConfiguration') AS "issuer_configuration",
       json_extract(r.provider_properties, '$.KeyConfiguration') AS "key_configuration",
       json_extract(r.provider_properties, '$.LambdaConfig') AS "lambda_config",
       json_extract(r.provider_properties, '$.MfaConfiguration') AS "mfa_configuration",
       json_extract(r.provider_properties, '$.Policies') AS "policies",
       json_extract(r.provider_properties, '$.ProviderName') AS "provider_name",
       json_extract(r.provider_properties, '$.ProviderURL') AS "provider_url",
       json_extract(r.provider_properties, '$.Schema') AS "schema",
       json_extract(r.provider_properties, '$.SmsAuthenticationMessage') AS "sms_authentication_message",
       json_extract(r.provider_properties, '$.SmsConfiguration') AS "sms_configuration",
       json_extract(r.provider_properties, '$.SmsVerificationMessage') AS "sms_verification_message",
       json_extract(r.provider_properties, '$.UserAttributeUpdateSettings') AS "user_attribute_update_settings",
       json_extract(r.provider_properties, '$.UserPoolAddOns') AS "user_pool_add_ons",
       json_extract(r.provider_properties, '$.UserPoolId') AS "user_pool_id",
       json_extract(r.provider_properties, '$.UserPoolName') AS "user_pool_name",
       json_extract(r.provider_properties, '$.UserPoolTags') AS "user_pool_tags",
       json_extract(r.provider_properties, '$.UserPoolTier') AS "user_pool_tier",
       json_extract(r.provider_properties, '$.UsernameAttributes') AS "username_attributes",
       json_extract(r.provider_properties, '$.UsernameConfiguration') AS "username_configuration",
       json_extract(r.provider_properties, '$.VerificationMessageTemplate') AS "verification_message_template",
       json_extract(r.provider_properties, '$.WebAuthnFactorConfiguration') AS "web_authn_factor_configuration",
       json_extract(r.provider_properties, '$.WebAuthnRelyingPartyID') AS "web_authn_relying_party_id",
       json_extract(r.provider_properties, '$.WebAuthnUserVerification') AS "web_authn_user_verification"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Cognito::UserPool';

CREATE VIEW aws_cc_code_deploy_application AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationName') AS "application_name",
       json_extract(r.provider_properties, '$.ComputePlatform') AS "compute_platform",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodeDeploy::Application';

CREATE VIEW aws_cc_code_deploy_deployment_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ComputePlatform') AS "compute_platform",
       json_extract(r.provider_properties, '$.DeploymentConfigName') AS "deployment_config_name",
       json_extract(r.provider_properties, '$.MinimumHealthyHosts') AS "minimum_healthy_hosts",
       json_extract(r.provider_properties, '$.TrafficRoutingConfig') AS "traffic_routing_config",
       json_extract(r.provider_properties, '$.ZonalConfig') AS "zonal_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodeDeploy::DeploymentConfig';

CREATE VIEW aws_cc_image_builder_image_pipeline AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ContainerRecipeArn') AS "container_recipe_arn",
       json_extract(r.provider_properties, '$.DeploymentId') AS "deployment_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DistributionConfigurationArn') AS "distribution_configuration_arn",
       json_extract(r.provider_properties, '$.EnhancedImageMetadataEnabled') AS "enhanced_image_metadata_enabled",
       json_extract(r.provider_properties, '$.ExecutionRole') AS "execution_role",
       json_extract(r.provider_properties, '$.ImageRecipeArn') AS "image_recipe_arn",
       json_extract(r.provider_properties, '$.ImageScanningConfiguration') AS "image_scanning_configuration",
       json_extract(r.provider_properties, '$.ImageTags') AS "image_tags",
       json_extract(r.provider_properties, '$.ImageTestsConfiguration') AS "image_tests_configuration",
       json_extract(r.provider_properties, '$.InfrastructureConfigurationArn') AS "infrastructure_configuration_arn",
       json_extract(r.provider_properties, '$.LoggingConfiguration') AS "logging_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Schedule') AS "schedule",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Workflows') AS "workflows"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ImageBuilder::ImagePipeline';

CREATE VIEW aws_cc_image_builder_infrastructure_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.InstanceMetadataOptions') AS "instance_metadata_options",
       json_extract(r.provider_properties, '$.InstanceProfileName') AS "instance_profile_name",
       json_extract(r.provider_properties, '$.InstanceTypes') AS "instance_types",
       json_extract(r.provider_properties, '$.KeyPair') AS "key_pair",
       json_extract(r.provider_properties, '$.Logging') AS "logging",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ParentInstance') AS "parent_instance",
       json_extract(r.provider_properties, '$.Placement') AS "placement",
       json_extract(r.provider_properties, '$.ResourceTags') AS "resource_tags",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SnsTopicArn') AS "sns_topic_arn",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TerminateInstanceOnCompletion') AS "terminate_instance_on_completion",
       json_extract(r.provider_properties, '$.TerminateInstanceOnFailure') AS "terminate_instance_on_failure"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ImageBuilder::InfrastructureConfiguration';

CREATE VIEW aws_cc_image_builder_distribution_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Distributions') AS "distributions",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ImageBuilder::DistributionConfiguration';

PRAGMA user_version = 5;
