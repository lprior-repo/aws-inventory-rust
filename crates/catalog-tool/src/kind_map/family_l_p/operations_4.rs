use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "medialive",
        operation: "ListCloudWatchAlarmTemplateGroups",
        kind: "AWS::MediaLive::CloudWatchAlarmTemplateGroup",
    },
    OperationMapping {
        service: "medialive",
        operation: "ListEventBridgeRuleTemplates",
        kind: "AWS::MediaLive::EventBridgeRuleTemplate",
    },
    OperationMapping {
        service: "medialive",
        operation: "ListEventBridgeRuleTemplateGroups",
        kind: "AWS::MediaLive::EventBridgeRuleTemplateGroup",
    },
    OperationMapping {
        service: "mediapackage-vod",
        operation: "ListPackagingConfigurations",
        kind: "AWS::MediaPackage::PackagingConfiguration",
    },
    OperationMapping {
        service: "mediapackage-vod",
        operation: "ListPackagingGroups",
        kind: "AWS::MediaPackage::PackagingGroup",
    },
    OperationMapping {
        service: "mediapackage",
        operation: "ListOriginEndpoints",
        kind: "AWS::MediaPackage::OriginEndpoint",
    },
    OperationMapping {
        service: "mediapackagev2",
        operation: "ListChannelGroups",
        kind: "AWS::MediaPackageV2::ChannelGroup",
    },
    OperationMapping {
        service: "mediapackagev2",
        operation: "ListChannels",
        kind: "AWS::MediaPackageV2::Channel",
    },
    OperationMapping {
        service: "mediapackagev2",
        operation: "ListOriginEndpoints",
        kind: "AWS::MediaPackageV2::OriginEndpoint",
    },
    OperationMapping {
        service: "mediastore",
        operation: "ListContainers",
        kind: "AWS::MediaStore::Container",
    },
    OperationMapping {
        service: "mediatailor",
        operation: "ListChannels",
        kind: "AWS::MediaTailor::Channel",
    },
    OperationMapping {
        service: "mediatailor",
        operation: "ListSourceLocations",
        kind: "AWS::MediaTailor::SourceLocation",
    },
    OperationMapping {
        service: "medical-imaging",
        operation: "ListDatastores",
        kind: "AWS::HealthImaging::Datastore",
    },
    OperationMapping {
        service: "memorydb",
        operation: "DescribeClusters",
        kind: "AWS::MemoryDB::Cluster",
    },
    OperationMapping {
        service: "memorydb",
        operation: "DescribeMultiRegionClusters",
        kind: "AWS::MemoryDB::MultiRegionCluster",
    },
    OperationMapping {
        service: "memorydb",
        operation: "DescribeParameterGroups",
        kind: "AWS::MemoryDB::ParameterGroup",
    },
    OperationMapping {
        service: "memorydb",
        operation: "DescribeSubnetGroups",
        kind: "AWS::MemoryDB::SubnetGroup",
    },
    OperationMapping {
        service: "memorydb",
        operation: "DescribeACLs",
        kind: "AWS::MemoryDB::ACL",
    },
    OperationMapping {
        service: "memorydb",
        operation: "DescribeUsers",
        kind: "AWS::MemoryDB::User",
    },
    OperationMapping {
        service: "memorydb",
        operation: "DescribeSnapshots",
        kind: "AWS::MemoryDB::Snapshot",
    },
];
