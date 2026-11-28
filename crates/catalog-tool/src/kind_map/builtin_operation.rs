use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "kafkaconnect",
        operation: "ListConnectors",
        kind: "AWS::KafkaConnect::Connector",
    },
    OperationMapping {
        service: "kafkaconnect",
        operation: "ListCustomPlugins",
        kind: "AWS::KafkaConnect::CustomPlugin",
    },
    OperationMapping {
        service: "kafkaconnect",
        operation: "ListWorkerConfigurations",
        kind: "AWS::KafkaConnect::WorkerConfiguration",
    },
    OperationMapping {
        service: "kafkaconnect",
        operation: "DescribeCustomPlugin",
        kind: "AWS::KafkaConnect::CustomPlugin",
    },
    OperationMapping {
        service: "kafkaconnect",
        operation: "ListConnectorOperations",
        kind: "AWS::ApiOnly::KafkaConnectConnectorOperation",
    },
    OperationMapping {
        service: "kinesis-video",
        operation: "ListStreams",
        kind: "AWS::KinesisVideo::Stream",
    },
    OperationMapping {
        service: "kinesis-video",
        operation: "ListSignalingChannels",
        kind: "AWS::KinesisVideo::SignalingChannel",
    },
    OperationMapping {
        service: "kinesis-video",
        operation: "DescribeMappedResourceConfiguration",
        kind: "AWS::ApiOnly::KinesisVideoMappedResourceConfiguration",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcs",
        kind: "AWS::EC2::VPC",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSubnets",
        kind: "AWS::EC2::Subnet",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeRegions",
        kind: "AWS::ApiOnly::Region",
    },
    OperationMapping {
        service: "sts",
        operation: "GetCallerIdentity",
        kind: "AWS::ApiOnly::Account",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInternetGateways",
        kind: "AWS::EC2::InternetGateway",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNatGateways",
        kind: "AWS::EC2::NatGateway",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkAcls",
        kind: "AWS::EC2::NetworkAcl",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkInterfaces",
        kind: "AWS::EC2::NetworkInterface",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeRouteTables",
        kind: "AWS::EC2::RouteTable",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSecurityGroups",
        kind: "AWS::EC2::SecurityGroup",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVolumes",
        kind: "AWS::EC2::Volume",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcEndpoints",
        kind: "AWS::EC2::VPCEndpoint",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcPeeringConnections",
        kind: "AWS::EC2::VPCPeeringConnection",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeAccountAttributes",
        kind: "AWS::ApiOnly::Ec2AccountAttribute",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeAddressTransfers",
        kind: "AWS::ApiOnly::Ec2AddressTransfer",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeAddresses",
        kind: "AWS::EC2::EIP",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeAddressesAttribute",
        kind: "AWS::ApiOnly::Ec2AddressAttribute",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeApplicationStatusChecks",
        kind: "AWS::EC2::ApplicationStatusCheck",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeByoipCidrs",
        kind: "AWS::ApiOnly::Ec2ByoipCidr",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCapacityBlockStatus",
        kind: "AWS::ApiOnly::Ec2CapacityBlockStatus",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCapacityBlocks",
        kind: "AWS::ApiOnly::Ec2CapacityBlock",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCapacityManagerDataExports",
        kind: "AWS::EC2::CapacityManagerDataExport",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCapacityReservationCancellationQuotes",
        kind: "AWS::ApiOnly::Ec2CapacityReservationCancellationQuote",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCapacityReservationDateChangeQuotes",
        kind: "AWS::ApiOnly::Ec2CapacityReservationModificationQuote",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCapacityReservationFleets",
        kind: "AWS::EC2::CapacityReservationFleet",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCapacityReservationTopology",
        kind: "AWS::ApiOnly::Ec2CapacityReservationTopology",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCapacityReservations",
        kind: "AWS::EC2::CapacityReservation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCarrierGateways",
        kind: "AWS::EC2::CarrierGateway",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeClassicLinkInstances",
        kind: "AWS::ApiOnly::Ec2ClassicLinkInstance",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeClientVpnConnections",
        kind: "AWS::ApiOnly::Ec2ClientVpnConnection",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeClientVpnEndpoints",
        kind: "AWS::EC2::ClientVpnEndpoint",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeClientVpnTargetNetworks",
        kind: "AWS::EC2::ClientVpnTargetNetworkAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCoipPools",
        kind: "AWS::ApiOnly::Ec2CoipPool",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeCustomerGateways",
        kind: "AWS::EC2::CustomerGateway",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeDeclarativePoliciesReports",
        kind: "AWS::ApiOnly::Ec2DeclarativePoliciesReport",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeDhcpOptions",
        kind: "AWS::EC2::DHCPOptions",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeEgressOnlyInternetGateways",
        kind: "AWS::EC2::EgressOnlyInternetGateway",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeElasticGpus",
        kind: "AWS::ApiOnly::Ec2ElasticGpu",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeExportImageTasks",
        kind: "AWS::ApiOnly::Ec2ExportImageTask",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeExportTasks",
        kind: "AWS::ApiOnly::Ec2ExportTask",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeFastLaunchImages",
        kind: "AWS::ApiOnly::Ec2FastLaunchImage",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeFastSnapshotRestores",
        kind: "AWS::ApiOnly::Ec2FastSnapshotRestore",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeFlowLogs",
        kind: "AWS::EC2::FlowLog",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeFpgaImages",
        kind: "AWS::EC2::FpgaImage",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeHostReservations",
        kind: "AWS::ApiOnly::Ec2HostReservation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeHosts",
        kind: "AWS::EC2::Host",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIamInstanceProfileAssociations",
        kind: "AWS::ApiOnly::Ec2IamInstanceProfileAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIdFormat",
        kind: "AWS::ApiOnly::Ec2IdFormat",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeImageUsageReportEntries",
        kind: "AWS::ApiOnly::Ec2ImageUsageReportEntry",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeImageUsageReports",
        kind: "AWS::ApiOnly::Ec2ImageUsageReport",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeImages",
        kind: "AWS::ApiOnly::Ec2Image",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInstanceConnectEndpoints",
        kind: "AWS::EC2::InstanceConnectEndpoint",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInstanceCreditSpecifications",
        kind: "AWS::ApiOnly::Ec2InstanceCreditSpecification",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInstanceEventWindows",
        kind: "AWS::EC2::InstanceEventWindow",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInstanceImageMetadata",
        kind: "AWS::EC2::Instance",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInstances",
        kind: "AWS::EC2::Instance",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInstanceSqlHaStates",
        kind: "AWS::ApiOnly::Ec2SqlHaInstance",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInstanceStatus",
        kind: "AWS::ApiOnly::Ec2InstanceStatus",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeInstanceTopology",
        kind: "AWS::ApiOnly::Ec2InstanceTopology",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamByoasn",
        kind: "AWS::ApiOnly::Ec2Byoasn",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamInternetRegistryAssociations",
        kind: "AWS::EC2::IpamInternetRegistryAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamPolicies",
        kind: "AWS::ApiOnly::Ec2IpamPolicy",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamPoolAllocations",
        kind: "AWS::EC2::IPAMAllocation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamPools",
        kind: "AWS::EC2::IPAMPool",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamPrefixListResolverTargets",
        kind: "AWS::EC2::IPAMPrefixListResolverTarget",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamPrefixListResolvers",
        kind: "AWS::EC2::IPAMPrefixListResolver",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamResourceDiscoveries",
        kind: "AWS::EC2::IPAMResourceDiscovery",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamResourceDiscoveryAssociations",
        kind: "AWS::EC2::IPAMResourceDiscoveryAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpamScopes",
        kind: "AWS::EC2::IPAMScope",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpams",
        kind: "AWS::EC2::IPAM",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeIpv6Pools",
        kind: "AWS::ApiOnly::Ec2Ipv6Pool",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeKeyPairs",
        kind: "AWS::EC2::KeyPair",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeLaunchTemplates",
        kind: "AWS::EC2::LaunchTemplate",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeLocalGatewayRouteTableVirtualInterfaceGroupAssociations",
        kind: "AWS::EC2::LocalGatewayRouteTableVirtualInterfaceGroupAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeLocalGatewayRouteTableVpcAssociations",
        kind: "AWS::EC2::LocalGatewayRouteTableVPCAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeLocalGatewayRouteTables",
        kind: "AWS::EC2::LocalGatewayRouteTable",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeLocalGatewayVirtualInterfaceGroups",
        kind: "AWS::EC2::LocalGatewayVirtualInterfaceGroup",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeLocalGatewayVirtualInterfaces",
        kind: "AWS::EC2::LocalGatewayVirtualInterface",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeLocalGateways",
        kind: "AWS::ApiOnly::Ec2LocalGateway",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeLockedSnapshots",
        kind: "AWS::ApiOnly::Ec2SnapshotLock",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeMacHosts",
        kind: "AWS::ApiOnly::Ec2MacHost",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeMacModificationTasks",
        kind: "AWS::ApiOnly::Ec2MacModificationTask",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeManagedPrefixLists",
        kind: "AWS::EC2::PrefixList",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeMetricCategories",
        kind: "AWS::ApiOnly::Ec2MetricCategory",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeMovingAddresses",
        kind: "AWS::ApiOnly::Ec2MovingAddressStatus",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkInsightsAccessScopeAnalyses",
        kind: "AWS::EC2::NetworkInsightsAccessScopeAnalysis",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkInsightsAccessScopes",
        kind: "AWS::EC2::NetworkInsightsAccessScope",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkInsightsAnalyses",
        kind: "AWS::EC2::NetworkInsightsAnalysis",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkInsightsPaths",
        kind: "AWS::EC2::NetworkInsightsPath",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkInterfacePermissions",
        kind: "AWS::EC2::NetworkInterfacePermission",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkingFlexCostAllocationPolicies",
        kind: "AWS::ApiOnly::Ec2NetworkingFlexCostAllocationPolicy",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeNetworkingFlexCostAllocationPolicyAssociations",
        kind: "AWS::ApiOnly::Ec2NetworkingFlexCostAllocationPolicyAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeOutpostLags",
        kind: "AWS::ApiOnly::Ec2OutpostLag",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribePlacementGroups",
        kind: "AWS::EC2::PlacementGroup",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribePrincipalIdFormat",
        kind: "AWS::ApiOnly::Ec2PrincipalIdFormat",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribePublicIpv4Pools",
        kind: "AWS::EC2::IPv4Pool",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeReplaceRootVolumeTasks",
        kind: "AWS::ApiOnly::Ec2ReplaceRootVolumeTask",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeReservedInstances",
        kind: "AWS::ApiOnly::Ec2ReservedInstance",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeReservedInstancesListings",
        kind: "AWS::ApiOnly::Ec2ReservedInstancesListing",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeReservedInstancesModifications",
        kind: "AWS::ApiOnly::Ec2ReservedInstancesModification",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeRouteServerEndpoints",
        kind: "AWS::EC2::RouteServerEndpoint",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeRouteServerPeers",
        kind: "AWS::EC2::RouteServerPeer",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeRouteServers",
        kind: "AWS::EC2::RouteServer",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeScheduledInstances",
        kind: "AWS::ApiOnly::Ec2ScheduledInstance",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSecondaryInterfaces",
        kind: "AWS::ApiOnly::Ec2SecondaryInterface",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSecondaryNetworks",
        kind: "AWS::EC2::SecondaryNetwork",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSecondarySubnets",
        kind: "AWS::ApiOnly::Ec2SecondarySubnet",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSecurityGroupRules",
        kind: "AWS::ApiOnly::Ec2SecurityGroupRule",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSecurityGroupVpcAssociations",
        kind: "AWS::EC2::SecurityGroupVpcAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeServiceLinkVirtualInterfaces",
        kind: "AWS::ApiOnly::Ec2ServiceLinkVirtualInterface",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSnapshotTierStatus",
        kind: "AWS::ApiOnly::Ec2SnapshotTierStatus",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSnapshots",
        kind: "AWS::EC2::Snapshot",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeSpotDatafeedSubscription",
        kind: "AWS::ApiOnly::Ec2SpotDatafeedSubscription",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeStaleSecurityGroups",
        kind: "AWS::ApiOnly::Ec2StaleSecurityGroup",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTags",
        kind: "AWS::ApiOnly::Ec2TagDescription",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTrafficMirrorFilterRules",
        kind: "AWS::EC2::TrafficMirrorFilterRule",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTrafficMirrorFilters",
        kind: "AWS::EC2::TrafficMirrorFilter",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTrafficMirrorSessions",
        kind: "AWS::EC2::TrafficMirrorSession",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTrafficMirrorTargets",
        kind: "AWS::EC2::TrafficMirrorTarget",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayAttachments",
        kind: "AWS::ApiOnly::Ec2TransitGatewayAttachment",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayConnectPeers",
        kind: "AWS::EC2::TransitGatewayConnectPeer",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayConnects",
        kind: "AWS::EC2::TransitGatewayConnect",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayMeteringPolicies",
        kind: "AWS::EC2::TransitGatewayMeteringPolicy",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayMulticastDomains",
        kind: "AWS::EC2::TransitGatewayMulticastDomain",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayPeeringAttachments",
        kind: "AWS::EC2::TransitGatewayPeeringAttachment",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayPolicyTables",
        kind: "AWS::EC2::TransitGatewayPolicyTable",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayRouteTableAnnouncements",
        kind: "AWS::ApiOnly::Ec2TransitGatewayRouteTableAnnouncement",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayRouteTables",
        kind: "AWS::EC2::TransitGatewayRouteTable",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGatewayVpcAttachments",
        kind: "AWS::EC2::TransitGatewayVpcAttachment",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTransitGateways",
        kind: "AWS::EC2::TransitGateway",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeTrunkInterfaceAssociations",
        kind: "AWS::ApiOnly::Ec2TrunkInterfaceAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVerifiedAccessEndpoints",
        kind: "AWS::EC2::VerifiedAccessEndpoint",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVerifiedAccessGroups",
        kind: "AWS::EC2::VerifiedAccessGroup",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVerifiedAccessInstanceLoggingConfigurations",
        kind: "AWS::ApiOnly::Ec2VerifiedAccessInstanceLoggingConfiguration",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVerifiedAccessInstances",
        kind: "AWS::EC2::VerifiedAccessInstance",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVolumeStatus",
        kind: "AWS::ApiOnly::Ec2VolumeStatus",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVolumesModifications",
        kind: "AWS::ApiOnly::Ec2VolumeModification",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcBlockPublicAccessExclusions",
        kind: "AWS::EC2::VPCBlockPublicAccessExclusion",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcBlockPublicAccessOptions",
        kind: "AWS::EC2::VPCBlockPublicAccessOptions",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcClassicLink",
        kind: "AWS::ApiOnly::Ec2VpcClassicLink",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcClassicLinkDnsSupport",
        kind: "AWS::ApiOnly::Ec2ClassicLinkDnsSupport",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcEncryptionControls",
        kind: "AWS::EC2::VPCEncryptionControl",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcEndpointAssociations",
        kind: "AWS::ApiOnly::Ec2VpcEndpointAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcEndpointConnectionNotifications",
        kind: "AWS::EC2::VPCEndpointConnectionNotification",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcEndpointConnections",
        kind: "AWS::ApiOnly::Ec2VpcEndpointConnection",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcEndpointServiceConfigurations",
        kind: "AWS::EC2::VPCEndpointService",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpcEndpointServicePermissions",
        kind: "AWS::EC2::VPCEndpointServicePermissions",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpnConcentrators",
        kind: "AWS::EC2::VPNConcentrator",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpnConnections",
        kind: "AWS::EC2::VPNConnection",
    },
    OperationMapping {
        service: "ec2",
        operation: "DescribeVpnGateways",
        kind: "AWS::EC2::VPNGateway",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetClientVpnEndpointAuthorizationPolicy",
        kind: "AWS::ApiOnly::Ec2ClientVpnEndpointAuthorizationPolicy",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetDeclarativePoliciesReportSummary",
        kind: "AWS::ApiOnly::Ec2DeclarativePoliciesReportSummary",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetIpamPoolAllocations",
        kind: "AWS::EC2::IPAMAllocation",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetIpamPoolCidrs",
        kind: "AWS::EC2::IPAMPoolCidr",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetIpamResourceCidrs",
        kind: "AWS::ApiOnly::Ec2IpamResourceCidr",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetNetworkInsightsAccessScopeAnalysisFindings",
        kind: "AWS::ApiOnly::Ec2AccessScopeAnalysisFinding",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetNetworkInsightsAccessScopeContent",
        kind: "AWS::ApiOnly::Ec2NetworkInsightsAccessScopeContent",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetNetworkingFlexCostAllocationPolicyVersions",
        kind: "AWS::ApiOnly::Ec2FlexCostAllocationPolicyVersion",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetRouteServerAssociations",
        kind: "AWS::EC2::RouteServerAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetRouteServerPropagations",
        kind: "AWS::EC2::RouteServerPropagation",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetSecurityGroupsForVpc",
        kind: "AWS::ApiOnly::Ec2SecurityGroupForVpc",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetTransitGatewayAttachmentNatConfiguration",
        kind: "AWS::EC2::TransitGatewayAttachmentNatConfiguration",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetTransitGatewayPolicyTableAssociations",
        kind: "AWS::EC2::TransitGatewayPolicyTableAssociation",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetTransitGatewayPrefixListReferences",
        kind: "AWS::ApiOnly::Ec2TransitGatewayPrefixListReference",
    },
    OperationMapping {
        service: "ec2",
        operation: "GetVpcResourcesBlockingEncryptionEnforcement",
        kind: "AWS::ApiOnly::Ec2VpcEncryptionNonCompliantResource",
    },
    OperationMapping {
        service: "ec2",
        operation: "ListImagesInRecycleBin",
        kind: "AWS::ApiOnly::Ec2ImageRecycleBinEntry",
    },
    OperationMapping {
        service: "ec2",
        operation: "ListSnapshotsInRecycleBin",
        kind: "AWS::ApiOnly::Ec2SnapshotRecycleBinEntry",
    },
    OperationMapping {
        service: "ec2",
        operation: "ListVolumesInRecycleBin",
        kind: "AWS::ApiOnly::Ec2VolumeRecycleBinEntry",
    },
    OperationMapping {
        service: "iam",
        operation: "GetUser",
        kind: "AWS::IAM::User",
    },
    OperationMapping {
        service: "iam",
        operation: "ListUsers",
        kind: "AWS::IAM::User",
    },
    OperationMapping {
        service: "iam",
        operation: "GetPolicy",
        kind: "AWS::IAM::ManagedPolicy",
    },
    OperationMapping {
        service: "iam",
        operation: "ListPolicies",
        kind: "AWS::IAM::ManagedPolicy",
    },
    OperationMapping {
        service: "iam",
        operation: "ListAccessKeys",
        kind: "AWS::IAM::AccessKey",
    },
    OperationMapping {
        service: "iam",
        operation: "ListGroups",
        kind: "AWS::IAM::Group",
    },
    OperationMapping {
        service: "iam",
        operation: "ListOpenIDConnectProviders",
        kind: "AWS::IAM::OIDCProvider",
    },
    OperationMapping {
        service: "iam",
        operation: "ListVirtualMFADevices",
        kind: "AWS::IAM::VirtualMFADevice",
    },
    OperationMapping {
        service: "iam",
        operation: "ListSAMLProviders",
        kind: "AWS::IAM::SAMLProvider",
    },
    OperationMapping {
        service: "iam",
        operation: "ListServerCertificates",
        kind: "AWS::IAM::ServerCertificate",
    },
    OperationMapping {
        service: "iam",
        operation: "GetLoginProfile",
        kind: "AWS::ApiOnly::IamLoginProfile",
    },
    OperationMapping {
        service: "iam",
        operation: "ListMFADevices",
        kind: "AWS::ApiOnly::IamMFADeviceAssociation",
    },
    OperationMapping {
        service: "iam",
        operation: "ListSSHPublicKeys",
        kind: "AWS::ApiOnly::IamSSHPublicKey",
    },
    OperationMapping {
        service: "iam",
        operation: "ListAccountAliases",
        kind: "AWS::ApiOnly::IAMAccountAlias",
    },
    OperationMapping {
        service: "iam",
        operation: "ListServiceSpecificCredentials",
        kind: "AWS::ApiOnly::IAMServiceSpecificCredential",
    },
    OperationMapping {
        service: "iam",
        operation: "ListGroupPolicies",
        kind: "AWS::IAM::GroupPolicy",
    },
    OperationMapping {
        service: "iam",
        operation: "ListRolePolicies",
        kind: "AWS::IAM::RolePolicy",
    },
    OperationMapping {
        service: "iam",
        operation: "ListUserPolicies",
        kind: "AWS::IAM::UserPolicy",
    },
    OperationMapping {
        service: "iam",
        operation: "ListPolicyVersions",
        kind: "AWS::ApiOnly::IAMPolicyVersion",
    },
    OperationMapping {
        service: "organizations",
        operation: "DescribeAccount",
        kind: "AWS::Organizations::Account",
    },
    OperationMapping {
        service: "organizations",
        operation: "ListAccounts",
        kind: "AWS::Organizations::Account",
    },
    OperationMapping {
        service: "organizations",
        operation: "DescribeOrganization",
        kind: "AWS::Organizations::Organization",
    },
    OperationMapping {
        service: "organizations",
        operation: "DescribeResourcePolicy",
        kind: "AWS::Organizations::ResourcePolicy",
    },
    OperationMapping {
        service: "organizations",
        operation: "DescribeCreateAccountStatus",
        kind: "AWS::ApiOnly::OrganizationsCreateAccountRequest",
    },
    OperationMapping {
        service: "organizations",
        operation: "ListCreateAccountStatus",
        kind: "AWS::ApiOnly::OrganizationsCreateAccountRequest",
    },
    OperationMapping {
        service: "organizations",
        operation: "ListAWSServiceAccessForOrganization",
        kind: "AWS::ApiOnly::OrganizationsEnabledServicePrincipal",
    },
    OperationMapping {
        service: "organizations",
        operation: "ListDelegatedAdministrators",
        kind: "AWS::ApiOnly::OrganizationsDelegatedAdministrator",
    },
    OperationMapping {
        service: "organizations",
        operation: "ListDelegatedServicesForAccount",
        kind: "AWS::ApiOnly::OrganizationsDelegatedService",
    },
    OperationMapping {
        service: "organizations",
        operation: "ListInboundResponsibilityTransfers",
        kind: "AWS::ApiOnly::OrganizationsResponsibilityTransfer",
    },
    OperationMapping {
        service: "organizations",
        operation: "ListOutboundResponsibilityTransfers",
        kind: "AWS::ApiOnly::OrganizationsResponsibilityTransfer",
    },
    OperationMapping {
        service: "organizations",
        operation: "ListRoots",
        kind: "AWS::ApiOnly::OrganizationsRoot",
    },
    OperationMapping {
        service: "s3",
        operation: "ListBuckets",
        kind: "AWS::S3::Bucket",
    },
    OperationMapping {
        service: "s3",
        operation: "ListDirectoryBuckets",
        kind: "AWS::S3Express::DirectoryBucket",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListHostedZones",
        kind: "AWS::Route53::HostedZone",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListHostedZonesByName",
        kind: "AWS::Route53::HostedZone",
    },
    OperationMapping {
        service: "route-53",
        operation: "GetDNSSEC",
        kind: "AWS::Route53::DNSSEC",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListHealthChecks",
        kind: "AWS::Route53::HealthCheck",
    },
    OperationMapping {
        service: "route-53",
        operation: "GetHealthCheck",
        kind: "AWS::Route53::HealthCheck",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListCidrCollections",
        kind: "AWS::Route53::CidrCollection",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListQueryLoggingConfigs",
        kind: "AWS::Route53::QueryLoggingConfig",
    },
    OperationMapping {
        service: "route-53",
        operation: "GetQueryLoggingConfig",
        kind: "AWS::Route53::QueryLoggingConfig",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListReusableDelegationSets",
        kind: "AWS::Route53::DelegationSet",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListTrafficPolicies",
        kind: "AWS::Route53::TrafficPolicy",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListTrafficPolicyInstances",
        kind: "AWS::Route53::TrafficPolicyInstance",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListTrafficPolicyInstancesByPolicy",
        kind: "AWS::Route53::TrafficPolicyInstance",
    },
    OperationMapping {
        service: "route-53",
        operation: "GetTrafficPolicyInstance",
        kind: "AWS::Route53::TrafficPolicyInstance",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListAnycastIpLists",
        kind: "AWS::CloudFront::AnycastIpList",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "GetAnycastIpList",
        kind: "AWS::CloudFront::AnycastIpList",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListCachePolicies",
        kind: "AWS::CloudFront::CachePolicy",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListCloudFrontOriginAccessIdentities",
        kind: "AWS::CloudFront::CloudFrontOriginAccessIdentity",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListConnectionGroups",
        kind: "AWS::CloudFront::ConnectionGroup",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "GetConnectionGroup",
        kind: "AWS::CloudFront::ConnectionGroup",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListContinuousDeploymentPolicies",
        kind: "AWS::CloudFront::ContinuousDeploymentPolicy",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListDistributionTenants",
        kind: "AWS::CloudFront::DistributionTenant",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListFieldLevelEncryptionConfigs",
        kind: "AWS::CloudFront::FieldLevelEncryptionConfig",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListFieldLevelEncryptionProfiles",
        kind: "AWS::CloudFront::FieldLevelEncryptionProfile",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListFunctions",
        kind: "AWS::CloudFront::Function",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListKeyGroups",
        kind: "AWS::CloudFront::KeyGroup",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListKeyValueStores",
        kind: "AWS::CloudFront::KeyValueStore",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListOriginAccessControls",
        kind: "AWS::CloudFront::OriginAccessControl",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListOriginRequestPolicies",
        kind: "AWS::CloudFront::OriginRequestPolicy",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListRealtimeLogConfigs",
        kind: "AWS::CloudFront::RealtimeLogConfig",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListResponseHeadersPolicies",
        kind: "AWS::CloudFront::ResponseHeadersPolicy",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListStreamingDistributions",
        kind: "AWS::CloudFront::StreamingDistribution",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "GetStreamingDistribution",
        kind: "AWS::CloudFront::StreamingDistribution",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListTrustStores",
        kind: "AWS::CloudFront::TrustStore",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "GetTrustStore",
        kind: "AWS::CloudFront::TrustStore",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "ListVpcOrigins",
        kind: "AWS::CloudFront::VpcOrigin",
    },
    OperationMapping {
        service: "cloudfront",
        operation: "GetVpcOrigin",
        kind: "AWS::CloudFront::VpcOrigin",
    },
    OperationMapping {
        service: "lambda",
        operation: "ListCapacityProviders",
        kind: "AWS::Lambda::CapacityProvider",
    },
    OperationMapping {
        service: "lambda",
        operation: "GetCapacityProvider",
        kind: "AWS::Lambda::CapacityProvider",
    },
    OperationMapping {
        service: "lambda",
        operation: "ListCodeSigningConfigs",
        kind: "AWS::Lambda::CodeSigningConfig",
    },
    OperationMapping {
        service: "lambda",
        operation: "GetCodeSigningConfig",
        kind: "AWS::Lambda::CodeSigningConfig",
    },
    OperationMapping {
        service: "lambda",
        operation: "ListFunctionsByCodeSigningConfig",
        kind: "AWS::Lambda::Function",
    },
    OperationMapping {
        service: "lambda",
        operation: "ListEventSourceMappings",
        kind: "AWS::Lambda::EventSourceMapping",
    },
    OperationMapping {
        service: "lambda",
        operation: "GetEventSourceMapping",
        kind: "AWS::Lambda::EventSourceMapping",
    },
    OperationMapping {
        service: "lambda",
        operation: "ListLayerVersions",
        kind: "AWS::Lambda::LayerVersion",
    },
    OperationMapping {
        service: "lambda",
        operation: "ListFunctionVersionsByCapacityProvider",
        kind: "AWS::Lambda::Version",
    },
    OperationMapping {
        service: "lambda",
        operation: "ListLayers",
        kind: "AWS::ApiOnly::LambdaLayer",
    },
    OperationMapping {
        service: "lambda",
        operation: "ListVersionsByFunction",
        kind: "AWS::Lambda::Version",
    },
    OperationMapping {
        service: "eks",
        operation: "DescribeCluster",
        kind: "AWS::EKS::Cluster",
    },
    OperationMapping {
        service: "eks",
        operation: "ListClusters",
        kind: "AWS::EKS::Cluster",
    },
    OperationMapping {
        service: "eks",
        operation: "ListCapabilities",
        kind: "AWS::EKS::Capability",
    },
    OperationMapping {
        service: "eks",
        operation: "ListPodIdentityAssociations",
        kind: "AWS::EKS::PodIdentityAssociation",
    },
    OperationMapping {
        service: "eks",
        operation: "DescribePodIdentityAssociation",
        kind: "AWS::EKS::PodIdentityAssociation",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListClusters",
        kind: "AWS::ECS::Cluster",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListServices",
        kind: "AWS::ECS::Service",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListDaemonTaskDefinitions",
        kind: "AWS::ECS::DaemonTaskDefinition",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListTaskDefinitions",
        kind: "AWS::ECS::TaskDefinition",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListDaemons",
        kind: "AWS::ECS::Daemon",
    },
    OperationMapping {
        service: "ecs",
        operation: "DescribeDaemon",
        kind: "AWS::ECS::Daemon",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListGateways",
        kind: "AWS::ECS::Gateway",
    },
    OperationMapping {
        service: "ecs",
        operation: "DescribeGateway",
        kind: "AWS::ECS::Gateway",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListGatewayClusterAssociations",
        kind: "AWS::ECS::GatewayClusterAssociations",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListContainerInstances",
        kind: "AWS::ApiOnly::EcsContainerInstance",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListTaskDefinitionFamilies",
        kind: "AWS::ApiOnly::EcsTaskDefinitionFamily",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListAttributes",
        kind: "AWS::ApiOnly::EcsAttribute",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListAccountSettings",
        kind: "AWS::ApiOnly::EcsAccountSetting",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListDaemonDeployments",
        kind: "AWS::ApiOnly::EcsDaemonDeployment",
    },
    OperationMapping {
        service: "ecs",
        operation: "ListGatewayRoutes",
        kind: "AWS::ApiOnly::EcsGatewayRoute",
    },
    OperationMapping {
        service: "ecs",
        operation: "DescribeGatewayRoute",
        kind: "AWS::ApiOnly::EcsGatewayRoute",
    },
    OperationMapping {
        service: "kms",
        operation: "ListKeys",
        kind: "AWS::KMS::Key",
    },
    OperationMapping {
        service: "kms",
        operation: "ListAliases",
        kind: "AWS::KMS::Alias",
    },
    OperationMapping {
        service: "kms",
        operation: "ListKeyPolicies",
        kind: "AWS::ApiOnly::KMSKeyPolicy",
    },
    OperationMapping {
        service: "kms",
        operation: "DescribeCustomKeyStores",
        kind: "AWS::ApiOnly::KmsCustomKeyStore",
    },
    OperationMapping {
        service: "kms",
        operation: "ListGrants",
        kind: "AWS::ApiOnly::KmsGrant",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "ListTables",
        kind: "AWS::DynamoDB::Table",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "DescribeTable",
        kind: "AWS::DynamoDB::Table",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "ListGlobalTables",
        kind: "AWS::DynamoDB::GlobalTable",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "DescribeGlobalTable",
        kind: "AWS::DynamoDB::GlobalTable",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "ListBackups",
        kind: "AWS::ApiOnly::DynamoDBBackup",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "DescribeBackup",
        kind: "AWS::ApiOnly::DynamoDBBackup",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "ListBulkOperations",
        kind: "AWS::ApiOnly::DynamoDBBulkOperation",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "DescribeBulkOperation",
        kind: "AWS::ApiOnly::DynamoDBBulkOperation",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "ListExports",
        kind: "AWS::ApiOnly::DynamoDBExport",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "ListImports",
        kind: "AWS::ApiOnly::DynamoDBImport",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "DescribeImport",
        kind: "AWS::ApiOnly::DynamoDBImport",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "DescribeGlobalTableSettings",
        kind: "AWS::ApiOnly::DynamoDBGlobalTableSettings",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "DescribeKinesisStreamingDestination",
        kind: "AWS::ApiOnly::DynamoDBKinesisStreamingDestination",
    },
    OperationMapping {
        service: "dynamodb",
        operation: "DescribeTableReplicaAutoScaling",
        kind: "AWS::ApiOnly::DynamoDBTableReplicaAutoScaling",
    },
    OperationMapping {
        service: "sns",
        operation: "ListTopics",
        kind: "AWS::SNS::Topic",
    },
    OperationMapping {
        service: "sns",
        operation: "ListOriginationNumbers",
        kind: "AWS::ApiOnly::SnsOriginationNumber",
    },
    OperationMapping {
        service: "sfn",
        operation: "ListActivities",
        kind: "AWS::StepFunctions::Activity",
    },
    OperationMapping {
        service: "sfn",
        operation: "DescribeActivity",
        kind: "AWS::StepFunctions::Activity",
    },
    OperationMapping {
        service: "sfn",
        operation: "ListStateMachines",
        kind: "AWS::StepFunctions::StateMachine",
    },
    OperationMapping {
        service: "sfn",
        operation: "ListStateMachineAliases",
        kind: "AWS::StepFunctions::StateMachineAlias",
    },
    OperationMapping {
        service: "sfn",
        operation: "DescribeStateMachineAlias",
        kind: "AWS::StepFunctions::StateMachineAlias",
    },
    OperationMapping {
        service: "sfn",
        operation: "ListStateMachineVersions",
        kind: "AWS::StepFunctions::StateMachineVersion",
    },
    OperationMapping {
        service: "elastic-load-balancing-v2",
        operation: "DescribeListeners",
        kind: "AWS::ElasticLoadBalancingV2::Listener",
    },
    OperationMapping {
        service: "elastic-load-balancing-v2",
        operation: "DescribeRules",
        kind: "AWS::ElasticLoadBalancingV2::ListenerRule",
    },
    OperationMapping {
        service: "elastic-load-balancing-v2",
        operation: "DescribeLoadBalancers",
        kind: "AWS::ElasticLoadBalancingV2::LoadBalancer",
    },
    OperationMapping {
        service: "elastic-load-balancing-v2",
        operation: "DescribeTargetGroups",
        kind: "AWS::ElasticLoadBalancingV2::TargetGroup",
    },
    OperationMapping {
        service: "elastic-load-balancing-v2",
        operation: "DescribeTrustStores",
        kind: "AWS::ElasticLoadBalancingV2::TrustStore",
    },
    OperationMapping {
        service: "elastic-load-balancing-v2",
        operation: "DescribeAccountLimits",
        kind: "AWS::ApiOnly::ElasticLoadBalancingV2AccountLimit",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBClusterEndpoints",
        kind: "AWS::RDS::DBClusterEndpoint",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBClusterParameterGroups",
        kind: "AWS::RDS::DBClusterParameterGroup",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBClusterSnapshots",
        kind: "AWS::RDS::ClusterSnapshot",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBClusters",
        kind: "AWS::RDS::DBCluster",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBInstances",
        kind: "AWS::RDS::DBInstance",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBParameterGroups",
        kind: "AWS::RDS::DBParameterGroup",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBProxies",
        kind: "AWS::RDS::DBProxy",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBProxyEndpoints",
        kind: "AWS::RDS::DBProxyEndpoint",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBSecurityGroups",
        kind: "AWS::RDS::DBSecurityGroup",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBShardGroups",
        kind: "AWS::RDS::DBShardGroup",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBSnapshots",
        kind: "AWS::RDS::DBSnapshot",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBSubnetGroups",
        kind: "AWS::RDS::DBSubnetGroup",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeEventSubscriptions",
        kind: "AWS::RDS::EventSubscription",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeGlobalClusters",
        kind: "AWS::RDS::GlobalCluster",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeIntegrations",
        kind: "AWS::RDS::Integration",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeAccountAttributes",
        kind: "AWS::ApiOnly::RdsAccountQuota",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeBlueGreenDeployments",
        kind: "AWS::RDS::Deployment",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBCapacityReservations",
        kind: "AWS::ApiOnly::RdsDBCapacityReservation",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBClusterAutomatedBackups",
        kind: "AWS::ApiOnly::RdsDBClusterAutomatedBackup",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeDBInstanceAutomatedBackups",
        kind: "AWS::ApiOnly::RdsDBInstanceAutomatedBackup",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeExportTasks",
        kind: "AWS::ApiOnly::RdsExportTask",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribePendingMaintenanceActions",
        kind: "AWS::ApiOnly::RdsPendingMaintenance",
    },
    OperationMapping {
        service: "rds",
        operation: "DescribeReservedDBInstances",
        kind: "AWS::ApiOnly::RdsReservedDBInstance",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "DescribeAnomalyDetectors",
        kind: "AWS::CloudWatch::AnomalyDetector",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "DescribeInsightRules",
        kind: "AWS::CloudWatch::InsightRule",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "ListAlarmMuteRules",
        kind: "AWS::CloudWatch::AlarmMuteRule",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "ListDashboards",
        kind: "AWS::CloudWatch::Dashboard",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "GetDashboard",
        kind: "AWS::CloudWatch::Dashboard",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "GetOTelEnrichment",
        kind: "AWS::CloudWatch::OTelEnrichment",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "ListMetricStreams",
        kind: "AWS::CloudWatch::MetricStream",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "GetMetricStream",
        kind: "AWS::CloudWatch::MetricStream",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "GetDataset",
        kind: "AWS::ApiOnly::CloudWatchDataset",
    },
    OperationMapping {
        service: "cloudwatch",
        operation: "ListMetricDataQueryDefinitions",
        kind: "AWS::ApiOnly::CloudWatchMetricDataQueryDefinition",
    },
];
