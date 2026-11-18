use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DRSRecoveryInstance",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DRSRecoveryPlan",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DRSRecoveryPlanExecution",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DRSSourceNetwork",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DRSSourceServer",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DynamoDBStream",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ECRPublicRegistry",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ECRReplicationBackfill",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EFSReplicationConfiguration",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElastiCacheServerlessCacheSnapshot",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElastiCacheSnapshot",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElasticsearchInboundCrossClusterConnection",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElasticsearchOutboundCrossClusterConnection",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElasticsearchPackage",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElasticsearchVpcEndpoint",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElementalInferenceDictionary",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ElementalInferenceFeed",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EMRContainersSecurityConfiguration",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EndUserMessagingBrandProfile",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventBridgeV2EventBus",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
];
