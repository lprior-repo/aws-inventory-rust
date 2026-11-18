use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DataExchangeDataGrant",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DataExchangeDataSet",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DataExchangeReceivedDataGrant",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DataSyncLocation",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DataSyncTaskExecution",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DeviceFarmDeviceInstance",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DeviceFarmInstanceProfile",
        reason: "DeviceFarm reboot and package-cleanup configuration is distinct from an EC2 IAM instance profile; no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DeviceFarmTestGridProject",
        reason: "Selenium test-grid project is distinct from a DeviceFarm Project; no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DeviceFarmVPCEConfiguration",
        reason: "DeviceFarm private endpoint routing configuration is distinct from an EC2 VPCEndpoint; no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DevOpsAgentAgentSpace",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DevOpsAgentPrivateConnection",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DirectConnectInterconnect",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DirectConnectResiliencyGroup",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DirectoryServiceAssessment",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DirectoryServiceLogSubscription",
        reason: "Per-directory CloudWatch Logs delivery subscription keyed by DirectoryId and LogGroupName is not a Logs SubscriptionFilter; no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DirectoryServiceSnapshot",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DirectoryServiceTrust",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DocDBDBClusterSnapshot",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DocDBElasticClusterSnapshot",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DRSJob",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
];
