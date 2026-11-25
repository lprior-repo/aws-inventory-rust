use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CancelGremlinQuery",
            "CancelIndexRebuild",
            "CancelLoaderJob",
            "CancelMLDataProcessingJob",
            "CancelMLModelTrainingJob",
            "CancelMLModelTransformJob",
            "CancelOpenCypherQuery",
            "CreateMLEndpoint",
            "DeleteMLEndpoint",
            "DeletePropertygraphStatistics",
            "DeleteSparqlStatistics",
            "ExecuteFastReset",
            "ExecuteGremlinExplainQuery",
            "ExecuteGremlinProfileQuery",
            "ExecuteGremlinQuery",
            "ExecuteOpenCypherExplainQuery",
            "ExecuteOpenCypherQuery",
            "ManagePropertygraphStatistics",
            "ManageSparqlStatistics",
            "StartIndexRebuild",
            "StartLoaderJob",
            "StartMLDataProcessingJob",
            "StartMLModelTrainingJob",
            "StartMLModelTransformJob",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetEngineStatus",
            "GetGremlinQueryStatus",
            "GetIndexRebuildStatus",
            "GetLoaderJobStatus",
            "GetMLDataProcessingJob",
            "GetMLEndpoint",
            "GetMLModelTrainingJob",
            "GetMLModelTransformJob",
            "GetOpenCypherQueryStatus",
            "GetPropertygraphStatistics",
            "GetPropertygraphSummary",
            "GetRDFGraphSummary",
            "GetSparqlStatistics",
            "GetStorageInfo",
            "ListGremlinQueries",
            "ListIndexRebuilds",
            "ListLoaderJobs",
            "ListMLDataProcessingJobs",
            "ListMLEndpoints",
            "ListMLModelTrainingJobs",
            "ListMLModelTransformJobs",
            "ListOpenCypherQueries",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Database data plane requires a discovered per-cluster endpoint and query/job context; regional control-plane endpoints cannot execute these operations safely.",
        ),
    },
    Group {
        operations: &["GetPropertygraphStream", "GetSparqlStream"],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("neptunedata", GROUPS, &[])
}
