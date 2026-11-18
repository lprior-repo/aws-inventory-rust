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
];
