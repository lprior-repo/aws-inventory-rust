use DenyReason::{Content, Mutating, Paid, ReferenceData, Secret};
use catalog_tool::classes::{Class, DenyList, DenyReason, OperationFacts, classify};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const POLICY: &str = include_str!("../../../catalog/overlay/must-deny.txt");
const VERIFIED: &[(&str, &str, DenyReason)] = &[
    ("bedrock-agent-runtime", "ListFlowExecutionEvents", Content),
    ("bedrock-agentcore", "ListEvents", Content),
    ("bedrock-agentcore", "ListMemoryRecords", Content),
    ("chime-sdk-meetings", "ListAttendees", Secret),
    ("chime-sdk-messaging", "ListChannelMessages", Content),
    ("clouddirectory", "ListObjectAttributes", Content),
    ("cloudfront-keyvaluestore", "ListKeys", Content),
    ("cloudwatch-logs", "ListAnomalies", Content),
    ("codebuild", "ListCommandExecutionsForSandbox", Content),
    ("cognito-identity-provider", "DescribeUserImportJob", Secret),
    (
        "cognito-identity-provider",
        "DescribeUserPoolClient",
        Secret,
    ),
    ("cognito-identity-provider", "ListUserImportJobs", Secret),
    ("cognito-sync", "ListRecords", Content),
    (
        "connect-contact-lens",
        "ListRealtimeContactAnalysisSegments",
        Content,
    ),
    ("connect", "ListDataTablePrimaryValues", Content),
    ("connect", "ListDataTableValues", Content),
    ("connect", "ListRealtimeContactAnalysisSegmentsV2", Content),
    ("device-farm", "ListArtifacts", Content),
    ("device-farm", "ListSamples", Content),
    ("device-farm", "ListUploads", Secret),
    ("devops-agent", "ListJournalRecords", Content),
    ("devops-agent", "ListPendingMessages", Content),
    ("ec2", "DescribeConversionTasks", Content),
    ("ec2", "DescribeInstanceTypes", ReferenceData),
    ("gamelift", "DescribeFleetEvents", Content),
    ("glue", "ListStatements", Content),
    ("iot-data-plane", "ListRetainedMessages", Paid),
    ("kms", "DescribeKey", Mutating),
    ("lex-models-v2", "DescribeBotRecommendation", Content),
    ("lex-models-v2", "DescribeExport", Content),
    ("lex-models-v2", "DescribeTestSetDiscrepancyReport", Content),
    ("lex-models-v2", "ListUtteranceAnalyticsData", Content),
    ("mailmanager", "ListAddressListImportJobs", Secret),
    ("medialive", "DescribeInputDeviceThumbnail", Content),
    ("medialive", "DescribeThumbnails", Content),
    ("mediapackage", "DescribeChannel", Secret),
    ("mediapackage", "ListChannels", Secret),
    ("qapps", "ListQAppSessionData", Content),
    ("qbusiness", "ListMessages", Content),
    ("qconnect", "ListMessages", Content),
    ("qconnect", "ListSpans", Content),
    ("quicksight", "DescribeAssetBundleExportJob", Content),
    ("quicksight", "DescribeAssetBundleImportJob", Content),
    ("rds", "DescribeDBEngineVersions", ReferenceData),
    ("redshift-data", "DescribeStatement", Content),
    ("redshift-data", "ListStatements", Content),
    ("rekognition", "ListDatasetEntries", Content),
    ("s3vectors", "ListVectors", Content),
    ("sagemaker", "DescribeModelCard", Content),
    ("security-ir", "ListComments", Content),
    ("security-ir", "ListInvestigations", Content),
    ("securityagent", "ListActorMessages", Secret),
    ("sfn", "DescribeExecution", Content),
    ("snowball", "DescribeJob", Content),
    ("snowball", "DescribeReturnShippingLabel", Content),
    ("socialmessaging", "ListWhatsAppFlowAssets", Content),
    ("ssm-contacts", "DescribeEngagement", Content),
    ("ssm-contacts", "DescribePage", Content),
    ("ssm", "ListCommandInvocations", Content),
    ("storage-gateway", "DescribeChapCredentials", Secret),
    ("support", "DescribeAttachment", Content),
    ("support", "DescribeCases", Content),
    ("support", "DescribeCommunications", Content),
    ("workdocs", "DescribeComments", Content),
    ("xray", "ListRetrievedTraces", Content),
];
const UNVERIFIED: &[(&str, &str, DenyReason)] = &[
    ("database-migration-service", "DescribeEndpoints", Secret),
    ("datasync", "DescribeLocationFsxOntap", Secret),
    ("datasync", "DescribeLocationFsxOpenZfs", Secret),
    ("ec2", "DescribeVerifiedAccessTrustProviders", Secret),
    ("datazone", "ListConnections", Secret),
];
const NOT_DENIED: &[(&str, &str)] = &[
    ("codebuild", "ListSourceCredentials"),
    ("kafka", "ListScramSecrets"),
    ("cognito-identity-provider", "ListUserPoolClientSecrets"),
    ("appstream", "DescribeDirectoryConfigs"),
    (
        "chime-sdk-voice",
        "ListVoiceConnectorTerminationCredentials",
    ),
    ("ec2", "DescribeImages"),
];

fn assert_denials(entries: &[(&str, &str, DenyReason)]) -> TestResult {
    let deny = DenyList::parse(POLICY)?;
    entries.iter().for_each(|(service, name, reason)| {
        let facts = OperationFacts {
            service,
            name,
            has_required_input: false,
            readonly: true,
        };
        let required = OperationFacts {
            has_required_input: true,
            ..facts
        };
        let other = OperationFacts {
            service: "unrelated-service",
            ..facts
        };
        assert_eq!(
            classify(&facts, &deny),
            Class::Denied(*reason),
            "{service}/{name}"
        );
        assert_eq!(
            classify(&required, &deny),
            Class::Denied(*reason),
            "{service}/{name}"
        );
        assert_eq!(
            classify(&other, &deny),
            Class::Enumerate,
            "{service}/{name}"
        );
    });
    Ok(())
}

#[test]
fn documentation_verified_list_describe_denials_are_service_scoped() -> TestResult {
    assert_eq!(VERIFIED.len(), 65);
    assert_denials(VERIFIED)
}

#[test]
fn unverified_credential_shaped_outputs_stay_denied_pending_overlay_review() -> TestResult {
    assert_eq!(UNVERIFIED.len(), 5);
    assert_denials(UNVERIFIED)
}

#[test]
fn reviewed_metadata_false_positives_and_customer_images_remain_candidates() -> TestResult {
    let deny = DenyList::parse(POLICY)?;
    NOT_DENIED.iter().for_each(|(service, name)| {
        let facts = OperationFacts {
            service,
            name,
            has_required_input: false,
            readonly: false,
        };
        let required = OperationFacts {
            has_required_input: true,
            ..facts
        };
        assert_eq!(
            classify(&facts, &deny),
            Class::Enumerate,
            "{service}/{name}"
        );
        assert_eq!(
            classify(&required, &deny),
            Class::Expand,
            "{service}/{name}"
        );
    });
    Ok(())
}
