// Pinned model: .local/aws-models/models/forecast/service/2018-06-26/forecast-2018-06-26.json
use super::support::{arn, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "forecast",
        vec![
        // Returns a list of dataset groups created using the CreateDatasetGroup operation.
        queried(inventory("ListDatasetGroups", "DatasetGroups", arn("DatasetGroupArn")?, token("NextToken", "NextToken")?)?, &["DatasetGroupArn", "DatasetGroupName"])?,
        // Returns a list of dataset import jobs created using the CreateDatasetImportJob operation.
        queried(inventory("ListDatasetImportJobs", "DatasetImportJobs", arn("DatasetImportJobArn")?, token("NextToken", "NextToken")?)?, &["DatasetImportJobArn", "DatasetImportJobName", "Status"])?,
        // Returns a list of datasets created using the CreateDataset operation.
        queried(inventory("ListDatasets", "Datasets", arn("DatasetArn")?, token("NextToken", "NextToken")?)?, &["DatasetArn", "DatasetName", "DatasetType"])?,
        // Returns a list of Explainability resources created using the CreateExplainability operation.
        queried(inventory("ListExplainabilities", "Explainabilities", arn("ExplainabilityArn")?, token("NextToken", "NextToken")?)?, &["ExplainabilityArn", "ExplainabilityName", "Status"])?,
        // Returns a list of Explainability exports created using the CreateExplainabilityExport operation.
        queried(inventory("ListExplainabilityExports", "ExplainabilityExports", arn("ExplainabilityExportArn")?, token("NextToken", "NextToken")?)?, &["ExplainabilityExportArn", "ExplainabilityExportName", "Status"])?,
        // Returns a list of forecast export jobs created using the CreateForecastExportJob operation.
        queried(inventory("ListForecastExportJobs", "ForecastExportJobs", arn("ForecastExportJobArn")?, token("NextToken", "NextToken")?)?, &["ForecastExportJobArn", "ForecastExportJobName", "Status"])?,
        // Returns a list of forecasts created using the CreateForecast operation.
        queried(inventory("ListForecasts", "Forecasts", arn("ForecastArn")?, token("NextToken", "NextToken")?)?, &["ForecastArn", "PredictorArn", "DatasetGroupArn", "ForecastName", "Status"])?,
        // Returns a list of monitors created with the CreateMonitor operation and CreateAutoPredictor operation.
        queried(inventory("ListMonitors", "Monitors", arn("MonitorArn")?, token("NextToken", "NextToken")?)?, &["MonitorArn", "MonitorName", "Status"])?,
        // Returns a list of predictor backtest export jobs created using the CreatePredictorBacktestExportJob operation.
        queried(inventory("ListPredictorBacktestExportJobs", "PredictorBacktestExportJobs", arn("PredictorBacktestExportJobArn")?, token("NextToken", "NextToken")?)?, &["PredictorBacktestExportJobArn", "PredictorBacktestExportJobName", "Status"])?,
        // Returns a list of predictors created using the CreateAutoPredictor or CreatePredictor operations.
        queried(inventory("ListPredictors", "Predictors", arn("PredictorArn")?, token("NextToken", "NextToken")?)?, &["PredictorArn", "DatasetGroupArn", "PredictorName", "Status"])?,
        // Returns a list of what-if analyses created using the CreateWhatIfAnalysis operation.
        queried(inventory("ListWhatIfAnalyses", "WhatIfAnalyses", arn("WhatIfAnalysisArn")?, token("NextToken", "NextToken")?)?, &["WhatIfAnalysisArn", "ForecastArn", "WhatIfAnalysisName", "Status"])?,
        // Returns a list of what-if forecast exports created using the CreateWhatIfForecastExport operation.
        queried(inventory("ListWhatIfForecastExports", "WhatIfForecastExports", arn("WhatIfForecastExportArn")?, token("NextToken", "NextToken")?)?, &["WhatIfForecastExportArn", "WhatIfForecastExportName", "Status"])?,
        // Returns a list of what-if forecasts created using the CreateWhatIfForecast operation.
        queried(inventory("ListWhatIfForecasts", "WhatIfForecasts", arn("WhatIfForecastArn")?, token("NextToken", "NextToken")?)?, &["WhatIfForecastArn", "WhatIfAnalysisArn", "WhatIfForecastName", "Status"])?,
        ],
        &[super::s_forecast_decisions_0::ENTRIES, super::s_forecast_decisions_1::ENTRIES],
        &[

        ],
    )
}
