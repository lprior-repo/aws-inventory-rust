use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "sso-admin",
        operation: "ListTrustedTokenIssuers",
        kind: "AWS::ApiOnly::SSOTrustedTokenIssuer",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "ListGateways",
        kind: "AWS::ApiOnly::StorageGateway",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "ListFileShares",
        kind: "AWS::ApiOnly::StorageGatewayFileShare",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "ListFileSystemAssociations",
        kind: "AWS::ApiOnly::StorageGatewayFileSystemAssociation",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "ListTapePools",
        kind: "AWS::ApiOnly::StorageGatewayTapePool",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "ListTapes",
        kind: "AWS::ApiOnly::StorageGatewayTape",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "ListVolumes",
        kind: "AWS::ApiOnly::StorageGatewayVolume",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "DescribeTapeArchives",
        kind: "AWS::ApiOnly::StorageGatewayTape",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "ListAutomaticTapeCreationPolicies",
        kind: "AWS::ApiOnly::StorageGatewayAutomaticTapePolicy",
    },
    OperationMapping {
        service: "storage-gateway",
        operation: "ListCacheReports",
        kind: "AWS::ApiOnly::StorageGatewayCacheReport",
    },
    OperationMapping {
        service: "supplychain",
        operation: "ListInstances",
        kind: "AWS::ApiOnly::SupplyChainInstance",
    },
    OperationMapping {
        service: "supplychain",
        operation: "ListDataLakeNamespaces",
        kind: "AWS::ApiOnly::SupplyChainDataLakeNamespace",
    },
    OperationMapping {
        service: "supplychain",
        operation: "ListDataLakeDatasets",
        kind: "AWS::ApiOnly::SupplyChainDataLakeDataset",
    },
    OperationMapping {
        service: "support-app",
        operation: "ListSlackChannelConfigurations",
        kind: "AWS::SupportApp::SlackChannelConfiguration",
    },
    OperationMapping {
        service: "support-app",
        operation: "ListSlackWorkspaceConfigurations",
        kind: "AWS::SupportApp::SlackWorkspaceConfiguration",
    },
    OperationMapping {
        service: "synthetics",
        operation: "ListGroups",
        kind: "AWS::Synthetics::Group",
    },
    OperationMapping {
        service: "route-53-domains",
        operation: "ListDomains",
        kind: "AWS::ApiOnly::Route53Domain",
    },
    OperationMapping {
        service: "route53-recovery-readiness",
        operation: "ListCells",
        kind: "AWS::Route53RecoveryReadiness::Cell",
    },
    OperationMapping {
        service: "route53-recovery-readiness",
        operation: "GetCell",
        kind: "AWS::Route53RecoveryReadiness::Cell",
    },
    OperationMapping {
        service: "route53-recovery-readiness",
        operation: "ListReadinessChecks",
        kind: "AWS::Route53RecoveryReadiness::ReadinessCheck",
    },
    OperationMapping {
        service: "route53-recovery-readiness",
        operation: "GetReadinessCheck",
        kind: "AWS::Route53RecoveryReadiness::ReadinessCheck",
    },
    OperationMapping {
        service: "route53-recovery-readiness",
        operation: "ListRecoveryGroups",
        kind: "AWS::Route53RecoveryReadiness::RecoveryGroup",
    },
    OperationMapping {
        service: "route53-recovery-readiness",
        operation: "GetRecoveryGroup",
        kind: "AWS::Route53RecoveryReadiness::RecoveryGroup",
    },
    OperationMapping {
        service: "route53-recovery-readiness",
        operation: "ListResourceSets",
        kind: "AWS::Route53RecoveryReadiness::ResourceSet",
    },
    OperationMapping {
        service: "route53-recovery-readiness",
        operation: "GetResourceSet",
        kind: "AWS::Route53RecoveryReadiness::ResourceSet",
    },
    OperationMapping {
        service: "s3-control",
        operation: "ListMultiRegionAccessPoints",
        kind: "AWS::S3::MultiRegionAccessPoint",
    },
    OperationMapping {
        service: "s3-control",
        operation: "ListAccessPointsForDirectoryBuckets",
        kind: "AWS::S3::AccessPoint",
    },
    OperationMapping {
        service: "sqs",
        operation: "ListQueues",
        kind: "AWS::SQS::Queue",
    },
    OperationMapping {
        service: "sqs",
        operation: "GetQueueAttributes",
        kind: "AWS::SQS::Queue",
    },
    OperationMapping {
        service: "sns",
        operation: "ListSubscriptions",
        kind: "AWS::SNS::Subscription",
    },
    OperationMapping {
        service: "sns",
        operation: "ListSubscriptionsByTopic",
        kind: "AWS::SNS::Subscription",
    },
    OperationMapping {
        service: "sns",
        operation: "GetSubscriptionAttributes",
        kind: "AWS::SNS::Subscription",
    },
    OperationMapping {
        service: "sns",
        operation: "GetTopicAttributes",
        kind: "AWS::SNS::Topic",
    },
    OperationMapping {
        service: "sns",
        operation: "GetDataProtectionPolicy",
        kind: "AWS::SNS::Topic",
    },
    OperationMapping {
        service: "route-53",
        operation: "GetHostedZone",
        kind: "AWS::Route53::HostedZone",
    },
    OperationMapping {
        service: "route-53",
        operation: "GetReusableDelegationSet",
        kind: "AWS::Route53::DelegationSet",
    },
    OperationMapping {
        service: "route-53",
        operation: "ListTrafficPolicyInstancesByHostedZone",
        kind: "AWS::Route53::TrafficPolicyInstance",
    },
];
