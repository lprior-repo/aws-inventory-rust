use super::*;
use crate::kind_map::{
    KindTarget, ReviewedRelation, ReviewedRelationSource, ReviewedRelationTarget, ReviewedValuePath,
};
use crate::names::Exceptions;
use sha2::{Digest, Sha256};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn source_fields(row: &ReviewedRelation) -> Option<&str> {
    match &row.source {
        ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(path))
        | ReviewedRelationSource::Request(ReviewedValuePath::Fields(path)) => Some(path),
        _ => None,
    }
}

fn target_fields(row: &ReviewedRelation) -> Option<&str> {
    match &row.target {
        ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields(path)) => Some(path),
        _ => None,
    }
}

fn scalar_relation_row(row: &ReviewedRelation) -> Result<String, Box<dyn std::error::Error>> {
    Ok(format!(
        "{}\t{}\t{}\t{}\t{}\t{}\n",
        row.source_kind,
        source_fields(row).ok_or("legacy relation lost its modeled source")?,
        row.target_kinds.primary(),
        target_fields(row).ok_or("legacy relation lost its modeled target")?,
        row.relation,
        row.scope
    ))
}

fn digest(rows: impl Iterator<Item = String>) -> String {
    let mut rows = rows.collect::<Vec<_>>();
    rows.sort();
    Sha256::digest(rows.concat().as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

const NEW_SAFE_METADATA_OPERATIONS: &[(&str, &str, &str)] = &[
    (
        "kafkaconnect",
        "ListConnectors",
        "AWS::KafkaConnect::Connector",
    ),
    (
        "kafkaconnect",
        "ListCustomPlugins",
        "AWS::KafkaConnect::CustomPlugin",
    ),
    (
        "kafkaconnect",
        "ListWorkerConfigurations",
        "AWS::KafkaConnect::WorkerConfiguration",
    ),
    (
        "kafkaconnect",
        "DescribeCustomPlugin",
        "AWS::KafkaConnect::CustomPlugin",
    ),
    (
        "kafkaconnect",
        "ListConnectorOperations",
        "AWS::ApiOnly::KafkaConnectConnectorOperation",
    ),
    ("kinesis-video", "ListStreams", "AWS::KinesisVideo::Stream"),
    (
        "kinesis-video",
        "ListSignalingChannels",
        "AWS::KinesisVideo::SignalingChannel",
    ),
    (
        "kinesis-video",
        "DescribeMappedResourceConfiguration",
        "AWS::ApiOnly::KinesisVideoMappedResourceConfiguration",
    ),
    (
        "timestream-write",
        "ListDatabases",
        "AWS::Timestream::Database",
    ),
    (
        "timestream-write",
        "DescribeDatabase",
        "AWS::Timestream::Database",
    ),
    ("timestream-write", "ListTables", "AWS::Timestream::Table"),
    (
        "timestream-write",
        "DescribeTable",
        "AWS::Timestream::Table",
    ),
    (
        "timestream-write",
        "ListBatchLoadTasks",
        "AWS::ApiOnly::TimestreamBatchLoadTask",
    ),
    (
        "timestream-write",
        "DescribeBatchLoadTask",
        "AWS::ApiOnly::TimestreamBatchLoadTask",
    ),
    (
        "timestream-query",
        "ListScheduledQueries",
        "AWS::Timestream::ScheduledQuery",
    ),
    (
        "timestream-query",
        "DescribeScheduledQuery",
        "AWS::Timestream::ScheduledQuery",
    ),
    ("wafv2", "ListIPSets", "AWS::WAFv2::IPSet"),
    ("wafv2", "GetIPSet", "AWS::WAFv2::IPSet"),
    (
        "wafv2",
        "ListLoggingConfigurations",
        "AWS::WAFv2::LoggingConfiguration",
    ),
    (
        "wafv2",
        "GetLoggingConfiguration",
        "AWS::WAFv2::LoggingConfiguration",
    ),
    (
        "wafv2",
        "ListManagedRuleSets",
        "AWS::ApiOnly::Wafv2ManagedRuleSet",
    ),
    (
        "wafv2",
        "GetManagedRuleSet",
        "AWS::ApiOnly::Wafv2ManagedRuleSet",
    ),
    (
        "wafv2",
        "ListRegexPatternSets",
        "AWS::WAFv2::RegexPatternSet",
    ),
    ("wafv2", "GetRegexPatternSet", "AWS::WAFv2::RegexPatternSet"),
    ("wafv2", "ListRuleGroups", "AWS::WAFv2::RuleGroup"),
    ("wafv2", "GetRuleGroup", "AWS::WAFv2::RuleGroup"),
    ("wafv2", "ListWebACLs", "AWS::WAFv2::WebACL"),
    ("wafv2", "GetWebACL", "AWS::WAFv2::WebACL"),
    ("transfer", "ListAccesses", "AWS::ApiOnly::TransferAccess"),
    ("transfer", "DescribeAccess", "AWS::ApiOnly::TransferAccess"),
    ("transfer", "DescribeUser", "AWS::Transfer::User"),
    ("transfer", "DescribeHostKey", "AWS::Transfer::HostKey"),
    (
        "workspaces-web",
        "ListTrustStoreCertificates",
        "AWS::ApiOnly::WorkSpacesWebTrustStoreCertificate",
    ),
    (
        "workspaces-web",
        "GetTrustStoreCertificate",
        "AWS::ApiOnly::WorkSpacesWebTrustStoreCertificate",
    ),
    (
        "route-53-domains",
        "ListDomains",
        "AWS::ApiOnly::Route53Domain",
    ),
    (
        "route53-recovery-readiness",
        "ListCells",
        "AWS::Route53RecoveryReadiness::Cell",
    ),
    (
        "route53-recovery-readiness",
        "GetCell",
        "AWS::Route53RecoveryReadiness::Cell",
    ),
    (
        "route53-recovery-readiness",
        "ListReadinessChecks",
        "AWS::Route53RecoveryReadiness::ReadinessCheck",
    ),
    (
        "route53-recovery-readiness",
        "GetReadinessCheck",
        "AWS::Route53RecoveryReadiness::ReadinessCheck",
    ),
    (
        "route53-recovery-readiness",
        "ListRecoveryGroups",
        "AWS::Route53RecoveryReadiness::RecoveryGroup",
    ),
    (
        "route53-recovery-readiness",
        "GetRecoveryGroup",
        "AWS::Route53RecoveryReadiness::RecoveryGroup",
    ),
    (
        "route53-recovery-readiness",
        "ListResourceSets",
        "AWS::Route53RecoveryReadiness::ResourceSet",
    ),
    (
        "route53-recovery-readiness",
        "GetResourceSet",
        "AWS::Route53RecoveryReadiness::ResourceSet",
    ),
    (
        "s3-control",
        "ListMultiRegionAccessPoints",
        "AWS::S3::MultiRegionAccessPoint",
    ),
    (
        "s3-control",
        "ListAccessPointsForDirectoryBuckets",
        "AWS::S3::AccessPoint",
    ),
    ("sqs", "GetQueueAttributes", "AWS::SQS::Queue"),
    ("sns", "ListSubscriptions", "AWS::SNS::Subscription"),
    ("sns", "ListSubscriptionsByTopic", "AWS::SNS::Subscription"),
    ("sns", "GetSubscriptionAttributes", "AWS::SNS::Subscription"),
    ("sns", "GetTopicAttributes", "AWS::SNS::Topic"),
    ("sns", "GetDataProtectionPolicy", "AWS::SNS::Topic"),
    ("route-53", "GetHostedZone", "AWS::Route53::HostedZone"),
    (
        "route-53",
        "GetReusableDelegationSet",
        "AWS::Route53::DelegationSet",
    ),
    (
        "route-53",
        "ListTrafficPolicyInstancesByHostedZone",
        "AWS::Route53::TrafficPolicyInstance",
    ),
    (
        "global-accelerator",
        "ListListeners",
        "AWS::GlobalAccelerator::Listener",
    ),
    (
        "global-accelerator",
        "ListEndpointGroups",
        "AWS::GlobalAccelerator::EndpointGroup",
    ),
    (
        "global-accelerator",
        "ListCustomRoutingListeners",
        "AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener",
    ),
    (
        "global-accelerator",
        "ListCustomRoutingEndpointGroups",
        "AWS::ApiOnly::GlobalAcceleratorCustomRoutingEndpointGroup",
    ),
    (
        "accessanalyzer",
        "ListArchiveRules",
        "AWS::AccessAnalyzer::ArchiveRule",
    ),
    (
        "accessanalyzer",
        "GetArchiveRule",
        "AWS::AccessAnalyzer::ArchiveRule",
    ),
    ("iot", "ListCustomMetrics", "AWS::IoT::CustomMetric"),
    ("iot", "DescribeCustomMetric", "AWS::IoT::CustomMetric"),
    ("iot", "ListDimensions", "AWS::IoT::Dimension"),
    (
        "lambda",
        "ListFunctionsByCodeSigningConfig",
        "AWS::Lambda::Function",
    ),
    (
        "organizations",
        "ListDelegatedServicesForAccount",
        "AWS::ApiOnly::OrganizationsDelegatedService",
    ),
    ("iam", "ListAccountAliases", "AWS::ApiOnly::IAMAccountAlias"),
    (
        "iam",
        "ListServiceSpecificCredentials",
        "AWS::ApiOnly::IAMServiceSpecificCredential",
    ),
    ("iam", "ListGroupPolicies", "AWS::IAM::GroupPolicy"),
    ("iam", "ListRolePolicies", "AWS::IAM::RolePolicy"),
    ("iam", "ListUserPolicies", "AWS::IAM::UserPolicy"),
    (
        "iam",
        "ListPolicyVersions",
        "AWS::ApiOnly::IAMPolicyVersion",
    ),
    ("kms", "ListKeyPolicies", "AWS::ApiOnly::KMSKeyPolicy"),
    (
        "s3",
        "ListDirectoryBuckets",
        "AWS::S3Express::DirectoryBucket",
    ),
    ("rds", "DescribeDBSnapshots", "AWS::RDS::DBSnapshot"),
    ("route-53", "GetDNSSEC", "AWS::Route53::DNSSEC"),
    (
        "eventbridge",
        "DescribeEventBus",
        "AWS::ApiOnly::EventBridgeEventBus",
    ),
    (
        "eventbridge",
        "DescribeEventSource",
        "AWS::ApiOnly::EventBridgeEventSource",
    ),
    (
        "eventbridge",
        "DescribeRule",
        "AWS::ApiOnly::EventBridgeEventRule",
    ),
    (
        "eventbridge",
        "DescribeEndpoint",
        "AWS::ApiOnly::EventBridgeEndpoint",
    ),
    (
        "eventbridgev2",
        "DescribeEventBus",
        "AWS::ApiOnly::EventBridgeV2EventBus",
    ),
    (
        "eventbridgev2",
        "DescribeEventSource",
        "AWS::ApiOnly::EventBridgeV2EventSource",
    ),
    (
        "eventbridgev2",
        "DescribeSubscriber",
        "AWS::ApiOnly::EventBridgeV2Subscriber",
    ),
    (
        "sagemaker",
        "DescribeTrainingPlanExtensionHistory",
        "AWS::ApiOnly::SageMakerTrainingPlanExtension",
    ),
    ("sagemaker", "ListAliases", "AWS::ApiOnly::SageMakerAlias"),
    ("sagemaker", "ListApps", "AWS::ApiOnly::SageMakerApp"),
    (
        "sagemaker",
        "ListCandidatesForAutoMLJob",
        "AWS::ApiOnly::SageMakerAutoMLCandidate",
    ),
    (
        "sagemaker",
        "ListClusterEvents",
        "AWS::ApiOnly::SageMakerClusterEvent",
    ),
    (
        "sagemaker",
        "ListClusterExtensions",
        "AWS::ApiOnly::SageMakerClusterExtension",
    ),
    (
        "ssm-contacts",
        "ListContactChannels",
        "AWS::ApiOnly::SSMContactChannel",
    ),
    (
        "ssm-contacts",
        "ListPageReceipts",
        "AWS::ApiOnly::SSMContactPageReceipt",
    ),
    (
        "ssm-contacts",
        "ListPageResolutions",
        "AWS::ApiOnly::SSMContactPageResolution",
    ),
    (
        "ssm-contacts",
        "ListPagesByContact",
        "AWS::ApiOnly::SSMContactPage",
    ),
    (
        "ssm-contacts",
        "ListPagesByEngagement",
        "AWS::ApiOnly::SSMContactPage",
    ),
    (
        "ssm-contacts",
        "ListRotationOverrides",
        "AWS::ApiOnly::SSMContactRotationOverride",
    ),
    (
        "ssm-contacts",
        "ListRotationShifts",
        "AWS::ApiOnly::SSMContactRotationShift",
    ),
    (
        "database-migration-service",
        "DescribeFleetAdvisorCollectors",
        "AWS::ApiOnly::DMSCollector",
    ),
    (
        "database-migration-service",
        "DescribeFleetAdvisorDatabases",
        "AWS::ApiOnly::DMSScannedDatabase",
    ),
    (
        "database-migration-service",
        "DescribePendingMaintenanceActions",
        "AWS::ApiOnly::DMSPendingMaintenanceAction",
    ),
    (
        "database-migration-service",
        "DescribeDataMigrations",
        "AWS::ApiOnly::DMSDataMigration",
    ),
    (
        "migrationhubstrategy",
        "ListAnalyzableServers",
        "AWS::ApiOnly::MigrationStrategyAnalyzableServer",
    ),
    (
        "migrationhubstrategy",
        "ListApplicationComponents",
        "AWS::ApiOnly::MigrationStrategyApplicationComponent",
    ),
    (
        "migrationhubstrategy",
        "ListImportFileTask",
        "AWS::ApiOnly::MigrationStrategyImportFileTask",
    ),
    (
        "migrationhubstrategy",
        "ListServers",
        "AWS::ApiOnly::MigrationStrategyServer",
    ),
    (
        "mediapackage",
        "ListVodAssets",
        "AWS::ApiOnly::MediaPackageVodAsset",
    ),
];
mod baseline;
const CENTRAL_ADDED_CFN_KINDS: &[&str] = &[
    "AWS::Lambda::Function",
    "AWS::IAM::GroupPolicy",
    "AWS::IAM::Role",
    "AWS::IAM::RolePolicy",
    "AWS::IAM::UserPolicy",
    "AWS::S3Express::DirectoryBucket",
    "AWS::RDS::DBSnapshot",
    "AWS::Route53::KeySigningKey",
    "AWS::Route53::DNSSEC",
    "AWS::KafkaConnect::Connector",
    "AWS::KafkaConnect::CustomPlugin",
    "AWS::KafkaConnect::WorkerConfiguration",
    "AWS::KinesisVideo::SignalingChannel",
    "AWS::KinesisVideo::Stream",
    "AWS::DMS::Endpoint",
    "AWS::DMS::ReplicationConfig",
    "AWS::DMS::ReplicationTask",
];
const CENTRAL_ADDED_API_KINDS: &[&str] = &[
    "AWS::ApiOnly::OrganizationsDelegatedService",
    "AWS::ApiOnly::IAMAccountAlias",
    "AWS::ApiOnly::IAMServiceSpecificCredential",
    "AWS::ApiOnly::IAMPolicyVersion",
    "AWS::ApiOnly::KMSKeyPolicy",
    "AWS::ApiOnly::KafkaConnectConnectorOperation",
    "AWS::ApiOnly::KinesisVideoMappedResourceConfiguration",
    "AWS::ApiOnly::EventBridgeEventBus",
    "AWS::ApiOnly::EventBridgeEventSource",
    "AWS::ApiOnly::EventBridgeEventRule",
    "AWS::ApiOnly::EventBridgeEndpoint",
    "AWS::ApiOnly::EventBridgeV2EventBus",
    "AWS::ApiOnly::EventBridgeV2EventSource",
    "AWS::ApiOnly::EventBridgeV2Subscriber",
    "AWS::ApiOnly::SageMakerTrainingPlanExtension",
    "AWS::ApiOnly::SageMakerAlias",
    "AWS::ApiOnly::SageMakerApp",
    "AWS::ApiOnly::SageMakerAutoMLCandidate",
    "AWS::ApiOnly::SageMakerClusterEvent",
    "AWS::ApiOnly::SageMakerClusterExtension",
    "AWS::ApiOnly::SSMContactChannel",
    "AWS::ApiOnly::SSMContactPageReceipt",
    "AWS::ApiOnly::SSMContactPageResolution",
    "AWS::ApiOnly::SSMContactPage",
    "AWS::ApiOnly::SSMContactRotationOverride",
    "AWS::ApiOnly::SSMContactRotationShift",
    "AWS::ApiOnly::DMSCollector",
    "AWS::ApiOnly::DMSScannedDatabase",
    "AWS::ApiOnly::DMSPendingMaintenanceAction",
    "AWS::ApiOnly::DMSDataMigration",
    "AWS::ApiOnly::MigrationStrategyAnalyzableServer",
    "AWS::ApiOnly::MigrationStrategyApplicationComponent",
    "AWS::ApiOnly::MigrationStrategyImportFileTask",
    "AWS::ApiOnly::MigrationStrategyServer",
    "AWS::ApiOnly::MediaPackageVodAsset",
];
mod compilation;
mod operations;
mod relations;
