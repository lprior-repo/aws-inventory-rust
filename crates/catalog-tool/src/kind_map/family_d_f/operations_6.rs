use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "eventbridgev2",
        operation: "ListEventBuses",
        kind: "AWS::ApiOnly::EventBridgeV2EventBus",
    },
    OperationMapping {
        service: "eventbridgev2",
        operation: "ListEventSources",
        kind: "AWS::ApiOnly::EventBridgeV2EventSource",
    },
    OperationMapping {
        service: "eventbridgev2",
        operation: "ListSubscribers",
        kind: "AWS::ApiOnly::EventBridgeV2Subscriber",
    },
    OperationMapping {
        service: "evs",
        operation: "ListEnvironments",
        kind: "AWS::EVS::Environment",
    },
    OperationMapping {
        service: "fis",
        operation: "ListExperimentTemplates",
        kind: "AWS::FIS::ExperimentTemplate",
    },
    OperationMapping {
        service: "fis",
        operation: "ListExperiments",
        kind: "AWS::ApiOnly::FISExperiment",
    },
    OperationMapping {
        service: "fms",
        operation: "ListPolicies",
        kind: "AWS::FMS::Policy",
    },
    OperationMapping {
        service: "fms",
        operation: "ListResourceSets",
        kind: "AWS::FMS::ResourceSet",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListDatasetGroups",
        kind: "AWS::Forecast::DatasetGroup",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListDatasetImportJobs",
        kind: "AWS::ApiOnly::ForecastDatasetImportJob",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListDatasets",
        kind: "AWS::Forecast::Dataset",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListExplainabilities",
        kind: "AWS::ApiOnly::ForecastExplainability",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListExplainabilityExports",
        kind: "AWS::ApiOnly::ForecastExplainabilityExport",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListForecastExportJobs",
        kind: "AWS::ApiOnly::ForecastForecastExportJob",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListForecasts",
        kind: "AWS::ApiOnly::ForecastForecast",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListMonitors",
        kind: "AWS::ApiOnly::ForecastMonitor",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListPredictorBacktestExportJobs",
        kind: "AWS::ApiOnly::ForecastPredictorBacktestExportJob",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListPredictors",
        kind: "AWS::ApiOnly::ForecastPredictor",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListWhatIfAnalyses",
        kind: "AWS::ApiOnly::ForecastWhatIfAnalysis",
    },
    OperationMapping {
        service: "forecast",
        operation: "ListWhatIfForecastExports",
        kind: "AWS::ApiOnly::ForecastWhatIfForecastExport",
    },
];
