use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::KafkaConnectConnectorOperation",
        reason: "Kafka Connect connector operation is an operation-level resource with no CFN type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::KinesisVideoMappedResourceConfiguration",
        reason: "Kinesis Video mapped resource configuration has no CFN type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Account",
        reason: "Account identity is bootstrap scope, not a CFN resource",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Region",
        reason: "Enabled region identity is bootstrap scope, not a CFN resource",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudWatchDataset",
        reason: "CloudWatch default dataset is an ARN-identified configuration resource with no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudWatchMetricDataQueryDefinition",
        reason: "CloudWatch saved metric-data query definition metadata has a stable Id; no sensitive expression is retrieved and no exact pinned CFN type exists",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DynamoDBBackup",
        reason: "DynamoDB backup has its own BackupArn and lifecycle, distinct from the source CFN Table",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DynamoDBBulkOperation",
        reason: "DynamoDB bulk operation is an ARN-identified inventory workflow, not table data",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DynamoDBExport",
        reason: "DynamoDB export job metadata has a stable ExportArn; no exported item contents are collected",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DynamoDBGlobalTableSettings",
        reason: "Legacy DynamoDB global-table regional settings are a name-keyed configuration attachment, not another global table",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DynamoDBImport",
        reason: "DynamoDB import job metadata has a stable ImportArn; no imported item contents are collected",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DynamoDBKinesisStreamingDestination",
        reason: "DynamoDB table-keyed Kinesis destination configuration is an attachment, not a duplicate CFN Table",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DynamoDBTableReplicaAutoScaling",
        reason: "DynamoDB table replica auto-scaling is a table-keyed configuration attachment, not a duplicate CFN Table",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2AccessScopeAnalysisFinding",
        reason: "EC2 GetNetworkInsightsAccessScopeAnalysisFindings returns identifiable AccessScopeAnalysisFinding inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2AccountAttribute",
        reason: "EC2 DescribeAccountAttributes returns identifiable AccountAttribute inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2AddressAttribute",
        reason: "EC2 elastic-IP reverse-DNS attributes are AllocationId-keyed configuration, not another CFN EIP; the selected schema differs from DescribeAddresses",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2AddressTransfer",
        reason: "EC2 DescribeAddressTransfers returns identifiable AddressTransfer inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2Byoasn",
        reason: "EC2 DescribeIpamByoasn returns identifiable Byoasn inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ByoipCidr",
        reason: "EC2 DescribeByoipCidrs returns identifiable ByoipCidr inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2CapacityBlock",
        reason: "EC2 DescribeCapacityBlocks returns identifiable CapacityBlock inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2CapacityBlockStatus",
        reason: "EC2 DescribeCapacityBlockStatus returns identifiable CapacityBlockStatus inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2CapacityReservationCancellationQuote",
        reason: "EC2 DescribeCapacityReservationCancellationQuotes returns identifiable CapacityReservationCancellationQuote inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2CapacityReservationModificationQuote",
        reason: "EC2 DescribeCapacityReservationDateChangeQuotes returns identifiable CapacityReservationModificationQuote inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2CapacityReservationTopology",
        reason: "EC2 DescribeCapacityReservationTopology returns identifiable CapacityReservationTopology inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ClassicLinkDnsSupport",
        reason: "EC2 DescribeVpcClassicLinkDnsSupport returns identifiable ClassicLinkDnsSupport inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ClassicLinkInstance",
        reason: "EC2 instance ClassicLink configuration is an InstanceId-keyed attachment, not another EC2 Instance",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ClientVpnConnection",
        reason: "EC2 DescribeClientVpnConnections returns identifiable ClientVpnConnection inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ClientVpnEndpointAuthorizationPolicy",
        reason: "EC2 GetClientVpnEndpointAuthorizationPolicy returns identifiable ClientVpnEndpointAuthorizationPolicy inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2CoipPool",
        reason: "EC2 DescribeCoipPools returns identifiable CoipPool inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2DeclarativePoliciesReport",
        reason: "EC2 DescribeDeclarativePoliciesReports returns identifiable DeclarativePoliciesReport inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2DeclarativePoliciesReportSummary",
        reason: "EC2 GetDeclarativePoliciesReportSummary returns identifiable DeclarativePoliciesReportSummary inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ElasticGpu",
        reason: "EC2 DescribeElasticGpus returns identifiable ElasticGpu inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ExportImageTask",
        reason: "EC2 DescribeExportImageTasks returns identifiable ExportImageTask inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ExportTask",
        reason: "EC2 DescribeExportTasks returns identifiable ExportTask inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2FastLaunchImage",
        reason: "EC2 DescribeFastLaunchImages returns identifiable FastLaunchImage inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2FastSnapshotRestore",
        reason: "EC2 DescribeFastSnapshotRestores returns identifiable FastSnapshotRestore inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2FlexCostAllocationPolicyVersion",
        reason: "EC2 GetNetworkingFlexCostAllocationPolicyVersions returns identifiable FlexCostAllocationPolicyVersion inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2HostReservation",
        reason: "EC2 DescribeHostReservations returns identifiable HostReservation inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2IamInstanceProfileAssociation",
        reason: "EC2 DescribeIamInstanceProfileAssociations returns identifiable IamInstanceProfileAssociation inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2IdFormat",
        reason: "EC2 DescribeIdFormat returns identifiable IdFormat inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2Image",
        reason: "EC2 DescribeImages returns identifiable Image inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ImageRecycleBinEntry",
        reason: "EC2 ListImagesInRecycleBin returns identifiable ImageRecycleBinEntry inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ImageUsageReport",
        reason: "EC2 DescribeImageUsageReports returns identifiable ImageUsageReport inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ImageUsageReportEntry",
        reason: "EC2 DescribeImageUsageReportEntries returns identifiable ImageUsageReportEntry inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2InstanceCreditSpecification",
        reason: "EC2 DescribeInstanceCreditSpecifications returns identifiable InstanceCreditSpecification inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2InstanceStatus",
        reason: "EC2 instance health/status is an InstanceId-keyed runtime status attachment, not another EC2 Instance",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2InstanceTopology",
        reason: "EC2 instance network-topology metadata is an InstanceId-keyed topology view, not another EC2 Instance",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2IpamPolicy",
        reason: "EC2 DescribeIpamPolicies returns identifiable IpamPolicy inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2IpamResourceCidr",
        reason: "EC2 GetIpamResourceCidrs returns identifiable IpamResourceCidr inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2Ipv6Pool",
        reason: "EC2 DescribeIpv6Pools returns identifiable Ipv6Pool inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2LocalGateway",
        reason: "EC2 DescribeLocalGateways returns identifiable LocalGateway inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2MacHost",
        reason: "EC2 Mac-host supported OS metadata is a HostId-keyed capability view, not another CFN Host",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2MacModificationTask",
        reason: "EC2 DescribeMacModificationTasks returns identifiable MacModificationTask inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2MetricCategory",
        reason: "EC2 DescribeMetricCategories returns identifiable MetricCategory inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2MovingAddressStatus",
        reason: "EC2 DescribeMovingAddresses returns identifiable MovingAddressStatus inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2NetworkInsightsAccessScopeContent",
        reason: "EC2 GetNetworkInsightsAccessScopeContent returns identifiable NetworkInsightsAccessScopeContent inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2NetworkingFlexCostAllocationPolicy",
        reason: "EC2 DescribeNetworkingFlexCostAllocationPolicies returns identifiable NetworkingFlexCostAllocationPolicy inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2NetworkingFlexCostAllocationPolicyAssociation",
        reason: "EC2 DescribeNetworkingFlexCostAllocationPolicyAssociations returns identifiable NetworkingFlexCostAllocationPolicyAssociation inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2OutpostLag",
        reason: "EC2 DescribeOutpostLags returns identifiable OutpostLag inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2PrincipalIdFormat",
        reason: "EC2 DescribePrincipalIdFormat returns identifiable PrincipalIdFormat inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ReplaceRootVolumeTask",
        reason: "EC2 DescribeReplaceRootVolumeTasks returns identifiable ReplaceRootVolumeTask inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ReservedInstance",
        reason: "EC2 DescribeReservedInstances returns identifiable ReservedInstance inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ReservedInstancesListing",
        reason: "EC2 DescribeReservedInstancesListings returns identifiable ReservedInstancesListing inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ReservedInstancesModification",
        reason: "EC2 DescribeReservedInstancesModifications returns identifiable ReservedInstancesModification inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ScheduledInstance",
        reason: "EC2 DescribeScheduledInstances returns identifiable ScheduledInstance inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SecondaryInterface",
        reason: "EC2 DescribeSecondaryInterfaces returns identifiable SecondaryInterface inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SecondarySubnet",
        reason: "EC2 DescribeSecondarySubnets returns identifiable SecondarySubnet inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SecurityGroupForVpc",
        reason: "EC2 VPC-filtered security-group membership is a GroupId-keyed configuration view with PrimaryVpcId rather than canonical VpcId; it is distinct from the full CFN SecurityGroup record",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SecurityGroupRule",
        reason: "EC2 DescribeSecurityGroupRules returns identifiable SecurityGroupRule inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2ServiceLinkVirtualInterface",
        reason: "EC2 DescribeServiceLinkVirtualInterfaces returns identifiable ServiceLinkVirtualInterface inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SnapshotLock",
        reason: "EC2 snapshot retention lock is SnapshotId-keyed configuration, not another CFN Snapshot",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SnapshotRecycleBinEntry",
        reason: "EC2 ListSnapshotsInRecycleBin returns identifiable SnapshotRecycleBinEntry inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SnapshotTierStatus",
        reason: "EC2 snapshot storage-tier status is a SnapshotId-keyed runtime attachment, not another CFN Snapshot",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SpotDatafeedSubscription",
        reason: "EC2 DescribeSpotDatafeedSubscription returns identifiable SpotDatafeedSubscription inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2SqlHaInstance",
        reason: "EC2 SQL HA registration state is an InstanceId-keyed configuration view including primary and standby instances; it is not exactly CFN SqlHaStandbyDetectedInstance",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2StaleSecurityGroup",
        reason: "EC2 DescribeStaleSecurityGroups returns identifiable StaleSecurityGroup inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2TagDescription",
        reason: "EC2 DescribeTags returns identifiable TagDescription inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2TransitGatewayAttachment",
        reason: "EC2 DescribeTransitGatewayAttachments mixes VPC, peering, VPN, and other attachment types; pinned CFN TransitGatewayAttachment represents only a VPC attachment, so this generic attachment inventory is distinct",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2TransitGatewayPrefixListReference",
        reason: "EC2 GetTransitGatewayPrefixListReferences returns identifiable TransitGatewayPrefixListReference inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2TransitGatewayRouteTableAnnouncement",
        reason: "EC2 DescribeTransitGatewayRouteTableAnnouncements returns identifiable TransitGatewayRouteTableAnnouncement inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2TrunkInterfaceAssociation",
        reason: "EC2 DescribeTrunkInterfaceAssociations returns identifiable TrunkInterfaceAssociation inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2VerifiedAccessInstanceLoggingConfiguration",
        reason: "EC2 DescribeVerifiedAccessInstanceLoggingConfigurations returns identifiable VerifiedAccessInstanceLoggingConfiguration inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2VolumeModification",
        reason: "EC2 volume modification is a VolumeId-keyed in-flight change attachment, not another CFN Volume",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2VolumeRecycleBinEntry",
        reason: "EC2 ListVolumesInRecycleBin returns identifiable VolumeRecycleBinEntry inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2VolumeStatus",
        reason: "EC2 volume health/status is a VolumeId-keyed runtime status attachment, not another CFN Volume",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2VpcClassicLink",
        reason: "EC2 DescribeVpcClassicLink returns identifiable VpcClassicLink inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2VpcEncryptionNonCompliantResource",
        reason: "EC2 GetVpcResourcesBlockingEncryptionEnforcement returns identifiable VpcEncryptionNonCompliantResource inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2VpcEndpointAssociation",
        reason: "EC2 DescribeVpcEndpointAssociations returns identifiable VpcEndpointAssociation inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Ec2VpcEndpointConnection",
        reason: "EC2 DescribeVpcEndpointConnections returns identifiable VpcEndpointConnection inventory metadata or configuration with no exact pinned CFN resource type; kept separate from its referenced resources",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EcsAccountSetting",
        reason: "ECS effective account setting is keyed by principalArn and name and has no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EcsAttribute",
        reason: "ECS attribute keyed by targetId and name is target configuration, not the target container instance itself",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EcsContainerInstance",
        reason: "ECS container-instance registration is an ARN-identified runtime component, not a CFN EC2 Instance",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EcsDaemonDeployment",
        reason: "ECS daemon deployment is an ARN-identified runtime rollout, distinct from the CFN Daemon",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EcsGatewayRoute",
        reason: "ECS gateway route is an ARN-identified routing resource with no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EcsTaskDefinitionFamily",
        reason: "ECS task-definition family name identifies a family, not a CFN TaskDefinition revision",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElasticLoadBalancingV2AccountLimit",
        reason: "ELBv2 account limit is Name-keyed quota configuration, not a load balancer or other CFN resource",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IamLoginProfile",
        reason: "IAM per-user login profile is an authentication configuration attachment, not a standalone CFN User resource",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IamMFADeviceAssociation",
        reason: "Assigned IAM MFA devices include hardware and virtual serial numbers; this is a user-device association, not exactly CFN VirtualMFADevice",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IamSSHPublicKey",
        reason: "IAM per-user SSH public-key metadata has a stable SSHPublicKeyId and no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IAMAccountAlias",
        reason: "IAM account alias is account-level naming configuration, not a new account or principal and has no exact pinned CloudFormation resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IAMServiceSpecificCredential",
        reason: "IAM service-specific credential metadata identifies an existing user's authentication attachment without retrieving credential secrets; no exact pinned CloudFormation resource type exists",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IAMPolicyVersion",
        reason: "IAM managed-policy version metadata is scoped by the parent managed-policy ARN and version ID, not a new policy or a standalone pinned CloudFormation resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::KMSKeyPolicy",
        reason: "KMS key-policy names are key-scoped configuration attachments, not policy content or additional keys; no exact standalone pinned CloudFormation key-policy resource type exists",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::KmsCustomKeyStore",
        reason: "KMS custom key store has a stable CustomKeyStoreId and no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::KmsGrant",
        reason: "KMS grant is a key-scoped permission entity keyed by KeyId and GrantId; it is not a CFN Key",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::LambdaLayer",
        reason: "Lambda layer family has a stable unversioned LayerArn; CFN LayerVersion identifies a specific immutable version, not the family",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OrganizationsCreateAccountRequest",
        reason: "Organizations asynchronous account-creation request identity is distinct from the resulting account and has no exact pinned CFN type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OrganizationsDelegatedAdministrator",
        reason: "Organizations delegated administrator registration is an account-scoped role attachment, distinct from the account itself",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OrganizationsDelegatedService",
        reason: "Organizations service delegation is an account-to-service-principal registration keyed by delegated account and service principal, not a standalone account or any exact pinned CloudFormation resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OrganizationsEnabledServicePrincipal",
        reason: "Organizations trusted-service access is keyed service-principal configuration, not a CFN organization resource",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OrganizationsResponsibilityTransfer",
        reason: "Organizations billing responsibility transfer is an ARN-identified inventory workflow with no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OrganizationsRoot",
        reason: "Organizations root is an ARN-identified organization container with no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::RdsAccountQuota",
        reason: "RDS account quota is AccountQuotaName-keyed configuration, not a database resource",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::RdsDBCapacityReservation",
        reason: "RDS capacity reservation has its own ARN and no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::RdsDBClusterAutomatedBackup",
        reason: "RDS automated cluster backup has a distinct backup ARN and lifecycle; it is not a CFN DBCluster or manual ClusterSnapshot",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::RdsDBInstanceAutomatedBackup",
        reason: "RDS automated instance backup has a distinct backup ARN and lifecycle; it is not a CFN DBInstance or DBSnapshot",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::RdsExportTask",
        reason: "RDS export task is identifier-keyed job metadata, not the source database or exported content",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::RdsPendingMaintenance",
        reason: "RDS pending maintenance is a ResourceIdentifier-keyed maintenance attachment, not a duplicate of the database resource",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::RdsReservedDBInstance",
        reason: "RDS purchased DB-instance reservation has its own ARN and no exact pinned CFN resource type",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::SnsOriginationNumber",
        reason: "SNS dedicated origination phone number is account-owned sender infrastructure identified by PhoneNumber, not a message or CFN Topic",
    },
];
