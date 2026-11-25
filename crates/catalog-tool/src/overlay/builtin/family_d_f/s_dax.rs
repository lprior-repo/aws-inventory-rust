use super::support::{arn, inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dax",
        vec![
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
            queried(
                inventory(
                    "DescribeParameterGroups",
                    "ParameterGroups",
                    key(&["ParameterGroupName"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ParameterGroupName"],
            )?,
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
