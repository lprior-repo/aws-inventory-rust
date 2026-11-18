use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "codeguru-security",
        operation: "ListScans",
        kind: "AWS::ApiOnly::CodeGuruSecurityScan",
    },
    OperationMapping {
        service: "codeguruprofiler",
        operation: "DescribeProfilingGroup",
        kind: "AWS::CodeGuruProfiler::ProfilingGroup",
    },
    OperationMapping {
        service: "codeguruprofiler",
        operation: "ListProfilingGroups",
        kind: "AWS::CodeGuruProfiler::ProfilingGroup",
    },
    OperationMapping {
        service: "codepipeline",
        operation: "ListPipelines",
        kind: "AWS::CodePipeline::Pipeline",
    },
    OperationMapping {
        service: "codepipeline",
        operation: "ListWebhooks",
        kind: "AWS::CodePipeline::Webhook",
    },
    OperationMapping {
        service: "codestar-connections",
        operation: "GetConnection",
        kind: "AWS::CodeStarConnections::Connection",
    },
    OperationMapping {
        service: "codestar-connections",
        operation: "GetRepositoryLink",
        kind: "AWS::CodeStarConnections::RepositoryLink",
    },
    OperationMapping {
        service: "codestar-connections",
        operation: "ListConnections",
        kind: "AWS::CodeStarConnections::Connection",
    },
    OperationMapping {
        service: "codestar-connections",
        operation: "ListHosts",
        kind: "AWS::ApiOnly::CodeStarConnectionsHost",
    },
    OperationMapping {
        service: "codestar-connections",
        operation: "ListRepositoryLinks",
        kind: "AWS::CodeStarConnections::RepositoryLink",
    },
    OperationMapping {
        service: "codestar-notifications",
        operation: "DescribeNotificationRule",
        kind: "AWS::CodeStarNotifications::NotificationRule",
    },
    OperationMapping {
        service: "codestar-notifications",
        operation: "ListNotificationRules",
        kind: "AWS::CodeStarNotifications::NotificationRule",
    },
    OperationMapping {
        service: "cognito-identity-provider",
        operation: "ListUserPoolClients",
        kind: "AWS::Cognito::UserPoolClient",
    },
    OperationMapping {
        service: "cognito-identity-provider",
        operation: "ListUserPools",
        kind: "AWS::Cognito::UserPool",
    },
    OperationMapping {
        service: "cognito-identity",
        operation: "DescribeIdentityPool",
        kind: "AWS::Cognito::IdentityPool",
    },
    OperationMapping {
        service: "cognito-identity",
        operation: "ListIdentityPools",
        kind: "AWS::Cognito::IdentityPool",
    },
    OperationMapping {
        service: "cognito-sync",
        operation: "ListIdentityPoolUsage",
        kind: "AWS::ApiOnly::CognitoSyncIdentityPoolUsage",
    },
    OperationMapping {
        service: "comprehend",
        operation: "ListDatasets",
        kind: "AWS::ApiOnly::ComprehendDataset",
    },
    OperationMapping {
        service: "comprehend",
        operation: "ListDocumentClassificationJobs",
        kind: "AWS::ApiOnly::ComprehendDocumentClassificationJob",
    },
    OperationMapping {
        service: "comprehend",
        operation: "ListDocumentClassifiers",
        kind: "AWS::Comprehend::DocumentClassifier",
    },
];
