use super::support::{
    arn, excluded, inventory, key, queried, reviewed, scoped, tagged, token,
};
use crate::overlay::{
    ContextSource, FixedValue, MemberPath, NameError, RowOperand, RowPredicate, RowValueType,
    Scope, ServiceOverlay,
};

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
    let mut instances = excluded(
        queried(
            inventory(
                "DescribeDBInstances",
                "DBInstances",
                key(&["DbiResourceId"])?,
                token("Marker", "Marker")?,
            )?,
            &[
                "DBInstanceArn",
                "DBInstanceIdentifier",
                "DBInstanceClass",
                "DBInstanceStatus",
                "DBSubnetGroup",
                "DbiResourceId",
                "Endpoint",
                "Engine",
                "EngineVersion",
                "KmsKeyId",
                "PerformanceInsightsEnabled",
                "PerformanceInsightsKMSKeyId",
                "PromotionTier",
                "StorageEncrypted",
                "VpcSecurityGroups",
            ],
        )?,
        &["DBInstances.PendingModifiedValues.MasterUserPassword"],
    )?;
    // Unlike the cluster operation, the pinned instance API documents no engine filter.
    instances.1.row_policy.owned = Some(RowPredicate::StringIn {
        operand: RowOperand {
            source: ContextSource::SelectedItem,
            path: MemberPath::selected("Engine")?,
            value_type: RowValueType::String,
        },
        values: vec!["docdb".to_owned()],
    });
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
            instances,
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
        &[],
    )
}
