// Pinned model: .local/aws-models/models/dax/service/2017-04-19/dax-2017-04-19.json
use super::support::{arn, inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dax",
        vec![
            // Returns information about all provisioned DAX clusters if no cluster identifier is specified, or about a specific DAX cluster if a cluster identifier is supplied.
            queried(
                inventory(
                    "DescribeClusters",
                    "Clusters",
                    arn("ClusterArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ClusterArn",
                    "IamRoleArn",
                    "ClusterName",
                    "NodeType",
                    "Status",
                    "ClusterEndpointEncryptionType",
                    "NetworkType",
                ],
            )?,
            // Returns a list of parameter group descriptions.
            queried(
                inventory(
                    "DescribeParameterGroups",
                    "ParameterGroups",
                    key(&["ParameterGroupName"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ParameterGroupName"],
            )?,
            // Returns a list of subnet group descriptions.
            queried(
                inventory(
                    "DescribeSubnetGroups",
                    "SubnetGroups",
                    key(&["SubnetGroupName"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["SubnetGroupName", "VpcId"],
            )?,
        ],
        &[super::s_dax_decisions_0::ENTRIES],
        &[],
    )
}
