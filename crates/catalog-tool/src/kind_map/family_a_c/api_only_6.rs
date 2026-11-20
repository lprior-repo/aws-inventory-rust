use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ApiGatewayV2Portal",
        reason: "API Gateway V2 portal has its own PortalArn; the pinned CFN registry contains PortalProduct, not the distinct portal resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DiscoveryContinuousExport",
        reason: "Application Discovery continuous export metadata identifies an existing export configuration, not an application or server; no pinned continuous-export type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventsEventSource",
        reason: "EventBridge partner event-source metadata keyed by Arn is a source registration rather than an EventBus; no exact pinned CFN event-source type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventsReplay",
        reason: "Retained EventBridge archive replay execution metadata keyed by ReplayName is not an archive or event bus definition; no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::LogsExportTask",
        reason: "CloudWatch Logs export-task metadata is a retained export execution, not log events or a log-group definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::LogsImportTask",
        reason: "CloudWatch Logs import-task metadata is a retained ingestion execution, not log events or a log-group definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::LogsLookupTable",
        reason: "CloudWatch Logs lookup-table metadata keyed by lookupTableArn is not log event content or a log-group resource; no matching pinned CFN lookup-table type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BatchServiceJob",
        reason: "AWS Batch service-job summaries describe service-managed job executions with their own serviceJobId, not batch job definitions or the regular Batch job executions in BatchJob; no exact pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ChimeSDKVoicePhoneNumber",
        reason: "Chime SDK Voice phone-number metadata keyed by PhoneNumberArn is a voice-service inventory view, not the Chime or SMS voice PhoneNumber resource; no pinned chime-sdk-voice CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ChimeSDKVoiceProfileDomain",
        reason: "Chime SDK Voice profile-domain summaries keyed by VoiceProfileDomainArn are not a pinned CFN resource type; no matching chime-sdk-voice CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudFormationRegistryType",
        reason: "CloudFormation registry type summaries cover resource, module and hook extensions keyed by TypeArn; no single pinned CFN type represents every registry extension kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudFormationRegistryTypeVersion",
        reason: "CloudFormation registry type-version summaries cover resource, module and hook extension versions keyed by Arn; the pinned AWS::CloudFormation::ResourceVersion covers resources only.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::PersonalizeSolutionVersion",
        reason: "Personalize solution-version summaries are trained model versions keyed by solutionVersionArn, not the Solution definition; no pinned CFN solution-version type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ConfigRuleEvaluationStatus",
        reason: "AWS Config rule evaluation status is runtime evaluation state keyed by ConfigRuleArn, not the ConfigRule definition; no pinned CFN type for evaluation status.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ConfigurationRecorderStatus",
        reason: "AWS Config configuration-recorder status is recording state keyed by recorder ARN, not the ConfigurationRecorder definition; no pinned CFN type for recorder status.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ConformancePackStatus",
        reason: "AWS Config conformance-pack status is compliance-run state keyed by ConformancePackArn, not the ConformancePack definition; no pinned CFN type for pack status.",
    },
];
