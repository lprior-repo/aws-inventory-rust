// Pinned model: .local/aws-models/models/dsql/service/2018-05-10/dsql-2018-05-10.json
use super::support::{arn, bound, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dsql",
        vec![
            // Retrieves information about a list of clusters.
            queried(
                inventory(
                    "ListClusters",
                    "clusters",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "identifier"],
            )?,
            // The cluster summary's identifier, not its ARN, satisfies the stream path label.
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
