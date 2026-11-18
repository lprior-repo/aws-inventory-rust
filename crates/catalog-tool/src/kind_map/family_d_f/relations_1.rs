use crate::kind_map::{RelationMapping, RelationPath, RelationSource, RelationTarget};

pub(super) const ENTRIES: &[RelationMapping<'static>] = &[
    RelationMapping {
        source_kind: "AWS::ApiOnly::ForecastForecast",
        source: RelationSource::SelectedOutput(RelationPath::Fields("DatasetGroupArn")),
        target_kind: "AWS::Forecast::DatasetGroup",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("DatasetGroupArn")),
        relation: "USES_DATASET_GROUP",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::ApiOnly::ForecastPredictor",
        source: RelationSource::SelectedOutput(RelationPath::Fields("DatasetGroupArn")),
        target_kind: "AWS::Forecast::DatasetGroup",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("DatasetGroupArn")),
        relation: "USES_DATASET_GROUP",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::ApiOnly::ForecastWhatIfAnalysis",
        source: RelationSource::SelectedOutput(RelationPath::Fields("ForecastArn")),
        target_kind: "AWS::ApiOnly::ForecastForecast",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("ForecastArn")),
        relation: "ANALYZES_FORECAST",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::ApiOnly::ForecastWhatIfForecast",
        source: RelationSource::SelectedOutput(RelationPath::Fields("WhatIfAnalysisArn")),
        target_kind: "AWS::ApiOnly::ForecastWhatIfAnalysis",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("WhatIfAnalysisArn")),
        relation: "USES_ANALYSIS",
        scope: "account_region",
        guard: None,
    },
];
