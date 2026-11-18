use super::{DenyList, Entry};
use crate::classes::DenyReason;

// Same fail-closed global and scoped rules reviewed against pinned AWS models.
// Original positions remain stable identifiers for unmatched-policy diagnostics.
pub(super) fn list() -> DenyList<'static> {
    DenyList(vec![
        Entry {
            key: "GetAuthorizationToken",
            reason: DenyReason::Secret,
            line: 21,
        },
        Entry {
            key: "GetCostAndUsage",
            reason: DenyReason::Paid,
            line: 22,
        },
        Entry {
            key: "GetFunction",
            reason: DenyReason::Content,
            line: 23,
        },
        Entry {
            key: "GetId",
            reason: DenyReason::Mutating,
            line: 24,
        },
        Entry {
            key: "GetLogEvents",
            reason: DenyReason::Content,
            line: 25,
        },
        Entry {
            key: "GetMetricData",
            reason: DenyReason::Paid,
            line: 26,
        },
        Entry {
            key: "GetObject",
            reason: DenyReason::Content,
            line: 27,
        },
        Entry {
            key: "GetParameter",
            reason: DenyReason::Secret,
            line: 28,
        },
        Entry {
            key: "GetPasswordData",
            reason: DenyReason::Secret,
            line: 29,
        },
        Entry {
            key: "GetRecords",
            reason: DenyReason::Content,
            line: 30,
        },
        Entry {
            key: "GetSecretValue",
            reason: DenyReason::Secret,
            line: 31,
        },
        Entry {
            key: "GetSessionToken",
            reason: DenyReason::Secret,
            line: 32,
        },
        Entry {
            key: "Query",
            reason: DenyReason::Content,
            line: 33,
        },
        Entry {
            key: "ReceiveMessage",
            reason: DenyReason::Mutating,
            line: 34,
        },
        Entry {
            key: "Scan",
            reason: DenyReason::Content,
            line: 35,
        },
        Entry {
            key: "bedrock-agent-runtime/ListFlowExecutionEvents",
            reason: DenyReason::Content,
            line: 37,
        },
        Entry {
            key: "bedrock-agentcore/ListEvents",
            reason: DenyReason::Content,
            line: 39,
        },
        Entry {
            key: "bedrock-agentcore/ListMemoryRecords",
            reason: DenyReason::Content,
            line: 41,
        },
        Entry {
            key: "chime-sdk-meetings/ListAttendees",
            reason: DenyReason::Secret,
            line: 43,
        },
        Entry {
            key: "chime-sdk-messaging/ListChannelMessages",
            reason: DenyReason::Content,
            line: 45,
        },
        Entry {
            key: "clouddirectory/ListObjectAttributes",
            reason: DenyReason::Content,
            line: 47,
        },
        Entry {
            key: "cloudfront-keyvaluestore/ListKeys",
            reason: DenyReason::Content,
            line: 49,
        },
        Entry {
            key: "cloudwatch-logs/ListAnomalies",
            reason: DenyReason::Content,
            line: 51,
        },
        Entry {
            key: "codebuild/ListCommandExecutionsForSandbox",
            reason: DenyReason::Content,
            line: 53,
        },
        Entry {
            key: "cognito-identity-provider/DescribeUserImportJob",
            reason: DenyReason::Secret,
            line: 55,
        },
        Entry {
            key: "cognito-identity-provider/DescribeUserPoolClient",
            reason: DenyReason::Secret,
            line: 57,
        },
        Entry {
            key: "cognito-identity-provider/ListUserImportJobs",
            reason: DenyReason::Secret,
            line: 59,
        },
        Entry {
            key: "cognito-sync/ListRecords",
            reason: DenyReason::Content,
            line: 61,
        },
        Entry {
            key: "connect-contact-lens/ListRealtimeContactAnalysisSegments",
            reason: DenyReason::Content,
            line: 63,
        },
        Entry {
            key: "connect/ListDataTablePrimaryValues",
            reason: DenyReason::Content,
            line: 65,
        },
        Entry {
            key: "connect/ListDataTableValues",
            reason: DenyReason::Content,
            line: 67,
        },
        Entry {
            key: "connect/ListRealtimeContactAnalysisSegmentsV2",
            reason: DenyReason::Content,
            line: 69,
        },
        Entry {
            key: "database-migration-service/DescribeEndpoints",
            reason: DenyReason::Secret,
            line: 72,
        },
        Entry {
            key: "datasync/DescribeLocationFsxOntap",
            reason: DenyReason::Secret,
            line: 75,
        },
        Entry {
            key: "datasync/DescribeLocationFsxOpenZfs",
            reason: DenyReason::Secret,
            line: 78,
        },
        Entry {
            key: "datazone/ListConnections",
            reason: DenyReason::Secret,
            line: 81,
        },
        Entry {
            key: "device-farm/ListArtifacts",
            reason: DenyReason::Content,
            line: 83,
        },
        Entry {
            key: "device-farm/ListSamples",
            reason: DenyReason::Content,
            line: 85,
        },
        Entry {
            key: "device-farm/ListUploads",
            reason: DenyReason::Secret,
            line: 88,
        },
        Entry {
            key: "devops-agent/ListJournalRecords",
            reason: DenyReason::Content,
            line: 90,
        },
        Entry {
            key: "devops-agent/ListPendingMessages",
            reason: DenyReason::Content,
            line: 92,
        },
        Entry {
            key: "ec2/DescribeConversionTasks",
            reason: DenyReason::Content,
            line: 94,
        },
        Entry {
            key: "ec2/DescribeInstanceTypes",
            reason: DenyReason::ReferenceData,
            line: 96,
        },
        Entry {
            key: "ec2/DescribeVerifiedAccessTrustProviders",
            reason: DenyReason::Secret,
            line: 99,
        },
        Entry {
            key: "gamelift/DescribeFleetEvents",
            reason: DenyReason::Content,
            line: 102,
        },
        Entry {
            key: "glue/ListStatements",
            reason: DenyReason::Content,
            line: 104,
        },
        Entry {
            key: "iot-data-plane/ListRetainedMessages",
            reason: DenyReason::Paid,
            line: 106,
        },
        Entry {
            key: "kms/DescribeKey",
            reason: DenyReason::Mutating,
            line: 109,
        },
        Entry {
            key: "lex-models-v2/DescribeBotRecommendation",
            reason: DenyReason::Content,
            line: 111,
        },
        Entry {
            key: "lex-models-v2/DescribeExport",
            reason: DenyReason::Content,
            line: 113,
        },
        Entry {
            key: "lex-models-v2/DescribeTestSetDiscrepancyReport",
            reason: DenyReason::Content,
            line: 115,
        },
        Entry {
            key: "lex-models-v2/ListUtteranceAnalyticsData",
            reason: DenyReason::Content,
            line: 117,
        },
        Entry {
            key: "mailmanager/ListAddressListImportJobs",
            reason: DenyReason::Secret,
            line: 119,
        },
        Entry {
            key: "medialive/DescribeInputDeviceThumbnail",
            reason: DenyReason::Content,
            line: 121,
        },
        Entry {
            key: "medialive/DescribeThumbnails",
            reason: DenyReason::Content,
            line: 123,
        },
        Entry {
            key: "mediapackage/DescribeChannel",
            reason: DenyReason::Secret,
            line: 125,
        },
        Entry {
            key: "mediapackage/ListChannels",
            reason: DenyReason::Secret,
            line: 127,
        },
        Entry {
            key: "qapps/ListQAppSessionData",
            reason: DenyReason::Content,
            line: 129,
        },
        Entry {
            key: "qbusiness/ListMessages",
            reason: DenyReason::Content,
            line: 131,
        },
        Entry {
            key: "qconnect/ListMessages",
            reason: DenyReason::Content,
            line: 133,
        },
        Entry {
            key: "qconnect/ListSpans",
            reason: DenyReason::Content,
            line: 135,
        },
        Entry {
            key: "quicksight/DescribeAssetBundleExportJob",
            reason: DenyReason::Content,
            line: 137,
        },
        Entry {
            key: "quicksight/DescribeAssetBundleImportJob",
            reason: DenyReason::Content,
            line: 139,
        },
        Entry {
            key: "rds/DescribeDBEngineVersions",
            reason: DenyReason::ReferenceData,
            line: 141,
        },
        Entry {
            key: "redshift-data/DescribeStatement",
            reason: DenyReason::Content,
            line: 143,
        },
        Entry {
            key: "redshift-data/ListStatements",
            reason: DenyReason::Content,
            line: 145,
        },
        Entry {
            key: "rekognition/ListDatasetEntries",
            reason: DenyReason::Content,
            line: 147,
        },
        Entry {
            key: "s3vectors/ListVectors",
            reason: DenyReason::Content,
            line: 149,
        },
        Entry {
            key: "sagemaker/DescribeModelCard",
            reason: DenyReason::Content,
            line: 151,
        },
        Entry {
            key: "security-ir/ListComments",
            reason: DenyReason::Content,
            line: 153,
        },
        Entry {
            key: "security-ir/ListInvestigations",
            reason: DenyReason::Content,
            line: 155,
        },
        Entry {
            key: "securityagent/ListActorMessages",
            reason: DenyReason::Secret,
            line: 157,
        },
        Entry {
            key: "sfn/DescribeExecution",
            reason: DenyReason::Content,
            line: 159,
        },
        Entry {
            key: "snowball/DescribeJob",
            reason: DenyReason::Content,
            line: 161,
        },
        Entry {
            key: "snowball/DescribeReturnShippingLabel",
            reason: DenyReason::Content,
            line: 163,
        },
        Entry {
            key: "socialmessaging/ListWhatsAppFlowAssets",
            reason: DenyReason::Content,
            line: 165,
        },
        Entry {
            key: "ssm-contacts/DescribeEngagement",
            reason: DenyReason::Content,
            line: 167,
        },
        Entry {
            key: "ssm-contacts/DescribePage",
            reason: DenyReason::Content,
            line: 169,
        },
        Entry {
            key: "ssm/ListCommandInvocations",
            reason: DenyReason::Content,
            line: 171,
        },
        Entry {
            key: "storage-gateway/DescribeChapCredentials",
            reason: DenyReason::Secret,
            line: 173,
        },
        Entry {
            key: "support/DescribeAttachment",
            reason: DenyReason::Content,
            line: 175,
        },
        Entry {
            key: "support/DescribeCases",
            reason: DenyReason::Content,
            line: 178,
        },
        Entry {
            key: "support/DescribeCommunications",
            reason: DenyReason::Content,
            line: 180,
        },
        Entry {
            key: "workdocs/DescribeComments",
            reason: DenyReason::Content,
            line: 182,
        },
        Entry {
            key: "xray/ListRetrievedTraces",
            reason: DenyReason::Content,
            line: 184,
        },
    ])
}
