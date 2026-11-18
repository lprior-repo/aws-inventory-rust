// Pinned model: .local/aws-models/models/docdb/service/2014-10-31/docdb-2014-10-31.json
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
            // Returns a list of DBClusterParameterGroup descriptions.
            queried(
                inventory(
                    "DescribeDBClusterParameterGroups",
                    "DBClusterParameterGroups",
                    arn("DBClusterParameterGroupArn")?,
                    token("Marker", "Marker")?,
                )?,
                &["DBClusterParameterGroupArn", "DBClusterParameterGroupName"],
            )?,
            // Returns information about cluster snapshots.
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
            // Returns information about provisioned Amazon DocumentDB clusters.
            clusters,
            // Returns a list of DBSubnetGroup descriptions.
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
            // Lists all the subscription descriptions for a customer account.
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
            // Returns information about Amazon DocumentDB global clusters.
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
        &[
            // DBInstances.PendingModifiedValues.MasterUserPassword is reachable and its pinned output docs do not guarantee redaction.
            ("DescribeDBInstances", DenyReason::Secret),
        ],
    )
}
