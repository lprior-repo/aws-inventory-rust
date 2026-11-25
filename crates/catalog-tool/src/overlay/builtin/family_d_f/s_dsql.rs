use super::support::{arn, bound, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dsql",
        vec![
            queried(
                inventory(
                    "ListClusters",
                    "clusters",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "identifier"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListStreams",
                        "streams",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("clusterIdentifier", "ListClusters", "identifier")],
                )?,
                &["arn", "clusterIdentifier", "streamIdentifier"],
            )?,
        ],
        &[super::s_dsql_decisions_0::ENTRIES],
        &[],
    )
}
