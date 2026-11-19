use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventBridgeV2EventSource",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventBridgeEventBus",
        reason: "Distinct API-managed EventBridgeEventBus resource identified by @Arn in events DescribeEventBus EventBus. It is not the containing service, parent resource or operation execution itself; preserve this explicit resource identity rather than inventing a CFN resource type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventBridgeEventSource",
        reason: "Distinct API-managed EventBridgeEventSource resource identified by @Arn in events DescribeEventSource EventSource. It is not the containing service, parent resource or operation execution itself; preserve this explicit resource identity rather than inventing a CFN resource type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventBridgeEventRule",
        reason: "Distinct API-managed EventBridgeEventRule resource identified by @Arn in events DescribeEventRule Rule. It is not the containing service, parent resource or operation execution itself; preserve this explicit resource identity rather than inventing a CFN resource type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventBridgeEndpoint",
        reason: "Distinct API-managed EventBridgeEndpoint resource identified by @Arn in events DescribeEndpoint Endpoint. It is not the containing service, parent resource or operation execution itself; preserve this explicit resource identity rather than inventing a CFN resource type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::EventBridgeV2Subscriber",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
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
        kind: "AWS::ApiOnly::FISExperiment",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastDatasetImportJob",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastExplainability",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastExplainabilityExport",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastForecast",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastForecastExportJob",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastMonitor",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastPredictor",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastPredictorBacktestExportJob",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastWhatIfAnalysis",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastWhatIfForecast",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ForecastWhatIfForecastExport",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::FraudDetectorModel",
        reason: "Trainable Fraud Detector model definition is not a SageMaker model or imported external endpoint; no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::FSxBackup",
        reason: "Pinned API resource has no equivalent CFN resource or is an unspecialized polymorphic summary; do not invent a CFN kind.",
    },
];
