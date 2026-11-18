use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudWatchOmniView",
        reason: "CloudWatch Omni named views are telemetry-view definitions, not CloudWatch dashboards or resource explorer views; no matching pinned Omni view type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeBuildBuild",
        reason: "A CodeBuild build ID identifies an existing build execution, not its Project or Fleet definition; build logs and environment details are not fetched.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeBuildBuildBatch",
        reason: "A CodeBuild build-batch ID identifies an existing execution group, not its Project or Fleet definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeBuildReport",
        reason: "A CodeBuild report ARN identifies an existing test report, not its ReportGroup definition; report contents are not fetched.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeBuildSandbox",
        reason: "A CodeBuild sandbox ID identifies an existing sandbox execution, not the build Project or Fleet definition; command output is not fetched.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeCommitApprovalRuleTemplate",
        reason: "CodeCommit approval-rule templates are account-level reusable review rules, not repositories; no exact pinned approval-template type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeDeployDeployment",
        reason: "A CodeDeploy deployment ID identifies an existing deployment execution, not its Application or DeploymentGroup definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeDeployOnPremisesInstance",
        reason: "CodeDeploy on-premises instance registrations describe external machines rather than EC2 instances; no exact pinned registration type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeGuruSecurityScan",
        reason: "CodeGuru Security scan summaries are existing analysis executions, not CodeGuru Reviewer repository associations; findings and source files are not fetched.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CodeStarConnectionsHost",
        reason: "CodeConnections provider host metadata keyed by HostArn represents a provider host installation, not a Connection; no exact pinned CFN host resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CognitoSyncIdentityPoolUsage",
        reason: "Cognito Sync identity-pool storage/session counters are usage observations, not the Cognito IdentityPool definition or its dataset contents.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendDataset",
        reason: "Comprehend dataset metadata identifies a registered flywheel dataset, not the classifier/recognizer or underlying documents; no exact pinned dataset type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendDocumentClassificationJob",
        reason: "Comprehend DocumentClassificationJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendDominantLanguageDetectionJob",
        reason: "Comprehend DominantLanguageDetectionJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendEndpoint",
        reason: "Comprehend real-time model endpoint metadata keyed by EndpointArn is a deployed inference endpoint, not its classifier/recognizer; no pinned CFN endpoint type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendEntitiesDetectionJob",
        reason: "Comprehend EntitiesDetectionJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendEventsDetectionJob",
        reason: "Comprehend EventsDetectionJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendKeyPhrasesDetectionJob",
        reason: "Comprehend KeyPhrasesDetectionJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendMedicalEntitiesDetectionJob",
        reason: "ComprehendMedical EntitiesDetectionJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ComprehendMedicalICD10CMInferenceJob",
        reason: "ComprehendMedical ICD10CMInferenceJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
];
