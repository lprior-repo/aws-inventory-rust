CREATE VIEW aws_cc_ec2_instance AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Affinity') AS "affinity",
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.BlockDeviceMappings') AS "block_device_mappings",
       json_extract(r.provider_properties, '$.CpuOptions') AS "cpu_options",
       json_extract(r.provider_properties, '$.CreditSpecification') AS "credit_specification",
       json_extract(r.provider_properties, '$.DisableApiTermination') AS "disable_api_termination",
       json_extract(r.provider_properties, '$.EbsOptimized') AS "ebs_optimized",
       json_extract(r.provider_properties, '$.ElasticGpuSpecifications') AS "elastic_gpu_specifications",
       json_extract(r.provider_properties, '$.ElasticInferenceAccelerators') AS "elastic_inference_accelerators",
       json_extract(r.provider_properties, '$.EnclaveOptions') AS "enclave_options",
       json_extract(r.provider_properties, '$.HibernationOptions') AS "hibernation_options",
       json_extract(r.provider_properties, '$.HostId') AS "host_id",
       json_extract(r.provider_properties, '$.HostResourceGroupArn') AS "host_resource_group_arn",
       json_extract(r.provider_properties, '$.IamInstanceProfile') AS "iam_instance_profile",
       json_extract(r.provider_properties, '$.ImageId') AS "image_id",
       json_extract(r.provider_properties, '$.InstanceId') AS "instance_id",
       json_extract(r.provider_properties, '$.InstanceInitiatedShutdownBehavior') AS "instance_initiated_shutdown_behavior",
       json_extract(r.provider_properties, '$.InstanceType') AS "instance_type",
       json_extract(r.provider_properties, '$.KernelId') AS "kernel_id",
       json_extract(r.provider_properties, '$.KeyName') AS "key_name",
       json_extract(r.provider_properties, '$.LicenseSpecifications') AS "license_specifications",
       json_extract(r.provider_properties, '$.MetadataOptions') AS "metadata_options",
       json_extract(r.provider_properties, '$.Monitoring') AS "monitoring",
       json_extract(r.provider_properties, '$.NetworkInterfaces') AS "network_interfaces",
       json_extract(r.provider_properties, '$.PlacementGroupName') AS "placement_group_name",
       json_extract(r.provider_properties, '$.PrivateDnsName') AS "private_dns_name",
       json_extract(r.provider_properties, '$.PrivateDnsNameOptions') AS "private_dns_name_options",
       json_extract(r.provider_properties, '$.PrivateIp') AS "private_ip",
       json_extract(r.provider_properties, '$.PrivateIpAddress') AS "private_ip_address",
       json_extract(r.provider_properties, '$.PublicDnsName') AS "public_dns_name",
       json_extract(r.provider_properties, '$.PublicIp') AS "public_ip",
       json_extract(r.provider_properties, '$.RamdiskId') AS "ramdisk_id",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SecurityGroups') AS "security_groups",
       json_extract(r.provider_properties, '$.SourceDestCheck') AS "source_dest_check",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Tenancy') AS "tenancy",
       json_extract(r.provider_properties, '$.Volumes') AS "volumes",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::Instance';

CREATE VIEW aws_cc_ec2_vpc AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CidrBlock') AS "cidr_block",
       json_extract(r.provider_properties, '$.CidrBlockAssociations') AS "cidr_block_associations",
       json_extract(r.provider_properties, '$.DefaultNetworkAcl') AS "default_network_acl",
       json_extract(r.provider_properties, '$.DefaultSecurityGroup') AS "default_security_group",
       json_extract(r.provider_properties, '$.EnableDnsHostnames') AS "enable_dns_hostnames",
       json_extract(r.provider_properties, '$.EnableDnsSupport') AS "enable_dns_support",
       json_extract(r.provider_properties, '$.InstanceTenancy') AS "instance_tenancy",
       json_extract(r.provider_properties, '$.Ipv6CidrBlocks') AS "ipv6_cidr_blocks",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcEncryptionControl') AS "vpc_encryption_control",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPC';

CREATE VIEW aws_cc_ec2_subnet AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AssignIpv6AddressOnCreation') AS "assign_ipv6_address_on_creation",
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.AvailabilityZoneId') AS "availability_zone_id",
       json_extract(r.provider_properties, '$.BlockPublicAccessStates') AS "block_public_access_states",
       json_extract(r.provider_properties, '$.CidrBlock') AS "cidr_block",
       json_extract(r.provider_properties, '$.EnableDns64') AS "enable_dns64",
       json_extract(r.provider_properties, '$.Ipv6CidrBlock') AS "ipv6_cidr_block",
       json_extract(r.provider_properties, '$.Ipv6CidrBlocks') AS "ipv6_cidr_blocks",
       json_extract(r.provider_properties, '$.Ipv6Native') AS "ipv6_native",
       json_extract(r.provider_properties, '$.MapPublicIpOnLaunch') AS "map_public_ip_on_launch",
       json_extract(r.provider_properties, '$.NetworkAclAssociationId') AS "network_acl_association_id",
       json_extract(r.provider_properties, '$.OutpostArn') AS "outpost_arn",
       json_extract(r.provider_properties, '$.PrivateDnsNameOptionsOnLaunch') AS "private_dns_name_options_on_launch",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::Subnet';

CREATE VIEW aws_cc_ec2_security_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.GroupDescription') AS "group_description",
       json_extract(r.provider_properties, '$.GroupId') AS "group_id",
       json_extract(r.provider_properties, '$.GroupName') AS "group_name",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.SecurityGroupEgress') AS "security_group_egress",
       json_extract(r.provider_properties, '$.SecurityGroupIngress') AS "security_group_ingress",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::SecurityGroup';

CREATE VIEW aws_cc_ec2_network_interface AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConnectionTrackingSpecification') AS "connection_tracking_specification",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EnablePrimaryIpv6') AS "enable_primary_ipv6",
       json_extract(r.provider_properties, '$.GroupSet') AS "group_set",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.InterfaceType') AS "interface_type",
       json_extract(r.provider_properties, '$.Ipv4PrefixCount') AS "ipv4_prefix_count",
       json_extract(r.provider_properties, '$.Ipv4Prefixes') AS "ipv4_prefixes",
       json_extract(r.provider_properties, '$.Ipv6AddressCount') AS "ipv6_address_count",
       json_extract(r.provider_properties, '$.Ipv6Addresses') AS "ipv6_addresses",
       json_extract(r.provider_properties, '$.Ipv6PrefixCount') AS "ipv6_prefix_count",
       json_extract(r.provider_properties, '$.Ipv6Prefixes') AS "ipv6_prefixes",
       json_extract(r.provider_properties, '$.PrimaryIpv6Address') AS "primary_ipv6_address",
       json_extract(r.provider_properties, '$.PrimaryPrivateIpAddress') AS "primary_private_ip_address",
       json_extract(r.provider_properties, '$.PrivateIpAddress') AS "private_ip_address",
       json_extract(r.provider_properties, '$.PrivateIpAddresses') AS "private_ip_addresses",
       json_extract(r.provider_properties, '$.PublicIpDnsNameOptions') AS "public_ip_dns_name_options",
       json_extract(r.provider_properties, '$.SecondaryPrivateIpAddressCount') AS "secondary_private_ip_address_count",
       json_extract(r.provider_properties, '$.SecondaryPrivateIpAddresses') AS "secondary_private_ip_addresses",
       json_extract(r.provider_properties, '$.SourceDestCheck') AS "source_dest_check",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NetworkInterface';

CREATE VIEW aws_cc_ec2_volume AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutoEnableIO') AS "auto_enable_io",
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.AvailabilityZoneId') AS "availability_zone_id",
       json_extract(r.provider_properties, '$.Encrypted') AS "encrypted",
       json_extract(r.provider_properties, '$.Iops') AS "iops",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.MultiAttachEnabled') AS "multi_attach_enabled",
       json_extract(r.provider_properties, '$.Size') AS "size",
       json_extract(r.provider_properties, '$.SnapshotId') AS "snapshot_id",
       json_extract(r.provider_properties, '$.SourceVolumeId') AS "source_volume_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Throughput') AS "throughput",
       json_extract(r.provider_properties, '$.VolumeId') AS "volume_id",
       json_extract(r.provider_properties, '$.VolumeInitializationRate') AS "volume_initialization_rate",
       json_extract(r.provider_properties, '$.VolumeType') AS "volume_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::Volume';

CREATE VIEW aws_cc_ec2_snapshot AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Encrypted') AS "encrypted",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.Progress') AS "progress",
       json_extract(r.provider_properties, '$.SnapshotId') AS "snapshot_id",
       json_extract(r.provider_properties, '$.StartTime') AS "start_time",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.StorageTier') AS "storage_tier",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VolumeId') AS "volume_id",
       json_extract(r.provider_properties, '$.VolumeSize') AS "volume_size"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::Snapshot';

CREATE VIEW aws_cc_ec2_internet_gateway AS
SELECT r.*,
       json_extract(r.provider_properties, '$.InternetGatewayId') AS "internet_gateway_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::InternetGateway';

CREATE VIEW aws_cc_ec2_nat_gateway AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllocationId') AS "allocation_id",
       json_extract(r.provider_properties, '$.AutoProvisionZones') AS "auto_provision_zones",
       json_extract(r.provider_properties, '$.AutoScalingIps') AS "auto_scaling_ips",
       json_extract(r.provider_properties, '$.AvailabilityMode') AS "availability_mode",
       json_extract(r.provider_properties, '$.AvailabilityZoneAddresses') AS "availability_zone_addresses",
       json_extract(r.provider_properties, '$.ConnectivityType') AS "connectivity_type",
       json_extract(r.provider_properties, '$.EniId') AS "eni_id",
       json_extract(r.provider_properties, '$.NatGatewayId') AS "nat_gateway_id",
       json_extract(r.provider_properties, '$.PrivateIpAddress') AS "private_ip_address",
       json_extract(r.provider_properties, '$.RouteTableId') AS "route_table_id",
       json_extract(r.provider_properties, '$.SecondaryAllocationIds') AS "secondary_allocation_ids",
       json_extract(r.provider_properties, '$.SecondaryPrivateIpAddressCount') AS "secondary_private_ip_address_count",
       json_extract(r.provider_properties, '$.SecondaryPrivateIpAddresses') AS "secondary_private_ip_addresses",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NatGateway';

CREATE VIEW aws_cc_ec2_route_table AS
SELECT r.*,
       json_extract(r.provider_properties, '$.RouteTableId') AS "route_table_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::RouteTable';

CREATE VIEW aws_cc_ec2_network_acl AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NetworkAcl';

CREATE VIEW aws_cc_ec2_vpc_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTimestamp') AS "creation_timestamp",
       json_extract(r.provider_properties, '$.DnsEntries') AS "dns_entries",
       json_extract(r.provider_properties, '$.DnsOptions') AS "dns_options",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.IpAddressType') AS "ip_address_type",
       json_extract(r.provider_properties, '$.NetworkInterfaceIds') AS "network_interface_ids",
       json_extract(r.provider_properties, '$.PolicyDocument') AS "policy_document",
       json_extract(r.provider_properties, '$.PrivateDnsEnabled') AS "private_dns_enabled",
       json_extract(r.provider_properties, '$.ResourceConfigurationArn') AS "resource_configuration_arn",
       json_extract(r.provider_properties, '$.RouteTableIds') AS "route_table_ids",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.ServiceName') AS "service_name",
       json_extract(r.provider_properties, '$.ServiceNetworkArn') AS "service_network_arn",
       json_extract(r.provider_properties, '$.ServiceRegion') AS "service_region",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcEndpointType') AS "vpc_endpoint_type",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPCEndpoint';

CREATE VIEW aws_cc_ec2_vpc_peering_connection AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.PeerOwnerId') AS "peer_owner_id",
       json_extract(r.provider_properties, '$.PeerRegion') AS "peer_region",
       json_extract(r.provider_properties, '$.PeerVpcId') AS "peer_vpc_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPCPeeringConnection';

CREATE VIEW aws_cc_ec2_egress_only_internet_gateway AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::EgressOnlyInternetGateway';

CREATE VIEW aws_cc_ec2_eip AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllocationId') AS "allocation_id",
       json_extract(r.provider_properties, '$.Domain') AS "domain",
       json_extract(r.provider_properties, '$.InstanceId') AS "instance_id",
       json_extract(r.provider_properties, '$.NetworkBorderGroup') AS "network_border_group",
       json_extract(r.provider_properties, '$.PublicIp') AS "public_ip",
       json_extract(r.provider_properties, '$.PublicIpv4Pool') AS "public_ipv4_pool",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::EIP';

CREATE VIEW aws_cc_ec2_customer_gateway AS
SELECT r.*,
       json_extract(r.provider_properties, '$.BgpAsn') AS "bgp_asn",
       json_extract(r.provider_properties, '$.BgpAsnExtended') AS "bgp_asn_extended",
       json_extract(r.provider_properties, '$.CertificateArn') AS "certificate_arn",
       json_extract(r.provider_properties, '$.CustomerGatewayId') AS "customer_gateway_id",
       json_extract(r.provider_properties, '$.DeviceName') AS "device_name",
       json_extract(r.provider_properties, '$.IpAddress') AS "ip_address",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::CustomerGateway';

CREATE VIEW aws_cc_ec2_vpn_gateway AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AmazonSideAsn') AS "amazon_side_asn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.VPNGatewayId') AS "vpn_gateway_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPNGateway';

CREATE VIEW aws_cc_ec2_vpn_connection AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CustomerGatewayId') AS "customer_gateway_id",
       json_extract(r.provider_properties, '$.EnableAcceleration') AS "enable_acceleration",
       json_extract(r.provider_properties, '$.LocalIpv4NetworkCidr') AS "local_ipv4_network_cidr",
       json_extract(r.provider_properties, '$.LocalIpv6NetworkCidr') AS "local_ipv6_network_cidr",
       json_extract(r.provider_properties, '$.OutsideIpAddressType') AS "outside_ip_address_type",
       json_extract(r.provider_properties, '$.RemoteIpv4NetworkCidr') AS "remote_ipv4_network_cidr",
       json_extract(r.provider_properties, '$.RemoteIpv6NetworkCidr') AS "remote_ipv6_network_cidr",
       json_extract(r.provider_properties, '$.StaticRoutesOnly') AS "static_routes_only",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id",
       json_extract(r.provider_properties, '$.TransportTransitGatewayAttachmentId') AS "transport_transit_gateway_attachment_id",
       json_extract(r.provider_properties, '$.TunnelBandwidth') AS "tunnel_bandwidth",
       json_extract(r.provider_properties, '$.TunnelInsideIpVersion') AS "tunnel_inside_ip_version",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.VpnConcentratorId') AS "vpn_concentrator_id",
       json_extract(r.provider_properties, '$.VpnConnectionId') AS "vpn_connection_id",
       json_extract(r.provider_properties, '$.VpnGatewayId') AS "vpn_gateway_id",
       json_extract(r.provider_properties, '$.VpnTunnelOptionsSpecifications') AS "vpn_tunnel_options_specifications"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPNConnection';

CREATE VIEW aws_cc_ec2_transit_gateway AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AmazonSideAsn') AS "amazon_side_asn",
       json_extract(r.provider_properties, '$.AssociationDefaultRouteTableId') AS "association_default_route_table_id",
       json_extract(r.provider_properties, '$.AutoAcceptSharedAttachments') AS "auto_accept_shared_attachments",
       json_extract(r.provider_properties, '$.DefaultRouteTableAssociation') AS "default_route_table_association",
       json_extract(r.provider_properties, '$.DefaultRouteTablePropagation') AS "default_route_table_propagation",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DnsSupport') AS "dns_support",
       json_extract(r.provider_properties, '$.EncryptionSupportState') AS "encryption_support_state",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.MulticastSupport') AS "multicast_support",
       json_extract(r.provider_properties, '$.PropagationDefaultRouteTableId') AS "propagation_default_route_table_id",
       json_extract(r.provider_properties, '$.SecurityGroupReferencingSupport') AS "security_group_referencing_support",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayArn') AS "transit_gateway_arn",
       json_extract(r.provider_properties, '$.TransitGatewayCidrBlocks') AS "transit_gateway_cidr_blocks",
       json_extract(r.provider_properties, '$.VpnEcmpSupport') AS "vpn_ecmp_support"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGateway';

CREATE VIEW aws_cc_ec2_transit_gateway_vpc_attachment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Options') AS "options",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGatewayVpcAttachment';

CREATE VIEW aws_cc_ec2_transit_gateway_peering_attachment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.PeerAccountId') AS "peer_account_id",
       json_extract(r.provider_properties, '$.PeerRegion') AS "peer_region",
       json_extract(r.provider_properties, '$.PeerTransitGatewayId') AS "peer_transit_gateway_id",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayAttachmentId') AS "transit_gateway_attachment_id",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGatewayPeeringAttachment';

CREATE VIEW aws_cc_ec2_transit_gateway_route_table AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id",
       json_extract(r.provider_properties, '$.TransitGatewayRouteTableId') AS "transit_gateway_route_table_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TransitGatewayRouteTable';

CREATE VIEW aws_cc_ec2_launch_template AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DefaultVersionNumber') AS "default_version_number",
       json_extract(r.provider_properties, '$.LatestVersionNumber') AS "latest_version_number",
       json_extract(r.provider_properties, '$.LaunchTemplateId') AS "launch_template_id",
       json_extract(r.provider_properties, '$.LaunchTemplateName') AS "launch_template_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::LaunchTemplate';

CREATE VIEW aws_cc_ec2_placement_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.GroupId') AS "group_id",
       json_extract(r.provider_properties, '$.GroupName') AS "group_name",
       json_extract(r.provider_properties, '$.ParentGroupId') AS "parent_group_id",
       json_extract(r.provider_properties, '$.PartitionCount') AS "partition_count",
       json_extract(r.provider_properties, '$.SpreadLevel') AS "spread_level",
       json_extract(r.provider_properties, '$.Strategy') AS "strategy",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::PlacementGroup';

CREATE VIEW aws_cc_ec2_capacity_reservation AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.AvailabilityZoneId') AS "availability_zone_id",
       json_extract(r.provider_properties, '$.AvailableInstanceCount') AS "available_instance_count",
       json_extract(r.provider_properties, '$.CapacityAllocationSet') AS "capacity_allocation_set",
       json_extract(r.provider_properties, '$.CapacityReservationArn') AS "capacity_reservation_arn",
       json_extract(r.provider_properties, '$.CapacityReservationFleetId') AS "capacity_reservation_fleet_id",
       json_extract(r.provider_properties, '$.CommitmentInfo') AS "commitment_info",
       json_extract(r.provider_properties, '$.CreateDate') AS "create_date",
       json_extract(r.provider_properties, '$.DeliveryPreference') AS "delivery_preference",
       json_extract(r.provider_properties, '$.EbsOptimized') AS "ebs_optimized",
       json_extract(r.provider_properties, '$.EndDate') AS "end_date",
       json_extract(r.provider_properties, '$.EndDateType') AS "end_date_type",
       json_extract(r.provider_properties, '$.EphemeralStorage') AS "ephemeral_storage",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.InstanceCount') AS "instance_count",
       json_extract(r.provider_properties, '$.InstanceMatchCriteria') AS "instance_match_criteria",
       json_extract(r.provider_properties, '$.InstancePlatform') AS "instance_platform",
       json_extract(r.provider_properties, '$.InstanceType') AS "instance_type",
       json_extract(r.provider_properties, '$.OutPostArn') AS "out_post_arn",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.PlacementGroupArn') AS "placement_group_arn",
       json_extract(r.provider_properties, '$.ReservationType') AS "reservation_type",
       json_extract(r.provider_properties, '$.StartDate') AS "start_date",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.TagSpecifications') AS "tag_specifications",
       json_extract(r.provider_properties, '$.Tenancy') AS "tenancy",
       json_extract(r.provider_properties, '$.TotalInstanceCount') AS "total_instance_count"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::CapacityReservation';

CREATE VIEW aws_cc_ec2_host AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AssetId') AS "asset_id",
       json_extract(r.provider_properties, '$.AutoPlacement') AS "auto_placement",
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.HostId') AS "host_id",
       json_extract(r.provider_properties, '$.HostMaintenance') AS "host_maintenance",
       json_extract(r.provider_properties, '$.HostRecovery') AS "host_recovery",
       json_extract(r.provider_properties, '$.InstanceFamily') AS "instance_family",
       json_extract(r.provider_properties, '$.InstanceType') AS "instance_type",
       json_extract(r.provider_properties, '$.OutpostArn') AS "outpost_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::Host';

CREATE VIEW aws_cc_ec2_flow_log AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DeliverCrossAccountRole') AS "deliver_cross_account_role",
       json_extract(r.provider_properties, '$.DeliverLogsPermissionArn') AS "deliver_logs_permission_arn",
       json_extract(r.provider_properties, '$.DestinationOptions') AS "destination_options",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.LogDestination') AS "log_destination",
       json_extract(r.provider_properties, '$.LogDestinationType') AS "log_destination_type",
       json_extract(r.provider_properties, '$.LogFormat') AS "log_format",
       json_extract(r.provider_properties, '$.LogGroupName') AS "log_group_name",
       json_extract(r.provider_properties, '$.MaxAggregationInterval') AS "max_aggregation_interval",
       json_extract(r.provider_properties, '$.ResourceId') AS "resource_id",
       json_extract(r.provider_properties, '$.ResourceType') AS "resource_type",
       json_extract(r.provider_properties, '$.TagFieldSpecifications') AS "tag_field_specifications",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrafficType') AS "traffic_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::FlowLog';

CREATE VIEW aws_cc_ec2_prefix_list AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AddressFamily') AS "address_family",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Entries') AS "entries",
       json_extract(r.provider_properties, '$.MaxEntries') AS "max_entries",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.PrefixListId') AS "prefix_list_id",
       json_extract(r.provider_properties, '$.PrefixListName') AS "prefix_list_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::PrefixList';

CREATE VIEW aws_cc_ec2_dhcp_options AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DhcpOptionsId') AS "dhcp_options_id",
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.DomainNameServers') AS "domain_name_servers",
       json_extract(r.provider_properties, '$.Ipv6AddressPreferredLeaseTime') AS "ipv6_address_preferred_lease_time",
       json_extract(r.provider_properties, '$.NetbiosNameServers') AS "netbios_name_servers",
       json_extract(r.provider_properties, '$.NetbiosNodeType') AS "netbios_node_type",
       json_extract(r.provider_properties, '$.NtpServers') AS "ntp_servers",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::DHCPOptions';

CREATE VIEW aws_cc_ec2_instance_connect_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.AvailabilityZoneId') AS "availability_zone_id",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.InstanceConnectEndpointArn') AS "instance_connect_endpoint_arn",
       json_extract(r.provider_properties, '$.NetworkInterfaceIds') AS "network_interface_ids",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.PreserveClientIp') AS "preserve_client_ip",
       json_extract(r.provider_properties, '$.PublicDnsNames') AS "public_dns_names",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.StateMessage') AS "state_message",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::InstanceConnectEndpoint';

CREATE VIEW aws_cc_ec2_network_insights_path AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreatedDate') AS "created_date",
       json_extract(r.provider_properties, '$.Destination') AS "destination",
       json_extract(r.provider_properties, '$.DestinationArn') AS "destination_arn",
       json_extract(r.provider_properties, '$.DestinationIp') AS "destination_ip",
       json_extract(r.provider_properties, '$.DestinationPort') AS "destination_port",
       json_extract(r.provider_properties, '$.FilterAtDestination') AS "filter_at_destination",
       json_extract(r.provider_properties, '$.FilterAtSource') AS "filter_at_source",
       json_extract(r.provider_properties, '$.NetworkInsightsPathArn') AS "network_insights_path_arn",
       json_extract(r.provider_properties, '$.NetworkInsightsPathId') AS "network_insights_path_id",
       json_extract(r.provider_properties, '$.Protocol') AS "protocol",
       json_extract(r.provider_properties, '$.Source') AS "provider_source",
       json_extract(r.provider_properties, '$.SourceArn') AS "source_arn",
       json_extract(r.provider_properties, '$.SourceIp') AS "source_ip",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::NetworkInsightsPath';

CREATE VIEW aws_cc_ec2_traffic_mirror_session AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.NetworkInterfaceId') AS "network_interface_id",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.PacketLength') AS "packet_length",
       json_extract(r.provider_properties, '$.SessionNumber') AS "session_number",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrafficMirrorFilterId') AS "traffic_mirror_filter_id",
       json_extract(r.provider_properties, '$.TrafficMirrorTargetId') AS "traffic_mirror_target_id",
       json_extract(r.provider_properties, '$.VirtualNetworkId') AS "virtual_network_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TrafficMirrorSession';

CREATE VIEW aws_cc_ec2_traffic_mirror_target AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.GatewayLoadBalancerEndpointId') AS "gateway_load_balancer_endpoint_id",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.NetworkInterfaceId') AS "network_interface_id",
       json_extract(r.provider_properties, '$.NetworkLoadBalancerArn') AS "network_load_balancer_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::TrafficMirrorTarget';

CREATE VIEW aws_cc_ec2_vpc_endpoint_service AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AcceptanceRequired') AS "acceptance_required",
       json_extract(r.provider_properties, '$.GatewayLoadBalancerArns') AS "gateway_load_balancer_arns",
       json_extract(r.provider_properties, '$.NetworkLoadBalancerArns') AS "network_load_balancer_arns",
       json_extract(r.provider_properties, '$.PayerResponsibility') AS "payer_responsibility",
       json_extract(r.provider_properties, '$.PrivateDnsName') AS "private_dns_name",
       json_extract(r.provider_properties, '$.PrivateDnsNameConfiguration') AS "private_dns_name_configuration",
       json_extract(r.provider_properties, '$.ServiceId') AS "service_id",
       json_extract(r.provider_properties, '$.SupportedIpAddressTypes') AS "supported_ip_address_types",
       json_extract(r.provider_properties, '$.SupportedRegions') AS "supported_regions",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::VPCEndpointService';

CREATE VIEW aws_cc_ec2_security_group_ingress AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CidrIp') AS "cidr_ip",
       json_extract(r.provider_properties, '$.CidrIpv6') AS "cidr_ipv6",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FromPort') AS "from_port",
       json_extract(r.provider_properties, '$.GroupId') AS "group_id",
       json_extract(r.provider_properties, '$.GroupName') AS "group_name",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.IpProtocol') AS "ip_protocol",
       json_extract(r.provider_properties, '$.SourcePrefixListId') AS "source_prefix_list_id",
       json_extract(r.provider_properties, '$.SourceSecurityGroupId') AS "source_security_group_id",
       json_extract(r.provider_properties, '$.SourceSecurityGroupName') AS "source_security_group_name",
       json_extract(r.provider_properties, '$.SourceSecurityGroupOwnerId') AS "source_security_group_owner_id",
       json_extract(r.provider_properties, '$.ToPort') AS "to_port"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::SecurityGroupIngress';

CREATE VIEW aws_cc_ec2_security_group_egress AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CidrIp') AS "cidr_ip",
       json_extract(r.provider_properties, '$.CidrIpv6') AS "cidr_ipv6",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DestinationPrefixListId') AS "destination_prefix_list_id",
       json_extract(r.provider_properties, '$.DestinationSecurityGroupId') AS "destination_security_group_id",
       json_extract(r.provider_properties, '$.FromPort') AS "from_port",
       json_extract(r.provider_properties, '$.GroupId') AS "group_id",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.IpProtocol') AS "ip_protocol",
       json_extract(r.provider_properties, '$.ToPort') AS "to_port"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::SecurityGroupEgress';

CREATE VIEW aws_cc_ec2_subnet_route_table_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.RouteTableId') AS "route_table_id",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::SubnetRouteTableAssociation';

CREATE VIEW aws_cc_ec2_subnet_network_acl_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AssociationId') AS "association_id",
       json_extract(r.provider_properties, '$.NetworkAclId') AS "network_acl_id",
       json_extract(r.provider_properties, '$.SubnetId') AS "subnet_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EC2::SubnetNetworkAclAssociation';

CREATE VIEW aws_cc_ecs_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CapacityProviders') AS "capacity_providers",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.ClusterSettings') AS "cluster_settings",
       json_extract(r.provider_properties, '$.Configuration') AS "configuration",
       json_extract(r.provider_properties, '$.DefaultCapacityProviderStrategy') AS "default_capacity_provider_strategy",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECS::Cluster';

CREATE VIEW aws_cc_ecs_service AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AvailabilityZoneRebalancing') AS "availability_zone_rebalancing",
       json_extract(r.provider_properties, '$.CapacityProviderStrategy') AS "capacity_provider_strategy",
       json_extract(r.provider_properties, '$.Cluster') AS "cluster",
       json_extract(r.provider_properties, '$.DeploymentConfiguration') AS "deployment_configuration",
       json_extract(r.provider_properties, '$.DeploymentController') AS "deployment_controller",
       json_extract(r.provider_properties, '$.DesiredCount') AS "desired_count",
       json_extract(r.provider_properties, '$.EnableECSManagedTags') AS "enable_ecs_managed_tags",
       json_extract(r.provider_properties, '$.EnableExecuteCommand') AS "enable_execute_command",
       json_extract(r.provider_properties, '$.HealthCheckGracePeriodSeconds') AS "health_check_grace_period_seconds",
       json_extract(r.provider_properties, '$.LaunchType') AS "launch_type",
       json_extract(r.provider_properties, '$.LoadBalancers') AS "load_balancers",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.NetworkConfiguration') AS "network_configuration",
       json_extract(r.provider_properties, '$.PlacementConstraints') AS "placement_constraints",
       json_extract(r.provider_properties, '$.PlacementStrategies') AS "placement_strategies",
       json_extract(r.provider_properties, '$.PlatformVersion') AS "platform_version",
       json_extract(r.provider_properties, '$.PropagateTags') AS "propagate_tags",
       json_extract(r.provider_properties, '$.Role') AS "role",
       json_extract(r.provider_properties, '$.SchedulingStrategy') AS "scheduling_strategy",
       json_extract(r.provider_properties, '$.ServiceArn') AS "service_arn",
       json_extract(r.provider_properties, '$.ServiceName') AS "service_name",
       json_extract(r.provider_properties, '$.ServiceRegistries') AS "service_registries",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TaskDefinition') AS "task_definition",
       json_extract(r.provider_properties, '$.VpcLatticeConfigurations') AS "vpc_lattice_configurations"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECS::Service';

CREATE VIEW aws_cc_ecs_task_definition AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ContainerDefinitions') AS "container_definitions",
       json_extract(r.provider_properties, '$.Cpu') AS "cpu",
       json_extract(r.provider_properties, '$.EnableFaultInjection') AS "enable_fault_injection",
       json_extract(r.provider_properties, '$.EphemeralStorage') AS "ephemeral_storage",
       json_extract(r.provider_properties, '$.ExecutionRoleArn') AS "execution_role_arn",
       json_extract(r.provider_properties, '$.Family') AS "family",
       json_extract(r.provider_properties, '$.InferenceAccelerators') AS "inference_accelerators",
       json_extract(r.provider_properties, '$.IpcMode') AS "ipc_mode",
       json_extract(r.provider_properties, '$.Memory') AS "memory",
       json_extract(r.provider_properties, '$.NetworkMode') AS "network_mode",
       json_extract(r.provider_properties, '$.PidMode') AS "pid_mode",
       json_extract(r.provider_properties, '$.PlacementConstraints') AS "placement_constraints",
       json_extract(r.provider_properties, '$.ProxyConfiguration') AS "proxy_configuration",
       json_extract(r.provider_properties, '$.RequiresCompatibilities') AS "requires_compatibilities",
       json_extract(r.provider_properties, '$.RuntimePlatform') AS "runtime_platform",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TaskDefinitionArn') AS "task_definition_arn",
       json_extract(r.provider_properties, '$.TaskRoleArn') AS "task_role_arn",
       json_extract(r.provider_properties, '$.Volumes') AS "volumes"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECS::TaskDefinition';

CREATE VIEW aws_cc_ecs_capacity_provider AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutoScalingGroupProvider') AS "auto_scaling_group_provider",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.ManagedInstancesProvider') AS "managed_instances_provider",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECS::CapacityProvider';

CREATE VIEW aws_cc_eks_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessConfig') AS "access_config",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CertificateAuthority') AS "certificate_authority",
       json_extract(r.provider_properties, '$.CertificateAuthorityData') AS "certificate_authority_data",
       json_extract(r.provider_properties, '$.ClusterSecurityGroupId') AS "cluster_security_group_id",
       json_extract(r.provider_properties, '$.ComputeConfig') AS "compute_config",
       json_extract(r.provider_properties, '$.ControlPlaneScalingConfig') AS "control_plane_scaling_config",
       json_extract(r.provider_properties, '$.DeletionProtection') AS "deletion_protection",
       json_extract(r.provider_properties, '$.EncryptionConfig') AS "encryption_config",
       json_extract(r.provider_properties, '$.EncryptionConfigKeyArn') AS "encryption_config_key_arn",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.KubeApiServerConfig') AS "kube_api_server_config",
       json_extract(r.provider_properties, '$.KubeControllerManagerConfig') AS "kube_controller_manager_config",
       json_extract(r.provider_properties, '$.KubeSchedulerConfig') AS "kube_scheduler_config",
       json_extract(r.provider_properties, '$.KubernetesNetworkConfig') AS "kubernetes_network_config",
       json_extract(r.provider_properties, '$.Logging') AS "logging",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OpenIdConnectIssuerUrl') AS "open_id_connect_issuer_url",
       json_extract(r.provider_properties, '$.OutpostConfig') AS "outpost_config",
       json_extract(r.provider_properties, '$.RemoteNetworkConfig') AS "remote_network_config",
       json_extract(r.provider_properties, '$.ResourcesVpcConfig') AS "resources_vpc_config",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.StorageConfig') AS "storage_config",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UpgradePolicy') AS "upgrade_policy",
       json_extract(r.provider_properties, '$.Version') AS "version",
       json_extract(r.provider_properties, '$.ZonalShiftConfig') AS "zonal_shift_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EKS::Cluster';

CREATE VIEW aws_cc_elastic_load_balancing_v2_load_balancer AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CanonicalHostedZoneID') AS "canonical_hosted_zone_id",
       json_extract(r.provider_properties, '$.DNSName') AS "dns_name",
       json_extract(r.provider_properties, '$.EnablePrefixForIpv6SourceNat') AS "enable_prefix_for_ipv6_source_nat",
       json_extract(r.provider_properties, '$.EnforceSecurityGroupInboundRulesOnPrivateLinkTraffic') AS "enforce_security_group_inbound_rules_on_private_link_traffic",
       json_extract(r.provider_properties, '$.IpAddressType') AS "ip_address_type",
       json_extract(r.provider_properties, '$.Ipv4IpamPoolId') AS "ipv4_ipam_pool_id",
       json_extract(r.provider_properties, '$.LoadBalancerArn') AS "load_balancer_arn",
       json_extract(r.provider_properties, '$.LoadBalancerAttributes') AS "load_balancer_attributes",
       json_extract(r.provider_properties, '$.LoadBalancerFullName') AS "load_balancer_full_name",
       json_extract(r.provider_properties, '$.LoadBalancerName') AS "load_balancer_name",
       json_extract(r.provider_properties, '$.MinimumLoadBalancerCapacity') AS "minimum_load_balancer_capacity",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Scheme') AS "scheme",
       json_extract(r.provider_properties, '$.SecurityGroups') AS "security_groups",
       json_extract(r.provider_properties, '$.SubnetMappings') AS "subnet_mappings",
       json_extract(r.provider_properties, '$.Subnets') AS "subnets",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElasticLoadBalancingV2::LoadBalancer';

CREATE VIEW aws_cc_elastic_load_balancing_v2_target_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.HealthCheckEnabled') AS "health_check_enabled",
       json_extract(r.provider_properties, '$.HealthCheckIntervalSeconds') AS "health_check_interval_seconds",
       json_extract(r.provider_properties, '$.HealthCheckPath') AS "health_check_path",
       json_extract(r.provider_properties, '$.HealthCheckPort') AS "health_check_port",
       json_extract(r.provider_properties, '$.HealthCheckProtocol') AS "health_check_protocol",
       json_extract(r.provider_properties, '$.HealthCheckTimeoutSeconds') AS "health_check_timeout_seconds",
       json_extract(r.provider_properties, '$.HealthyThresholdCount') AS "healthy_threshold_count",
       json_extract(r.provider_properties, '$.IpAddressType') AS "ip_address_type",
       json_extract(r.provider_properties, '$.LoadBalancerArns') AS "load_balancer_arns",
       json_extract(r.provider_properties, '$.Matcher') AS "matcher",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.Protocol') AS "protocol",
       json_extract(r.provider_properties, '$.ProtocolVersion') AS "protocol_version",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetControlPort') AS "target_control_port",
       json_extract(r.provider_properties, '$.TargetGroupArn') AS "target_group_arn",
       json_extract(r.provider_properties, '$.TargetGroupAttributes') AS "target_group_attributes",
       json_extract(r.provider_properties, '$.TargetGroupFullName') AS "target_group_full_name",
       json_extract(r.provider_properties, '$.TargetGroupName') AS "target_group_name",
       json_extract(r.provider_properties, '$.TargetType') AS "target_type",
       json_extract(r.provider_properties, '$.Targets') AS "targets",
       json_extract(r.provider_properties, '$.UnhealthyThresholdCount') AS "unhealthy_threshold_count",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElasticLoadBalancingV2::TargetGroup';

CREATE VIEW aws_cc_lambda_function AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Architectures') AS "architectures",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CapacityProviderConfig') AS "capacity_provider_config",
       json_extract(r.provider_properties, '$.CodeSigningConfigArn') AS "code_signing_config_arn",
       json_extract(r.provider_properties, '$.DeadLetterConfig') AS "dead_letter_config",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DurableConfig') AS "durable_config",
       json_extract(r.provider_properties, '$.EphemeralStorage') AS "ephemeral_storage",
       json_extract(r.provider_properties, '$.FileSystemConfigs') AS "file_system_configs",
       json_extract(r.provider_properties, '$.FunctionName') AS "function_name",
       json_extract(r.provider_properties, '$.FunctionScalingConfig') AS "function_scaling_config",
       json_extract(r.provider_properties, '$.Handler') AS "handler",
       json_extract(r.provider_properties, '$.ImageConfig') AS "image_config",
       json_extract(r.provider_properties, '$.KmsKeyArn') AS "kms_key_arn",
       json_extract(r.provider_properties, '$.Layers') AS "layers",
       json_extract(r.provider_properties, '$.LoggingConfig') AS "logging_config",
       json_extract(r.provider_properties, '$.MemorySize') AS "memory_size",
       json_extract(r.provider_properties, '$.PackageType') AS "package_type",
       json_extract(r.provider_properties, '$.RecursiveLoop') AS "recursive_loop",
       json_extract(r.provider_properties, '$.ReservedConcurrentExecutions') AS "reserved_concurrent_executions",
       json_extract(r.provider_properties, '$.Role') AS "role",
       json_extract(r.provider_properties, '$.Runtime') AS "runtime",
       json_extract(r.provider_properties, '$.RuntimeManagementConfig') AS "runtime_management_config",
       json_extract(r.provider_properties, '$.SnapStartResponse') AS "snap_start_response",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TenancyConfig') AS "tenancy_config",
       json_extract(r.provider_properties, '$.Timeout') AS "timeout",
       json_extract(r.provider_properties, '$.TracingConfig') AS "tracing_config",
       json_extract(r.provider_properties, '$.VpcConfig') AS "vpc_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Lambda::Function';

CREATE VIEW aws_cc_lambda_event_source_mapping AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AmazonManagedKafkaEventSourceConfig') AS "amazon_managed_kafka_event_source_config",
       json_extract(r.provider_properties, '$.BatchSize') AS "batch_size",
       json_extract(r.provider_properties, '$.BisectBatchOnFunctionError') AS "bisect_batch_on_function_error",
       json_extract(r.provider_properties, '$.DestinationConfig') AS "destination_config",
       json_extract(r.provider_properties, '$.DocumentDBEventSourceConfig') AS "document_db_event_source_config",
       json_extract(r.provider_properties, '$.Enabled') AS "enabled",
       json_extract(r.provider_properties, '$.EventSourceArn') AS "event_source_arn",
       json_extract(r.provider_properties, '$.EventSourceMappingArn') AS "event_source_mapping_arn",
       json_extract(r.provider_properties, '$.FilterCriteria') AS "filter_criteria",
       json_extract(r.provider_properties, '$.FunctionName') AS "function_name",
       json_extract(r.provider_properties, '$.FunctionResponseTypes') AS "function_response_types",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.KmsKeyArn') AS "kms_key_arn",
       json_extract(r.provider_properties, '$.LoggingConfig') AS "logging_config",
       json_extract(r.provider_properties, '$.MaximumBatchingWindowInSeconds') AS "maximum_batching_window_in_seconds",
       json_extract(r.provider_properties, '$.MaximumRecordAgeInSeconds') AS "maximum_record_age_in_seconds",
       json_extract(r.provider_properties, '$.MaximumRetryAttempts') AS "maximum_retry_attempts",
       json_extract(r.provider_properties, '$.MetricsConfig') AS "metrics_config",
       json_extract(r.provider_properties, '$.ParallelizationFactor') AS "parallelization_factor",
       json_extract(r.provider_properties, '$.ProvisionedPollerConfig') AS "provisioned_poller_config",
       json_extract(r.provider_properties, '$.Queues') AS "queues",
       json_extract(r.provider_properties, '$.ScalingConfig') AS "scaling_config",
       json_extract(r.provider_properties, '$.SelfManagedEventSource') AS "self_managed_event_source",
       json_extract(r.provider_properties, '$.SelfManagedKafkaEventSourceConfig') AS "self_managed_kafka_event_source_config",
       json_extract(r.provider_properties, '$.SourceAccessConfigurations') AS "source_access_configurations",
       json_extract(r.provider_properties, '$.StartingPosition') AS "starting_position",
       json_extract(r.provider_properties, '$.StartingPositionTimestamp') AS "starting_position_timestamp",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Topics') AS "topics",
       json_extract(r.provider_properties, '$.TumblingWindowInSeconds') AS "tumbling_window_in_seconds"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Lambda::EventSourceMapping';

CREATE VIEW aws_cc_lambda_code_signing_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllowedPublishers') AS "allowed_publishers",
       json_extract(r.provider_properties, '$.CodeSigningConfigArn') AS "code_signing_config_arn",
       json_extract(r.provider_properties, '$.CodeSigningConfigId') AS "code_signing_config_id",
       json_extract(r.provider_properties, '$.CodeSigningPolicies') AS "code_signing_policies",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Lambda::CodeSigningConfig';

CREATE VIEW aws_cc_dynamo_db_table AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AttributeDefinitions') AS "attribute_definitions",
       json_extract(r.provider_properties, '$.BillingMode') AS "billing_mode",
       json_extract(r.provider_properties, '$.ContributorInsightsSpecification') AS "contributor_insights_specification",
       json_extract(r.provider_properties, '$.DeletionProtectionEnabled') AS "deletion_protection_enabled",
       json_extract(r.provider_properties, '$.GlobalSecondaryIndexes') AS "global_secondary_indexes",
       json_extract(r.provider_properties, '$.KeySchema') AS "key_schema",
       json_extract(r.provider_properties, '$.KinesisStreamSpecification') AS "kinesis_stream_specification",
       json_extract(r.provider_properties, '$.LocalSecondaryIndexes') AS "local_secondary_indexes",
       json_extract(r.provider_properties, '$.OnDemandThroughput') AS "on_demand_throughput",
       json_extract(r.provider_properties, '$.PointInTimeRecoverySpecification') AS "point_in_time_recovery_specification",
       json_extract(r.provider_properties, '$.ProvisionedThroughput') AS "provisioned_throughput",
       json_extract(r.provider_properties, '$.ResourcePolicy') AS "resource_policy",
       json_extract(r.provider_properties, '$.SSESpecification') AS "sse_specification",
       json_extract(r.provider_properties, '$.StreamArn') AS "stream_arn",
       json_extract(r.provider_properties, '$.StreamSpecification') AS "stream_specification",
       json_extract(r.provider_properties, '$.TableClass') AS "table_class",
       json_extract(r.provider_properties, '$.TableName') AS "table_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TimeToLiveSpecification') AS "time_to_live_specification",
       json_extract(r.provider_properties, '$.VectorIndexes') AS "vector_indexes",
       json_extract(r.provider_properties, '$.WarmThroughput') AS "warm_throughput"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DynamoDB::Table';

CREATE VIEW aws_cc_rds_db_instance AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdditionalStorageVolumes') AS "additional_storage_volumes",
       json_extract(r.provider_properties, '$.AllocatedStorage') AS "allocated_storage",
       json_extract(r.provider_properties, '$.AssociatedRoles') AS "associated_roles",
       json_extract(r.provider_properties, '$.AutoMinorVersionUpgrade') AS "auto_minor_version_upgrade",
       json_extract(r.provider_properties, '$.AutomaticBackupReplicationRegion') AS "automatic_backup_replication_region",
       json_extract(r.provider_properties, '$.AutomaticBackupReplicationRetentionPeriod') AS "automatic_backup_replication_retention_period",
       json_extract(r.provider_properties, '$.AutomaticRestartTime') AS "automatic_restart_time",
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.BackupRetentionPeriod') AS "backup_retention_period",
       json_extract(r.provider_properties, '$.BackupTarget') AS "backup_target",
       json_extract(r.provider_properties, '$.CACertificateIdentifier') AS "ca_certificate_identifier",
       json_extract(r.provider_properties, '$.CertificateDetails') AS "certificate_details",
       json_extract(r.provider_properties, '$.CharacterSetName') AS "character_set_name",
       json_extract(r.provider_properties, '$.CopyTagsToSnapshot') AS "copy_tags_to_snapshot",
       json_extract(r.provider_properties, '$.CustomIAMInstanceProfile') AS "custom_iam_instance_profile",
       json_extract(r.provider_properties, '$.DBClusterIdentifier') AS "db_cluster_identifier",
       json_extract(r.provider_properties, '$.DBClusterSnapshotIdentifier') AS "db_cluster_snapshot_identifier",
       json_extract(r.provider_properties, '$.DBInstanceArn') AS "db_instance_arn",
       json_extract(r.provider_properties, '$.DBInstanceClass') AS "db_instance_class",
       json_extract(r.provider_properties, '$.DBInstanceIdentifier') AS "db_instance_identifier",
       json_extract(r.provider_properties, '$.DBInstanceStatus') AS "db_instance_status",
       json_extract(r.provider_properties, '$.DBName') AS "db_name",
       json_extract(r.provider_properties, '$.DBParameterGroupName') AS "db_parameter_group_name",
       json_extract(r.provider_properties, '$.DBSecurityGroups') AS "db_security_groups",
       json_extract(r.provider_properties, '$.DBSubnetGroupName') AS "db_subnet_group_name",
       json_extract(r.provider_properties, '$.DBSystemId') AS "db_system_id",
       json_extract(r.provider_properties, '$.DatabaseInsightsMode') AS "database_insights_mode",
       json_extract(r.provider_properties, '$.DbiResourceId') AS "dbi_resource_id",
       json_extract(r.provider_properties, '$.DedicatedLogVolume') AS "dedicated_log_volume",
       json_extract(r.provider_properties, '$.DeletionProtection') AS "deletion_protection",
       json_extract(r.provider_properties, '$.Domain') AS "domain",
       json_extract(r.provider_properties, '$.DomainAuthSecretArn') AS "domain_auth_secret_arn",
       json_extract(r.provider_properties, '$.DomainDnsIps') AS "domain_dns_ips",
       json_extract(r.provider_properties, '$.DomainFqdn') AS "domain_fqdn",
       json_extract(r.provider_properties, '$.DomainIAMRoleName') AS "domain_iam_role_name",
       json_extract(r.provider_properties, '$.DomainOu') AS "domain_ou",
       json_extract(r.provider_properties, '$.EnableCloudwatchLogsExports') AS "enable_cloudwatch_logs_exports",
       json_extract(r.provider_properties, '$.EnableIAMDatabaseAuthentication') AS "enable_iam_database_authentication",
       json_extract(r.provider_properties, '$.EnablePerformanceInsights') AS "enable_performance_insights",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.EngineLifecycleSupport') AS "engine_lifecycle_support",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.InstanceCreateTime') AS "instance_create_time",
       json_extract(r.provider_properties, '$.Iops') AS "iops",
       json_extract(r.provider_properties, '$.IsStorageConfigUpgradeAvailable') AS "is_storage_config_upgrade_available",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LatestRestorableTime') AS "latest_restorable_time",
       json_extract(r.provider_properties, '$.LicenseModel') AS "license_model",
       json_extract(r.provider_properties, '$.ListenerEndpoint') AS "listener_endpoint",
       json_extract(r.provider_properties, '$.ManageMasterUserPassword') AS "manage_master_user_password",
       json_extract(r.provider_properties, '$.MasterUserSecret') AS "master_user_secret",
       json_extract(r.provider_properties, '$.MasterUsername') AS "master_username",
       json_extract(r.provider_properties, '$.MaxAllocatedStorage') AS "max_allocated_storage",
       json_extract(r.provider_properties, '$.MonitoringInterval') AS "monitoring_interval",
       json_extract(r.provider_properties, '$.MonitoringRoleArn') AS "monitoring_role_arn",
       json_extract(r.provider_properties, '$.MultiAZ') AS "multi_az",
       json_extract(r.provider_properties, '$.NcharCharacterSetName') AS "nchar_character_set_name",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.OptionGroupName') AS "option_group_name",
       json_extract(r.provider_properties, '$.PercentProgress') AS "percent_progress",
       json_extract(r.provider_properties, '$.PerformanceInsightsKMSKeyId') AS "performance_insights_kms_key_id",
       json_extract(r.provider_properties, '$.PerformanceInsightsRetentionPeriod') AS "performance_insights_retention_period",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PreferredBackupWindow') AS "preferred_backup_window",
       json_extract(r.provider_properties, '$.PreferredMaintenanceWindow') AS "preferred_maintenance_window",
       json_extract(r.provider_properties, '$.ProcessorFeatures') AS "processor_features",
       json_extract(r.provider_properties, '$.PromotionTier') AS "promotion_tier",
       json_extract(r.provider_properties, '$.PubliclyAccessible') AS "publicly_accessible",
       json_extract(r.provider_properties, '$.ReadReplicaDBClusterIdentifiers') AS "read_replica_db_cluster_identifiers",
       json_extract(r.provider_properties, '$.ReadReplicaDBInstanceIdentifiers') AS "read_replica_db_instance_identifiers",
       json_extract(r.provider_properties, '$.ReplicaMode') AS "replica_mode",
       json_extract(r.provider_properties, '$.ResumeFullAutomationModeTime') AS "resume_full_automation_mode_time",
       json_extract(r.provider_properties, '$.SecondaryAvailabilityZone') AS "secondary_availability_zone",
       json_extract(r.provider_properties, '$.SourceDBClusterIdentifier') AS "source_db_cluster_identifier",
       json_extract(r.provider_properties, '$.StatusInfos') AS "status_infos",
       json_extract(r.provider_properties, '$.StorageEncrypted') AS "storage_encrypted",
       json_extract(r.provider_properties, '$.StorageOperationPercentProgress') AS "storage_operation_percent_progress",
       json_extract(r.provider_properties, '$.StorageOperationStatus') AS "storage_operation_status",
       json_extract(r.provider_properties, '$.StorageThroughput') AS "storage_throughput",
       json_extract(r.provider_properties, '$.StorageType') AS "storage_type",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TdeCredentialArn') AS "tde_credential_arn",
       json_extract(r.provider_properties, '$.Timezone') AS "timezone",
       json_extract(r.provider_properties, '$.VPCSecurityGroups') AS "vpc_security_groups"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBInstance';

CREATE VIEW aws_cc_rds_db_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllocatedStorage') AS "allocated_storage",
       json_extract(r.provider_properties, '$.AssociatedRoles') AS "associated_roles",
       json_extract(r.provider_properties, '$.AutoMinorVersionUpgrade') AS "auto_minor_version_upgrade",
       json_extract(r.provider_properties, '$.AvailabilityZones') AS "availability_zones",
       json_extract(r.provider_properties, '$.BacktrackWindow') AS "backtrack_window",
       json_extract(r.provider_properties, '$.BackupRetentionPeriod') AS "backup_retention_period",
       json_extract(r.provider_properties, '$.CopyTagsToSnapshot') AS "copy_tags_to_snapshot",
       json_extract(r.provider_properties, '$.DBClusterArn') AS "db_cluster_arn",
       json_extract(r.provider_properties, '$.DBClusterIdentifier') AS "db_cluster_identifier",
       json_extract(r.provider_properties, '$.DBClusterInstanceClass') AS "db_cluster_instance_class",
       json_extract(r.provider_properties, '$.DBClusterParameterGroupName') AS "db_cluster_parameter_group_name",
       json_extract(r.provider_properties, '$.DBClusterResourceId') AS "db_cluster_resource_id",
       json_extract(r.provider_properties, '$.DBSubnetGroupName') AS "db_subnet_group_name",
       json_extract(r.provider_properties, '$.DBSystemId') AS "db_system_id",
       json_extract(r.provider_properties, '$.DatabaseInsightsMode') AS "database_insights_mode",
       json_extract(r.provider_properties, '$.DatabaseName') AS "database_name",
       json_extract(r.provider_properties, '$.DeletionProtection') AS "deletion_protection",
       json_extract(r.provider_properties, '$.Domain') AS "domain",
       json_extract(r.provider_properties, '$.DomainIAMRoleName') AS "domain_iam_role_name",
       json_extract(r.provider_properties, '$.EnableCloudwatchLogsExports') AS "enable_cloudwatch_logs_exports",
       json_extract(r.provider_properties, '$.EnableGlobalWriteForwarding') AS "enable_global_write_forwarding",
       json_extract(r.provider_properties, '$.EnableHttpEndpoint') AS "enable_http_endpoint",
       json_extract(r.provider_properties, '$.EnableIAMDatabaseAuthentication') AS "enable_iam_database_authentication",
       json_extract(r.provider_properties, '$.EnableLocalWriteForwarding') AS "enable_local_write_forwarding",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.EngineLifecycleSupport') AS "engine_lifecycle_support",
       json_extract(r.provider_properties, '$.EngineMode') AS "engine_mode",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.GlobalClusterIdentifier') AS "global_cluster_identifier",
       json_extract(r.provider_properties, '$.Iops') AS "iops",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.ManageMasterUserPassword') AS "manage_master_user_password",
       json_extract(r.provider_properties, '$.MasterUserSecret') AS "master_user_secret",
       json_extract(r.provider_properties, '$.MasterUsername') AS "master_username",
       json_extract(r.provider_properties, '$.MonitoringInterval') AS "monitoring_interval",
       json_extract(r.provider_properties, '$.MonitoringRoleArn') AS "monitoring_role_arn",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.PerformanceInsightsEnabled') AS "performance_insights_enabled",
       json_extract(r.provider_properties, '$.PerformanceInsightsKmsKeyId') AS "performance_insights_kms_key_id",
       json_extract(r.provider_properties, '$.PerformanceInsightsRetentionPeriod') AS "performance_insights_retention_period",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PreferredBackupWindow') AS "preferred_backup_window",
       json_extract(r.provider_properties, '$.PreferredMaintenanceWindow') AS "preferred_maintenance_window",
       json_extract(r.provider_properties, '$.PubliclyAccessible') AS "publicly_accessible",
       json_extract(r.provider_properties, '$.ReadEndpoint') AS "read_endpoint",
       json_extract(r.provider_properties, '$.ReplicationSourceIdentifier') AS "replication_source_identifier",
       json_extract(r.provider_properties, '$.ScalingConfiguration') AS "scaling_configuration",
       json_extract(r.provider_properties, '$.ServerlessV2ScalingConfiguration') AS "serverless_v2_scaling_configuration",
       json_extract(r.provider_properties, '$.StorageEncrypted') AS "storage_encrypted",
       json_extract(r.provider_properties, '$.StorageEncryptionType') AS "storage_encryption_type",
       json_extract(r.provider_properties, '$.StorageThroughput') AS "storage_throughput",
       json_extract(r.provider_properties, '$.StorageType') AS "storage_type",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcSecurityGroupIds') AS "vpc_security_group_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBCluster';

CREATE VIEW aws_cc_rds_db_subnet_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DBSubnetGroupArn') AS "db_subnet_group_arn",
       json_extract(r.provider_properties, '$.DBSubnetGroupDescription') AS "db_subnet_group_description",
       json_extract(r.provider_properties, '$.DBSubnetGroupName') AS "db_subnet_group_name",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBSubnetGroup';

CREATE VIEW aws_cc_rds_db_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DBParameterGroupArn') AS "db_parameter_group_arn",
       json_extract(r.provider_properties, '$.DBParameterGroupName') AS "db_parameter_group_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Family') AS "family",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBParameterGroup';

CREATE VIEW aws_cc_rds_db_cluster_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DBClusterParameterGroupName') AS "db_cluster_parameter_group_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Family') AS "family",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBClusterParameterGroup';

CREATE VIEW aws_cc_rds_db_proxy AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Auth') AS "auth",
       json_extract(r.provider_properties, '$.DBProxyArn') AS "db_proxy_arn",
       json_extract(r.provider_properties, '$.DBProxyName') AS "db_proxy_name",
       json_extract(r.provider_properties, '$.DebugLogging') AS "debug_logging",
       json_extract(r.provider_properties, '$.DefaultAuthScheme') AS "default_auth_scheme",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.EndpointNetworkType') AS "endpoint_network_type",
       json_extract(r.provider_properties, '$.EngineFamily') AS "engine_family",
       json_extract(r.provider_properties, '$.IdleClientTimeout') AS "idle_client_timeout",
       json_extract(r.provider_properties, '$.RequireTLS') AS "require_tls",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetConnectionNetworkType') AS "target_connection_network_type",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id",
       json_extract(r.provider_properties, '$.VpcSecurityGroupIds') AS "vpc_security_group_ids",
       json_extract(r.provider_properties, '$.VpcSubnetIds') AS "vpc_subnet_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBProxy';

CREATE VIEW aws_cc_rds_db_proxy_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DBProxyEndpointArn') AS "db_proxy_endpoint_arn",
       json_extract(r.provider_properties, '$.DBProxyEndpointName') AS "db_proxy_endpoint_name",
       json_extract(r.provider_properties, '$.DBProxyName') AS "db_proxy_name",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.EndpointNetworkType') AS "endpoint_network_type",
       json_extract(r.provider_properties, '$.IsDefault') AS "is_default",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetRole') AS "target_role",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id",
       json_extract(r.provider_properties, '$.VpcSecurityGroupIds') AS "vpc_security_group_ids",
       json_extract(r.provider_properties, '$.VpcSubnetIds') AS "vpc_subnet_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBProxyEndpoint';

CREATE VIEW aws_cc_rds_db_cluster_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CustomEndpointType') AS "custom_endpoint_type",
       json_extract(r.provider_properties, '$.DBClusterEndpointArn') AS "db_cluster_endpoint_arn",
       json_extract(r.provider_properties, '$.DBClusterEndpointIdentifier') AS "db_cluster_endpoint_identifier",
       json_extract(r.provider_properties, '$.DBClusterEndpointResourceIdentifier') AS "db_cluster_endpoint_resource_identifier",
       json_extract(r.provider_properties, '$.DBClusterIdentifier') AS "db_cluster_identifier",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.EndpointType') AS "endpoint_type",
       json_extract(r.provider_properties, '$.ExcludedMembers') AS "excluded_members",
       json_extract(r.provider_properties, '$.StaticMembers') AS "static_members",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBClusterEndpoint';

CREATE VIEW aws_cc_rds_db_snapshot AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllocatedStorage') AS "allocated_storage",
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.DBInstanceIdentifier') AS "db_instance_identifier",
       json_extract(r.provider_properties, '$.DBSnapshotArn') AS "db_snapshot_arn",
       json_extract(r.provider_properties, '$.DBSnapshotIdentifier') AS "db_snapshot_identifier",
       json_extract(r.provider_properties, '$.DbiResourceId') AS "dbi_resource_id",
       json_extract(r.provider_properties, '$.Encrypted') AS "encrypted",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.IAMDatabaseAuthenticationEnabled') AS "iam_database_authentication_enabled",
       json_extract(r.provider_properties, '$.InstanceCreateTime') AS "instance_create_time",
       json_extract(r.provider_properties, '$.Iops') AS "iops",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LicenseModel') AS "license_model",
       json_extract(r.provider_properties, '$.MasterUsername') AS "master_username",
       json_extract(r.provider_properties, '$.OptionGroupName') AS "option_group_name",
       json_extract(r.provider_properties, '$.OriginalSnapshotCreateTime') AS "original_snapshot_create_time",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.SnapshotCreateTime') AS "snapshot_create_time",
       json_extract(r.provider_properties, '$.SnapshotType') AS "snapshot_type",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StorageThroughput') AS "storage_throughput",
       json_extract(r.provider_properties, '$.StorageType') AS "storage_type",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::DBSnapshot';

CREATE VIEW aws_cc_rds_option_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.EngineName') AS "engine_name",
       json_extract(r.provider_properties, '$.MajorEngineVersion') AS "major_engine_version",
       json_extract(r.provider_properties, '$.OptionConfigurations') AS "option_configurations",
       json_extract(r.provider_properties, '$.OptionGroupDescription') AS "option_group_description",
       json_extract(r.provider_properties, '$.OptionGroupName') AS "option_group_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RDS::OptionGroup';

CREATE VIEW aws_cc_elasti_cache_cache_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AZMode') AS "az_mode",
       json_extract(r.provider_properties, '$.AutoMinorVersionUpgrade') AS "auto_minor_version_upgrade",
       json_extract(r.provider_properties, '$.CacheNodeType') AS "cache_node_type",
       json_extract(r.provider_properties, '$.CacheParameterGroupName') AS "cache_parameter_group_name",
       json_extract(r.provider_properties, '$.CacheSubnetGroupName') AS "cache_subnet_group_name",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.ConfigurationEndpoint') AS "configuration_endpoint",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.IpDiscovery') AS "ip_discovery",
       json_extract(r.provider_properties, '$.LogDeliveryConfigurations') AS "log_delivery_configurations",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.NotificationTopicArn') AS "notification_topic_arn",
       json_extract(r.provider_properties, '$.NumCacheNodes') AS "num_cache_nodes",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PreferredAvailabilityZones') AS "preferred_availability_zones",
       json_extract(r.provider_properties, '$.PreferredMaintenanceWindow') AS "preferred_maintenance_window",
       json_extract(r.provider_properties, '$.RedisEndpoint') AS "redis_endpoint",
       json_extract(r.provider_properties, '$.SnapshotRetentionLimit') AS "snapshot_retention_limit",
       json_extract(r.provider_properties, '$.SnapshotWindow') AS "snapshot_window",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitEncryptionEnabled') AS "transit_encryption_enabled",
       json_extract(r.provider_properties, '$.VpcSecurityGroupIds') AS "vpc_security_group_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::CacheCluster';

CREATE VIEW aws_cc_elasti_cache_replication_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AtRestEncryptionEnabled') AS "at_rest_encryption_enabled",
       json_extract(r.provider_properties, '$.AutoMinorVersionUpgrade') AS "auto_minor_version_upgrade",
       json_extract(r.provider_properties, '$.AutomaticFailoverEnabled') AS "automatic_failover_enabled",
       json_extract(r.provider_properties, '$.CacheNodeType') AS "cache_node_type",
       json_extract(r.provider_properties, '$.ClusterMode') AS "cluster_mode",
       json_extract(r.provider_properties, '$.ConfigurationEndPoint') AS "configuration_end_point",
       json_extract(r.provider_properties, '$.DataTieringEnabled') AS "data_tiering_enabled",
       json_extract(r.provider_properties, '$.Durability') AS "durability",
       json_extract(r.provider_properties, '$.EffectiveDurability') AS "effective_durability",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.GlobalReplicationGroupId') AS "global_replication_group_id",
       json_extract(r.provider_properties, '$.IpDiscovery') AS "ip_discovery",
       json_extract(r.provider_properties, '$.LogDeliveryConfigurations') AS "log_delivery_configurations",
       json_extract(r.provider_properties, '$.MultiAZEnabled') AS "multi_az_enabled",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.NumCacheClusters') AS "num_cache_clusters",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PrimaryEndPoint') AS "primary_end_point",
       json_extract(r.provider_properties, '$.ReadEndPoint') AS "read_end_point",
       json_extract(r.provider_properties, '$.ReaderEndPoint') AS "reader_end_point",
       json_extract(r.provider_properties, '$.ReplicasPerNodeGroup') AS "replicas_per_node_group",
       json_extract(r.provider_properties, '$.ReplicationGroupDescription') AS "replication_group_description",
       json_extract(r.provider_properties, '$.ReplicationGroupId') AS "replication_group_id",
       json_extract(r.provider_properties, '$.SnapshotRetentionLimit') AS "snapshot_retention_limit",
       json_extract(r.provider_properties, '$.SnapshotWindow') AS "snapshot_window",
       json_extract(r.provider_properties, '$.SnapshottingClusterId') AS "snapshotting_cluster_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitEncryptionEnabled') AS "transit_encryption_enabled",
       json_extract(r.provider_properties, '$.TransitEncryptionMode') AS "transit_encryption_mode",
       json_extract(r.provider_properties, '$.UserGroupIds') AS "user_group_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::ReplicationGroup';

CREATE VIEW aws_cc_elasti_cache_subnet_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CacheSubnetGroupName') AS "cache_subnet_group_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::SubnetGroup';

CREATE VIEW aws_cc_elasti_cache_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CacheParameterGroupFamily') AS "cache_parameter_group_family",
       json_extract(r.provider_properties, '$.CacheParameterGroupName') AS "cache_parameter_group_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Properties') AS "configuration_properties",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::ParameterGroup';

CREATE VIEW aws_cc_elasti_cache_serverless_cache AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ARN') AS "provider_arn",
       json_extract(r.provider_properties, '$.CacheUsageLimits') AS "cache_usage_limits",
       json_extract(r.provider_properties, '$.ConnectionType') AS "connection_type",
       json_extract(r.provider_properties, '$.CreateTime') AS "create_time",
       json_extract(r.provider_properties, '$.DailySnapshotTime') AS "daily_snapshot_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.FullEngineVersion') AS "full_engine_version",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.MajorEngineVersion') AS "major_engine_version",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.ReaderEndpoint') AS "reader_endpoint",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.ServerlessCacheName') AS "serverless_cache_name",
       json_extract(r.provider_properties, '$.SnapshotRetentionLimit') AS "snapshot_retention_limit",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UserGroupId') AS "user_group_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElastiCache::ServerlessCache';

CREATE VIEW aws_cc_memory_db_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ACLName') AS "acl_name",
       json_extract(r.provider_properties, '$.ARN') AS "provider_arn",
       json_extract(r.provider_properties, '$.AutoMinorVersionUpgrade') AS "auto_minor_version_upgrade",
       json_extract(r.provider_properties, '$.ClusterEndpoint') AS "cluster_endpoint",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.DataTiering') AS "data_tiering",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Engine') AS "engine",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.IpDiscovery') AS "ip_discovery",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.MaintenanceWindow') AS "maintenance_window",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.NodeType') AS "node_type",
       json_extract(r.provider_properties, '$.NumReplicasPerShard') AS "num_replicas_per_shard",
       json_extract(r.provider_properties, '$.NumShards') AS "num_shards",
       json_extract(r.provider_properties, '$.ParameterGroupName') AS "parameter_group_name",
       json_extract(r.provider_properties, '$.ParameterGroupStatus') AS "parameter_group_status",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SnapshotRetentionLimit') AS "snapshot_retention_limit",
       json_extract(r.provider_properties, '$.SnapshotWindow') AS "snapshot_window",
       json_extract(r.provider_properties, '$.SnsTopicArn') AS "sns_topic_arn",
       json_extract(r.provider_properties, '$.SnsTopicStatus') AS "sns_topic_status",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.SubnetGroupName') AS "subnet_group_name",
       json_extract(r.provider_properties, '$.TLSEnabled') AS "tls_enabled",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MemoryDB::Cluster';

CREATE VIEW aws_cc_memory_db_subnet_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ARN') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.SubnetGroupName') AS "subnet_group_name",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.SupportedNetworkTypes') AS "supported_network_types",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MemoryDB::SubnetGroup';

CREATE VIEW aws_cc_memory_db_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ARN') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Family') AS "family",
       json_extract(r.provider_properties, '$.ParameterGroupName') AS "parameter_group_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MemoryDB::ParameterGroup';

CREATE VIEW aws_cc_memory_db_acl AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ACLName') AS "acl_name",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UserNames') AS "user_names"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MemoryDB::ACL';

CREATE VIEW aws_cc_neptune_db_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AssociatedRoles') AS "associated_roles",
       json_extract(r.provider_properties, '$.AvailabilityZones') AS "availability_zones",
       json_extract(r.provider_properties, '$.BackupRetentionPeriod') AS "backup_retention_period",
       json_extract(r.provider_properties, '$.ClusterResourceId') AS "cluster_resource_id",
       json_extract(r.provider_properties, '$.CopyTagsToSnapshot') AS "copy_tags_to_snapshot",
       json_extract(r.provider_properties, '$.DBClusterIdentifier') AS "db_cluster_identifier",
       json_extract(r.provider_properties, '$.DBClusterParameterGroupName') AS "db_cluster_parameter_group_name",
       json_extract(r.provider_properties, '$.DBPort') AS "db_port",
       json_extract(r.provider_properties, '$.DBSubnetGroupName') AS "db_subnet_group_name",
       json_extract(r.provider_properties, '$.DeletionProtection') AS "deletion_protection",
       json_extract(r.provider_properties, '$.EnableCloudwatchLogsExports') AS "enable_cloudwatch_logs_exports",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.GlobalClusterIdentifier') AS "global_cluster_identifier",
       json_extract(r.provider_properties, '$.IamAuthEnabled') AS "iam_auth_enabled",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.NetworkType') AS "network_type",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PreferredBackupWindow') AS "preferred_backup_window",
       json_extract(r.provider_properties, '$.PreferredMaintenanceWindow') AS "preferred_maintenance_window",
       json_extract(r.provider_properties, '$.ReadEndpoint') AS "read_endpoint",
       json_extract(r.provider_properties, '$.ServerlessScalingConfiguration') AS "serverless_scaling_configuration",
       json_extract(r.provider_properties, '$.StorageEncrypted') AS "storage_encrypted",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcSecurityGroupIds') AS "vpc_security_group_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Neptune::DBCluster';

CREATE VIEW aws_cc_neptune_db_instance AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutoMinorVersionUpgrade') AS "auto_minor_version_upgrade",
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.DBClusterIdentifier') AS "db_cluster_identifier",
       json_extract(r.provider_properties, '$.DBInstanceClass') AS "db_instance_class",
       json_extract(r.provider_properties, '$.DBInstanceIdentifier') AS "db_instance_identifier",
       json_extract(r.provider_properties, '$.DBParameterGroupName') AS "db_parameter_group_name",
       json_extract(r.provider_properties, '$.DBSubnetGroupName') AS "db_subnet_group_name",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PreferredMaintenanceWindow') AS "preferred_maintenance_window",
       json_extract(r.provider_properties, '$.PubliclyAccessible') AS "publicly_accessible",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Neptune::DBInstance';

CREATE VIEW aws_cc_neptune_db_subnet_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DBSubnetGroupDescription') AS "db_subnet_group_description",
       json_extract(r.provider_properties, '$.DBSubnetGroupName') AS "db_subnet_group_name",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Neptune::DBSubnetGroup';

CREATE VIEW aws_cc_doc_db_db_subnet_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DBSubnetGroupDescription') AS "db_subnet_group_description",
       json_extract(r.provider_properties, '$.DBSubnetGroupName') AS "db_subnet_group_name",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DocDB::DBSubnetGroup';

CREATE VIEW aws_cc_doc_db_db_cluster_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Family') AS "family",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DocDB::DBClusterParameterGroup';

CREATE VIEW aws_cc_redshift_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllowVersionUpgrade') AS "allow_version_upgrade",
       json_extract(r.provider_properties, '$.AquaConfigurationStatus') AS "aqua_configuration_status",
       json_extract(r.provider_properties, '$.AutomatedSnapshotRetentionPeriod') AS "automated_snapshot_retention_period",
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.AvailabilityZoneRelocation') AS "availability_zone_relocation",
       json_extract(r.provider_properties, '$.AvailabilityZoneRelocationStatus') AS "availability_zone_relocation_status",
       json_extract(r.provider_properties, '$.ClusterIdentifier') AS "cluster_identifier",
       json_extract(r.provider_properties, '$.ClusterNamespaceArn') AS "cluster_namespace_arn",
       json_extract(r.provider_properties, '$.ClusterParameterGroupName') AS "cluster_parameter_group_name",
       json_extract(r.provider_properties, '$.ClusterSecurityGroups') AS "cluster_security_groups",
       json_extract(r.provider_properties, '$.ClusterSubnetGroupName') AS "cluster_subnet_group_name",
       json_extract(r.provider_properties, '$.ClusterType') AS "cluster_type",
       json_extract(r.provider_properties, '$.ClusterVersion') AS "cluster_version",
       json_extract(r.provider_properties, '$.DBName') AS "db_name",
       json_extract(r.provider_properties, '$.DeferMaintenanceEndTime') AS "defer_maintenance_end_time",
       json_extract(r.provider_properties, '$.DeferMaintenanceIdentifier') AS "defer_maintenance_identifier",
       json_extract(r.provider_properties, '$.DeferMaintenanceStartTime') AS "defer_maintenance_start_time",
       json_extract(r.provider_properties, '$.DestinationRegion') AS "destination_region",
       json_extract(r.provider_properties, '$.ElasticIp') AS "elastic_ip",
       json_extract(r.provider_properties, '$.Encrypted') AS "encrypted",
       json_extract(r.provider_properties, '$.Endpoint') AS "endpoint",
       json_extract(r.provider_properties, '$.EnhancedVpcRouting') AS "enhanced_vpc_routing",
       json_extract(r.provider_properties, '$.HsmClientCertificateIdentifier') AS "hsm_client_certificate_identifier",
       json_extract(r.provider_properties, '$.HsmConfigurationIdentifier') AS "hsm_configuration_identifier",
       json_extract(r.provider_properties, '$.IamRoles') AS "iam_roles",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LoggingProperties') AS "logging_properties",
       json_extract(r.provider_properties, '$.MaintenanceTrackName') AS "maintenance_track_name",
       json_extract(r.provider_properties, '$.ManualSnapshotRetentionPeriod') AS "manual_snapshot_retention_period",
       json_extract(r.provider_properties, '$.MasterPasswordSecretArn') AS "master_password_secret_arn",
       json_extract(r.provider_properties, '$.MasterPasswordSecretKmsKeyId') AS "master_password_secret_kms_key_id",
       json_extract(r.provider_properties, '$.MasterUsername') AS "master_username",
       json_extract(r.provider_properties, '$.MultiAZ') AS "multi_az",
       json_extract(r.provider_properties, '$.NamespaceResourcePolicy') AS "namespace_resource_policy",
       json_extract(r.provider_properties, '$.NodeType') AS "node_type",
       json_extract(r.provider_properties, '$.NumberOfNodes') AS "number_of_nodes",
       json_extract(r.provider_properties, '$.OwnerAccount') AS "owner_account",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PreferredMaintenanceWindow') AS "preferred_maintenance_window",
       json_extract(r.provider_properties, '$.PubliclyAccessible') AS "publicly_accessible",
       json_extract(r.provider_properties, '$.ResourceAction') AS "resource_action",
       json_extract(r.provider_properties, '$.RevisionTarget') AS "revision_target",
       json_extract(r.provider_properties, '$.RotateEncryptionKey') AS "rotate_encryption_key",
       json_extract(r.provider_properties, '$.SnapshotClusterIdentifier') AS "snapshot_cluster_identifier",
       json_extract(r.provider_properties, '$.SnapshotCopyGrantName') AS "snapshot_copy_grant_name",
       json_extract(r.provider_properties, '$.SnapshotCopyManual') AS "snapshot_copy_manual",
       json_extract(r.provider_properties, '$.SnapshotCopyRetentionPeriod') AS "snapshot_copy_retention_period",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcSecurityGroupIds') AS "vpc_security_group_ids"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::Cluster';

CREATE VIEW aws_cc_redshift_cluster_parameter_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.ParameterGroupFamily') AS "parameter_group_family",
       json_extract(r.provider_properties, '$.ParameterGroupName') AS "parameter_group_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::ClusterParameterGroup';

CREATE VIEW aws_cc_redshift_cluster_subnet_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ClusterSubnetGroupName') AS "cluster_subnet_group_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::ClusterSubnetGroup';

CREATE VIEW aws_cc_redshift_endpoint_access AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Address') AS "address",
       json_extract(r.provider_properties, '$.ClusterIdentifier') AS "cluster_identifier",
       json_extract(r.provider_properties, '$.EndpointCreateTime') AS "endpoint_create_time",
       json_extract(r.provider_properties, '$.EndpointName') AS "endpoint_name",
       json_extract(r.provider_properties, '$.EndpointStatus') AS "endpoint_status",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.ResourceOwner') AS "resource_owner",
       json_extract(r.provider_properties, '$.SubnetGroupName') AS "subnet_group_name",
       json_extract(r.provider_properties, '$.VpcEndpoint') AS "vpc_endpoint",
       json_extract(r.provider_properties, '$.VpcSecurityGroupIds') AS "vpc_security_group_ids",
       json_extract(r.provider_properties, '$.VpcSecurityGroups') AS "vpc_security_groups"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::EndpointAccess';

CREATE VIEW aws_cc_redshift_snapshot AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AvailabilityZone') AS "availability_zone",
       json_extract(r.provider_properties, '$.ClusterCreateTime') AS "cluster_create_time",
       json_extract(r.provider_properties, '$.ClusterIdentifier') AS "cluster_identifier",
       json_extract(r.provider_properties, '$.ClusterVersion') AS "cluster_version",
       json_extract(r.provider_properties, '$.DBName') AS "db_name",
       json_extract(r.provider_properties, '$.Encrypted') AS "encrypted",
       json_extract(r.provider_properties, '$.EncryptedWithHSM') AS "encrypted_with_hsm",
       json_extract(r.provider_properties, '$.EngineFullVersion') AS "engine_full_version",
       json_extract(r.provider_properties, '$.EnhancedVpcRouting') AS "enhanced_vpc_routing",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.MaintenanceTrackName') AS "maintenance_track_name",
       json_extract(r.provider_properties, '$.ManualSnapshotRetentionPeriod') AS "manual_snapshot_retention_period",
       json_extract(r.provider_properties, '$.MasterUsername') AS "master_username",
       json_extract(r.provider_properties, '$.NodeType') AS "node_type",
       json_extract(r.provider_properties, '$.NumberOfNodes') AS "number_of_nodes",
       json_extract(r.provider_properties, '$.OwnerAccount') AS "owner_account",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.SnapshotArn') AS "snapshot_arn",
       json_extract(r.provider_properties, '$.SnapshotCreateTime') AS "snapshot_create_time",
       json_extract(r.provider_properties, '$.SnapshotIdentifier') AS "snapshot_identifier",
       json_extract(r.provider_properties, '$.SnapshotType') AS "snapshot_type",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Redshift::Snapshot';

CREATE VIEW aws_cc_redshift_serverless_namespace AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdminPasswordSecretKmsKeyId') AS "admin_password_secret_kms_key_id",
       json_extract(r.provider_properties, '$.AdminUsername') AS "admin_username",
       json_extract(r.provider_properties, '$.DbName') AS "db_name",
       json_extract(r.provider_properties, '$.DefaultIamRoleArn') AS "default_iam_role_arn",
       json_extract(r.provider_properties, '$.IamRoles') AS "iam_roles",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LogExports') AS "log_exports",
       json_extract(r.provider_properties, '$.Namespace') AS "namespace",
       json_extract(r.provider_properties, '$.NamespaceName') AS "namespace_name",
       json_extract(r.provider_properties, '$.NamespaceResourcePolicy') AS "namespace_resource_policy",
       json_extract(r.provider_properties, '$.SnapshotCopyConfigurations') AS "snapshot_copy_configurations",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RedshiftServerless::Namespace';

CREATE VIEW aws_cc_redshift_serverless_workgroup AS
SELECT r.*,
       json_extract(r.provider_properties, '$.BaseCapacity') AS "base_capacity",
       json_extract(r.provider_properties, '$.EnhancedVpcRouting') AS "enhanced_vpc_routing",
       json_extract(r.provider_properties, '$.MaxCapacity') AS "max_capacity",
       json_extract(r.provider_properties, '$.NamespaceName') AS "namespace_name",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PricePerformanceTarget') AS "price_performance_target",
       json_extract(r.provider_properties, '$.PubliclyAccessible') AS "publicly_accessible",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TrackName') AS "track_name",
       json_extract(r.provider_properties, '$.Workgroup') AS "workgroup",
       json_extract(r.provider_properties, '$.WorkgroupName') AS "workgroup_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RedshiftServerless::Workgroup';

CREATE VIEW aws_cc_open_search_serverless_collection AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CollectionEndpoint') AS "collection_endpoint",
       json_extract(r.provider_properties, '$.CollectionGroupName') AS "collection_group_name",
       json_extract(r.provider_properties, '$.DashboardEndpoint') AS "dashboard_endpoint",
       json_extract(r.provider_properties, '$.DeletionProtection') AS "deletion_protection",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FipsEndpoints') AS "fips_endpoints",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.KmsKeyArn') AS "kms_key_arn",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.StandbyReplicas') AS "standby_replicas",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.VectorOptions') AS "vector_options"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::OpenSearchServerless::Collection';

CREATE VIEW aws_cc_efs_file_system AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AvailabilityZoneName') AS "availability_zone_name",
       json_extract(r.provider_properties, '$.BackupPolicy') AS "backup_policy",
       json_extract(r.provider_properties, '$.Encrypted') AS "encrypted",
       json_extract(r.provider_properties, '$.FileSystemId') AS "file_system_id",
       json_extract(r.provider_properties, '$.FileSystemPolicy') AS "file_system_policy",
       json_extract(r.provider_properties, '$.FileSystemProtection') AS "file_system_protection",
       json_extract(r.provider_properties, '$.FileSystemTags') AS "file_system_tags",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LifecyclePolicies') AS "lifecycle_policies",
       json_extract(r.provider_properties, '$.PerformanceMode') AS "performance_mode",
       json_extract(r.provider_properties, '$.ProvisionedThroughputInMibps') AS "provisioned_throughput_in_mibps",
       json_extract(r.provider_properties, '$.ReplicationConfiguration') AS "replication_configuration",
       json_extract(r.provider_properties, '$.ThroughputMode') AS "throughput_mode"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EFS::FileSystem';

CREATE VIEW aws_cc_efs_access_point AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessPointId') AS "access_point_id",
       json_extract(r.provider_properties, '$.AccessPointTags') AS "access_point_tags",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ClientToken') AS "client_token",
       json_extract(r.provider_properties, '$.FileSystemId') AS "file_system_id",
       json_extract(r.provider_properties, '$.PosixUser') AS "posix_user",
       json_extract(r.provider_properties, '$.RootDirectory') AS "root_directory"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EFS::AccessPoint';

CREATE VIEW aws_cc_f_sx_volume AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OntapConfiguration') AS "ontap_configuration",
       json_extract(r.provider_properties, '$.OpenZFSConfiguration') AS "open_zfs_configuration",
       json_extract(r.provider_properties, '$.ResourceARN') AS "resource_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UUID') AS "uuid",
       json_extract(r.provider_properties, '$.VolumeId') AS "volume_id",
       json_extract(r.provider_properties, '$.VolumeType') AS "volume_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::FSx::Volume';

CREATE VIEW aws_cc_f_sx_storage_virtual_machine AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ActiveDirectoryConfiguration') AS "active_directory_configuration",
       json_extract(r.provider_properties, '$.FileSystemId') AS "file_system_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ResourceARN') AS "resource_arn",
       json_extract(r.provider_properties, '$.StorageVirtualMachineId') AS "storage_virtual_machine_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UUID') AS "uuid"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::FSx::StorageVirtualMachine';

CREATE VIEW aws_cc_f_sx_data_repository_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AssociationId') AS "association_id",
       json_extract(r.provider_properties, '$.BatchImportMetaDataOnCreate') AS "batch_import_meta_data_on_create",
       json_extract(r.provider_properties, '$.DataRepositoryPath') AS "data_repository_path",
       json_extract(r.provider_properties, '$.FileSystemId') AS "file_system_id",
       json_extract(r.provider_properties, '$.FileSystemPath') AS "file_system_path",
       json_extract(r.provider_properties, '$.ImportedFileChunkSize') AS "imported_file_chunk_size",
       json_extract(r.provider_properties, '$.ResourceARN') AS "resource_arn",
       json_extract(r.provider_properties, '$.S3') AS "s3",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::FSx::DataRepositoryAssociation';

CREATE VIEW aws_cc_s3_access_point AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Alias') AS "alias",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Bucket') AS "bucket",
       json_extract(r.provider_properties, '$.BucketAccountId') AS "bucket_account_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.NetworkOrigin') AS "network_origin",
       json_extract(r.provider_properties, '$.Policy') AS "policy",
       json_extract(r.provider_properties, '$.PublicAccessBlockConfiguration') AS "public_access_block_configuration",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcConfiguration') AS "vpc_configuration"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::S3::AccessPoint';

CREATE VIEW aws_cc_s3_access_grants_instance AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessGrantsInstanceArn') AS "access_grants_instance_arn",
       json_extract(r.provider_properties, '$.AccessGrantsInstanceId') AS "access_grants_instance_id",
       json_extract(r.provider_properties, '$.IdentityCenterArn') AS "identity_center_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::S3::AccessGrantsInstance';

CREATE VIEW aws_cc_s3_access_grants_location AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessGrantsLocationArn') AS "access_grants_location_arn",
       json_extract(r.provider_properties, '$.AccessGrantsLocationId') AS "access_grants_location_id",
       json_extract(r.provider_properties, '$.IamRoleArn') AS "iam_role_arn",
       json_extract(r.provider_properties, '$.LocationScope') AS "location_scope",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::S3::AccessGrantsLocation';

CREATE VIEW aws_cc_s3_express_directory_bucket AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AvailabilityZoneName') AS "availability_zone_name",
       json_extract(r.provider_properties, '$.BucketEncryption') AS "bucket_encryption",
       json_extract(r.provider_properties, '$.BucketName') AS "bucket_name",
       json_extract(r.provider_properties, '$.DataRedundancy') AS "data_redundancy",
       json_extract(r.provider_properties, '$.InventoryConfigurations') AS "inventory_configurations",
       json_extract(r.provider_properties, '$.LifecycleConfiguration') AS "lifecycle_configuration",
       json_extract(r.provider_properties, '$.LocationName') AS "location_name",
       json_extract(r.provider_properties, '$.MetricsConfigurations') AS "metrics_configurations",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::S3Express::DirectoryBucket';

CREATE VIEW aws_cc_s3_tables_table_bucket AS
SELECT r.*,
       json_extract(r.provider_properties, '$.EncryptionConfiguration') AS "encryption_configuration",
       json_extract(r.provider_properties, '$.LoggingConfiguration') AS "logging_configuration",
       json_extract(r.provider_properties, '$.MetricsConfiguration') AS "metrics_configuration",
       json_extract(r.provider_properties, '$.ReplicationConfiguration') AS "replication_configuration",
       json_extract(r.provider_properties, '$.StorageClassConfiguration') AS "storage_class_configuration",
       json_extract(r.provider_properties, '$.TableBucketARN') AS "table_bucket_arn",
       json_extract(r.provider_properties, '$.TableBucketName') AS "table_bucket_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.UnreferencedFileRemoval') AS "unreferenced_file_removal"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::S3Tables::TableBucket';

CREATE VIEW aws_cc_ecr_repository AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.EncryptionConfiguration') AS "encryption_configuration",
       json_extract(r.provider_properties, '$.ImageScanningConfiguration') AS "image_scanning_configuration",
       json_extract(r.provider_properties, '$.ImageTagMutability') AS "image_tag_mutability",
       json_extract(r.provider_properties, '$.ImageTagMutabilityExclusionFilters') AS "image_tag_mutability_exclusion_filters",
       json_extract(r.provider_properties, '$.LifecyclePolicy') AS "lifecycle_policy",
       json_extract(r.provider_properties, '$.RepositoryName') AS "repository_name",
       json_extract(r.provider_properties, '$.RepositoryPolicyText') AS "repository_policy_text",
       json_extract(r.provider_properties, '$.RepositoryUri') AS "repository_uri",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECR::Repository';

CREATE VIEW aws_cc_ecr_registry_scanning_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.RegistryId') AS "registry_id",
       json_extract(r.provider_properties, '$.Rules') AS "rules",
       json_extract(r.provider_properties, '$.ScanType') AS "scan_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECR::RegistryScanningConfiguration';

CREATE VIEW aws_cc_ecr_replication_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.RegistryId') AS "registry_id",
       json_extract(r.provider_properties, '$.ReplicationConfiguration') AS "replication_configuration"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ECR::ReplicationConfiguration';

CREATE VIEW aws_cc_auto_scaling_auto_scaling_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AutoScalingGroupARN') AS "auto_scaling_group_arn",
       json_extract(r.provider_properties, '$.AutoScalingGroupName') AS "auto_scaling_group_name",
       json_extract(r.provider_properties, '$.AvailabilityZoneDistribution') AS "availability_zone_distribution",
       json_extract(r.provider_properties, '$.AvailabilityZoneIds') AS "availability_zone_ids",
       json_extract(r.provider_properties, '$.AvailabilityZoneImpairmentPolicy') AS "availability_zone_impairment_policy",
       json_extract(r.provider_properties, '$.AvailabilityZones') AS "availability_zones",
       json_extract(r.provider_properties, '$.CapacityRebalance') AS "capacity_rebalance",
       json_extract(r.provider_properties, '$.CapacityReservationSpecification') AS "capacity_reservation_specification",
       json_extract(r.provider_properties, '$.Context') AS "context",
       json_extract(r.provider_properties, '$.Cooldown') AS "cooldown",
       json_extract(r.provider_properties, '$.DefaultInstanceWarmup') AS "default_instance_warmup",
       json_extract(r.provider_properties, '$.DeletionProtection') AS "deletion_protection",
       json_extract(r.provider_properties, '$.DesiredCapacity') AS "desired_capacity",
       json_extract(r.provider_properties, '$.DesiredCapacityType') AS "desired_capacity_type",
       json_extract(r.provider_properties, '$.HealthCheckGracePeriod') AS "health_check_grace_period",
       json_extract(r.provider_properties, '$.HealthCheckType') AS "health_check_type",
       json_extract(r.provider_properties, '$.InstanceLifecyclePolicy') AS "instance_lifecycle_policy",
       json_extract(r.provider_properties, '$.InstanceMaintenancePolicy') AS "instance_maintenance_policy",
       json_extract(r.provider_properties, '$.LaunchConfigurationName') AS "launch_configuration_name",
       json_extract(r.provider_properties, '$.LaunchTemplate') AS "launch_template",
       json_extract(r.provider_properties, '$.LifecycleHookSpecificationList') AS "lifecycle_hook_specification_list",
       json_extract(r.provider_properties, '$.LoadBalancerNames') AS "load_balancer_names",
       json_extract(r.provider_properties, '$.MaxInstanceLifetime') AS "max_instance_lifetime",
       json_extract(r.provider_properties, '$.MaxSize') AS "max_size",
       json_extract(r.provider_properties, '$.MetricsCollection') AS "metrics_collection",
       json_extract(r.provider_properties, '$.MinSize') AS "min_size",
       json_extract(r.provider_properties, '$.MixedInstancesPolicy') AS "mixed_instances_policy",
       json_extract(r.provider_properties, '$.NewInstancesProtectedFromScaleIn') AS "new_instances_protected_from_scale_in",
       json_extract(r.provider_properties, '$.NotificationConfiguration') AS "notification_configuration",
       json_extract(r.provider_properties, '$.NotificationConfigurations') AS "notification_configurations",
       json_extract(r.provider_properties, '$.PlacementGroup') AS "placement_group",
       json_extract(r.provider_properties, '$.ServiceLinkedRoleARN') AS "service_linked_role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetGroupARNs') AS "target_group_ar_ns",
       json_extract(r.provider_properties, '$.TerminationPolicies') AS "termination_policies",
       json_extract(r.provider_properties, '$.TrafficSources') AS "traffic_sources",
       json_extract(r.provider_properties, '$.VPCZoneIdentifier') AS "vpc_zone_identifier"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AutoScaling::AutoScalingGroup';

CREATE VIEW aws_cc_auto_scaling_launch_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AssociatePublicIpAddress') AS "associate_public_ip_address",
       json_extract(r.provider_properties, '$.BlockDeviceMappings') AS "block_device_mappings",
       json_extract(r.provider_properties, '$.EbsOptimized') AS "ebs_optimized",
       json_extract(r.provider_properties, '$.IamInstanceProfile') AS "iam_instance_profile",
       json_extract(r.provider_properties, '$.ImageId') AS "image_id",
       json_extract(r.provider_properties, '$.InstanceMonitoring') AS "instance_monitoring",
       json_extract(r.provider_properties, '$.InstanceType') AS "instance_type",
       json_extract(r.provider_properties, '$.KeyName') AS "key_name",
       json_extract(r.provider_properties, '$.LaunchConfigurationName') AS "launch_configuration_name",
       json_extract(r.provider_properties, '$.MetadataOptions') AS "metadata_options",
       json_extract(r.provider_properties, '$.PlacementTenancy') AS "placement_tenancy",
       json_extract(r.provider_properties, '$.SecurityGroups') AS "security_groups",
       json_extract(r.provider_properties, '$.SpotPrice') AS "spot_price"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AutoScaling::LaunchConfiguration';

CREATE VIEW aws_cc_application_auto_scaling_scalable_target AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.MaxCapacity') AS "max_capacity",
       json_extract(r.provider_properties, '$.MinCapacity') AS "min_capacity",
       json_extract(r.provider_properties, '$.ResourceId') AS "resource_id",
       json_extract(r.provider_properties, '$.ScalableDimension') AS "scalable_dimension",
       json_extract(r.provider_properties, '$.ScheduledActions') AS "scheduled_actions",
       json_extract(r.provider_properties, '$.ServiceNamespace') AS "service_namespace",
       json_extract(r.provider_properties, '$.SuspendedState') AS "suspended_state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApplicationAutoScaling::ScalableTarget';

CREATE VIEW aws_cc_batch_compute_environment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ComputeEnvironmentArn') AS "compute_environment_arn",
       json_extract(r.provider_properties, '$.ComputeEnvironmentName') AS "compute_environment_name",
       json_extract(r.provider_properties, '$.ComputeResources') AS "compute_resources",
       json_extract(r.provider_properties, '$.Context') AS "context",
       json_extract(r.provider_properties, '$.EcsSettings') AS "ecs_settings",
       json_extract(r.provider_properties, '$.EksConfiguration') AS "eks_configuration",
       json_extract(r.provider_properties, '$.ServiceRole') AS "service_role",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.UnmanagedvCpus') AS "unmanagedv_cpus"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Batch::ComputeEnvironment';

CREATE VIEW aws_cc_batch_job_definition AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConsumableResourceProperties') AS "consumable_resource_properties",
       json_extract(r.provider_properties, '$.ContainerProperties') AS "container_properties",
       json_extract(r.provider_properties, '$.EcsProperties') AS "ecs_properties",
       json_extract(r.provider_properties, '$.EksProperties') AS "eks_properties",
       json_extract(r.provider_properties, '$.JobDefinitionArn') AS "job_definition_arn",
       json_extract(r.provider_properties, '$.JobDefinitionName') AS "job_definition_name",
       json_extract(r.provider_properties, '$.NodeProperties') AS "node_properties",
       json_extract(r.provider_properties, '$.PlatformCapabilities') AS "platform_capabilities",
       json_extract(r.provider_properties, '$.PropagateTags') AS "propagate_tags",
       json_extract(r.provider_properties, '$.RetryStrategy') AS "retry_strategy",
       json_extract(r.provider_properties, '$.SchedulingPriority') AS "scheduling_priority",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Timeout') AS "timeout",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Batch::JobDefinition';

CREATE VIEW aws_cc_batch_job_queue AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ComputeEnvironmentOrder') AS "compute_environment_order",
       json_extract(r.provider_properties, '$.JobQueueArn') AS "job_queue_arn",
       json_extract(r.provider_properties, '$.JobQueueName') AS "job_queue_name",
       json_extract(r.provider_properties, '$.JobQueueType') AS "job_queue_type",
       json_extract(r.provider_properties, '$.JobStateTimeLimitActions') AS "job_state_time_limit_actions",
       json_extract(r.provider_properties, '$.Priority') AS "priority",
       json_extract(r.provider_properties, '$.SchedulingPolicyArn') AS "scheduling_policy_arn",
       json_extract(r.provider_properties, '$.ServiceEnvironmentOrder') AS "service_environment_order",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Batch::JobQueue';

CREATE VIEW aws_cc_elastic_beanstalk_application AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationName') AS "application_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.ResourceLifecycleConfig') AS "resource_lifecycle_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElasticBeanstalk::Application';

CREATE VIEW aws_cc_elastic_beanstalk_environment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationName') AS "application_name",
       json_extract(r.provider_properties, '$.CNAMEPrefix') AS "cname_prefix",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EndpointURL') AS "endpoint_url",
       json_extract(r.provider_properties, '$.EnvironmentName') AS "environment_name",
       json_extract(r.provider_properties, '$.OperationsRole') AS "operations_role",
       json_extract(r.provider_properties, '$.PlatformArn') AS "platform_arn",
       json_extract(r.provider_properties, '$.SolutionStackName') AS "solution_stack_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Tier') AS "tier",
       json_extract(r.provider_properties, '$.VersionLabel') AS "version_label"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElasticBeanstalk::Environment';

CREATE VIEW aws_cc_api_gateway_rest_api AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApiKeySourceType') AS "api_key_source_type",
       json_extract(r.provider_properties, '$.BinaryMediaTypes') AS "binary_media_types",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DisableExecuteApiEndpoint') AS "disable_execute_api_endpoint",
       json_extract(r.provider_properties, '$.EndpointAccessMode') AS "endpoint_access_mode",
       json_extract(r.provider_properties, '$.EndpointConfiguration') AS "endpoint_configuration",
       json_extract(r.provider_properties, '$.MinimumCompressionSize') AS "minimum_compression_size",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Policy') AS "policy",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id",
       json_extract(r.provider_properties, '$.RootResourceId') AS "root_resource_id",
       json_extract(r.provider_properties, '$.SecurityPolicy') AS "security_policy",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::RestApi';

CREATE VIEW aws_cc_api_gateway_domain_name AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CertificateArn') AS "certificate_arn",
       json_extract(r.provider_properties, '$.DistributionDomainName') AS "distribution_domain_name",
       json_extract(r.provider_properties, '$.DistributionHostedZoneId') AS "distribution_hosted_zone_id",
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.DomainNameArn') AS "domain_name_arn",
       json_extract(r.provider_properties, '$.EndpointAccessMode') AS "endpoint_access_mode",
       json_extract(r.provider_properties, '$.EndpointConfiguration') AS "endpoint_configuration",
       json_extract(r.provider_properties, '$.MutualTlsAuthentication') AS "mutual_tls_authentication",
       json_extract(r.provider_properties, '$.OwnershipVerificationCertificateArn') AS "ownership_verification_certificate_arn",
       json_extract(r.provider_properties, '$.RegionalCertificateArn') AS "regional_certificate_arn",
       json_extract(r.provider_properties, '$.RegionalDomainName') AS "regional_domain_name",
       json_extract(r.provider_properties, '$.RegionalHostedZoneId') AS "regional_hosted_zone_id",
       json_extract(r.provider_properties, '$.RoutingMode') AS "routing_mode",
       json_extract(r.provider_properties, '$.SecurityPolicy') AS "security_policy",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::DomainName';

CREATE VIEW aws_cc_api_gateway_vpc_link AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetArns') AS "target_arns",
       json_extract(r.provider_properties, '$.VpcLinkId') AS "vpc_link_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::VpcLink';

CREATE VIEW aws_cc_api_gateway_usage_plan AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApiStages') AS "api_stages",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Quota') AS "quota",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Throttle') AS "throttle",
       json_extract(r.provider_properties, '$.UsagePlanName') AS "usage_plan_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::UsagePlan';

CREATE VIEW aws_cc_api_gateway_v2_api AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApiEndpoint') AS "api_endpoint",
       json_extract(r.provider_properties, '$.ApiId') AS "api_id",
       json_extract(r.provider_properties, '$.ApiKeySelectionExpression') AS "api_key_selection_expression",
       json_extract(r.provider_properties, '$.CorsConfiguration') AS "cors_configuration",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DisableExecuteApiEndpoint') AS "disable_execute_api_endpoint",
       json_extract(r.provider_properties, '$.ExecuteApiArn') AS "execute_api_arn",
       json_extract(r.provider_properties, '$.IpAddressType') AS "ip_address_type",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ProtocolType') AS "protocol_type",
       json_extract(r.provider_properties, '$.RouteSelectionExpression') AS "route_selection_expression",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGatewayV2::Api';

CREATE VIEW aws_cc_api_gateway_v2_domain_name AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.DomainNameArn') AS "domain_name_arn",
       json_extract(r.provider_properties, '$.DomainNameConfigurations') AS "domain_name_configurations",
       json_extract(r.provider_properties, '$.MutualTlsAuthentication') AS "mutual_tls_authentication",
       json_extract(r.provider_properties, '$.RegionalDomainName') AS "regional_domain_name",
       json_extract(r.provider_properties, '$.RegionalHostedZoneId') AS "regional_hosted_zone_id",
       json_extract(r.provider_properties, '$.RoutingMode') AS "routing_mode",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGatewayV2::DomainName';

CREATE VIEW aws_cc_api_gateway_v2_vpc_link AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcLinkId') AS "vpc_link_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGatewayV2::VpcLink';

CREATE VIEW aws_cc_kinesis_stream AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DesiredShardLevelMetrics') AS "desired_shard_level_metrics",
       json_extract(r.provider_properties, '$.MaxRecordSizeInKiB') AS "max_record_size_in_ki_b",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RecordDistributionStrategy') AS "record_distribution_strategy",
       json_extract(r.provider_properties, '$.RetentionPeriodHours') AS "retention_period_hours",
       json_extract(r.provider_properties, '$.ShardCount') AS "shard_count",
       json_extract(r.provider_properties, '$.StreamEncryption') AS "stream_encryption",
       json_extract(r.provider_properties, '$.StreamModeDetails') AS "stream_mode_details",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.WarmThroughputObject') AS "warm_throughput_object"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Kinesis::Stream';

CREATE VIEW aws_cc_kinesis_stream_consumer AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConsumerARN') AS "consumer_arn",
       json_extract(r.provider_properties, '$.ConsumerCreationTimestamp') AS "consumer_creation_timestamp",
       json_extract(r.provider_properties, '$.ConsumerName') AS "consumer_name",
       json_extract(r.provider_properties, '$.ConsumerStatus') AS "consumer_status",
       json_extract(r.provider_properties, '$.StreamARN') AS "stream_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Kinesis::StreamConsumer';

CREATE VIEW aws_cc_kinesis_firehose_delivery_stream AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DeliveryStreamEncryptionConfigurationInput') AS "delivery_stream_encryption_configuration_input",
       json_extract(r.provider_properties, '$.DeliveryStreamName') AS "delivery_stream_name",
       json_extract(r.provider_properties, '$.DeliveryStreamType') AS "delivery_stream_type",
       json_extract(r.provider_properties, '$.DirectPutSourceConfiguration') AS "direct_put_source_configuration",
       json_extract(r.provider_properties, '$.ExtendedS3DestinationConfiguration') AS "extended_s3_destination_configuration",
       json_extract(r.provider_properties, '$.HttpEndpointDestinationConfiguration') AS "http_endpoint_destination_configuration",
       json_extract(r.provider_properties, '$.RedshiftDestinationConfiguration') AS "redshift_destination_configuration",
       json_extract(r.provider_properties, '$.SplunkDestinationConfiguration') AS "splunk_destination_configuration",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::KinesisFirehose::DeliveryStream';

CREATE VIEW aws_cc_kinesis_analytics_v2_application AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationConfiguration') AS "application_configuration",
       json_extract(r.provider_properties, '$.ApplicationDescription') AS "application_description",
       json_extract(r.provider_properties, '$.ApplicationMaintenanceConfiguration') AS "application_maintenance_configuration",
       json_extract(r.provider_properties, '$.ApplicationMode') AS "application_mode",
       json_extract(r.provider_properties, '$.ApplicationName') AS "application_name",
       json_extract(r.provider_properties, '$.RuntimeEnvironment') AS "runtime_environment",
       json_extract(r.provider_properties, '$.ServiceExecutionRole') AS "service_execution_role",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::KinesisAnalyticsV2::Application';

CREATE VIEW aws_cc_sqs_queue AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ContentBasedDeduplication') AS "content_based_deduplication",
       json_extract(r.provider_properties, '$.DeduplicationScope') AS "deduplication_scope",
       json_extract(r.provider_properties, '$.DelaySeconds') AS "delay_seconds",
       json_extract(r.provider_properties, '$.FifoQueue') AS "fifo_queue",
       json_extract(r.provider_properties, '$.FifoThroughputLimit') AS "fifo_throughput_limit",
       json_extract(r.provider_properties, '$.KmsDataKeyReusePeriodSeconds') AS "kms_data_key_reuse_period_seconds",
       json_extract(r.provider_properties, '$.KmsMasterKeyId') AS "kms_master_key_id",
       json_extract(r.provider_properties, '$.MaximumMessageSize') AS "maximum_message_size",
       json_extract(r.provider_properties, '$.MessageRetentionPeriod') AS "message_retention_period",
       json_extract(r.provider_properties, '$.QueueName') AS "queue_name",
       json_extract(r.provider_properties, '$.QueueUrl') AS "queue_url",
       json_extract(r.provider_properties, '$.ReceiveMessageWaitTimeSeconds') AS "receive_message_wait_time_seconds",
       json_extract(r.provider_properties, '$.RedriveAllowPolicy') AS "redrive_allow_policy",
       json_extract(r.provider_properties, '$.RedrivePolicy') AS "redrive_policy",
       json_extract(r.provider_properties, '$.SqsManagedSseEnabled') AS "sqs_managed_sse_enabled",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VisibilityTimeout') AS "visibility_timeout"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SQS::Queue';

CREATE VIEW aws_cc_sns_topic AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ArchivePolicy') AS "archive_policy",
       json_extract(r.provider_properties, '$.ContentBasedDeduplication') AS "content_based_deduplication",
       json_extract(r.provider_properties, '$.DataProtectionPolicy') AS "data_protection_policy",
       json_extract(r.provider_properties, '$.DeliveryStatusLogging') AS "delivery_status_logging",
       json_extract(r.provider_properties, '$.DisplayName') AS "display_name",
       json_extract(r.provider_properties, '$.FifoThroughputScope') AS "fifo_throughput_scope",
       json_extract(r.provider_properties, '$.FifoTopic') AS "fifo_topic",
       json_extract(r.provider_properties, '$.KmsMasterKeyId') AS "kms_master_key_id",
       json_extract(r.provider_properties, '$.MaximumMessageSize') AS "maximum_message_size",
       json_extract(r.provider_properties, '$.SignatureVersion') AS "signature_version",
       json_extract(r.provider_properties, '$.Subscription') AS "subscription",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TopicArn') AS "topic_arn",
       json_extract(r.provider_properties, '$.TopicName') AS "topic_name",
       json_extract(r.provider_properties, '$.TracingConfig') AS "tracing_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SNS::Topic';

CREATE VIEW aws_cc_sns_subscription AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DeliveryPolicy') AS "delivery_policy",
       json_extract(r.provider_properties, '$.FilterPolicy') AS "filter_policy",
       json_extract(r.provider_properties, '$.FilterPolicyScope') AS "filter_policy_scope",
       json_extract(r.provider_properties, '$.Protocol') AS "protocol",
       json_extract(r.provider_properties, '$.RawMessageDelivery') AS "raw_message_delivery",
       json_extract(r.provider_properties, '$.RedrivePolicy') AS "redrive_policy",
       json_extract(r.provider_properties, '$.ReplayPolicy') AS "replay_policy",
       json_extract(r.provider_properties, '$.SubscriptionRoleArn') AS "subscription_role_arn",
       json_extract(r.provider_properties, '$.TopicArn') AS "topic_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SNS::Subscription';

CREATE VIEW aws_cc_events_event_bus AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DeadLetterConfig') AS "dead_letter_config",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.KmsKeyIdentifier') AS "kms_key_identifier",
       json_extract(r.provider_properties, '$.LogConfig') AS "log_config",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Policy') AS "policy",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Events::EventBus';

CREATE VIEW aws_cc_events_rule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EventBusName') AS "event_bus_name",
       json_extract(r.provider_properties, '$.EventPattern') AS "event_pattern",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.RuleName') AS "rule_name",
       json_extract(r.provider_properties, '$.ScheduleExpression') AS "schedule_expression",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Targets') AS "targets"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Events::Rule';

CREATE VIEW aws_cc_events_archive AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ArchiveName') AS "archive_name",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EventPattern') AS "event_pattern",
       json_extract(r.provider_properties, '$.KmsKeyIdentifier') AS "kms_key_identifier",
       json_extract(r.provider_properties, '$.RetentionDays') AS "retention_days",
       json_extract(r.provider_properties, '$.SourceArn') AS "source_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Events::Archive';

CREATE VIEW aws_cc_events_api_destination AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ArnForPolicy') AS "arn_for_policy",
       json_extract(r.provider_properties, '$.ConnectionArn') AS "connection_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.HttpMethod') AS "http_method",
       json_extract(r.provider_properties, '$.InvocationRateLimitPerSecond') AS "invocation_rate_limit_per_second",
       json_extract(r.provider_properties, '$.Name') AS "provider_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Events::ApiDestination';

CREATE VIEW aws_cc_events_connection AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ArnForPolicy') AS "arn_for_policy",
       json_extract(r.provider_properties, '$.AuthorizationType') AS "authorization_type",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.InvocationConnectivityParameters') AS "invocation_connectivity_parameters",
       json_extract(r.provider_properties, '$.KmsKeyIdentifier') AS "kms_key_identifier",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.SecretArn') AS "secret_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Events::Connection';

CREATE VIEW aws_cc_event_schemas_registry AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.RegistryArn') AS "registry_arn",
       json_extract(r.provider_properties, '$.RegistryName') AS "registry_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EventSchemas::Registry';

CREATE VIEW aws_cc_event_schemas_discoverer AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CrossAccount') AS "cross_account",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DiscovererArn') AS "discoverer_arn",
       json_extract(r.provider_properties, '$.DiscovererId') AS "discoverer_id",
       json_extract(r.provider_properties, '$.SourceArn') AS "source_arn",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EventSchemas::Discoverer';

CREATE VIEW aws_cc_step_functions_state_machine AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.EncryptionConfiguration') AS "encryption_configuration",
       json_extract(r.provider_properties, '$.LoggingConfiguration') AS "logging_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.StateMachineName') AS "state_machine_name",
       json_extract(r.provider_properties, '$.StateMachineRevisionId') AS "state_machine_revision_id",
       json_extract(r.provider_properties, '$.StateMachineType') AS "state_machine_type",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TracingConfiguration') AS "tracing_configuration"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::StepFunctions::StateMachine';

CREATE VIEW aws_cc_step_functions_activity AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.EncryptionConfiguration') AS "encryption_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::StepFunctions::Activity';

CREATE VIEW aws_cc_logs_log_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.BearerTokenAuthenticationEnabled') AS "bearer_token_authentication_enabled",
       json_extract(r.provider_properties, '$.DataProtectionPolicy') AS "data_protection_policy",
       json_extract(r.provider_properties, '$.DeletionProtectionEnabled') AS "deletion_protection_enabled",
       json_extract(r.provider_properties, '$.FieldIndexPolicies') AS "field_index_policies",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.LogGroupClass') AS "log_group_class",
       json_extract(r.provider_properties, '$.LogGroupName') AS "log_group_name",
       json_extract(r.provider_properties, '$.ResourcePolicyDocument') AS "resource_policy_document",
       json_extract(r.provider_properties, '$.RetentionInDays') AS "retention_in_days",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::LogGroup';

CREATE VIEW aws_cc_logs_destination AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DestinationName') AS "destination_name",
       json_extract(r.provider_properties, '$.DestinationPolicy') AS "destination_policy",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetArn') AS "target_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::Destination';

CREATE VIEW aws_cc_logs_resource_policy AS
SELECT r.*,
       json_extract(r.provider_properties, '$.PolicyDocument') AS "policy_document",
       json_extract(r.provider_properties, '$.PolicyName') AS "policy_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::ResourcePolicy';

CREATE VIEW aws_cc_logs_query_definition AS
SELECT r.*,
       json_extract(r.provider_properties, '$.LogGroupNames') AS "log_group_names",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Parameters') AS "parameters",
       json_extract(r.provider_properties, '$.QueryDefinitionId') AS "query_definition_id",
       json_extract(r.provider_properties, '$.QueryLanguage') AS "query_language"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::QueryDefinition';

CREATE VIEW aws_cc_logs_metric_filter AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplyOnTransformedLogs') AS "apply_on_transformed_logs",
       json_extract(r.provider_properties, '$.EmitSystemFieldDimensions') AS "emit_system_field_dimensions",
       json_extract(r.provider_properties, '$.FieldSelectionCriteria') AS "field_selection_criteria",
       json_extract(r.provider_properties, '$.FilterName') AS "filter_name",
       json_extract(r.provider_properties, '$.FilterPattern') AS "filter_pattern",
       json_extract(r.provider_properties, '$.LogGroupName') AS "log_group_name",
       json_extract(r.provider_properties, '$.MetricTransformations') AS "metric_transformations"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::MetricFilter';

CREATE VIEW aws_cc_cloud_watch_alarm AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ActionConfiguration') AS "action_configuration",
       json_extract(r.provider_properties, '$.ActionsEnabled') AS "actions_enabled",
       json_extract(r.provider_properties, '$.AlarmActions') AS "alarm_actions",
       json_extract(r.provider_properties, '$.AlarmDescription') AS "alarm_description",
       json_extract(r.provider_properties, '$.AlarmName') AS "alarm_name",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ComparisonOperator') AS "comparison_operator",
       json_extract(r.provider_properties, '$.DatapointsToAlarm') AS "datapoints_to_alarm",
       json_extract(r.provider_properties, '$.Dimensions') AS "dimensions",
       json_extract(r.provider_properties, '$.EvaluateLowSampleCountPercentile') AS "evaluate_low_sample_count_percentile",
       json_extract(r.provider_properties, '$.EvaluationCriteria') AS "evaluation_criteria",
       json_extract(r.provider_properties, '$.EvaluationInterval') AS "evaluation_interval",
       json_extract(r.provider_properties, '$.EvaluationPeriods') AS "evaluation_periods",
       json_extract(r.provider_properties, '$.EvaluationWindow') AS "evaluation_window",
       json_extract(r.provider_properties, '$.ExtendedStatistic') AS "extended_statistic",
       json_extract(r.provider_properties, '$.InsufficientDataActions') AS "insufficient_data_actions",
       json_extract(r.provider_properties, '$.MetricName') AS "metric_name",
       json_extract(r.provider_properties, '$.Metrics') AS "metrics",
       json_extract(r.provider_properties, '$.Namespace') AS "namespace",
       json_extract(r.provider_properties, '$.OKActions') AS "ok_actions",
       json_extract(r.provider_properties, '$.Period') AS "period",
       json_extract(r.provider_properties, '$.Statistic') AS "statistic",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Threshold') AS "threshold",
       json_extract(r.provider_properties, '$.ThresholdMetricId') AS "threshold_metric_id",
       json_extract(r.provider_properties, '$.TreatMissingData') AS "treat_missing_data",
       json_extract(r.provider_properties, '$.Unit') AS "unit",
       json_extract(r.provider_properties, '$.WarmUpConfiguration') AS "warm_up_configuration"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CloudWatch::Alarm';

CREATE VIEW aws_cc_cloud_watch_composite_alarm AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ActionConfiguration') AS "action_configuration",
       json_extract(r.provider_properties, '$.ActionsEnabled') AS "actions_enabled",
       json_extract(r.provider_properties, '$.ActionsSuppressor') AS "actions_suppressor",
       json_extract(r.provider_properties, '$.ActionsSuppressorExtensionPeriod') AS "actions_suppressor_extension_period",
       json_extract(r.provider_properties, '$.ActionsSuppressorWaitPeriod') AS "actions_suppressor_wait_period",
       json_extract(r.provider_properties, '$.AlarmActions') AS "alarm_actions",
       json_extract(r.provider_properties, '$.AlarmDescription') AS "alarm_description",
       json_extract(r.provider_properties, '$.AlarmName') AS "alarm_name",
       json_extract(r.provider_properties, '$.AlarmRule') AS "alarm_rule",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.InsufficientDataActions') AS "insufficient_data_actions",
       json_extract(r.provider_properties, '$.OKActions') AS "ok_actions",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CloudWatch::CompositeAlarm';

CREATE VIEW aws_cc_cloud_watch_insight_rule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplyOnTransformedLogs') AS "apply_on_transformed_logs",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.RuleBody') AS "rule_body",
       json_extract(r.provider_properties, '$.RuleName') AS "rule_name",
       json_extract(r.provider_properties, '$.RuleState') AS "rule_state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CloudWatch::InsightRule';

CREATE VIEW aws_cc_cloud_watch_metric_stream AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationDate') AS "creation_date",
       json_extract(r.provider_properties, '$.ExcludeFilters') AS "exclude_filters",
       json_extract(r.provider_properties, '$.FirehoseArn') AS "firehose_arn",
       json_extract(r.provider_properties, '$.IncludeFilters') AS "include_filters",
       json_extract(r.provider_properties, '$.IncludeLinkedAccountsMetrics') AS "include_linked_accounts_metrics",
       json_extract(r.provider_properties, '$.LastUpdateDate') AS "last_update_date",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OutputFormat') AS "output_format",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.StatisticsConfigurations') AS "statistics_configurations",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CloudWatch::MetricStream';

CREATE VIEW aws_cc_cloud_trail_event_data_store AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AdvancedEventSelectors') AS "advanced_event_selectors",
       json_extract(r.provider_properties, '$.BillingMode') AS "billing_mode",
       json_extract(r.provider_properties, '$.ContextKeySelectors') AS "context_key_selectors",
       json_extract(r.provider_properties, '$.CreatedTimestamp') AS "created_timestamp",
       json_extract(r.provider_properties, '$.EventDataStoreArn') AS "event_data_store_arn",
       json_extract(r.provider_properties, '$.FederationEnabled') AS "federation_enabled",
       json_extract(r.provider_properties, '$.FederationRoleArn') AS "federation_role_arn",
       json_extract(r.provider_properties, '$.IngestionEnabled') AS "ingestion_enabled",
       json_extract(r.provider_properties, '$.InsightSelectors') AS "insight_selectors",
       json_extract(r.provider_properties, '$.InsightsDestination') AS "insights_destination",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.MaxEventSize') AS "max_event_size",
       json_extract(r.provider_properties, '$.MultiRegionEnabled') AS "multi_region_enabled",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OrganizationEnabled') AS "organization_enabled",
       json_extract(r.provider_properties, '$.RetentionPeriod') AS "retention_period",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TerminationProtectionEnabled') AS "termination_protection_enabled",
       json_extract(r.provider_properties, '$.UpdatedTimestamp') AS "updated_timestamp"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CloudTrail::EventDataStore';

CREATE VIEW aws_cc_cloud_trail_channel AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ChannelArn') AS "channel_arn",
       json_extract(r.provider_properties, '$.Destinations') AS "destinations",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Source') AS "provider_source",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CloudTrail::Channel';

CREATE VIEW aws_cc_kms_key AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EnableKeyRotation') AS "enable_key_rotation",
       json_extract(r.provider_properties, '$.Enabled') AS "enabled",
       json_extract(r.provider_properties, '$.KeyId') AS "key_id",
       json_extract(r.provider_properties, '$.KeyPolicy') AS "key_policy",
       json_extract(r.provider_properties, '$.KeySpec') AS "key_spec",
       json_extract(r.provider_properties, '$.KeyUsage') AS "key_usage",
       json_extract(r.provider_properties, '$.MultiRegion') AS "multi_region",
       json_extract(r.provider_properties, '$.Origin') AS "origin",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::KMS::Key';

CREATE VIEW aws_cc_kms_alias AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AliasName') AS "alias_name",
       json_extract(r.provider_properties, '$.TargetKeyId') AS "target_key_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::KMS::Alias';

CREATE VIEW aws_cc_secrets_manager_secret AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ReplicaRegions') AS "replica_regions",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SecretsManager::Secret';

CREATE VIEW aws_cc_ssm_parameter AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DataType') AS "data_type",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SSM::Parameter';

CREATE VIEW aws_cc_ssm_document AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DocumentFormat') AS "document_format",
       json_extract(r.provider_properties, '$.DocumentType') AS "document_type",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Requires') AS "requires",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetType') AS "target_type",
       json_extract(r.provider_properties, '$.VersionName') AS "version_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SSM::Document';

CREATE VIEW aws_cc_ssm_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplyOnlyAtCronInterval') AS "apply_only_at_cron_interval",
       json_extract(r.provider_properties, '$.AssociationDispatchAssumeRole') AS "association_dispatch_assume_role",
       json_extract(r.provider_properties, '$.AssociationId') AS "association_id",
       json_extract(r.provider_properties, '$.AssociationName') AS "association_name",
       json_extract(r.provider_properties, '$.AutomationTargetParameterName') AS "automation_target_parameter_name",
       json_extract(r.provider_properties, '$.CalendarNames') AS "calendar_names",
       json_extract(r.provider_properties, '$.ComplianceSeverity') AS "compliance_severity",
       json_extract(r.provider_properties, '$.DocumentVersion') AS "document_version",
       json_extract(r.provider_properties, '$.InstanceId') AS "instance_id",
       json_extract(r.provider_properties, '$.MaxConcurrency') AS "max_concurrency",
       json_extract(r.provider_properties, '$.MaxErrors') AS "max_errors",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OutputLocation') AS "output_location",
       json_extract(r.provider_properties, '$.ScheduleExpression') AS "schedule_expression",
       json_extract(r.provider_properties, '$.ScheduleOffset') AS "schedule_offset",
       json_extract(r.provider_properties, '$.SyncCompliance') AS "sync_compliance",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Targets') AS "targets"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SSM::Association';

CREATE VIEW aws_cc_ssm_maintenance_window AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllowUnassociatedTargets') AS "allow_unassociated_targets",
       json_extract(r.provider_properties, '$.Cutoff') AS "cutoff",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Duration') AS "duration",
       json_extract(r.provider_properties, '$.EndDate') AS "end_date",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Schedule') AS "schedule",
       json_extract(r.provider_properties, '$.ScheduleOffset') AS "schedule_offset",
       json_extract(r.provider_properties, '$.ScheduleTimezone') AS "schedule_timezone",
       json_extract(r.provider_properties, '$.StartDate') AS "start_date",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.WindowId') AS "window_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SSM::MaintenanceWindow';

CREATE VIEW aws_cc_backup_backup_vault AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessPolicy') AS "access_policy",
       json_extract(r.provider_properties, '$.BackupVaultArn') AS "backup_vault_arn",
       json_extract(r.provider_properties, '$.BackupVaultName') AS "backup_vault_name",
       json_extract(r.provider_properties, '$.BackupVaultTags') AS "backup_vault_tags",
       json_extract(r.provider_properties, '$.EncryptionKeyArn') AS "encryption_key_arn",
       json_extract(r.provider_properties, '$.LockConfiguration') AS "lock_configuration",
       json_extract(r.provider_properties, '$.Notifications') AS "notifications"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Backup::BackupVault';

CREATE VIEW aws_cc_backup_backup_plan AS
SELECT r.*,
       json_extract(r.provider_properties, '$.BackupPlan') AS "backup_plan",
       json_extract(r.provider_properties, '$.BackupPlanArn') AS "backup_plan_arn",
       json_extract(r.provider_properties, '$.BackupPlanId') AS "backup_plan_id",
       json_extract(r.provider_properties, '$.BackupPlanTags') AS "backup_plan_tags",
       json_extract(r.provider_properties, '$.VersionId') AS "version_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Backup::BackupPlan';

CREATE VIEW aws_cc_backup_restore_testing_plan AS
SELECT r.*,
       json_extract(r.provider_properties, '$.RecoveryPointSelection') AS "recovery_point_selection",
       json_extract(r.provider_properties, '$.RestoreTestingPlanArn') AS "restore_testing_plan_arn",
       json_extract(r.provider_properties, '$.RestoreTestingPlanName') AS "restore_testing_plan_name",
       json_extract(r.provider_properties, '$.ScheduleExpression') AS "schedule_expression",
       json_extract(r.provider_properties, '$.ScheduleExpressionTimezone') AS "schedule_expression_timezone",
       json_extract(r.provider_properties, '$.StartWindowHours') AS "start_window_hours",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Backup::RestoreTestingPlan';

CREATE VIEW aws_cc_dms_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CertificateArn') AS "certificate_arn",
       json_extract(r.provider_properties, '$.DatabaseName') AS "database_name",
       json_extract(r.provider_properties, '$.DocDbSettings') AS "doc_db_settings",
       json_extract(r.provider_properties, '$.DynamoDbSettings') AS "dynamo_db_settings",
       json_extract(r.provider_properties, '$.ElasticsearchSettings') AS "elasticsearch_settings",
       json_extract(r.provider_properties, '$.EndpointArn') AS "endpoint_arn",
       json_extract(r.provider_properties, '$.EndpointIdentifier') AS "endpoint_identifier",
       json_extract(r.provider_properties, '$.EndpointType') AS "endpoint_type",
       json_extract(r.provider_properties, '$.EngineName') AS "engine_name",
       json_extract(r.provider_properties, '$.ExternalId') AS "external_id",
       json_extract(r.provider_properties, '$.GcpMySQLSettings') AS "gcp_my_sql_settings",
       json_extract(r.provider_properties, '$.IbmDb2Settings') AS "ibm_db2_settings",
       json_extract(r.provider_properties, '$.KafkaSettings') AS "kafka_settings",
       json_extract(r.provider_properties, '$.KinesisSettings') AS "kinesis_settings",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.MicrosoftSqlServerSettings') AS "microsoft_sql_server_settings",
       json_extract(r.provider_properties, '$.MongoDbSettings') AS "mongo_db_settings",
       json_extract(r.provider_properties, '$.MySqlSettings') AS "my_sql_settings",
       json_extract(r.provider_properties, '$.NeptuneSettings') AS "neptune_settings",
       json_extract(r.provider_properties, '$.OracleSettings') AS "oracle_settings",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.PostgreSqlSettings') AS "postgre_sql_settings",
       json_extract(r.provider_properties, '$.RedisSettings') AS "redis_settings",
       json_extract(r.provider_properties, '$.RedshiftSettings') AS "redshift_settings",
       json_extract(r.provider_properties, '$.S3Settings') AS "s3_settings",
       json_extract(r.provider_properties, '$.ServerName') AS "server_name",
       json_extract(r.provider_properties, '$.SslMode') AS "ssl_mode",
       json_extract(r.provider_properties, '$.SybaseSettings') AS "sybase_settings",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Username') AS "username"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::Endpoint';

CREATE VIEW aws_cc_dms_replication_subnet_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ReplicationSubnetGroupDescription') AS "replication_subnet_group_description",
       json_extract(r.provider_properties, '$.ReplicationSubnetGroupIdentifier') AS "replication_subnet_group_identifier",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::ReplicationSubnetGroup';

CREATE VIEW aws_cc_dms_replication_task AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CdcStartPosition') AS "cdc_start_position",
       json_extract(r.provider_properties, '$.CdcStopPosition') AS "cdc_stop_position",
       json_extract(r.provider_properties, '$.MigrationType') AS "migration_type",
       json_extract(r.provider_properties, '$.ReplicationInstanceArn') AS "replication_instance_arn",
       json_extract(r.provider_properties, '$.ReplicationTaskArn') AS "replication_task_arn",
       json_extract(r.provider_properties, '$.ReplicationTaskIdentifier') AS "replication_task_identifier",
       json_extract(r.provider_properties, '$.ResourceIdentifier') AS "provider_resource_identifier",
       json_extract(r.provider_properties, '$.SourceEndpointArn') AS "source_endpoint_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetEndpointArn') AS "target_endpoint_arn",
       json_extract(r.provider_properties, '$.TaskData') AS "task_data"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::ReplicationTask';

CREATE VIEW aws_cc_dms_replication_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ComputeConfig') AS "compute_config",
       json_extract(r.provider_properties, '$.ReplicationConfigArn') AS "replication_config_arn",
       json_extract(r.provider_properties, '$.ReplicationConfigIdentifier') AS "replication_config_identifier",
       json_extract(r.provider_properties, '$.ReplicationType') AS "replication_type",
       json_extract(r.provider_properties, '$.SourceEndpointArn') AS "source_endpoint_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetEndpointArn') AS "target_endpoint_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::DMS::ReplicationConfig';

CREATE VIEW aws_cc_msk_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.BrokerNodeGroupInfo') AS "broker_node_group_info",
       json_extract(r.provider_properties, '$.ClientAuthentication') AS "client_authentication",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.ConfigurationInfo') AS "configuration_info",
       json_extract(r.provider_properties, '$.CurrentVersion') AS "current_version",
       json_extract(r.provider_properties, '$.EncryptionInfo') AS "encryption_info",
       json_extract(r.provider_properties, '$.EnhancedMonitoring') AS "enhanced_monitoring",
       json_extract(r.provider_properties, '$.KafkaVersion') AS "kafka_version",
       json_extract(r.provider_properties, '$.LoggingInfo') AS "logging_info",
       json_extract(r.provider_properties, '$.NumberOfBrokerNodes') AS "number_of_broker_nodes",
       json_extract(r.provider_properties, '$.OpenMonitoring') AS "open_monitoring",
       json_extract(r.provider_properties, '$.Rebalancing') AS "rebalancing",
       json_extract(r.provider_properties, '$.StorageMode') AS "storage_mode",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MSK::Cluster';

CREATE VIEW aws_cc_msk_serverless_cluster AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ClientAuthentication') AS "client_authentication",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.VpcConfigs') AS "vpc_configs"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MSK::ServerlessCluster';

CREATE VIEW aws_cc_msk_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.KafkaVersionsList') AS "kafka_versions_list",
       json_extract(r.provider_properties, '$.LatestRevision') AS "latest_revision",
       json_extract(r.provider_properties, '$.Name') AS "provider_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::MSK::Configuration';

CREATE VIEW aws_cc_amazon_mq_broker AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AmqpEndpoints') AS "amqp_endpoints",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AuthenticationStrategy') AS "authentication_strategy",
       json_extract(r.provider_properties, '$.AutoMinorVersionUpgrade') AS "auto_minor_version_upgrade",
       json_extract(r.provider_properties, '$.BrokerName') AS "broker_name",
       json_extract(r.provider_properties, '$.ConfigurationId') AS "configuration_id",
       json_extract(r.provider_properties, '$.ConfigurationRevision') AS "configuration_revision",
       json_extract(r.provider_properties, '$.ConsoleURLs') AS "console_ur_ls",
       json_extract(r.provider_properties, '$.DataReplicationMode') AS "data_replication_mode",
       json_extract(r.provider_properties, '$.DeploymentMode') AS "deployment_mode",
       json_extract(r.provider_properties, '$.EncryptionOptions') AS "encryption_options",
       json_extract(r.provider_properties, '$.EngineType') AS "engine_type",
       json_extract(r.provider_properties, '$.EngineVersionCurrent') AS "engine_version_current",
       json_extract(r.provider_properties, '$.HostInstanceType') AS "host_instance_type",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.IpAddresses') AS "ip_addresses",
       json_extract(r.provider_properties, '$.LdapServerMetadata') AS "ldap_server_metadata",
       json_extract(r.provider_properties, '$.Logs') AS "logs",
       json_extract(r.provider_properties, '$.MaintenanceWindowStartTime') AS "maintenance_window_start_time",
       json_extract(r.provider_properties, '$.MqttEndpoints') AS "mqtt_endpoints",
       json_extract(r.provider_properties, '$.OpenWireEndpoints') AS "open_wire_endpoints",
       json_extract(r.provider_properties, '$.PubliclyAccessible') AS "publicly_accessible",
       json_extract(r.provider_properties, '$.ResourceShareArns') AS "resource_share_arns",
       json_extract(r.provider_properties, '$.SecurityGroups') AS "security_groups",
       json_extract(r.provider_properties, '$.StompEndpoints') AS "stomp_endpoints",
       json_extract(r.provider_properties, '$.StorageSize') AS "storage_size",
       json_extract(r.provider_properties, '$.StorageType') AS "storage_type",
       json_extract(r.provider_properties, '$.SubnetIds') AS "subnet_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.WssEndpoints') AS "wss_endpoints"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AmazonMQ::Broker';

CREATE VIEW aws_cc_amazon_mq_configuration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AuthenticationStrategy') AS "authentication_strategy",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EngineType') AS "engine_type",
       json_extract(r.provider_properties, '$.EngineVersion') AS "engine_version",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Revision') AS "revision",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AmazonMQ::Configuration';

CREATE VIEW aws_cc_route53_resolver_resolver_endpoint AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Direction') AS "direction",
       json_extract(r.provider_properties, '$.Dns64Enabled') AS "dns64_enabled",
       json_extract(r.provider_properties, '$.HostVPCId') AS "host_vpc_id",
       json_extract(r.provider_properties, '$.IpAddressCount') AS "ip_address_count",
       json_extract(r.provider_properties, '$.IpAddresses') AS "ip_addresses",
       json_extract(r.provider_properties, '$.Ipv6InternetAccessEnabled') AS "ipv6_internet_access_enabled",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OutpostArn') AS "outpost_arn",
       json_extract(r.provider_properties, '$.PreferredInstanceType') AS "preferred_instance_type",
       json_extract(r.provider_properties, '$.Protocols') AS "protocols",
       json_extract(r.provider_properties, '$.ResolverEndpointId') AS "resolver_endpoint_id",
       json_extract(r.provider_properties, '$.ResolverEndpointType') AS "resolver_endpoint_type",
       json_extract(r.provider_properties, '$.RniEnhancedMetricsEnabled') AS "rni_enhanced_metrics_enabled",
       json_extract(r.provider_properties, '$.SecurityGroupIds') AS "security_group_ids",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetNameServerMetricsEnabled') AS "target_name_server_metrics_enabled"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::ResolverEndpoint';

CREATE VIEW aws_cc_route53_resolver_resolver_rule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DelegationRecord') AS "delegation_record",
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ResolverEndpointId') AS "resolver_endpoint_id",
       json_extract(r.provider_properties, '$.ResolverRuleId') AS "resolver_rule_id",
       json_extract(r.provider_properties, '$.RuleType') AS "rule_type",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TargetIps') AS "target_ips"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::ResolverRule';

CREATE VIEW aws_cc_route53_resolver_resolver_rule_association AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ResolverRuleAssociationId') AS "resolver_rule_association_id",
       json_extract(r.provider_properties, '$.ResolverRuleId') AS "resolver_rule_id",
       json_extract(r.provider_properties, '$.VPCId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::ResolverRuleAssociation';

CREATE VIEW aws_cc_route53_resolver_resolver_query_logging_config AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AssociationCount') AS "association_count",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.CreatorRequestId') AS "creator_request_id",
       json_extract(r.provider_properties, '$.DestinationArn') AS "destination_arn",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.ShareStatus') AS "share_status",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::ResolverQueryLoggingConfig';

CREATE VIEW aws_cc_route53_resolver_firewall_domain_list AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.CreatorRequestId') AS "creator_request_id",
       json_extract(r.provider_properties, '$.DomainCount') AS "domain_count",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.ManagedOwnerName') AS "managed_owner_name",
       json_extract(r.provider_properties, '$.ModificationTime') AS "modification_time",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusMessage') AS "status_message",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::FirewallDomainList';

CREATE VIEW aws_cc_route53_resolver_firewall_rule_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.CreatorRequestId') AS "creator_request_id",
       json_extract(r.provider_properties, '$.FirewallRules') AS "firewall_rules",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.ModificationTime') AS "modification_time",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OwnerId') AS "owner_id",
       json_extract(r.provider_properties, '$.RuleCount') AS "rule_count",
       json_extract(r.provider_properties, '$.ShareStatus') AS "share_status",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.StatusMessage') AS "status_message",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Route53Resolver::FirewallRuleGroup';

CREATE VIEW aws_cc_network_firewall_firewall AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AvailabilityZoneChangeProtection') AS "availability_zone_change_protection",
       json_extract(r.provider_properties, '$.AvailabilityZoneMappings') AS "availability_zone_mappings",
       json_extract(r.provider_properties, '$.DeleteProtection') AS "delete_protection",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EnabledAnalysisTypes') AS "enabled_analysis_types",
       json_extract(r.provider_properties, '$.EndpointIds') AS "endpoint_ids",
       json_extract(r.provider_properties, '$.FirewallArn') AS "firewall_arn",
       json_extract(r.provider_properties, '$.FirewallId') AS "firewall_id",
       json_extract(r.provider_properties, '$.FirewallName') AS "firewall_name",
       json_extract(r.provider_properties, '$.FirewallPolicyArn') AS "firewall_policy_arn",
       json_extract(r.provider_properties, '$.FirewallPolicyChangeProtection') AS "firewall_policy_change_protection",
       json_extract(r.provider_properties, '$.NatGatewayMappings') AS "nat_gateway_mappings",
       json_extract(r.provider_properties, '$.NoSourcePreservation') AS "no_source_preservation",
       json_extract(r.provider_properties, '$.NumberOfAssociations') AS "number_of_associations",
       json_extract(r.provider_properties, '$.ProxySettings') AS "proxy_settings",
       json_extract(r.provider_properties, '$.SubnetChangeProtection') AS "subnet_change_protection",
       json_extract(r.provider_properties, '$.SubnetMappings') AS "subnet_mappings",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TransitGatewayAttachmentId') AS "transit_gateway_attachment_id",
       json_extract(r.provider_properties, '$.TransitGatewayId') AS "transit_gateway_id",
       json_extract(r.provider_properties, '$.VpcEndpoint') AS "vpc_endpoint",
       json_extract(r.provider_properties, '$.VpcId') AS "vpc_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::NetworkFirewall::Firewall';

CREATE VIEW aws_cc_network_firewall_firewall_policy AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FirewallPolicy') AS "firewall_policy",
       json_extract(r.provider_properties, '$.FirewallPolicyArn') AS "firewall_policy_arn",
       json_extract(r.provider_properties, '$.FirewallPolicyId') AS "firewall_policy_id",
       json_extract(r.provider_properties, '$.FirewallPolicyName') AS "firewall_policy_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::NetworkFirewall::FirewallPolicy';

CREATE VIEW aws_cc_network_firewall_rule_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Capacity') AS "capacity",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.RuleGroup') AS "rule_group",
       json_extract(r.provider_properties, '$.RuleGroupArn') AS "rule_group_arn",
       json_extract(r.provider_properties, '$.RuleGroupId') AS "rule_group_id",
       json_extract(r.provider_properties, '$.RuleGroupName') AS "rule_group_name",
       json_extract(r.provider_properties, '$.SummaryConfiguration') AS "summary_configuration",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::NetworkFirewall::RuleGroup';

CREATE VIEW aws_cc_service_discovery_http_namespace AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ServiceDiscovery::HttpNamespace';

CREATE VIEW aws_cc_service_discovery_private_dns_namespace AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.HostedZoneId') AS "hosted_zone_id",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Properties') AS "configuration_properties",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ServiceDiscovery::PrivateDnsNamespace';

CREATE VIEW aws_cc_service_discovery_public_dns_namespace AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.HostedZoneId') AS "hosted_zone_id",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Properties') AS "configuration_properties",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ServiceDiscovery::PublicDnsNamespace';

CREATE VIEW aws_cc_service_discovery_service AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DnsConfig') AS "dns_config",
       json_extract(r.provider_properties, '$.HealthCheckConfig') AS "health_check_config",
       json_extract(r.provider_properties, '$.HealthCheckCustomConfig') AS "health_check_custom_config",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.NamespaceId') AS "namespace_id",
       json_extract(r.provider_properties, '$.ServiceAttributes') AS "service_attributes",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ServiceDiscovery::Service';

CREATE VIEW aws_cc_app_runner_service AS
SELECT r.*,
       json_extract(r.provider_properties, '$.EncryptionConfiguration') AS "encryption_configuration",
       json_extract(r.provider_properties, '$.HealthCheckConfiguration') AS "health_check_configuration",
       json_extract(r.provider_properties, '$.InstanceConfiguration') AS "instance_configuration",
       json_extract(r.provider_properties, '$.NetworkConfiguration') AS "network_configuration",
       json_extract(r.provider_properties, '$.ObservabilityConfiguration') AS "observability_configuration",
       json_extract(r.provider_properties, '$.ServiceArn') AS "service_arn",
       json_extract(r.provider_properties, '$.ServiceId') AS "service_id",
       json_extract(r.provider_properties, '$.ServiceName') AS "service_name",
       json_extract(r.provider_properties, '$.ServiceUrl') AS "service_url",
       json_extract(r.provider_properties, '$.SourceConfiguration') AS "source_configuration",
       json_extract(r.provider_properties, '$.Status') AS "status"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppRunner::Service';

CREATE VIEW aws_cc_app_runner_vpc_connector AS
SELECT r.*,
       json_extract(r.provider_properties, '$.SecurityGroups') AS "security_groups",
       json_extract(r.provider_properties, '$.Subnets') AS "subnets",
       json_extract(r.provider_properties, '$.VpcConnectorArn') AS "vpc_connector_arn",
       json_extract(r.provider_properties, '$.VpcConnectorName') AS "vpc_connector_name",
       json_extract(r.provider_properties, '$.VpcConnectorRevision') AS "vpc_connector_revision"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppRunner::VpcConnector';

CREATE VIEW aws_cc_app_runner_vpc_ingress_connection AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.IngressVpcConfiguration') AS "ingress_vpc_configuration",
       json_extract(r.provider_properties, '$.ServiceArn') AS "service_arn",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.VpcIngressConnectionArn') AS "vpc_ingress_connection_arn",
       json_extract(r.provider_properties, '$.VpcIngressConnectionName') AS "vpc_ingress_connection_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppRunner::VpcIngressConnection';

CREATE VIEW aws_cc_app_config_application AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationId') AS "application_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppConfig::Application';

CREATE VIEW aws_cc_app_config_deployment_strategy AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DeploymentDurationInMinutes') AS "deployment_duration_in_minutes",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.FinalBakeTimeInMinutes') AS "final_bake_time_in_minutes",
       json_extract(r.provider_properties, '$.GrowthFactor') AS "growth_factor",
       json_extract(r.provider_properties, '$.GrowthType') AS "growth_type",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ReplicateTo') AS "replicate_to",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppConfig::DeploymentStrategy';

CREATE VIEW aws_cc_athena_work_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.WorkGroupConfiguration') AS "work_group_configuration"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Athena::WorkGroup';

CREATE VIEW aws_cc_athena_data_catalog AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConnectionType') AS "connection_type",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Error') AS "error",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Athena::DataCatalog';

CREATE VIEW aws_cc_glue_database AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CatalogId') AS "catalog_id",
       json_extract(r.provider_properties, '$.DatabaseInput') AS "database_input",
       json_extract(r.provider_properties, '$.DatabaseName') AS "database_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Database';

CREATE VIEW aws_cc_glue_crawler AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Classifiers') AS "classifiers",
       json_extract(r.provider_properties, '$.CrawlerSecurityConfiguration') AS "crawler_security_configuration",
       json_extract(r.provider_properties, '$.DatabaseName') AS "database_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.LakeFormationConfiguration') AS "lake_formation_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RecrawlPolicy') AS "recrawl_policy",
       json_extract(r.provider_properties, '$.Role') AS "role",
       json_extract(r.provider_properties, '$.Schedule') AS "schedule",
       json_extract(r.provider_properties, '$.SchemaChangePolicy') AS "schema_change_policy",
       json_extract(r.provider_properties, '$.TablePrefix') AS "table_prefix",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Targets') AS "targets"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Crawler';

CREATE VIEW aws_cc_glue_job AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllocatedCapacity') AS "allocated_capacity",
       json_extract(r.provider_properties, '$.Connections') AS "connections",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.ExecutionClass') AS "execution_class",
       json_extract(r.provider_properties, '$.ExecutionProperty') AS "execution_property",
       json_extract(r.provider_properties, '$.GlueVersion') AS "glue_version",
       json_extract(r.provider_properties, '$.JobMode') AS "job_mode",
       json_extract(r.provider_properties, '$.JobRunQueuingEnabled') AS "job_run_queuing_enabled",
       json_extract(r.provider_properties, '$.LogUri') AS "log_uri",
       json_extract(r.provider_properties, '$.MaintenanceWindow') AS "maintenance_window",
       json_extract(r.provider_properties, '$.MaxCapacity') AS "max_capacity",
       json_extract(r.provider_properties, '$.MaxRetries') AS "max_retries",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.NotificationProperty') AS "notification_property",
       json_extract(r.provider_properties, '$.NumberOfWorkers') AS "number_of_workers",
       json_extract(r.provider_properties, '$.Role') AS "role",
       json_extract(r.provider_properties, '$.SecurityConfiguration') AS "security_configuration",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Timeout') AS "timeout",
       json_extract(r.provider_properties, '$.WorkerType') AS "worker_type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Job';

CREATE VIEW aws_cc_glue_workflow AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.MaxConcurrentRuns') AS "max_concurrent_runs",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Glue::Workflow';

CREATE VIEW aws_cc_code_build_project AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Artifacts') AS "artifacts",
       json_extract(r.provider_properties, '$.AutoRetryLimit') AS "auto_retry_limit",
       json_extract(r.provider_properties, '$.BadgeEnabled') AS "badge_enabled",
       json_extract(r.provider_properties, '$.BuildBatchConfig') AS "build_batch_config",
       json_extract(r.provider_properties, '$.Cache') AS "cache",
       json_extract(r.provider_properties, '$.ConcurrentBuildLimit') AS "concurrent_build_limit",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EncryptionKey') AS "encryption_key",
       json_extract(r.provider_properties, '$.FileSystemLocations') AS "file_system_locations",
       json_extract(r.provider_properties, '$.LogsConfig') AS "logs_config",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.QueuedTimeoutInMinutes') AS "queued_timeout_in_minutes",
       json_extract(r.provider_properties, '$.ResourceAccessRole') AS "resource_access_role",
       json_extract(r.provider_properties, '$.SecondaryArtifacts') AS "secondary_artifacts",
       json_extract(r.provider_properties, '$.SecondarySourceVersions') AS "secondary_source_versions",
       json_extract(r.provider_properties, '$.SecondarySources') AS "secondary_sources",
       json_extract(r.provider_properties, '$.ServiceRole') AS "service_role",
       json_extract(r.provider_properties, '$.Source') AS "provider_source",
       json_extract(r.provider_properties, '$.SourceVersion') AS "source_version",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TimeoutInMinutes') AS "timeout_in_minutes",
       json_extract(r.provider_properties, '$.Visibility') AS "visibility",
       json_extract(r.provider_properties, '$.VpcConfig') AS "vpc_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodeBuild::Project';

CREATE VIEW aws_cc_code_build_report_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ExportConfig') AS "export_config",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodeBuild::ReportGroup';

CREATE VIEW aws_cc_code_pipeline_pipeline AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ArtifactStore') AS "artifact_store",
       json_extract(r.provider_properties, '$.ArtifactStores') AS "artifact_stores",
       json_extract(r.provider_properties, '$.DisableInboundStageTransitions') AS "disable_inbound_stage_transitions",
       json_extract(r.provider_properties, '$.ExecutionMode') AS "execution_mode",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.PipelineType') AS "pipeline_type",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.Stages') AS "stages",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Triggers') AS "triggers",
       json_extract(r.provider_properties, '$.Version') AS "version"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodePipeline::Pipeline';

CREATE VIEW aws_cc_code_commit_repository AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CloneUrlHttp') AS "clone_url_http",
       json_extract(r.provider_properties, '$.CloneUrlSsh') AS "clone_url_ssh",
       json_extract(r.provider_properties, '$.KmsKeyId') AS "kms_key_id",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RepositoryDescription') AS "repository_description",
       json_extract(r.provider_properties, '$.RepositoryId') AS "repository_id",
       json_extract(r.provider_properties, '$.RepositoryName') AS "repository_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Triggers') AS "triggers"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodeCommit::Repository';

CREATE VIEW aws_cc_code_artifact_domain AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.EncryptionKey') AS "encryption_key",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Owner') AS "owner",
       json_extract(r.provider_properties, '$.PermissionsPolicyDocument') AS "permissions_policy_document",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodeArtifact::Domain';

CREATE VIEW aws_cc_code_artifact_repository AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.DomainOwner') AS "domain_owner",
       json_extract(r.provider_properties, '$.ExternalConnections') AS "external_connections",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.PermissionsPolicyDocument') AS "permissions_policy_document",
       json_extract(r.provider_properties, '$.RepositoryName') AS "repository_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Upstreams') AS "upstreams"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CodeArtifact::Repository';

CREATE VIEW aws_cc_config_configuration_aggregator AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccountAggregationSources') AS "account_aggregation_sources",
       json_extract(r.provider_properties, '$.ConfigurationAggregatorArn') AS "configuration_aggregator_arn",
       json_extract(r.provider_properties, '$.ConfigurationAggregatorName') AS "configuration_aggregator_name",
       json_extract(r.provider_properties, '$.OrganizationAggregationSource') AS "organization_aggregation_source",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::ConfigurationAggregator';

CREATE VIEW aws_cc_config_configuration_recorder AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RecordingGroup') AS "recording_group",
       json_extract(r.provider_properties, '$.RecordingMode') AS "recording_mode",
       json_extract(r.provider_properties, '$.ResourceARN') AS "resource_arn",
       json_extract(r.provider_properties, '$.RoleARN') AS "role_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::ConfigurationRecorder';

CREATE VIEW aws_cc_config_config_rule AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Compliance') AS "compliance",
       json_extract(r.provider_properties, '$.ConfigRuleId') AS "config_rule_id",
       json_extract(r.provider_properties, '$.ConfigRuleName') AS "config_rule_name",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EvaluationModes') AS "evaluation_modes",
       json_extract(r.provider_properties, '$.MaximumExecutionFrequency') AS "maximum_execution_frequency",
       json_extract(r.provider_properties, '$.Scope') AS "scope",
       json_extract(r.provider_properties, '$.Source') AS "provider_source"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::ConfigRule';

CREATE VIEW aws_cc_config_delivery_channel AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ConfigSnapshotDeliveryProperties') AS "config_snapshot_delivery_properties",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.S3BucketName') AS "s3_bucket_name",
       json_extract(r.provider_properties, '$.S3KeyPrefix') AS "s3_key_prefix",
       json_extract(r.provider_properties, '$.S3KmsKeyArn') AS "s3_kms_key_arn",
       json_extract(r.provider_properties, '$.SnsTopicARN') AS "sns_topic_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Config::DeliveryChannel';

CREATE VIEW aws_cc_access_analyzer_analyzer AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AnalyzerConfiguration') AS "analyzer_configuration",
       json_extract(r.provider_properties, '$.AnalyzerName') AS "analyzer_name",
       json_extract(r.provider_properties, '$.ArchiveRules') AS "archive_rules",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AccessAnalyzer::Analyzer';

CREATE VIEW aws_cc_certificate_manager_certificate AS
SELECT r.*,
       json_extract(r.provider_properties, '$.CertificateArn') AS "certificate_arn",
       json_extract(r.provider_properties, '$.CertificateAuthorityArn') AS "certificate_authority_arn",
       json_extract(r.provider_properties, '$.CertificateExport') AS "certificate_export",
       json_extract(r.provider_properties, '$.CertificateTransparencyLoggingPreference') AS "certificate_transparency_logging_preference",
       json_extract(r.provider_properties, '$.DomainName') AS "domain_name",
       json_extract(r.provider_properties, '$.DomainValidationOptions') AS "domain_validation_options",
       json_extract(r.provider_properties, '$.KeyAlgorithm') AS "key_algorithm",
       json_extract(r.provider_properties, '$.SubjectAlternativeNames') AS "subject_alternative_names",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::CertificateManager::Certificate';

CREATE VIEW aws_cc_acmpca_certificate_authority AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CertificateSigningRequest') AS "certificate_signing_request",
       json_extract(r.provider_properties, '$.KeyAlgorithm') AS "key_algorithm",
       json_extract(r.provider_properties, '$.SigningAlgorithm') AS "signing_algorithm",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.UsageMode') AS "usage_mode"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ACMPCA::CertificateAuthority';

CREATE VIEW aws_cc_guard_duty_detector AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DataSources') AS "data_sources",
       json_extract(r.provider_properties, '$.Enable') AS "enable",
       json_extract(r.provider_properties, '$.Features') AS "features",
       json_extract(r.provider_properties, '$.FindingPublishingFrequency') AS "finding_publishing_frequency",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::GuardDuty::Detector';

CREATE VIEW aws_cc_security_hub_hub AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ARN') AS "provider_arn",
       json_extract(r.provider_properties, '$.AutoEnableControls') AS "auto_enable_controls",
       json_extract(r.provider_properties, '$.ControlFindingGenerator') AS "control_finding_generator",
       json_extract(r.provider_properties, '$.SubscribedAt') AS "subscribed_at",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::SecurityHub::Hub';

CREATE VIEW aws_cc_scheduler_schedule_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationDate') AS "creation_date",
       json_extract(r.provider_properties, '$.LastModificationDate') AS "last_modification_date",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.State') AS "state",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Scheduler::ScheduleGroup';

CREATE VIEW aws_cc_pipes_pipe AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.CurrentState') AS "current_state",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DesiredState') AS "desired_state",
       json_extract(r.provider_properties, '$.Enrichment') AS "enrichment",
       json_extract(r.provider_properties, '$.EnrichmentParameters') AS "enrichment_parameters",
       json_extract(r.provider_properties, '$.KmsKeyIdentifier') AS "kms_key_identifier",
       json_extract(r.provider_properties, '$.LastModifiedTime') AS "last_modified_time",
       json_extract(r.provider_properties, '$.LogConfiguration') AS "log_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn",
       json_extract(r.provider_properties, '$.Source') AS "provider_source",
       json_extract(r.provider_properties, '$.StateReason') AS "state_reason",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Target') AS "target"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Pipes::Pipe';

CREATE VIEW aws_cc_ram_resource_share AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AllowExternalPrincipals') AS "allow_external_principals",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CreationTime') AS "creation_time",
       json_extract(r.provider_properties, '$.FeatureSet') AS "feature_set",
       json_extract(r.provider_properties, '$.LastUpdatedTime') AS "last_updated_time",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.OwningAccountId') AS "owning_account_id",
       json_extract(r.provider_properties, '$.ResourceShareConfiguration') AS "resource_share_configuration",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::RAM::ResourceShare';

CREATE VIEW aws_cc_vpc_lattice_service AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AuthType') AS "auth_type",
       json_extract(r.provider_properties, '$.CertificateArn') AS "certificate_arn",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.CustomDomainName') AS "custom_domain_name",
       json_extract(r.provider_properties, '$.DnsEntry') AS "dns_entry",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.IdleTimeoutSeconds') AS "idle_timeout_seconds",
       json_extract(r.provider_properties, '$.LastUpdatedAt') AS "last_updated_at",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::VpcLattice::Service';

CREATE VIEW aws_cc_vpc_lattice_service_network AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.AuthType') AS "auth_type",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.LastUpdatedAt') AS "last_updated_at",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.SharingConfig') AS "sharing_config",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::VpcLattice::ServiceNetwork';

CREATE VIEW aws_cc_vpc_lattice_target_group AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.Config') AS "config",
       json_extract(r.provider_properties, '$.CreatedAt') AS "created_at",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.LastUpdatedAt') AS "last_updated_at",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Status') AS "status",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Targets') AS "targets",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::VpcLattice::TargetGroup';

