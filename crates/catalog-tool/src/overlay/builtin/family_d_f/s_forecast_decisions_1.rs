pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "DescribeForecastExportJob",
        "Requires ForecastExportJobArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes a forecast export job created using the CreateForecastExportJob operation.",
    ),
    (
        "DescribeMonitor",
        "Requires MonitorArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes a monitor resource.",
    ),
    (
        "DescribePredictor",
        "Requires PredictorArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation is only valid for legacy predictors created with CreatePredictor.",
    ),
    (
        "DescribePredictorBacktestExportJob",
        "Requires PredictorBacktestExportJobArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes a predictor backtest export job created using the CreatePredictorBacktestExportJob operation.",
    ),
    (
        "DescribeWhatIfAnalysis",
        "Requires WhatIfAnalysisArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the what-if analysis created using the CreateWhatIfAnalysis operation.",
    ),
    (
        "DescribeWhatIfForecast",
        "Requires WhatIfForecastArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the what-if forecast created using the CreateWhatIfForecast operation.",
    ),
    (
        "DescribeWhatIfForecastExport",
        "Requires WhatIfForecastExportArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the what-if forecast export created using the CreateWhatIfForecastExport operation.",
    ),
    (
        "GetAccuracyMetrics",
        "Requires PredictorArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Provides metrics on the accuracy of the models that were trained by the CreatePredictor operation.",
    ),
    (
        "ListMonitorEvaluations",
        "Requires MonitorArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns a list of the monitoring evaluation results and predictor events collected by the monitor resource during different windows of time.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the tags for an Amazon Forecast resource.",
    ),
    (
        "ResumeResource",
        "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Resumes a stopped monitor resource.",
    ),
    (
        "StopResource",
        "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops a resource.",
    ),
    (
        "TagResource",
        "Requires ResourceArn, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates the specified tags to a resource with the specified resourceArn .",
    ),
    (
        "UntagResource",
        "Requires ResourceArn, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified tags from a resource.",
    ),
    (
        "UpdateDatasetGroup",
        "Requires DatasetGroupArn, DatasetArns; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Replaces the datasets in a dataset group with the specified datasets.",
    ),
];
