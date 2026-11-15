// Pinned model: .local/aws-models/models/docdb-elastic/service/2022-11-28/docdb-elastic-2022-11-28.json
use super::support::{arn, bound, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "docdb-elastic",
        vec![
        // Returns information about provisioned Amazon DocumentDB elastic clusters.
        queried(inventory("ListClusters", "clusters", arn("clusterArn")?, token("nextToken", "nextToken")?)?, &["clusterArn", "clusterName", "status"])?,
        // Returns information about snapshots for a specified elastic cluster.
        queried(bound(inventory("ListClusterSnapshots", "snapshots", arn("snapshotArn")?, token("nextToken", "nextToken")?)?, &[("clusterArn", "ListClusters", "clusterArn")])?, &["snapshotArn", "snapshotName", "status"])?,
        ],
        &[super::s_docdb_elastic_decisions_0::ENTRIES],
        &[

        ],
    )
}
