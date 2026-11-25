use super::support::{arn, inventory, queried, reviewed, scoped, tagged, token};
use crate::classes::DenyReason;
use crate::overlay::{FixedValue, MemberPath, NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut clusters = queried(
        inventory(
            "DescribeDBClusters",
            "DBClusters",
            arn("DBClusterArn")?,
            token("Marker", "Marker")?,
        )?,
        &[
            "DBClusterArn",
            "Status",
            "MasterUsername",
            "HostedZoneId",
            "KmsKeyId",
            "DbClusterResourceId",
            "CloneGroupId",
            "StorageType",
            "NetworkType",
        ],
    )?;
    clusters.1.fixed.insert(
        MemberPath::new("Filters")?,
        FixedValue::list([FixedValue::structure([
            ("Name", FixedValue::String("engine".to_owned())),
            ("Values", FixedValue::Strings(vec!["docdb".to_owned()])),
        ])?]),
    );
    reviewed(
        "docdb",
        vec![
            queried(
                inventory(
                    "DescribeDBClusterParameterGroups",
                    "DBClusterParameterGroups",
                    arn("DBClusterParameterGroupArn")?,
                    token("Marker", "Marker")?,
                )?,
                &["DBClusterParameterGroupArn", "DBClusterParameterGroupName"],
            )?,
            queried(
                inventory(
                    "DescribeDBClusterSnapshots",
                    "DBClusterSnapshots",
                    arn("DBClusterSnapshotArn")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "DBClusterSnapshotArn",
                    "Status",
                    "VpcId",
                    "MasterUsername",
                    "SnapshotType",
                    "KmsKeyId",
                    "StorageType",
                ],
            )?,
            clusters,
            queried(
                inventory(
                    "DescribeDBSubnetGroups",
                    "DBSubnetGroups",
                    arn("DBSubnetGroupArn")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "DBSubnetGroupArn",
                    "DBSubnetGroupName",
                    "VpcId",
                    "SubnetGroupStatus",
                ],
            )?,
            queried(
                inventory(
                    "DescribeEventSubscriptions",
                    "EventSubscriptionsList",
                    arn("EventSubscriptionArn")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "EventSubscriptionArn",
                    "CustomerAwsId",
                    "CustSubscriptionId",
                    "Status",
                    "SourceType",
                ],
            )?,
            queried(
                scoped(
                    tagged(
                        inventory(
                            "DescribeGlobalClusters",
                            "GlobalClusters",
                            arn("GlobalClusterArn")?,
                            token("Marker", "Marker")?,
                        )?,
                        "TagList",
                    )?,
                    Scope::Account,
                ),
                &[
                    "GlobalClusterArn",
                    "GlobalClusterResourceId",
                    "Status",
                    "DatabaseName",
                ],
            )?,
        ],
        &[
            super::s_docdb_decisions_0::ENTRIES,
            super::s_docdb_decisions_1::ENTRIES,
        ],
        &[("DescribeDBInstances", DenyReason::Secret)],
    )
}
