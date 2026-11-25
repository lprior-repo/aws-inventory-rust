use super::support::{arn, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "forecast",
        vec![
            queried(
                inventory(
                    "ListDatasetGroups",
                    "DatasetGroups",
                    arn("DatasetGroupArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["DatasetGroupArn", "DatasetGroupName"],
            )?,
            queried(
                inventory(
                    "ListDatasetImportJobs",
                    "DatasetImportJobs",
                    arn("DatasetImportJobArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["DatasetImportJobArn", "DatasetImportJobName", "Status"],
            )?,
            queried(
                inventory(
                    "ListDatasets",
                    "Datasets",
                    arn("DatasetArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["DatasetArn", "DatasetName", "DatasetType"],
            )?,
            queried(
                inventory(
                    "ListExplainabilities",
                    "Explainabilities",
                    arn("ExplainabilityArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ExplainabilityArn", "ExplainabilityName", "Status"],
            )?,
            queried(
                inventory(
                    "ListExplainabilityExports",
                    "ExplainabilityExports",
                    arn("ExplainabilityExportArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ExplainabilityExportArn",
                    "ExplainabilityExportName",
                    "Status",
                ],
            )?,
            queried(
                inventory(
                    "ListForecastExportJobs",
                    "ForecastExportJobs",
                    arn("ForecastExportJobArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ForecastExportJobArn", "ForecastExportJobName", "Status"],
            )?,
            queried(
                inventory(
                    "ListForecasts",
                    "Forecasts",
                    arn("ForecastArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ForecastArn",
                    "PredictorArn",
                    "DatasetGroupArn",
                    "ForecastName",
                    "Status",
                ],
            )?,
            queried(
                inventory(
                    "ListMonitors",
                    "Monitors",
                    arn("MonitorArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["MonitorArn", "MonitorName", "Status"],
            )?,
            queried(
                inventory(
                    "ListPredictorBacktestExportJobs",
                    "PredictorBacktestExportJobs",
                    arn("PredictorBacktestExportJobArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "PredictorBacktestExportJobArn",
                    "PredictorBacktestExportJobName",
                    "Status",
                ],
            )?,
            queried(
                inventory(
                    "ListPredictors",
                    "Predictors",
                    arn("PredictorArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["PredictorArn", "DatasetGroupArn", "PredictorName", "Status"],
            )?,
            queried(
                inventory(
                    "ListWhatIfAnalyses",
                    "WhatIfAnalyses",
                    arn("WhatIfAnalysisArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "WhatIfAnalysisArn",
                    "ForecastArn",
                    "WhatIfAnalysisName",
                    "Status",
                ],
            )?,
            queried(
                inventory(
                    "ListWhatIfForecastExports",
                    "WhatIfForecastExports",
                    arn("WhatIfForecastExportArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "WhatIfForecastExportArn",
                    "WhatIfForecastExportName",
                    "Status",
                ],
            )?,
            queried(
                inventory(
                    "ListWhatIfForecasts",
                    "WhatIfForecasts",
                    arn("WhatIfForecastArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "WhatIfForecastArn",
                    "WhatIfAnalysisArn",
                    "WhatIfForecastName",
                    "Status",
                ],
            )?,
        ],
        &[
            super::s_forecast_decisions_0::ENTRIES,
            super::s_forecast_decisions_1::ENTRIES,
        ],
        &[],
    )
}
