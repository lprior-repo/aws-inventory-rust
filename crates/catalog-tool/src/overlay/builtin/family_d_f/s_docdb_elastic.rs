use super::support::{arn, bound, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "docdb-elastic",
        vec![
            queried(
                inventory(
                    "ListClusters",
                    "clusters",
                    arn("clusterArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["clusterArn", "clusterName", "status"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListClusterSnapshots",
                        "snapshots",
                        arn("snapshotArn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("clusterArn", "ListClusters", "clusterArn")],
                )?,
                &["snapshotArn", "snapshotName", "status"],
            )?,
        ],
        &[super::s_docdb_elastic_decisions_0::ENTRIES],
        &[],
    )
}
