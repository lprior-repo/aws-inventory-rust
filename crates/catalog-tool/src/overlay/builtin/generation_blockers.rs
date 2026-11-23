use crate::overlay::{GenerationBlocker, OverlayClass, ServiceOverlay};

// These reviewed defers describe missing request bindings, not a safety veto.
// Their pinned models and advisory service audits were compared before migration.
// The marker authorizes no call: generation must still derive and validate every field.
const REQUIRED_BINDINGS: &[(&str, &[&str])] = &[
    ("account-access", &["GetApplication"]),
    (
        "app-mesh",
        &["DescribeVirtualRouter", "DescribeVirtualService"],
    ),
    (
        "cleanroomsml",
        &["GetAudienceModel", "GetConfiguredModelAlgorithm"],
    ),
    (
        "comprehend",
        &["DescribeDataset", "DescribeEndpoint", "DescribeFlywheel"],
    ),
    (
        "datasync",
        &[
            "DescribeLocationAzureBlob",
            "DescribeLocationFsxLustre",
            "DescribeLocationFsxWindows",
            "DescribeLocationS3",
        ],
    ),
    ("dsql", &["GetCluster"]),
    (
        "forecast",
        &[
            "DescribeDatasetGroup",
            "DescribeForecast",
            "DescribeWhatIfForecastExport",
        ],
    ),
    ("mailmanager", &["GetArchive"]),
    (
        "mediaconnect",
        &[
            "DescribeBridge",
            "DescribeGateway",
            "DescribeGatewayInstance",
            "DescribeReservation",
            "GetDestinationEndpoint",
            "GetRouterInput",
            "GetRouterNetworkInterface",
            "GetRouterOutput",
            "GetSourceEndpoint",
        ],
    ),
    ("mediapackagev2", &["GetChannelGroup"]),
    (
        "omics",
        &[
            "GetAnnotationStore",
            "GetConfiguration",
            "GetRunCache",
            "GetRunGroup",
            "GetVariantStore",
        ],
    ),
    ("qbusiness", &["GetRetriever"]),
    ("rekognition", &["DescribeProjectVersions"]),
    ("rum", &["GetAppMonitor"]),
];

pub(super) fn apply(policies: &mut [ServiceOverlay]) {
    for policy in policies {
        let operations = REQUIRED_BINDINGS
            .iter()
            .find(|(service, _)| *service == policy.service());
        if let Some((_, operations)) = operations {
            for operation in *operations {
                if let Some(entry) = policy.operations.get_mut(*operation)
                    && entry.class == Some(OverlayClass::Defer)
                {
                    entry.generation_blocker = Some(GenerationBlocker::RequiredBindings);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
